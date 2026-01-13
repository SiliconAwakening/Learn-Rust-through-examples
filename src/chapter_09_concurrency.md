# 第9章：并发编程

## 学习目标
- 掌握Rust的安全并发机制
- 理解线程和异步编程的区别
- 学会使用消息传递和共享内存进行并发通信
- 掌握异步编程的基础概念和实践
- 构建高并发的Web服务器

---

## 9.1 并发基础概念

### 9.1.1 并发与并行的区别

**并发（Concurrency）**：多个任务在重叠的时间段内执行，但不一定是同时执行
**并行（Parallelism）**：多个任务真正同时执行

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== 并发示例：切换执行 ===");
    
    // 模拟三个独立的任务
    for i in 1..=3 {
        thread::spawn(move || {
            for j in 1..=5 {
                println!("任务 {}: 步骤 {}", i, j);
                thread::sleep(Duration::from_millis(200));
            }
        });
    }
    
    thread::sleep(Duration::from_millis(1000));
    println!("主线程完成");
}
```

```rust
use rayon::prelude::*;

fn main() {
    println!("=== 并行示例：真正同时执行 ===");
    
    let data: Vec<i32> = (1..=1000).collect();
    
    // 使用Rayon进行并行处理
    let result: Vec<i32> = data
        .par_iter()
        .map(|&x| {
            // 模拟CPU密集型计算
            (0..1000).sum::<i32>() * x
        })
        .collect();
    
    println!("并行处理完成，结果数量: {}", result.len());
}
```

### 9.1.2 Rust的并发哲学

Rust的并发安全性基于三个核心原则：

1. **所有权系统**：防止数据竞争
2. **借用检查器**：确保内存安全
3. **类型系统**：通过`Send`和`Sync`标记保证线程安全

```rust
use std::sync::{Arc, Mutex};
use std::thread;

// 不安全的示例（编译失败）
fn unsafe_concurrent_access() {
    let mut counter = 0;
    
    // 这会导致编译错误，因为多个线程试图访问可变数据
    // thread::spawn(move || {
    //     counter += 1;  // 编译错误
    // });
    //
    // thread::spawn(move || {
    //     counter += 1;  // 编译错误
    // });
}

// 安全的示例
fn safe_concurrent_access() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("最终计数: {}", *counter.lock().unwrap());
}
```

---

## 9.2 线程编程

### 9.2.1 基础线程操作

```rust
use std::thread;
use std::time::Duration;

fn basic_thread_creation() {
    // 创建线程的基本方法
    let handle = thread::spawn(|| {
        for i in 1..=5 {
            println!("新线程: {}", i);
            thread::sleep(Duration::from_millis(100));
        }
    });
    
    // 主线程继续执行
    for i in 1..=3 {
        println!("主线程: {}", i);
        thread::sleep(Duration::from_millis(150));
    }
    
    // 等待子线程完成
    handle.join().unwrap();
    println!("所有线程完成");
}
```

#### 线程参数传递

```rust
fn thread_with_parameters() {
    let data = vec![1, 2, 3, 4, 5];
    
    // 使用move关键字将所有权转移给线程
    let handle = thread::spawn(move || {
        let sum: i32 = data.iter().sum();
        println!("子线程计算的和: {}", sum);
    });
    
    // 在主线程中无法再使用data
    // println!("{:?}", data);  // 编译错误
    
    handle.join().unwrap();
}
```

#### 线程返回值

```rust
use std::sync::mpsc;

fn thread_return_value() {
    // 创建通道
    let (tx, rx) = mpsc::channel();
    
    let handle = thread::spawn(move || {
        let result = calculate_fibonacci(10);
        tx.send(result).unwrap();
    });
    
    // 接收线程的返回值
    let result = rx.recv().unwrap();
    println!("斐波那契数列第10项: {}", result);
    
    handle.join().unwrap();
}

fn calculate_fibonacci(n: u32) -> u32 {
    if n <= 1 {
        n
    } else {
        calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)
    }
}
```

### 9.2.2 线程池

#### 简单的线程池实现

```rust
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    _id: usize,
    thread: thread::JoinHandle<()>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        ThreadPool { workers, sender }
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
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();
            
            match job {
                Ok(job) => {
                    println!("Worker {} 执行任务", id);
                    job();
                }
                Err(_) => {
                    println!("Worker {} 断开连接，退出", id);
                    break;
                }
            }
        });
        
        Worker { _id: id, thread }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 发送停止信号
        for _ in &self.workers {
            if let Err(e) = self.sender.send(Box::new(|| {})) {
                eprintln!("发送停止信号失败: {:?}", e);
            }
        }
        
        // 等待所有线程完成
        for worker in &mut self.workers {
            if let Some(handle) = worker.thread.take() {
                handle.join().unwrap();
            }
        }
    }
}
```

#### 使用线程池

```rust
fn thread_pool_example() {
    let pool = ThreadPool::new(4);
    
    for i in 1..=10 {
        pool.execute(move || {
            println!("处理任务 {} (线程ID: {:?})", 
                    i, 
                    std::thread::current().id());
            thread::sleep(Duration::from_millis(1000));
        });
    }
    
    println!("所有任务已提交，等待完成...");
    // ThreadPool在作用域结束时自动清理
}
```

---

## 9.3 消息传递

### 9.3.1 通道（Channel）

Rust的标准库提供了通道（mpsc - multiple producer, single consumer）进行消息传递。

#### 基础通道使用

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn basic_channel() {
    let (tx, rx) = mpsc::channel();
    
    // 创建生产者线程
    let handle = thread::spawn(move || {
        for i in 1..=5 {
            tx.send(format!("消息 {}", i)).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
    });
    
    // 接收消息
    for received in rx.iter() {
        println!("收到: {}", received);
    }
    
    handle.join().unwrap();
}
```

