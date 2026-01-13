# 第五章：泛型与特征

## 5.1 章节概述

泛型（Generics）和特征（Traits）是Rust语言中最重要的抽象机制之一。它们允许我们编写既灵活又类型安全的代码，通过抽象出通用的算法和数据结构，而不需要为每种具体类型编写重复的代码。

在本章中，我们将通过构建一个**通用数据处理与分析框架**（dataflow-framework）来深入学习这些概念。这个框架将展示如何在实际项目中应用泛型和特征来创建可扩展、可维护的企业级系统。

### 学习目标

完成本章学习后，您将能够：

- 理解泛型的基本概念和语法
- 掌握特征的定义、实现和使用
- 学会特征边界和泛型约束
- 掌握特征对象和动态分发的概念
- 理解关联类型和泛型关联类型
- 学会如何使用泛型和特征设计可扩展的架构
- 构建一个完整的数据处理框架

### 实战项目预览

本章实战项目将构建一个通用数据处理框架，支持：
- 多种数据源（文件、数据库、API、实时流）
- 灵活的数据处理管道
- 多种输出格式
- 性能优化和并发处理

## 5.2 泛型基础

### 5.2.1 什么是泛型

泛型允许我们编写可以处理多种数据类型的代码，而不需要为每种类型单独实现。通过泛型，我们可以创建：
- 泛型函数
- 泛型结构体
- 泛型枚举
- 泛型方法

### 5.2.2 泛型函数

让我们从一个简单的泛型函数开始：

```rust
// 泛型函数示例
fn compare<T>(a: T, b: T) -> i32 
where
    T: PartialOrd,
{
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

// 使用泛型函数
fn main() {
    println!("比较整数: {}", compare(5, 3));  // 输出: 1
    println!("比较浮点数: {}", compare(3.14, 2.71));  // 输出: 1
    println!("比较字符串: {}", compare("abc", "xyz"));  // 输出: -1
}
```

在上面的例子中：
- `T` 是类型参数，表示函数可以处理任何类型
- `where T: PartialOrd` 是特征边界，指定T必须实现`PartialOrd`特征
- 这样函数就能对所有实现了比较操作符的类型工作

### 5.2.3 泛型结构体

```rust
// 泛型结构体
#[derive(Debug, Clone)]
struct Container<T> {
    items: Vec<T>,
    capacity: usize,
}

impl<T> Container<T> {
    fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    fn push(&mut self, item: T) {
        if self.items.len() < self.capacity {
            self.items.push(item);
        }
    }
    
    fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
    
    fn len(&self) -> usize {
        self.items.len()
    }
}

// 泛型结构体的方法
impl<T: std::fmt::Display> Container<T> {
    fn print_all(&self) {
        for item in &self.items {
            println!("{}", item);
        }
    }
}

fn main() {
    let mut int_container = Container::new(3);
    int_container.push(1);
    int_container.push(2);
    int_container.push(3);
    
    println!("整数容器内容: {:?}", int_container.items);
    println!("容器大小: {}", int_container.len());
    
    let mut string_container = Container::new(2);
    string_container.push("hello");
    string_container.push("world");
    string_container.print_all();  // 需要Display trait
}
```

### 5.2.4 泛型枚举

```rust
// 泛型枚举示例
#[derive(Debug, Clone)]
enum Result<T, E> {
    Ok(T),
    Err(E),
}

#[derive(Debug, Clone)]
enum Option<T> {
    Some(T),
    None,
}

// 实用函数
impl<T, E> Result<T, E> {
    fn is_ok(&self) -> bool {
        matches!(self, Ok(_))
    }
    
    fn is_err(&self) -> bool {
        matches!(self, Err(_))
    }
}

impl<T> Option<T> {
    fn unwrap(self) -> T {
        match self {
            Some(value) => value,
            None => panic!("Called Option::unwrap() on a None value"),
        }
    }
    
    fn unwrap_or(self, default: T) -> T {
        match self {
            Some(value) => value,
            None => default,
        }
    }
}

fn main() {
    let success: Result<i32, &str> = Ok(42);
    let failure: Result<i32, &str> = Err("something went wrong");
    
    let present: Option<i32> = Some(100);
    let absent: Option<i32> = None;
    
    println!("成功: {}, 失败: {}", success.is_ok(), failure.is_err());
    println!("存在: {}, 缺失: {}", present.is_some(), absent.is_none());
    println!("unwrap 结果: {}", present.unwrap());
    println!("unwrap_or 结果: {}", absent.unwrap_or(0));
}
```

## 5.3 特征基础

### 5.3.1 什么是特征

特征（Trait）定义了一组可以由不同类型实现的方法。它们类似于其他语言中的接口，但功能更强大。

### 5.3.2 定义和使用特征

```rust
// 定义一个特征
pub trait Drawable {
    fn draw(&self) -> String;
    
    // 默认实现
    fn area(&self) -> f64 {
        0.0  // 默认面积为0
    }
    
    // 可以有其他方法
    fn is_visible(&self) -> bool {
        true  // 默认可见
    }
}

// 实现特征的类型
struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

struct Triangle {
    base: f64,
    height: f64,
}

// 为每个类型实现Drawable特征
impl Drawable for Circle {
    fn draw(&self) -> String {
        format!("画一个半径为 {} 的圆形", self.radius)
    }
    
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl Drawable for Rectangle {
    fn draw(&self) -> String {
        format!("画一个 {}x{} 的矩形", self.width, self.height)
    }
    
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl Drawable for Triangle {
    fn draw(&self) -> String {
        format!("画一个底边 {}，高 {} 的三角形", self.base, self.height)
    }
    
    fn area(&self) -> f64 {
        (self.base * self.height) / 2.0
    }
}

// 函数接受实现了特征的类型
fn draw_shape<T: Drawable>(shape: &T) {
    println!("{}", shape.draw());
    println!("面积: {:.2}", shape.area());
    println!("可见: {}", shape.is_visible());
    println!("---");
}

fn main() {
    let circle = Circle { radius: 5.0 };
    let rectangle = Rectangle { width: 4.0, height: 6.0 };
    let triangle = Triangle { base: 3.0, height: 4.0 };
    
    draw_shape(&circle);
    draw_shape(&rectangle);
    draw_shape(&triangle);
}
```

### 5.3.3 特征作为参数

```rust
// 使用特征作为函数参数
fn summarize_shape(shape: &impl Drawable) -> String {
    format!(
        "这是一个图形，面积是 {:.2}，状态是 {}",
        shape.area(),
        if shape.is_visible() { "可见" } else { "隐藏" }
    )
}

// 多个特征约束
fn create_summary<T: Drawable + Clone>(shape: &T) -> String {
    // 可以调用两个特征的方法
    let original = format!("原始: {}", shape.draw());
    let clone = format!("克隆: {}", shape.clone().draw());
    format!("{}\n{}", original, clone)
}

// 返回实现了特征的类型
fn create_circle() -> impl Drawable {
    Circle { radius: 3.0 }
}

// 泛型约束语法
fn complex_draw<T>(shapes: &[T]) -> Vec<String>
where
    T: Drawable,
{
    shapes.iter().map(|shape| shape.draw()).collect()
}
```

### 5.3.4 特征与泛型结合

