# 第10章：网络编程

Rust 是系统级语言，这意味着与网络打交道是一等公民。标准库提供同步的 TCP 与 UDP；生态（Tokio、Hyper）提供高性能的异步网络。本章从原始 socket 一路讲到 HTTP 服务器，让你理解每一层，而不是只会调框架。

## 学习目标

- 用 `std::net` 建立 TCP 与 UDP 连接。
- 实现一个简单的同步 TCP echo 服务器与客户端。
- 用 Tokio 处理异步、并发的网络连接。
- 手写一个极简 HTTP 请求处理。
- 用 `serde` 序列化与反序列化结构化数据。

**实战项目**：构建一个分布式聊天系统，支持多房间、消息持久化等功能。

---

## 10.1 标准库的 TCP

`std::net::TcpStream` 是一个双向字节流。最简单的客户端：连接、写入、读回：

```rust
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("example.com:80")?;

    // 手动发一个 HTTP/1.0 请求
    write!(stream, "GET / HTTP/1.0\r\nHost: example.com\r\n\r\n")?;

    // 读响应的第一行
    let mut reader = BufReader::new(stream);
    let mut status = String::new();
    reader.read_line(&mut status)?;
    println!("{status}");
    Ok(())
}
```

TCP **服务器**在循环里 accept 连接：

```rust
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};

fn handle(mut stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    println!("收到: {line}");
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

`TcpListener::bind` 返回 `io::Result`——绑定可能失败（端口被占）。`incoming()` 是迭代器，每次产出一个 `io::Result<TcpStream>`。

### 每连接一线程

上面的服务器串行处理客户端。要并发服务，把每个连接挪到独立线程：

```rust
use std::net::TcpListener;
use std::thread;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(move || {
            let _ = std::io::copy(&mut &stream[..], &mut &stream[..]);
        });
    }
    Ok(())
}
```

这对数千空闲连接够用，但每个客户端耗一个 OS 线程——高并发场景该用异步。

---

## 10.2 UDP

UDP 是无连接的：发数据报，不建立流。

```rust
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:34254")?;
    let mut buf = [0; 1024];

    // 把收到的数据报回显给发送方
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        socket.send_to(&buf[..amt], src)?;
    }
}
```

UDP 适用于可丢包的场景（遥测、游戏、DNS），或你打算自己实现可靠性层时。

---

## 10.3 用 Tokio 做异步网络

Tokio 提供非阻塞 TCP/UDP，API 形状相同，只是前缀 `Async`。优势：单线程即可靠 I/O 多路复用（epoll/kqueue/IOCP）同时等待成千上万个 socket。

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
        // 每连接派生一个任务——开销小，不是 OS 线程
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => return,   // 对端关闭
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

这是一个能并发服务大量客户端的 **echo 服务器**。每个 `tokio::spawn` 创建轻量任务，不是 OS 线程。

---

## 10.4 一个极简 HTTP 服务器

HTTP/1.1 是 TCP 之上的文本。一个小服务器只需解析请求行并响应：

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn handle(mut stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");
    let (status, body) = match path {
        "/" => ("200 OK", "hello, world".to_string()),
        "/time" => ("200 OK", format!("{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())),
        _ => ("404 Not Found", "not found".to_string()),
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await?;
    Ok(())
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

超出玩具规模就该上框架——`axum`、`actix-web` 或直接用 `hyper`——它们正确处理分块编码、keep-alive、路由与 TLS。

---

## 10.5 用 `serde` 序列化

网络数据是字节，程序想要结构体。`serde` 是标准序列化框架，`serde_json` 是它的 JSON 前端。

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

    let json = serde_json::to_string(&user).unwrap();
    println!("{json}"); // {"name":"alice","age":30}

    let parsed: User = serde_json::from_str(&json).unwrap();
    println!("{parsed:?}");
}
```

`serde` 支持多种格式——`bincode`（紧凑二进制）、`toml`、`yaml`、经 `prost` 的 `protobuf`——背后是同一套 `Serialize`/`Deserialize` 派生。

---

## 10.6 最佳实践

1. **用 `BufReader` / `BufWriter`。** 逐字节读 socket 慢得可怕；缓冲几乎总是对的。
2. **给读取设上限。** 永远别按不可信的长度字段无上限分配缓冲——经典的 DoS 入口。
3. **设超时。** 永不收数据的 socket 会无限挂起。用 `stream.set_read_timeout(Some(...))`，或异步里用 `tokio::time::timeout`。
4. **高扇出用异步。** 预期数千并发连接时，每连接一线程浪费内存。
5. **生产用 TLS。** 学习用明文 TCP 无妨；面向互联网的任何东西都要终结 TLS（如 `rustls`）。

---

## 10.7 小结

`std::net` 给你同步 TCP/UDP；Tokio 给你同样原语的非阻塞版本，让单线程管理数千 socket。HTTP 是 TCP 上的文本，小服务器触手可及，但生产代码应交给 `axum`/`hyper`。无论何处，`serde` 在字节与类型化结构体之间搬运。Rust 的网络编程该底层时能底层，该省事时能省事。

### 练习

1. 把同步 TCP 服务器改为逐行回显，直到客户端断开。
2. 用 Tokio 改写为异步版本，并为每个连接加 5 秒读超时。
3. 实现一个基于 TCP 的 JSON 服务：接收 `serde` 请求结构体，返回响应结构体。
