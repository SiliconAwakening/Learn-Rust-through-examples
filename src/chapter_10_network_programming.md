# 第10章：网络编程

## 章节概述

网络编程是现代应用程序开发的核心技能。在本章中，我们将深入探索Rust的网络编程能力，从基础的TCP/UDP通信到高级的WebSocket和分布式系统设计。本章不仅关注技术实现，更强调生产环境下的稳定性和性能优化。

**学习目标**：
- 掌握Rust网络编程的核心概念和API
- 理解TCP/UDP协议的工作原理和适用场景
- 学会构建高性能的HTTP客户端和服务器
- 掌握WebSocket协议实现实时双向通信
- 掌握JSON、MessagePack等序列化技术
- 设计并实现一个可扩展的分布式聊天系统

**实战项目**：构建一个企业级分布式聊天系统，支持多房间、文件传输、消息持久化等功能。

## 10.1 网络编程基础

### 10.1.1 Rust网络编程特性

Rust在网络编程方面具有以下独特优势：

- **内存安全**：防止缓冲区溢出、竞态条件等常见网络编程错误
- **零成本抽象**：高性能的网络编程抽象，接近C++的性能
- **类型安全**：编译时类型检查，避免运行时错误
- **异步支持**：优秀的异步/await支持，适合高并发网络应用

### 10.1.2 核心网络库介绍

#### std::net模块
```rust
// TCP连接示例
use std::net::TcpStream;
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    stream.write_all(b"Hello, Server!")?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("Response: {}", response);
    Ok(())
}
```

#### tokio异步网络库
```rust
// 异步TCP服务器
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New client: {}", addr);
        
        tokio::spawn(async move {
            handle_client(stream).await
        });
    }
}

async fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    if n == 0 {
        return Ok(());
    }
    
    let response = format!("Echo: {}", String::from_utf8_lossy(&buffer[..n]));
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}
```

### 10.1.3 网络编程最佳实践

1. **错误处理**：使用Result类型处理网络错误
2. **资源管理**：确保网络资源正确释放
3. **超时设置**：避免无限等待
4. **安全考虑**：验证输入、防止注入攻击
5. **性能优化**：使用连接池、批量处理等技术

## 10.2 TCP/UDP编程

### 10.2.1 TCP协议编程

#### 基础TCP客户端

```rust
use std::net::{TcpStream, SocketAddr};
use std::io::{Read, Write, BufReader, BufRead};
use std::time::Duration;

pub struct TcpClient {
    stream: TcpStream,
    buffer_size: usize,
    timeout: Duration,
}

impl TcpClient {
    pub fn connect(addr: SocketAddr) -> Result<Self, std::io::Error> {
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(10))?;
        stream.set_non_blocking(false)?;
        
        Ok(TcpClient {
            stream,
            buffer_size: 4096,
            timeout: Duration::from_secs(30),
        })
    }
    
    pub fn send(&mut self, data: &[u8]) -> Result<usize, std::io::Error> {
        self.stream.set_write_timeout(Some(self.timeout))?;
        self.stream.write_all(data)?;
        Ok(data.len())
    }
    
    pub fn receive(&mut self) -> Result<Vec<u8>, std::io::Error> {
        self.stream.set_read_timeout(Some(self.timeout))?;
        let mut buffer = vec![0u8; self.buffer_size];
        let n = self.stream.read(&mut buffer)?;
        buffer.truncate(n);
        Ok(buffer)
    }
    
    pub fn send_message(&mut self, message: &str) -> Result<String, std::io::Error> {
        self.send(message.as_bytes())?;
        let response = self.receive()?;
        Ok(String::from_utf8_lossy(&response).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::ToSocketAddrs;
    
    #[test]
    fn test_tcp_client_creation() {
        let addr = "127.0.0.1:0".to_socket_addrs().unwrap().next().unwrap();
        // 测试客户端创建
    }
}
```

#### 高级TCP服务器

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{timeout, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TcpServer {
    listener: TcpListener,
    clients: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
    message_rx: mpsc::UnboundedReceiver<String>,
    message_tx: mpsc::UnboundedSender<String>,
}

impl TcpServer {
    pub async fn bind(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        
        Ok(TcpServer {
            listener,
            clients: Arc::new(Mutex::new(HashMap::new())),
            message_rx,
            message_tx,
        })
    }
    
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let clients = Arc::clone(&self.clients);
        
        // 处理消息广播
        let broadcast_task = {
            let message_tx = self.message_tx.clone();
            tokio::spawn(async move {
                while let Some(message) = self.message_rx.recv().await {
                    let clients = clients.lock().await;
                    for (_, sender) in &clients {
                        let _ = sender.send(message.clone());
                    }
                }
            })
        };
        
        // 处理新连接
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    let client_id = format!("{}", addr);
                    println!("New client connected: {}", client_id);
                    
                    let clients = Arc::clone(&self.clients);
                    let message_tx = self.message_tx.clone();
                    
                    tokio::spawn(async move {
                        Self::handle_client(stream, client_id, clients, message_tx).await
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_client(
        stream: TcpStream,
        client_id: String,
        clients: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
        message_tx: mpsc::UnboundedSender<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = tokio::io::BufReader::new(reader);
        
        // 创建客户端消息发送通道
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        {
            let mut clients = clients.lock().await;
            clients.insert(client_id.clone(), tx);
        }
        
        let mut client = TcpClientHandler::new(client_id.clone());
        client.register_disconnect_hook(clients.clone());
        
        // 处理客户端消息
        let message_task = tokio::spawn(async move {
            let mut buffer = String::new();
            loop {
                match timeout(Duration::from_secs(30), buf_reader.read_line(&mut buffer)).await {
                    Ok(Ok(0)) => break, // 连接关闭
                    Ok(Ok(n)) => {
                        if n > 0 {
                            let message = format!("{}: {}", client_id, buffer.trim());
                            let _ = message_tx.send(message);
                            buffer.clear();
                        }
                    }
                    Ok(Err(e)) => {
                        eprintln!("Error reading from client {}: {}", client_id, e);
                        break;
                    }
                    Err(_) => {
                        // 超时，继续等待
                    }
                }
            }
        });
        
        // 处理发送消息
        let send_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = writer.write_all(message.as_bytes()).await {
                    eprintln!("Error writing to client {}: {}", client_id, e);
                    break;
                }
            }
        });
        
        tokio::select! {
            result = message_task => {
                if let Err(e) = result {
                    eprintln!("Message task error: {}", e);
                }
            }
            result = send_task => {
                if let Err(e) = result {
                    eprintln!("Send task error: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

struct TcpClientHandler {
    client_id: String,
    disconnect_hooks: Vec<Box<dyn Fn(&str) + Send + Sync>>,
}

impl TcpClientHandler {
    pub fn new(client_id: String) -> Self {
        TcpClientHandler {
            client_id,
            disconnect_hooks: Vec::new(),
        }
    }
    
    pub fn register_disconnect_hook(&mut self, clients: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>) {
        let client_id = self.client_id.clone();
        self.disconnect_hooks.push(Box::new(move |_| {
            let mut clients = clients.lock().unwrap();
            clients.remove(&client_id);
            println!("Client {} disconnected and removed", client_id);
        }));
    }
}
```

### 10.2.2 UDP协议编程

UDP是无连接的协议，适用于对实时性要求高、能够容忍丢包的场景，如视频流、实时游戏等。

```rust
use std::net::{UdpSocket, SocketAddr};
use tokio::net::UdpSocket as AsyncUdpSocket;
use tokio::time::Duration;

pub struct UdpClient {
    socket: AsyncUdpSocket,
    target_addr: SocketAddr,
    buffer_size: usize,
}

impl UdpClient {
    pub async fn connect(target_addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = AsyncUdpSocket::bind("0.0.0.0:0").await?;
        Ok(UdpClient {
            socket,
            target_addr,
            buffer_size: 1024,
        })
    }
    
    pub async fn send(&self, data: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        let result = self.socket.send_to(data, self.target_addr).await?;
        Ok(result)
    }
    
    pub async fn receive(&self) -> Result<(Vec<u8>, SocketAddr), Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; self.buffer_size];
        let (size, source) = self.socket.recv_from(&mut buffer).await?;
        buffer.truncate(size);
        Ok((buffer, source))
    }
    
    pub async fn send_and_receive(
        &self,
        data: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.send(data)?;
        
        tokio::time::timeout(timeout, self.receive())
            .await
            .map_err(|_| "Timeout waiting for response")?
            .map(|(response, _)| response)
    }
}

pub struct UdpServer {
    socket: AsyncUdpSocket,
    clients: Arc<Mutex<HashMap<SocketAddr, ClientInfo>>>,
}

#[derive(Clone)]
struct ClientInfo {
    last_seen: std::time::Instant,
    message_count: u64,
}

impl UdpServer {
    pub async fn bind(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = AsyncUdpSocket::bind(addr).await?;
        Ok(UdpServer {
            socket,
            clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let clients = Arc::clone(&self.clients);
        let mut buffer = vec![0u8; 65536];
        
        loop {
            match self.socket.recv_from(&mut buffer).await {
                Ok((size, source)) => {
                    println!("Received {} bytes from {}", size, source);
                    
                    // 更新客户端信息
                    {
                        let mut clients = clients.lock().await;
                        let info = clients.entry(source).or_insert_with(|| ClientInfo {
                            last_seen: std::time::Instant::now(),
                            message_count: 0,
                        });
                        info.last_seen = std::time::Instant::now();
                        info.message_count += 1;
                    }
                    
                    // 处理数据
                    let response = self.process_packet(&buffer[..size]);
                    
                    // 发送响应
                    if !response.is_empty() {
                        if let Err(e) = self.socket.send_to(&response, source).await {
                            eprintln!("Failed to send response to {}: {}", source, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("UDP receive error: {}", e);
                }
            }
        }
    }
    
    fn process_packet(&self, data: &[u8]) -> Vec<u8> {
        // 简单的回应协议
        format!("ACK: {}", data.len()).into_bytes()
    }
    
    pub async fn get_client_stats(&self) -> Vec<(SocketAddr, ClientInfo)> {
        let clients = self.clients.lock().await;
        clients.clone().into_iter().collect()
    }
}

#[cfg(test)]
mod udp_tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_udp_echo() -> Result<(), Box<dyn std::error::Error>> {
        let server_addr: SocketAddr = "127.0.0.1:0".parse()?;
        let server = UdpServer::bind("127.0.0.1:0").await?;
        let bind_addr = server.socket.local_addr()?;
        
        let client = UdpClient::connect(bind_addr).await?;
        
        let test_data = b"Hello, UDP!";
        let response = client.send_and_receive(test_data, Duration::from_secs(1)).await?;
        
        assert_eq!(response, b"ACK: 10");
        Ok(())
    }
}
```

## 10.3 HTTP客户端与服务器

### 10.3.1 HTTP客户端

#### 基于原生std库的HTTP客户端

```rust
use std::io::{Read, Write};
use std::net::TcpStream;
use std::collections::HashMap;

pub struct HttpClient {
    host: String,
    port: u16,
    timeout: std::time::Duration,
}

impl HttpClient {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        HttpClient {
            host: host.into(),
            port,
            timeout: std::time::Duration::from_secs(10),
        }
    }
    
    pub fn get(&self, path: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        self.request("GET", path, None)
    }
    
    pub fn post(&self, path: &str, body: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        self.request("POST", path, Some(body))
    }
    
    fn request(&self, method: &str, path: &str, body: Option<&str>) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let address = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(&address)?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;
        
        // 构建HTTP请求
        let mut headers = HashMap::new();
        headers.insert("Host", &self.host);
        headers.insert("Connection", "close");
        headers.insert("User-Agent", "Rust-HttpClient/1.0");
        
        if let Some(body) = body {
            headers.insert("Content-Type", "application/json");
            headers.insert("Content-Length", &body.len().to_string());
        }
        
        let request = self.build_request(method, path, &headers, body);
        stream.write_all(request.as_bytes())?;
        
        // 读取响应
        let mut response_buffer = String::new();
        stream.read_to_string(&mut response_buffer)?;
        
        HttpResponse::parse(&response_buffer)
    }
    
    fn build_request(&self, method: &str, path: &str, headers: &HashMap<&str, &str>, body: Option<&str>) -> String {
        let mut request = format!("{} {} HTTP/1.1\r\n", method, path);
        
        for (key, value) in headers {
            request.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        request.push_str("\r\n");
        
        if let Some(body) = body {
            request.push_str(body);
        }
        
        request
    }
}

pub struct HttpResponse {
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponse {
    fn parse(response: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = response.split("\r\n").collect();
        
        if lines.is_empty() {
            return Err("Empty response".into());
        }
        
        // 解析状态行
        let status_line = lines[0];
        let status_parts: Vec<&str> = status_line.split(' ').collect();
        if status_parts.len() < 2 {
            return Err("Invalid status line".into());
        }
        
        let status_code = status_parts[1].parse::<u16>()?;
        
        // 找到空行分隔头部和主体
        let mut header_lines = Vec::new();
        let mut body_lines = Vec::new();
        let mut in_headers = true;
        
        for line in &lines[1..] {
            if line.is_empty() {
                in_headers = false;
                continue;
            }
            
            if in_headers {
                header_lines.push(*line);
            } else {
                body_lines.push(*line);
            }
        }
        
        // 解析头部
        let mut headers = HashMap::new();
        for line in header_lines {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }
        
        // 构建主体
        let body = body_lines.join("\r\n");
        
        Ok(HttpResponse {
            status_code,
            headers,
            body,
        })
    }
    
    pub fn status(&self) -> u16 {
        self.status_code
    }
    
    pub fn body(&self) -> &str {
        &self.body
    }
    
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_http_response_parsing() {
        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 13\r\n\r\n{\"status\":\"ok\"}";
        let parsed = HttpResponse::parse(response).unwrap();
        
        assert_eq!(parsed.status(), 200);
        assert_eq!(parsed.body(), "{\"status\":\"ok\"}");
        assert_eq!(parsed.header("Content-Type"), Some("application/json"));
    }
}
```

#### 异步HTTP客户端（使用reqwest）

```rust
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

pub struct AsyncHttpClient {
    client: Client,
    base_url: String,
    default_headers: HashMap<String, String>,
}

impl AsyncHttpClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        AsyncHttpClient {
            client: Client::new(),
            base_url: base_url.into(),
            default_headers: HashMap::new(),
        }
    }
    
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }
    
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request = self.client.get(&url);
        
        // 添加默认头部
        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }
        
        let response = request.send().await?;
        self.handle_response::<T>(response).await
    }
    
    pub async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request = self.client.post(&url).json(body);
        
        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }
        
        let response = request.send().await?;
        self.handle_response::<T>(response).await
    }
    
    pub async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request = self.client.put(&url).json(body);
        
        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }
        
        let response = request.send().await?;
        self.handle_response::<T>(response).await
    }
    
    pub async fn delete<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request = self.client.delete(&url);
        
        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }
        
        let response = request.send().await?;
        self.handle_response::<T>(response).await
    }
    
    async fn handle_response<T: for<'de> Deserialize<'de>>(&self, response: Response) -> Result<T, Box<dyn std::error::Error>> {
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await?;
            let error: ApiError = serde_json::from_str(&error_text)
                .unwrap_or_else(|_| ApiError {
                    error: "Unknown error".to_string(),
                    message: error_text,
                });
            
            return Err(format!("HTTP {}: {} - {}", status.as_u16(), error.error, error.message).into());
        }
        
        let result = response.json::<T>().await?;
        Ok(result)
    }
}

