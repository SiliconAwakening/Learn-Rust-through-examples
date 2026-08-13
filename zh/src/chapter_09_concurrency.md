# 第9章：并发编程

Rust “无畏并发”（fearless concurrency）的承诺建立在一个事实上：防止内存错误的同一套所有权与借用规则，也在**编译期**防止了数据竞争。如果两个线程共享数据，编译器会坚持要求这种共享是安全的——要么只读，要么有同步原语守护。本章覆盖两种主流模型——**带共享状态的线程**与**消息传递**——以及让它们安全的 `Send`/`Sync` trait。

## 学习目标

- 用 `thread::spawn` 创建线程并用 `join` 等待。
- 用 `Arc`、`Mutex`、`RwLock` 安全地共享数据。
- 用 `mpsc` 通道在线程间通信。
- 理解 `Send` 与 `Sync` 及其意义。
- 避开共享状态并发的常见死锁与陷阱。

---

## 9.1 并发与并行

先厘清两个常被混用的词：

- **并发（Concurrency）**：多个任务在重叠的时间段内推进，由调度器在它们之间切换。强调“同时处理多件事”。
- **并行（Parallelism）**：多个任务真正在同一时刻执行（在多个 CPU 核上）。强调“同时做多件事”。

并发是结构，并行是执行。Rust 的所有权规则对两者都提供安全保证。

---

## 9.2 线程

`std::thread::spawn` 启动一个 OS 线程，返回 `JoinHandle`。调用 `.join()` 等待它结束：

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        for i in 0..5 {
            println!("子线程说 {i}");
        }
    });

    for i in 0..3 {
        println!("主线程说 {i}");
    }

    handle.join().unwrap(); // 等待子线程
}
```

### 闭包与 `move`

派生线程不能借用局部变量——除非这些变量能活得够久，而 `main` 可能在子线程结束前就返回了。解法是 `move`，它把捕获的变量所有权转移进闭包：

```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3];

    let handle = thread::spawn(move || {
        // data 现在归这个线程所有
        println!("拿到 {} 项", data.len());
    });

    handle.join().unwrap();
    // println!("{:?}", data); // 错误：data 已被 move 进线程
}
```

---

## 9.3 `Send` 与 `Sync`

这两个标记 trait 是线程安全的基石。你不必自己实现它们；当所有字段都是 `Send`/`Sync` 时，编译器会自动派生。

- **`Send`**：类型 `T` 是 `Send`，表示把 `T` **移动**到另一个线程是安全的。
- **`Sync`**：类型 `T` 是 `Sync`，表示多个线程同时持有 `&T` 是安全的（即 `&T` 是 `Send` 的）。

绝大多数类型两者皆是。例外是“无同步的内可变性”类型——`Rc<T>` 是经典反例：它的引用计数不是原子的，因此**不是** `Send`/`Sync`。跨线程共享要用 `Arc<T>`（原子引用计数）。

---

## 9.4 共享状态：`Arc` + `Mutex`

要在线程间共享**可变**数据，组合使用：

- **`Arc<T>`**：原子引用计数指针，多个线程可共享同一份分配。
- **`Mutex<T>`**：锁，保证对内部值的独占访问。

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // 受锁保护的共享计数器，经 Arc 共享
    let counter = Arc::new(Mutex::new(0));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                let mut num = counter.lock().unwrap();
                *num += 1;
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    println!("最终 = {}", *counter.lock().unwrap()); // 10
}
```

`.lock().unwrap()` 需要解释：持有锁的线程若 panic，`Mutex` 会被**中毒**（poisoned），此后 `.lock()` 返回 `Err`。调用 `.unwrap()` 会让 panic 继续传播，这通常是对的——中毒的锁意味着数据可能处于不一致状态。

### `RwLock`：多读单写

读远多于写时，`RwLock` 允许多个读者同时持锁：

```rust
use std::sync::RwLock;

let cache = RwLock::new(0);

{
    let r1 = cache.read().unwrap();
    let r2 = cache.read().unwrap(); // 多个读锁可并存
    println!("读: {} {}", *r1, *r2);
}
{
    let mut w = cache.write().unwrap(); // 写锁独占
    *w += 1;
}
```

