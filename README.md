Selfish Server
===

A Rust HTTP server I'm making for my own learning (i.e., for purely selfish reasons)

NB: This is not meant to be a production server! I'm trying to learn for now, competing with Apache and Nginx comes 
later :)

## Why Am I Doing This? How'd I Get Into It? Where Am I Going?

I started by building on the Python `http.server` module because I needed a local server for a class and anyway I wanted
to learn more about network programming. I decided I wanted to go lower level and started by working off the 
[Building A Multithreaded Webserver](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html) tutorial 
from the Rust website. (In fact the thread pooling code is basically that code verbatim at the moment.) I don't know how
featured I'll make this but hopefully I can make turn it into something rudimentary but still usable and practical.

## Features

- Serves GET and HEAD requests
- Multi-threaded
- TLS
- Configurable

## Todo

- Routing (or better routing)
- Logging
- More robust multi-threading
- Better session management
- Proper command line interface
- Better file organization

## Further down the Line

- Implement the TCP/IP stack myself (because I want to go as low-level as possible)
- Make it so it can function as a BitTorrent peer (for the helluvit)
- Graphical interface

## httpd.ron

The server's main configuration file is located at `/src/httpd.ron`. This serves the same purpose as `httpd.conf` for
Apache servers but its scope is much more limited because this is not as sophisticated.

### `HttpdConfig` Struct

This is the toplevel struct for the config file. All configurations are specified in this struct. Its definition can be
 found at `/src/http/types#HttpdConfig`.
 
### Required Fields

There are three required fields in the config file: `host`, `port`, and `allowed_methods`. `host` is a string with the
IP address (v4 or v6) the server should listen on, and `port` is the port it should listen on. The included `httpd.ron` 
file has these as 0.0.0.0 (all interfaces) on port 8779. The `allowed_methods` in this file are GET and HEAD (which are 
the only requests a server *must* serve). All other requests will result in 405 Method Not Allowed. (You may add POST, 
PUT, etc. requests but since the server isn't set up to work with these properly they'll all be treated like GET 
requests.)

### Optional Fields

`threads`: The server is single-threaded by default but specifying this field as a number greater than 1 will make the
server multithreaded with a thread pool size of the specified amount. This is the maximum number of simultaneous
requests the server can handle at once.

`owner`: Metadata on the administrator of the server.

`security`: TLS configurations

### `ServerOwner` Struct

The value of the `owner` field is an instance of the `ServerOwner` struct. It has fields for the `name`, `email`, and
`website` (optional) of the owner, which are all strings.

### `ServerSecurity` Struct

The value of the `security` field is an instance of the `ServerSecurity` struct. The only required field is `use_tls`,
which is a boolean value that declares whether or not this server should accept secure connections. You can optionally
specify where the private key (`key_file`) and certificate (`cert_file`) are located. If they arn't specified the server
will just use the key/cert file in the current working directory.

### Example Full `httpd.ron` File

```rust
#![enable(implicit_some)]

HttpdConfig (
    host: "192.168.12.4",
    port: 3900,
    allowed_methods: ["GET", "HEAD"],
    threads: 4,
    owner: (
        name: "John Doe",
        email: "johndoe1@example.com",
        website: "example.com",
    ),
    security: (
        use_tls: true,
        key_file: "/home/johndoe/.ssl/key.pem",
        cert_file: "/home/johndoe/.ssl/cert.pem",
    )
)
```