#### 多生产者通道

```rust
fn multiple_producers() {
    let (tx, rx) = mpsc::channel();
    let tx1 = mpsc::Sender::clone(&tx);
    let tx2 = mpsc::Sender::clone(&tx);
    
    // 启动三个生产者
    thread::spawn(move || {
        for i in 1..=3 {
            tx1.send(format!("生产者1: 消息{}", i)).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });
    
    thread::spawn(move || {
        for i in 1..=3 {
            tx2.send(format!("生产者2: 消息{}", i)).unwrap();
            thread::sleep(Duration::from_millis(150));
        }
    });
    
    thread::spawn(move || {
        for i in 1..=3 {
            tx.send(format!("生产者3: 消息{}", i)).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
    });
    
    // 接收所有消息
    for received in rx.iter().take(9) {
        println!("消费者收到: {}", received);
    }
}
```

#### 同步通道

```rust
fn synchronous_channel() {
    // 创建同步通道，容量为0，需要接收者准备好
    let (tx, rx) = mpsc::sync_channel(0);
    
    let handle = thread::spawn(move || {
        println!("生产者: 准备发送消息");
        tx.send("同步消息").unwrap();
        println!("生产者: 消息已发送");
    });
    
    println!("消费者: 准备接收消息");
    let message = rx.recv().unwrap();
    println!("消费者: 收到消息: {}", message);
    
    handle.join().unwrap();
}
```

### 9.3.2 生产者-消费者模式

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct SharedQueue<T> {
    queue: Arc<Mutex<Vec<T>>>,
    capacity: usize,
}

impl<T> SharedQueue<T> {
    fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            capacity,
        }
    }
    
    fn push(&self, item: T) -> Result<(), String> {
        let mut queue = self.queue.lock().unwrap();
        if queue.len() >= self.capacity {
            return Err("队列已满".to_string());
        }
        queue.push(item);
        Ok(())
    }
    
    fn pop(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop()
    }
    
    fn len(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }
}

fn producer_consumer_example() {
    let shared_queue = SharedQueue::new(10);
    let shared_queue_clone = shared_queue.clone();
    
    // 生产者
    let producer = thread::spawn(move || {
        for i in 1..=20 {
            while let Err(_) = shared_queue_clone.push(i) {
                thread::sleep(Duration::from_millis(10));
            }
            println!("生产者: 添加了 {}", i);
            thread::sleep(Duration::from_millis(50));
        }
    });
    
    // 消费者
    let consumer = thread::spawn(move || {
        loop {
            match shared_queue.pop() {
                Some(item) => {
                    println!("消费者: 处理了 {}", item);
                    thread::sleep(Duration::from_millis(100));
                }
                None => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    });
    
    producer.join().unwrap();
    
    // 等待一段时间后消费结束
    thread::sleep(Duration::from_secs(3));
    println!("程序结束");
}
```

### 9.3.3 复杂消息传递

#### 任务调度器

```rust
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
enum Task {
    Immediate(String),
    Delayed(String, Duration),
}

struct TaskScheduler {
    sender: mpsc::Sender<Task>,
}

impl TaskScheduler {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        
        // 启动调度器线程
        thread::spawn(move || {
            let mut scheduled_tasks: Vec<(Instant, Task)> = Vec::new();
            
            for task in receiver.iter() {
                match task {
                    Task::Immediate(task) => {
                        println!("立即执行任务: {}", task);
                    }
                    Task::Delayed(task, delay) => {
                        let execution_time = Instant::now() + delay;
                        scheduled_tasks.push((execution_time, Task::Immediate(task)));
                    }
                }
                
                // 检查是否有任务到期
                let now = Instant::now();
                scheduled_tasks.retain(|(time, task)| {
                    if *time <= now {
                        if let Task::Immediate(task) = task {
                            println!("执行延迟任务: {}", task);
                        }
                        false
                    } else {
                        true
                    }
                });
            }
        });
        
        Self { sender }
    }
    
    fn schedule_immediate(&self, task: String) {
        self.sender.send(Task::Immediate(task)).unwrap();
    }
    
    fn schedule_delayed(&self, task: String, delay: Duration) {
        self.sender.send(Task::Delayed(task, delay)).unwrap();
    }
}

fn task_scheduler_example() {
    let scheduler = TaskScheduler::new();
    
    scheduler.schedule_immediate("立即任务1".to_string());
    scheduler.schedule_immediate("立即任务2".to_string());
    scheduler.schedule_delayed("延迟任务".to_string(), Duration::from_secs(2));
    
    thread::sleep(Duration::from_secs(3));
}
```

---

## 9.4 共享内存

### 9.4.1 Arc和Mutex

#### Arc（原子引用计数）

```rust
use std::sync::Arc;
use std::thread;

fn arc_example() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    
    let mut handles = vec![];
    for i in 0..5 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let len = data.len();
            println!("线程 {}: 数组长度 = {}", i, len);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

#### Mutex（互斥锁）

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn mutex_example() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("最终计数: {}", *counter.lock().unwrap());
}
```

### 9.4.2 读写锁（RwLock）

```rust
use std::sync::{Arc, RwLock};
use std::thread;

