# 第三章：所有权与借用

## 学习目标

通过本章学习，您将掌握：
- Rust所有权系统的核心概念
- 借用检查器的工作原理
- 生命周期如何保证内存安全
- 如何安全地处理文件和内存管理
- 实战项目：构建一个内存安全的文件处理工具

## 3.1 引言：Rust的内存安全革命

在现代编程中，内存安全是一个至关重要的课题。传统的系统编程语言如C和C++虽然提供了强大的性能，但开发者需要手动管理内存，这往往导致：

- **内存泄漏**：忘记释放已分配的内存
- **双重释放**：释放已经被释放的内存
- **悬空指针**：访问已释放的内存
- **缓冲区溢出**：写入超出分配范围的内存

Rust通过其独特的所有权系统，在编译时就确保了内存安全，无需垃圾回收器。这是Rust能够在性能上匹敌C++，同时提供内存安全保证的核心机制。

## 3.2 所有权基础

### 3.2.1 什么是所有权？

所有权是Rust的核心概念，它定义了程序如何管理内存。每当创建变量时，该变量"拥有"其值。当值的所有者超出作用域时，Rust自动清理该值。

**基本规则：**
1. Rust中的每个值都有一个所有者
2. 同时只能有一个所有者
3. 当所有者超出作用域时，值将被自动丢弃

让我们通过一个简单的示例来理解：

```rust
fn main() {
    let s1 = String::from("Hello");
    let s2 = s1; // s1的所有权转移给s2
    
    // 编译错误！s1已不再拥有这个值
    // println!("{}, world!", s1); 
    println!("{}, world!", s2); // 正常，可以访问s2
}
```

在这个例子中：
- `s1`创建了一个String类型的值，并拥有它
- `let s2 = s1;`将所有权从s1转移到s2
- s1在转移后不再拥有值，无法使用

### 3.2.2 转移所有权（Move）

Rust中的移动（Move）是所有权的转移。当我们将一个值赋给另一个变量时，原始变量会失效：

```rust
fn main() {
    let v1 = vec![1, 2, 3, 4];
    let v2 = v1; // v1的所有权转移给v2
    
    // 编译错误！v1已失效
    // println!("v1: {:?}", v1);
    
    println!("v2: {:?}", v2); // 正常
}
```

对于基本类型（实现了Copy trait的类型），赋值时会复制值而不是移动：

```rust
fn main() {
    let x = 42;
    let y = x; // 复制值，x仍然有效
    
    println!("x: {}", x); // 正常，x仍然有效
    println!("y: {}", y); // 正常
}
```

### 3.2.3 函数中的所有权

当我们将值传递给函数时，所有权也会转移：

```rust
fn main() {
    let s = String::from("hello");
    takes_ownership(s); // s的所有权转移给函数
    
    // 编译错误！s不再拥有值
    // println!("{}", s);
    
    let x = 5;
    makes_copy(x); // x被复制，x仍然有效
    println!("x: {}", x); // 正常
}

fn takes_ownership(some_string: String) {
    println!("{}", some_string);
} // some_string超出作用域，值被丢弃

fn makes_copy(some_integer: i32) {
    println!("{}", some_integer);
} // some_integer超出作用域，但基本类型没有影响
```

## 3.3 引用与借用

### 3.3.1 什么是借用？

借用是创建引用的过程，允许我们使用值而不取得所有权。这解决了在函数中使用值但不转移所有权的问题。

```rust
fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1); // 借用s1
    
    println!("The length of '{}' is {}.", s1, len); // s1仍然可用
}

fn calculate_length(s: &String) -> usize { // s是String的引用
    s.len()
} // s超出作用域，但不会丢弃引用指向的值，因为s没有所有权
```

### 3.3.2 可变引用

如果我们需要修改引用的值，可以使用可变引用：

```rust
fn main() {
    let mut s = String::from("hello");
    change(&mut s); // 传递可变引用
    
    println!("{}", s); // 输出 "hello, world"
}

fn change(some_string: &mut String) {
    some_string.push_str(", world");
}
```

**可变引用的重要限制：**
- 同一时间，对同一个值只能有一个可变引用
- 不能同时拥有可变引用和不可变引用

```rust
fn main() {
    let mut s = String::from("hello");
    
    let r1 = &s; // 不可变引用
    let r2 = &s; // 另一个不可变引用
    // let r3 = &mut s; // 编译错误！不能同时拥有可变和不可变引用
    
    println!("{} and {}", r1, r2);
    // r1和r2在作用域结束前都可以使用
    
    println!("{}", r1);
    // 重新使用可变引用
    let r3 = &mut s; // 现在可以创建可变引用了
    println!("{}", r3);
}
```

### 3.3.3 借用检查器

Rust的借用检查器在编译时验证引用的有效性，确保：

