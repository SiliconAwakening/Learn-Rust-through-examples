# 第8章：模块系统与工程化

## 学习目标
- 掌握Rust模块系统的核心概念
- 学会组织和构建大型项目结构
- 理解包和Crate的关系
- 掌握Cargo工作空间的使用
- 学习第三方依赖管理
- 构建一个完整的企业级微服务框架

---

## 8.1 模块系统基础

### 8.1.1 模块的创建与使用

在Rust中，模块是一种组织代码的方式，可以将相关的功能组合在一起。模块系统是Rust语言的核心特性之一，它帮助我们管理大型项目的复杂性。

#### 基本模块定义

```rust
// src/main.rs
mod math {
    // 模块中的私有函数
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    
    // 标记为pub的公共函数
    pub fn multiply(a: i32, b: i32) -> i32 {
        a * b
    }
    
    // 子模块
    pub mod advanced {
        pub fn power(base: f64, exponent: f64) -> f64 {
            base.powf(exponent)
        }
    }
}

fn main() {
    // 访问模块中的函数
    let result = math::multiply(5, 3);
    println!("5 * 3 = {}", result);
    
    // 访问子模块中的函数
    let power_result = math::advanced::power(2.0, 3.0);
    println!("2^3 = {}", power_result);
    
    // 无法调用私有函数
    // math::add(1, 2); // 编译错误
}
```

#### 可见性规则

Rust的模块系统有一个清晰的可见性规则：
- 默认情况下，模块中的所有项（函数、结构体、枚举等）都是私有的
- 只有标记为`pub`的项才能被外部访问
- 即使是公共项，也需要通过模块路径来访问

```rust
mod calculator {
    // 私有结构体
    struct Calculator {
        result: f64,
    }
    
    // 公共结构体
    pub struct Config {
        pub precision: u8,
        pub rounding: bool,
    }
    
    // 私有函数
    fn get_default_config() -> Config {
        Config {
            precision: 2,
            rounding: true,
        }
    }
    
    // 公共函数
    pub fn calculate(operation: &str, a: f64, b: f64) -> Result<f64, String> {
        match operation {
            "add" => Ok(a + b),
            "subtract" => Ok(a - b),
            "multiply" => Ok(a * b),
            "divide" => {
                if b == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(a / b)
                }
            },
            _ => Err("Unknown operation".to_string()),
        }
    }
    
    // 重新导出公共项
    pub use Config;
}

fn main() {
    // 使用公共结构体
    let config = calculator::Config {
        precision: 4,
        rounding: true,
    };
    
    // 使用公共函数
    match calculator::calculate("divide", 10.0, 3.0) {
        Ok(result) => println!("Result: {:.2}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // 使用重新导出的类型
    let config = calculator::Config;
}
```

### 8.1.2 路径与模块引用

在Rust中，有两种方式引用模块中的项：
1. **绝对路径**（从crate根开始）：`crate::module::item`
2. **相对路径**（从当前模块开始）：`module::item`、`self::item`、`super::item`

```rust
// src/main.rs
mod network {
    pub mod client {
        pub struct HttpClient {
            pub base_url: String,
        }
        
        impl HttpClient {
            pub fn new(url: &str) -> Self {
                Self {
                    base_url: url.to_string(),
                }
            }
        }
    }
    
    pub mod server {
        use super::client::HttpClient; // 相对路径导入
        
        pub struct WebServer {
            clients: Vec<HttpClient>,
        }
        
        impl WebServer {
            pub fn new() -> Self {
                Self {
                    clients: Vec::new(),
                }
            }
        }
    }
}

fn main() {
    // 使用绝对路径
    let client = network::client::HttpClient::new("https://api.example.com");
    
    // 使用相对路径
    use network::server::WebServer;
    let server = WebServer::new();
}
```

#### 路径导入与别名

```rust
use std::collections::HashMap;
use std::io::{self, Read, Write}; // 多项导入
use std::fs::File as MyFile; // 重命名导入

// 全局导入常用项
use std::result::Result as StdResult;

// 嵌套路径导入
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

fn example_usage() {
    // 使用导入的别名
    let file = MyFile::open("data.txt").unwrap();
    
    // 使用嵌套路径导入
    let path = Path::new("example.txt");
    let duration = Duration::from_millis(100);
}
```

### 8.1.3 模块文件的组织

对于大型项目，模块可以组织在不同的文件中。

**单文件组织方式（推荐用于简单项目）：**
```rust
// src/main.rs
mod utils {
    mod math {
        pub fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    }
    
    mod string {
        pub fn capitalize(s: &str) -> String {
            s.chars()
                .next()
                .map_or_else(String::new, |c| {
                    c.to_uppercase().collect::<String>() + &s[1..]
                })
        }
    }
}
```

**多文件组织方式（推荐用于大型项目）：**
```rust
// src/main.rs
// 导入子模块
mod utils {
    pub mod math;
    pub mod string;
}

// 重新导出，方便使用
pub use utils::math::add;
pub use utils::string::capitalize;

fn main() {
    let result = add(5, 3);
    let name = capitalize("hello world");
    println!("Result: {}, Capitalized: {}", result, name);
}
```

```rust
// src/utils/mod.rs
// 声明子模块
pub mod math;
pub mod string;

// 公共工具函数
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}
```

```rust
// src/utils/math.rs
// 数学运算模块
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

pub fn power(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}
```

```rust
// src/utils/string.rs
// 字符串处理模块
pub fn capitalize(s: &str) -> String {
    s.chars()
        .next()
        .map_or_else(String::new, |c| {
            c.to_uppercase().collect::<String>() + &s[1..]
        })
}

pub fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

pub fn word_count(s: &str) -> usize {
    s.split_whitespace().count()
}
```

---

## 8.2 Crate与包管理

### 8.2.1 包（Package）与Crate的区别

在Rust中，理解包（Package）和Crate的区别很重要：
- **包（Package）**：一个包含0个或多个Crate的目录
- **Crate**：编译的基本单元，是一个库或可执行程序