// 使用示例
#[cfg(test)]
mod http_client_tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_async_client() {
        // 创建模拟服务器进行测试
        // 或者使用公共API进行集成测试
    }
}
```

### 10.3.2 HTTP服务器

#### 基于tokio的HTTP服务器

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct RouteHandler {
    handlers: Arc<Mutex<HashMap<String, Handler>>>,
}

impl RouteHandler {
    pub fn new() -> Self {
        RouteHandler {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn get(&self, path: &str, handler: Handler) {
        let mut handlers = self.handlers.blocking_lock();
        handlers.insert(format!("GET:{}", path), handler);
    }
    
    pub fn post(&self, path: &str, handler: Handler) {
        let mut handlers = self.handlers.blocking_lock();
        handlers.insert(format!("POST:{}", path), handler);
    }
    
    pub fn put(&self, path: &str, handler: Handler) {
        let mut handlers = self.handlers.blocking_lock();
        handlers.insert(format!("PUT:{}", path), handler);
    }
    
    pub fn delete(&self, path: &str, handler: Handler) {
        let mut handlers = self.handlers.blocking_lock();
        handlers.insert(format!("DELETE:{}", path), handler);
    }
    
    pub async fn handle_request(&self, method: &str, path: &str, body: &[u8]) -> Result<Response, Box<dyn std::error::Error>> {
        let handlers = self.handlers.lock().await;
        let handler_key = format!("{}:{}", method, path);
        
        if let Some(handler) = handlers.get(&handler_key) {
            handler(method, path, body).await
        } else {
            Ok(Response::not_found("Route not found"))
        }
    }
}

type Handler = fn(&str, &str, &[u8]) -> Pin<Box<dyn std::future::Future<Output = Result<Response, Box<dyn std::error::Error>>> + Send>>;

pub struct Response {
    status_code: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    pub fn ok(body: impl AsRef<[u8]>) -> Self {
        Response {
            status_code: 200,
            headers: [(String::from("Content-Type"), String::from("application/json"))]
                .iter()
                .cloned()
                .collect(),
            body: body.as_ref().to_vec(),
        }
    }
    
    pub fn created(body: impl AsRef<[u8]>) -> Self {
        Response {
            status_code: 201,
            headers: [(String::from("Content-Type"), String::from("application/json"))]
                .iter()
                .cloned()
                .collect(),
            body: body.as_ref().to_vec(),
        }
    }
    
    pub fn bad_request(message: &str) -> Self {
        Response {
            status_code: 400,
            headers: [(String::from("Content-Type"), String::from("application/json"))]
                .iter()
                .cloned()
                .collect(),
            body: json!({ "error": message }).to_string().into_bytes(),
        }
    }
    
    pub fn not_found(message: &str) -> Self {
        Response {
            status_code: 404,
            headers: [(String::from("Content-Type"), String::from("application/json"))]
                .iter()
                .cloned()
                .collect(),
            body: json!({ "error": message }).to_string().into_bytes(),
        }
    }
    
    pub fn internal_server_error(message: &str) -> Self {
        Response {
            status_code: 500,
            headers: [(String::from("Content-Type"), String::from("application/json"))]
                .iter()
                .cloned()
                .collect(),
            body: json!({ "error": message }).to_string().into_bytes(),
        }
    }
    
    pub fn to_http_response(&self) -> String {
        let mut response = format!("HTTP/1.1 {} OK\r\n", self.status_code);
        
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        response.push_str("Connection: close\r\n");
        response.push_str("\r\n");
        response.push_str(&String::from_utf8_lossy(&self.body));
        
        response
    }
}

pub struct HttpServer {
    listener: TcpListener,
    routes: RouteHandler,
    static_files: Option<Arc<std::path::PathBuf>>,
}

impl HttpServer {
    pub async fn bind(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        
        Ok(HttpServer {
            listener,
            routes: RouteHandler::new(),
            static_files: None,
        })
    }
    
    pub fn with_static_files(&mut self, path: &std::path::Path) {
        self.static_files = Some(Arc::new(path.to_path_buf()));
    }
    
    pub fn setup_routes(&self) {
        // 根路径
        self.routes.get("/", |_, _, _| {
            Box::pin(async {
                Ok(Response::ok(json!({ "message": "Welcome to Rust HTTP Server" })))
            })
        });
        
        // 健康检查
        self.routes.get("/health", |_, _, _| {
            Box::pin(async {
                Ok(Response::ok(json!({ "status": "healthy" })))
            })
        });
        
        // API示例
        self.routes.get("/api/users", |_, _, _| {
            Box::pin(async {
                let users = json!([
                    { "id": 1, "name": "Alice", "email": "alice@example.com" },
                    { "id": 2, "name": "Bob", "email": "bob@example.com" }
                ]);
                Ok(Response::ok(users))
            })
        });
        
        self.routes.post("/api/echo", |_, _, body| {
            Box::pin(async {
                if let Ok(value) = serde_json::from_slice::<Value>(body) {
                    Ok(Response::ok(value))
                } else {
                    Ok(Response::bad_request("Invalid JSON"))
                }
            })
        });
    }
    
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("HTTP Server listening on {}", self.listener.local_addr()?);
        
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    println!("New connection from {}", addr);
                    
                    let routes = self.routes.clone();
                    let static_files = self.static_files.clone();
                    
                    tokio::spawn(async move {
                        handle_client(stream, routes, static_files).await
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

async fn handle_client(
    stream: TcpStream,
    routes: RouteHandler,
    static_files: Option<Arc<std::path::PathBuf>>,
) {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut buffer = String::new();
    
    // 读取HTTP请求
    if let Err(e) = buf_reader.read_to_string(&mut buffer).await {
        eprintln!("Failed to read request: {}", e);
        return;
    }
    
    // 解析HTTP请求
    if let Ok((method, path, _headers, body)) = parse_http_request(&buffer) {
        // 处理静态文件
        if let Some(static_path) = &static_files {
            if let Some(file_response) = serve_static_file(&path, static_path).await {
                let _ = writer.write_all(file_response.to_http_response().as_bytes()).await;
                return;
            }
        }
        
        // 处理路由
        match routes.handle_request(&method, &path, body.as_bytes()).await {
            Ok(response) => {
                let http_response = response.to_http_response();
                if let Err(e) = writer.write_all(http_response.as_bytes()).await {
                    eprintln!("Failed to send response: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Request handling error: {}", e);
                let error_response = Response::internal_server_error("Internal server error");
                let _ = writer.write_all(error_response.to_http_response().as_bytes()).await;
            }
        }
    } else {
        let error_response = Response::bad_request("Invalid HTTP request");
        let _ = writer.write_all(error_response.to_http_response().as_bytes()).await;
    }
}

fn parse_http_request(request: &str) -> Result<(String, String, HashMap<String, String>, String), Box<dyn std::error::Error>> {
    let lines: Vec<&str> = request.split("\r\n").collect();
    
    if lines.is_empty() {
        return Err("Empty request".into());
    }
    
    // 解析请求行
    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split(' ').collect();
    if parts.len() != 3 {
        return Err("Invalid request line".into());
    }
    
    let method = parts[0].to_string();
    let path = parts[1].to_string();
    
    // 解析头部
    let mut headers = HashMap::new();
    let mut body = String::new();
    let mut in_headers = true;
    
    for line in &lines[1..] {
        if line.is_empty() {
            in_headers = false;
            continue;
        }
        
        if in_headers {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        } else {
            body.push_str(line);
        }
    }
    
    Ok((method, path, headers, body))
}

async fn serve_static_file(path: &str, static_dir: &std::path::Path) -> Option<Response> {
    let mut file_path = static_dir.to_path_buf();
    
    // 移除开头的斜杠
    let clean_path = path.trim_start_matches('/');
    file_path.push(clean_path);
    
    // 防止目录遍历攻击
    if file_path.to_string_lossy().contains("..") {
        return None;
    }
    
    // 默认索引文件
    if clean_path.is_empty() || clean_path.ends_with('/') {
        file_path.push("index.html");
    }
    
    if let Ok(content) = tokio::fs::read(&file_path).await {
        let content_type = match file_path.extension()?.to_str()? {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        };
        
        let mut response = Response::ok(content);
        response.headers.insert("Content-Type".to_string(), content_type.to_string());
        Some(response)
    } else {
        None
    }
}

use std::pin::Pin;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_http_server() {
        let mut server = HttpServer::bind("127.0.0.1:0").await.unwrap();
        server.setup_routes();
        
        let addr = server.listener.local_addr().unwrap();
        
        // 在真实测试中，我们启动服务器并发送HTTP请求
        // 这里省略具体实现
    }
}
```

## 10.4 WebSocket实现

WebSocket提供了全双工通信能力，特别适合实时应用如聊天、游戏、股票交易等场景。

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use base64::{Engine as _, engine::general_purpose};
use sha1::{Digest, Sha1};

const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Close,
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

pub struct WebSocketServer {
    listener: TcpListener,
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<WebSocketMessage>>>>,
    message_tx: broadcast::Sender<WebSocketMessage>,
}

impl WebSocketServer {
    pub async fn bind(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        let (message_tx, _) = broadcast::channel(1000);
        
        Ok(WebSocketServer {
            listener,
            clients: Arc::new(Mutex::new(HashMap::new())),
            message_tx,
        })
    }
    
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("WebSocket Server listening on {}", self.listener.local_addr()?);
        
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    let client_id = format!("{}", addr);
                    println!("New WebSocket client: {}", client_id);
                    
                    let clients = Arc::clone(&self.clients);
                    let message_tx = self.message_tx.clone();
                    