```rust
// 泛型特征
trait Calculate {
    type Output;  // 关联类型
    
    fn calculate(&self) -> Self::Output;
}

struct MathOperations<T> {
    value: T,
}

impl<T> Calculate for MathOperations<T>
where
    T: std::ops::Add<Output = T>
    + std::ops::Sub<Output = T>
    + std::ops::Mul<Output = T>
    + Copy,
{
    type Output = T;
    
    fn calculate(&self) -> Self::Output {
        // 使用泛型进行数学运算
        let a = self.value;
        let b = self.value;
        (a + b) * b  // 使用实现了这些运算的类型
    }
}

// 泛型特征约束
fn process_calculate<T>(op: &MathOperations<T>) -> T
where
    T: std::ops::Add<Output = T>
    + std::ops::Sub<Output = T>
    + std::ops::Mul<Output = T>
    + Copy
    + std::fmt::Debug,
{
    let result = op.calculate();
    println!("操作结果: {:?}", result);
    result
}

fn main() {
    let int_op = MathOperations { value: 5 };
    let float_op = MathOperations { value: 3.14 };
    
    let int_result = process_calculate(&int_op);  // 40
    let float_result = process_calculate(&float_op);  // 19.4784
}
```

## 5.4 特征边界高级用法

### 5.4.1 多个特征约束

```rust
// 定义多个特征
trait Printable {
    fn print(&self);
}

trait Cloneable {
    fn clone_me(&self) -> Self;
}

trait Validatable {
    fn is_valid(&self) -> bool;
}

// 使用多个特征约束
fn process_item<T>(item: &T) 
where
    T: Printable + Cloneable + Validatable,
{
    if item.is_valid() {
        item.print();
        let cloned = item.clone_me();
        cloned.print();
    }
}

// 或者使用 + 语法
fn process_item_shorthand<T: Printable + Cloneable + Validatable>(item: &T) {
    // 同样的实现
}

// 复杂约束示例
fn complex_processing<T, U, V>(item1: T, item2: U, item3: V) 
where
    T: std::fmt::Display + Cloneable,
    U: Printable + Validatable,
    V: Cloneable + Validatable + std::fmt::Debug,
{
    println!("项目1: {}", item1);
    if item2.is_valid() {
        item2.print();
    }
    println!("项目3: {:?}", item3);
}
```

### 5.4.2 特征对象

```rust
// 特征对象允许我们使用不同类型的相同特征
fn demonstrate_trait_objects() {
    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Rectangle { width: 2.0, height: 3.0 }),
        Box::new(Triangle { base: 4.0, height: 5.0 }),
    ];
    
    // 动态分派 - 运行时决定调用哪个方法
    for shape in &shapes {
        println!("{}", shape.draw());
        println!("面积: {:.2}", shape.area());
    }
}

// 特征对象的返回类型
fn create_shape(shape_type: &str) -> Box<dyn Drawable> {
    match shape_type {
        "circle" => Box::new(Circle { radius: 2.0 }),
        "rectangle" => Box::new(Rectangle { width: 3.0, height: 4.0 }),
        "triangle" => Box::new(Triangle { base: 5.0, height: 6.0 }),
        _ => Box::new(Circle { radius: 1.0 }),
    }
}

// 特征对象作为参数
fn draw_all_shapes(shapes: &[Box<dyn Drawable>]) {
    for (i, shape) in shapes.iter().enumerate() {
        println!("形状 {}: {}", i + 1, shape.draw());
    }
}
```

### 5.4.3 特征对象 vs 泛型

```rust
// 泛型方式 - 编译时分派，性能更好
fn draw_shapes_generic<T>(shapes: &[T])
where
    T: Drawable,
{
    for shape in shapes {
        shape.draw();
    }
}

// 特征对象方式 - 运行时动态分派，更灵活
fn draw_shapes_trait_object(shapes: &[Box<dyn Drawable>]) {
    for shape in shapes {
        shape.draw();
    }
}

// 使用泛型
fn main() {
    let circles = vec![Circle { radius: 1.0 }, Circle { radius: 2.0 }];
    // draw_shapes_generic(&circles);  // 只处理同一种类型
    
    let mixed_shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Rectangle { width: 2.0, height: 3.0 }),
    ];
    // draw_shapes_trait_object(&mixed_shapes);  // 可以处理不同类型
}
```

## 5.5 实战项目：数据流框架架构设计

现在让我们开始构建实战项目。首先，我们需要设计数据处理框架的核心架构。

### 5.5.1 框架概述

我们的数据流框架将使用以下设计模式：

1. **流水线模式**：数据从源到处理到输出的完整流程
2. **插件架构**：可插拔的处理器和适配器
3. **特征约束**：确保组件间的类型安全交互
4. **泛型实现**：支持多种数据类型和格式

### 5.5.2 核心特征设计

```rust
// 核心特征定义
use std::fmt::Debug;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// 数据源特征
pub trait DataSource<T> {
    type Error: Debug;
    
    /// 读取所有数据
    fn read(&self) -> Result<Vec<T>, Self::Error>;
    
    /// 读取流数据（用于大文件）
    fn read_stream(&self) -> Result<Box<dyn Iterator<Item = Result<T, Self::Error>>>, Self::Error>;
    
    /// 获取数据计数
    fn count(&self) -> Result<u64, Self::Error>;
    
    /// 检查数据源是否有效
    fn is_valid(&self) -> bool;
}

// 数据处理器特征
pub trait DataProcessor<T, U> {
    type Error: Debug;
    
    /// 批量处理数据
    fn process(&self, data: Vec<T>) -> Result<Vec<U>, Self::Error>;
    
    /// 单项处理数据
    fn process_item(&self, item: T) -> Result<U, Self::Error>;
    
    /// 处理数据流
    fn process_stream(&self, stream: Box<dyn Iterator<Item = T>>) -> Result<Box<dyn Iterator<Item = Result<U, Self::Error>>>, Self::Error>;
    
    /// 获取处理器信息
    fn info(&self) -> ProcessorInfo;
}

// 数据输出特征
pub trait DataSink<T> {
    type Error: Debug;
    
    /// 写入数据
    fn write(&self, data: Vec<T>) -> Result<(), Self::Error>;
    
    /// 写入数据流
    fn write_stream(&self, stream: Box<dyn Iterator<Item = T>>) -> Result<(), Self::Error>;
    
    /// 刷新输出
    fn flush(&self) -> Result<(), Self::Error>;
    
    /// 获取输出统计
    fn stats(&self) -> SinkStats;
}

// 处理器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub input_type: String,
    pub output_type: String,
    pub performance_metrics: PerformanceMetrics,
}

// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub processing_time_ms: u64,
    pub throughput_items_per_second: f64,
    pub memory_usage_mb: f64,
}

// 接收器统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkStats {
    pub total_written: u64,
    pub write_time_ms: u64,
    pub last_write: Option<std::time::SystemTime>,
}
```

### 5.5.3 数据管道实现