#### 创建包

```bash
# 创建新的包
cargo new my-package
# 这会创建：
# my-package/
# ├── Cargo.toml
# └── src/
#     └── main.rs
```

```toml
# Cargo.toml - 包配置文件
[package]
name = "my-package"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <email@example.com>"]
description = "A sample package"
repository = "https://github.com/username/my-package"
license = "MIT"
keywords = ["sample", "tutorial"]
categories = ["development-tools"]

[dependencies]
# 依赖项
serde = "1.0"
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
# 开发依赖（仅用于测试和构建）
tempfile = "3.0"

[build-dependencies]
# 构建依赖（build.rs中使用）
cc = "1.0"

[features]
# 特性标志
default = []
optimized = ["serde/derive"]
```

### 8.2.2 库Crate与二进制Crate

#### 库Crate（生成lib文件）

```rust
// src/lib.rs
// 库 crate 的入口文件
pub mod math {
    pub struct Calculator {
        precision: u8,
    }
    
    impl Calculator {
        pub fn new(precision: u8) -> Self {
            Self { precision }
        }
        
        pub fn add(&self, a: f64, b: f64) -> f64 {
            self.round(a + b)
        }
        
        pub fn multiply(&self, a: f64, b: f64) -> f64 {
            self.round(a * b)
        }
        
        fn round(&self, value: f64) -> f64 {
            let factor = 10f64.powi(self.precision as i32);
            (value * factor).round() / factor
        }
    }
}

pub mod io {
    use std::fs::File;
    use std::io::{self, Read, Write};
    
    pub fn read_file(path: &str) -> Result<String, io::Error> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }
    
    pub fn write_file(path: &str, content: &str) -> Result<(), io::Error> {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
```

#### 二进制Crate

```rust
// src/main.rs
// 入口文件
use my_package::{math::Calculator, io::{read_file, write_file}};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 4 {
        println!("Usage: {} <operation> <num1> <num2> [precision]", args[0]);
        println!("Operations: add, multiply");
        return Ok(());
    }
    
    let operation = &args[1];
    let num1: f64 = args[2].parse()?;
    let num2: f64 = args[3].parse()?;
    let precision = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(2);
    
    let calculator = Calculator::new(precision);
    
    let result = match operation.as_str() {
        "add" => calculator.add(num1, num2),
        "multiply" => calculator.multiply(num1, num2),
        _ => {
            eprintln!("Unknown operation: {}", operation);
            return Ok(());
        }
    };
    
    println!("Result: {}", result);
    
    // 如果提供了文件路径，保存结果
    if args.len() > 5 {
        let output_path = &args[5];
        let output = format!("{} {} {} = {}\n", num1, operation, num2, result);
        write_file(output_path, &output)?;
        println!("Result saved to {}", output_path);
    }
    
    Ok(())
}
```

### 8.2.3 内部模块与外部模块

#### 内部模块

```rust
// src/lib.rs
pub mod database {
    pub mod mysql {
        pub struct MySqlConnection {
            pub connection_string: String,
        }
        
        impl MySqlConnection {
            pub fn new(connection_string: &str) -> Self {
                Self {
                    connection_string: connection_string.to_string(),
                }
            }
            
            pub fn connect(&self) -> Result<(), String> {
                // 模拟连接
                println!("Connecting to MySQL: {}", self.connection_string);
                Ok(())
            }
        }
    }
    
    pub mod postgresql {
        pub struct PostgresConnection {
            pub connection_string: String,
        }
        
        impl PostgresConnection {
            pub fn new(connection_string: &str) -> Self {
                Self {
                    connection_string: connection_string.to_string(),
                }
            }
            
            pub fn connect(&self) -> Result<(), String> {
                // 模拟连接
                println!("Connecting to PostgreSQL: {}", self.connection_string);
                Ok(())
            }
        }
    }
}
```

#### 外部模块

```rust
// src/lib.rs
// 外部模块的声明
pub mod external;
```

```rust
// src/external/mod.rs
// 外部模块的实现
pub mod api;
pub mod utils;
```

```rust
// src/external/api.rs
pub mod rest {
    use reqwest::{Client, Response};
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ApiResponse<T> {
        pub success: bool,
        pub data: Option<T>,
        pub message: Option<String>,
    }
    
    pub struct RestClient {
        client: Client,
        base_url: String,
    }
    
    impl RestClient {
        pub fn new(base_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
            let client = Client::new();
            Ok(Self {
                client,
                base_url: base_url.to_string(),
            })
        }
        
        pub async fn get<T>(&self, endpoint: &str) -> Result<ApiResponse<T>, Box<dyn std::error::Error>>
        where
            T: serde::de::DeserializeOwned,
        {
            let url = format!("{}/{}", self.base_url, endpoint);
            let response = self.client.get(&url).send().await?;
            let result = response.json::<ApiResponse<T>>().await?;
            Ok(result)
        }
        
        pub async fn post<T, U>(
            &self,
            endpoint: &str,
            data: &U,
        ) -> Result<ApiResponse<T>, Box<dyn std::error::Error>>
        where
            T: serde::de::DeserializeOwned,
            U: serde::ser::Serialize,
        {
            let url = format!("{}/{}", self.base_url, endpoint);
            let response = self.client.post(&url).json(data).send().await?;
            let result = response.json::<ApiResponse<T>>().await?;
            Ok(result)
        }
    }
}
```

---

## 8.3 Cargo工作空间

### 8.3.1 工作空间的概念

Cargo工作空间允许您在多个包之间共享依赖项并简化构建过程。这对于大型项目特别有用。

### 8.3.2 创建工作空间

#### 工作空间目录结构

```toml
# Cargo.toml (工作空间根目录)
[workspace]
members = [
    "framework-core",
    "http-server",
    "service-registry",
    "config-manager",
    "monitoring",
    "examples/user-service",
    "examples/order-service",
    "examples/gateway",
]

[workspace.dependencies]
# 共享的依赖项
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
log = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
```

