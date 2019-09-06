use rustls::{
    ServerSession,
    ServerConfig,
};
use std::net::{
    TcpStream,
    TcpListener
};
use std::sync::Arc;

pub struct SessionManager {
    hostname: String,
    connections: Vec<Connection>,
    next_id: usize,
    config: Arc<ServerConfig>,
}

impl SessionManager {
    pub fn new(server: &TcpListener, config: Arc<ServerConfig>) -> Self {
        Self {
            hostname: server.local_addr().unwrap().ip().to_string(),
            connections: vec![],
            next_id: 2,
            config
        }
    }

    pub fn add_session(&mut self, conn: Connection) {
        let session = ServerSession::new(&self.config);
        let token = self.next_id;
        self.next_id += 1;
        self.connections.push(conn);
    }

    pub fn handle_incoming(&self, stream: TcpStream) {

    }
}

pub struct Connection {
    socket: TcpStream,
    session: ServerSession,
    token: usize
}

impl Connection {
    pub fn new_session(socket: TcpStream, session: ServerSession, token: usize) -> Self {
        Self {
            socket,
            session,
            token
        }
    }
}