```rust
// 数据管道
pub struct DataPipeline<S, P, K> 
where
    S: DataSource<serde_json::Value>,
    P: DataProcessor<serde_json::Value, serde_json::Value>,
    K: DataSink<serde_json::Value>,
{
    source: S,
    processor: P,
    sink: K,
    config: PipelineConfig,
    metrics: PipelineMetrics,
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub batch_size: usize,
    pub parallel_processing: bool,
    pub max_concurrency: usize,
    pub enable_cache: bool,
    pub cache_ttl_seconds: u64,
    pub retry_attempts: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    pub start_time: std::time::Instant,
    pub items_processed: u64,
    pub items_failed: u64,
    pub bytes_processed: u64,
    pub total_time_ms: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            parallel_processing: true,
            max_concurrency: 4,
            enable_cache: true,
            cache_ttl_seconds: 3600,
            retry_attempts: 3,
            timeout_seconds: 300,
        }
    }
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            items_processed: 0,
            items_failed: 0,
            bytes_processed: 0,
            total_time_ms: 0,
        }
    }
}

impl<S, P, K> DataPipeline<S, P, K> 
where
    S: DataSource<serde_json::Value>,
    P: DataProcessor<serde_json::Value, serde_json::Value>,
    K: DataSink<serde_json::Value>,
{
    /// 创建新的数据管道
    pub fn new(source: S, processor: P, sink: K) -> Self {
        Self {
            source,
            processor,
            sink,
            config: PipelineConfig::default(),
            metrics: PipelineMetrics::default(),
        }
    }
    
    /// 使用自定义配置创建管道
    pub fn with_config(source: S, processor: P, sink: K, config: PipelineConfig) -> Self {
        Self {
            source,
            processor,
            sink,
            config,
            metrics: PipelineMetrics::default(),
        }
    }
    
    /// 运行数据处理管道
    pub async fn run(&mut self) -> Result<PipelineMetrics, PipelineError> {
        println!("开始运行数据处理管道...");
        
        let start_time = std::time::Instant::now();
        
        // 验证组件
        self.validate_pipeline()?;
        
        // 选择处理模式
        if self.config.parallel_processing {
            self.run_parallel().await
        } else {
            self.run_sequential().await
        }?;
        
        // 更新指标
        self.metrics.total_time_ms = start_time.elapsed().as_millis() as u64;
        
        println!("管道运行完成，处理了 {} 项数据", self.metrics.items_processed);
        Ok(self.metrics.clone())
    }
    
    /// 顺序执行处理
    async fn run_sequential(&mut self) -> Result<(), PipelineError> {
        // 读取数据
        let data = self.source.read()
            .map_err(PipelineError::SourceError)?;
        
        if data.is_empty() {
            println!("没有数据需要处理");
            return Ok(());
        }
        
        println!("读取到 {} 项数据", data.len());
        
        // 批量处理数据
        let chunks = data.chunks(self.config.batch_size);
        
        for (chunk_index, chunk) in chunks.enumerate() {
            let chunk_vec: Vec<_> = chunk.to_vec();
            
            // 处理数据块
            let processed = self.processor.process(chunk_vec)
                .map_err(PipelineError::ProcessorError)?;
            
            // 输出结果
            self.sink.write(processed)
                .map_err(PipelineError::SinkError)?;
            
            // 更新统计
            self.metrics.items_processed += chunk_vec.len() as u64;
            
            // 进度报告
            if (chunk_index + 1) % 10 == 0 {
                println!("已处理 {} 个批次", chunk_index + 1);
            }
        }
        
        // 刷新输出
        self.sink.flush()
            .map_err(PipelineError::SinkError)?;
            
        Ok(())
    }
    
    /// 并行执行处理
    async fn run_parallel(&mut self) -> Result<(), PipelineError> {
        use tokio::task;
        use std::sync::Arc;
        
        // 读取数据
        let data = self.source.read()
            .map_err(PipelineError::SourceError)?;
        
        if data.is_empty() {
            println!("没有数据需要处理");
            return Ok(());
        }
        
        // 分块处理
        let chunks: Vec<_> = data.chunks(self.config.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();
        
        println!("开始并行处理 {} 个数据块", chunks.len());
        
        // 并行处理数据块
        let mut handles = Vec::new();
        let max_concurrency = self.config.max_concurrency;
        
        for chunk in chunks {
            if handles.len() >= max_concurrency {
                // 等待一个任务完成
                let handle = handles.remove(0);
                handle.await.map_err(|_| PipelineError::TaskJoinError)?;
            }
            
            let processor = self.processor;
            let sink = &self.sink;
            let config = self.config.clone();
            
            let handle = task::spawn(async move {
                // 处理数据
                let processed = processor.process(chunk)
                    .map_err(PipelineError::ProcessorError)?;
                
                // 写入结果
                sink.write(processed)
                    .map_err(PipelineError::SinkError)?;
                
                Ok(())
            });
            
            handles.push(handle);
        }
        
        // 等待所有任务完成
        for handle in handles {
            handle.await.map_err(|_| PipelineError::TaskJoinError)??;
        }
        
        // 刷新输出
        self.sink.flush()
            .map_err(PipelineError::SinkError)?;
            
        self.metrics.items_processed = data.len() as u64;
        Ok(())
    }
    
    /// 验证管道组件
    fn validate_pipeline(&self) -> Result<(), PipelineError> {
        // 验证源数据源
        if !self.source.is_valid() {
            return Err(PipelineError::SourceInvalid);
        }
        
        // 验证处理器信息
        let info = self.processor.info();
        if info.input_type.is_empty() || info.output_type.is_empty() {
            return Err(PipelineError::InvalidProcessorInfo);
        }
        
        Ok(())
    }
    
    /// 获取管道状态
    pub fn get_status(&self) -> PipelineStatus {
        PipelineStatus {
            is_running: false, // 简化为非运行状态
            items_processed: self.metrics.items_processed,
            items_failed: self.metrics.items_failed,
            total_time_ms: self.metrics.total_time_ms,
            throughput_per_second: if self.metrics.total_time_ms > 0 {
                (self.metrics.items_processed as f64) / (self.metrics.total_time_ms as f64 / 1000.0)
            } else {
                0.0
            },
        }
    }
}

/// 管道状态
#[derive(Debug, Clone)]
pub struct PipelineStatus {
    pub is_running: bool,
    pub items_processed: u64,
    pub items_failed: u64,
    pub total_time_ms: u64,
    pub throughput_per_second: f64,
}

/// 管道错误
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("源数据源错误: {0}")]
    SourceError(#[source] Box<dyn std::error::Error>),
    
    #[error("数据处理器错误: {0}")]
    ProcessorError(#[source] Box<dyn std::error::Error>),
    
    #[error("数据接收器错误: {0}")]
    SinkError(#[source] Box<dyn std::error::Error>),
    
    #[error("任务Join错误")]
    TaskJoinError,
    
    #[error("源数据源无效")]
    SourceInvalid,
    
    #[error("处理器信息无效")]
    InvalidProcessorInfo,
    
    #[error("配置错误: {0}")]
    ConfigError(String),
}
```

## 5.6 具体实现：CSV数据源

现在让我们实现一个具体的CSV数据源来演示如何使用这些特征。

