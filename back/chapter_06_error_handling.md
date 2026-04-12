# 第六章：错误处理

## 6.1 章节概述

错误处理是任何健壮软件系统的核心组成部分。在Rust中，错误处理不仅是一种编程习惯，更是一种编译时保证。Rust的错误处理机制通过`Result<T, E>`和`Option<T>`类型，结合强大的模式匹配和错误传播机制，为开发者提供了构建可靠系统的强大工具。

在本章中，我们将通过构建一个**企业级API客户端库**（enterprise-api-client）来深入学习Rust的错误处理机制。这个项目将展示如何在实际企业环境中处理各种复杂的错误场景，包括网络错误、业务逻辑错误、验证错误等。

### 学习目标

完成本章学习后，您将能够：

- 理解Rust错误处理的基本原则
- 掌握`Result<T, E>`和`Option<T>`的使用
- 学会自定义错误类型的设计
- 掌握错误传播和转换机制
- 理解`?`操作符的使用场景
- 学会错误处理在异步环境中的最佳实践
- 构建健壮的错误恢复和重试机制
- 实现细粒度的错误分类和处理策略

### 实战项目预览

本章实战项目将构建一个企业级API客户端库，支持：

- 细粒度错误分类和处理
- 自动重试和熔断器模式
- 异步错误处理
- 限流和缓存机制
- 监控和指标收集
- 多种认证方式

## 6.2 Rust错误处理基础

### 6.2.1 为什么需要健壮的错误处理

在现代软件开发中，错误不仅仅是程序失败，它们是系统正常运行的一部分：

1. **网络连接问题**：超时、连接失败、服务器不可用
2. **数据验证问题**：无效输入、格式错误、业务规则违反
3. **资源限制**：内存不足、磁盘空间不够、CPU负载过高
4. **业务逻辑错误**：权限不足、配置错误、状态冲突
5. **外部依赖问题**：第三方API失败、数据库连接丢失

Rust的设计哲学是"让错误处理变得显式和强大"，而不是试图隐藏或忽略错误。

### 6.2.2 Option\<T\>：处理可能为空的值

`Option<T>`是Rust中处理可能为空值的标准方式，它强制开发者明确处理空值情况。

```rust
// Option的基本使用
fn demonstrate_option() {
    // 一些可能返回空值的函数
    let numbers = vec![1, 2, 3, 4, 5];
    
    // vec::get返回Option<&T>
    let first = numbers.get(0);
    let tenth = numbers.get(9);
    
    println!("第一个数字: {:?}", first);    // Some(1)
    println!("第十个数字: {:?}", tenth);   // None
    
    // 模式匹配处理Option
    match first {
        Some(value) => println!("值: {}", value),
        None => println!("没有值"),
    }
    
    // 使用if let进行简洁的匹配
    if let Some(value) = tenth {
        println!("第十个数字: {}", value);
    } else {
        println!("第十个数字不存在");
    }
    
    // 链式操作
    let result = numbers.get(0)
        .map(|x| x * 2)
        .unwrap_or(0);
    println!("翻倍结果: {}", result);
    
    // 组合多个Option
    let value1 = numbers.get(0);
    let value2 = numbers.get(1);
    
    if let (Some(v1), Some(v2)) = (value1, value2) {
        println!("两个值: {} + {} = {}", v1, v2, v1 + v2);
    }
}

// 自定义Option使用示例
#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
    email: Option<String>,
}

impl User {
    fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            email: None,
        }
    }
    
    fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
    
    fn get_display_name(&self) -> &str {
        if let Some(ref email) = self.email {
            email
        } else {
            &self.name
        }
    }
}

fn option_practical_example() {
    let user1 = User::new(1, "Alice".to_string());
    let user2 = User::new(2, "Bob".to_string()).with_email("bob@example.com".to_string());
    
    println!("用户1显示名: {}", user1.get_display_name());
    println!("用户2显示名: {}", user2.get_display_name());
    
    // 处理可能的空值情况
    let users = vec![user1, user2];
    
    for user in &users {
        match &user.email {
            Some(email) => println!("用户 {} 邮箱: {}", user.name, email),
            None => println!("用户 {} 没有邮箱", user.name),
        }
    }
}
```

### 6.2.3 Result<T, E>：处理可能失败的操作

`Result<T, E>`是处理可能失败操作的标准方式，它明确区分成功和失败的情况。

```rust
// Result的基本使用
fn demonstrate_result() {
    // 可能失败的除法操作
    let divide = |a: f64, b: f64| -> Result<f64, String> {
        if b == 0.0 {
            Err("除数不能为零".to_string())
        } else {
            Ok(a / b)
        }
    };
    
    // 使用match处理结果
    match divide(10.0, 2.0) {
        Ok(result) => println!("10 / 2 = {}", result),
        Err(error) => println!("错误: {}", error),
    }
    
    match divide(10.0, 0.0) {
        Ok(result) => println!("10 / 0 = {}", result),
        Err(error) => println!("错误: {}", error),
    }
    
    // 使用?操作符传播错误
    fn calculate_average(numbers: &[f64]) -> Result<f64, String> {
        if numbers.is_empty() {
            return Err("数字列表不能为空".to_string());
        }
        
        let sum: f64 = numbers.iter().sum();
        let average = sum / numbers.len() as f64;
        
        if average.is_nan() {
            return Err("计算结果无效".to_string());
        }
        
        Ok(average)
    }
    
    // 组合多个Result
    let numbers1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let numbers2 = vec![];
    
    println!("数组1平均值: {:?}", calculate_average(&numbers1));
    println!("数组2平均值: {:?}", calculate_average(&numbers2));
    
    // 使用链式操作
    let result = calculate_average(&numbers1)
        .map(|avg| avg * 2.0)  // 如果成功，将平均值翻倍
        .map_err(|e| format!("计算错误: {}", e));  // 如果失败，添加上下文
    
    println!("翻倍后的平均值: {:?}", result);
}

// 文件操作的Result示例
use std::fs::File;
use std::io::{Read, Write};

fn file_operations() -> Result<String, String> {
    // 尝试打开文件
    let mut file = match File::open("config.json") {
        Ok(file) => file,
        Err(e) => return Err(format!("无法打开文件: {}", e)),
    };
    
    // 读取文件内容
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => contents,
        Err(e) => return Err(format!("无法读取文件: {}", e)),
    }
    
    // 验证内容
    if contents.is_empty() {
        return Err("文件内容为空".to_string());
    }
    
    Ok(contents)
}

fn write_to_file() -> Result<(), String> {
    let data = "Hello, World!";
    
    let mut file = match File::create("output.txt") {
        Ok(file) => file,
        Err(e) => return Err(format!("无法创建文件: {}", e)),
    };
    
    match file.write_all(data.as_bytes()) {
        Ok(_) => println!("文件写入成功"),
        Err(e) => return Err(format!("写入失败: {}", e)),
    }
    
    Ok(())
}

// 使用简化的错误传播
fn simplified_file_operations() -> Result<String, std::io::Error> {
    let mut file = File::open("config.json")?;  // ?操作符自动传播错误
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;  // 如果失败立即返回错误
    Ok(contents)
}
```

### 6.2.4 错误传播和转换

```rust
// 错误转换和处理
#[derive(Debug)]
enum ParseError {
    InvalidNumber(String),
    EmptyInput,
    OutOfRange { value: f64, min: f64, max: f64 },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidNumber(s) => write!(f, "无效数字: {}", s),
            ParseError::EmptyInput => write!(f, "输入为空"),
            ParseError::OutOfRange { value, min, max } => {
                write!(f, "值 {} 不在范围 [{}, {}] 内", value, min, max)
            }
        }
    }
}

impl std::error::Error for ParseError {}

// 从其他错误类型转换
impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError::InvalidNumber(format!("IO错误: {}", error))
    }
}

impl From<&str> for ParseError {
    fn from(msg: &str) -> Self {
        ParseError::InvalidNumber(msg.to_string())
    }
}

// 数字解析函数
fn parse_number(input: &str, min: f64, max: f64) -> Result<f64, ParseError> {
    if input.trim().is_empty() {
        return Err(ParseError::EmptyInput);
    }
    
    let number: f64 = input.trim()
        .parse()
        .map_err(|_| ParseError::InvalidNumber(input.to_string()))?;
    
    if number < min || number > max {
        return Err(ParseError::OutOfRange { value: number, min, max });
    }
    
    Ok(number)
}

// 链式错误处理
fn process_user_input() -> Result<f64, ParseError> {
    let inputs = vec!["", "not_a_number", "50", "150"];
    
    for input in inputs {
        match parse_number(input, 0.0, 100.0) {
            Ok(number) => {
                println!("成功解析: {} -> {}", input, number);
                return Ok(number);
            }
            Err(error) => {
                println!("解析失败 '{}': {}", input, error);
                // 继续尝试下一个输入
            }
        }
    }
    
    Err("所有输入都无效".into())
}

// 错误恢复策略
fn robust_calculation() -> Result<f64, String> {
    let values = vec!["10", "20", "invalid", "30", ""];
    
    let mut sum = 0.0;
    let mut valid_count = 0;
    let mut errors = Vec::new();
    
    for value in values {
        match parse_number(value, 0.0, 1000.0) {
            Ok(num) => {
                sum += num;
                valid_count += 1;
            }
            Err(error) => {
                errors.push(format!("'{}': {}", value, error));
            }
        }
    }
    
    if valid_count == 0 {
        return Err(format!("没有有效值，错误: {:?}", errors));
    }
    
    let average = sum / valid_count as f64;
    
    if !errors.is_empty() {
        println!("警告: 跳过了一些无效值: {:?}", errors);
    }
    
    Ok(average)
}
```