fn rwlock_example() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));
    let mut handles = vec![];
    
    // 多个读者
    for i in 0..5 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let data = data.read().unwrap();
            println!("读者 {}: 数据 = {:?}", i, data);
        });
        handles.push(handle);
    }
    
    // 多个写者
    for i in 0..3 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let mut data = data.write().unwrap();
            data.push(i);
            println!("写者 {}: 添加了 {}", i, i);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let data = data.read().unwrap();
    println!("最终数据: {:?}", *data);
}
```

### 9.4.3 条件变量（Condvar）

```rust
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;

struct SharedState {
    ready: bool,
    data: Option<String>,
}

fn condition_variable_example() {
    let state = Arc::new((Mutex::new(SharedState { ready: false, data: None }), Condvar::new()));
    let state_clone = Arc::clone(&state);
    
    // 生产者线程
    let producer = thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        let (lock, cvar) = &*state_clone;
        let mut shared = lock.lock().unwrap();
        shared.ready = true;
        shared.data = Some("数据已准备".to_string());
        cvar.notify_all();
        println!("生产者: 数据已准备");
    });
    
    // 消费者线程
    let mut consumers = vec![];
    for i in 0..3 {
        let state = Arc::clone(&state);
        let handle = thread::spawn(move || {
            let (lock, cvar) = &*state;
            let mut shared = lock.lock().unwrap();
            
            while !shared.ready {
                shared = cvar.wait(shared).unwrap();
            }
            
            if let Some(ref data) = shared.data {
                println!("消费者 {}: 收到数据: {}", i, data);
            }
        });
        consumers.push(handle);
    }
    
    producer.join().unwrap();
    for handle in consumers {
        handle.join().unwrap();
    }
}
```

### 9.4.4 无锁数据结构

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct LockFreeCounter {
    count: AtomicU64,
}

impl LockFreeCounter {
    fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
        }
    }
    
    fn increment(&self) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }
    
    fn get(&self) -> u64 {
        self.count.load(Ordering::SeqCst)
    }
}

fn lock_free_counter_example() {
    let counter = Arc::new(LockFreeCounter::new());
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                counter.increment();
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("最终计数: {}", counter.get());
}
```

---

## 9.5 异步编程

### 9.5.1 async/await基础

#### 异步函数定义

```rust
use tokio::time::{sleep, Duration};

async fn async_function() -> String {
    println!("异步函数开始执行");
    sleep(Duration::from_secs(1)).await;
    println!("异步函数等待完成");
    "异步结果".to_string()
}

fn async_main() {
    // 异步函数需要运行时执行
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async_function());
    println!("结果: {}", result);
}
```

#### 多个异步任务

```rust
async fn multiple_async_tasks() {
    let task1 = async {
        sleep(Duration::from_secs(1)).await;
        "任务1完成"
    };
    
    let task2 = async {
        sleep(Duration::from_secs(2)).await;
        "任务2完成"
    };
    
    let task3 = async {
        sleep(Duration::from_millis(500)).await;
        "任务3完成"
    };
    
    // 并行执行所有任务
    let (result1, result2, result3) = tokio::join!(task1, task2, task3);
    
    println!("{} - {} - {}", result1, result2, result3);
}
```

### 9.5.2 Tokio异步运行时

#### 基础Tokio使用

```rust
use tokio::time::{sleep, Duration, Instant};

#[tokio::main]
async fn main() {
    println!("=== Tokio 异步示例 ===");
    
    let start = Instant::now();
    
    // 启动多个异步任务
    let task1 = tokio::spawn(async {
        sleep(Duration::from_secs(2)).await;
        "任务1完成"
    });
    
    let task2 = tokio::spawn(async {
        sleep(Duration::from_secs(1)).await;
        "任务2完成"
    });
    
    let task3 = tokio::spawn(async {
        sleep(Duration::from_millis(500)).await;
        "任务3完成"
    });
    
    // 等待所有任务完成
    let (result1, result2, result3) = tokio::join!(task1, task2, task3);
    
    println!("执行时间: {:?}", start.elapsed());
    println!("结果: {} - {} - {}", 
             result1.unwrap(), 
             result2.unwrap(), 
             result3.unwrap());
}
```

#### 异步通道

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn async_channel_example() {
    let (tx, mut rx) = mpsc::channel(100);
    
    // 启动发送者
    let tx_clone = tx.clone();
    let sender = tokio::spawn(async move {
        for i in 1..=10 {
            let message = format!("消息 {}", i);
            tx_clone.send(message).await.unwrap();
            sleep(Duration::from_millis(100)).await;
        }
    });
    
    // 启动接收者
    let receiver = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            println!("收到: {}", message);
        }
    });
    
    // 等待完成
    let _ = tokio::join!(sender, receiver);
}
```

### 9.5.3 异步流（Stream）

```rust
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

async fn stream_example() {
    // 创建定时器流
    let mut interval = interval(Duration::from_millis(500));
    let mut counter = 0;
    
    loop {
        interval.tick().await;
        counter += 1;
        println!("流事件: {}", counter);
        
        if counter >= 5 {
            break;
        }
    }
    
    println!("流处理完成");
}