```rust
// CSV数据源实现
use csv::Reader;
use serde_json::{Value, Map, Number};
use std::fs::File;
use std::path::Path;
use std::io::BufReader;
use std::io::Read;

/// CSV数据源
pub struct CsvDataSource {
    path: PathBuf,
    delimiter: char,
    has_header: bool,
    encoding: String,
    buffer_size: usize,
}

impl CsvDataSource {
    /// 创建新的CSV数据源
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            delimiter: ',',
            has_header: true,
            encoding: "UTF-8".to_string(),
            buffer_size: 8192,
        }
    }
    
    /// 设置分隔符
    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }
    
    /// 设置是否包含标题行
    pub fn has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }
    
    /// 设置编码
    pub fn encoding(mut self, encoding: &str) -> Self {
        self.encoding = encoding.to_string();
        self
    }
    
    /// 设置缓冲区大小
    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }
}

impl DataSource<Value> for CsvDataSource {
    type Error = CsvError;
    
    fn read(&self) -> Result<Vec<Value>, Self::Error> {
        // 打开文件
        let file = File::open(&self.path)
            .map_err(|e| CsvError::FileOpenError(e))?;
        
        // 创建CSV读取器
        let mut reader = Reader::new(BufReader::new(file))
            .delimiter(self.delimiter as u8);
        
        let mut records = Vec::new();
        
        if self.has_header {
            self.read_with_header(&mut reader, &mut records)?;
        } else {
            self.read_without_header(&mut reader, &mut records)?;
        }
        
        Ok(records)
    }
    
    fn read_stream(&self) -> Result<Box<dyn Iterator<Item = Result<Value, Self::Error>>>, Self::Error> {
        // 创建流式读取器
        let file = File::open(&self.path)
            .map_err(|e| CsvError::FileOpenError(e))?;
        
        let mut reader = Reader::new(BufReader::new(file))
            .delimiter(self.delimiter as u8);
        
        if self.has_header {
            let headers = reader.headers()
                .map_err(|e| CsvError::ReadError(e))?
                .iter()
                .map(|h| h.to_string())
                .collect::<Vec<_>>();
            
            Ok(Box::new(CsvRecordIterator {
                reader: Some(reader),
                headers: Some(headers),
                has_header: true,
                finished: false,
            }))
        } else {
            Ok(Box::new(CsvRecordIterator {
                reader: Some(reader),
                headers: None,
                has_header: false,
                finished: false,
            }))
        }
    }
    
    fn count(&self) -> Result<u64, Self::Error> {
        let mut count = 0u64;
        let file = File::open(&self.path)
            .map_err(|e| CsvError::FileOpenError(e))?;
        
        let mut reader = Reader::new(BufReader::new(file))
            .delimiter(self.delimiter as u8);
        
        if self.has_header {
            // 跳过标题行
            for _ in reader.records() {
                count += 1;
            }
        } else {
            for _ in reader.records() {
                count += 1;
            }
        }
        
        Ok(count)
    }
    
    fn is_valid(&self) -> bool {
        self.path.exists() && 
        self.path.is_file() && 
        self.path.extension()
            .map(|ext| ext == "csv" || ext == "tsv")
            .unwrap_or(false)
    }
}

impl CsvDataSource {
    /// 读取带标题的CSV
    fn read_with_header(
        &self, 
        reader: &mut Reader<BufReader<File>>, 
        records: &mut Vec<Value>
    ) -> Result<(), CsvError> {
        // 读取标题行
        let headers = reader.headers()
            .map_err(|e| CsvError::ReadError(e))?
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>();
        
        // 读取数据记录
        for result in reader.records() {
            let record = result.map_err(|e| CsvError::ReadError(e))?;
            
            // 将记录转换为JSON对象
            let mut obj = Map::new();
            for (i, field) in record.iter().enumerate() {
                if i < headers.len() {
                    // 尝试解析为数字或布尔值
                    let value = if field == "true" {
                        Value::Bool(true)
                    } else if field == "false" {
                        Value::Bool(false)
                    } else if let Ok(num) = field.parse::<i64>() {
                        Value::Number(Number::from(num))
                    } else if let Ok(num) = field.parse::<f64>() {
                        Value::Number(Number::from_f64(num).unwrap())
                    } else {
                        Value::String(field.to_string())
                    };
                    obj.insert(headers[i].clone(), value);
                }
            }
            
            records.push(Value::Object(obj));
        }
        
        Ok(())
    }
    
    /// 读取无标题的CSV
    fn read_without_header(
        &self, 
        reader: &mut Reader<BufReader<File>>, 
        records: &mut Vec<Value>
    ) -> Result<(), CsvError> {
        for result in reader.records() {
            let record = result.map_err(|e| CsvError::ReadError(e))?;
            
            // 将记录转换为JSON数组
            let mut array = Vec::new();
            for field in record.iter() {
                // 尝试解析为数字或布尔值
                let value = if field == "true" {
                    Value::Bool(true)
                } else if field == "false" {
                    Value::Bool(false)
                } else if let Ok(num) = field.parse::<i64>() {
                    Value::Number(Number::from(num))
                } else if let Ok(num) = field.parse::<f64>() {
                    Value::Number(Number::from_f64(num).unwrap())
                } else {
                    Value::String(field.to_string())
                };
                array.push(value);
            }
            
            records.push(Value::Array(array));
        }
        
        Ok(())
    }
}

/// CSV记录迭代器（流式读取）
struct CsvRecordIterator<R: Read> {
    reader: Option<Reader<BufReader<R>>>,
    headers: Option<Vec<String>>,
    has_header: bool,
    finished: bool,
}

impl<R: Read> Iterator for CsvRecordIterator<R> {
    type Item = Result<Value, CsvError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.reader.is_none() {
            return None;
        }
        
        let reader = self.reader.as_mut()?;
        let headers = self.headers.as_ref();
        
        match reader.records().next() {
            Some(Ok(record)) => {
                // 将记录转换为Value
                if self.has_header {
                    if let Some(headers) = headers {
                        let mut obj = Map::new();
                        for (i, field) in record.iter().enumerate() {
                            if i < headers.len() {
                                let value = if field == "true" {
                                    Value::Bool(true)
                                } else if field == "false" {
                                    Value::Bool(false)
                                } else if let Ok(num) = field.parse::<i64>() {
                                    Value::Number(Number::from(num))
                                } else if let Ok(num) = field.parse::<f64>() {
                                    Value::Number(Number::from_f64(num).unwrap())
                                } else {
                                    Value::String(field.to_string())
                                };
                                obj.insert(headers[i].clone(), value);
                            }
                        }
                        Some(Ok(Value::Object(obj)))
                    } else {
                        Some(Ok(Value::Array(Vec::new())))
                    }
                } else {
                    let mut array = Vec::new();
                    for field in record.iter() {
                        let value = if field == "true" {
                            Value::Bool(true)
                        } else if field == "false" {
                            Value::Bool(false)
                        } else if let Ok(num) = field.parse::<i64>() {
                            Value::Number(Number::from(num))
                        } else if let Ok(num) = field.parse::<f64>() {
                            Value::Number(Number::from_f64(num).unwrap())
                        } else {
                            Value::String(field.to_string())
                        };
                        array.push(value);
                    }
                    Some(Ok(Value::Array(array)))
                }
            }
            Some(Err(e)) => Some(Err(CsvError::ReadError(e))),
            None => {
                self.finished = true;
                self.reader = None;
                None
            }
        }
    }
}

/// CSV错误类型
#[derive(Debug, thiserror::Error)]
pub enum CsvError {
    #[error("文件打开错误: {0}")]
    FileOpenError(std::io::Error),
    
    #[error("读取错误: {0}")]
    ReadError(csv::Error),
    
    #[error("编码错误: {0}")]
    EncodingError(String),
    
    #[error("格式错误: {0}")]
    FormatError(String),
    
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
}
```

## 5.7 数据处理器实现

接下来实现一个数据处理器，用于转换和验证数据。