                    tokio::spawn(async move {
                        WebSocketServer::handle_websocket_client(stream, client_id, clients, message_tx).await
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept WebSocket connection: {}", e);
                }
            }
        }
    }
    
    async fn handle_websocket_client(
        stream: TcpStream,
        client_id: String,
        clients: Arc<Mutex<HashMap<String, broadcast::Sender<WebSocketMessage>>>>,
        message_tx: broadcast::Sender<WebSocketMessage>,
    ) {
        if let Err(e) = WebSocketServer::handle_connection(stream, &client_id, clients, message_tx).await {
            eprintln!("WebSocket error for client {}: {}", client_id, e);
        }
    }
    
    async fn handle_connection(
        stream: TcpStream,
        client_id: &str,
        clients: Arc<Mutex<HashMap<String, broadcast::Sender<WebSocketMessage>>>>,
        message_tx: broadcast::Sender<WebSocketMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (mut reader, mut writer) = stream.into_split();
        
        // 读取初始HTTP请求
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer).await?;
        
        // 解析WebSocket握手请求
        let (key, response_key) = parse_websocket_handshake(&buffer)?;
        
        // 发送握手响应
        let handshake_response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\
             \r\n",
            response_key
        );
        
        writer.write_all(handshake_response.as_bytes()).await?;
        
        // 创建客户端消息通道
        let (tx, mut rx) = broadcast::channel(100);
        {
            let mut clients = clients.lock().await;
            clients.insert(client_id.to_string(), tx);
        }
        
        // 读取WebSocket消息的异步任务
        let read_task = tokio::spawn(async move {
            let mut frame_buffer = vec![0u8; 4096];
            
            loop {
                match WebSocketFrame::read_frame(&mut reader, &mut frame_buffer).await {
                    Ok(Some(frame)) => {
                        match frame.opcode() {
                            0x8 => { // Close frame
                                break;
                            }
                            0x9 => { // Ping frame
                                let pong_frame = WebSocketFrame::pong(frame.payload());
                                if let Ok(data) = pong_frame.to_bytes() {
                                    let _ = writer.write_all(&data).await;
                                }
                            }
                            _ => {
                                // 处理其他帧类型
                                println!("Received frame: {:?}", frame.opcode());
                            }
                        }
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(e) => {
                        eprintln!("Frame read error: {}", e);
                        break;
                    }
                }
            }
        });
        
        // 发送消息的异步任务
        let write_task = tokio::spawn(async move {
            while let Ok(message) = rx.recv().await {
                match message {
                    WebSocketMessage::Text(text) => {
                        let frame = WebSocketFrame::text(text.as_bytes());
                        if let Ok(data) = frame.to_bytes() {
                            if let Err(e) = writer.write_all(&data).await {
                                eprintln!("Failed to write WebSocket message: {}", e);
                                break;
                            }
                        }
                    }
                    WebSocketMessage::Binary(binary) => {
                        let frame = WebSocketFrame::binary(&binary);
                        if let Ok(data) = frame.to_bytes() {
                            if let Err(e) = writer.write_all(&data).await {
                                eprintln!("Failed to write WebSocket message: {}", e);
                                break;
                            }
                        }
                    }
                    WebSocketMessage::Close => {
                        let close_frame = WebSocketFrame::close();
                        if let Ok(data) = close_frame.to_bytes() {
                            let _ = writer.write_all(&data).await;
                        }
                        break;
                    }
                    _ => {
                        // 忽略ping/pong，它们在读取端处理
                    }
                }
            }
        });
        
        tokio::select! {
            result = read_task => {
                if let Err(e) = result {
                    eprintln!("Read task error: {}", e);
                }
            }
            result = write_task => {
                if let Err(e) = result {
                    eprintln!("Write task error: {}", e);
                }
            }
        }
        
        // 清理客户端
        {
            let mut clients = clients.lock().await;
            clients.remove(client_id);
        }
        
        Ok(())
    }
}

fn parse_websocket_handshake(request: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let lines: Vec<&str> = request.split("\r\n").collect();
    
    // 查找Sec-WebSocket-Key
    let mut key = None;
    for line in &lines {
        if line.starts_with("Sec-WebSocket-Key:") {
            key = Some(line[19..].trim().to_string());
            break;
        }
    }
    
    let key = key.ok_or("Missing Sec-WebSocket-Key header")?;
    
    // 计算响应密钥
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(WEBSOCKET_GUID.as_bytes());
    let hash = hasher.finalize();
    let response_key = general_purpose::STANDARD.encode(&hash);
    
    Ok((key, response_key))
}

struct WebSocketFrame {
    fin: bool,
    opcode: u8,
    mask: bool,
    payload: Vec<u8>,
}

impl WebSocketFrame {
    pub fn text(data: &[u8]) -> Self {
        WebSocketFrame {
            fin: true,
            opcode: 0x1,
            mask: true,
            payload: data.to_vec(),
        }
    }
    
    pub fn binary(data: &[u8]) -> Self {
        WebSocketFrame {
            fin: true,
            opcode: 0x2,
            mask: true,
            payload: data.to_vec(),
        }
    }
    
    pub fn close() -> Self {
        WebSocketFrame {
            fin: true,
            opcode: 0x8,
            mask: true,
            payload: vec![],
        }
    }
    
    pub fn pong(payload: &[u8]) -> Self {
        WebSocketFrame {
            fin: true,
            opcode: 0xA,
            mask: true,
            payload: payload.to_vec(),
        }
    }
    
    pub fn opcode(&self) -> u8 {
        self.opcode
    }
    
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    
    pub async fn read_frame<R: AsyncReadExt>(
        reader: &mut R,
        buffer: &mut [u8],
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let n = reader.read(buffer).await?;
        if n == 0 {
            return Ok(None);
        }
        
        if n < 2 {
            return Err("Insufficient data for WebSocket frame".into());
        }
        
        let first_byte = buffer[0];
        let second_byte = buffer[1];
        
        let fin = (first_byte & 0x80) != 0;
        let opcode = first_byte & 0x0F;
        let mask = (second_byte & 0x80) != 0;
        let mut payload_len = (second_byte & 0x7F) as usize;
        
        let mut offset = 2;
        
        // 处理扩展长度
        if payload_len == 126 {
            if n < offset + 2 {
                return Err("Insufficient data for extended payload length".into());
            }
            payload_len = ((buffer[offset] as usize) << 8) | (buffer[offset + 1] as usize);
            offset += 2;
        } else if payload_len == 127 {
            if n < offset + 8 {
                return Err("Insufficient data for extended payload length".into());
            }
            // 64位长度支持（这里简化处理）
            payload_len = ((buffer[offset + 4] as usize) << 24) | 
                         ((buffer[offset + 5] as usize) << 16) | 
                         ((buffer[offset + 6] as usize) << 8) | 
                          (buffer[offset + 7] as usize);
            offset += 8;
        }
        
        // 处理掩码
        let mut payload = vec![0u8; payload_len];
        if mask {
            if n < offset + 4 {
                return Err("Insufficient data for masking key".into());
            }
            let masking_key = &buffer[offset..offset + 4];
            offset += 4;
            
            if n < offset + payload_len {
                return Err("Insufficient data for payload".into());
            }
            
            for i in 0..payload_len {
                payload[i] = buffer[offset + i] ^ masking_key[i % 4];
            }
        } else {
            if n < offset + payload_len {
                return Err("Insufficient data for unmasked payload".into());
            }
            payload.copy_from_slice(&buffer[offset..offset + payload_len]);
        }
        
        Ok(Some(WebSocketFrame {
            fin,
            opcode,
            mask,
            payload,
        }))
    }
    
    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut frame = Vec::new();
        
        // 第一个字节
        let mut first_byte = if self.fin { 0x80 } else { 0x00 } | (self.opcode & 0x0F);
        frame.push(first_byte);
        
        // 第二个字节
        let mut second_byte = if self.mask { 0x80 } else { 0x00 };
        
        let payload_len = self.payload.len();
        if payload_len < 126 {
            second_byte |= payload_len as u8;
            frame.push(second_byte);
        } else if payload_len <= 65535 {
            second_byte |= 126;
            frame.push(second_byte);
            frame.push((payload_len >> 8) as u8);
            frame.push(payload_len as u8);
        } else {
            second_byte |= 127;
            frame.push(second_byte);
            // 64位长度（简化）
            for i in (0..8).rev() {
                frame.push((payload_len >> (i * 8)) as u8);
            }
        }
        
        // 掩码密钥
        if self.mask {
            let masking_key = [0x12, 0x34, 0x56, 0x78]; // 固定掩码（实际应该随机生成）
            frame.extend_from_slice(&masking_key);
            
            for (i, byte) in self.payload.iter().enumerate() {
                frame.push(*byte ^ masking_key[i % 4]);
            }
        } else {
            frame.extend_from_slice(&self.payload);
        }
        
        Ok(frame)
    }
}

pub struct WebSocketClient {
    stream: TcpStream,
    server_url: String,
}

impl WebSocketClient {
    pub async fn connect(server_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 解析URL（简化版本）
        let url_parts: Vec<&str> = server_url.split('/').collect();
        let host = url_parts[2];
        let path = if url_parts.len() > 3 {
            format!("/{}", url_parts[3..].join("/"))
        } else {
            "/".to_string()
        };
        
        // 建立TCP连接
        let (host, port) = if host.contains(':') {
            let parts: Vec<&str> = host.split(':').collect();
            (parts[0].to_string(), parts[1].parse()?)
        } else {
            (host.to_string(), 80)
        };
        
        let stream = TcpStream::connect((host.as_str(), port))?;
        
        Ok(WebSocketClient {
            stream,
            server_url: server_url.to_string(),
        })
    }
    
    pub async fn handshake(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 生成随机密钥
        let key = generate_random_key();
        
        // 发送握手请求
        let handshake_request = format!(
            "GET {} HTTP/1.1\r\n\
             Host: {}\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Key: {}\r\n\
             Sec-WebSocket-Version: 13\r\n\
             \r\n",
            self.extract_path()?,
            self.extract_host()?,
            key
        );
        
        self.stream.write_all(handshake_request.as_bytes())?;
        
        // 读取握手响应
        let mut response = String::new();
        self.stream.read_to_string(&mut response)?;
        
        if !response.contains("101 Switching Protocols") {
            return Err("WebSocket handshake failed".into());
        }
        
        Ok(())
    }
    
    pub async fn send_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let frame = WebSocketFrame::text(text.as_bytes());
        let data = frame.to_bytes()?;
        self.stream.write_all(&data)?;
        Ok(())
    }
    
    pub async fn send_binary(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let frame = WebSocketFrame::binary(data);
        let data = frame.to_bytes()?;
        self.stream.write_all(&data)?;
        Ok(())
    }
    
    pub async fn receive(&mut self) -> Result<Option<WebSocketMessage>, Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; 4096];
        let n = self.stream.read(&mut buffer).await?;
        
        if n == 0 {
            return Ok(None);
        }
        
        let frame = WebSocketFrame::read_frame(&mut &buffer[..n]).await?;
        Ok(frame.map(|f| match f.opcode() {
            0x1 => WebSocketMessage::Text(String::from_utf8_lossy(f.payload()).to_string()),
            0x2 => WebSocketMessage::Binary(f.payload().to_vec()),
            0x8 => WebSocketMessage::Close,
            0x9 => WebSocketMessage::Ping(f.payload().to_vec()),
            0xA => WebSocketMessage::Pong(f.payload().to_vec()),
            _ => WebSocketMessage::Text(String::from_utf8_lossy(f.payload()).to_string()),
        }))
    }
    
    fn extract_host(&self) -> Result<String, Box<dyn std::error::Error>> {
        // 简化实现
        Ok("localhost".to_string())
    }
    
    fn extract_path(&self) -> Result<String, Box<dyn std::error::Error>> {
        // 简化实现
        Ok("/".to_string())
    }
}

fn generate_random_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut key = Vec::with_capacity(16);
    for _ in 0..16 {
        key.push(rng.gen::<u8>());
    }
    general_purpose::STANDARD.encode(&key)
}

#[cfg(test)]
mod websocket_tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_websocket_echo() {
        // 这是一个端到端测试，需要服务器和客户端
        // 实际测试中需要启动WebSocket服务器和客户端
    }
}
```

## 10.5 序列化技术

### 10.5.1 JSON序列化

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value, Map};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn error(message: String) -> ApiResponse<T> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
            timestamp: chrono::Utc::now(),
        }
    }
}

pub struct JsonSerializer {
    pretty_print: bool,
    null_value_handling: bool,
}

impl JsonSerializer {
    pub fn new() -> Self {
        JsonSerializer {
            pretty_print: false,
            null_value_handling: true,
        }
    }
    
    pub fn with_pretty_print(mut self, enable: bool) -> Self {
        self.pretty_print = enable;
        self
    }
    
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<String, Box<dyn std::error::Error>> {
        if self.pretty_print {
            Ok(serde_json::to_string_pretty(data)?)
        } else {
            Ok(serde_json::to_string(data)?)
        }
    }
    
    pub fn deserialize<T: DeserializeOwned>(&self, json: &str) -> Result<T, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(json)?)
    }
    
    pub fn serialize_to_bytes<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(serde_json::to_vec(data)?)
    }
    
    pub fn deserialize_from_bytes<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

// 自定义序列化器用于处理复杂的JSON结构
pub struct DynamicJsonHandler {
    serializer: JsonSerializer,
}

impl DynamicJsonHandler {
    pub fn new() -> Self {
        DynamicJsonHandler {
            serializer: JsonSerializer::new(),
        }
    }
    
    pub fn create_nested_object(&self, fields: Vec<(String, Value)>) -> Value {
        let mut map = Map::new();
        for (key, value) in fields {
            map.insert(key, value);
        }
        Value::Object(map)
    }
    
    pub fn create_array(&self, items: Vec<Value>) -> Value {
        Value::Array(items)
    }
    
    pub fn transform_json(&self, input: &Value, transformer: fn(&Value) -> Value) -> Result<String, Box<dyn std::error::Error>> {
        let transformed = transformer(input);
        Ok(self.serializer.serialize(&transformed)?)
    }
}

use serde::de::DeserializeOwned;

#[cfg(test)]
mod json_tests {
    use super::*;
    
    #[test]
    fn test_user_serialization() {
        let user = User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        let serializer = JsonSerializer::new();
        let json = serializer.serialize(&user).unwrap();
        let deserialized: User = serializer.deserialize(&json).unwrap();
        
        assert_eq!(user.id, deserialized.id);
        assert_eq!(user.name, deserialized.name);
        assert_eq!(user.email, deserialized.email);
    }
    
    #[test]
    fn test_api_response() {
        let response = ApiResponse::success("Hello, World!");
        let json = serde_json::to_string(&response).unwrap();
        let parsed: ApiResponse<String> = serde_json::from_str(&json).unwrap();
        
        assert!(parsed.success);
        assert_eq!(parsed.data, Some("Hello, World!".to_string()));
    }
}
```

### 10.5.2 MessagePack序列化

