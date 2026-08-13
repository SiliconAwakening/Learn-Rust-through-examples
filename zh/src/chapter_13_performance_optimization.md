# 第13章：性能优化

优化的第一条法则是：**先测量，再优化。** Rust 默认就很快，所以多数“优化”其实是别把这份速度挥霍掉——避免无谓的分配、为缓存布局数据、选择合适的并发模型。本章既是寻找瓶颈的工具箱，也是一张“哪些技术真正管用”的清单。

## 学习目标

- 在改动代码前用基准测试建立性能基线。
- 用 CPU 与内存剖析找到真实瓶颈，而非凭猜测。
- 减少分配与拷贝——Rust 中最常见的性能红利。
- 为缓存局部性布局数据。
- 根据工作负载在线程与异步之间做选择。

**实战项目**：构建一个高性能缓存服务，覆盖内存池、性能监控与故障恢复。

---

## 13.1 先测量

别凭直觉优化。用 [`criterion`](https://docs.rs/criterion) 建立基线，它跑统计基准测试并报告噪声：

```toml
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "string_join"
harness = false
```

```rust
// benches/string_join.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_join(c: &mut Criterion) {
    let words: Vec<String> = (0..1000).map(|i| i.to_string()).collect();

    c.bench_function("join_with_plus", |b| {
        b.iter(|| {
            let mut s = String::new();
            for w in &words { s += w; }
            s
        })
    });

    c.bench_function("join_with_iter", |b| {
        b.iter(|| words.join(""))
    });
}

criterion_group!(benches, bench_join);
criterion_main!(benches);
```

`cargo bench` 运行，criterion 告诉你改动是真提升还是落在噪声内。

---

## 13.2 剖析

对整个程序，用 `perf`（Linux）、`Instruments`（macOS）或 `cargo flamegraph` 剖析：

```bash
cargo install flamegraph
cargo flamegraph --bin myapp
```

火焰图按调用树聚合展示 CPU 时间花在哪。留意意外的热点——你没料到的 `clone`、紧凑循环里的 `format!`、占比惊人的哈希函数。

---

## 13.3 分配是通常的嫌犯

堆分配很便宜，但每秒成千上万次就累积成成本。最大的红利通常来自减少分配：

```rust
// 差：每次调用都分配新 String
fn bad(items: &[i32]) -> String {
    let mut s = String::new();
    for x in items { s += &x.to_string(); }
    s
}

// 好：一次分配，预先留好容量
fn good(items: &[i32]) -> String {
    // 每个 i32 最多 11 个字符；预分配避免反复扩容
    let mut s = String::with_capacity(items.len() * 11);
    for x in items { s.push_str(&x.to_string()); }
    s
}
```

值得质疑的常见分配模式：

- 循环里的 `clone()`——能不能改借用？
- 为了和 `&str` 比较而 `to_string()`——直接用 `==` 比较。
- 通常被关掉的日志仍用 `format!`——改用 `log`/`tracing` 宏，级别关闭时跳过格式化。
- 收集进 `Vec` 只为迭代一次——保持迭代器惰性。

---

## 13.4 字符串处理

`String` 是堆分配的可增长字符串；`&str` 是借用的切片。函数参数优先用 `&str`。必须构建字符串时，用 `String::with_capacity` 或 `write!` 写入 `String`：

```rust
use std::fmt::Write;

let mut out = String::with_capacity(64);
write!(out, "x={}, y={}", 10, 20).unwrap();
```

对 ASCII 标识符，`CompactString` 或内联的 `&'static str` 可避免每次调用分配。

---

## 13.5 缓存局部性与数据布局

现代 CPU 算术快、访存慢。连续且顺序访问的数据远快于指针跳转。这就是 `Vec` 几乎总胜过 `LinkedList` 的原因，也是“结构体数组”在只迭代某字段时常胜过“数组结构体”的原因：

```rust
// 数组结构体——自然，但每项要碰三条缓存行
struct Particle { x: f64, y: f64, v: f64 }
let aos: Vec<Particle> = /* ... */;

// 结构体数组——迭代 xs 时流式扫一块连续内存
struct Particles { xs: Vec<f64>, ys: Vec<f64>, vs: Vec<f64> }
```

如果剖析发现你花时间加载了从没用到的数据，把数据重构成结构体数组（或把大结构体拆成“热/冷”两部分）常常是 2–10 倍的收益。

---

## 13.6 哈希

默认 `HashMap` 用 SipHash，抗 DoS 但较慢。对可信、非对抗性键，`ahash` 或 `rustc-hash`（FNV 风格）快好几倍：

```toml
[dependencies]
ahash = "0.8"
```

```rust
use ahash::AHashMap;
let mut m: AHashMap<&str, i32> = AHashMap::new();
```

---

## 13.7 内联与泛型

Rust 的泛型函数是**单态化**的——编译器为每个具体类型生成一份副本，从而能内联，通常比动态分发快。热路径里优先用泛型而非 `dyn Trait`：

```rust
// 泛型——单态化、可内联、快
fn sum<T: Copy + std::ops::Add<Output = T>>(xs: &[T], zero: T) -> T {
    xs.iter().fold(zero, |a, &b| a + b)
}
```

`#[inline]` 要节制——编译器自己很在行；只对跨 crate 边界的小叶子函数留提示。

---

## 13.8 异步 vs 线程

- **CPU 密集：** 用线程或 `rayon` 的数据并行。`rayon` 把 `iter()` 变 `par_iter()`：

  ```rust
  use rayon::prelude::*;
  let total: u64 = (0..1_000_000).into_par_iter().map(|i| i * i).sum();
  ```

- **I/O 密集：** 用异步。每连接派生一个任务远比派生线程便宜。
- **混合：** 把阻塞型 CPU 工作挪出异步运行时，用 `tokio::task::spawn_blocking`，别让它卡住反应器。

---

## 13.9 最佳实践

1. **改前改后都测。** 没有测量的改动只是猜测。
2. **剖析整个程序**，而非微基准——当你关心端到端速度时。
3. **先砍分配。** 这是地道 Rust 里最容易的大红利。
4. **尊重缓存。** 连续、顺序、可预测的访问胜出。
5. **别和优化器较劲。** 写清晰、单态化的代码；`cargo build --release` 负责其余。

---

## 13.10 小结

Rust 的性能工作从测量开始：`criterion` 做微基准，`flamegraph` 看全程序。常见红利是更少分配、更好的缓存布局、合适的并发模型——CPU 用线程与 `rayon`，I/O 用异步。多数 Rust 代码已经很快；这些技术让它随规模增长依然快。

### 练习

1. 对 `Vec::push` 有无 `with_capacity` 做基准测试，报告差异。
2. 把一个数组结构体示例改成结构体数组，对单字段求和做基准对比。
3. 在热循环里把 `HashMap` 换成 `AHashMap`，测量变化。