#### 共享依赖配置

```toml
# framework-core/Cargo.toml
[package]
name = "framework-core"
version = "0.1.0"
edition = "2021"

[dependencies]
# 继承工作空间的依赖
tokio = { workspace = true }
serde = { workspace = true }
anyhow = { workspace = true }

# 特定依赖
thiserror = "1.0"
async-trait = "0.1"
```

### 8.3.3 框架核心模块

```rust
// framework-core/src/lib.rs
pub mod service;
pub mod config;
pub mod error;
pub mod health;

// 导出常用的类型
pub use service::{Service, ServiceBuilder};
pub use config::Config;
pub use error::{FrameworkError, FrameworkResult};
pub use health::{HealthChecker, HealthStatus};

use tracing::{info, error};
use std::sync::Arc;

/// 框架主入口
pub struct Framework {
    services: Vec<Arc<dyn Service>>,
    config: Config,
}

impl Framework {
    pub fn new(config: Config) -> Self {
        Self {
            services: Vec::new(),
            config,
        }
    }
    
    pub fn register_service(&mut self, service: Arc<dyn Service>) {
        info!("Registering service: {}", service.name());
        self.services.push(service);
    }
    
    pub async fn start(&self) -> FrameworkResult<()> {
        info!("Starting Framework with {} services", self.services.len());
        
        for service in &self.services {
            if let Err(e) = service.start().await {
                error!("Failed to start service {}: {}", service.name(), e);
                return Err(e.into());
            }
        }
        
        Ok(())
    }
    
    pub async fn stop(&self) -> FrameworkResult<()> {
        info!("Stopping Framework");
        
        for service in &self.services {
            if let Err(e) = service.stop().await {
                error!("Failed to stop service {}: {}", service.name(), e);
                // 继续停止其他服务
            }
        }
        
        Ok(())
    }
}
```

```rust
// framework-core/src/service.rs
use crate::{FrameworkError, FrameworkResult};
use async_trait::async_trait;
use std::time::{Duration, Instant};

/// 服务特征定义
#[async_trait]
pub trait Service: Send + Sync {
    /// 获取服务名称
    fn name(&self) -> &str;
    
    /// 启动服务
    async fn start(&self) -> FrameworkResult<()>;
    
    /// 停止服务
    async fn stop(&self) -> FrameworkResult<()>;
    
    /// 健康检查
    async fn health_check(&self) -> FrameworkResult<bool>;
}

/// 基础服务实现
pub struct BaseService {
    name: String,
    started_at: Option<Instant>,
    state: ServiceState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

impl BaseService {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            started_at: None,
            state: ServiceState::Stopped,
        }
    }
    
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        // 配置启动超时
        self
    }
}

#[async_trait]
impl Service for BaseService {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn start(&self) -> FrameworkResult<()> {
        tracing::info!("Starting service: {}", self.name);
        // 模拟启动过程
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
    
    async fn stop(&self) -> FrameworkResult<()> {
        tracing::info!("Stopping service: {}", self.name);
        // 模拟停止过程
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }
    
    async fn health_check(&self) -> FrameworkResult<bool> {
        // 简单的健康检查
        Ok(true)
    }
}

/// 服务构建器
pub struct ServiceBuilder {
    name: String,
    startup_timeout: Option<Duration>,
    shutdown_timeout: Option<Duration>,
    health_check_interval: Option<Duration>,
}

impl ServiceBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            startup_timeout: Some(Duration::from_secs(30)),
            shutdown_timeout: Some(Duration::from_secs(10)),
            health_check_interval: Some(Duration::from_secs(30)),
        }
    }
    
    pub fn startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = Some(timeout);
        self
    }
    
    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = Some(timeout);
        self
    }
    
    pub fn health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = Some(interval);
        self
    }
    
    pub fn build(self) -> BaseService {
        BaseService::new(&self.name)
    }
}
```

### 8.3.4 HTTP服务器模块

```rust
// http-server/src/lib.rs
pub mod server;
pub mod router;
pub mod middleware;
pub mod request_handler;

use framework_core::{Service, FrameworkResult};
use server::HttpServer;
use std::sync::Arc;
use tokio::sync::oneshot;

/// HTTP服务实现
pub struct HttpService {
    name: String,
    server: HttpServer,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl HttpService {
    pub fn new(port: u16, routes: router::Router) -> Self {
        let name = format!("http-server-{}", port);
        let server = HttpServer::new(port, routes);
        
        Self {
            name,
            server,
            shutdown_tx: None,
        }
    }
}

#[async_trait::async_trait]
impl Service for HttpService {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn start(&self) -> FrameworkResult<()> {
        use framework_core::tracing;
        
        tracing::info!("Starting HTTP server on port {}", self.server.port());
        
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        
        // 启动服务器
        let server_handle = self.server.start();
        
        // 启动后台任务监听关闭信号
        tokio::spawn(async move {
            shutdown_rx.await.ok();
            tracing::info!("Received shutdown signal for HTTP server");
        });
        
        // 等待服务器启动完成
        server_handle.await?;
        
        Ok(())
    }
    
    async fn stop(&self) -> FrameworkResult<()> {
        use framework_core::tracing;
        
        tracing::info!("Stopping HTTP server");
        
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
        
        Ok(())
    }
    
    async fn health_check(&self) -> FrameworkResult<bool> {
        // 简单的健康检查 - 检查服务器是否响应
        Ok(true)
    }
}
```

### 8.3.5 服务注册模块