## 6.3 异步错误处理

在现代网络编程中，异步错误处理是关键技术。Rust的async/await语法与错误处理完美结合。

### 6.3.1 异步错误处理基础

```rust
// 异步错误处理示例
use tokio::time::{sleep, Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
enum AsyncError {
    NetworkTimeout,
    ConnectionFailed,
    InvalidResponse,
    FileNotFound,
    PermissionDenied,
}

impl std::fmt::Display for AsyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsyncError::NetworkTimeout => write!(f, "网络超时"),
            AsyncError::ConnectionFailed => write!(f, "连接失败"),
            AsyncError::InvalidResponse => write!(f, "无效响应"),
            AsyncError::FileNotFound => write!(f, "文件未找到"),
            AsyncError::PermissionDenied => write!(f, "权限拒绝"),
        }
    }
}

impl std::error::Error for AsyncError {}

// 模拟异步网络请求
async fn fetch_data(url: &str) -> Result<String, AsyncError> {
    println!("开始请求: {}", url);
    
    // 模拟网络延迟
    sleep(Duration::from_millis(100)).await;
    
    // 模拟可能的错误
    if url.contains("timeout") {
        return Err(AsyncError::NetworkTimeout);
    }
    
    if url.contains("404") {
        return Err(AsyncError::FileNotFound);
    }
    
    if url.contains("500") {
        return Err(AsyncError::ConnectionFailed);
    }
    
    // 模拟成功响应
    Ok(format!("响应来自: {}", url))
}

// 异步错误恢复
async fn fetch_with_retry(url: &str, max_retries: usize) -> Result<String, AsyncError> {
    let mut last_error = None;
    
    for attempt in 1..=max_retries {
        match fetch_data(url).await {
            Ok(data) => {
                println!("第{}次尝试成功", attempt);
                return Ok(data);
            }
            Err(error) => {
                println!("第{}次尝试失败: {}", attempt, error);
                last_error = Some(error);
                
                if attempt < max_retries {
                    // 指数退避
                    let delay = Duration::from_millis(100 * (2_u64.pow(attempt as u32 - 1)));
                    println!("等待 {}ms 后重试", delay.as_millis());
                    sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

// 并发异步操作和错误处理
async fn fetch_multiple_urls(urls: &[&str]) -> Result<Vec<String>, AsyncError> {
    use futures::future::join_all;
    
    // 并发执行所有请求
    let futures: Vec<_> = urls.iter()
        .map(|&url| fetch_data(url))
        .collect();
    
    let results = join_all(futures).await;
    
    // 收集成功和失败的结果
    let mut successful = Vec::new();
    let mut errors = Vec::new();
    
    for result in results {
        match result {
            Ok(data) => successful.push(data),
            Err(error) => errors.push(error),
        }
    }
    
    if !errors.is_empty() {
        return Err(format!("{} 个请求失败", errors.len()).into());
    }
    
    Ok(successful)
}

// 选择最快的响应
async fn fetch_fastest_response(urls: &[&str]) -> Result<String, AsyncError> {
    use futures::future::select;
    use futures::pin_mut;
    
    let futures: Vec<_> = urls.iter()
        .map(|&url| Box::pin(fetch_data(url)))
        .collect();
    
    // 选择最先完成的任务
    let mut completed = false;
    
    for future in futures {
        if completed {
            break;
        }
        
        pin_mut!(future);
        match select(future, sleep(Duration::from_secs(5))).await {
            std::task::Poll::Ready((result, _)) => {
                match result {
                    Ok(data) => {
                        completed = true;
                        return Ok(data);
                    }
                    Err(error) => {
                        eprintln!("请求失败: {}", error);
                    }
                }
            }
            std::task::Poll::Pending => {
                // 继续下一个请求
                continue;
            }
        }
    }
    
    Err("所有请求都失败了".into())
}
```

### 6.3.2 异步错误处理最佳实践

```rust
// 异步错误处理最佳实践
use std::sync::Arc;
use tokio::sync::Mutex;

// 错误累积器
struct ErrorCollector {
    errors: Vec<String>,
    max_errors: usize,
}

impl ErrorCollector {
    fn new(max_errors: usize) -> Self {
        Self {
            errors: Vec::new(),
            max_errors,
        }
    }
    
    async fn add_error(&mut self, error: String) {
        if self.errors.len() < self.max_errors {
            self.errors.push(error);
        }
    }
    
    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    fn get_errors(&self) -> &[String] {
        &self.errors
    }
}

// 批量异步操作
async fn batch_process_with_error_handling(
    items: Vec<String>,
    processor: Arc<dyn ProcessItem + Send + Sync>,
) -> Result<Vec<String>, String> {
    use tokio::sync::Semaphore;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let semaphore = Arc::new(Semaphore::new(5)); // 限制并发数
    let error_collector = Arc::new(Mutex::new(ErrorCollector::new(10)));
    let processed_count = Arc::new(AtomicUsize::new(0));
    
    let mut handles = Vec::new();
    
    for item in items {
        let semaphore = semaphore.clone();
        let processor = processor.clone();
        let error_collector = error_collector.clone();
        let processed_count = processed_count.clone();
        
        let handle = tokio::spawn(async move {
            // 获取信号量许可
            let _permit = semaphore.acquire().await.unwrap();
            
            match processor.process_item(&item).await {
                Ok(result) => {
                    processed_count.fetch_add(1, Ordering::Relaxed);
                    Some(result)
                }
                Err(error) => {
                    let error_msg = format!("处理项目 '{}' 失败: {}", item, error);
                    error_collector.lock().await.add_error(error_msg).await;
                    None
                }
            }
        });
        
        handles.push(handle);
    }
    
    // 等待所有任务完成
    let mut results = Vec::new();
    for handle in handles {
        if let Some(result) = handle.await.map_err(|e| e.to_string())? {
            results.push(result);
        }
    }
    
    // 检查是否有错误
    let errors = error_collector.lock().await.get_errors().to_vec();
    if !errors.is_empty() {
        return Err(format!("处理失败: {:?}", errors));
    }
    
    println!("成功处理了 {} 个项目", processed_count.load(Ordering::Relaxed));
    Ok(results)
}

// 异步处理trait
#[async_trait::async_trait]
trait ProcessItem {
    async fn process_item(&self, item: &str) -> Result<String, String>;
}

// 具体的处理器实现
struct DataProcessor {
    delay_ms: u64,
}

impl DataProcessor {
    fn new(delay_ms: u64) -> Self {
        Self { delay_ms }
    }
}

#[async_trait::async_trait]
impl ProcessItem for DataProcessor {
    async fn process_item(&self, item: &str) -> Result<String, String> {
        // 模拟处理延迟
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        
        // 模拟可能的处理错误
        if item.contains("error") {
            return Err("包含错误标记".to_string());
        }
        
        Ok(format!("处理完成: {}", item.to_uppercase()))
    }
}

// 超时处理
async fn with_timeout<T, F, Fut>(timeout: Duration, future: F) -> Result<T, AsyncError>
where
    F: Future<Output = T>,
    Fut: Future<Output = Result<T, String>>,
{
    use futures::future::select;
    
    let timeout_future = sleep(timeout);
    let operation_future = future.map_err(|e| AsyncError::InvalidResponse);
    
    pin_mut!(timeout_future);
    pin_mut!(operation_future);
    
    match select(timeout_future, operation_future).await {
        std::task::Poll::Ready(_) => Err(AsyncError::NetworkTimeout),
        std::task::Poll::Ready((result, _)) => result.map_err(|e| AsyncError::InvalidResponse),
    }
}
```

## 6.4 实战项目：企业级API客户端库

现在开始构建我们的实战项目。首先设计错误处理架构。

### 6.4.1 错误类型设计