async fn broadcast_stream_example() {
    use tokio::sync::broadcast;
    
    let (tx, _) = broadcast::channel(16);
    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();
    
    // 启动接收者
    let receiver1 = tokio::spawn(async move {
        while let Ok(message) = rx1.recv().await {
            println!("接收者1: {}", message);
        }
    });
    
    let receiver2 = tokio::spawn(async move {
        while let Ok(message) = rx2.recv().await {
            println!("接收者2: {}", message);
        }
    });
    
    // 发送消息
    for i in 1..=5 {
        tx.send(format!("广播消息 {}", i)).unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    let _ = tokio::join!(receiver1, receiver2);
}
```

### 9.5.4 Select语句

```rust
use tokio::time::{sleep, Duration, Instant};

async fn select_example() {
    let start = Instant::now();
    
    // 模拟多个竞争的操作
    let operation1 = async {
        sleep(Duration::from_millis(800)).await;
        "操作1完成 (800ms)"
    };
    
    let operation2 = async {
        sleep(Duration::from_millis(300)).await;
        "操作2完成 (300ms)"
    };
    
    // 使用select选择最快完成的操作
    tokio::select! {
        result1 = operation1 => {
            println!("操作1: {}", result1);
        }
        result2 = operation2 => {
            println!("操作2: {}", result2);
        }
    }
    
    println!("总用时: {:?}", start.elapsed());
}

#[tokio::main]
async fn select_with_channel() {
    let (tx1, mut rx1) = mpsc::channel(32);
    let (tx2, mut rx2) = mpsc::channel(32);
    
    // 发送消息
    tokio::spawn(async move {
        tx1.send("通道1消息").await.unwrap();
    });
    
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        tx2.send("通道2消息").await.unwrap();
    });
    
    // 从最快响应的通道接收
    tokio::select! {
        msg1 = rx1.recv() => {
            println!("从通道1收到: {:?}", msg1);
        }
        msg2 = rx2.recv() => {
            println!("从通道2收到: {:?}", msg2);
        }
    }
}
```

---

## 9.6 实战项目：高并发Web服务器

### 9.6.1 服务器架构设计

让我们构建一个完整的高并发Web服务器，展示所有并发技术的应用。

```rust
// src/main.rs
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::server::HttpServer;
use crate::thread_pool::ThreadPool;
use crate::config::ServerConfig;

mod server;
mod config;
mod handler;
mod thread_pool;
mod http;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let config = ServerConfig::load()?;
    
    // 启动服务器
    let server = HttpServer::new(config.port, config.max_connections);
    println!("启动HTTP服务器，监听端口 {}", config.port);
    
    server.listen().await?;
    Ok(())
}
```

### 9.6.2 HTTP服务器实现

```rust
// src/server.rs
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use crate::thread_pool::ThreadPool;
use crate::http::HttpRequest;
use crate::handler::RequestHandler;

pub struct HttpServer {
    port: u16,
    max_connections: usize,
    thread_pool: ThreadPool,
}

impl HttpServer {
    pub fn new(port: u16, max_connections: usize) -> Self {
        Self {
            port,
            max_connections,
            thread_pool: ThreadPool::new(4),
        }
    }
    
    pub async fn listen(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(("127.0.0.1", self.port)).await?;
        println!("服务器监听在 127.0.0.1:{}", self.port);
        
        // 接受连接
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let handler = RequestHandler::new();
                    self.handle_connection(stream, handler);
                }
                Err(e) => {
                    eprintln!("接受连接失败: {:?}", e);
                }
            }
        }
    }
    
    fn handle_connection(&self, stream: TcpStream, handler: RequestHandler) {
        self.thread_pool.execute(move || {
            // 使用阻塞式I/O处理HTTP请求
            match handler.handle(stream) {
                Ok(_) => println!("请求处理完成"),
                Err(e) => eprintln!("请求处理失败: {:?}", e),
            }
        });
    }
}
```

### 9.6.3 HTTP解析器

```rust
// src/http.rs
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpRequest {
    pub fn parse(stream: &mut TcpStream) -> Result<Self, std::io::Error> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        
        // 解析请求行
        reader.read_line(&mut line)?;
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        
        if parts.len() != 3 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "无效的HTTP请求行"
            ));
        }
        
        let method = parts[0].to_string();
        let path = parts[1].to_string();
        let version = parts[2].to_string();
        
        // 解析请求头
        let mut headers = HashMap::new();
        let mut content_length = 0;
        
        loop {
            line.clear();
            reader.read_line(&mut line)?;
            let line = line.trim();
            
            if line.is_empty() {
                break;
            }
            
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                
                if key.to_lowercase() == "content-length" {
                    content_length = value.parse().unwrap_or(0);
                }
                
                headers.insert(key, value);
            }
        }
        
        // 读取请求体
        let mut body = None;
        if content_length > 0 {
            let mut body_bytes = vec![0u8; content_length];
            reader.read_exact(&mut body_bytes)?;
            body = Some(String::from_utf8_lossy(&body_bytes).to_string());
        }
        
        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}

pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn new(status_code: u16, body: String) -> Self {
        let status_text = match status_code {
            200 => "OK",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/html".to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());
        headers.insert("Server".to_string(), "Rust-HTTP-Server/1.0".to_string());
        headers.insert("Date".to_string(), Self::current_date());
        
        Self {
            status_code,
            status_text: status_text.to_string(),
            headers,
            body,
        }
    }
    
    pub fn not_found() -> Self {
        Self::new(404, "404 Not Found".to_string())
    }
    
    pub fn internal_error() -> Self {
        Self::new(500, "500 Internal Server Error".to_string())
    }
    
    pub fn ok() -> Self {
        Self::new(200, "OK".to_string())
    }
    
    fn current_date() -> String {
        let now = SystemTime::now();
        let datetime = now.duration_since(UNIX_EPOCH).unwrap();
        format!("{}", datetime.as_secs())
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = String::new();
        response.push_str(&format!("HTTP/1.1 {} {}\r\n", self.status_code, self.status_text));
        
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str("\r\n");
        response.push_str(&self.body);
        
        response.into_bytes()
    }
}
```

### 9.6.4 请求处理器

```rust
// src/handler.rs
use std::net::TcpStream;
use crate::http::{HttpRequest, HttpResponse};
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