---

## 9.5 消息传递：通道

另一种并发模型是“通过通信来共享内存”，而非共享内存。Rust 的 `std::sync::mpsc`（多生产者、单消费者）通道正是为此而生。

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    let sender = thread::spawn(move || {
        let msgs = ["hi", "from", "the", "thread"];
        for m in msgs {
            tx.send(m).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    });

    // rx 是迭代器：持续产出收到的值，直到发送端被丢弃
    for received in rx {
        println!("收到: {received}");
    }

    sender.join().unwrap();
}
```

- `send` 返回 `Result`，因为接收端可能已被丢弃。
- 多生产者：用 `tx.clone()` 复制发送端，分别移进不同线程。

通道解耦了线程：发送方不必知道谁在读，加锁逻辑藏在通道实现里。

---

## 9.6 一个小型工作池

把组件拼起来——一组 worker 线程从共享队列消费任务：

```rust
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send>>();
    let rx = Arc::new(Mutex::new(rx));

    // 起四个 worker，共享接收端
    let mut workers = Vec::new();
    for _ in 0..4 {
        let rx = Arc::clone(&rx);
        workers.push(thread::spawn(move || loop {
            let job = {
                let lock = rx.lock().unwrap();
                lock.recv()
            };
            match job {
                Ok(task) => task(),
                Err(_) => break, // 所有发送端已丢弃——退出
            }
        }));
    }

    // 投递几个任务
    for i in 0..8 {
        tx.send(Box::new(move || {
            println!("任务 {i} 跑在 {:?}", thread::current().id());
        }))
        .unwrap();
    }

    drop(tx); // 关闭通道，让 worker 能退出
    for w in workers {
        w.join().unwrap();
    }
}
```

接收端外面包 `Mutex` 是必要的——`mpsc::Receiver` 不是 `Sync`，同一时刻只能有一个线程调用 `recv`。

---

## 9.7 异步并发

`async`/`await` 是另一种并发模型：用 `.await` 点替代 OS 线程切换，单线程即可管理成千上万个并发任务，非常适合 I/O 密集型场景。其错误处理与所有权规则与本章一致，只是 `Future` 在 `.await` 点之间被挂起。

> **跨线程共享 `Future` 的注意点**：不要让 `std::sync::Mutex` 的守卫跨越 `.await`——它不是为异步设计的。在异步代码里用 `tokio::sync::Mutex`，或把守卫的作用域限制在 `.await` 之前。

异步网络的实战在第 10、12 章展开。

---

## 9.8 陷阱

1. **死锁**：不同线程以不同顺序获取两把锁会死锁。按全局一致顺序加锁，或用一把锁守护两份资源。
2. **跨 `.await` 持有 `std::sync::Mutex`**：见上节，改用 `tokio::sync::Mutex` 或缩小作用域。
3. **跨线程用 `Rc`**：编译器会拒绝（`Rc` 不是 `Send`），改用 `Arc`。
4. **忘记 `join`**：分离的线程可能比它引用的数据活得久——不过 Rust 强制 `move` 或 `'static` 借用，这在编译期就被捕获。
5. **锁太多**：每个操作都取全局锁，等于串行化了。改用更细粒度的锁、分片数据或通道。

---

## 9.9 小结

Rust 让数据竞争在构造上不可能：共享可变状态需要 `Mutex`，跨线程引用计数需要 `Arc`，`Send`/`Sync` 在编译期检查。线程间通信用通道，共享可变数据用 `Arc<Mutex<T>>`（或 `RwLock`）。借用检查器把其他语言里难以复现的并发 bug，在这里变成了编译错误。

### 练习

1. 起 10 个线程，各对一个共享的 `Arc<Mutex<i32>>` 自增 1000 次，打印最终值。
2. 用通道重做上题：每个线程把自己的增量发给接收端汇总。
3. 用两个通道搭一条流水线：一线程产生数字、一线程平方、一线程打印。