```rust
// 企业级错误处理系统
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// API客户端错误类型
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("请求超时: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    
    #[error("网络错误: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("HTTP错误: {status} - {message}")]
    Http { 
        status: reqwest::StatusCode, 
        message: String,
        body: Option<String>,
        headers: HashMap<String, String>,
    },
    
    #[error("认证错误: {0}")]
    Authentication(String),
    
    #[error("授权错误: {0}")]
    Authorization(String),
    
    #[error("频率限制: {remaining:?}")]
    RateLimit { 
        remaining: Option<Duration>,
        reset_time: Option<SystemTime>,
        retry_after: Option<Duration>,
    },
    
    #[error("服务不可用: {reason}")]
    ServiceUnavailable { reason: String },
    
    #[error("配置错误: {0}")]
    Configuration(String),
    
    #[error("重试耗尽: 已尝试 {attempts} 次")]
    RetryExhausted { attempts: u32 },
    
    #[error("熔断器开启")]
    CircuitBreakerOpen,
    
    #[error("缓存错误: {0}")]
    Cache(#[from] CacheError),
    
    #[error("验证错误: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("业务逻辑错误: {0}")]
    Business(String),
    
    #[error("系统错误: {0}")]
    System(String),
}

impl ApiError {
    /// 判断错误是否是可重试的
    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::Network(_) | ApiError::Timeout(_) => true,
            ApiError::Http { status, .. } => {
                status.is_server_error() || status.as_u16() == 429
            }
            ApiError::ServiceUnavailable { .. } => true,
            ApiError::CircuitBreakerOpen => false,
            ApiError::RateLimit { retry_after, .. } => retry_after.is_some(),
            ApiError::Configuration(_) | ApiError::Authentication(_) | ApiError::Authorization(_) => false,
            ApiError::Cache(_) | ApiError::Validation(_) | ApiError::Serialization(_) => true,
            ApiError::Business(_) | ApiError::System(_) => false,
        }
    }
    
    /// 获取重试建议的延迟时间
    pub fn recommended_delay(&self) -> Option<Duration> {
        match self {
            ApiError::Network(_) | ApiError::Timeout(_) => Some(Duration::from_millis(100)),
            ApiError::Http { status, .. } if status.is_server_error() => Some(Duration::from_secs(1)),
            ApiError::RateLimit { retry_after, .. } => *retry_after,
            ApiError::ServiceUnavailable { .. } => Some(Duration::from_secs(5)),
            _ => None,
        }
    }
    
    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        match self {
            ApiError::Network(_) | ApiError::Timeout(_) => ErrorCategory::Network,
            ApiError::Http { .. } => ErrorCategory::Http,
            ApiError::Authentication(_) | ApiError::Authorization(_) => ErrorCategory::Auth,
            ApiError::RateLimit { .. } => ErrorCategory::RateLimit,
            ApiError::Configuration(_) => ErrorCategory::Configuration,
            ApiError::Validation(_) | ApiError::Serialization(_) => ErrorCategory::Data,
            ApiError::Business(_) => ErrorCategory::Business,
            ApiError::System(_) | ApiError::ServiceUnavailable { .. } => ErrorCategory::System,
            ApiError::Cache(_) => ErrorCategory::Cache,
            ApiError::CircuitBreakerOpen | ApiError::RetryExhausted { .. } => ErrorCategory::Reliability,
        }
    }
}

/// 错误分类
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    Network,
    Http,
    Auth,
    RateLimit,
    Configuration,
    Data,
    Business,
    System,
    Cache,
    Reliability,
}

/// 验证错误
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("缺少必需字段: {field}")]
    MissingField { field: String },
    
    #[error("字段格式错误: {field} - {reason}")]
    InvalidFormat { field: String, reason: String },
    
    #[error("字段值超出范围: {field} - {min} 到 {max}")]
    OutOfRange { field: String, min: String, max: String },
    
    #[error("字段值不符合模式: {field} - 模式: {pattern}")]
    PatternMismatch { field: String, pattern: String, value: String },
    
    #[error("自定义验证失败: {0}")]
    Custom(String),
}

/// 缓存错误
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("键不存在: {key}")]
    KeyNotFound { key: String },
    
    #[error("键已过期: {key}")]
    KeyExpired { key: String, expired_at: SystemTime },
    
    #[error("缓存未命中: {key}")]
    CacheMiss { key: String },
    
    #[error("缓存存储失败: {key} - {reason}")]
    StorageError { key: String, reason: String },
    
    #[error("连接错误: {0}")]
    ConnectionError(String),
    
    #[error("配置错误: {0}")]
    ConfigurationError(String),
}

/// 错误上下文信息
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub timestamp: SystemTime,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub status_code: Option<reqwest::StatusCode>,
    pub response_time_ms: Option<u64>,
    pub retry_count: Option<u32>,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now(),
            request_id: None,
            user_id: None,
            endpoint: None,
            method: None,
            status_code: None,
            response_time_ms: None,
            retry_count: None,
        }
    }
}

/// 错误统计
#[derive(Debug, Clone)]
pub struct ErrorStats {
    pub total_errors: u64,
    pub errors_by_category: HashMap<ErrorCategory, u64>,
    pub last_error_time: Option<SystemTime>,
    pub error_rate: f64, // 每秒错误数
}

impl Default for ErrorStats {
    fn default() -> Self {
        Self {
            total_errors: 0,
            errors_by_category: HashMap::new(),
            last_error_time: None,
            error_rate: 0.0,
        }
    }
}
```

### 6.4.2 重试策略和熔断器

```rust
// 重试策略实现
use std::collections::VecDeque;
use std::time::Instant;

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter_enabled: bool,
    pub jitter_range: f64, // 0.0 到 1.0
    pub retryable_errors: Vec<ErrorCategory>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_enabled: true,
            jitter_range: 0.1, // 10%的抖动
            retryable_errors: vec![
                ErrorCategory::Network,
                ErrorCategory::Http,
                ErrorCategory::RateLimit,
            ],
        }
    }
}

/// 熔断器状态
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,     // 正常状态，允许请求
    Open,       // 熔断状态，拒绝请求
    HalfOpen,   // 半开状态，允许少量请求测试
}

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,  // 失败阈值
    pub success_threshold: u32,  // 成功阈值
    pub timeout: Duration,       // 熔断持续时间
    pub monitor_window: Duration, // 监控时间窗口
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            monitor_window: Duration::from_secs(60),
        }
    }
}

/// 熔断器实现
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    last_success_time: Option<Instant>,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            last_success_time: None,
        }
    }
    
    /// 检查是否可以执行请求
    pub fn can_execute(&mut self) -> bool {
        let now = Instant::now();
        
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // 检查是否可以转换到半开状态
                if let Some(last_failure) = self.last_failure_time {
                    if now.duration_since(last_failure) > self.config.timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    /// 记录成功
    pub fn on_success(&mut self) {
        let now = Instant::now();
        
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0; // 清除失败计数
            }
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitBreakerState::Open => {
                // 在熔断状态下不应该有成功的调用
            }
        }
        
        self.last_success_time = Some(now);
    }
    
    /// 记录失败
    pub fn on_failure(&mut self) {
        let now = Instant::now();
        
        self.failure_count += 1;
        self.last_failure_time = Some(now);
        
        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::HalfOpen => {
                // 在半开状态下的任何失败都回到打开状态
                self.state = CircuitBreakerState::Open;
                self.success_count = 0;
            }
            CircuitBreakerState::Open => {
                // 保持在打开状态
            }
        }
    }
    
    /// 获取当前状态
    pub fn state(&self) -> CircuitBreakerState {
        self.state.clone()
    }
    
    /// 获取状态信息
    pub fn status(&self) -> CircuitBreakerStatus {
        CircuitBreakerStatus {
            state: self.state(),
            failure_count: self.failure_count,
            success_count: self.success_count,
            last_failure_time: self.last_failure_time.map(|i| i.elapsed()),
            last_success_time: self.last_success_time.map(|i| i.elapsed()),
        }
    }
}

/// 熔断器状态信息
#[derive(Debug, Clone)]
pub struct CircuitBreakerStatus {
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: Option<Duration>,
    pub last_success_time: Option<Duration>,
}

/// 重试器
pub struct RetryHandler {
    config: RetryConfig,
    attempt_history: VecDeque<Duration>,
    max_history_size: usize,
}

impl RetryHandler {
    /// 创建新的重试处理器
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            attempt_history: VecDeque::new(),
            max_history_size: 100,
        }
    }
    
    /// 执行带重试的操作
    pub async fn execute_with_retry<T, F, Fut>(
        &mut self,
        operation: F,
        initial_context: ErrorContext,
    ) -> Result<T, ApiError>
    where
        F: Fn(u32, ErrorContext) -> Fut,
        Fut: Future<Output = Result<T, ApiError>>,
    {
        let mut context = initial_context;
        let mut last_error = None;
        
        for attempt in 1..=self.config.max_attempts {
            context.retry_count = Some(attempt);
            
            match operation(attempt, context.clone()).await {
                Ok(result) => {
                    // 成功：记录重试历史并返回结果
                    self.record_attempt(attempt, true);
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error);
                    
                    // 记录失败历史
                    self.record_attempt(attempt, false);
                    
                    // 检查是否应该重试
                    if !self.should_retry(&error) || attempt == self.config.max_attempts {
                        return Err(error);
                    }
                    
                    // 计算延迟时间
                    let delay = self.calculate_delay(attempt);
                    println!("重试 {} 将在 {:?} 后执行", attempt, delay);
                    tokio::time::sleep(delay).await;
                    
                    // 更新上下文
                    context.timestamp = SystemTime::now();
                }
            }
        }
        
        unreachable!()
    }
    
    /// 检查是否应该重试
    fn should_retry(&self, error: &ApiError) -> bool {
        // 检查错误类型是否可重试
        if !self.config.retryable_errors.contains(&error.category()) {
            return false;
        }
        
        // 检查是否在推荐的重试窗口内
        if let Some(recommended_delay) = error.recommended_delay() {
            if recommended_delay > self.config.max_delay {
                return false;
            }
        }
        
        true
    }
    
    /// 计算重试延迟
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let mut delay = self.config.base_delay;
        
        if attempt > 1 {
            let exponential_delay = self.config.base_delay.as_millis() as f64 * 
                self.config.backoff_multiplier.powi(attempt as i32 - 1);
            
            delay = Duration::from_millis(exponential_delay as u64);
            delay = delay.min(self.config.max_delay);
            
            // 添加抖动
            if self.config.jitter_enabled {
                let jitter_range = delay.as_millis() as f64 * self.config.jitter_range;
                let jitter = rand::thread_rng().gen_range(-jitter_range..jitter_range);
                let jittered_delay = delay.as_millis() as f64 + jitter;
                delay = Duration::from_millis(jittered_delay.max(0.0) as u64);
            }
        }
        
        delay
    }
    
    /// 记录尝试历史
    fn record_attempt(&mut self, attempt: u32, success: bool) {
        if success {
            // 记录成功尝试的延迟时间
            self.attempt_history.push_back(Duration::from_millis(100)); // 简化实现
        } else {
            // 记录失败尝试的延迟时间
            self.attempt_history.push_back(Duration::from_millis(200)); // 简化实现
        }
        
        // 保持历史记录大小
        if self.attempt_history.len() > self.max_history_size {
            self.attempt_history.pop_front();
        }
    }
    
    /// 获取重试统计
    pub fn get_stats(&self) -> RetryStats {
        let total_attempts = self.attempt_history.len() as u64;
        let total_time: u64 = self.attempt_history.iter()
            .map(|d| d.as_millis() as u64)
            .sum();
        
        RetryStats {
            total_attempts,
            average_delay_ms: if total_attempts > 0 {
                total_time / total_attempts
            } else {
                0
            },
            success_rate: if total_attempts > 0 {
                // 简化计算
                0.5
            } else {
                0.0
            },
        }
    }
}

/// 重试统计
#[derive(Debug, Clone)]
pub struct RetryStats {
    pub total_attempts: u64,
    pub average_delay_ms: u64,
    pub success_rate: f64,
}
```