pub struct RequestHandler {
    request_counter: Arc<AtomicU64>,
}

impl RequestHandler {
    pub fn new() -> Self {
        Self {
            request_counter: Arc::new(AtomicU64::new(0)),
        }
    }
    
    pub fn handle(&self, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        // 增加请求计数
        let count = self.request_counter.fetch_add(1, Ordering::SeqCst);
        
        // 解析请求
        let request = match HttpRequest::parse(&mut stream) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("解析请求失败: {:?}", e);
                let response = HttpResponse::internal_error();
                stream.write_all(&response.to_bytes())?;
                return Ok(());
            }
        };
        
        println!("处理请求 #{}: {} {}", count, request.method, request.path);
        
        // 路由处理
        let response = self.route(&request);
        
        // 发送响应
        stream.write_all(&response.to_bytes())?;
        stream.flush()?;
        
        Ok(())
    }
    
    fn route(&self, request: &HttpRequest) -> HttpResponse {
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/") => self.handle_index(),
            ("GET", "/health") => self.handle_health(),
            ("GET", "/status") => self.handle_status(),
            ("GET", path) if path.starts_with("/api/") => self.handle_api(request),
            ("POST", "/api/data") => self.handle_post_data(request),
            _ => HttpResponse::not_found(),
        }
    }
    
    fn handle_index(&self) -> HttpResponse {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Rust HTTP Server</title>
        </head>
        <body>
            <h1>欢迎使用Rust高并发HTTP服务器</h1>
            <p>支持的端点:</p>
            <ul>
                <li><a href="/health">健康检查</a></li>
                <li><a href="/status">状态信息</a></li>
                <li>GET /api/data - 获取数据</li>
                <li>POST /api/data - 提交数据</li>
            </ul>
        </body>
        </html>
        "#;
        
        HttpResponse::new(200, html.to_string())
    }
    
    fn handle_health(&self) -> HttpResponse {
        let health = r#"{"status": "healthy", "timestamp": "}"#;
        HttpResponse::new(200, health.to_string())
    }
    
    fn handle_status(&self) -> HttpResponse {
        let total_requests = self.request_counter.load(Ordering::SeqCst);
        let status = format!(
            r#"{{"total_requests": {}, "server": "Rust-HTTP-Server", "status": "running"}}"#,
            total_requests
        );
        HttpResponse::new(200, status)
    }
    
    fn handle_api(&self, request: &HttpRequest) -> HttpResponse {
        let response = format!(
            r#"{{"method": "{}", "path": "{}", "message": "API响应"}}"#,
            request.method, request.path
        );
        HttpResponse::new(200, response)
    }
    
    fn handle_post_data(&self, request: &HttpRequest) -> HttpResponse {
        let body = request.body.as_deref().unwrap_or("");
        let response = format!(
            r#"{{"received": "{}", "processed": true}}"#,
            body
        );
        HttpResponse::new(200, response)
    }
}
```

### 9.6.5 配置管理

```rust
// src/config.rs
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub max_connections: usize,
    pub max_request_size: usize,
    pub worker_threads: usize,
}

impl ServerConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // 尝试从配置文件加载，如果没有则使用默认值
        let config_path = "server.conf";
        
        if Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path)?;
            self::parse_config(&content)
        } else {
            Ok(Self::default())
        }
    }
    
    fn parse_config(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut port = 8080;
        let mut max_connections = 1000;
        let mut max_request_size = 1024 * 1024; // 1MB
        let mut worker_threads = 4;
        
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            
            if let Some(equals_pos) = line.find('=') {
                let key = line[..equals_pos].trim();
                let value = line[equals_pos + 1..].trim();
                
                match key {
                    "port" => port = value.parse()?,
                    "max_connections" => max_connections = value.parse()?,
                    "max_request_size" => max_request_size = value.parse()?,
                    "worker_threads" => worker_threads = value.parse()?,
                    _ => {}
                }
            }
        }
        
        Ok(ServerConfig {
            port,
            max_connections,
            max_request_size,
            worker_threads,
        })
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            max_connections: 1000,
            max_request_size: 1024 * 1024,
            worker_threads: 4,
        }
    }
}
```

### 9.6.6 性能测试

```rust
// src/benchmarks.rs
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use crate::server::HttpServer;

pub struct BenchmarkRunner {
    server: Arc<HttpServer>,
}

impl BenchmarkRunner {
    pub fn new(server: HttpServer) -> Self {
        Self {
            server: Arc::new(server),
        }
    }
    