```rust
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

pub struct MessagePackSerializer;

impl MessagePackSerializer {
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        data.serialize(&mut Serializer::new(&mut buf).with_bin_config()
            .with_struct_map()
            .with_human_readable())?;
        Ok(buf)
    }
    
    pub fn deserialize<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        let mut deserializer = Deserializer::new(data);
        Ok(T::deserialize(&mut deserializer)?)
    }
    
    pub fn serialize_to_writer<T: Serialize, W: std::io::Write>(
        &self,
        data: &T,
        writer: W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut serializer = Serializer::new(writer)
            .with_bin_config()
            .with_struct_map()
            .with_human_readable();
        data.serialize(&mut serializer)?;
        Ok(())
    }
    
    pub fn deserialize_from_reader<T: DeserializeOwned, R: std::io::Read>(
        &self,
        reader: R,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let mut deserializer = Deserializer::new(reader);
        Ok(T::deserialize(&mut deserializer)?)
    }
}

// 高性能二进制协议
pub struct BinaryProtocol {
    serializer: MessagePackSerializer,
    compression: bool,
}

impl BinaryProtocol {
    pub fn new() -> Self {
        BinaryProtocol {
            serializer: MessagePackSerializer,
            compression: false,
        }
    }
    
    pub fn with_compression(mut self, enable: bool) -> Self {
        self.compression = enable;
        self
    }
    
    pub fn encode_message<T: Serialize>(&self, message_id: u16, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let payload = self.serializer.serialize(data)?;
        let payload = if self.compression {
            self.compress_data(&payload)?
        } else {
            payload
        };
        
        // 创建消息头
        let mut message = vec![0u8; 4 + payload.len()];
        
        // 消息ID (2字节)
        message[0] = (message_id >> 8) as u8;
        message[1] = (message_id & 0xFF) as u8;
        
        // 消息长度 (2字节)
        let length = payload.len() as u16;
        message[2] = (length >> 8) as u8;
        message[3] = (length & 0xFF) as u8;
        
        // 负载数据
        message[4..].copy_from_slice(&payload);
        
        Ok(message)
    }
    
    pub fn decode_message<T: DeserializeOwned>(&self, data: &[u8]) -> Result<(u16, T), Box<dyn std::error::Error>> {
        if data.len() < 4 {
            return Err("Message too short".into());
        }
        
        // 解析消息头
        let message_id = ((data[0] as u16) << 8) | (data[1] as u16);
        let length = ((data[2] as u16) << 8) | (data[3] as u16);
        
        if data.len() < 4 + length as usize {
            return Err("Message length mismatch".into());
        }
        
        let payload = &data[4..4 + length as usize];
        let payload = if self.compression {
            self.decompress_data(payload)?
        } else {
            payload.to_vec()
        };
        
        let data = self.serializer.deserialize(&payload)?;
        Ok((message_id, data))
    }
    
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 使用flate2进行压缩
        useflate2::{Compression, Compress, Decompress};
        
        let mut compressed = Vec::new();
        let mut encoder = Compress::new(Compression::fast(), true);
        
        let status = encoder.compress_vec(data, &mut compressed)?;
        if status != flate2::Status::StreamEnd {
            return Err("Compression failed".into());
        }
        
        Ok(compressed)
    }
    
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use flate2::Decompress;
        
        let mut decompressed = Vec::new();
        let mut decoder = Decompress::new(true);
        
        let status = decoder.decompress_vec(data, &mut decompressed)?;
        if status != flate2::Status::StreamEnd && status != flate2::Status::BufExhausted {
            return Err("Decompression failed".into());
        }
        
        Ok(decompressed)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkMessage {
    pub id: u16,
    pub message_type: MessageType,
    pub data: MessageData,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub sequence_number: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum MessageType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "binary")]
    Binary,
    #[serde(rename = "command")]
    Command,
    #[serde(rename = "notification")]
    Notification,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "data_type")]
pub enum MessageData {
    #[serde(rename = "string")]
    StringData { value: String },
    #[serde(rename = "bytes")]
    BinaryData { value: Vec<u8> },
    #[serde(rename = "number")]
    NumberData { value: f64 },
    #[serde(rename = "object")]
    ObjectData { value: Map<String, Value> },
}

#[cfg(test)]
mod msgpack_tests {
    use super::*;
    
    #[test]
    fn test_msgpack_serialization() {
        let message = NetworkMessage {
            id: 1,
            message_type: MessageType::Text,
            data: MessageData::StringData { 
                value: "Hello, MessagePack!".to_string() 
            },
            timestamp: chrono::Utc::now(),
            sequence_number: 1,
        };
        
        let serializer = MessagePackSerializer;
        let bytes = serializer.serialize(&message).unwrap();
        let deserialized: NetworkMessage = serializer.deserialize(&bytes).unwrap();
        
        assert_eq!(message.id, deserialized.id);
        assert_eq!(message.sequence_number, deserialized.sequence_number);
    }
    
    #[test]
    fn test_binary_protocol() {
        let protocol = BinaryProtocol::new();
        let message = "Test message".to_string();
        
        let encoded = protocol.encode_message(1, &message).unwrap();
        let (message_id, decoded): (u16, String) = protocol.decode_message(&encoded).unwrap();
        
        assert_eq!(message_id, 1);
        assert_eq!(message, decoded);
    }
}
```

## 10.6 企业级分布式聊天系统

现在我们来构建一个完整的分布式聊天系统，集成所有学到的网络编程技术。

```rust
// 分布式聊天系统主项目文件
// File: chat-system/Cargo.toml
/*
[package]
name = "distributed-chat-system"
version = "1.0.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rmp-serde = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
tokio-rustls = "0.23"
rustls = "0.21"
clap = { version = "4.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }
redis = { version = "0.23", features = ["tokio-comp"] }
*/

// 消息定义模块
// File: chat-system/src/messages.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "typing")]
    Typing,
    #[serde(rename = "presence")]
    Presence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    #[serde(rename = "sending")]
    Sending,
    #[serde(rename = "sent")]
    Sent,
    #[serde(rename = "delivered")]
    Delivered,
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_online: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub room_type: RoomType,
    pub is_private: bool,
    pub members: Vec<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomType {
    Private,
    Group,
    Public,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender_id: Uuid,
    pub message_type: MessageType,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub reply_to: Option<Uuid>,
    pub status: MessageStatus,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub delivered_to: Vec<Uuid>,
    pub read_by: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub id: Uuid,
    pub message_type: WebSocketMessageType,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketMessageType {
    Connect { user_id: Uuid },
    Disconnect { user_id: Uuid },
    JoinRoom { room_id: Uuid, user_id: Uuid },
    LeaveRoom { room_id: Uuid, user_id: Uuid },
    SendMessage(ChatMessage),
    MessageStatus { message_id: Uuid, status: MessageStatus },
    Typing { room_id: Uuid, user_id: Uuid, is_typing: bool },
    Presence { user_id: Uuid, is_online: bool },
    Error { code: String, message: String },
    Pong,
}

// 协议定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub version: String,
    pub message_id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub data: MessageData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageData {
    AuthRequest { username: String, password_hash: String },
    AuthResponse { success: bool, token: Option<String>, user: Option<User> },
    RegisterRequest { username: String, email: String, password_hash: String },
    RoomListRequest,
    RoomListResponse { rooms: Vec<Room> },
    JoinRoomRequest { room_id: Uuid },
    LeaveRoomRequest { room_id: Uuid },
    SendMessageRequest { room_id: Uuid, content: String, message_type: MessageType },
    MessageReceived { message: ChatMessage },
    UserListRequest,
    UserListResponse { users: Vec<User> },
    Heartbeat,
    HeartbeatResponse,
    Error { code: String, message: String, details: Option<serde_json::Value> },
}

// HTTP API 结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub request_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub description: Option<String>,
    pub room_type: RoomType,
    pub is_private: bool,
    pub members: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<Uuid>,
    pub metadata: HashMap<String, serde_json::Value>,
}

// 错误定义
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ChatError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        ChatError {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

// 常用的错误类型
pub const ERROR_USER_NOT_FOUND: &str = "USER_NOT_FOUND";
pub const ERROR_ROOM_NOT_FOUND: &str = "ROOM_NOT_FOUND";
pub const ERROR_MESSAGE_NOT_FOUND: &str = "MESSAGE_NOT_FOUND";
pub const ERROR_UNAUTHORIZED: &str = "UNAUTHORIZED";
pub const ERROR_FORBIDDEN: &str = "FORBIDDEN";
pub const ERROR_INVALID_MESSAGE: &str = "INVALID_MESSAGE";
pub const ERROR_RATE_LIMIT: &str = "RATE_LIMIT";
pub const ERROR_USER_OFFLINE: &str = "USER_OFFLINE";
```