1. **悬空引用**：不允许存在悬空引用
2. **引用作用域**：引用不能比其引用的值存在更久

```rust
fn main() {
    let r;
    {
        let x = 5;
        r = &x; // 编译错误！x的作用域比r短
    }
    println!("{}", r);
}
```

在这个例子中，x的作用域在}处结束，但r在println!中仍在使用，这会导致悬空引用，Rust会拒绝编译。

## 3.4 生命周期

### 3.4.1 什么是生命周期？

生命周期是引用保持有效的作用域。Rust需要确保引用的有效性，这就是为什么借用检查器需要跟踪生命周期。

大多数情况下，Rust可以自动推断生命周期：

```rust
fn main() {
    let string1 = String::from("abcd");
    let string2 = "xyz";
    
    let result = longest(string1.as_str(), string2);
    println!("The longest string is {}", result);
}

fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

这个函数可以工作，因为Rust可以推断返回值的生命周期。

### 3.4.2 生命周期注解

当函数返回引用时，我们需要显式注解生命周期：

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

这里`'a`表示返回的引用生命周期至少与两个参数中较短的那个一样长。

### 3.4.3 生命周期在struct中的应用

当结构体包含引用时，需要显式注解生命周期：

```rust
struct ImportantExcerpt<'a> {
    part: &'a str,
}

impl<'a> ImportantExcerpt<'a> {
    fn level(&self) -> i32 {
        3
    }
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().expect("Could not find a '.'");
    
    let i = ImportantExcerpt {
        part: first_sentence,
    };
    
    println!("Excerpt: {}", i.part);
}
```

## 3.5 智能指针

### 3.5.1 Box<T>

`Box<T>`允许在堆上分配值，当box超出作用域时被自动清理：

```rust
fn main() {
    let b = Box::new(5);
    println!("b = {}", b);
} // b被自动清理
```

### 3.5.2 Rc<T>

`Rc<T>`（Reference Counting）允许多个所有者：

```rust
use std::rc::Rc;

fn main() {
    let s = Rc::new(String::from("hello"));
    let s1 = Rc::clone(&s);
    let s2 = Rc::clone(&s);
    
    println!("s: {}, ref count: {}", s, Rc::strong_count(&s));
    println!("s1: {}, ref count: {}", s1, Rc::strong_count(&s));
    println!("s2: {}, ref count: {}", s2, Rc::strong_count(&s));
} // s1和s2被清理后，s也被清理
```

### 3.5.3 Arc<T>

`Arc<T>`（Atomic Reference Counting）是线程安全版本的Rc：

```rust
use std::sync::Arc;

fn main() {
    let s = Arc::new(String::from("hello"));
    let s1 = Arc::clone(&s);
    let s2 = Arc::clone(&s);
    
    println!("Reference count: {}", Arc::strong_count(&s));
    println!("s1: {}", s1);
    println!("s2: {}", s2);
}
```

## 3.6 实战项目：内存安全的文件处理工具

现在我们来实现一个完整的文件处理工具，演示所有权、借用和生命周期的实际应用。

### 3.6.1 项目设计

**项目名称**：`rust-file-processor`

**核心功能**：
1. 安全文件读取（避免内存泄漏）
2. 流式大文件处理
3. 批量文件重命名
4. 文件完整性验证
5. 并发文件处理

### 3.6.2 项目结构

```
rust-file-processor/
├── src/
│   ├── main.rs
│   ├── processors/
│   │   ├── mod.rs
│   │   ├── csv.rs
│   │   ├── json.rs
│   │   ├── text.rs
│   │   └── image.rs
│   ├── utilities/
│   │   ├── mod.rs
│   │   ├── file_ops.rs
│   │   ├── encoding.rs
│   │   └── validation.rs
│   ├── concurrent/
│   │   ├── mod.rs
│   │   ├── worker.rs
│   │   └── pool.rs
│   └── config/
│       ├── mod.rs
│       └── settings.rs
├── tests/
├── examples/
└── fixtures/
    ├── sample.csv
    ├── sample.json
    └── large_file.txt
```

### 3.6.3 核心实现

#### 3.6.3.1 安全的文件读取器

**src/utilities/file_ops.rs**