```rust
// 数据处理器实现
use std::collections::HashMap;

/// 数据转换处理器
pub struct DataTransformProcessor {
    transformations: Vec<DataTransform>,
    validations: Vec<DataValidation>,
    filters: Vec<DataFilter>,
    config: TransformConfig,
}

#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub fail_on_error: bool,
    pub continue_on_warning: bool,
    pub max_errors: usize,
    pub enable_logging: bool,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            fail_on_error: true,
            continue_on_warning: true,
            max_errors: 100,
            enable_logging: true,
        }
    }
}

/// 数据转换操作
#[derive(Debug, Clone)]
pub enum DataTransform {
    /// 字段重命名
    RenameField { from: String, to: String },
    /// 字段类型转换
    ConvertType { field: String, to_type: FieldType },
    /// 字段计算
    ComputeField { 
        target: String, 
        operation: ComputeOperation 
    },
    /// 字段映射
    MapField { 
        field: String, 
        mapping: HashMap<String, String> 
    },
    /// 添加常量
    AddConstant { field: String, value: Value },
    /// 删除字段
    RemoveField(String),
    /// JSON路径操作
    JsonPath { path: String, operation: JsonPathOperation },
}

/// 字段类型
#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Email,
    Url,
}

/// 计算操作
#[derive(Debug, Clone)]
pub enum ComputeOperation {
    /// 数值运算
    Math { operation: MathOperation, operands: Vec<String> },
    /// 字符串操作
    String { operation: StringOperation, source_field: String },
    /// 条件计算
    Conditional { condition: Condition, then_value: Value, else_value: Option<Value> },
    /// 聚合操作
    Aggregate { operation: AggregateOperation, group_by: Vec<String> },
}

/// 数学运算
#[derive(Debug, Clone)]
pub enum MathOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
}

/// 字符串操作
#[derive(Debug, Clone)]
pub enum StringOperation {
    Uppercase,
    Lowercase,
    Trim,
    Replace { from: String, to: String },
    Substring { start: usize, length: Option<usize> },
    Length,
    Contains(String),
    StartsWith(String),
    EndsWith(String),
}

/// 条件
#[derive(Debug, Clone)]
pub struct Condition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Contains,
    In(Vec<Value>),
    NotIn(Vec<Value>),
    IsNull,
    IsNotNull,
}

/// 聚合操作
#[derive(Debug, Clone)]
pub enum AggregateOperation {
    Count,
    Sum,
    Average,
    Min,
    Max,
}

/// JSON路径操作
#[derive(Debug, Clone)]
pub enum JsonPathOperation {
    Get(String),
    Set(String, Value),
    Delete(String),
    Exists(String),
}

/// 数据验证规则
#[derive(Debug, Clone)]
pub enum DataValidation {
    Required { fields: Vec<String> },
    TypeCheck { field: String, expected_type: FieldType },
    Range { field: String, min: Option<Value>, max: Option<Value> },
    Pattern { field: String, pattern: String },
    Unique { field: String },
    Custom { field: String, rule: String },
}

/// 数据过滤规则
#[derive(Debug, Clone)]
pub enum DataFilter {
    Include { condition: Condition },
    Exclude { condition: Condition },
    FieldPresence { field: String, present: bool },
}

impl DataTransformProcessor {
    /// 创建新的转换处理器
    pub fn new() -> Self {
        Self {
            transformations: Vec::new(),
            validations: Vec::new(),
            filters: Vec::new(),
            config: TransformConfig::default(),
        }
    }
    
    /// 添加转换操作
    pub fn add_transform(mut self, transform: DataTransform) -> Self {
        self.transformations.push(transform);
        self
    }
    
    /// 添加验证规则
    pub fn add_validation(mut self, validation: DataValidation) -> Self {
        self.validations.push(validation);
        self
    }
    
    /// 添加过滤规则
    pub fn add_filter(mut self, filter: DataFilter) -> Self {
        self.filters.push(filter);
        self
    }
    
    /// 设置配置
    pub fn with_config(mut self, config: TransformConfig) -> Self {
        self.config = config;
        self
    }
    
    /// 检查数据是否通过过滤
    fn passes_filters(&self, data: &Value) -> bool {
        for filter in &self.filters {
            if !self.apply_filter(filter, data) {
                return false;
            }
        }
        true
    }
    
    /// 应用单个过滤器
    fn apply_filter(&self, filter: &DataFilter, data: &Value) -> bool {
        match filter {
            DataFilter::Include { condition } => self.evaluate_condition(condition, data),
            DataFilter::Exclude { condition } => !self.evaluate_condition(condition, data),
            DataFilter::FieldPresence { field, present } => {
                let has_field = self.has_field(data, field);
                has_field == *present
            }
        }
    }
    
    /// 评估条件
    fn evaluate_condition(&self, condition: &Condition, data: &Value) -> bool {
        let field_value = self.get_field_value(data, &condition.field);
        
        match condition.operator {
            ConditionOperator::Equals => field_value == Some(condition.value.clone()),
            ConditionOperator::NotEquals => field_value != Some(condition.value.clone()),
            ConditionOperator::IsNull => field_value.is_none(),
            ConditionOperator::IsNotNull => field_value.is_some(),
            _ => {
                // 数值比较和其他操作
                if let (Some(Value::Number(lhs)), Some(Value::Number(rhs))) = (field_value, Some(condition.value.clone())) {
                    match condition.operator {
                        ConditionOperator::GreaterThan => lhs.as_f64() > rhs.as_f64(),
                        ConditionOperator::LessThan => lhs.as_f64() < rhs.as_f64(),
                        ConditionOperator::GreaterEqual => lhs.as_f64() >= rhs.as_f64(),
                        ConditionOperator::LessEqual => lhs.as_f64() <= rhs.as_f64(),
                        _ => false,
                    }
                } else {
                    false
                }
            }
        }
    }
    
    /// 获取字段值
    fn get_field_value(&self, data: &Value, field: &str) -> Option<Value> {
        match data {
            Value::Object(obj) => obj.get(field).cloned(),
            Value::Array(arr) => {
                if let Ok(index) = field.parse::<usize>() {
                    arr.get(index).cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// 检查字段是否存在
    fn has_field(&self, data: &Value, field: &str) -> bool {
        self.get_field_value(data, field).is_some()
    }
    
    /// 应用所有转换
    fn apply_transformations(&self, mut data: Value) -> Result<Value, TransformError> {
        for transform in &self.transformations {
            data = self.apply_transform(transform, data)?;
        }
        Ok(data)
    }
    
    /// 应用单个转换
    fn apply_transform(&self, transform: &DataTransform, data: Value) -> Result<Value, TransformError> {
        match transform {
            DataTransform::RenameField { from, to } => {
                if let Value::Object(ref mut obj) = data {
                    if let Some(value) = obj.remove(from) {
                        obj.insert(to.clone(), value);
                    }
                    Ok(data)
                } else {
                    Err(TransformError::InvalidOperation("Cannot rename field in non-object data".to_string()))
                }
            }
            
            DataTransform::ConvertType { field, to_type } => {
                if let Value::Object(ref mut obj) = data {
                    if let Some(value) = obj.get_mut(field) {
                        *value = self.convert_type(value.clone(), to_type)?;
                    }
                    Ok(data)
                } else {
                    Err(TransformError::InvalidOperation("Cannot convert type in non-object data".to_string()))
                }
            }
            
            DataTransform::AddConstant { field, value } => {
                if let Value::Object(ref mut obj) = data {
                    obj.insert(field.clone(), value.clone());
                    Ok(data)
                } else {
                    Err(TransformError::InvalidOperation("Cannot add constant to non-object data".to_string()))
                }
            }
            
            DataTransform::RemoveField(field_name) => {
                if let Value::Object(ref mut obj) = data {
                    obj.remove(field_name);
                    Ok(data)
                } else {
                    Err(TransformError::InvalidOperation("Cannot remove field from non-object data".to_string()))
                }
            }
            
            _ => {
                // 简化实现，其他转换类型
                Ok(data)
            }
        }
    }
    
    /// 类型转换
    fn convert_type(&self, value: Value, to_type: &FieldType) -> Result<Value, TransformError> {
        match to_type {
            FieldType::String => {
                let string_value = match value {
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::String(s) => s,
                    Value::Array(_) | Value::Object(_) => {
                        return Err(TransformError::TypeConversionError("Cannot convert complex type to string".to_string()))
                    }
                };
                Ok(Value::String(string_value))
            }
            
            FieldType::Integer => {
                match value {
                    Value::String(s) => {
                        if let Ok(num) = s.parse::<i64>() {
                            Ok(Value::Number(serde_json::Number::from(num)))
                        } else {
                            Err(TransformError::TypeConversionError("Cannot convert string to integer".to_string()))
                        }
                    }
                    Value::Number(n) => {
                        if n.is_i64() {
                            Ok(Value::Number(n))
                        } else {
                            Err(TransformError::TypeConversionError("Cannot convert float to integer".to_string()))
                        }
                    }
                    Value::Bool(b) => Ok(Value::Number(serde_json::Number::from(if b { 1 } else { 0 }))),
                    _ => Err(TransformError::TypeConversionError("Invalid type conversion".to_string())),
                }
            }
            
            FieldType::Boolean => {
                match value {
                    Value::String(s) => Ok(Value::Bool(s.parse::<bool>().unwrap_or(false))),
                    Value::Number(n) => Ok(Value::Bool(n.as_i64() != Some(0))),
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    _ => Err(TransformError::TypeConversionError("Invalid type conversion to boolean".to_string())),
                }
            }
            
            _ => Ok(value), // 简化实现
        }
    }
}

impl DataProcessor<Value, Value> for DataTransformProcessor {
    type Error = TransformError;
    
    fn process(&self, data: Vec<Value>) -> Result<Vec<Value>, Self::Error> {
        let mut results = Vec::with_capacity(data.len());
        let mut error_count = 0;
        
        for item in data {
            // 检查是否通过过滤器
            if !self.passes_filters(&item) {
                continue;
            }
            
            // 应用转换
            match self.apply_transformations(item) {
                Ok(transformed) => {
                    results.push(transformed);
                }
                Err(e) => {
                    error_count += 1;
                    
                    if self.config.fail_on_error && error_count > self.config.max_errors {
                        return Err(e);
                    }
                    
                    if self.config.enable_logging {
                        eprintln!("转换错误: {:?}", e);
                    }
                    
                    if self.config.fail_on_error {
                        return Err(e);
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    fn process_item(&self, item: Value) -> Result<Value, Self::Error> {
        if !self.passes_filters(&item) {
            return Err(TransformError::FilteredOut("Item filtered out".to_string()));
        }
        
        self.apply_transformations(item)
    }
    
    fn process_stream(&self, stream: Box<dyn Iterator<Item = Value>>) -> Result<Box<dyn Iterator<Item = Result<Value, Self::Error>>>, Self::Error> {
        let processor = self.clone();
        let config = self.config.clone();
        
        Ok(Box::new(stream.map(move |item| {
            if !processor.passes_filters(&item) {
                return Ok(item); // 保留原始数据或根据需求过滤
            }
            
            match processor.apply_transformations(item) {
                Ok(transformed) => Ok(transformed),
                Err(e) => {
                    if config.fail_on_error {
                        Err(e)
                    } else {
                        Ok(item) // 返回原始数据
                    }
                }
            }
        })))
    }
    
    fn info(&self) -> ProcessorInfo {
        ProcessorInfo {
            name: "DataTransformProcessor".to_string(),
            version: "1.0.0".to_string(),
            description: "数据转换和验证处理器".to_string(),
            input_type: "serde_json::Value".to_string(),
            output_type: "serde_json::Value".to_string(),
            performance_metrics: PerformanceMetrics {
                processing_time_ms: 0,
                throughput_items_per_second: 0.0,
                memory_usage_mb: 0.0,
            },
        }
    }
}

/// 转换错误
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    #[error("类型转换错误: {0}")]
    TypeConversionError(String),
    
    #[error("无效操作: {0}")]
    InvalidOperation(String),
    
    #[error("验证错误: {0}")]
    ValidationError(String),
    
    #[error("字段错误: {0}")]
    FieldError(String),
    
    #[error("被过滤: {0}")]
    FilteredOut(String),
    
    #[error("处理错误: {0}")]
    ProcessingError(String),
}
```