```rust
// WebSocket服务器模块
// File: chat-system/src/websocket_server.rs
use super::messages::*;
use super::database::Database;
use super::redis_cache::RedisCache;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, broadcast, oneshot, Arc, Mutex};
use tokio::time::{timeout, Duration};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{info, warn, error, instrument};
use uuid::Uuid;

#[derive(Clone)]
pub struct WebSocketServerConfig {
    pub max_connections: usize,
    pub heartbeat_interval: Duration,
    pub message_buffer_size: usize,
    pub rate_limit_per_minute: u32,
}

impl Default for WebSocketServerConfig {
    fn default() -> Self {
        WebSocketServerConfig {
            max_connections: 10000,
            heartbeat_interval: Duration::from_secs(30),
            message_buffer_size: 1000,
            rate_limit_per_minute: 100,
        }
    }
}

pub struct ConnectedClient {
    pub user_id: Uuid,
    pub stream: TcpStream,
    pub last_heartbeat: std::time::Instant,
    pub rooms: HashMap<Uuid, broadcast::Receiver<ChatMessage>>,
    pub rate_limiter: RateLimiter,
    pub sequence: Arc<AtomicU64>,
}

pub struct RateLimiter {
    messages: Arc<Mutex<Vec<std::time::Instant>>>,
    max_per_minute: u32,
}

impl RateLimiter {
    pub fn new(max_per_minute: u32) -> Self {
        RateLimiter {
            messages: Arc::        RateLimiter {
            messages: Arc::new(Mutex::new(Vec::new())),
            max_per_minute,
        }
    }
    
    pub async fn allow_message(&self) -> bool {
        let mut messages = self.messages.lock().await;
        let now = std::time::Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);
        
        // 清理一分钟前的消息
        messages.retain(|&time| time > one_minute_ago);
        
        if messages.len() < self.max_per_minute as usize {
            messages.push(now);
            true
        } else {
            false
        }
    }
}

pub struct WebSocketServer {
    config: WebSocketServerConfig,
    clients: Arc<Mutex<HashMap<Uuid, ConnectedClient>>>,
    database: Database,
    redis: RedisCache,
    message_sender: broadcast::Sender<WebSocketMessage>,
    shutdown: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl WebSocketServer {
    pub fn new(
        config: WebSocketServerConfig,
        database: Database,
        redis: RedisCache,
    ) -> Self {
        let (message_sender, _) = broadcast::channel(config.message_buffer_size);
        
        WebSocketServer {
            config,
            clients: Arc::new(Mutex::new(HashMap::new())),
            database,
            redis,
            message_sender,
            shutdown: Arc::new(Mutex::new(None)),
        }
    }
    
    #[instrument(skip(self))]
    pub async fn run(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        info!("WebSocket server listening on {}", addr);
        
        // 启动后台任务
        let shutdown_signal = self.start_background_tasks().await?;
        
        // 启动关闭处理
        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            shutdown.lock().await.as_ref().unwrap().send(()).ok();
        });
        
        // 处理连接
        loop {
            match timeout(Duration::from_secs(1), listener.accept()).await {
                Ok(Ok((stream, peer_addr))) => {
                    info!("New WebSocket connection from {}", peer_addr);
                    
                    let clients = Arc::clone(&self.clients);
                    let server = self.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream, peer_addr).await {
                            error!("Connection handling error: {}", e);
                        }
                    });
                }
                Ok(Err(e)) => {
                    warn!("Failed to accept connection: {}", e);
                }
                Err(_) => {
                    // 超时检查关闭信号
                    let shutdown_rx = shutdown_signal.clone();
                    if let Some(mut rx) = shutdown_rx {
                        if let Ok(_) = rx.try_recv() {
                            info!("Shutting down WebSocket server");
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn start_background_tasks(&self) -> Result<oneshot::Receiver<()>, Box<dyn std::error::Error>> {
        let (tx, rx) = oneshot::channel();
        *self.shutdown.lock().await = Some(tx);
        
        // 启动心跳任务
        {
            let clients = Arc::clone(&self.clients);
            let config = self.config.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(config.heartbeat_interval);
                loop {
                    interval.tick().await;
                    
                    let mut disconnected = Vec::new();
                    {
                        let clients_guard = clients.lock().await;
                        for (user_id, client) in clients_guard.iter() {
                            if client.last_heartbeat.elapsed() > config.heartbeat_interval * 2 {
                                disconnected.push(*user_id);
                            }
                        }
                    }
                    
                    for user_id in disconnected {
                        let mut clients = clients.lock().await;
                        if let Some(client) = clients.remove(&user_id) {
                            info!("Client {} timed out", user_id);
                            
                            // 更新用户状态
                            drop(client);
                        }
                    }
                }
            });
        }
        
        // 启动消息分发任务
        {
            let message_rx = self.message_sender.subscribe();
            let clients = Arc::clone(&self.clients);
            tokio::spawn(async move {
                let mut message_rx = message_rx;
                loop {
                    if let Ok(message) = message_rx.recv().await {
                        let clients = clients.lock().await;
                        
                        match &message.message_type {
                            WebSocketMessageType::SendMessage(msg) => {
                                // 分发消息给房间成员
                                let room_members = clients.values()
                                    .filter(|client| client.rooms.contains_key(&msg.room_id))
                                    .collect::<Vec<_>>();
                                
                                for client in room_members {
                                    // 发送消息到客户端的消息通道
                                    // 实现细节...
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });
        }
        
        Ok(rx)
    }
    
    #[instrument(skip(self, stream))]
    async fn handle_connection(&self, stream: TcpStream, peer_addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        // 读取WebSocket握手请求
        let (mut reader, mut writer) = stream.into_split();
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer).await?;
        
        // 解析握手
        let (key, response_key) = parse_websocket_handshake(&buffer)?;
        
        // 验证用户身份（简化版）
        // 实际实现中应该验证JWT token或session
        let user_id = self.authenticate_user(&buffer).await?;
        
        if user_id.is_none() {
            return Err("Authentication failed".into());
        }
        
        let user_id = user_id.unwrap();
        
        // 发送握手响应
        let handshake_response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\
             \r\n",
            response_key
        );
        
        writer.write_all(handshake_response.as_bytes()).await?;
        
        // 创建客户端实例
        let client = ConnectedClient {
            user_id,
            stream: TcpStream::from_std(writer.into_inner())?,
            last_heartbeat: std::time::Instant::now(),
            rooms: HashMap::new(),
            rate_limiter: RateLimiter::new(self.config.rate_limit_per_minute),
            sequence: Arc::new(AtomicU64::new(0)),
        };
        
        // 添加到客户端集合
        {
            let mut clients = self.clients.lock().await;
            if clients.len() >= self.config.max_connections {
                return Err("Maximum connections reached".into());
            }
            clients.insert(user_id, client);
        }
        
        // 处理客户端消息
        self.handle_client_messages(user_id).await?;
        
        Ok(())
    }
    
    async fn authenticate_user(&self, request: &str) -> Result<Option<Uuid>, Box<dyn std::error::Error>> {
        // 解析认证头或token
        // 这里简化处理，实际应该验证JWT
        if request.contains("Authorization: Bearer valid_token") {
            // 返回示例用户ID
            Ok(Some(Uuid::new_v4()))
        } else {
            Ok(None)
        }
    }
    
    async fn handle_client_messages(&self, user_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let clients = Arc::clone(&self.clients);
        let mut clients_guard = clients.lock().await;
        
        if let Some(client) = clients_guard.get_mut(&user_id) {
            // 处理客户端消息循环
            let mut buffer = vec![0u8; 4096];
            
            loop {
                match WebSocketFrame::read_frame(&mut &client.stream, &mut buffer).await {
                    Ok(Some(frame)) => {
                        client.last_heartbeat = std::time::Instant::now();
                        
                        match frame.opcode() {
                            0x8 => { // Close frame
                                break;
                            }
                            0x9 => { // Ping frame
                                let pong_frame = WebSocketFrame::pong(frame.payload());
                                if let Ok(data) = pong_frame.to_bytes() {
                                    let _ = client.stream.write_all(&data).await;
                                }
                            }
                            0xA => { // Pong frame
                                // 更新最后心跳时间
                            }
                            0x1 | 0x2 => { // Text or Binary frame
                                if let Ok(message) = self.process_message(&user_id, frame.payload()).await {
                                    let _ = self.message_sender.send(message);
                                }
                            }
                            _ => {
                                warn!("Unknown frame opcode: {}", frame.opcode());
                            }
                        }
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(e) => {
                        error!("Frame read error: {}", e);
                        break;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn process_message(&self, user_id: &Uuid, payload: &[u8]) -> Result<WebSocketMessage, Box<dyn std::error::Error>> {
        // 解析客户端消息
        let protocol_message: ProtocolMessage = serde_json::from_slice(payload)?;
        
        match &protocol_message.data {
            MessageData::AuthRequest { .. } => {
                // 处理认证请求
                self.handle_auth_request(user_id, &protocol_message).await
            }
            MessageData::JoinRoomRequest { room_id } => {
                self.handle_join_room(user_id, room_id, &protocol_message).await
            }
            MessageData::LeaveRoomRequest { room_id } => {
                self.handle_leave_room(user_id, room_id, &protocol_message).await
            }
            MessageData::SendMessageRequest { room_id, content, message_type } => {
                self.handle_send_message(user_id, room_id, content, message_type, &protocol_message).await
            }
            MessageData::Heartbeat => {
                Ok(WebSocketMessage {
                    id: Uuid::new_v4(),
                    message_type: WebSocketMessageType::Pong,
                    data: serde_json::Value::Null,
                    timestamp: Utc::now(),
                    sequence: 0,
                })
            }
            _ => {
                Ok(WebSocketMessage {
                    id: Uuid::new_v4(),
                    message_type: WebSocketMessageType::Error { 
                        code: "UNKNOWN_MESSAGE".to_string(), 
                        message: "Unknown message type".to_string() 
                    },
                    data: serde_json::Value::Null,
                    timestamp: Utc::now(),
                    sequence: 0,
                })
            }
        }
    }
    
    async fn handle_auth_request(
        &self,
        user_id: &Uuid,
        protocol_message: &ProtocolMessage,
    ) -> Result<WebSocketMessage, Box<dyn std::error::Error>> {
        // 从数据库获取用户信息
        let user = self.database.get_user(user_id).await?;
        
        match user {
            Some(user) => {
                // 发送认证成功响应
                let response = MessageData::AuthResponse {
                    success: true,
                    token: Some("jwt_token_here".to_string()),
                    user: Some(user),
                };
                
                Ok(WebSocketMessage {
                    id: protocol_message.message_id,
                    message_type: WebSocketMessageType::Connect { user_id: *user_id },
                    data: serde_json::to_value(response)?,
                    timestamp: Utc::now(),
                    sequence: 0,
                })
            }
            None => {
                let error = MessageData::Error {
                    code: ERROR_USER_NOT_FOUND.to_string(),
                    message: "User not found".to_string(),
                    details: None,
                };
                
                Ok(WebSocketMessage {
                    id: protocol_message.message_id,
                    message_type: WebSocketMessageType::Error { 
                        code: ERROR_USER_NOT_FOUND.to_string(), 
                        message: "User not found".to_string() 
                    },
                    data: serde_json::to_value(error)?,
                    timestamp: Utc::now(),
                    sequence: 0,
                })
            }
        }
    }
    
    async fn handle_join_room(
        &self,
        user_id: &Uuid,
        room_id: &Uuid,
        protocol_message: &ProtocolMessage,
    ) -> Result<WebSocketMessage, Box<dyn std::error::Error>> {
        // 检查用户是否有权限加入房间
        let has_permission = self.database.check_room_permission(user_id, room_id).await?;
        
        if !has_permission {
            let error = MessageData::Error {
                code: ERROR_FORBIDDEN.to_string(),
                message: "Permission denied".to_string(),
                details: None,
            };
            
            return Ok(WebSocketMessage {
                id: protocol_message.message_id,
                message_type: WebSocketMessageType::Error { 
                    code: ERROR_FORBIDDEN.to_string(), 
                    message: "Permission denied".to_string() 
                },
                data: serde_json::to_value(error)?,
                timestamp: Utc::now(),
                sequence: 0,
            });
        }
        
        // 添加用户到房间
        self.database.add_user_to_room(user_id, room_id).await?;
        
        // 通知其他房间成员
        let join_notification = WebSocketMessage {
            id: Uuid::new_v4(),
            message_type: WebSocketMessageType::JoinRoom { 
                room_id: *room_id, 
                user_id: *user_id 
            },
            data: serde_json::Value::Null,
            timestamp: Utc::now(),
            sequence: 0,
        };
        
        self.message_sender.send(join_notification)?;
        
        Ok(WebSocketMessage {
            id: protocol_message.message_id,
            message_type: WebSocketMessageType::JoinRoom { 
                room_id: *room_id, 
                user_id: *user_id 
            },
            data: serde_json::Value::Null,
            timestamp: Utc::now(),
            sequence: 0,
        })
    }
    
    async fn handle_leave_room(
        &self,
        user_id: &Uuid,
        room_id: &Uuid,
        protocol_message: &ProtocolMessage,
    ) -> Result<WebSocketMessage, Box<dyn std::error::Error>> {
        // 从房间移除用户
        self.database.remove_user_from_room(user_id, room_id).await?;
        
        // 通知其他房间成员
        let leave_notification = WebSocketMessage {
            id: Uuid::new_v4(),
            message_type: WebSocketMessageType::LeaveRoom { 
                room_id: *room_id, 
                user_id: *user_id 
            },
            data: serde_json::Value::Null,
            timestamp: Utc::now(),
            sequence: 0,
        };
        
        self.message_sender.send(leave_notification)?;
        
        Ok(WebSocketMessage {
            id: protocol_message.message_id,
            message_type: WebSocketMessageType::LeaveRoom { 
                room_id: *room_id, 
                user_id: *user_id 
            },
            data: serde_json::Value::Null,
            timestamp: Utc::now(),
            sequence: 0,
        })
    }
    
    async fn handle_send_message(
        &self,
        user_id: &Uuid,
        room_id: &Uuid,
        content: &str,
        message_type: &MessageType,
        protocol_message: &ProtocolMessage,
    ) -> Result<WebSocketMessage, Box<dyn std::error::Error>> {
        // 创建消息
        let message = ChatMessage {
            id: Uuid::new_v4(),
            room_id: *room_id,
            sender_id: *user_id,
            message_type: message_type.clone(),
            content: content.to_string(),
            metadata: HashMap::new(),
            reply_to: None,
            status: MessageStatus::Sending,
            created_at: Utc::now(),
            edited_at: None,
            delivered_to: vec![],
            read_by: vec![*user_id],
        };
        
        // 保存到数据库
        self.database.save_message(&message).await?;
        
        // 更新消息状态
        let mut message = message;
        message.status = MessageStatus::Sent;
        message.delivered_to = self.database.get_room_members(room_id).await?;
        
        // 发送消息给房间成员
        let ws_message = WebSocketMessage {
            id: Uuid::new_v4(),
            message_type: WebSocketMessageType::SendMessage(message.clone()),
            data: serde_json::Value::Null,
            timestamp: Utc::now(),
            sequence: 0,
        };
        
        self.message_sender.send(ws_message)?;
        
        Ok(WebSocketMessage {
            id: protocol_message.message_id,
            message_type: WebSocketMessageType::SendMessage(message),
            data: serde_json::Value::Null,
            timestamp: Utc::now(),
            sequence: 0,
        })
    }
}

impl Clone for WebSocketServer {
    fn clone(&self) -> Self {
        WebSocketServer {
            config: self.config.clone(),
            clients: Arc::clone(&self.clients),
            database: self.database.clone(),
            redis: self.redis.clone(),
            message_sender: self.message_sender.clone(),
            shutdown: Arc::clone(&self.shutdown),
        }
    }
}
```