```rust
// service-registry/src/lib.rs
pub mod registry;
pub mod service_info;
pub mod health_check;
pub mod discovery;

use registry::ServiceRegistry;
use service_info::ServiceInfo;
use health_check::HealthChecker;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use framework_core::{Service, FrameworkResult};
use tracing::{info, warn, error};

/// 服务注册服务
pub struct RegistryService {
    name: String,
    registry: Arc<RwLock<ServiceRegistry>>,
    health_checker: HealthChecker,
}

impl RegistryService {
    pub fn new() -> Self {
        Self {
            name: "service-registry".to_string(),
            registry: Arc::new(RwLock::new(ServiceRegistry::new())),
            health_checker: HealthChecker::new(),
        }
    }
    
    /// 注册新服务
    pub async fn register_service(&self, service: ServiceInfo) -> FrameworkResult<()> {
        let mut registry = self.registry.write().await;
        registry.register(service).await?;
        info!("Service registered successfully");
        Ok(())
    }
    
    /// 查找服务
    pub async fn discover_service(&self, name: &str) -> FrameworkResult<Option<ServiceInfo>> {
        let registry = self.registry.read().await;
        Ok(registry.find_service(name).await)
    }
    
    /// 获取所有服务
    pub async fn list_services(&self) -> FrameworkResult<Vec<ServiceInfo>> {
        let registry = self.registry.read().await;
        Ok(registry.list_services().await)
    }
}

#[async_trait::async_trait]
impl Service for RegistryService {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn start(&self) -> FrameworkResult<()> {
        info!("Starting service registry");
        
        // 启动健康检查器
        self.health_checker.start().await?;
        
        // 开始定期健康检查
        let registry_clone = self.registry.clone();
        let health_checker = self.health_checker.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let mut registry = registry_clone.write().await;
                if let Err(e) = registry.perform_health_checks(&health_checker).await {
                    error!("Health check failed: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    async fn stop(&self) -> FrameworkResult<()> {
        info!("Stopping service registry");
        self.health_checker.stop().await?;
        Ok(())
    }
    
    async fn health_check(&self) -> FrameworkResult<bool> {
        Ok(true) // 简单实现
    }
}
```

---

## 8.4 实战项目：企业级微服务开发框架

### 8.4.1 项目整体架构

让我们创建一个完整的企业级微服务开发框架，展示模块化架构设计的最佳实践。

#### 完整项目结构

```
microservice-framework/
├── Cargo.toml                    # 工作空间根配置
├── framework-core/               # 框架核心
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── service.rs
│   │   ├── config.rs
│   │   ├── error.rs
│   │   ├── health.rs
│   │   └── tracing.rs
│   └── examples/
│       └── basic_service.rs
│
├── http-server/                 # HTTP服务器
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── server.rs
│   │   ├── router.rs
│   │   ├── middleware.rs
│   │   ├── request_handler.rs
│   │   └── response.rs
│   └── examples/
│       └── simple_server.rs
│
├── service-registry/            # 服务注册与发现
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── registry.rs
│   │   ├── service_info.rs
│   │   ├── health_check.rs
│   │   └── discovery.rs
│   └── examples/
│       └── service_registration.rs
│
├── config-manager/              # 配置管理
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs
│   │   ├── loader.rs
│   │   ├── watcher.rs
│   │   └── provider.rs
│   └── examples/
│       └── config_demo.rs
│
├── monitoring/                  # 监控与指标
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── metrics.rs
│   │   ├── logger.rs
│   │   └── health.rs
│   └── examples/
│       └── monitoring_demo.rs
│
├── examples/                    # 示例服务
│   ├── user-service/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── handlers.rs
│   │       ├── models.rs
│   │       └── config.rs
│   │
│   ├── order-service/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── handlers.rs
│   │       ├── models.rs
│   │       └── config.rs
│   │
│   └── gateway/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── proxy.rs
│           └── config.rs
│
├── docs/                        # 文档
│   ├── architecture.md
│   ├── api.md
│   └── deployment.md
│
└── scripts/                     # 部署脚本
    ├── deploy.sh
    └── setup.sh
```

### 8.4.2 配置文件管理

```rust
// config-manager/src/lib.rs
pub mod config;
pub mod loader;
pub mod watcher;
pub mod provider;

pub use config::ConfigManager;
pub use loader::ConfigLoader;
pub use watcher::ConfigWatcher;
pub use provider::ConfigProvider;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

/// 配置管理器主入口
pub struct ConfigManager {
    configs: Arc<RwLock<HashMap<String, ConfigValue>>>,
    watchers: Vec<ConfigWatcher>,
    provider: ConfigProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    pub value: serde_json::Value,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}

impl ConfigManager {
    pub fn new(provider: ConfigProvider) -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            watchers: Vec::new(),
            provider,
        }
    }
    
    /// 加载配置
    pub async fn load_config(&self, key: &str) -> Result<ConfigValue, ConfigError> {
        let value = self.provider.load(key).await?;
        let mut configs = self.configs.write().await;
        configs.insert(key.to_string(), value.clone());
        Ok(value)
    }
    
    /// 获取配置值
    pub async fn get_config(&self, key: &str) -> Option<ConfigValue> {
        let configs = self.configs.read().await;
        configs.get(key).cloned()
    }
    
    /// 设置配置值
    pub async fn set_config(&self, key: &str, value: ConfigValue) -> Result<(), ConfigError> {
        self.provider.save(key, &value).await?;
        let mut configs = self.configs.write().await;
        configs.insert(key.to_string(), value);
        
        // 通知观察者
        self.notify_watchers(key).await;
        Ok(())
    }
    
    /// 添加配置观察者
    pub fn add_watcher(&mut self, watcher: ConfigWatcher) {
        self.watchers.push(watcher);
    }
    
    /// 通知所有观察者
    async fn notify_watchers(&self, key: &str) {
        for watcher in &self.watchers {
            if let Err(e) = watcher.notify(key).await {
                error!("Failed to notify watcher: {}", e);
            }
        }
    }
    
    /// 监听配置变化
    pub async fn watch_config(&self, key: &str) -> Result<tokio::sync::mpsc::Receiver<ConfigValue>, ConfigError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let config_key = key.to_string();
        
        let watcher = ConfigWatcher::new(config_key, tx);
        // 在实际实现中，需要避免无限引用循环
        
        Ok(rx)
    }
}
```

### 8.4.3 监控与指标