    pub fn run_concurrent_requests(&self, concurrent_count: usize, total_requests: usize) {
        println!("开始性能测试: {} 并发，{} 总请求", concurrent_count, total_requests);
        
        let start = Instant::now();
        let mut handles = vec![];
        
        // 启动多个客户端线程
        for _ in 0..concurrent_count {
            let server = Arc::clone(&self.server);
            let handle = thread::spawn(move || {
                Self::simulate_client(server, total_requests / concurrent_count);
            });
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        
        let duration = start.elapsed();
        let qps = total_requests as f64 / duration.as_secs_f64();
        
        println!("性能测试结果:");
        println!("  总耗时: {:?}", duration);
        println!("  QPS: {:.2} 请求/秒", qps);
        println!("  平均延迟: {:?} 毫秒", 
                 duration / total_requests as u32);
    }
    
    fn simulate_client(server: Arc<HttpServer>, request_count: usize) {
        for _ in 0..request_count {
            // 这里可以模拟实际的HTTP客户端
            // 在实际实现中会连接到服务器并发送请求
            thread::sleep(Duration::from_millis(1));
        }
    }
}
```

---

## 9.7 最佳实践与性能优化

### 9.7.1 并发模式选择

#### 1. I/O密集型 vs CPU密集型

```rust
// I/O密集型：使用异步编程
use tokio::fs;
use tokio::time::sleep;

async fn io_intensive_task() {
    // 并发I/O操作
    let file_tasks = vec![
        fs::read_to_string("file1.txt"),
        fs::read_to_string("file2.txt"),
        fs::read_to_string("file3.txt"),
    ];
    
    let results = tokio::join!(futures::future::join_all(file_tasks));
    println!("I/O密集型任务完成");
}

// CPU密集型：使用多线程
use rayon::prelude::*;

fn cpu_intensive_task() {
    let data: Vec<i32> = (1..=1_000_000).collect();
    
    let result: Vec<i32> = data
        .par_iter()
        .map(|&x| {
            // CPU密集型计算
            (0..1000).sum::<i32>() * x
        })
        .collect();
    
    println!("CPU密集型任务完成，处理的元素数: {}", result.len());
}
```

#### 2. 选择合适的同步原语

```rust
use std::sync::{Arc, Mutex, RwLock, atomic::AtomicU64};
use std::collections::HashMap;

// 大量写入操作：使用Mutex
fn heavy_writes() {
    let data = Arc::new(Mutex::new(HashMap::new()));
    
    // 每个线程都在写入
    for i in 0..100 {
        let data = Arc::clone(&data);
        thread::spawn(move || {
            for j in 0..1000 {
                let mut map = data.lock().unwrap();
                map.insert(format!("key_{}_{}", i, j), j);
            }
        });
    }
}

// 大量读取操作：使用RwLock
fn heavy_reads() {
    let data = Arc::new(RwLock::new(HashMap::new()));
    
    // 多个读取者，多个写入者
    for _ in 0..10 {
        let data = Arc::clone(&data);
        // 读取线程
        thread::spawn(move || {
            loop {
                let map = data.read().unwrap();
                // 读取操作
                let _size = map.len();
            }
        });
    }
    
    for _ in 0..5 {
        let data = Arc::clone(&data);
        // 写入线程
        thread::spawn(move || {
            loop {
                let mut map = data.write().unwrap();
                // 写入操作
                map.insert("new_key", "new_value");
            }
        });
    }
}

// 无锁操作：使用原子类型
fn lock_free_counter() {
    let counter = Arc::new(AtomicU64::new(0));
    
    let mut handles = vec![];
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("最终计数: {}", counter.load(Ordering::SeqCst));
}
```

### 9.7.2 内存管理优化

#### 1. 减少锁持有时间

```rust
// 不好的做法：长时间持有锁
fn bad_locking_example() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    
    thread::spawn(move || {
        let mut vec = data.lock().unwrap();
        
        // 长时间计算 - 锁被长时间持有
        for i in 0..10_000_000 {
            // 耗时计算
            let _ = (i * i).sqrt();
        }
        
        vec.push(42); // 最终修改数据
    });
}

// 好的做法：快速获取数据，然后释放锁
fn good_locking_example() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    
    thread::spawn(move || {
        // 复制数据，释放锁
        let cloned_data = {
            let vec = data.lock().unwrap();
            vec.clone()
        };
        
        // 长时间计算
        for i in 0..10_000_000 {
            let _ = (i * i).sqrt();
        }
        
        // 快速修改
        let mut vec = data.lock().unwrap();
        *vec = cloned_data;
        vec.push(42);
    });
}
```

#### 2. 使用无锁数据结构

```rust
use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering};
use std::ptr::NonNull;

// 无锁队列实现
struct LockFreeQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

struct Node<T> {
    data: Option<T>,
    next: AtomicPtr<Node<T>>,
}

impl<T> LockFreeQueue<T> {
    fn new() -> Self {
        let dummy = Box::into_raw(Box::new(Node {
            data: None,
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));
        
        Self {
            head: AtomicPtr::new(dummy),
            tail: AtomicPtr::new(dummy),
        }
    }
    
    fn push(&self, item: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data: Some(item),
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));
        
