use std::collections::HashMap;
use std::env::current_dir;
use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! from_cargo {
    ($e:expr) => { concat!(env!("CARGO_MANIFEST_DIR"), "/", $e) };
}

lazy_static! {
    pub static ref HTTP_RESPONSE_STATUSES: HashMap<u32, &'static str> = {
        let mut map = HashMap::new();

        // 1xx: Information
        map.insert(100, "Continue");
        map.insert(101, "Switching Protocol");
        map.insert(102, "Processing");
        map.insert(103, "Early Hints");

        // 2xx: Success
        map.insert(200, "OK");
        map.insert(201, "Created");
        map.insert(202, "Accepted");
        map.insert(203, "Non-Authoritative Information");
        map.insert(204, "No Content");
        map.insert(205, "Reset Content");
        map.insert(206, "Partial Content");
        map.insert(207, "Multi-Status");
        map.insert(208, "Already Reported");
        map.insert(226, "IM Used");

        // 3xx: Redirect
        map.insert(300, "Multiple Choice");
        map.insert(301, "Moved Permanently");
        map.insert(302, "Found");
        map.insert(303, "See Other");
        map.insert(304, "Not Modified");
        map.insert(307, "Temporary Redirect");
        map.insert(308, "Permanent Redirect");

        // 4xx: Client Errors
        map.insert(400, "Bad Request");
        map.insert(401, "Unauthorized");
        map.insert(402, "Payment Required");
        map.insert(403, "Forbidden");
        map.insert(404, "Not Found");
        map.insert(405, "Method Not Allowed");
        map.insert(406, "Not Acceptable");
        map.insert(407, "Proxy Authentication Required");
        map.insert(408, "Request Timeout");
        map.insert(409, "Conflict");
        map.insert(410, "Gone");
        map.insert(411, "Length Required");
        map.insert(412, "Precondition Failed");
        map.insert(413, "Payload Too Large");
        map.insert(414, "URI Too Long");
        map.insert(415, "Unsupported Media Type");
        map.insert(416, "Requested Range Not Satisfiable");
        map.insert(417, "Expectation Failed");
        map.insert(418, "I'm A Teapot");
        map.insert(421, "Misdirected Request");
        map.insert(422, "Unprocessable Entity");
        map.insert(423, "Locked");
        map.insert(424, "Failed Dependency");
        map.insert(425, "Too Early");
        map.insert(426, "Upgrade Required");
        map.insert(428, "Precondition Required");
        map.insert(429, "Too Many Requests");
        map.insert(431, "Request Header Fields Too Large");
        map.insert(451, "Unavailable For Legal Reasons");

        // 5xx: Server Errors
        map.insert(500, "Internal Server Error");
        map.insert(501, "Not Implemented");
        map.insert(502, "Bad Gateway");
        map.insert(503, "Service Unavailable");
        map.insert(504, "Gateway Timeout");
        map.insert(506, "Variant Also Negotiates");
        map.insert(507, "Insufficient Storage");
        map.insert(508, "Loop Detected");
        map.insert(510, "Not Extended");
        map.insert(511, "Network Authentication Required");

        map
    };
}

pub fn uri_to_path(uri: &str) -> PathBuf {
    current_dir().unwrap().join(uri.trim_left_matches("/"))
}

pub fn parse_content_type(file: &str) -> String {
    let file_with_ext = Path::new(file);
    if file_with_ext.is_dir() {
        return "text/html; charset=utf-8".to_string();
    }

    match file_with_ext.extension() {
        Some(ext) => {
            let extension = ext.to_str().unwrap();
            match extension {
                "html" | "htm" => "text/html; charset=utf-8",
                "js" => "text/javascript",
                "css" => "text/css",
                "ico" | "cur" => "image/x-icon",
                "jpg" | "jpeg" | "jfif" | "pjpeg" | "pjp" => "image/jpeg",
                "png" => "image/png",
                "svg" => "image/svg+xml",
                "json" => "application/json",
                _ => "application/octet-stream",
            }
        },
        None => "text/plain"
    }.to_string()
}