```rust
// monitoring/src/lib.rs
pub mod metrics;
pub mod logger;
pub mod health;

use metrics::{MetricsCollector, Counter, Histogram, Gauge};
use logger::StructuredLogger;
use health::{HealthChecker, HealthStatus};
use std::time::{Duration, Instant};
use tokio::time::interval;

/// 监控主服务
pub struct MonitoringService {
    name: String,
    metrics_collector: MetricsCollector,
    logger: StructuredLogger,
    health_checker: HealthChecker,
    start_time: Instant,
}

impl MonitoringService {
    pub fn new() -> Self {
        Self {
            name: "monitoring".to_string(),
            metrics_collector: MetricsCollector::new(),
            logger: StructuredLogger::new(),
            health_checker: HealthChecker::new(),
            start_time: Instant::now(),
        }
    }
    
    /// 记录请求指标
    pub fn record_request(&self, method: &str, path: &str, status_code: u16, duration: Duration) {
        self.metrics_collector
            .counter("http_requests_total")
            .with_labels(&[("method", method), ("path", path), ("status", &status_code.to_string())])
            .inc();
            
        self.metrics_collector
            .histogram("http_request_duration")
            .with_labels(&[("method", method), ("path", path)])
            .observe(duration.as_secs_f64());
    }
    
    /// 记录服务指标
    pub fn record_service_metric(&self, service: &str, metric: &str, value: f64) {
        self.metrics_collector
            .gauge("service_metric")
            .with_labels(&[("service", service), ("metric", metric)])
            .set(value);
    }
    
    /// 获取系统健康状态
    pub async fn get_health_status(&self) -> HealthStatus {
        let uptime = self.start_time.elapsed();
        let memory_usage = self.get_memory_usage();
        
        HealthStatus::healthy()
            .with_detail("uptime_seconds", uptime.as_secs())
            .with_detail("memory_usage_mb", memory_usage)
    }
    
    fn get_memory_usage(&self) -> f64 {
        // 简化的内存使用量获取
        0.0 // 实际实现中需要使用系统调用
    }
}

#[async_trait::async_trait]
impl framework_core::Service for MonitoringService {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn start(&self) -> framework_core::FrameworkResult<()> {
        tracing::info!("Starting monitoring service");
        
        // 启动指标收集器
        self.metrics_collector.start().await?;
        
        // 启动日志系统
        self.logger.start().await?;
        
        // 启动健康检查
        self.health_checker.start().await?;
        
        // 启动定期指标报告
        let metrics_collector = self.metrics_collector.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                if let Err(e) = metrics_collector.report().await {
                    tracing::error!("Failed to report metrics: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    async fn stop(&self) -> framework_core::FrameworkResult<()> {
        tracing::info!("Stopping monitoring service");
        
        self.metrics_collector.stop().await?;
        self.logger.stop().await?;
        self.health_checker.stop().await?;
        
        Ok(())
    }
    
    async fn health_check(&self) -> framework_core::FrameworkResult<bool> {
        Ok(self.health_checker.check_all().await)
    }
}
```

### 8.4.4 示例用户服务

```rust
// examples/user-service/src/main.rs
use microservice_framework::{
    framework_core::{Framework, ServiceBuilder},
    http_server::HttpService,
    service_registry::RegistryService,
    config_manager::ConfigManager,
    monitoring::MonitoringService,
};
use std::sync::Arc;
use tracing::{info, error};
use tokio;

mod handlers;
mod models;
mod config;

use handlers::UserHandlers;
use models::User;
use config::UserServiceConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("Starting User Service");
    
    // 加载配置
    let config = UserServiceConfig::load().await?;
    info!("Configuration loaded: {:?}", config);
    
    // 创建框架
    let mut framework = Framework::new(config.framework.clone());
    
    // 创建配置管理器
    let config_manager = ConfigManager::new(config.config_provider);
    framework.register_service(Arc::new(config_manager));
    
    // 创建服务注册器
    let registry_service = RegistryService::new();
    framework.register_service(Arc::new(registry_service));
    
    // 创建监控服务
    let monitoring_service = MonitoringService::new();
    framework.register_service(Arc::new(monitoring_service));
    
    // 创建HTTP服务器
    let user_handlers = UserHandlers::new(config.database.clone());
    let routes = user_handlers.create_routes();
    let http_service = HttpService::new(config.http.port, routes);
    framework.register_service(Arc::new(http_service));
    
    // 启动框架
    framework.start().await?;
    
    // 等待优雅关闭
    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal");
    
    framework.stop().await?;
    info!("User Service stopped gracefully");
    
    Ok(())
}
```