```rust
// HTTP API服务器模块
// File: chat-system/src/http_server.rs
use super::messages::*;
use super::websocket_server::WebSocketServer;
use super::database::Database;
use super::redis_cache::RedisCache;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error, instrument};

pub struct HttpServer {
    listener: TcpListener,
    database: Database,
    redis: RedisCache,
    websocket_server: WebSocketServer,
}

impl HttpServer {
    pub fn new(
        database: Database,
        redis: RedisCache,
        websocket_server: WebSocketServer,
    ) -> Self {
        HttpServer {
            listener: TcpListener::bind("0.0.0.0:8080").await.unwrap(),
            database,
            redis,
            websocket_server,
        }
    }
    
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("HTTP API server starting on port 8080");
        
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New HTTP request from {}", addr);
                    
                    let database = self.database.clone();
                    let redis = self.redis.clone();
                    
                    tokio::spawn(async move {
                        handle_http_request(stream, database, redis).await
                    });
                }
                Err(e) => {
                    warn!("Failed to accept HTTP connection: {}", e);
                }
            }
        }
    }
}

#[instrument(skip(stream, database, redis))]
async fn handle_http_request(
    stream: TcpStream,
    database: Database,
    redis: RedisCache,
) {
    let (mut reader, mut writer) = stream.into_split();
    let mut buffer = String::new();
    
    if let Err(e) = reader.read_to_string(&mut buffer).await {
        error!("Failed to read HTTP request: {}", e);
        return;
    }
    
    match parse_http_request(&buffer) {
        Ok((method, path, headers, body)) => {
            match handle_route(&method, &path, &headers, &body, &database, &redis).await {
                Ok(response) => {
                    if let Err(e) = writer.write_all(response.as_bytes()).await {
                        error!("Failed to send HTTP response: {}", e);
                    }
                }
                Err(e) => {
                    error!("Route handling error: {}", e);
                    let error_response = create_error_response(500, &format!("Internal server error: {}", e));
                    let _ = writer.write_all(error_response.as_bytes()).await;
                }
            }
        }
        Err(e) => {
            error!("Failed to parse HTTP request: {}", e);
            let error_response = create_error_response(400, "Bad request");
            let _ = writer.write_all(error_response.as_bytes()).await;
        }
    }
}

fn parse_http_request(request: &str) -> Result<(String, String, HashMap<String, String>, String), Box<dyn std::error::Error>> {
    let lines: Vec<&str> = request.split("\r\n").collect();
    
    if lines.is_empty() {
        return Err("Empty request".into());
    }
    
    // 解析请求行
    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split(' ').collect();
    if parts.len() != 3 {
        return Err("Invalid request line".into());
    }
    
    let method = parts[0].to_string();
    let path = parts[1].to_string();
    
    // 解析头部
    let mut headers = HashMap::new();
    let mut body = String::new();
    let mut in_headers = true;
    
    for line in &lines[1..] {
        if line.is_empty() {
            in_headers = false;
            continue;
        }
        
        if in_headers {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        } else {
            body.push_str(line);
        }
    }
    
    Ok((method, path, headers, body))
}

async fn handle_route(
    method: &str,
    path: &str,
    headers: &HashMap<String, String>,
    body: &str,
    database: &Database,
    redis: &RedisCache,
) -> Result<String, Box<dyn std::error::Error>> {
    let content_type = headers.get("Content-Type").unwrap_or(&"application/json".to_string());
    
    match (method, path) {
        ("GET", "/") => Ok(create_json_response(200, json!({
            "message": "Distributed Chat System API",
            "version": "1.0.0",
            "endpoints": ["/api/users", "/api/rooms", "/api/messages"]
        }))),
        
        ("GET", "/health") => Ok(create_json_response(200, json!({
            "status": "healthy",
            "timestamp": Utc::now().to_rfc3339()
        }))),
        
        // 用户管理
        ("GET", "/api/users") => {
            let users = database.get_all_users().await?;
            Ok(create_json_response(200, ApiResponse {
                success: true,
                data: Some(users),
                error: None,
                timestamp: Utc::now(),
                request_id: Uuid::new_v4(),
            }))
        },
        
        ("POST", "/api/users") if content_type == "application/json" => {
            let user_data: CreateUserRequest = serde_json::from_str(body)?;
            let user = database.create_user(&user_data).await?;
            Ok(create_json_response(201, ApiResponse {
                success: true,
                data: Some(user),
                error: None,
                timestamp: Utc::now(),
                request_id: Uuid::new_v4(),
            }))
        },
        
        ("GET", path) if path.starts_with("/api/users/") => {
            let user_id = path.trim_start_matches("/api/users/");
            if let Ok(uuid) = Uuid::parse_str(user_id) {
                if let Some(user) = database.get_user(&uuid).await? {
                    Ok(create_json_response(200, ApiResponse {
                        success: true,
                        data: Some(user),
                        error: None,
                        timestamp: Utc::now(),
                        request_id: Uuid::new_v4(),
                    }))
                } else {
                    Ok(create_error_response(404, "User not found"))
                }
            } else {
                Ok(create_error_response(400, "Invalid user ID"))
            }
        },
        
        // 房间管理
        ("GET", "/api/rooms") => {
            let rooms = database.get_all_rooms().await?;
            Ok(create_json_response(200, ApiResponse {
                success: true,
                data: Some(rooms),
                error: None,
                timestamp: Utc::now(),
                request_id: Uuid::new_v4(),
            }))
        },
        
        ("POST", "/api/rooms") if content_type == "application/json" => {
            let room_data: CreateRoomRequest = serde_json::from_str(body)?;
            let room = database.create_room(&room_data).await?;
            Ok(create_json_response(201, ApiResponse {
                success: true,
                data: Some(room),
                error: None,
                timestamp: Utc::now(),
                request_id: Uuid::new_v4(),
            }))
        },
        
        ("GET", path) if path.starts_with("/api/rooms/") => {
            let room_id = path.trim_start_matches("/api/rooms/");
            if let Ok(uuid) = Uuid::parse_str(room_id) {
                if let Some(room) = database.get_room(&uuid).await? {
                    Ok(create_json_response(200, ApiResponse {
                        success: true,
                        data: Some(room),
                        error: None,
                        timestamp: Utc::now(),
                        request_id: Uuid::new_v4(),
                    }))
                } else {
                    Ok(create_error_response(404, "Room not found"))
                }
            } else {
                Ok(create_error_response(400, "Invalid room ID"))
            }
        },
        
        // 消息管理
        ("GET", path) if path.starts_with("/api/messages/") && path.contains("/history") => {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 5 {
                let room_id = Uuid::parse_str(parts[3])?;
                let limit = parts[5].parse::<u32>().unwrap_or(50);
                
                let messages = database.get_room_messages(&room_id, limit).await?;
                Ok(create_json_response(200, ApiResponse {
                    success: true,
                    data: Some(messages),
                    error: None,
                    timestamp: Utc::now(),
                    request_id: Uuid::new_v4(),
                }))
            } else {
                Ok(create_error_response(400, "Invalid path format"))
            }
        },
        
        ("POST", path) if path.starts_with("/api/messages/") => {
            let room_id = path.trim_start_matches("/api/messages/");
            if let Ok(uuid) = Uuid::parse_str(room_id) {
                let message_data: SendMessageRequest = serde_json::from_str(body)?;
                // 实际实现中需要获取用户ID从认证token
                let user_id = Uuid::new_v4(); // 临时用户ID
                
                let message = ChatMessage {
                    id: Uuid::new_v4(),
                    room_id: uuid,
                    sender_id: user_id,
                    message_type: message_data.message_type,
                    content: message_data.content,
                    metadata: message_data.metadata,
                    reply_to: message_data.reply_to,
                    status: MessageStatus::Sending,
                    created_at: Utc::now(),
                    edited_at: None,
                    delivered_to: vec![],
                    read_by: vec![user_id],
                };
                
                let saved_message = database.save_message(&message).await?;
                Ok(create_json_response(201, ApiResponse {
                    success: true,
                    data: Some(saved_message),
                    error: None,
                    timestamp: Utc::now(),
                    request_id: Uuid::new_v4(),
                }))
            } else {
                Ok(create_error_response(400, "Invalid room ID"))
            }
        },
        
        // WebSocket升级端点
        ("GET", "/ws") => {
            // 这里应该升级到WebSocket连接
            // 实际实现中需要处理WebSocket握手
            Ok("HTTP/1.1 400 Bad Request\r\n\r\nWebSocket upgrade should be handled by WebSocket server".to_string())
        },
        
        _ => Ok(create_error_response(404, "Not found")),
    }
}

fn create_json_response(status: u16, data: impl Serialize) -> String {
    let json = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());
    format!(
        "HTTP/1.1 {} OK\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        status,
        json.len(),
        json
    )
}

fn create_error_response(status: u16, message: &str) -> String {
    let error = json!({
        "success": false,
        "error": message,
        "timestamp": Utc::now().to_rfc3339()
    });
    let json = serde_json::to_string(&error).unwrap_or_else(|_| "{}".to_string());
    format!(
        "HTTP/1.1 {} Bad Request\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        status,
        json.len(),
        json
    )
}

// 请求结构定义
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub description: Option<String>,
    pub room_type: RoomType,
    pub is_private: bool,
    pub members: Vec<Uuid>,
}
```

```rust
// 数据库模块
// File: chat-system/src/database.rs
use super::messages::*;
use sqlx::{PgPool, Row};
use tracing::{info, warn, error, instrument};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPool::connect(database_url).await?;
        
        // 运行数据库迁移
        Self::run_migrations(&pool).await?;
        
        Ok(Database { pool })
    }
    
    async fn run_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
        // 创建用户表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                username VARCHAR(50) UNIQUE NOT NULL,
                display_name VARCHAR(100) NOT NULL,
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                avatar_url TEXT,
                is_online BOOLEAN DEFAULT FALSE,
                last_seen TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
        "#).execute(pool).await?;
        
        // 创建房间表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS rooms (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(100) NOT NULL,
                description TEXT,
                room_type VARCHAR(20) NOT NULL DEFAULT 'group',
                is_private BOOLEAN DEFAULT FALSE,
                created_by UUID NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                FOREIGN KEY (created_by) REFERENCES users(id)
            )
        "#).execute(pool).await?;
        
        // 创建房间成员表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS room_members (
                room_id UUID NOT NULL,
                user_id UUID NOT NULL,
                role VARCHAR(20) DEFAULT 'member',
                joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                PRIMARY KEY (room_id, user_id),
                FOREIGN KEY (room_id) REFERENCES rooms(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
        "#).execute(pool).await?;
        
        // 创建消息表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS messages (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                room_id UUID NOT NULL,
                sender_id UUID NOT NULL,
                message_type VARCHAR(20) NOT NULL DEFAULT 'text',
                content TEXT NOT NULL,
                metadata JSONB DEFAULT '{}',
                reply_to UUID,
                status VARCHAR(20) DEFAULT 'sending',
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                edited_at TIMESTAMP WITH TIME ZONE,
                FOREIGN KEY (room_id) REFERENCES rooms(id) ON DELETE CASCADE,
                FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (reply_to) REFERENCES messages(id) ON DELETE SET NULL
            )
        "#).execute(pool).await?;
        
        // 创建消息投递状态表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS message_delivery (
                message_id UUID NOT NULL,
                user_id UUID NOT NULL,
                delivered_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                read_at TIMESTAMP WITH TIME ZONE,
                PRIMARY KEY (message_id, user_id),
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
        "#).execute(pool).await?;
        
        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)").execute(pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_room_id ON messages(room_id)").execute(pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at)").execute(pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_members_room_id ON room_members(room_id)").execute(pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_members_user_id ON room_members(user_id)").execute(pool).await?;
        
        info!("Database migrations completed");
        Ok(())
    }
    
    // 用户管理
    #[instrument(skip(self))]
    pub async fn create_user(&self, user_data: &CreateUserRequest) -> Result<User, Box<dyn std::error::Error>> {
        let user = sqlx::query_as!(User,
            r#"
            INSERT INTO users (username, display_name, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, username, display_name, avatar_url, is_online, last_seen, created_at
            "#,
            user_data.username,
            user_data.display_name,
            user_data.email,
            user_data.password_hash
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    #[instrument(skip(self))]
    pub async fn get_user(&self, user_id: &Uuid) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let user = sqlx::query_as!(User,
            "SELECT id, username, display_name, avatar_url, is_online, last_seen, created_at FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    #[instrument(skip(self))]
    pub async fn get_all_users(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let users = sqlx::query_as!(User,
            "SELECT id, username, display_name, avatar_url, is_online, last_seen, created_at FROM users ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(users)
    }
    
    #[instrument(skip(self))]
    pub async fn update_user_online_status(&self, user_id: &Uuid, is_online: bool) -> Result<(), Box<dyn std::error::Error>> {
        let last_seen = if is_online { None } else { Some(Utc::now()) };
        
        sqlx::query!(
            r#"
            UPDATE users 
            SET is_online = $1, last_seen = $2, updated_at = NOW()
            WHERE id = $3
            "#,
            is_online,
            last_seen,
            user_id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // 房间管理
    #[instrument(skip(self))]
    pub async fn create_room(&self, room_data: &CreateRoomRequest) -> Result<Room, Box<dyn std::error::Error>> {
        // 开始事务
        let mut tx = self.pool.begin().await?;
        
        // 创建房间
        let room = sqlx::query_as!(Room,
            r#"
            INSERT INTO rooms (name, description, room_type, is_private, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, description, room_type, is_private, created_by, created_at, last_message_at
            "#,
            room_data.name,
            room_data.description,
            room_data.room_type.to_string(),
            room_data.is_private,
            room_data.created_by
        )
        .fetch_one(&mut *tx)
        .await?;
        
        // 添加创建者为成员
        for member_id in &room_data.members {
            sqlx::query!(
                r#"
                INSERT INTO room_members (room_id, user_id)
                VALUES ($1, $2)
                "#,
                room.id,
                member_id
            )
            .execute(&mut *tx)
            .await?;
        }
        
        // 添加创建者
        sqlx::query!(
            r#"
            INSERT INTO room_members (room_id, user_id)
            VALUES ($1, $2)
            "#,
            room.id,
            room_data.created_by
        )
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        
        Ok(Room {
            id: room.id,
            name: room.name,
            description: room.description,
            room_type: serde_str_to_enum(&room.room_type)?,
            is_private: room.is_private,
            members: room_data.members.clone(),
            created_by: room.created_by,
            created_at: room.created_at,
            last_message_at: room.last_message_at,
        })
    }
    
    #[instrument(skip(self))]
    pub async fn get_room(&self, room_id: &Uuid) -> Result<Option<Room>, Box<dyn std::error::Error>> {
        let room = sqlx::query!(
            r#"
            SELECT r.id, r.name, r.description, r.room_type, r.is_private, r.created_by, r.created_at, r.last_message_at,
                   array_agg(rm.user_id) as members
            FROM rooms r
            LEFT JOIN room_members rm ON r.id = rm.room_id
            WHERE r.id = $1
            GROUP BY r.id, r.name, r.description, r.room_type, r.is_private, r.created_by, r.created_at, r.last_message_at
            "#,
            room_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(room) = room {
            Ok(Some(Room {
                id: room.id,
                name: room.name,
                description: room.description,
                room_type: serde_str_to_enum(&room.room_type)?,
                is_private: room.is_private,
                members: room.members.unwrap_or_default(),
                created_by: room.created_by,
                created_at: room.created_at,
                last_message_at: room.last_message_at,
            }))
        } else {
            Ok(None)
        }
    }
    
    #[instrument(skip(self))]
    pub async fn get_all_rooms(&self) -> Result<Vec<Room>, Box<dyn std::error::Error>> {
        let rooms = sqlx::query!(
            r#"
            SELECT r.id, r.name, r.description, r.room_type, r.is_private, r.created_by, r.created_at, r.last_message_at,
                   array_agg(rm.user_id) as members
            FROM rooms r
            LEFT JOIN room_members rm ON r.id = rm.room_id
            GROUP BY r.id, r.name, r.description, r.room_type, r.is_private, r.created_by, r.created_at, r.last_message_at
            ORDER BY r.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut result = Vec::new();
        for room in rooms {
            result.push(Room {
                id: room.id,
                name: room.name,
                description: room.description,
                room_type: serde_str_to_enum(&room.room_type)?,
                is_private: room.is_private,
                members: room.members.unwrap_or_default(),
                created_by: room.created_by,
                created_at: room.created_at,
                last_message_at: room.last_message_at,
            });
        }
        
        Ok(result)
    }
    
    #[instrument(skip(self))]
    pub async fn get_user_rooms(&self, user_id: &Uuid) -> Result<Vec<Room>, Box<dyn std::error::Error>> {
        let rooms = sqlx::query!(
            r#"
            SELECT r.id, r.name, r.description, r.room_type, r.is_private, r.created_by, r.created_at, r.last_message_at,
                   array_agg(rm.user_id) as members
            FROM rooms r
            INNER JOIN room_members rm ON r.id = rm.room_id
            WHERE rm.user_id = $1
            GROUP BY r.id, r.name, r.description, r.room_type, r.is_private, r.created_by, r.created_at, r.last_message_at
            ORDER BY r.last_message_at DESC NULLS LAST, r.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut result = Vec::new();
        for room in rooms {
            result.push(Room {
                id: room.id,
                name: room.name,
                description: room.description,
                room_type: serde_str_to_enum(&room.room_type)?,
                is_private: room.is_private,
                members: room.members.unwrap_or_default(),
                created_by: room.created_by,
                created_at: room.created_at,
                last_message_at: room.last_message_at,
            });
        }
        
        Ok(result)
    }
    
    // 消息管理
    #[instrument(skip(self))]
    pub async fn save_message(&self, message: &ChatMessage) -> Result<ChatMessage, Box<dyn std::error::Error>> {
        let saved_message = sqlx::query!(
            r#"
            INSERT INTO messages (id, room_id, sender_id, message_type, content, metadata, reply_to, status, created_at, edited_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, room_id, sender_id, message_type, content, metadata, reply_to, status, created_at, edited_at
            "#,
            message.id,
            message.room_id,
            message.sender_id,
            message.message_type.to_string(),
            message.content,
            serde_json::to_value(&message.metadata)?,
            message.reply_to,
            message.status.to_string(),
            message.created_at,
            message.edited_at
        )
        .fetch_one(&self.pool)
        .await?;
        
        // 更新房间最后消息时间
        sqlx::query!(
            "UPDATE rooms SET last_message_at = $1, updated_at = NOW() WHERE id = $2",
            message.created_at,
            message.room_id
        )
        .execute(&self.pool)
        .await?;
        
        // 标记消息已发送给发送者
        sqlx::query!(
            r#"
            INSERT INTO message_delivery (message_id, user_id, delivered_at)
            VALUES ($1, $2, $3)
            "#,
            saved_message.id,
            message.sender_id,
            message.created_at
        )
        .execute(&self.pool)
        .await?;
        
        Ok(ChatMessage {
            id: saved_message.id,
            room_id: saved_message.room_id,
            sender_id: saved_message.sender_id,
            message_type: serde_str_to_enum(&saved_message.message_type)?,
            content: saved_message.content,
            metadata: serde_json::from_value(saved_message.metadata).unwrap_or_default(),
            reply_to: saved_message.reply_to,
            status: serde_str_to_enum(&saved_message.status)?,
            created_at: saved_message.created_at,
            edited_at: saved_message.edited_at,
            delivered_to: vec![message.sender_id],
            read_by: vec![message.sender_id],
        })
    }
    
    #[instrument(skip(self))]
    pub async fn get_room_messages(&self, room_id: &Uuid, limit: u32) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error>> {
        let messages = sqlx::query!(
            r#"
            SELECT m.id, m.room_id, m.sender_id, m.message_type, m.content, m.metadata, m.reply_to, m.status, 
                   m.created_at, m.edited_at,
                   array_agg(DISTINCT md.user_id) FILTER (WHERE md.user_id IS NOT NULL) as delivered_to,
                   array_agg(DISTINCT md2.user_id) FILTER (WHERE md2.read_at IS NOT NULL) as read_by
            FROM messages m
            LEFT JOIN message_delivery md ON m.id = md.message_id
            LEFT JOIN message_delivery md2 ON m.id = md2.message_id AND md2.read_at IS NOT NULL
            WHERE m.room_id = $1
            GROUP BY m.id, m.room_id, m.sender_id, m.message_type, m.content, m.metadata, m.reply_to, m.status, m.created_at, m.edited_at
            ORDER BY m.created_at DESC
            LIMIT $2
            "#,
            room_id,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut result = Vec::new();
        for msg in messages {
            result.push(ChatMessage {
                id: msg.id,
                room_id: msg.room_id,
                sender_id: msg.sender_id,
                message_type: serde_str_to_enum(&msg.message_type)?,
                content: msg.content,
                metadata: serde_json::from_value(msg.metadata).unwrap_or_default(),
                reply_to: msg.reply_to,
                status: serde_str_to_enum(&msg.status)?,
                created_at: msg.created_at,
                edited_at: msg.edited_at,
                delivered_to: msg.delivered_to.unwrap_or_default(),
                read_by: msg.read_by.unwrap_or_default(),
            });
        }
        
        // 反转列表使最新的消息在后面
        result.reverse();
        Ok(result)
    }
    
    // 权限管理
    #[instrument(skip(self))]
    pub async fn check_room_permission(&self, user_id: &Uuid, room_id: &Uuid) -> Result<bool, Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            "SELECT 1 FROM room_members WHERE room_id = $1 AND user_id = $2",
            room_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result.is_some())
    }
    
    #[instrument(skip(self))]
    pub async fn get_room_members(&self, room_id: &Uuid) -> Result<Vec<Uuid>, Box<dyn std::error::Error>> {
        let members = sqlx::query!(
            "SELECT user_id FROM room_members WHERE room_id = $1",
            room_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(members.into_iter().map(|row| row.user_id).collect())
    }
    
    #[instrument(skip(self))]
    pub async fn add_user_to_room(&self, user_id: &Uuid, room_id: &Uuid) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "INSERT INTO room_members (room_id, user_id) VALUES ($1, $2) ON CONFLICT (room_id, user_id) DO NOTHING",
            room_id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn remove_user_from_room(&self, user_id: &Uuid, room_id: &Uuid) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "DELETE FROM room_members WHERE room_id = $1 AND user_id = $2",
            room_id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}

// 辅助函数
fn serde_str_to_enum<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(&format!("\"{}\"", s))?)
}
```