### 6.4.3 限流器实现

```rust
// 限流器实现
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// 限流器配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: f64,
    pub burst_size: u32,
    pub window_size: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10.0,
            burst_size: 5,
            window_size: Duration::from_secs(1),
        }
    }
}

/// 滑动窗口限流器
pub struct SlidingWindowRateLimiter {
    config: RateLimitConfig,
    request_times: VecDeque<Instant>,
    allowed_tokens: AtomicU64,
    max_tokens: u64,
}

impl SlidingWindowRateLimiter {
    /// 创建新的限流器
    pub fn new(config: RateLimitConfig) -> Self {
        let max_tokens = (config.requests_per_second * config.window_size.as_secs_f64()) as u64 + config.burst_size as u64;
        
        Self {
            config,
            request_times: VecDeque::new(),
            allowed_tokens: AtomicU64::new(max_tokens),
            max_tokens,
        }
    }
    
    /// 尝试获取执行许可
    pub async fn acquire(&self) -> Result<(), ApiError> {
        // 简化实现：直接使用原子变量
        let current_tokens = self.allowed_tokens.load(Ordering::Relaxed);
        
        if current_tokens > 0 {
            if self.allowed_tokens.compare_exchange_weak(
                current_tokens,
                current_tokens - 1,
                Ordering::Relaxed,
                Ordering::Relaxed
            ).is_ok() {
                return Ok(());
            }
        }
        
        // 如果没有可用令牌，抛出错误
        Err(ApiError::RateLimit {
            remaining: Some(Duration::from_millis(100)), // 模拟延迟
            reset_time: Some(SystemTime::now()),
            retry_after: Some(Duration::from_millis(100)),
        })
    }
    
    /// 释放令牌
    pub fn release(&self) {
        let current_tokens = self.allowed_tokens.load(Ordering::Relaxed);
        if current_tokens < self.max_tokens {
            self.allowed_tokens.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// 获取当前状态
    pub fn status(&self) -> RateLimitStatus {
        let current_tokens = self.allowed_tokens.load(Ordering::Relaxed);
        
        RateLimitStatus {
            available_tokens: current_tokens,
            max_tokens: self.max_tokens,
            tokens_per_second: self.config.requests_per_second,
            remaining: Some(Duration::from_secs((current_tokens as f64 / self.config.requests_per_second) as u64)),
        }
    }
}

/// 限流器状态
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub available_tokens: u64,
    pub max_tokens: u64,
    pub tokens_per_second: f64,
    pub remaining: Option<Duration>,
}

/// 令牌桶限流器
pub struct TokenBucketRateLimiter {
    config: RateLimitConfig,
    tokens: AtomicU64,
    last_refill: Instant,
}

impl TokenBucketRateLimiter {
    /// 创建令牌桶限流器
    pub fn new(config: RateLimitConfig) -> Self {
        let initial_tokens = config.burst_size as u64;
        
        Self {
            config,
            tokens: AtomicU64::new(initial_tokens),
            last_refill: Instant::now(),
        }
    }
    
    /// 尝试获取令牌
    pub async fn acquire(&self) -> Result<(), ApiError> {
        self.refill_tokens();
        
        let current_tokens = self.tokens.load(Ordering::Relaxed);
        
        if current_tokens > 0 {
            if self.tokens.compare_exchange_weak(
                current_tokens,
                current_tokens - 1,
                Ordering::Relaxed,
                Ordering::Relaxed
            ).is_ok() {
                return Ok(());
            }
        }
        
        Err(ApiError::RateLimit {
            remaining: Some(self.time_to_next_token()),
            reset_time: Some(SystemTime::now()),
            retry_after: Some(self.time_to_next_token()),
        })
    }
    
    /// 补充令牌
    fn refill_tokens(&self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        
        let tokens_to_add = (elapsed.as_secs_f64() * self.config.requests_per_second) as u64;
        
        if tokens_to_add > 0 {
            let current_tokens = self.tokens.load(Ordering::Relaxed);
            let new_tokens = (current_tokens + tokens_to_add).min(self.config.burst_size as u64);
            
            self.tokens.store(new_tokens, Ordering::Relaxed);
            
            // 更新最后补充时间
            let _ = std::sync::Arc::new(self) as *const Self; // 简化实现
        }
    }
    
    /// 计算到下一个令牌的时间
    fn time_to_next_token(&self) -> Duration {
        Duration::from_millis((1.0 / self.config.requests_per_second * 1000.0) as u64)
    }
}
```

## 6.5 完整的API客户端实现

现在实现完整的API客户端：