```rust
// examples/user-service/src/handlers.rs
use microservice_framework::http_server::{Router, Request, Response, Result as HttpResult};
use crate::models::User;
use crate::config::DatabaseConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

/// 用户处理器
pub struct UserHandlers {
    database: Arc<Mutex<dyn UserRepository>>,
}

impl UserHandlers {
    pub fn new(database_config: DatabaseConfig) -> Self {
        let database = Arc::new(Mutex::new(InMemoryUserRepository::new()));
        Self { database }
    }
    
    pub fn create_routes(&self) -> Router {
        let mut router = Router::new();
        
        // GET /users - 获取所有用户
        router.get("/users", self.handle_get_users());
        
        // GET /users/:id - 获取特定用户
        router.get("/users/:id", self.handle_get_user());
        
        // POST /users - 创建新用户
        router.post("/users", self.handle_create_user());
        
        // PUT /users/:id - 更新用户
        router.put("/users/:id", self.handle_update_user());
        
        // DELETE /users/:id - 删除用户
        router.delete("/users/:id", self.handle_delete_user());
        
        router
    }
    
    async fn handle_get_users(&self) -> Arc<dyn RequestHandler> {
        Arc::new(GetUsersHandler {
            database: self.database.clone(),
        })
    }
    
    async fn handle_get_user(&self) -> Arc<dyn RequestHandler> {
        Arc::new(GetUserHandler {
            database: self.database.clone(),
        })
    }
    
    async fn handle_create_user(&self) -> Arc<dyn RequestHandler> {
        Arc::new(CreateUserHandler {
            database: self.database.clone(),
        })
    }
    
    async fn handle_update_user(&self) -> Arc<dyn RequestHandler> {
        Arc::new(UpdateUserHandler {
            database: self.database.clone(),
        })
    }
    
    async fn handle_delete_user(&self) -> Arc<dyn RequestHandler> {
        Arc::new(DeleteUserHandler {
            database: self.database.clone(),
        })
    }
}

/// 处理器特征
#[async_trait::async_trait]
pub trait RequestHandler: Send + Sync {
    async fn handle(&self, req: &Request) -> HttpResult<Response>;
}

/// GET /users处理器
pub struct GetUsersHandler {
    database: Arc<Mutex<dyn UserRepository>>,
}

#[async_trait::async_trait]
impl RequestHandler for GetUsersHandler {
    async fn handle(&self, _req: &Request) -> HttpResult<Response> {
        let database = self.database.lock().await;
        let users = database.get_all().await?;
        
        Ok(Response::json(users))
    }
}

/// GET /users/:id处理器
pub struct GetUserHandler {
    database: Arc<Mutex<dyn UserRepository>>,
}

#[async_trait::async_trait]
impl RequestHandler for GetUserHandler {
    async fn handle(&self, req: &Request) -> HttpResult<Response> {
        let id = req.param("id")?.parse::<uuid::Uuid>()?;
        
        let database = self.database.lock().await;
        match database.get_by_id(id).await? {
            Some(user) => Ok(Response::json(user)),
            None => Ok(Response::not_found("User not found")),
        }
    }
}

/// POST /users处理器
pub struct CreateUserHandler {
    database: Arc<Mutex<dyn UserRepository>>,
}

#[async_trait::async_trait]
impl RequestHandler for CreateUserHandler {
    async fn handle(&self, req: &Request) -> HttpResult<Response> {
        let create_user: CreateUserRequest = req.json()?;
        
        let user = User {
            id: uuid::Uuid::new_v4(),
            name: create_user.name,
            email: create_user.email,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let database = self.database.lock().await;
        database.create(user.clone()).await?;
        
        info!("User created: {}", user.id);
        Ok(Response::created(user))
    }
}

/// 用户仓库特征
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_all(&self) -> HttpResult<Vec<User>>;
    async fn get_by_id(&self, id: uuid::Uuid) -> HttpResult<Option<User>>;
    async fn create(&self, user: User) -> HttpResult<()>;
    async fn update(&self, id: uuid::Uuid, user: Partial<User>) -> HttpResult<()>;
    async fn delete(&self, id: uuid::Uuid) -> HttpResult<()>;
}

/// 内存用户仓库（示例实现）
pub struct InMemoryUserRepository {
    users: std::collections::HashMap<uuid::Uuid, User>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: std::collections::HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn get_all(&self) -> HttpResult<Vec<User>> {
        Ok(self.users.values().cloned().collect())
    }
    
    async fn get_by_id(&self, id: uuid::Uuid) -> HttpResult<Option<User>> {
        Ok(self.users.get(&id).cloned())
    }
    
    async fn create(&self, user: User) -> HttpResult<()> {
        self.users.insert(user.id, user);
        Ok(())
    }
    
    async fn update(&self, id: uuid::Uuid, partial_user: Partial<User>) -> HttpResult<()> {
        if let Some(existing_user) = self.users.get_mut(&id) {
            if let Some(name) = partial_user.name {
                existing_user.name = name;
            }
            if let Some(email) = partial_user.email {
                existing_user.email = email;
            }
            existing_user.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(microservice_framework::http_server::Error::NotFound)
        }
    }
    
    async fn delete(&self, id: uuid::Uuid) -> HttpResult<()> {
        self.users.remove(&id);
        Ok(())
    }
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}
```

### 8.4.5 部署与运维

```bash
#!/bin/bash
# scripts/deploy.sh - 部署脚本

set -e

# 配置
SERVICE_NAME=$1
BUILD_PROFILE=${2:-release}
DOCKER_IMAGE_PREFIX="microservice-framework"

if [ -z "$SERVICE_NAME" ]; then
    echo "Usage: $0 <service_name> [build_profile]"
    echo "Available services: user-service, order-service, gateway"
    exit 1
fi

# 检查服务名称
case $SERVICE_NAME in
    "user-service")
        DOCKERFILE="examples/user-service/Dockerfile"
        ;;
    "order-service")
        DOCKERFILE="examples/order-service/Dockerfile"
        ;;
    "gateway")
        DOCKERFILE="examples/gateway/Dockerfile"
        ;;
    *)
        echo "Unknown service: $SERVICE_NAME"
        exit 1
        ;;
esac

echo "Deploying $SERVICE_NAME with profile: $BUILD_PROFILE"

# 构建应用
echo "Building application..."
cargo build --$BUILD_PROFILE

# 构建Docker镜像
echo "Building Docker image..."
docker build -f $DOCKERFILE -t $DOCKER_IMAGE_PREFIX/$SERVICE_NAME:$BUILD_PROFILE .

# 推送到镜像仓库（如果配置了）
if [ ! -z "$DOCKER_REGISTRY" ]; then
    echo "Pushing to registry: $DOCKER_REGISTRY"
    docker tag $DOCKER_IMAGE_PREFIX/$SERVICE_NAME:$BUILD_PROFILE $DOCKER_REGISTRY/$DOCKER_IMAGE_PREFIX/$SERVICE_NAME:$BUILD_PROFILE
    docker push $DOCKER_REGISTRY/$DOCKER_IMAGE_PREFIX/$SERVICE_NAME:$BUILD_PROFILE
fi

# 部署到Kubernetes（如果配置了）
if [ -f "k8s/$SERVICE_NAME.yaml" ]; then
    echo "Deploying to Kubernetes..."
    kubectl apply -f k8s/$SERVICE_NAME.yaml
    kubectl rollout status deployment/$SERVICE_NAME
fi

echo "Deployment completed successfully!"

# 显示状态
echo "Deployment status:"
if [ -f "k8s/$SERVICE_NAME.yaml" ]; then
    kubectl get pods -l app=$SERVICE_NAME
fi
```

