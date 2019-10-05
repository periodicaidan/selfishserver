use std::collections::HashMap;
use std::fs;
use std::fmt;
use super::utils::*;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::env::current_dir;
use std::str::FromStr;
use colored::*;
use chrono::prelude::*;

/*** REQUEST/RESPONSE LOGIC ***/

#[derive(Debug)]
pub struct HttpRequest<'r> {
    pub method: &'r str,
    pub uri: &'r str,
    pub version: &'r str,
    pub headers: HashMap<&'r str, &'r str>,
    pub body: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<'r> HttpRequest<'r> {
    pub fn new(request_line: &'r str) -> Self {
        let parts: Vec<&str> = request_line.split(char::is_whitespace).collect();
        let lines: Vec<&str> = request_line.split("\r\n").collect();
        let mut headers = HashMap::new();
        for line in lines {
            let kvpair: Vec<&str> = line.split(":").collect();
            if kvpair.len() != 2 {
                continue;
            }
            headers.insert(kvpair[0].trim(), kvpair[1].trim());
        }

        Self {
            method: parts[0],
            uri: parts[1],
            version: parts[2],
            headers,
            body: None,
            timestamp: Utc::now(),
        }
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        string.push_str(&format!("{} {} {}\r\n", self.method, self.uri, self.version));
        for (k, v) in self.headers.iter() {
            string.push_str(&format!("{}: {}\r\n", k, v));
        }
        if let Some(b) = &self.body {
            string.push_str(&format!("\r\n{}", &b));
        }

        string
    }

    pub fn status_string(&self) -> String {
        let color = match self.method {
            "GET" => "green",
            "HEAD" => "cyan",
            _ => "white"
        };

        format!("[{} | {} {}]", self.timestamp.to_rfc2822().italic(), self.method.color(color), self.uri)
    }
}

impl fmt::Display for HttpRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_string())
    }
}

pub struct HttpResponse<'r> {
    pub version: &'r str,
    pub status: u32,
    pub reason: String,
    pub headers: HashMap<&'r str, String>,
    pub body: Option<Vec<u8>>,
    pub uri: &'r str,
    pub timestamp: DateTime<Utc>,
}

impl<'r> HttpResponse<'r> {
    pub fn new(request: &'r HttpRequest, path: &Path, status: u32) -> Self {
        let reason = HTTP_RESPONSE_STATUSES.get(&status).unwrap();

        let mut headers = HashMap::new();
        let content_type = ContentType::parse_from_filename(path);
        headers.insert("Content-Type", content_type.to_string());

        let body = match request.method {
            "HEAD" => None,
            "GET" => Some(fs::read(path).unwrap()),
            _ => None
        };

        Self {
            version: request.version,
            status,
            reason: reason.to_string(),
            headers,
            body,
            uri: request.uri,
            timestamp: Utc::now(),
        }
    }

    /// Creates a 404 response
    pub fn not_found(request: &'r HttpRequest) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type", "text/html; charset=utf-8".to_string());

        // For some reason trying to access this causes a panic because it returns None (on MacOS)
        let status_text = match HTTP_RESPONSE_STATUSES.get(&404) {
            Some(s) => s.to_string(),
            None => String::from("Not Found")
        };

        Self {
            version: request.version,
            status: 404,
            reason: status_text,
            headers,
            body: Some(fs::read(from_cargo!("src/error_pages/404.html")).unwrap()),
            uri: request.uri,
            timestamp: Utc::now(),
        }
    }

    pub fn with_header(mut self, k: &'r str, v: &'r str) -> Self {
        self.headers.entry(k)
            .and_modify(|e| { *e = v.to_string(); })
            .or_insert(v.to_string());

        self
    }

    pub fn to_vectored_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        let mut string = String::new();

        bytes.extend_from_slice({
            string.push_str(&format!("{} {} {}\r\n", self.version, self.status, &self.reason));
            for (k, v) in self.headers.iter() {
                string.push_str(&format!("{}: {}\r\n", k, v));
            }
            string.push_str("\r\n");

            string.as_bytes()
        });

        if let Some(body) = &self.body {
            bytes.extend_from_slice(&body);
        }