```rust
// 完整的API客户端实现
use reqwest::{Client as HttpClient, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// HTTP方法
#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

/// API客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub user_agent: String,
    pub retry_config: RetryConfig,
    pub rate_limit_config: RateLimitConfig,
    pub circuit_breaker_config: CircuitBreakerConfig,
    pub cache_config: CacheConfig,
    pub auth_config: AuthConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.example.com".to_string(),
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            user_agent: "EnterpriseAPIClient/1.0".to_string(),
            retry_config: RetryConfig::default(),
            rate_limit_config: RateLimitConfig::default(),
            circuit_breaker_config: CircuitBreakerConfig::default(),
            cache_config: CacheConfig::default(),
            auth_config: AuthConfig::default(),
        }
    }
}

/// 认证配置
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub credentials: AuthCredentials,
    pub refresh_strategy: RefreshStrategy,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            auth_type: AuthType::None,
            credentials: AuthCredentials::None,
            refresh_strategy: RefreshStrategy::Never,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthType {
    None,
    Bearer,
    ApiKey,
    Basic,
    OAuth2,
}

#[derive(Debug, Clone)]
pub enum AuthCredentials {
    None,
    Bearer { token: String },
    ApiKey { key: String, header_name: String },
    Basic { username: String, password: String },
    OAuth2 { client_id: String, client_secret: String, token_url: String },
}

#[derive(Debug, Clone)]
pub enum RefreshStrategy {
    Never,
    Automatic { refresh_threshold: Duration },
    Manual,
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub enabled: bool,
    pub default_ttl: Duration,
    pub max_size: usize,
    pub cache_type: CacheType,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: Duration::from_secs(300),
            max_size: 1000,
            cache_type: CacheType::Memory,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CacheType {
    Memory,
    Redis,
}

/// API客户端
pub struct ApiClient {
    http_client: HttpClient,
    config: ClientConfig,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    rate_limiter: Arc<SlidingWindowRateLimiter>,
    retry_handler: Arc<RwLock<RetryHandler>>,
    cache: Arc<dyn Cache>,
    metrics: Arc<Metrics>,
}

impl ApiClient {
    /// 创建新的API客户端
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        // 构建HTTP客户端
        let http_client = ClientBuilder::new()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(ApiError::Network)?;
        
        // 创建组件
        let circuit_breaker = Arc::new(RwLock::new(CircuitBreaker::new(config.circuit_breaker_config.clone())));
        let rate_limiter = Arc::new(SlidingWindowRateLimiter::new(config.rate_limit_config.clone()));
        let retry_handler = Arc::new(RwLock::new(RetryHandler::new(config.retry_config.clone())));
        let cache = Arc::new(match config.cache_config.cache_type {
            CacheType::Memory => CacheImpl::Memory(MemoryCache::new()),
            CacheType::Redis => todo!("Redis缓存实现"),
        });
        let metrics = Arc::new(Metrics::new());
        
        Ok(Self {
            http_client,
            config,
            circuit_breaker,
            rate_limiter,
            retry_handler,
            cache,
            metrics,
        })
    }
    
    /// 发送GET请求
    pub async fn get<T>(&self, endpoint: &str) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        self.request::<(), T>(HttpMethod::GET, endpoint, None).await
    }
    
    /// 发送POST请求
    pub async fn post<B, T>(&self, endpoint: &str, body: &B) -> Result<T, ApiError>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        self.request_with_body(HttpMethod::POST, endpoint, Some(body)).await
    }
    
    /// 通用请求方法
    pub async fn request<T>(&self, method: HttpMethod, endpoint: &str, body: Option<&impl Serialize>) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        self.request_with_body::<T>(method, endpoint, body).await
    }
    
    /// 通用请求方法（带请求体）
    pub async fn request_with_body<T>(&self, method: HttpMethod, endpoint: &str, body: Option<&impl Serialize>) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let start_time = Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        
        // 构建请求URL
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", self.config.base_url.trim_end_matches('/'), endpoint)
        };
        
        // 检查熔断器
        {
            let mut breaker = self.circuit_breaker.write().await;
            if !breaker.can_execute() {
                return Err(ApiError::CircuitBreakerOpen);
            }
        }
        
        // 检查限流器
        self.rate_limiter.acquire().await?;
        
        // 检查缓存
        let cache_key = self.generate_cache_key(&method, &url, body);
        if let Some(cached_response) = self.get_cached_response::<T>(&cache_key).await? {
            self.metrics.record_cache_hit();
            return Ok(cached_response);
        }
        
        // 执行请求
        let response = self.execute_request_with_retry(method, &url, body, request_id).await?;
        
        // 更新熔断器
        {
            let mut breaker = self.circuit_breaker.write().await;
            if response.is_ok() {
                breaker.on_success();
            } else {
                breaker.on_failure();
            }
        }
        
        // 记录指标
        self.metrics.record_request(
            method.as_str().to_string(),
            url,
            response.is_ok(),
            start_time.elapsed(),
        );
        
        match response {
            Ok(data) => {
                // 更新缓存
                if self.config.cache_config.enabled {
                    self.set_cache_response(&cache_key, &data).await?;
                }
                Ok(data)
            }
            Err(error) => {
                Err(error)
            }
        }
    }
    
    /// 执行带重试的请求
    async fn execute_request_with_retry<T>(
        &self,
        method: HttpMethod,
        url: &str,
        body: Option<&impl Serialize>,
        request_id: String,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let initial_context = ErrorContext {
            request_id: Some(request_id),
            endpoint: Some(url.to_string()),
            method: Some(method.as_str().to_string()),
            ..Default::default()
        };
        
        let operation = |attempt: u32, context: ErrorContext| async move {
            self.perform_http_request::<T>(method.clone(), url, body, context).await
        };
        
        let mut retry_handler = self.retry_handler.write().await;
        retry_handler.execute_with_retry(operation, initial_context).await
    }
    
    /// 执行HTTP请求
    async fn perform_http_request<T>(
        &self,
        method: HttpMethod,
        url: &str,
        body: Option<&impl Serialize>,
        context: ErrorContext,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let mut request = self.http_client
            .request(reqwest::Method::from_str(method.as_str()), url)
            .header("X-Request-ID", context.request_id.clone().unwrap_or_default());
        
        // 添加认证
        if let Err(e) = self.add_authentication(&mut request).await {
            return Err(e);
        }
        
        // 添加请求体
        if let Some(body_data) = body {
            request = request.json(body_data);
        }
        
        // 执行请求
        let response = request.send().await.map_err(ApiError::Network)?;
        let status = response.status();
        
        // 更新上下文
        let context = ErrorContext {
            status_code: Some(status),
            ..context
        };
        
        // 处理响应
        self.handle_response::<T>(response, status, context).await
    }
    
    /// 处理HTTP响应
    async fn handle_response<T>(
        &self,
        response: reqwest::Response,
        status: reqwest::StatusCode,
        context: ErrorContext,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|s| (k.as_str().to_string(), s.to_string()))
            })
            .collect();
        
        match status {
            reqwest::StatusCode::OK | reqwest::StatusCode::CREATED | reqwest::StatusCode::ACCEPTED => {
                // 成功响应
                let data: T = response.json().await.map_err(ApiError::Serialization)?;
                Ok(data)
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                // 认证错误
                let body = response.text().await.ok();
                let message = body.as_deref().unwrap_or("未授权");
                Err(ApiError::Authentication(message.to_string()))
            }
            reqwest::StatusCode::FORBIDDEN => {
                // 授权错误
                let body = response.text().await.ok();
                let message = body.as_deref().unwrap_or("禁止访问");
                Err(ApiError::Authorization(message.to_string()))
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                // 频率限制
                let retry_after = response.headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(Duration::from_secs);
                
                Err(ApiError::RateLimit {
                    remaining: None,
                    reset_time: Some(SystemTime::now()),
                    retry_after,
                })
            }
            status if status.is_server_error() => {
                // 服务器错误
                let body = response.text().await.ok();
                let message = body.as_deref().unwrap_or("服务器内部错误");
                Err(ApiError::Http {
                    status,
                    message: message.to_string(),
                    body,
                    headers,
                })
            }
            status => {
                // 其他HTTP错误
                let body = response.text().await.ok();
                let message = format!("HTTP {} 错误", status.as_u16());
                Err(ApiError::Http {
                    status,
                    message,
                    body,
                    headers,
                })
            }
        }
    }
    
    /// 添加认证
    async fn add_authentication(&self, request: &mut reqwest::RequestBuilder) -> Result<(), ApiError> {
        match &self.config.auth_config.auth_type {
            AuthType::None => Ok(()),
            AuthType::Bearer => {
                if let AuthCredentials::Bearer { token } = &self.config.auth_config.credentials {
                    let header_value = format!("Bearer {}", token);
                    Ok(request.bearer_auth(header_value))
                } else {
                    Err(ApiError::Configuration("无效的认证凭据".to_string()))
                }
            }
            AuthType::ApiKey => {
                if let AuthCredentials::ApiKey { key, header_name } = &self.config.auth_config.credentials {
                    Ok(request.header(header_name, key))
                } else {
                    Err(ApiError::Configuration("无效的API密钥".to_string()))
                }
            }
            _ => Ok(()), // 其他认证类型简化实现
        }
    }
    
    /// 生成缓存键
    fn generate_cache_key(&self, method: &HttpMethod, url: &str, body: Option<&impl Serialize>) -> String {
        let mut key = format!("{}:{}", method.as_str(), url);
        
        if let Some(body_data) = body {
            if let Ok(body_str) = serde_json::to_string(body_data) {
                key.push_str(&format!(":body:{}", body_str));
            }
        }
        
        // 简单哈希
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("cache:{:x}", hasher.finish())
    }
    
    /// 从缓存获取响应
    async fn get_cached_response<T>(&self, key: &str) -> Result<Option<T>, ApiError>
    where
        T: DeserializeOwned + Serialize,
    {
        if !self.config.cache_config.enabled {
            return Ok(None);
        }
        
        match self.cache.get(key).await {
            Ok(cached_data) => {
                if let Some(data) = cached_data {
                    let result = serde_json::from_value::<T>(data)
                        .map_err(ApiError::Serialization)?;
                    Ok(Some(result))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                eprintln!("缓存获取失败: {:?}", e);
                Ok(None)
            }
        }
    }
    
    /// 设置缓存响应
    async fn set_cache_response<T>(&self, key: &str, data: &T) -> Result<(), ApiError>
    where
        T: Serialize,
    {
        if !self.config.cache_config.enabled {
            return Ok(());
        }
        
        let json_value = serde_json::to_value(data)
            .map_err(ApiError::Serialization)?;
        
        self.cache.set(key, &json_value, self.config.cache_config.default_ttl).await
    }
    
    /// 获取客户端状态
    pub async fn get_status(&self) -> ClientStatus {
        let circuit_breaker_status = self.circuit_breaker.read().await.status();
        let rate_limit_status = self.rate_limiter.status();
        let retry_stats = self.retry_handler.read().await.get_stats();
        let error_stats = self.metrics.get_error_stats();
        
        ClientStatus {
            circuit_breaker: circuit_breaker_status,
            rate_limiter: rate_limit_status,
            retry_stats,
            error_stats,
            total_requests: self.metrics.get_total_requests(),
        }
    }
}

/// 内存缓存实现
struct MemoryCache {
    data: Arc<RwLock<HashMap<String, (serde_json::Value, SystemTime)>>>,
    max_size: usize,
}

impl MemoryCache {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            max_size: 1000,
        }
    }
}

#[async_trait::async_trait]
impl Cache for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, CacheError> {
        let data = self.data.read().await;
        if let Some((value, expires_at)) = data.get(key) {
            if *expires_at > SystemTime::now() {
                Ok(Some(value.clone()))
            } else {
                // 已过期，删除
                drop(data);
                let mut data = self.data.write().await;
                data.remove(key);
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    async fn set(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<(), CacheError> {
        let mut data = self.data.write().await;
        
        // 检查大小限制
        if data.len() >= self.max_size {
            // 简单的LRU实现：删除最旧的条目
            if let Some((oldest_key, _)) = data.iter().next() {
                data.remove(oldest_key);
            }
        }
        
        let expires_at = SystemTime::now() + ttl;
        data.insert(key.to_string(), (value.clone(), expires_at));
        
        Ok(())
    }
    
    async fn remove(&self, key: &str) -> Result<(), CacheError> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }
    
    async fn clear(&self) -> Result<(), CacheError> {
        let mut data = self.data.write().await;
        data.clear();
        Ok(())
    }
}

/// 缓存trait
#[async_trait::async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, CacheError>;
    async fn set(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<(), CacheError>;
    async fn remove(&self, key: &str) -> Result<(), CacheError>;
    async fn clear(&self) -> Result<(), CacheError>;
}

/// 客户端状态
#[derive(Debug, Clone)]
pub struct ClientStatus {
    pub circuit_breaker: CircuitBreakerStatus,
    pub rate_limiter: RateLimitStatus,
    pub retry_stats: RetryStats,
    pub error_stats: ErrorStats,
    pub total_requests: u64,
}

/// 指标收集
pub struct Metrics {
    request_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    response_time_sum: Arc<AtomicU64>,
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
    errors_by_category: Arc<RwLock<HashMap<ErrorCategory, AtomicU64>>>,
}

impl Metrics {
    fn new() -> Self {
        Self {
            request_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            response_time_sum: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            errors_by_category: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    fn record_request(&self, method: String, url: String, success: bool, duration: Duration) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        
        if success {
            self.success_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
        
        self.response_time_sum.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    }
    
    fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    fn record_error(&self, error: &ApiError) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
        
        let category = error.category();
        let errors = self.errors_by_category.clone();
        tokio::spawn(async move {
            let mut errors_map = errors.write().await;
            let counter = errors_map.entry(category).or_insert_with(|| AtomicU64::new(0));
            counter.fetch_add(1, Ordering::Relaxed);
        });
    }
    
    fn get_total_requests(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }
    
    fn get_error_stats(&self) -> ErrorStats {
        let mut errors_by_category = HashMap::new();
        let errors_map = self.errors_by_category.blocking_read();
        for (category, counter) in errors_map.iter() {
            errors_by_category.insert(category.clone(), counter.load(Ordering::Relaxed));
        }
        
        ErrorStats {
            total_errors: self.error_count.load(Ordering::Relaxed),
            errors_by_category,
            last_error_time: Some(SystemTime::now()),
            error_rate: 0.0, // 简化计算
        }
    }
}
```

