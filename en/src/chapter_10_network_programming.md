# Chapter 10: Network Programming

Rust is a systems language, which means talking to the network is a first-class concern. The standard library gives you synchronous TCP and UDP; the ecosystem (Tokio, Hyper) gives you high-performance async networking. This chapter moves from raw sockets up to an HTTP server, so you understand each layer rather than just calling a framework.

## Learning Objectives

- Open TCP and UDP connections with `std::net`.
- Build a simple synchronous TCP echo server and client.
- Use Tokio for async, concurrent network handling.
- Serve a minimal HTTP request with `hyper` or a tiny hand-rolled parser.
- Serialize and deserialize structured data with `serde`.

---

## 10.1 TCP with the standard library

`std::net::TcpStream` is a bidirectional byte stream. The simplest client connects, writes, and reads back:

```rust
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("example.com:80")?;

    // Send an HTTP/1.0 request by hand.
    write!(stream, "GET / HTTP/1.0\r\nHost: example.com\r\n\r\n")?;

    // Read the first line of the response.
    let mut reader = BufReader::new(stream);
    let mut status = String::new();
    reader.read_line(&mut status)?;
    println!("{status}");
    Ok(())
}
```

A TCP **server** accepts connections in a loop:

```rust
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};

fn handle(mut stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    println!("received: {line}");
    stream.write_all(b"ack\n")?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    for stream in listener.incoming() {
        let stream = stream?;
        handle(stream)?;
    }
    Ok(())
}
```

`TcpListener::bind` returns `io::Result` because binding can fail (port in use). `incoming()` is an iterator yielding one `io::Result<TcpStream>` per connection.

### Handling one client per thread

The synchronous server above processes clients serially. To serve them concurrently, move each connection onto its own thread:

```rust
use std::net::TcpListener;
use std::thread;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(move || {
            // handle(stream) — see above
            let _ = std::io::copy(&mut &stream[..], &mut &stream[..]);
        });
    }
    Ok(())
}
```

This scales to thousands of idle connections but spends an OS thread per client — fine for many workloads, wasteful for very high concurrency, which is where async shines.

---

## 10.2 UDP

UDP is connectionless: you send datagrams without establishing a stream.

```rust
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:34254")?;
    let mut buf = [0; 1024];

    // Echo received datagrams back to their sender.
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        socket.send_to(&buf[..amt], src)?;
    }
}
```

Use UDP when a lost packet is acceptable (telemetry, games, DNS) or when you implement a reliability layer yourself.

---

## 10.3 Async networking with Tokio

Tokio provides non-blocking TCP/UDP with the same API shape, prefixed with `Async`. The advantage: a single thread can wait on tens of thousands of sockets at once via the OS's I/O multiplexer (epoll/kqueue/IOCP).

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        // Spawn a task per connection — cheap, not an OS thread.
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => return,   // peer closed
                    Ok(n) => n,
                    Err(_) => return,
                };
                if socket.write_all(&buf[..n]).await.is_err() {
                    return;
                }
            }
        });
    }
}
```

This is an **echo server** that handles many clients concurrently on a small pool of threads. Each `tokio::spawn` creates a lightweight task, not an OS thread.

---

## 10.4 A minimal HTTP server

HTTP/1.1 is text on top of TCP. A tiny server can parse just the request line and respond:

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn handle(mut stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    let (method, path) = parse_request_line(&request_line);
    let body = match (method.as_str(), path.as_str()) {
        ("GET", "/") => "hello, world".to_string(),
        ("GET", "/time") => format!("{}", chrono::Utc::now()),
        _ => "not found".to_string(),
    };
    let status = if path == "/" || path == "/time" { "200 OK" } else { "404 Not Found" };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

fn parse_request_line(line: &str) -> (String, String) {
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    (method, path)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle(stream).await {
                eprintln!("error: {e}");
            }
        });
    }
}
```

For anything beyond a toy, reach for a framework — `axum`, `actix-web`, or `hyper` directly — which handles chunked encoding, keep-alive, routing, and TLS correctly.

---

## 10.5 Serialization with `serde`

Network data is bytes; your program wants structs. `serde` is the standard serialization framework, and `serde_json` is its JSON frontend.

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    age: u8,
}

fn main() {
    let user = User { name: "alice".into(), age: 30 };

    // Serialize to JSON.
    let json = serde_json::to_string(&user).unwrap();
    println!("{json}"); // {"name":"alice","age":30}

    // Deserialize back.
    let parsed: User = serde_json::from_str(&json).unwrap();
    println!("{parsed:?}");
}
```

`serde` works with many formats — `bincode` (compact binary), `toml`, `yaml`, `protobuf` via `prost` — all behind the same `Serialize`/`Deserialize` derive.

---

## 10.6 Best Practices

1. **Use `BufReader` / `BufWriter`.** Reading byte-by-byte off a socket is catastrophically slow; buffering is almost always right.
2. **Bound your reads.** Never allocate a buffer based on an untrusted length field without a cap — a classic denial-of-service vector.
3. **Set timeouts.** A socket that never receives data can hang forever. Use `stream.set_read_timeout(Some(...))` or, in async, `tokio::time::timeout`.
4. **Prefer async for high fan-out.** If you expect thousands of concurrent connections, a thread-per-connection model wastes memory.
5. **TLS in production.** Plaintext TCP is fine for learning; for anything exposed to the internet, terminate TLS (e.g. `rustls`).

---

## 10.7 Summary

`std::net` gives you synchronous TCP and UDP; Tokio gives you the same primitives non-blocking, so one thread can manage thousands of sockets. HTTP is text on TCP, and a small server is within reach, though production code should lean on `axum` or `hyper`. Everywhere, `serde` moves between bytes and typed structs. Network programming in Rust is low-level when you need it to be and ergonomic when you want it to be.

### Exercises

1. Extend the sync TCP server so it echoes each line back to the client until the client disconnects.
2. Convert it to async with Tokio, and add a 5-second read timeout per connection.
3. Build a JSON-over-TCP server that receives a `serde` request struct and replies with a response struct.