```rust
// Redis缓存模块
// File: chat-system/src/redis_cache.rs
use redis::{Client, Connection, AsyncCommands};
use tracing::{info, warn, error, instrument};
use std::time::Duration;

#[derive(Clone)]
pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_url)?;
        
        // 测试连接
        let mut conn = client.get_async_connection().await?;
        redis::cmd("PING")
            .query_async::<(), String>(&mut conn)
            .await?;
        
        info!("Redis cache connection established");
        Ok(RedisCache { client })
    }
    
    pub async fn get_connection(&self) -> Result<Connection, Box<dyn std::error::Error>> {
        Ok(self.client.get_connection()?)
    }
    
    // 用户在线状态缓存
    #[instrument(skip(self))]
    pub async fn set_user_online(&self, user_id: String, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("user:online:{}", user_id);
        
        redis::cmd("SETEX")
            .arg(&key)
            .arg(duration.as_secs())
            .arg("1")
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn is_user_online(&self, user_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("user:online:{}", user_id);
        
        let result: Option<String> = conn.get(&key).await?;
        Ok(result.is_some())
    }
    
    #[instrument(skip(self))]
    pub async fn set_user_offline(&self, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("user:online:{}", user_id);
        
        redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
    
    // 房间在线成员缓存
    #[instrument(skip(self))]
    pub async fn add_user_to_room_cache(&self, room_id: &str, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("room:online:{}", room_id);
        
        redis::cmd("SADD")
            .arg(&key)
            .arg(user_id)
            .query_async(&mut conn)
            .await?;
        
        // 设置过期时间
        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(3600) // 1小时
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn remove_user_from_room_cache(&self, room_id: &str, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("room:online:{}", room_id);
        
        redis::cmd("SREM")
            .arg(&key)
            .arg(user_id)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn get_room_online_users(&self, room_id: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("room:online:{}", room_id);
        
        let members: Vec<String> = conn.smembers(&key).await?;
        Ok(members)
    }
    
    // 消息缓存
    #[instrument(skip(self))]
    pub async fn cache_message(&self, room_id: &str, message_id: &str, message: &serde_json::Value, ttl: Duration) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("message:{}:{}", room_id, message_id);
        
        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl.as_secs())
            .arg(message.to_string())
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn get_cached_message(&self, room_id: &str, message_id: &str) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("message:{}:{}", room_id, message_id);
        
        let result: Option<String> = conn.get(&key).await?;
        if let Some(json_str) = result {
            Ok(Some(serde_json::from_str(&json_str)?))
        } else {
            Ok(None)
        }
    }
    
    // 房间最近消息缓存
    #[instrument(skip(self))]
    pub async fn cache_recent_messages(&self, room_id: &str, messages: Vec<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("room:recent:{}", room_id);
        
        // 使用Redis列表存储最近的50条消息
        let mut pipeline = redis::pipe();
        
        // 清空旧列表
        pipeline.del(&key);
        
        // 添加新消息
        for message in messages {
            pipeline.lpush(&key, message.to_string());
        }
        
        // 限制列表长度
        pipeline.ltrim(&key, 0, 49);
        
        // 设置过期时间
        pipeline.expire(&key, 3600); // 1小时
        
        let _: () = pipeline.query_async(&mut conn).await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn get_recent_messages(&self, room_id: &str) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("room:recent:{}", room_id);
        
        let messages: Vec<String> = conn.lrange(&key, 0, -1).await?;
        let mut result = Vec::new();
        
        for json_str in messages {
            if let Ok(value) = serde_json::from_str(&json_str) {
                result.push(value);
            }
        }
        
        Ok(result)
    }
    
    // 速率限制
    #[instrument(skip(self))]
    pub async fn check_rate_limit(&self, user_id: &str, action: &str, limit: u32, window: Duration) -> Result<bool, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("rate_limit:{}:{}", user_id, action);
        
        let current: i64 = conn.incr(&key, 1).await?;
        
        if current == 1 {
            // 第一次请求，设置过期时间
            let _: () = conn.expire(&key, window.as_secs() as i64).await?;
        }
        
        Ok(current <= limit as i64)
    }
    
    // 会话管理
    #[instrument(skip(self))]
    pub async fn create_session(&self, user_id: &str, session_data: &serde_json::Value, ttl: Duration) -> Result<String, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let session_id = uuid::Uuid::new_v4().to_string();
        let key = format!("session:{}", session_id);
        
        let mut session_data = session_data.clone();
        if let Some(obj) = session_data.as_object_mut() {
            obj.insert("user_id".to_string(), serde_json::Value::String(user_id.to_string()));
        }
        
        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl.as_secs())
            .arg(session_data.to_string())
            .query_async(&mut conn)
            .await?;
        
        // 将session ID关联到用户
        let user_sessions_key = format!("user_sessions:{}", user_id);
        redis::cmd("SADD")
            .arg(&user_sessions_key)
            .arg(&session_id)
            .query_async(&mut conn)
            .await?;
        
        // 设置session集合的过期时间
        redis::cmd("EXPIRE")
            .arg(&user_sessions_key)
            .arg(ttl.as_secs())
            .query_async(&mut conn)
            .await?;
        
        Ok(session_id)
    }
    
    #[instrument(skip(self))]
    pub async fn get_session(&self, session_id: &str) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("session:{}", session_id);
        
        let result: Option<String> = conn.get(&key).await?;
        if let Some(json_str) = result {
            Ok(Some(serde_json::from_str(&json_str)?))
        } else {
            Ok(None)
        }
    }
    
    #[instrument(skip(self))]
    pub async fn delete_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("session:{}", session_id);
        
        // 获取session数据以获取user_id
        if let Some(session_data) = self.get_session(session_id).await? {
            if let Some(user_id) = session_data.get("user_id").and_then(|v| v.as_str()) {
                let user_sessions_key = format!("user_sessions:{}", user_id);
                redis::cmd("SREM")
                    .arg(&user_sessions_key)
                    .arg(session_id)
                    .query_async(&mut conn)
                    .await?;
            }
        }
        
        redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn invalidate_user_sessions(&self, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let user_sessions_key = format!("user_sessions:{}", user_id);
        
        // 获取所有session ID
        let session_ids: Vec<String> = conn.smembers(&user_sessions_key).await?;
        
        // 删除所有会话
        for session_id in session_ids {
            let key = format!("session:{}", session_id);
            redis::cmd("DEL").arg(&key).query_async(&mut conn).await?;
        }
        
        // 删除会话集合
        redis::cmd("DEL")
            .arg(&user_sessions_key)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
}
```