```yaml
# examples/user-service/Dockerfile
FROM rust:1.70-slim as builder

# 安装系统依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制Cargo文件
COPY Cargo.toml Cargo.lock ./
COPY framework-core ./framework-core
COPY http-server ./http-server
COPY service-registry ./service-registry
COPY config-manager ./config-manager
COPY monitoring ./monitoring
COPY examples/user-service ./examples/user-service

# 构建应用
RUN cargo build --release --bin user-service

# 运行时镜像
FROM debian:bullseye-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -r -s /bin/false microservice

# 设置工作目录
WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/user-service /app/user-service

# 复制配置文件
COPY examples/user-service/config/ ./config/

# 设置权限
RUN chown -R microservice:microservice /app
USER microservice

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 启动应用
CMD ["./user-service"]
```

---

## 8.5 性能优化与最佳实践

### 8.5.1 模块设计原则

#### 单一职责原则

```rust
// good: 清晰的单一职责
pub mod http {
    pub mod server {
        pub struct HttpServer { /* ... */ }
        impl HttpServer { /* ... */ }
    }
    
    pub mod client {
        pub struct HttpClient { /* ... */ }
        impl HttpClient { /* ... */ }
    }
}

pub mod database {
    pub mod connection {
        pub struct DatabaseConnection { /* ... */ }
    }
    
    pub mod query {
        pub struct QueryBuilder { /* ... */ }
    }
}

// bad: 混合职责
pub mod network {
    // 混合了太多功能
    pub struct MixedService {
        http_server: HttpServer,
        database: DatabaseConnection,
        cache: Cache,
        // 太多不相关的功能
    }
}
```

#### 依赖倒置

```rust
// 定义特征而非具体实现
pub mod repositories {
    use crate::models::User;
    
    #[async_trait::async_trait]
    pub trait UserRepository: Send + Sync {
        async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, Error>;
        async fn save(&self, user: User) -> Result<(), Error>;
        async fn delete(&self, id: uuid::Uuid) -> Result<(), Error>;
    }
}

pub mod services {
    use super::repositories::{UserRepository, Error};
    use crate::models::User;
    
    pub struct UserService<R: UserRepository> {
        repository: R,
    }
    
    impl<R: UserRepository> UserService<R> {
        pub fn new(repository: R) -> Self {
            Self { repository }
        }
        
        pub async fn get_user(&self, id: uuid::Uuid) -> Result<Option<User>, Error> {
            self.repository.find_by_id(id).await
        }
        
        pub async fn create_user(&self, user: User) -> Result<User, Error> {
            // 业务逻辑
            let saved_user = self.repository.save(user.clone()).await?;
            Ok(user)
        }
    }
}
```

### 8.5.2 编译时优化

#### 条件编译

```rust
#[cfg(feature = "metrics")]
pub mod metrics {
    use std::sync::atomic::{AtomicU64, Ordering};
    
    static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
    
    pub fn record_request() {
        REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_request_count() -> u64 {
        REQUEST_COUNT.load(Ordering::Relaxed)
    }
}

#[cfg(not(feature = "metrics"))]
pub mod metrics {
    pub fn record_request() {
        // 空实现
    }
    
    pub fn get_request_count() -> u64 {
        0
    }
}

// 在主代码中使用
pub fn handle_request() {
    #[cfg(feature = "metrics")]
    metrics::record_request();
    
    // 处理请求的逻辑
}
```

#### 泛型特化

```rust
pub trait Serializer<T> {
    fn serialize(&self, value: &T) -> Result<String, Error>;
    fn deserialize(&self, data: &str) -> Result<T, Error>;
}

pub struct JsonSerializer;

impl<T> Serializer<T> for JsonSerializer 
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn serialize(&self, value: &T) -> Result<String, Error> {
        serde_json::to_string(value).map_err(Error::Serialization)
    }
    
    fn deserialize(&self, data: &str) -> Result<T, Error> {
        serde_json::from_str(data).map_err(Error::Deserialization)
    }
}

pub struct MsgPackSerializer;

impl<T> Serializer<T> for MsgPackSerializer 
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn serialize(&self, value: &T) -> Result<String, Error> {
        let data = rmp_serde::to_vec(value).map_err(Error::Serialization)?;
        Ok(base64::encode(&data))
    }
    
    fn deserialize(&self, data: &str) -> Result<T, Error> {
        let bytes = base64::decode(data).map_err(Error::Encoding)?;
        rmp_serde::from_slice(&bytes).map_err(Error::Deserialization)
    }
}
```

### 8.5.3 运行时性能

#### 零拷贝数据结构

```rust
use std::ops::{Deref, DerefMut};

pub struct Bytes {
    data: Vec<u8>,
}

impl Bytes {
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data }
    }
    
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.data).unwrap()
    }
    
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl Deref for Bytes {
    type Target = [u8];
    
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// 避免不必要的数据复制
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Bytes>,  // 使用Bytes而不是String
}
```

#### 内存池

```rust
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct ObjectPool<T: Default> {
    pool: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Default> ObjectPool<T> {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    pub fn get(&self) -> PooledObject<T> {
        let mut pool = self.pool.lock().unwrap();
        
        if let Some(obj) = pool.pop_front() {
            PooledObject::new(obj, self.pool.clone())
        } else {
            PooledObject::new(T::default(), self.pool.clone())
        }
    }
    
    pub fn return_object(&self, mut obj: T) {
        let mut pool = self.pool.lock().unwrap();
        pool.push_back(std::mem::take(&mut obj));
    }
}

pub struct PooledObject<T> {
    object: Option<T>,
    pool: Arc<Mutex<VecDeque<T>>>,
}

impl<T> PooledObject<T> {
    fn new(object: T, pool: Arc<Mutex<VecDeque<T>>>) -> Self {
        Self {
            object: Some(object),
            pool,
        }
    }
}

impl<T> Deref for PooledObject<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            let mut pool = self.pool.lock().unwrap();
            pool.push_back(object);
        }
    }
}
```