## 5.8 数据输出实现

现在实现一个文件输出处理器：

```rust
// 数据输出实现
use serde_json::{Value, Map, Number};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// JSON文件输出处理器
pub struct JsonFileSink {
    path: PathBuf,
    format: OutputFormat,
    config: OutputConfig,
    stats: SinkStats,
    buffer: Vec<Value>,
    buffer_size: usize,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    /// 标准JSON格式
    Json {
        pretty: bool,
        pretty_indent: usize,
    },
    /// NDJSON (每行一个JSON对象)
    Ndjson,
    /// 压缩JSON
    JsonCompressed {
        compression: CompressionType,
    },
    /// CSV格式
    Csv {
        delimiter: char,
        has_header: bool,
        include_nulls: bool,
    },
}

#[derive(Debug, Clone)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
    Bzip2,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub buffer_size: usize,
    pub auto_flush: bool,
    pub create_dirs: bool,
    pub overwrite_existing: bool,
    pub encoding: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            auto_flush: true,
            create_dirs: true,
            overwrite_existing: false,
            encoding: "UTF-8".to_string(),
        }
    }
}

impl JsonFileSink {
    /// 创建新的JSON文件输出
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            format: OutputFormat::Json { pretty: true, pretty_indent: 2 },
            config: OutputConfig::default(),
            stats: SinkStats {
                total_written: 0,
                write_time_ms: 0,
                last_write: None,
            },
            buffer: Vec::new(),
            buffer_size: 0,
        }
    }
    
    /// 设置输出格式
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }
    
    /// 设置配置
    pub fn with_config(mut self, config: OutputConfig) -> Self {
        self.config = config;
        self
    }
    
    /// 创建目录
    fn create_directories(&self) -> Result<(), SinkError> {
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| SinkError::IoError(e))?;
            }
        }
        Ok(())
    }
    
    /// 打开文件（如果需要的话）
    fn open_file(&self) -> Result<File, SinkError> {
        let file = if self.config.overwrite_existing {
            File::create(&self.path)
        } else {
            File::options()
                .write(true)
                .create_new(true)
                .open(&self.path)
        };
        
        file.map_err(|e| SinkError::FileOpenError(e))
    }
    
    /// 写入单个记录
    fn write_record(&mut self, record: &Value) -> Result<(), SinkError> {
        let start_time = std::time::Instant::now();
        
        // 格式化数据
        let formatted = match &self.format {
            OutputFormat::Json { pretty, indent } => {
                if *pretty {
                    serde_json::to_string_pretty(record)
                } else {
                    serde_json::to_string(record)
                }
                .map_err(|e| SinkError::SerializationError(e))?
            }
            OutputFormat::Ndjson => {
                serde_json::to_string(record)
                    .map_err(|e| SinkError::SerializationError(e))?
            }
            OutputFormat::JsonCompressed { .. } => {
                // 简化实现，实际应该压缩
                serde_json::to_string(record)
                    .map_err(|e| SinkError::SerializationError(e))?
            }
            OutputFormat::Csv { .. } => {
                self.convert_to_csv_line(record)?
            }
        };
        
        // 写入文件（这里简化实现，实际应该保持文件句柄）
        let mut file = self.open_file()?;
        writeln!(file, "{}", formatted)
            .map_err(|e| SinkError::WriteError(e))?;
        
        // 更新统计
        self.stats.total_written += 1;
        self.stats.write_time_ms += start_time.elapsed().as_millis() as u64;
        self.stats.last_write = Some(std::time::SystemTime::now());
        
        Ok(())
    }
    
    /// 转换为CSV行
    fn convert_to_csv_line(&self, record: &Value) -> Result<String, SinkError> {
        match record {
            Value::Object(obj) => {
                // 对象转换为CSV行
                let mut values = Vec::new();
                for (_, value) in obj {
                    let csv_value = match value {
                        Value::Null => "".to_string(),
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Array(_) | Value::Object(_) => {
                            return Err(SinkError::ConversionError("Complex types not supported in CSV".to_string()))
                        }
                    };
                    values.push(csv_value);
                }
                Ok(values.join(","))
            }
            Value::Array(arr) => {
                // 数组直接转换为CSV行
                let mut values = Vec::new();
                for value in arr {
                    let csv_value = match value {
                        Value::Null => "".to_string(),
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Array(_) | Value::Object(_) => {
                            return Err(SinkError::ConversionError("Complex types not supported in CSV".to_string()))
                        }
                    };
                    values.push(csv_value);
                }
                Ok(values.join(","))
            }
            _ => {
                Err(SinkError::ConversionError("Record is not object or array".to_string()))
            }
        }
    }
    
    /// 刷新缓冲区
    fn flush_buffer(&mut self) -> Result<(), SinkError> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        
        // 批量写入
        for record in &self.buffer {
            self.write_record(record)?;
        }
        
        self.buffer.clear();
        self.buffer_size = 0;
        
        Ok(())
    }
}

impl DataSink<Value> for JsonFileSink {
    type Error = SinkError;
    
    fn write(&mut self, data: Vec<Value>) -> Result<(), Self::Error> {
        // 创建目录
        if self.config.create_dirs {
            self.create_directories()?;
        }
        
        for record in data {
            if self.config.buffer_size > 0 {
                // 使用缓冲区
                self.buffer.push(record);
                self.buffer_size += 1;
                
                if self.buffer_size >= self.config.buffer_size || self.config.auto_flush {
                    self.flush_buffer()?;
                }
            } else {
                // 直接写入
                self.write_record(&record)?;
            }
        }
        
        Ok(())
    }
    
    fn write_stream(&mut self, stream: Box<dyn Iterator<Item = Value>>) -> Result<(), Self::Error> {
        // 创建目录
        if self.config.create_dirs {
            self.create_directories()?;
        }
        
        for record in stream {
            if self.config.buffer_size > 0 {
                self.buffer.push(record);
                self.buffer_size += 1;
                
                if self.buffer_size >= self.config.buffer_size || self.config.auto_flush {
                    self.flush_buffer()?;
                }
            } else {
                self.write_record(&record)?;
            }
        }
        
        Ok(())
    }
    
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush_buffer()?;
        
        // 这里可以刷新底层的文件句柄
        // 简化实现中我们已经在每次写入时刷新了
        Ok(())
    }
    
    fn stats(&self) -> SinkStats {
        self.stats.clone()
    }
}

/// 接收器错误
#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    #[error("文件打开错误: {0}")]
    FileOpenError(std::io::Error),
    
    #[error("写入错误: {0}")]
    WriteError(std::io::Error),
    
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO错误: {0}")]
    IoError(std::io::Error),
    
    #[error("转换错误: {0}")]
    ConversionError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
}
```