## 6.6 使用示例和测试

```rust
// 使用示例
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端配置
    let config = ClientConfig {
        base_url: "https://jsonplaceholder.typicode.com".to_string(),
        timeout: Duration::from_secs(10),
        retry_config: RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            ..Default::default()
        },
        ..Default::default()
    };
    
    // 创建API客户端
    let client = ApiClient::new(config)?;
    
    // 示例1: GET请求
    println!("=== GET请求示例 ===");
    match client.get::<Vec<User>>("/users").await {
        Ok(users) => {
            println!("成功获取 {} 个用户", users.len());
            if let Some(user) = users.first() {
                println!("第一个用户: {} ({})", user.name, user.email);
            }
        }
        Err(error) => {
            println!("请求失败: {}", error);
        }
    }
    
    // 示例2: POST请求
    println!("\n=== POST请求示例 ===");
    let new_user = CreateUserRequest {
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
    };
    
    match client.post("/posts", &new_user).await {
        Ok(post) => {
            println!("创建帖子成功: {:?}", post);
        }
        Err(error) => {
            println!("创建失败: {}", error);
        }
    }
    
    // 示例3: 错误处理
    println!("\n=== 错误处理示例 ===");
    // 尝试访问不存在的端点
    match client.get::<ApiResponse<User>>("/users/9999").await {
        Ok(response) => {
            println!("响应: {:?}", response);
        }
        Err(error) => {
            println!("预期错误: {}", error);
            
            // 检查错误类型
            match error {
                ApiError::Http { status, .. } => {
                    println!("HTTP状态码: {}", status);
                }
                ApiError::Network(e) => {
                    println!("网络错误: {}", e);
                }
                _ => {
                    println!("其他错误类型");
                }
            }
        }
    }
    
    // 示例4: 获取客户端状态
    println!("\n=== 客户端状态 ===");
    let status = client.get_status().await;
    println!("熔断器状态: {:?}", status.circuit_breaker.state);
    println!("总请求数: {}", status.total_requests);
    println!("错误统计: {:?}", status.error_stats);
    
    Ok(())
}

// 测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_error_classification() {
        let network_error = ApiError::Network(reqwest::Error::from(reqwest::Error::new(
            reqwest::ErrorKind::Timeout,
            "Connection timeout"
        )));
        
        let http_error = ApiError::Http {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            message: "Internal Server Error".to_string(),
            body: None,
            headers: HashMap::new(),
        };
        
        assert!(network_error.is_retryable());
        assert!(http_error.is_retryable());
        
        assert_eq!(network_error.category(), ErrorCategory::Network);
        assert_eq!(http_error.category(), ErrorCategory::Http);
    }
    
    #[test]
    fn test_circuit_breaker() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            ..Default::default()
        };
        
        let mut breaker = CircuitBreaker::new(config);
        
        // 测试初始状态
        assert!(breaker.can_execute());
        assert_eq!(breaker.state(), CircuitBreakerState::Closed);
        
        // 触发失败
        breaker.on_failure();
        assert_eq!(breaker.failure_count, 1);
        
        // 再次触发失败，应该进入打开状态
        breaker.on_failure();
        assert_eq!(breaker.failure_count, 2);
        assert_eq!(breaker.state(), CircuitBreakerState::Open);
        
        // 熔断器打开时不能执行
        assert!(!breaker.can_execute());
    }
    
    #[test]
    fn test_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_second: 2.0,
            burst_size: 1,
            ..Default::default()
        };
        
        let limiter = SlidingWindowRateLimiter::new(config);
        
        // 第一个请求应该成功
        assert!(tokio_test::block_on(limiter.acquire()).is_ok());
        
        // 第二个请求可能会被限制
        let result = tokio_test::block_on(limiter.acquire());
        // 结果取决于具体的实现细节
    }
    
    #[test]
    fn test_retry_handler() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            ..Default::default()
        };
        
        let mut handler = RetryHandler::new(config);
        
        // 模拟一个总是失败的操作
        let operation = |_attempt: u32, _context: ErrorContext| async {
            Err(ApiError::Network(reqwest::Error::new(
                reqwest::ErrorKind::Timeout,
                "Connection timeout"
            )))
        };
        
        let result = tokio_test::block_on(handler.execute_with_retry(operation, ErrorContext::default()));
        
        // 应该最终返回错误
        assert!(result.is_err());
        
        // 检查重试统计
        let stats = handler.get_stats();
        assert_eq!(stats.total_attempts, 3); // 应该尝试3次
    }
}
```

## 6.7 最佳实践和高级技巧

### 6.7.1 错误处理最佳实践

```rust
// 错误处理最佳实践

/// 1. 错误上下文和跟踪
#[derive(Debug, Clone)]
struct ErrorWithContext {
    error: ApiError,
    context: ErrorContext,
    chain: Vec<ErrorContext>,
}

impl ErrorWithContext {
    fn new(error: ApiError, context: ErrorContext) -> Self {
        Self {
            error,
            context,
            chain: Vec::new(),
        }
    }
    
    fn with_chain(mut self, prev_error: ErrorWithContext) -> Self {
        self.chain.push(prev_error.context);
        self
    }
    
    fn print_chain(&self) {
        println!("错误链:");
        for (i, ctx) in self.chain.iter().enumerate() {
            println!("  {}: {:?}", i + 1, ctx);
        }
        println!("  当前: {:?}", self.context);
        println!("  错误: {}", self.error);
    }
}

/// 2. 错误聚合器
struct ErrorAggregator {
    errors: Vec<ErrorWithContext>,
    max_errors: usize,
}

impl ErrorAggregator {
    fn new(max_errors: usize) -> Self {
        Self {
            errors: Vec::new(),
            max_errors,
        }
    }
    
    fn add_error(&mut self, error: ErrorWithContext) {
        if self.errors.len() < self.max_errors {
            self.errors.push(error);
        }
    }
    
    fn has_critical_error(&self) -> bool {
        self.errors.iter().any(|e| matches!(e.error, ApiError::CircuitBreakerOpen))
    }
    
    fn summarize(&self) -> String {
        let total = self.errors.len();
        let categories: HashMap<ErrorCategory, usize> = self.errors
            .iter()
            .map(|e| e.error.category())
            .fold(HashMap::new(), |mut acc, cat| {
                *acc.entry(cat).or_insert(0) += 1;
                acc
            });
        
        let mut summary = format!("总错误数: {}\n", total);
        for (category, count) in categories {
            summary.push_str(&format!("  {:?}: {} 个\n", category, count));
        }
        summary
    }
}

/// 3. 优雅降级策略
struct GracefulDegradation {
    primary_service: Arc<ApiClient>,
    fallback_service: Arc<ApiClient>,
    degradation_threshold: f64, // 0.0 到 1.0
    current_error_rate: f64,
}

impl GracefulDegradation {
    fn new(primary: Arc<ApiClient>, fallback: Arc<ApiClient>) -> Self {
        Self {
            primary_service: primary,
            fallback_service: fallback,
            degradation_threshold: 0.1, // 10%错误率触发降级
            current_error_rate: 0.0,
        }
    }
    
    async fn request_with_fallback<T>(&self, method: HttpMethod, endpoint: &str, body: Option<&impl Serialize>) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        // 首先尝试主要服务
        match self.primary_service.request_with_body(method.clone(), endpoint, body).await {
            Ok(result) => {
                self.update_success_rate();
                Ok(result)
            }
            Err(primary_error) => {
                // 记录错误并检查是否需要降级
                self.update_error_rate();
                
                if self.should_degrade() {
                    println!("主要服务失败，尝试降级服务");
                    match self.fallback_service.request_with_body(method, endpoint, body).await {
                        Ok(result) => {
                            println!("降级服务成功");
                            Ok(result)
                        }
                        Err(fallback_error) => {
                            // 两个服务都失败了，返回主服务错误
                            primary_error
                        }
                    }
                } else {
                    primary_error
                }
            }
           }
    
    fn should_degrade(&self) -> bool {
        self.current_error_rate > self.degradation_threshold
    }
    
    fn update_success_rate(&mut self) {
        // 更新错误率（简化实现）
        if self.current_error_rate > 0.0 {
            self.current_error_rate *= 0.9; // 成功时降低错误率
        }
    }
    
    fn update_error_rate(&mut self) {
        // 更新错误率（简化实现）
        self.current_error_rate = (self.current_error_rate + 0.1).min(1.0);
    }
    
    fn get_status(&self) -> DegradationStatus {
        DegradationStatus {
            current_error_rate: self.current_error_rate,
            threshold: self.degradation_threshold,
            should_degrade: self.should_degrade(),
        }
    }
}

#[derive(Debug, Clone)]
struct DegradationStatus {
    current_error_rate: f64,
    threshold: f64,
    should_degrade: bool,
}
```

### 6.7.2 异步错误处理高级模式