---

## 8.6 测试策略

### 8.6.1 单元测试

```rust
// src/utils/math.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }
    
    #[test]
    fn test_multiply() {
        assert_eq!(multiply(3, 4), 12);
        assert_eq!(multiply(0, 100), 0);
        assert_eq!(multiply(-2, 3), -6);
    }
    
    #[test]
    fn test_power() {
        assert!((power(2.0, 3.0) - 8.0).abs() < 1e-10);
        assert!((power(5.0, 2.0) - 25.0).abs() < 1e-10);
        assert!((power(10.0, 0.0) - 1.0).abs() < 1e-10);
    }
    
    #[test]
    #[should_panic]
    fn test_division_by_zero() {
        divide(10.0, 0.0);
    }
}
```

### 8.6.2 集成测试

```rust
// tests/integration/api_test.rs
use microservice_framework::http_server::TestClient;
use serde_json;

#[tokio::test]
async fn test_user_crud_operations() {
    let app = user_service::create_test_app().await;
    let client = TestClient::new(app);
    
    // 创建用户
    let create_response = client
        .post("/users")
        .json(&serde_json::json!({
            "name": "John Doe",
            "email": "john@example.com"
        }))
        .send()
        .await;
    
    assert!(create_response.status().is_success());
    let created_user: User = create_response.json().await;
    assert_eq!(created_user.name, "John Doe");
    assert_eq!(created_user.email, "john@example.com");
    
    // 获取用户
    let get_response = client
        .get(&format!("/users/{}", created_user.id))
        .send()
        .await;
    
    assert!(get_response.status().is_success());
    let retrieved_user: User = get_response.json().await;
    assert_eq!(created_user.id, retrieved_user.id);
    
    // 更新用户
    let update_response = client
        .put(&format!("/users/{}", created_user.id))
        .json(&serde_json::json!({
            "name": "John Smith"
        }))
        .send()
        .await;
    
    assert!(update_response.status().is_success());
    
    // 验证更新
    let get_updated_response = client
        .get(&format!("/users/{}", created_user.id))
        .send()
        .await;
    
    assert!(get_updated_response.status().is_success());
    let updated_user: User = get_updated_response.json().await;
    assert_eq!(updated_user.name, "John Smith");
    
    // 删除用户
    let delete_response = client
        .delete(&format!("/users/{}", created_user.id))
        .send()
        .await;
    
    assert!(delete_response.status().is_success());
    
    // 验证删除
    let get_deleted_response = client
        .get(&format!("/users/{}", created_user.id))
        .send()
        .await;
    
    assert!(get_deleted_response.status().is_not_found());
}
```

### 8.6.3 性能测试

```rust
// benches/performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use microservice_framework::http_server::HttpServer;

fn bench_http_request(c: &mut Criterion) {
    c.bench_function("http_request", |b| {
        b.iter(|| {
            // 创建测试请求
            let request = HttpRequest::new("GET", "/users", "{}", HashMap::new());
            // 模拟处理
            black_box(process_request(request));
        });
    });
}

fn bench_database_operations(c: &mut Criterion) {
    c.bench_function("database_insert", |b| {
        b.iter(|| {
            let user = User::new("test_user", "test@example.com");
            black_box(insert_user(&user));
        });
    });
    
    c.bench_function("database_query", |b| {
        b.iter(|| {
            let user_id = Uuid::new_v4();
            black_box(query_user_by_id(&user_id));
        });
    });
}

fn process_request(request: HttpRequest) -> HttpResponse {
    // 简化实现
    HttpResponse::ok("OK")
}

fn insert_user(user: &User) -> Result<(), Error> {
    // 简化实现
    Ok(())
}

fn query_user_by_id(id: &Uuid) -> Result<Option<User>, Error> {
    // 简化实现
    Ok(None)
}

criterion_group!(benches, bench_http_request, bench_database_operations);
criterion_main!(benches);
```

---

## 8.7 总结与进阶

### 8.7.1 本章要点回顾

1. **模块系统基础**：
   - 理解模块的创建、导入和使用
   - 掌握可见性规则和路径引用
   - 学会组织大型项目的文件结构

2. **包与Crate管理**：
   - 区分包（Package）和Crate的概念
   - 掌握库Crate和二进制Crate的区别
   - 学会配置Cargo.toml和依赖管理

3. **Cargo工作空间**：
   - 理解工作空间的概念和优势
   - 学会共享依赖和配置
   - 掌握大型项目的组织方式

4. **企业级框架开发**：
   - 构建了完整的微服务开发框架
   - 实现了服务注册、配置管理、监控等核心功能
   - 学会了模块化架构设计原则

### 8.7.2 实际应用建议

1. **项目结构规划**：
   - 为大型项目设计清晰的模块边界
   - 使用工作空间管理多个相关包
   - 建立统一的代码风格和约定

2. **依赖管理**：
   - 定期更新依赖项
   - 使用特性标志控制编译选项
   - 避免依赖冲突和重复

3. **性能优化**：
   - 使用条件编译减少二进制大小
   - 实现内存池提高性能
   - 进行性能基准测试

4. **测试策略**：
   - 建立完整的测试套件
   - 进行集成测试验证组件协作
   - 定期执行性能测试

### 8.7.3 扩展学习方向

1. **高级模块化**：
   - 插件系统设计
   - 动态模块加载
   - 模块间通信模式

2. **构建系统**：
   - 自定义构建脚本
   - 代码生成工具
   - 编译时优化技术

3. **部署架构**：
   - 容器化部署策略
   - 服务网格架构
   - 云原生开发模式

通过本章的学习，您已经掌握了Rust模块系统的核心概念和最佳实践，能够构建大型、可维护的企业级应用程序。下一章将介绍并发编程，帮助您实现高并发的网络应用。