```rust
use std::fs::File;
use std::io::{BufReader, Read, Lines, BufRead};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::sync::Arc;
use rayon::prelude::*;

#[derive(Debug)]
pub struct FileReader {
    path: Arc<PathBuf>,
    buffer_size: usize,
    encoding: Encoding,
}

#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    UTF8,
    GBK,
    ASCII,
}

impl FileReader {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: Arc::new(path.into()),
            buffer_size: 8192,
            encoding: Encoding::UTF8,
        }
    }
    
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
    
    pub fn with_encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }
    
    /// 使用借用避免所有权转移
    pub fn process_lines<F, T>(&self, processor: F) -> Result<T, Box<dyn Error>>
    where
        F: Fn(&str) -> Result<T, Box<dyn Error>> + Send + Sync,
        T: Send,
    {
        let file = File::open(&*self.path)?;
        let reader = BufReader::new(file);
        
        // 使用流式处理，避免加载整个文件到内存
        let lines = reader.lines().filter_map(|line| match line {
            Ok(line) => Some(line),
            Err(e) => {
                eprintln!("Warning: Skipping invalid line: {}", e);
                None
            }
        });
        
        // 并发处理行，只借用引用
        let results: Vec<Result<T, Box<dyn Error>>> = lines
            .par_iter()
            .map(|line| processor(line))
            .collect();
        
        // 收集结果，如果任一处理失败则返回错误
        let mut processed_results = Vec::new();
        for result in results {
            match result {
                Ok(result) => processed_results.push(result),
                Err(e) => return Err(e),
            }
        }
        
        Ok(self.combine_results(processed_results))
    }
    
    /// 批量处理文件
    pub fn process_batch<P, F, T>(&self, files: &[P], processor: F) -> Result<Vec<T>, Box<dyn Error>>
    where
        P: AsRef<Path> + Send + Sync,
        F: Fn(&Path) -> Result<T, Box<dyn Error>> + Send + Sync,
        T: Send,
    {
        files.par_iter().map(|path| {
            processor(path.as_ref())
        }).collect()
    }
    
    /// 流式处理大文件
    pub fn stream_process<P, F, T>(&self, output: &P, processor: F) -> Result<T, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: Fn(&str) -> Result<String, Box<dyn Error>>,
    {
        let input_file = File::open(&*self.path)?;
        let output_file = File::create(output.as_ref())?;
        
        let mut reader = BufReader::new(input_file);
        let mut writer = BufWriter::new(output_file);
        
        let mut buffer = String::new();
        let mut results = Vec::new();
        
        while reader.read_line(&mut buffer)? > 0 {
            let processed_line = processor(&buffer)?;
            writeln!(writer, "{}", processed_line)?;
            results.push(processed_line);
            buffer.clear();
        }
        
        Ok(self.combine_results(results))
    }
    
    /// 文件完整性验证
    pub fn verify_integrity(&self) -> Result<bool, Box<dyn Error>> {
        let metadata = self.path.metadata()?;
        let file_size = metadata.len();
        
        // 简单的完整性检查：验证文件可以完整读取
        let file = File::open(&*self.path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        
        reader.read_to_end(&mut buffer)?;
        
        Ok(buffer.len() == file_size as usize)
    }
    
    /// 重命名文件
    pub fn rename<P: AsRef<Path>>(&self, new_path: P) -> Result<(), Box<dyn Error>> {
        std::fs::rename(&*self.path, new_path.as_ref())?;
        Ok(())
    }
    
    /// 获取文件元数据
    pub fn metadata(&self) -> Result<std::fs::Metadata, Box<dyn Error>> {
        self.path.metadata().map_err(|e| e.into())
    }
    
    /// 合并处理结果（根据类型）
    fn combine_results(&self, results: Vec<T>) -> T {
        // 这里应该根据具体类型实现合并逻辑
        // 简化示例
        if !results.is_empty() {
            results.into_iter().next().unwrap()
        } else {
            // 根据具体类型返回默认值
            todo!("Return appropriate default value based on type")
        }
    }
}

impl Clone for FileReader {
    fn clone(&self) -> Self {
        Self {
            path: Arc::clone(&self.path),
            buffer_size: self.buffer_size,
            encoding: self.encoding,
        }
    }
}
```

#### 3.6.3.2 文件编码处理

**src/utilities/encoding.rs**

```rust
use std::io::{Read, Write, Result as IoResult};
use std::str;
use encoding_rs::{GBK, UTF_8};
use encoding_rs_io::DecodeReaderBytesBuilder;

pub enum TextEncoding {
    UTF8,
    GBK,
    ASCII,
}

impl TextEncoding {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "utf-8" | "utf8" => Some(TextEncoding::UTF8),
            "gbk" | "gb2312" => Some(TextEncoding::GBK),
            "ascii" => Some(TextEncoding::ASCII),
            _ => None,
        }
    }
    
    pub fn decode(&self, bytes: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            TextEncoding::UTF8 => {
                Ok(String::from_utf8(bytes.to_vec())?)
            }
            TextEncoding::GBK => {
                let (decoded, _, _) = GBK.decode(bytes);
                Ok(decoded.into())
            }
            TextEncoding::ASCII => {
                Ok(String::from_utf8(bytes.to_vec())?)
            }
        }
    }
    
    pub fn encode(&self, text: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match self {
            TextEncoding::UTF8 => {
                Ok(text.as_bytes().to_vec())
            }
            TextEncoding::GBK => {
                let (encoded, _, _) = GBK.encode(text);
                Ok(encoded.to_vec())
            }
            TextEncoding::ASCII => {
                Ok(text.as_bytes().to_vec())
            }
        }
    }
}

/// 通用编码读取器
pub struct EncodingReader<R> {
    reader: R,
    encoding: TextEncoding,
}

impl<R: Read> EncodingReader<R> {
    pub fn new(reader: R, encoding: TextEncoding) -> Self {
        Self { reader, encoding }
    }
    
    pub fn read_to_string(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        self.reader.read_to_end(&mut buffer)?;
        self.encoding.decode(&buffer)
    }
    
    pub fn read_lines(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let content = self.read_to_string()?;
        Ok(content.lines().map(|line| line.to_string()).collect())
    }
}
```