        loop {
            let current_tail = self.tail.load(Ordering::SeqCst);
            let next = unsafe { (*current_tail).next.load(Ordering::SeqCst) };
            
            if next.is_null() {
                if (*current_tail).next.compare_exchange(
                    std::ptr::null_mut(),
                    new_node,
                    Ordering::SeqCst,
                    Ordering::SeqCst
                ).is_ok() {
                    self.tail.store(new_node, Ordering::SeqCst);
                    break;
                }
            } else {
                self.tail.compare_exchange(
                    current_tail,
                    next,
                    Ordering::SeqCst,
                    Ordering::SeqCst
                ).ok();
            }
        }
    }
}
```

### 9.7.3 错误处理与资源管理

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum ConcurrencyError {
    LockPoisoned,
    ThreadPanicked,
    Timeout,
}

// RAII风格的资源管理
struct SharedResource {
    data: Arc<Mutex<Vec<String>>>,
    threads: Vec<thread::JoinHandle<Result<(), ConcurrencyError>>>,
}

impl SharedResource {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Vec::new())),
            threads: Vec::new(),
        }
    }
    
    fn spawn_worker<F>(&mut self, task: F)
    where
        F: FnOnce(Arc<Mutex<Vec<String>>>) -> Result<(), ConcurrencyError> + Send + 'static,
    {
        let data = Arc::clone(&self.data);
        let handle = thread::spawn(move || {
            task(data)
        });
        
        self.threads.push(handle);
    }
    
    fn wait_for_completion(&mut self) -> Result<(), ConcurrencyError> {
        for handle in self.threads.drain(..) {
            match handle.join() {
                Ok(result) => result?,
                Err(_) => return Err(ConcurrencyError::ThreadPanicked),
            }
        }
        Ok(())
    }
    
    fn get_data(&self) -> Vec<String> {
        self.data.lock().unwrap().clone()
    }
}

impl Drop for SharedResource {
    fn drop(&mut self) {
        // 等待所有线程完成
        for handle in &self.threads {
            let _ = handle.join();
        }
    }
}
```

---

## 9.8 测试策略

### 9.8.1 并发测试

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_concurrent_counter() {
        let counter = Arc::new(Mutex::new(0));
        let mut handles = vec![];
        
        // 创建10个线程，每个线程递增1000次
        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    let mut num = counter.lock().unwrap();
                    *num += 1;
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_count = *counter.lock().unwrap();
        assert_eq!(final_count, 10_000);
    }
    
    #[test]
    fn test_producer_consumer() {
        use std::sync::mpsc;
        
        let (tx, rx) = mpsc::channel();
        let tx = mpsc::Sender::clone(&tx);
        
        // 生产者线程
        let producer = thread::spawn(move || {
            for i in 0..10 {
                tx.send(i).unwrap();
            }
        });
        
        // 消费者线程
        let consumer = thread::spawn(move || {
            let mut sum = 0;
            for received in rx.take(10) {
                sum += received;
            }
            sum
        });
        
        producer.join().unwrap();
        let result = consumer.join().unwrap();
        
        assert_eq!(result, 45); // 0+1+2+...+9 = 45
    }
    
    #[test]
    fn test_async_runtime() {
        use tokio;
        
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let start = tokio::time::Instant::now();
            
            let task1 = tokio::spawn(async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                "task1"
            });
            
            let task2 = tokio::spawn(async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                "task2"
            });
            
            let (result1, result2) = tokio::join!(task1, task2);
            
            assert_eq!(result1.unwrap(), "task1");
            assert_eq!(result2.unwrap(), "task2");
            
            // 验证并发执行
            let elapsed = start.elapsed();
            assert!(elapsed < Duration::from_millis(250)); // 应该接近200ms而不是300ms
        });
    }
}
```

### 9.8.2 压力测试

```rust
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

struct StressTest {
    concurrent_threads: usize,
    operations_per_thread: usize,
    shared_resource: Arc<Mutex<Vec<i32>>>,
}