```rust
// 高级异步错误处理模式

/// 1. 批处理和错误聚合
async fn batch_process_with_aggregation(
    items: Vec<String>,
    processor: Arc<dyn BatchProcessor + Send + Sync>,
) -> Result<BatchResult, BatchError> {
    use tokio::sync::Semaphore;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let semaphore = Arc::new(Semaphore::new(10));
    let processed_count = Arc::new(AtomicUsize::new(0));
    let error_aggregator = Arc::new(Mutex::new(ErrorAggregator::new(100)));
    
    let mut handles = Vec::new();
    
    for (index, item) in items.into_iter().enumerate() {
        let semaphore = semaphore.clone();
        let processor = processor.clone();
        let processed_count = processed_count.clone();
        let error_aggregator = error_aggregator.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            match processor.process_batch_item(index, &item).await {
                Ok(_) => {
                    processed_count.fetch_add(1, Ordering::Relaxed);
                    (index, Ok(()))
                }
                Err(error) => {
                    let error_with_context = ErrorWithContext::new(
                        error,
                        ErrorContext {
                            request_id: Some(format!("item_{}", index)),
                            ..Default::default()
                        }
                    );
                    
                    let mut aggregator = error_aggregator.lock().await;
                    aggregator.add_error(error_with_context);
                    (index, Err(()))
                }
            }
        });
        
        handles.push(handle);
    }
    
    // 等待所有任务完成
    let mut results = Vec::new();
    for handle in handles {
        if let Ok((index, result)) = handle.await {
            results.push((index, result));
        }
    }
    
    // 检查结果
    let mut aggregator = error_aggregator.lock().await;
    let error_count = aggregator.errors.len();
    
    if aggregator.has_critical_error() {
        return Err(BatchError::Critical(aggregator.errors));
    }
    
    if error_count > 0 {
        return Err(BatchError::Partial {
            success_count: processed_count.load(Ordering::Relaxed),
            error_count,
            errors: aggregator.errors.clone(),
        });
    }
    
    Ok(BatchResult {
        total_processed: processed_count.load(Ordering::Relaxed),
        errors: Vec::new(),
    })
}

/// 批处理错误类型
#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("关键错误: {0:?}")]
    Critical(Vec<ErrorWithContext>),
    
    #[error("部分成功: 成功 {success_count}, 失败 {error_count}")]
    Partial {
        success_count: usize,
        error_count: usize,
        errors: Vec<ErrorWithContext>,
    },
}

/// 批处理结果
#[derive(Debug)]
struct BatchResult {
    total_processed: usize,
    errors: Vec<ErrorWithContext>,
}

/// 批处理器trait
#[async_trait::async_trait]
pub trait BatchProcessor: Send + Sync {
    async fn process_batch_item(&self, index: usize, item: &str) -> Result<(), ApiError>;
}

/// 2. 错误恢复策略
enum RecoveryStrategy {
    RetryWithBackoff,
    UseCache,
    CallFallbackService,
    SkipAndContinue,
    FailFast,
}

impl RecoveryStrategy {
    fn select_strategy(error: &ApiError) -> Self {
        match error {
            ApiError::Network(_) | ApiError::Timeout(_) => RecoveryStrategy::RetryWithBackoff,
            ApiError::Http { status, .. } if status.is_server_error() => RecoveryStrategy::UseCache,
            ApiError::RateLimit { .. } => RecoveryStrategy::RetryWithBackoff,
            ApiError::CircuitBreakerOpen => RecoveryStrategy::CallFallbackService,
            ApiError::Validation(_) | ApiError::Business(_) => RecoveryStrategy::FailFast,
            _ => RecoveryStrategy::SkipAndContinue,
        }
    }
}

async fn recover_from_error<T>(
    original_result: Result<T, ApiError>,
    recovery_context: &RecoveryContext,
) -> Result<T, ApiError> {
    match original_result {
        Ok(data) => Ok(data),
        Err(error) => {
            let strategy = RecoveryStrategy::select_strategy(&error);
            
            match strategy {
                RecoveryStrategy::RetryWithBackoff => {
                    // 执行重试
                    let delay = error.recommended_delay().unwrap_or(Duration::from_millis(100));
                    tokio::time::sleep(delay).await;
                    Err(error)
                }
                RecoveryStrategy::UseCache => {
                    // 尝试从缓存获取
                    if let Some(cached_data) = &recovery_context.cached_data {
                        Ok(cached_data.clone())
                    } else {
                        Err(error)
                    }
                }
                RecoveryStrategy::CallFallbackService => {
                    // 使用备用服务
                    if let Some(fallback_result) = &recovery_context.fallback_result {
                        fallback_result.clone()
                    } else {
                        Err(error)
                    }
                }
                RecoveryStrategy::SkipAndContinue => {
                    // 跳过错误（适用于批量操作）
                    if let Some(default_data) = &recovery_context.default_data {
                        Ok(default_data.clone())
                    } else {
                        Err(error)
                    }
                }
                RecoveryStrategy::FailFast => Err(error),
            }
        }
    }
}

/// 恢复上下文
struct RecoveryContext {
    cached_data: Option<serde_json::Value>,
    fallback_result: Option<serde_json::Value>,
    default_data: Option<serde_json::Value>,
}
```

### 6.7.3 监控和告警

```rust
// 错误监控和告警系统

/// 错误监控器
pub struct ErrorMonitor {
    config: MonitorConfig,
    metrics: Arc<Metrics>,
    alerts: Arc<AlertManager>,
    history: Arc<RwLock<Vec<ErrorRecord>>>,
}

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub error_rate_threshold: f64,
    pub error_count_threshold: u64,
    pub time_window: Duration,
    pub alert_channels: Vec<AlertChannel>,
}

#[derive(Debug, Clone)]
pub enum AlertChannel {
    Email { smtp_server: String, recipients: Vec<String> },
    Webhook { url: String, headers: HashMap<String, String> },
    Slack { webhook_url: String, channel: String },
}

impl ErrorMonitor {
    pub fn new(config: MonitorConfig, metrics: Arc<Metrics>) -> Self {
        Self {
            config,
            metrics,
            alerts: Arc::new(AlertManager::new()),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn record_error(&self, error: &ApiError) {
        // 记录错误
        let record = ErrorRecord::new(error);
        let mut history = self.history.write().await;
        history.push(record);
        
        // 清理过期记录
        self.cleanup_old_records(&mut history).await;
        
        // 检查是否需要告警
        self.check_alerts().await;
    }
    
    async fn check_alerts(&self) {
        let history = self.history.read().await;
        let recent_errors = self.get_recent_errors(&history).await;
        
        if recent_errors.len() > self.config.error_count_threshold as usize {
            self.trigger_alert(AlertType::HighErrorCount {
                count: recent_errors.len(),
                threshold: self.config.error_count_threshold,
            }).await;
        }
        
        let error_rate = self.calculate_error_rate(&recent_errors);
        if error_rate > self.config.error_rate_threshold {
            self.trigger_alert(AlertType::HighErrorRate {
                rate: error_rate,
                threshold: self.config.error_rate_threshold,
            }).await;
        }
    }
    
    async fn get_recent_errors(&self, history: &[ErrorRecord]) -> Vec<&ErrorRecord> {
        let now = SystemTime::now();
        history.iter()
            .filter(|record| {
                now.duration_since(record.timestamp)
                    .map(|duration| duration < self.config.time_window)
                    .unwrap_or(false)
            })
            .collect()
    }
    
    fn calculate_error_rate(&self, errors: Vec<&ErrorRecord>) -> f64 {
        if errors.is_empty() {
            return 0.0;
        }
        
        let time_span = errors
            .iter()
            .map(|r| r.timestamp)
            .min()
            .and_then(|min_time| {
                errors
                    .iter()
                    .map(|r| r.timestamp)
                    .max()
                    .map(|max_time| max_time.duration_since(min_time))
            })
            .unwrap_or_else(|| Duration::from_secs(1));
        
        errors.len() as f64 / time_span.as_secs_f64()
    }
    
    async fn trigger_alert(&self, alert_type: AlertType) {
        for channel in &self.config.alert_channels {
            match self.send_alert(channel, &alert_type).await {
                Ok(_) => println!("告警发送成功: {:?}", alert_type),
                Err(e) => eprintln!("告警发送失败: {:?}", e),
            }
        }
    }
    
    async fn send_alert(&self, channel: &AlertChannel, alert: &AlertType) -> Result<(), AlertError> {
        match channel {
            AlertChannel::Email { smtp_server, recipients } => {
                // 简化实现：实际应该使用SMTP库
                println!("发送邮件告警到 {:?}: {:?}", recipients, alert);
                Ok(())
            }
            AlertChannel::Webhook { url, headers } => {
                // 使用reqwest发送webhook
                let client = reqwest::Client::new();
                let response = client
                    .post(url)
                    .headers(headers.clone())
                    .json(alert)
                    .send()
                    .await
                    .map_err(|e| AlertError::Network(e))?;
                
                if !response.status().is_success() {
                    return Err(AlertError::Http(response.status()));
                }
                
                Ok(())
            }
            AlertChannel::Slack { webhook_url, channel } => {
                // 发送Slack消息
                let client = reqwest::Client::new();
                let payload = SlackPayload {
                    channel,
                    text: format!("告警: {:?}", alert),
                    username: "ErrorMonitor",
                    icon_emoji: ":warning:",
                };
                
                client
                    .post(webhook_url)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| AlertError::Network(e))?;
                
                Ok(())
            }
        }
    }
    
    async fn cleanup_old_records(&self, history: &mut Vec<ErrorRecord>) {
        let now = SystemTime::now();
        history.retain(|record| {
            now.duration_since(record.timestamp)
                .map(|duration| duration < self.config.time_window * 2) // 保留两倍时间窗口
                .unwrap_or(false)
        });
    }
}

/// 错误记录
#[derive(Debug, Clone)]
struct ErrorRecord {
    timestamp: SystemTime,
    error: ApiError,
    context: ErrorContext,
}

/// 告警类型
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum AlertType {
    #[serde(rename = "high_error_count")]
    HighErrorCount { count: usize, threshold: u64 },
    
    #[serde(rename = "high_error_rate")]
    HighErrorRate { rate: f64, threshold: f64 },
    
    #[serde(rename = "circuit_breaker_opened")]
    CircuitBreakerOpened,
    
    #[serde(rename = "service_unavailable")]
    ServiceUnavailable { reason: String },
}

/// 告警管理器
struct AlertManager;

impl AlertManager {
    fn new() -> Self {
        Self
    }
}

/// 告警错误
#[derive(Debug, thiserror::Error)]
pub enum AlertError {
    #[error("网络错误: {0}")]
    Network(reqwest::Error),
    
    #[error("HTTP错误: {0}")]
    Http(reqwest::StatusCode),
    
    #[error("配置错误: {0}")]
    Config(String),
    
    #[error("发送失败: {0}")]
    SendFailed(String),
}

/// Slack消息负载
#[derive(Debug, Serialize)]
struct SlackPayload {
    channel: String,
    text: String,
    username: String,
    icon_emoji: String,
}
```