        bytes
    }

    /// Returns a string with the response line and the headers, without the body since that's not
    /// UTF-8 safe
    pub fn get_header_string(&self) -> String {
        let mut head = format!("{} {} {}\r\n", &self.version, self.status, &self.reason);
        for (k, v) in self.headers.iter() {
            head.push_str(&format!("{}: {}\r\n", k, v));
        }
        head.push_str("\r\n");

        head
    }

    pub fn status_string(&self) -> String {
        let status_color = match self.status {
            100...199 => "cyan",
            200...299 => "green",
            300...399 => "yellow",
            400...499 => "red",
            500...599 => "magenta",
            _ => "white",
        };

        format!("[{} | {} {} {}]", &self.timestamp.to_rfc2822().italic(), &format!("{}", self.status).color(status_color).reversed(), &self.reason.color(status_color), &self.uri)
    }
}

impl fmt::Debug for HttpResponse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.get_header_string())
    }
}

/*** CONTENT TYPES ***/

pub struct ContentType<'c> {
    pub media_type: MediaType<'c>,
    pub parameter: Option<(&'c str, &'c str)>,
}

impl<'c> ContentType<'c> {
    pub fn to_string(&self) -> String {
        if let Some((key, val)) = &self.parameter {
            format!("{};{}={}", &self.media_type.to_string(), &key, &val)
        } else {
            self.media_type.to_string()
        }
    }

    pub fn parse_from_filename(file: &Path) -> Self {
        use self::MediaType::*;
        if file.is_dir() {
            return Self {
                media_type: Text("html"),
                parameter: Some(("charset", "utf-8"))
            };
        }

        let media_type = match file.extension() {
            Some(ext) => {
                let extension = ext.to_str().unwrap();
                match extension {
                    "html" | "htm" => Text("html"),
                    "js" => Text("javascript"),
                    "css" => Text("css"),
                    "ico" | "cur" => Image("x-icon"),
                    "jpg" | "jpeg" | "jfif" | "pjpeg" | "pjp" => Image("jpeg"),
                    "png" => Image("png"),
                    "svg" => Image("svg+xml"),
                    "json" => Application("json"),
                    _ => Application("octet-stream"),
                }
            },
            None => Text("plain")
        };

        let parameter = match media_type {
            Text("html") => Some(("charset", "utf-8")),
            _ => None
        };

        Self {
            media_type,
            parameter
        }
    }
}

pub enum MediaType<'m> {
    Application(&'m str),
    Audio(&'m str),
    Example(&'m str),
    Font(&'m str),
    Image(&'m str),
    Model(&'m str),
    Text(&'m str),
    Video(&'m str),
}

impl<'m> MediaType<'m> {
    pub fn to_string(&self) -> String {
        match self {
            MediaType::Application(s) => format!("application/{}", &s),
            MediaType::Audio(s) => format!("audio/{}", &s),
            MediaType::Example(s) => format!("example/{}", &s),
            MediaType::Font(s) => format!("font/{}", &s),
            MediaType::Image(s) => format!("image/{}", &s),
            MediaType::Model(s) => format!("model/{}", &s),
            MediaType::Text(s) => format!("text/{}", &s),
            MediaType::Video(s) => format!("video/{}", &s),
        }
    }
}

/*** CONFIG FILE ***/

/// Configuration details for the HTTP daemon (i.e., the server)
///
/// This is what the httpd.ron file resolves to
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpdConfig {
    pub host: String,
    pub port: u32,
    pub allowed_methods: Vec<String>,
    pub threads: Option<usize>,
    pub owner: Option<ServerOwner>,
    pub security: Option<ServerSecurity>
}

impl HttpdConfig {
    pub fn new(config_file: &str) -> Self {
        ron::de::from_str(config_file).unwrap()
    }
}

impl Default for HttpdConfig {
    fn default() -> Self {
        Self::new(&fs::read_to_string(from_cargo!("src/httpd.ron")).unwrap())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerOwner {
    pub name: String,
    pub email: String,
    pub website: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerSecurity {
    pub use_tls: bool,
    pub key_file: Option<String>,
    pub cert_file: Option<String>,
}