#### 3.6.3.3 并发处理

**src/concurrent/worker.rs**

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::error::Error;
use rayon::prelude::*;
use futures::executor::ThreadPool;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct WorkerPool {
    workers: Vec<Worker>,
    sender: crossbeam::channel::Sender<Job>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl WorkerPool {
    pub fn new(size: usize) -> WorkerPool {
        assert!(size > 0);
        
        let (sender, receiver) = crossbeam::channel::unbounded();
        let receiver = Arc::new(Mutex::new(receiver));
        
        let mut workers = Vec::with_capacity(size);
        
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        WorkerPool { workers, sender }
    }
    
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<crossbeam::channel::Receiver<Job>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();
            match job {
                Ok(job) => {
                    job();
                }
                Err(_) => {
                    // Shutdown signal received
                    break;
                }
            }
        });
        
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        // 关闭所有通道，发送shutdown信号
        drop(self.sender);
        
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// 文件处理作业
pub struct FileProcessingJob {
    files: Vec<PathBuf>,
    processor: Arc<dyn Fn(&Path) -> Result<(), Box<dyn Error>> + Send + Sync>,
    progress: Arc<Mutex<Progress>>,
}

#[derive(Debug, Default)]
pub struct Progress {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
}

impl FileProcessingJob {
    pub fn new(
        files: Vec<PathBuf>,
        processor: Arc<dyn Fn(&Path) -> Result<(), Box<dyn Error>> + Send + Sync>,
    ) -> Self {
        let total = files.len();
        Self {
            files,
            processor,
            progress: Arc::new(Mutex::new(Progress { total, ..Default::default() })),
        }
    }
    
    pub fn execute(&self, pool: &WorkerPool) -> Result<Progress, Box<dyn Error>> {
        // 使用rayon进行并行处理
        let results: Vec<Result<(), Box<dyn Error>>> = self.files
            .par_iter()
            .map(|file| {
                let result = (self.processor)(file);
                
                {
                    let mut progress = self.progress.lock().unwrap();
                    if result.is_ok() {
                        progress.completed += 1;
                    } else {
                        progress.failed += 1;
                    }
                }
                
                result
            })
            .collect();
        
        // 检查是否有失败的任务
        for result in results {
            result?;
        }
        
        Ok(self.progress.lock().unwrap().clone())
    }
    
    pub fn get_progress(&self) -> Progress {
        self.progress.lock().unwrap().clone()
    }
}
```

#### 3.6.3.4 文件处理器

**src/processors/text.rs**

```rust
use crate::utilities::file_ops::FileReader;
use crate::utilities::encoding::TextEncoding;
use std::path::Path;
use std::error::Error;
use std::collections::HashMap;

pub struct TextProcessor {
    encoding: TextEncoding,
    ignore_patterns: Vec<String>,
}

impl TextProcessor {
    pub fn new(encoding: TextEncoding) -> Self {
        Self {
            encoding,
            ignore_patterns: Vec::new(),
        }
    }
    
    pub fn add_ignore_pattern(&mut self, pattern: String) {
        self.ignore_patterns.push(pattern);
    }
    
    /// 文本搜索替换
    pub fn find_and_replace<P, Q>(
        &self,
        input: P,
        output: Q,
        replacements: &HashMap<&str, &str>,
    ) -> Result<usize, Box<dyn Error>>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let reader = FileReader::new(input).with_encoding(self.encoding);
        
        reader.stream_process(output, |line| {
            let mut result = line.to_string();
            for (from, to) in replacements {
                result = result.replace(from, to);
            }
            Ok(result)
        })?;
        