impl StressTest {
    fn new(concurrent_threads: usize, operations_per_thread: usize) -> Self {
        Self {
            concurrent_threads,
            operations_per_thread,
            shared_resource: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut handles = vec![];
        
        for _ in 0..self.concurrent_threads {
            let resource = Arc::clone(&self.shared_resource);
            let handle = thread::spawn(move || {
                for i in 0..self.operations_per_thread {
                    // 模拟读写操作
                    {
                        let mut vec = resource.lock().unwrap();
                        vec.push(i);
                    }
                    
                    thread::sleep(Duration::from_millis(1));
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let duration = start.elapsed();
        let total_operations = self.concurrent_threads * self.operations_per_thread;
        
        TestResult {
            duration,
            total_operations,
            final_size: self.shared_resource.lock().unwrap().len(),
        }
    }
}

#[derive(Debug)]
struct TestResult {
    duration: Duration,
    total_operations: usize,
    final_size: usize,
}

impl TestResult {
    fn throughput(&self) -> f64 {
        self.total_operations as f64 / self.duration.as_secs_f64()
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;
    
    #[test]
    fn test_concurrent_stress() {
        let test = StressTest::new(10, 1000);
        let result = test.run();
        
        println!("压力测试结果:");
        println!("  总操作数: {}", result.total_operations);
        println!("  执行时间: {:?}", result.duration);
        println!("  吞吐量: {:.2} ops/sec", result.throughput());
        println!("  最终数据大小: {}", result.final_size);
        
        // 验证数据完整性
        assert_eq!(result.final_size, result.total_operations);
    }
}
```

---

## 9.9 性能调优

### 9.9.1 Profiling与监控

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

struct PerformanceMetrics {
    total_requests: AtomicU64,
    total_latency: AtomicU64,
    active_connections: AtomicUsize,
    peak_connections: AtomicUsize,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_latency: AtomicU64::new(0),
            active_connections: AtomicUsize::new(0),
            peak_connections: AtomicUsize::new(0),
        }
    }
    
    fn record_request(&self, latency: Duration) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(
            latency.as_micros() as u64, 
            Ordering::Relaxed
        );
    }
    
    fn connection_opened(&self) {
        let active = self.active_connections.fetch_add(1, Ordering::Relaxed) + 1;
        let peak = self.peak_connections.load(Ordering::Relaxed);
        if active > peak {
            self.peak_connections.store(active, Ordering::Relaxed);
        }
    }
    
    fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
    
    fn get_stats(&self) -> ServerStats {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let total_latency = self.total_latency.load(Ordering::Relaxed);
        let active_connections = self.active_connections.load(Ordering::Relaxed);
        let peak_connections = self.peak_connections.load(Ordering::Relaxed);
        
        let avg_latency = if total_requests > 0 {
            Duration::from_micros(total_latency / total_requests)
        } else {
            Duration::from_micros(0)
        };
        
        ServerStats {
            total_requests,
            active_connections,
            peak_connections,
            average_latency: avg_latency,
        }
    }
}

#[derive(Debug, Clone)]
struct ServerStats {
    total_requests: u64,
    active_connections: usize,
    peak_connections: usize,
    average_latency: Duration,
}

// 在服务器中使用
pub struct HttpServerWithMetrics {
    metrics: Arc<PerformanceMetrics>,
    // 其他字段...
}

impl HttpServerWithMetrics {
    pub fn new(port: u16) -> Self {
        Self {
            metrics: Arc::new(PerformanceMetrics::new()),
        }
    }
    
    pub async fn handle_request(&self) {
        let start = Instant::now();
        self.metrics.connection_opened();
        
        // 处理请求...
        
        let duration = start.elapsed();
        self.metrics.record_request(duration);
        self.metrics.connection_closed();
    }
    
    pub fn get_performance_stats(&self) -> ServerStats {
        self.metrics.get_stats()
    }
}
```

### 9.9.2 内存优化

```rust
// 减少内存分配的技巧

// 1. 使用对象池
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

struct ObjectPool<T: Default> {
    pool: Mutex<VecDeque<T>>,
    capacity: usize,
}

impl<T: Default> ObjectPool<T> {
    fn new(capacity: usize) -> Self {
        Self {
            pool: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }
    
    fn get(&self) -> T {
        let mut pool = self.pool.lock().unwrap();
        pool.pop_front().unwrap_or_default()
    }
    
    fn return_object(&self, item: T) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.capacity {
            pool.push_back(item);
        }
    }
}

// 2. 使用栈分配的缓冲区
fn stack_allocated_buffer() {
    // 使用固定大小的数组而不是Vec
    let mut buffer = [0u8; 4096]; // 4KB 栈缓冲区
    
    // 填充数据
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    
    println!("栈分配的缓冲区大小: {} 字节", buffer.len());
}

// 3. 避免不必要的clone
use std::sync::Arc;

fn avoid_unnecessary_clones(data: &str) -> String {
    // 不好的做法
    // let owned = data.to_string(); // 不必要的分配
    
    // 好的做法
    if data.is_empty() {
        String::new()
    } else {
        data.to_string() // 只在需要时分配
    }
}

// 4. 使用Cow (Clone-on-Write)
use std::borrow::Cow;

fn use_cow_optimization(input: &str) -> Cow<str> {
    if input.is_empty() {
        Cow::Owned("default".to_string())
    } else if input.len() > 10 {
        Cow::Owned(input.to_uppercase())
    } else {
        Cow::Borrowed(input)
    }
}
```

---

## 9.10 总结与进阶

### 9.10.1 本章要点回顾

1. **并发基础概念**：
   - 理解并发与并行的区别
   - 掌握Rust的并发安全哲学
   - 学会所有权和借用在并发中的作用

2. **线程编程**：
   - 掌握线程创建和管理的各种方式
   - 实现线程池和任务调度
   - 理解线程间参数传递和返回值

3. **消息传递**：
   - 使用通道进行线程间通信
   - 实现生产者和消费者模式
   - 掌握同步和异步通道的区别

4. **共享内存**：
   - 使用Arc和Mutex进行安全共享
   - 掌握RwLock和Condvar的使用
   - 了解无锁数据结构的概念

5. **异步编程**：
   - 理解async/await的语法和语义
   - 掌握Tokio异步运行时的使用
   - 学会使用select和流处理

6. **实战项目**：
   - 构建了高并发Web服务器
   - 实现了性能监控和指标收集
   - 掌握了并发架构设计模式

### 9.10.2 最佳实践总结

1. **选择合适的并发模型**：
   - I/O密集型：使用异步编程
   - CPU密集型：使用多线程和Rayon
   - 混合场景：结合两种方法

2. **同步原语选择指南**：
   - 大量写入：使用Mutex
   - 大量读取：使用RwLock
   - 简单计数器：使用原子类型
   - 复杂通信：使用通道

3. **性能优化要点**：
   - 减少锁持有时间
   - 避免死锁和活锁
   - 使用对象池减少分配
   - 监控和调优

4. **错误处理策略**：
   - 使用Result类型传播错误
   - 实现RAII资源管理
   - 处理线程panic

### 9.10.3 进阶学习方向

1. **高级并发模式**：
   - Actor模型实现
   - 分布式系统设计
   - 实时系统编程

2. **性能调优技术**：
   - CPU缓存优化
   - 内存布局优化
   - SIMD指令使用

3. **系统级编程**：
   - 零拷贝I/O
   - 内存映射文件
   - 信号处理

4. **分布式系统**：
   - 集群协调
   - 分布式缓存
   - 容错和恢复

通过本章的学习，您已经掌握了Rust强大的并发编程能力，能够构建高性能、可扩展的并发应用程序。下一章将介绍网络编程，进一步扩展您的系统编程技能。