## 6.8 性能优化和调试

```rust
// 性能优化和调试工具

/// 性能分析器
pub struct PerformanceProfiler {
    operations: Arc<RwLock<HashMap<String, OperationStats>>>,
    enabled: bool,
}

#[derive(Debug, Clone)]
struct OperationStats {
    call_count: u64,
    total_time: Duration,
    min_time: Option<Duration>,
    max_time: Option<Duration>,
    error_count: u64,
}

impl PerformanceProfiler {
    fn new(enabled: bool) -> Self {
        Self {
            operations: Arc::new(RwLock::new(HashMap::new())),
            enabled,
        }
    }
    
    async fn record_operation(&self, name: &str, duration: Duration, success: bool) {
        if !self.enabled {
            return;
        }
        
        let mut stats_map = self.operations.write().await;
        let stats = stats_map.entry(name.to_string()).or_insert_with(|| OperationStats {
            call_count: 0,
            total_time: Duration::from_millis(0),
            min_time: None,
            max_time: None,
            error_count: 0,
        });
        
        stats.call_count += 1;
        stats.total_time += duration;
        stats.min_time = Some(stats.min_time.map_or(duration, |min| min.min(duration)));
        stats.max_time = Some(stats.max_time.map_or(duration, |max| max.max(duration)));
        
        if !success {
            stats.error_count += 1;
        }
    }
    
    async fn get_report(&self) -> PerformanceReport {
        let stats_map = self.operations.read().await;
        
        let mut report = PerformanceReport::new();
        
        for (name, stats) in stats_map.iter() {
            let avg_time = if stats.call_count > 0 {
                Duration::from_nanos(stats.total_time.as_nanos() as u64 / stats.call_count)
            } else {
                Duration::from_millis(0)
            };
            
            let success_rate = if stats.call_count > 0 {
                (stats.call_count - stats.error_count) as f64 / stats.call_count as f64
            } else {
                1.0
            };
            
            report.add_operation(OperationReport {
                name: name.clone(),
                call_count: stats.call_count,
                total_time: stats.total_time,
                avg_time,
                min_time: stats.min_time,
                max_time: stats.max_time,
                success_rate,
                error_count: stats.error_count,
            });
        }
        
        report
    }
}

/// 性能报告
#[derive(Debug)]
struct PerformanceReport {
    operations: Vec<OperationReport>,
    total_operations: u64,
    total_time: Duration,
}

impl PerformanceReport {
    fn new() -> Self {
        Self {
            operations: Vec::new(),
            total_operations: 0,
            total_time: Duration::from_millis(0),
        }
    }
    
    fn add_operation(&mut self, operation: OperationReport) {
        self.operations.push(operation);
        self.total_operations += 1;
    }
    
    fn generate_text(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("=== 性能报告 ===\n"));
        report.push_str(&format!("总操作数: {}\n", self.total_operations));
        report.push_str(&format!("总耗时: {:?}\n\n", self.total_time));
        
        for op in &self.operations {
            report.push_str(&format!("操作: {}\n", op.name));
            report.push_str(&format!("  调用次数: {}\n", op.call_count));
            report.push_str(&format!("  平均耗时: {:?}\n", op.avg_time));
            report.push_str(&format!("  最小耗时: {:?}\n", op.min_time));
            report.push_str(&format!("  最大耗时: {:?}\n", op.max_time));
            report.push_str(&format!("  成功率: {:.2}%\n", op.success_rate * 100.0));
            report.push_str(&format!("  错误数: {}\n", op.error_count));
            report.push_str("\n");
        }
        
        report
    }
}

#[derive(Debug)]
struct OperationReport {
    name: String,
    call_count: u64,
    total_time: Duration,
    avg_time: Duration,
    min_time: Option<Duration>,
    max_time: Option<Duration>,
    success_rate: f64,
    error_count: u64,
}

/// 调试工具
pub struct DebugTools {
    profiler: PerformanceProfiler,
    trace_collector: TraceCollector,
}

impl DebugTools {
    fn new() -> Self {
        Self {
            profiler: PerformanceProfiler::new(true),
            trace_collector: TraceCollector::new(),
        }
    }
    
    async fn start_trace(&self, trace_id: &str) -> TraceContext {
        let span = self.trace_collector.start_span(trace_id);
        TraceContext::new(span)
    }
    
    async fn record_metric(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        self.trace_collector.record_metric(name, value, tags).await;
    }
}

/// 分布式跟踪收集器
struct TraceCollector {
    spans: Arc<RwLock<Vec<TraceSpan>>>,
}

#[derive(Debug, Clone)]
struct TraceSpan {
    id: String,
    parent_id: Option<String>,
    operation: String,
    start_time: SystemTime,
    end_time: Option<SystemTime>,
    tags: HashMap<String, String>,
    metrics: Vec<Metric>,
}

#[derive(Debug, Clone)]
struct Metric {
    name: String,
    value: f64,
    tags: HashMap<String, String>,
    timestamp: SystemTime,
}

impl TraceCollector {
    fn new() -> Self {
        Self {
            spans: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    fn start_span(&self, operation: &str) -> String {
        let span_id = uuid::Uuid::new_v4().to_string();
        let span = TraceSpan {
            id: span_id.clone(),
            parent_id: None,
            operation: operation.to_string(),
            start_time: SystemTime::now(),
            end_time: None,
            tags: HashMap::new(),
            metrics: Vec::new(),
        };
        
        let mut spans = self.spans.blocking_write();
        spans.push(span);
        
        span_id
    }
    
    async fn end_span(&self, span_id: &str) {
        let mut spans = self.spans.write().await;
        
        if let Some(span) = spans.iter_mut().find(|s| s.id == span_id) {
            span.end_time = Some(SystemTime::now());
        }
    }
    
    async fn record_metric(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value,
            tags,
            timestamp: SystemTime::now(),
        };
        
        // 简化实现：记录到最后一个活跃span
        let mut spans = self.spans.write().await;
        if let Some(span) = spans.iter_mut().last() {
            span.metrics.push(metric);
        }
    }
    
    fn get_trace(&self) -> Vec<TraceSpan> {
        self.spans.blocking_read().clone()
    }
}

/// 跟踪上下文
struct TraceContext {
    span_id: String,
    collector: Arc<TraceCollector>,
}

impl TraceContext {
    fn new(span_id: String) -> Self {
        Self {
            span_id,
            collector: Arc::new(TraceCollector::new()), // 简化实现
        }
    }
    
    fn add_tag(&self, key: &str, value: &str) {
        // 简化实现
    }
    
    fn add_metric(&self, name: &str, value: f64) {
        // 简化实现
    }
}

impl Drop for TraceContext {
    fn drop(&mut self) {
        let collector = self.collector.clone();
        let span_id = self.span_id.clone();
        
        tokio::spawn(async move {
            collector.end_span(&span_id).await;
        });
    }
}
```

## 6.9 总结

在本章中，我们深入学习了Rust的错误处理机制，并通过构建一个企业级API客户端库来实践这些概念。主要内容包括：

### 6.9.1 核心概念

1. **Option<T>和Result<T, E>**：Rust中处理可选值和可能失败操作的基础
2. **错误传播**：`?`操作符和Result链式操作
3. **自定义错误类型**：为特定领域定义有意义的错误类型
4. **错误分类和恢复**：根据错误类型选择合适的恢复策略

### 6.9.2 实战项目亮点

1. **细粒度错误分类**：网络错误、HTTP错误、认证错误、业务错误等
2. **重试机制**：指数退避、抖动算法、智能重试判断
3. **熔断器模式**：防止级联故障，提高系统稳定性
4. **限流控制**：滑动窗口和令牌桶算法
5. **监控和告警**：错误率监控、实时告警

### 6.9.3 最佳实践

1. **显式错误处理**：不忽略任何可能的错误
2. **错误上下文**：记录足够的调试信息
3. **优雅降级**：主要服务失败时使用备用服务
4. **性能监控**：跟踪操作耗时和成功率
5. **告警机制**：及时发现和响应问题

通过这个项目，我们展示了如何在实际企业环境中应用Rust的错误处理特性来构建可靠、可维护的异步网络应用。错误处理不仅仅是异常捕获，更是系统设计和架构决策的重要组成部分。

这个API客户端库可以作为企业级网络应用的基础框架，支持：
- 高并发请求处理
- 智能错误恢复
- 实时性能监控
- 多级告警机制
- 完整的错误跟踪

在下一章中，我们将学习Rust的集合类型和数据结构，进一步扩展我们的知识体系。