        Ok(replacements.len())
    }
    
    /// 提取文本统计信息
    pub fn analyze_text(&self, file: &Path) -> Result<TextStats, Box<dyn Error>> {
        let reader = FileReader::new(file);
        
        let stats = reader.process_lines(|line| {
            Ok((
                line.len(),
                line.chars().count(),
                line.split_whitespace().count(),
            ))
        })?;
        
        Ok(stats)
    }
    
    /// 去重文本行
    pub fn deduplicate<P, Q>(&self, input: P, output: Q) -> Result<usize, Box<dyn Error>>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let mut lines = std::fs::read_to_string(input)?;
        
        // 去重并保持顺序
        lines.lines().collect::<std::collections::HashSet<_>>();
        
        let reader = FileReader::new(input).with_encoding(self.encoding);
        let mut unique_lines = Vec::new();
        
        reader.process_lines(|line| {
            if !unique_lines.contains(&line) {
                unique_lines.push(line);
            }
            Ok(())
        })?;
        
        let output_content = unique_lines.join("\n");
        std::fs::write(output, output_content)?;
        
        Ok(unique_lines.len())
    }
}

#[derive(Debug, Default)]
pub struct TextStats {
    pub total_lines: usize,
    pub total_chars: usize,
    pub total_words: usize,
    pub avg_line_length: f64,
    pub longest_line: usize,
    pub shortest_line: usize,
}

impl std::fmt::Display for TextStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Text Statistics:
    Total lines: {}
    Total characters: {}
    Total words: {}
    Average line length: {:.2}
    Longest line: {} characters
    Shortest line: {} characters", 
            self.total_lines, self.total_chars, self.total_words, 
            self.avg_line_length, self.longest_line, self.shortest_line)
    }
}
```

#### 3.6.3.5 主程序

**src/main.rs**

```rust
use rust_file_processor::processors::text::TextProcessor;
use rust_file_processor::utilities::encoding::TextEncoding;
use rust_file_processor::concurrent::worker::FileProcessingJob;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use std::error::Error;
use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle};
use rayon;