## 5.9 完整的示例程序

现在让我们创建一个完整的示例程序来展示整个数据流框架的使用：

```rust
// 主程序示例
use dataflow_framework::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 数据流框架示例 ===\n");
    
    // 1. 创建数据源（CSV文件）
    println!("1. 创建CSV数据源");
    let source = CsvDataSource::new("data/sample.csv")
        .has_header(true)
        .delimiter(',');
    
    // 2. 创建数据处理器
    println!("2. 创建数据转换处理器");
    let mut transforms = Vec::new();
    
    // 添加字段重命名
    transforms.push(DataTransform::RenameField { 
        from: "name".to_string(), 
        to: "full_name".to_string() 
    });
    
    // 添加类型转换
    transforms.push(DataTransform::ConvertType { 
        field: "age".to_string(), 
        to_type: FieldType::Integer 
    });
    
    // 添加常量字段
    transforms.push(DataTransform::AddConstant { 
        field: "source".to_string(), 
        Value::String("csv_import".to_string()) 
    });
    
    // 添加验证
    let mut validations = Vec::new();
    validations.push(DataValidation::Required { 
        fields: vec!["name".to_string(), "age".to_string()] 
    });
    
    // 添加过滤
    let mut filters = Vec::new();
    filters.push(DataFilter::Include { 
        condition: Condition {
            field: "age".to_string(),
            operator: ConditionOperator::GreaterEqual,
            value: Value::Number(Number::from(18)),
        }
    });
    
    let processor = DataTransformProcessor::new()
        .add_transforms(transforms)
        .add_validations(validations)
        .add_filters(filters);
    
    // 3. 创建数据接收器
    println!("3. 创建JSON文件输出");
    let output_format = OutputFormat::Json { 
        pretty: true, 
        pretty_indent: 2 
    };
    
    let output_config = OutputConfig {
        buffer_size: 100,
        auto_flush: true,
        create_dirs: true,
        overwrite_existing: true,
        encoding: "UTF-8".to_string(),
    };
    
    let sink = JsonFileSink::new("output/processed_data.json")
        .format(output_format)
        .with_config(output_config);
    
    // 4. 创建数据管道
    println!("4. 创建数据处理管道");
    let pipeline_config = PipelineConfig {
        batch_size: 50,
        parallel_processing: false,  // 示例中关闭并行处理
        max_concurrency: 4,
        enable_cache: true,
        cache_ttl_seconds: 3600,
        retry_attempts: 3,
        timeout_seconds: 300,
    };
    
    let mut pipeline = DataPipeline::with_config(source, processor, sink, pipeline_config);
    
    // 5. 运行管道
    println!("5. 开始处理数据...\n");
    let start_time = std::time::Instant::now();
    
    let metrics = pipeline.run().await?;
    
    let total_time = start_time.elapsed();
    
    // 6. 显示结果
    println!("\n=== 处理完成 ===");
    println!("总处理时间: {:?}", total_time);
    println!("处理的数据项数: {}", metrics.items_processed);
    println!("失败的数据项数: {}", metrics.items_failed);
    println!("处理吞吐量: {:.2} 项/秒", 
             metrics.items_processed as f64 / total_time.as_secs_f64());
    
    if metrics.items_failed > 0 {
        println!("警告: 有 {} 项数据处理失败", metrics.items_failed);
    }
    
    // 7. 获取管道状态
    let status = pipeline.get_status();
    println!("\n=== 管道状态 ===");
    println!("是否运行中: {}", status.is_running);
    println!("吞吐量: {:.2} 项/秒", status.throughput_per_second);
    
    Ok(())
}

// 为DataTransformProcessor添加方便的方法
trait DataTransformProcessorBuilder {
    fn add_transforms(self, transforms: Vec<DataTransform>) -> Self;
    fn add_validations(self, validations: Vec<DataValidation>) -> Self;
    fn add_filters(self, filters: Vec<DataFilter>) -> Self;
}

impl DataTransformProcessorBuilder for DataTransformProcessor {
    fn add_transforms(mut self, transforms: Vec<DataTransform>) -> Self {
        for transform in transforms {
            self = self.add_transform(transform);
        }
        self
    }
    
    fn add_validations(mut self, validations: Vec<DataValidation>) -> Self {
        for validation in validations {
            self = self.add_validation(validation);
        }
        self
    }
    
    fn add_filters(mut self, filters: Vec<DataFilter>) -> Self {
        for filter in filters {
            self = self.add_filter(filter);
        }
        self
    }
}
```

## 5.10 测试代码

让我们为框架创建全面的测试：

