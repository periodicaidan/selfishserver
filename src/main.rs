#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde;
extern crate ron;
extern crate rustls;
extern crate webpki_roots;

use std::net::{
    TcpListener,
    TcpStream
};
use std::io::{
    prelude::*,
    BufReader,
};
use std::collections::HashMap;
use std::time::Duration;
use std::fs::File;
use std::sync::Arc;

use rustls::{
    Session,
    ServerSession,
    ServerConfig,
    StreamOwned,
    AllowAnyAnonymousOrAuthenticatedClient,
    KeyLogFile,
    RootCertStore,
    Certificate,
    PrivateKey
};

mod thread_pool;
#[macro_use] mod http;
mod security;

use crate::thread_pool::ThreadPool;
use crate::http::{
    HttpRequest,
    HttpResponse,
    HttpdConfig,
};
use std::borrow::Borrow;

enum Stream {
    Insecure(TcpStream),
    Secure(StreamOwned<ServerSession, TcpStream>)
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Stream::Insecure(s) => s.read(buf),
            Stream::Secure(s) => s.read(buf),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Stream::Insecure(s) => s.write(buf),
            Stream::Secure(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Stream::Insecure(s) => s.flush(),
            Stream::Secure(s) => s.flush(),
        }
    }
}

fn main() {
    let config = HttpdConfig::default();
    println!("{:#?}", config);
    let listener = TcpListener::bind(&format!("{}:{}", config.host, config.port)).unwrap();
    println!("Starting server at {}", listener.local_addr().unwrap());
    println!("Mounting on {}", std::env::current_dir().unwrap().display());
    let pool = ThreadPool::new(config.threads.unwrap_or(1));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let cfg = config.clone();
        pool.execute(move || handle_connection(stream, cfg));
    }
}

fn handle_connection(mut socket: TcpStream, config: HttpdConfig) {
    // A buffer for holding the HTTP request
    let mut buffer = [0; 1048];

    // Check the config file for whether or not this request should be served over TLS
    let is_secure = match config.security.borrow() {
        Some(sec) => sec.use_tls,
        None => false
    };

    // Wrap the socket in a secure stream if it should be served over TLS
    let mut stream = if is_secure {
        // Set up the TLS configurations
        // TODO: This is slow and I should manage sessions rather than remake them each time
        let mut tls_cfg = ServerConfig::new(AllowAnyAnonymousOrAuthenticatedClient::new(RootCertStore::empty()));
        tls_cfg.key_log = Arc::new(KeyLogFile::new());
        let certs = load_certs(
            &config.security.clone().unwrap()
                .cert_file.unwrap_or(from_cargo!("cert.pem").to_string())
        );
        let key = load_key(
            &config.security.clone().unwrap()
                .key_file.unwrap_or(from_cargo!("key.pem").to_string())
        );
        tls_cfg.set_single_cert(certs, key).unwrap();

        let mut session = ServerSession::new(&Arc::new(tls_cfg));
        let mut s = StreamOwned::new(session, socket);

        Stream::Secure(s)
    } else {
        Stream::Insecure(socket)
    };

    // Read the request from the peer into the buffer
    stream.read(&mut buffer);

    let request_line = String::from_utf8_lossy(&mut buffer);
    println!("{}", request_line);
    let request = HttpRequest::new(&request_line);

    // Make the response
    let response = HttpResponse::new(
        &request,
        if config.allowed_methods.contains(&request.method.to_string()) {
            200
        } else {
            405
        }
    );

    let mut response_string = response.to_vectored_bytes();

    // Send the response to the peer
    stream.write(&response_string).unwrap();
    stream.flush().unwrap();
}

fn load_certs(filename: &str) -> Vec<Certificate> {
    let certfile = File::open(filename).unwrap();
    let mut reader = BufReader::new(certfile);
    rustls::internal::pemfile::certs(&mut reader).unwrap()
}

fn load_key(filename: &str) -> PrivateKey {
    let rsa_key = {
        let keyfile = File::open(filename).unwrap();
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::rsa_private_keys(&mut reader).unwrap()
    };

    let pkcs8_key = {
        let keyfile = File::open(filename).unwrap();
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::pkcs8_private_keys(&mut reader).unwrap()
    };

    if pkcs8_key.len() > 0 {
        return pkcs8_key[0].clone();
    } else {
        assert!(rsa_key.len() > 0);
        return rsa_key[0].clone()
    }
}