```rust
// 主应用文件
// File: chat-system/src/main.rs
use clap::{Parser, Subcommand};
use tracing::{info, warn, error, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use tokio::sync::RwLock;

mod messages;
mod websocket_server;
mod http_server;
mod database;
mod redis_cache;

use messages::*;
use websocket_server::{WebSocketServer, WebSocketServerConfig};
use http_server::HttpServer;
use database::Database;
use redis_cache::RedisCache;

#[derive(Parser, Debug)]
#[command(name = "distributed-chat-system")]
#[command(about = "A distributed chat system built with Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the WebSocket server
    WebSocket {
        #[arg(short, long, default_value = "0.0.0.0:8080")]
        addr: String,
        
        #[arg(short, long, default_value = "postgres://chat_user:password@localhost/chat_db")]
        database_url: String,
        
        #[arg(short, long, default_value = "redis://localhost:6379")]
        redis_url: String,
    },
    /// Start the HTTP API server
    Http {
        #[arg(short, long, default_value = "0.0.0.0:8081")]
        addr: String,
        
        #[arg(short, long, default_value = "postgres://chat_user:password@localhost/chat_db")]
        database_url: String,
        
        #[arg(short, long, default_value = "redis://localhost:6379")]
        redis_url: String,
    },
    /// Start both servers
    Both {
        #[arg(short, long, default_value = "0.0.0.0:8080")]
        ws_addr: String,
        
        #[arg(short, long, default_value = "0.0.0.0:8081")]
        http_addr: String,
        
        #[arg(short, long, default_value = "postgres://chat_user:password@localhost/chat_db")]
        database_url: String,
        
        #[arg(short, long, default_value = "redis://localhost:6379")]
        redis_url: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "distributed_chat_system=debug,tokio=warn,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::WebSocket { addr, database_url, redis_url } => {
            run_websocket_server(addr, database_url, redis_url).await
        }
        Commands::Http { addr, database_url, redis_url } => {
            run_http_server(addr, database_url, redis_url).await
        }
        Commands::Both { ws_addr, http_addr, database_url, redis_url } => {
            run_both_servers(ws_addr, http_addr, database_url, redis_url).await
        }
    }
}

#[instrument]
async fn run_websocket_server(
    addr: String,
    database_url: String,
    redis_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting WebSocket server on {}", addr);
    
    // 初始化数据库和缓存
    let database = Database::new(&database_url).await?;
    let redis = RedisCache::new(&redis_url).await?;
    
    // 配置WebSocket服务器
    let config = WebSocketServerConfig {
        max_connections: 10000,
        heartbeat_interval: std::time::Duration::from_secs(30),
        message_buffer_size: 1000,
        rate_limit_per_minute: 100,
    };
    
    let server = WebSocketServer::new(config, database, redis);
    
    // 启动服务器
    if let Err(e) = server.run(&addr).await {
        error!("WebSocket server error: {}", e);
        return Err(e);
    }
    
    Ok(())
}

#[instrument]
async fn run_http_server(
    addr: String,
    database_url: String,
    redis_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting HTTP server on {}", addr);
    
    // 初始化数据库和缓存
    let database = Database::new(&database_url).await?;
    let redis = RedisCache::new(&redis_url).await?;
    
    // 启动WebSocket服务器（需要为HTTP服务器提供引用）
    let config = WebSocketServerConfig::default();
    let ws_server = WebSocketServer::new(config, database.clone(), redis.clone());
    
    let http_server = HttpServer::new(database, redis, ws_server);
    
    // 启动HTTP服务器
    if let Err(e) = http_server.run().await {
        error!("HTTP server error: {}", e);
        return Err(e);
    }
    
    Ok(())
}

#[instrument]
async fn run_both_servers(
    ws_addr: String,
    http_addr: String,
    database_url: String,
    redis_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting both servers - WebSocket: {}, HTTP: {}", ws_addr, http_addr);
    
    // 初始化数据库和缓存
    let database = Database::new(&database_url).await?;
    let redis = RedisCache::new(&redis_url).await?;
    
    // 配置WebSocket服务器
    let config = WebSocketServerConfig {
        max_connections: 10000,
        heartbeat_interval: std::time::Duration::from_secs(30),
        message_buffer_size: 1000,
        rate_limit_per_minute: 100,
    };
    
    let ws_server = WebSocketServer::new(config, database.clone(), redis.clone());
    let http_server = HttpServer::new(database, redis, ws_server.clone());
    
    // 启动两个服务器
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = ws_server.run(&ws_addr).await {
            error!("WebSocket server error: {}", e);
        }
    });
    
    let http_handle = tokio::spawn(async move {
        if let Err(e) = http_server.run().await {
            error!("HTTP server error: {}", e);
        }
    });
    
    // 等待两个服务器
    tokio::select! {
        result = ws_handle => {
            if let Err(e) = result {
                error!("WebSocket server task error: {}", e);
            }
        }
        result = http_handle => {
            if let Err(e) = result {
                error!("HTTP server task error: {}", e);
            }
        }
    }
    
    Ok(())
}

// 性能监控工具
pub struct SystemMetrics {
    active_connections: std::sync::atomic::AtomicUsize,
    messages_processed: std::sync::atomic::AtomicU64,
    errors_total: std::sync::atomic::AtomicU64,
    start_time: std::time::Instant,
}

impl SystemMetrics {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(SystemMetrics {
            active_connections: std::sync::atomic::AtomicUsize::new(0),
            messages_processed: std::sync::atomic::AtomicU64::new(0),
            errors_total: std::sync::atomic::AtomicU64::new(0),
            start_time: std::time::Instant::now(),
        }))
    }
    
    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn increment_messages(&self) {
        self.messages_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn increment_errors(&self) {
        self.errors_total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn get_stats(&self) -> SystemStats {
        SystemStats {
            active_connections: self.active_connections.load(std::sync::atomic::Ordering::Relaxed),
            messages_processed: self.messages_processed.load(std::sync::atomic::Ordering::Relaxed),
            errors_total: self.errors_total.load(std::sync::atomic::Ordering::Relaxed),
            uptime: self.start_time.elapsed(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemStats {
    pub active_connections: usize,
    pub messages_processed: u64,
    pub errors_total: u64,
    pub uptime: std::time::Duration,
}
```

```rust
// 部署配置和文档
// File: chat-system/docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: chat_db
      POSTGRES_USER: chat_user
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  chat-app:
    build: .
    ports:
      - "8080:8080"  # WebSocket
      - "8081:8081"  # HTTP API
    environment:
      DATABASE_URL: postgres://chat_user:password@postgres:5432/chat_db
      REDIS_URL: redis://redis:6379
    depends_on:
      - postgres
      - redis
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - chat-app

volumes:
  postgres_data:
  redis_data:
```

```rust
// File: chat-system/nginx.conf
events {
    worker_connections 1024;
}

http {
    upstream websocket_backend {
        server chat-app:8080;
    }
    
    upstream http_backend {
        server chat-app:8081;
    }
    
    # WebSocket代理配置
    server {
        listen 80;
        server_name localhost;
        
        # WebSocket端点
        location /ws {
            proxy_pass http://websocket_backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_connect_timeout 7d;
            proxy_send_timeout 7d;
            proxy_read_timeout 7d;
        }
        
        # HTTP API代理
        location /api/ {
            proxy_pass http://http_backend/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

```rust
// File: chat-system/Dockerfile
FROM rust:1.70 as builder

WORKDIR /app

# 复制Cargo文件
COPY Cargo.toml Cargo.lock ./

# 创建空main.rs以缓存依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 构建依赖
RUN cargo build --release
RUN rm src/main.rs

# 复制源代码
COPY src ./src

# 构建应用
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/distributed-chat-system ./

# 创建非root用户
RUN useradd -r -s /bin/false chatuser
USER chatuser

EXPOSE 8080 8081

CMD ["./distributed-chat-system", "both"]
```

```rust
// File: chat-system/README.md
# 分布式聊天系统

一个基于Rust构建的企业级分布式聊天系统，支持实时通信、房间管理、消息持久化等功能。

## 功能特性

### 核心功能
- **实时消息传输**：基于WebSocket的双向通信
- **多房间支持**：私人房间、群组房间、公共房间
- **消息类型**：文本、图片、文件、表情等
- **用户管理**：用户注册、认证、在线状态
- **消息状态**：发送、已送达、已读状态跟踪

### 企业级特性
- **高可用性**：支持负载均衡和集群部署
- **可扩展性**：水平扩展，支持大量并发用户
- **数据持久化**：PostgreSQL数据库存储
- **缓存优化**：Redis缓存提升性能
- **监控告警**：完整的系统监控和错误追踪
- **安全防护**：JWT认证、SQL注入防护、XSS防护

## 架构设计

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Load Balancer │    │   Load Balancer │    │   Load Balancer │
│     (Nginx)     │    │     (Nginx)     │    │     (Nginx)     │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │   Application Servers   │
                    │  ┌────────────────────┐ │
                    │  │ WebSocket Server   │ │
                    │  │ - 实时通信处理      │ │
                    │  │ - 连接管理         │ │
                    │  │ - 消息分发         │ │
                    │  └────────────────────┘ │
                    │  ┌────────────────────┐ │
                    │  │ HTTP API Server    │ │
                    │  │ - RESTful API      │ │
                    │  │ - 业务逻辑处理      │ │
                    │  │ - 认证授权         │ │
                    │  └────────────────────┘ │
                    └────────────┬────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │    Data Layer          │
                    │  ┌────────────────────┐ │
                    │  │ PostgreSQL         │ │
                    │  │ - 消息持久化        │ │
                    │  │ - 用户数据          │ │
                    │  │ - 房间信息          │ │
                    │  └────────────────────┘ │
                    │  ┌────────────────────┐ │
                    │  │ Redis              │ │
                    │  │ - 会话管理          │ │
                    │  │ - 在线状态          │ │
                    │  │ - 消息缓存          │ │
                    │  └────────────────────┘ │
                    └────────────────────────┘
```

## 快速开始

### 使用Docker Compose（推荐）

1. 克隆项目
```bash2. 克隆仓库
git clone <repository-url>
cd distributed-chat-system

3. 启动服务
docker-compose up -d

4. 访问系统
- WebSocket: ws://localhost/ws
- HTTP API: http://localhost/api
- Web界面: http://localhost

### 本地开发

1. **前置要求**
- Rust 1.70+
- PostgreSQL 13+
- Redis 6+
- Docker (可选)

2. **安装依赖**
```bash
# 安装PostgreSQL和Redis
sudo apt-get install postgresql redis-server

# 创建数据库
createdb chat_db
psql chat_db -f init.sql

# 运行应用
cargo run both -- \
  --database-url "postgres://chat_user:password@localhost/chat_db" \
  --redis-url "redis://localhost:6379"
```

## API文档

### WebSocket API

#### 连接
```javascript
const ws = new WebSocket('ws://localhost/ws');

// 发送认证消息
ws.send(JSON.stringify({
  message_id: 'uuid',
  data: {
    type: 'auth_request',
    username: 'user123',
    password_hash: 'hashed_password'
  }
}));
```

#### 消息格式
```json
{
  "id": "message-uuid",
  "message_type": "send_message",
  "data": {
    "type": "send_message_request",
    "room_id": "room-uuid",
    "content": "Hello, world!",
    "message_type": "text"
  }
}
```

### HTTP API

#### 用户管理
- `GET /api/users` - 获取所有用户
- `POST /api/users` - 创建新用户
- `GET /api/users/{id}` - 获取特定用户

#### 房间管理
- `GET /api/rooms` - 获取所有房间
- `POST /api/rooms` - 创建新房间
- `GET /api/rooms/{id}` - 获取特定房间

#### 消息管理
- `GET /api/messages/{room_id}/history?limit=50` - 获取消息历史
- `POST /api/messages/{room_id}` - 发送消息

## 性能优化

### 数据库优化
- 使用连接池管理数据库连接
- 为常用查询创建索引
- 使用分页查询减少数据传输
- 实施消息归档策略

### 缓存策略
- Redis缓存用户在线状态
- 缓存最近的房间消息
- 使用Redis进行会话管理
- 实施合适的TTL策略

### 网络优化
- 使用WebSocket进行实时通信
- 实施消息批处理
- 压缩JSON数据
- 使用CDN提供静态资源

## 安全考虑

### 认证授权
- JWT token认证
- 基于角色的访问控制
- 房间权限验证
- 会话管理和过期

### 数据安全
- 密码哈希存储（bcrypt）
- SQL注入防护
- XSS防护
- CSRF保护

### 网络安全
- HTTPS/WSS加密传输
- 速率限制
- 输入验证
- 错误处理

## 监控和运维

### 性能监控
- 系统资源监控
- 实时连接数统计
- 消息吞吐量监控
- 错误率跟踪

### 日志记录
- 结构化日志记录
- 错误跟踪和报告
- 性能指标收集
- 安全事件日志

### 健康检查
- 服务健康检查端点
- 数据库连接状态
- Redis连接状态
- 内存使用监控

## 扩展性设计

### 水平扩展
- 无状态应用设计
- 数据库读写分离
- Redis集群支持
- 负载均衡配置

### 微服务架构
- 用户服务
- 消息服务
- 房间服务
- 通知服务

## 故障处理

### 高可用
- 多实例部署
- 数据库主从复制
- Redis哨兵模式
- 故障转移机制

### 数据备份
- 定期数据库备份
- Redis数据持久化
- 消息队列备份
- 配置文件备份

## 开发和测试

### 开发环境
- 本地开发配置
- 调试工具设置
- 单元测试框架
- 集成测试

### 部署流程
- CI/CD管道
- 自动化测试
- 蓝绿部署
- 回滚策略

## 许可证

MIT License

## 贡献

欢迎提交Issue和Pull Request来改进这个项目。

---

**联系信息**：
- 作者：MiniMax Agent
- 邮箱：developer@minimax.com
- 文档：https://docs.minimax.com/chat-system