```rust
// 测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use serde_json::{json, Value};

    #[test]
    fn test_csv_data_source() {
        // 创建临时CSV文件
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,25,New York").unwrap();
        writeln!(temp_file, "Bob,30,Los Angeles").unwrap();
        temp_file.flush().unwrap();
        
        // 测试CSV数据源
        let source = CsvDataSource::new(temp_file.path())
            .has_header(true);
        
        let data = source.read().unwrap();
        
        assert_eq!(data.len(), 2);
        assert_eq!(data[0]["name"], "Alice");
        assert_eq!(data[0]["age"], 25);
        assert_eq!(data[0]["city"], "New York");
    }
    
    #[test]
    fn test_data_transform_processor() {
        let processor = DataTransformProcessor::new()
            .add_transform(DataTransform::RenameField {
                from: "name".to_string(),
                to: "full_name".to_string(),
            })
            .add_transform(DataTransform::AddConstant {
                field: "source".to_string(),
                Value::String("test".to_string()),
            });
        
        let input_data = vec![
            json!({
                "name": "Alice",
                "age": 25
            }),
            json!({
                "name": "Bob",
                "age": 30
            }),
        ];
        
        let result = processor.process(input_data).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["full_name"], "Alice");
        assert_eq!(result[0]["source"], "test");
        assert_eq!(result[1]["full_name"], "Bob");
        assert_eq!(result[1]["source"], "test");
    }
    
    #[test]
    fn test_data_filters() {
        let processor = DataTransformProcessor::new()
            .add_filter(DataFilter::Include {
                condition: Condition {
                    field: "age".to_string(),
                    operator: ConditionOperator::GreaterEqual,
                    value: json!(25),
                }
            });
        
        let input_data = vec![
            json!({"name": "Alice", "age": 25}),
            json!({"name": "Bob", "age": 20}),
            json!({"name": "Carol", "age": 30}),
        ];
        
        let result = processor.process(input_data).unwrap();
        
        // 应该过滤掉Bob（age < 25）
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["name"], "Alice");
        assert_eq!(result[1]["name"], "Carol");
    }
    
    #[test]
    fn test_type_conversion() {
        let processor = DataTransformProcessor::new()
            .add_transform(DataTransform::ConvertType {
                field: "age".to_string(),
                to_type: FieldType::Integer,
            });
        
        let input_data = vec![json!({"age": "25"})];
        
        let result = processor.process(input_data).unwrap();
        
        assert_eq!(result[0]["age"], 25);
    }
    
    #[test]
    fn test_json_file_sink() {
        // 创建临时文件
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        drop(temp_file); // 关闭文件句柄
        
        let mut sink = JsonFileSink::new(path.clone())
            .format(OutputFormat::Json { pretty: true, pretty_indent: 2 });
        
        let data = vec![
            json!({"name": "Alice", "age": 25}),
            json!({"name": "Bob", "age": 30}),
        ];
        
        sink.write(data).unwrap();
        sink.flush().unwrap();
        
        // 验证输出文件
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("Bob"));
        assert!(content.contains("25"));
    }
    
    #[test]
    fn test_data_pipeline() {
        // 简化测试：使用内存数据源
        struct MemoryDataSource {
            data: Vec<Value>,
        }
        
        impl MemoryDataSource {
            fn new(data: Vec<Value>) -> Self {
                Self { data }
            }
        }
        
        impl DataSource<Value> for MemoryDataSource {
            type Error = Box<dyn std::error::Error>;
            
            fn read(&self) -> Result<Vec<Value>, Self::Error> {
                Ok(self.data.clone())
            }
            
            fn read_stream(&self) -> Result<Box<dyn Iterator<Item = Result<Value, Self::Error>>>, Self::Error> {
                let data = self.data.clone();
                Ok(Box::new(data.into_iter().map(Ok)))
            }
            
            fn count(&self) -> Result<u64, Self::Error> {
                Ok(self.data.len() as u64)
            }
            
            fn is_valid(&self) -> bool {
                !self.data.is_empty()
            }
        }
        
        let source = MemoryDataSource::new(vec![
            json!({"name": "Alice", "age": 25}),
            json!({"name": "Bob", "age": 30}),
        ]);
        
        let processor = DataTransformProcessor::new()
            .add_transform(DataTransform::AddConstant {
                field: "processed".to_string(),
                Value::Bool(true),
            });
        
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        drop(temp_file);
        
        let sink = JsonFileSink::new(path);
        
        let pipeline = DataPipeline::new(source, processor, sink);
        let status = pipeline.get_status();
        
        assert_eq!(status.items_processed, 0); // 管道还没运行
    }
    
    #[test]
    fn test_error_handling() {
        let processor = DataTransformProcessor::new()
            .with_config(TransformConfig {
                fail_on_error: true,
                continue_on_warning: true,
                max_errors: 1,
                enable_logging: false,
            });
        
        // 无效的转换（尝试重命名字段但数据不是对象）
        let input_data = vec![json!(42)]; // 数字不是对象
        
        let result = processor.process(input_data);
        
        // 应该返回错误
        assert!(result.is_err());
    }
}
```

## 5.11 性能优化技巧

在企业级应用中，性能是关键考虑因素。以下是一些优化数据流框架性能的方法：

### 5.11.1 内存管理优化

```rust
// 内存优化的数据处理
pub struct StreamingDataProcessor<T> {
    buffer_size: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> StreamingDataProcessor<T> {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer_size,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// 流式处理大量数据
    pub async fn process_stream<'a, S, P>(
        &'a self,
        source: S,
        processor: P,
    ) -> Result<StreamingStats, Box<dyn std::error::Error + Send + Sync>>
    where
        S: futures::stream::Stream<Item = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
        P: Fn(&[T]) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync,
        T: Send + Sync + 'a,
    {
        let mut buffer = Vec::with_capacity(self.buffer_size);
        let mut output = Vec::new();
        let mut stats = StreamingStats::default();
        
        // 使用异步流处理
        let mut stream = source.fuse();
        
        while let Some(item_result) = stream.next().await {
            let item = item_result?;
            
            buffer.push(item);
            stats.input_count += 1;
            
            // 当缓冲区满时处理
            if buffer.len() >= self.buffer_size {
                let processed_batch = processor(&buffer)?;
                output.extend(processed_batch);
                stats.output_count += processed_batch.len() as u64;
                buffer.clear();
                
                // 强制释放内存
                if buffer.capacity() > self.buffer_size * 2 {
                    buffer.shrink_to_fit();
                }
            }
        }
        
        // 处理剩余数据
        if !buffer.is_empty() {
            let processed_batch = processor(&buffer)?;
            output.extend(processed_batch);
            stats.output_count += processed_batch.len() as u64;
        }
        
        Ok(stats)
    }
}

#[derive(Debug, Default)]
pub struct StreamingStats {
    pub input_count: u64,
    pub output_count: u64,
    pub processing_time_ms: u64,
    pub memory_peak_mb: f64,
}
```

### 5.11.2 并发优化

```rust
// 并发数据处理
use rayon::prelude::*;

pub struct ParallelDataProcessor {
    chunk_size: usize,
    worker_threads: usize,
}

impl ParallelDataProcessor {
    pub fn new(chunk_size: usize, worker_threads: usize) -> Self {
        rayon::ThreadPoolBuilder::new()
            .num_threads(worker_threads)
            .build_global()
            .ok();
            
        Self { chunk_size, worker_threads }
    }
    
    /// 并行处理数据
    pub fn process_parallel<T, P, R>(
        &self,
        data: &[T],
        processor: P,
    ) -> Result<Vec<R>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: Send + Sync,
        R: Send + Sync,
        P: Fn(&[T]) -> Result<Vec<R>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + Clone,
    {
        // 将数据分块
        let chunks: Vec<_> = data.chunks(self.chunk_size).collect();
        
        // 并行处理每个块
        let results: Vec<_> = chunks
            .par_iter()
            .map(|chunk| {
                let processed = processor(chunk)?;
                Ok(processed)
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        // 合并结果
        let mut output = Vec::new();
        for result_chunk in results {
            output.extend(result_chunk);
        }
        
        Ok(output)
    }
}
```

## 5.12 总结

在本章中，我们深入学习了Rust的泛型和特征，并构建了一个完整