fn main() -> Result<(), Box<dyn Error>> {
    // 配置rayon线程池
    rayon::ThreadPoolBuilder::new()
        .thread_name(|i| format!("worker-{}", i))
        .build_global()?;
    
    let matches = Command::new("rust-file-processor")
        .version("1.0")
        .about("Memory-safe file processing tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        
        .subcommand(
            Command::new("process")
                .about("Process files with text transformations")
                .arg(Arg::new("input")
                    .required(true)
                    .help("Input file or directory"))
                .arg(Arg::new("output")
                    .required(true)
                    .help("Output file or directory"))
                .arg(Arg::new("encoding")
                    .long("encoding")
                    .value_name("ENCODING")
                    .help("Text encoding (utf-8, gbk, ascii)")
                    .default_value("utf-8"))
                .arg(Arg::new("replace")
                    .long("replace")
                    .value_name("FROM=TO")
                    .help("Text replacement in format FROM=TO")
                    .multiple_values(true))
        )
        
        .subcommand(
            Command::new("analyze")
                .about("Analyze text files")
                .arg(Arg::new("input")
                    .required(true)
                    .help("Input file or directory"))
                .arg(Arg::new("encoding")
                    .long("encoding")
                    .value_name("ENCODING")
                    .help("Text encoding")
                    .default_value("utf-8"))
        )
        
        .subcommand(
            Command::new("verify")
                .about("Verify file integrity")
                .arg(Arg::new("input")
                    .required(true)
                    .help("Input file or directory"))
        )
        
        .get_matches();
    
    match matches.subcommand() {
        Some(("process", args)) => {
            let input = PathBuf::from(args.value_of("input").unwrap());
            let output = PathBuf::from(args.value_of("output").unwrap());
            let encoding = TextEncoding::from_name(args.value_of("encoding").unwrap())
                .ok_or("Invalid encoding")?;
            
            let mut processor = TextProcessor::new(encoding);
            
            if let Some(replacements) = args.values_of("replace") {
                let mut replace_map = HashMap::new();
                for replacement in replacements {
                    if let Some((from, to)) = replacement.split_once('=') {
                        replace_map.insert(from, to);
                    }
                }
                
                if input.is_file() {
                    let files = vec![input.clone()];
                    process_files_batch(&files, &output, &replace_map, &processor)?;
                } else {
                    process_directory_batch(&input, &output, &replace_map, &processor)?;
                }
            }
            
            println!("Processing completed successfully!");
        }
        
        Some(("analyze", args)) => {
            let input = PathBuf::from(args.value_of("input").unwrap());
            let encoding = TextEncoding::from_name(args.value_of("encoding").unwrap())
                .ok_or("Invalid encoding")?;
            
            let processor = TextProcessor::new(encoding);
            
            if input.is_file() {
                let stats = processor.analyze_text(&input)?;
                println!("{}", stats);
            } else {
                analyze_directory(&input, &processor)?;
            }
        }
        
        Some(("verify", args)) => {
            let input = PathBuf::from(args.value_of("input").unwrap());
            
            if input.is_file() {
                let result = rust_file_processor::utilities::file_ops::FileReader::new(&input)
                    .verify_integrity()?;
                println!("File integrity: {}", if result { "OK" } else { "FAILED" });
            } else {
                verify_directory(&input)?;
            }
        }
        
        _ => unreachable!(),
    }
    
    Ok(())
}

fn process_files_batch(
    files: &[PathBuf],
    output: &PathBuf,
    replacements: &HashMap<&str, &str>,
    processor: &TextProcessor,
) -> Result<(), Box<dyn Error>> {
    let total_files = files.len();
    let progress_bar = ProgressBar::new(total_files as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
    );
    
    for (i, file) in files.iter().enumerate() {
        let output_file = output.join(file.file_name().unwrap());
        let replacements = replacements.clone();
        let processor = processor.clone();
        
        let result = processor.find_and_replace(file, &output_file, &replacements);
        
        match result {
            Ok(_) => {
                progress_bar.set_message(format!("Processing: {:?}", file.file_name().unwrap()));
            }
            Err(e) => {
                eprintln!("Error processing {:?}: {}", file, e);
            }
        }
        
        progress_bar.inc(1);
    }
    
    progress_bar.finish_with_message("Processing complete!");
    Ok(())
}

fn process_directory_batch(
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    replacements: &HashMap<&str, &str>,
    processor: &TextProcessor,
) -> Result<(), Box<dyn Error>> {
    // 递归查找所有文件
    let files: Vec<PathBuf> = walkdir::WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .collect();
    
    std::fs::create_dir_all(output_dir)?;
    
    let total_files = files.len();
    let progress_bar = ProgressBar::new(total_files as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
    );
    
    // 使用并发处理
    let pool = rust_file_processor::concurrent::worker::WorkerPool::new(num_cpus::get());
    
    for file in files {
        let output_file = output_dir.join(file.strip_prefix(input_dir).unwrap());
        let replacements = replacements.clone();
        let processor = processor.clone();
        
        pool.execute(move || {
            let result = processor.find_and_replace(&file, &output_file, &replacements);
            match result {
                Ok(_) => println!("Processed: {:?}", file),
                Err(e) => eprintln!("Error processing {:?}: {}", file, e),
            }
        });
    }
    
    progress_bar.set_message("Processing all files...");
    drop(pool); // 等待所有任务完成
    
    progress_bar.finish_with_message("Batch processing complete!");
    Ok(())
}

fn analyze_directory(
    input_dir: &PathBuf,
    processor: &TextProcessor,
) -> Result<(), Box<dyn Error>> {
    let files: Vec<PathBuf> = walkdir::WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .collect();
    
    let progress_bar = ProgressBar::new(files.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
    );
    
    for file in files {
        if let Ok(stats) = processor.analyze_text(&file) {
            println!("\nFile: {:?}", file);
            println!("{}", stats);
        }
        progress_bar.inc(1);
    }
    
    progress_bar.finish_with_message("Analysis complete!");
    Ok(())
}

fn verify_directory(input_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let files: Vec<PathBuf> = walkdir::WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .collect();
    
    let progress_bar = ProgressBar::new(files.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
    );
    
    let mut failed_files = Vec::new();
    
    for file in files {
        let result = rust_file_processor::utilities::file_ops::FileReader::new(&file)
            .verify_integrity();
        
        match result {
            Ok(true) => println!("✓ {:?}", file),
            Ok(false) => {
                println!("✗ {:?}", file);
                failed_files.push(file);
            }
            Err(e) => {
                println!("✗ {:?} (Error: {})", file, e);
                failed_files.push(file);
            }
        }
        
        progress_bar.inc(1);
    }
    
    progress_bar.finish_with_message("Verification complete!");
    
    if !failed_files.is_empty() {
        eprintln!("\nFailed files:");
        for file in failed_files {
            eprintln!("  {:?}", file);
        }
        return Err("Some files failed integrity check".into());
    }
    
    println!("\nAll files passed integrity check!");
    Ok(())
}
```

#### 3.6.3.6 项目配置

**Cargo.toml**

```toml
[package]
name = "rust-file-processor"
version = "1.0.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Memory-safe file processing tool"
license = "MIT"
repository = "https://github.com/yourname/rust-file-processor"

[dependencies]
# 异步和并发
rayon = "1.7"
crossbeam = "0.8"
futures = "0.3"

# 文件处理
walkdir = "2.4"
encoding_rs = "0.8"
encoding_rs_io = "0.1"

# CLI
clap = { version = "4.0", features = ["derive"] }

# 用户界面
indicatif = "0.17"
colored = "2.0"

# 系统信息
num_cpus = "1.16"

# 错误处理
anyhow = "1.0"

[dev-dependencies]
tempfile = "3.5"
criterion = "0.5"

[[example]]
name = "basic_usage"
path = "examples/basic_usage.rs"

[[example]]
name = "concurrent_processing"
path = "examples/concurrent_processing.rs"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### 3.6.4 使用示例

#### 3.6.4.1 基础使用

**examples/basic_usage.rs**

```rust
use rust_file_processor::utilities::file_ops::FileReader;
use rust_file_processor::utilities::encoding::TextEncoding;
use rust_file_processor::processors::text::TextProcessor;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建文本处理器
    let processor = TextProcessor::new(TextEncoding::UTF8);
    
    // 读取文件并分析
    let stats = processor.analyze_text("sample.txt")?;
    println!("Text statistics: {}", stats);
    
    // 文本替换
    let mut replacements = HashMap::new();
    replacements.insert("hello", "world");
    replacements.insert("foo", "bar");
    
    processor.find_and_replace("input.txt", "output.txt", &replacements)?;
    println!("Text replacement completed");
    
    Ok(())
}
```

#### 3.6.4.2 并发处理示例

**examples/concurrent_processing.rs**

```rust
use rust_file_processor::concurrent::worker::{FileProcessingJob, WorkerPool};
use std::path::PathBuf;
use std::sync::Arc;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // 准备文件列表
    let files = vec![
        PathBuf::from("file1.txt"),
        PathBuf::from("file2.txt"),
        PathBuf::from("file3.txt"),
    ];
    
    // 创建处理器闭包
    let processor = Arc::new(|file: &Path| -> Result<(), Box<dyn Error>> {
        println!("Processing: {:?}", file);
        // 模拟文件处理
        std::thread::sleep(std::time::Duration::from_millis(100));
        Ok(())
    });
    
    // 创建处理作业
    let job = FileProcessingJob::new(files, processor);
    
    // 创建工作池
    let pool = WorkerPool::new(num_cpus::get());
    
    // 执行处理
    let progress = job.execute(&pool)?;
    
    println!("Processing completed:");
    println!("  Total: {}", progress.total);
    println!("  Completed: {}", progress.completed);
    println!("  Failed: {}", progress.failed);
    
    Ok(())
}
```

### 3.6.5 性能测试

**tests/performance.rs**

```rust
use rust_file_processor::utilities::file_ops::FileReader;
use std::path::Path;
use std::time::Instant;

#[test]
fn test_large_file_processing() {
    // 创建测试大文件
    let test_file = "test_large.txt";
    create_large_file(test_file, 1024 * 1024); // 1MB
    
    let start = Instant::now();
    
    // 测试流式处理
    let reader = FileReader::new(test_file).with_buffer_size(8192);
    let line_count = reader.process_lines(|_| Ok(()));
    
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 1000); // 应该在1秒内完成
    assert!(line_count.is_ok());
    
    // 清理
    std::fs::remove_file(test_file).ok();
}

fn create_large_file(path: &str, size: usize) {
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create(path).unwrap();
    let line = "This is a test line for large file processing. ".repeat(10);
    
    for _ in 0..(size / line.len()) {
        writeln!(file, "{}", line).unwrap();
    }
}
```

### 3.6.6 生产级考虑

#### 3.6.6.1 内存使用监控

在生产环境中，监控内存使用至关重要：

```rust
use sysinfo::{System, SystemExt, ProcessExt};

pub struct MemoryMonitor {
    sys: System,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
        }
    }
    
    pub fn get_memory_usage(&mut self) -> MemoryUsage {
        self.sys.refresh_all();
        
        MemoryUsage {
            total: self.sys.total_memory(),
            available: self.sys.available_memory(),
            used: self.sys.used_memory(),
            used_percent: (self.sys.used_memory() as f64 / self.sys.total_memory() as f64) * 100.0,
        }
    }
    
    pub fn warn_if_high_usage(&mut self, threshold: f64) -> bool {
        let usage = self.get_memory_usage();
        if usage.used_percent > threshold {
            eprintln!("Warning: Memory usage is {}%", usage.used_percent);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub used_percent: f64,
}
```

#### 3.6.6.2 原子操作

确保文件操作的原子性：

```rust
use std::fs;
use std::path::Path;
use std::io::{self, Write, Read};

pub fn atomic_file_write<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    content: C
) -> io::Result<()> {
    let temp_path = format!("{}.tmp", path.as_ref().display());
    
    // 写入临时文件
    {
        let mut temp_file = fs::File::create(&temp_path)?;
        temp_file.write_all(content.as_ref())?;
    }
    
    // 原子性重命名
    fs::rename(&temp_path, path)?;
    Ok(())
}
```

#### 3.6.6.3 错误恢复机制

```rust
use std::collections::HashSet;
use std::path::PathBuf;

pub struct ErrorRecovery {
    processed_files: HashSet<PathBuf>,
    failed_files: Vec<PathBuf>,
    max_retries: usize,
}

impl ErrorRecovery {
    pub fn new(max_retries: usize) -> Self {
        Self {
            processed_files: HashSet::new(),
            failed_files: Vec::new(),
            max_retries,
        }
    }
    
    pub fn mark_processed(&mut self, file: PathBuf) {
        self.processed_files.insert(file);
    }
    
    pub fn mark_failed(&mut self, file: PathBuf) {
        self.failed_files.push(file);
    }
    
    pub fn retry_failed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut retry_count = 0;
        
        while !self.failed_files.is_empty() && retry_count < self.max_retries {
            retry_count += 1;
            let failed = std::mem::take(&mut self.failed_files);
            
            for file in failed {
                if self.processed_files.contains(&file) {
                    continue; // 已处理，跳过
                }
                
                // 重试处理
                match self.process_file(&file) {
                    Ok(_) => {
                        self.mark_processed(file);
                    }
                    Err(_) => {
                        self.failed_files.push(file); // 重新加入失败列表
                    }
                }
            }
        }
        
        if !self.failed_files.is_empty() {
            return Err("Some files failed to process after retries".into());
        }
        
        Ok(())
    }
    
    fn process_file(&self, file: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // 实现文件处理逻辑
        Ok(())
    }
}
```

## 3.7 最佳实践

### 3.7.1 所有权使用原则

1. **优先借用**：除非必要，否则使用引用而非获取所有权
2. **移动而非复制**：对于大型数据结构，优先移动所有权而非复制
3. **智能指针选择**：
   - 使用`Box<T>`在堆上分配单所有权值
   - 使用`Rc<T>`在单线程环境共享多所有权
   - 使用`Arc<T>`在多线程环境共享多所有权

### 3.7.2 借用检查器技巧

1. **使用可变借用分离**：
   ```rust
   let mut data = vec![1, 2, 3];
   let first = &data[0];
   // 这里不能获取可变引用
   // let first_mut = &mut data[0];
   ```

2. **使用内部可变性**：
   ```rust
   use std::cell::RefCell;
   let data = RefCell::new(vec![1, 2, 3]);
   data.borrow_mut().push(4); // 可以在不可变引用内部修改
   ```

3. **使用生命周期参数**：
   ```rust
   fn longest_with_announcement<'a, T>(
       x: &'a str,
       y: &'a str,
       ann: T,
   ) -> &'a str
   where
       T: std::fmt::Display,
   {
       println!("Announcement: {}", ann);
       if x.len() > y.len() {
           x
       } else {
           y
       }
   }
   ```

### 3.7.3 性能优化

1. **避免不必要的所有权转移**：
   ```rust
   // 不好的方式
   fn process_string(s: String) -> String {
       // 处理字符串
       s
   }
   
   // 好的方式
   fn process_string(s: &str) -> String {
       // 处理字符串
       s.to_string()
   }
   ```

2. **使用零拷贝技术**：
   ```rust
   fn parse_csv_line(line: &str) -> (&str, &str, &str) {
       // 使用切片而不是创建新字符串
       let mut iter = line.split(',');
       (
           iter.next().unwrap(),
           iter.next().unwrap(),
           iter.next().unwrap(),
       )
   }
   ```

3. **批量处理减少内存分配**：
   ```rust
   fn process_batch(items: &[Item], batch_size: usize) {
       for chunk in items.chunks(batch_size) {
           process_chunk(chunk); // 批量处理减少分配
       }
   }
   ```

## 3.8 总结

本章深入探讨了Rust的所有权系统，这是Rust语言的核心特性之一。通过本章的学习，您已经：

1. **理解了所有权概念**：每个值都有一个所有者，作用域结束时自动清理
2. **掌握了借用机制**：通过引用安全地使用值而不转移所有权
3. **学会了生命周期管理**：确保引用的有效性
4. **了解了智能指针**：处理复杂的所有权情况
5. **构建了实用项目**：开发了内存安全的文件处理工具

所有权系统让Rust在保持高性能的同时提供了内存安全保证，这是现代系统编程的重要进步。在实际开发中，合理使用借用、智能指针和生命周期注解，可以构建既安全又高效的Rust应用程序。

## 3.9 验收标准

完成本章后，您应该能够：

- [ ] 解释所有权、借用和生命周期的关系
- [ ] 识别并解决借用检查器错误
- [ ] 选择合适的智能指针类型
- [ ] 实现内存安全的文件处理程序
- [ ] 编写高效的批量数据处理代码
- [ ] 设计生产级的错误处理和恢复机制

## 3.10 练习题

1. **所有权转换**：实现一个函数，接受一个Vec<T>的所有权，返回处理后的Vec<T>
2. **借用优化**：重构代码以使用借用而不是所有权的转移
3. **生命周期注解**：为复杂的函数添加正确的生命周期参数
4. **性能测试**：对比使用借用和所有权转移的性能差异
5. **错误处理**：为文件处理工具添加重试机制和回滚功能

## 3.11 扩展阅读

- [Rust官方文档：所有权](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Rustnomicon：高级所有权](https://doc.rust-lang.org/nomicon/ownership.html)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)