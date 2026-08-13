# 第15章：测试与调试

Rust 的测试能力内建于语言：编译器认识 `#[test]`，标准库自带断言，`cargo test` 一条命令跑完一切。本章覆盖单元测试、集成测试与文档测试，再讲到编译器查不出的那些 bug 的工具——基于性质的测试、结构化日志与调试器。

## 学习目标

- 编写单元测试、集成测试与文档测试。
- 组织测试模块并使用常用断言宏。
- 测试异步代码与外部依赖。
- 用基于性质的测试生成随机输入。
- 用 `tracing` 与 `lldb` 调试。

**实战项目**：构建一个自动化测试系统，覆盖多层级测试、性能监控与报告生成。

---

## 15.1 单元测试

单元测试紧挨着被测代码，放在 `#[cfg(test)]` 模块里，只在 `cargo test` 时编译：

```rust
// src/math.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_works() {
        assert_eq!(add(2, 2), 4);
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn overflow_panics() {
        // 演示断言会按预期 panic
        panic!("overflow");
    }
}
```

核心断言：

| 宏 | 检查 |
|----|------|
| `assert!(cond)` | 条件为真 |
| `assert_eq!(a, b)` | 两值相等 |
| `assert_ne!(a, b)` | 两值不等 |
| `should_panic` | 测试会 panic（可带消息） |

测试要小、聚焦、独立——每个测一种行为。

---

## 15.2 集成测试

集成测试放在 `tests/` 目录，像外部用户一样操作 crate 的公开 API。每个文件是独立二进制：

```rust
// tests/api.rs
use my_crate::add;

#[test]
fn add_from_outside() {
    assert_eq!(add(3, 4), 7);
}
```

端到端路径用集成测试，内部分支用单元测试。

---

## 15.3 文档测试

`///` 文档注释里的代码块会被 `cargo test` 编译并运行。它既是示例，又是“文档里写的 API 确实能用”的正确性检查：

```rust
/// 把两个整数相加。
///
/// ```
/// use my_crate::add;
/// assert_eq!(add(2, 2), 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

示例不该运行就标 ```` ```no_run ```` 或 ```` ```ignore ````。文档测试让你的文档保持诚实。

---

## 15.4 测试异步代码

`tokio` 提供 `#[tokio::test]` 属性，把测试包进运行时：

```rust
#[tokio::test]
async fn fetches_a_value() {
    let result = some_async_fn().await;
    assert_eq!(result, 42);
}
```

带定时器的代码，用 `tokio::time::pause` 与 `advance` 让测试无需真实延时即可确定性推进。

---

## 15.5 依赖注入与“模拟”

Rust 没有内置 mock 框架，这是有意为之——地道的做法是用 trait 做依赖注入。为外部依赖定义一个小 trait，测试里写假实现，传进去：

```rust
pub trait Clock {
    fn now(&self) -> u64;
}

pub fn greet(name: &str, clock: &dyn Clock) -> String {
    let hour = (clock.now() / 3600) % 24;
    if hour < 12 { format!("早上好, {name}") }
    else { format!("你好, {name}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedClock(u64);
    impl Clock for FixedClock {
        fn now(&self) -> u64 { self.0 }
    }

    #[test]
    fn morning() {
        assert_eq!(greet("alice", &FixedClock(7 * 3600)), "早上好, alice");
    }
}
```

更重的 mock，用 `mockall` 自动生成 trait 的 mock 实现。

---

## 15.6 基于性质的测试

与其一次写一个示例，不如陈述一个应始终成立的**性质**，让框架搜索反例。`proptest` 是标准 crate：

```toml
[dev-dependencies]
proptest = "1"
```

```rust
proptest::proptest! {
    #[test]
    fn add_is_commutative(a in -1000i32..1000, b in -1000i32..1000) {
        proptest::prop_assert_eq!(add(a, b), add(b, a));
    }

    #[test]
    fn sort_is_idempotent(mut v in proptest::collection::vec(-100i32..100, 0..100)) {
        v.sort();
        let mut w = v.clone();
        w.sort();
        proptest::prop_assert_eq!(v, w);
    }
}
```

性质测试能找到你不会想到的边界——空输入、最大值、差一——并把失败的随机用例**收缩**到最小复现。

---

## 15.7 用 `tracing` 调试

`println!` 能用，但 `tracing` 给你结构化、分级、带上下文的日志，跨异步任务也能用：

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

```rust
use tracing::{info, instrument, span, Level};

#[instrument]
fn process(user: &str) {
    let _span = span!(Level::INFO, "step", user = %user).entered();
    info!("开始处理");
    // ...
}

fn main() {
    tracing_subscriber::fmt::init();
    process("alice");
}
```

span 给其下每条日志附加上下文（函数名、参数），这在大量请求交错时极有价值。

---

## 15.8 调试器

日志不够时，用 `lldb`（或 `gdb`、IDE 调试器）配合调试构建：

```bash
cargo build
lldb -- target/debug/myapp
```

用 `b 函数名` 设断点，`n`/`s` 单步，`p 变量` 检查。对 panic，用 `RUST_BACKTRACE=1` 无需调试器就拿到栈回溯：

```bash
RUST_BACKTRACE=1 cargo run
```

---

## 15.9 最佳实践

1. **测行为，不测实现。** 钻进私有内部的测试每次重构都会碎。
2. **一个测试一个断言**（尽量）。窄测试能定位失败。
3. **保持快速测试快。** 把慢的集成测试藏在 feature flag 后，让 `cargo test` 保持利落。
4. **先写失败的测试。** 它先确认 bug 存在，再修。
5. **纯函数用性质测试。** 那是 `proptest` 的用武之地。

---

## 15.10 小结

`cargo test` 跑 `#[cfg(test)]` 模块里的单元测试、`tests/` 里的集成测试、`///` 注释里的文档测试——一条命令，三种覆盖。用 trait 注入依赖来隔离测试，用 `proptest` 猎杀边界，行为出错时上 `tracing` 与 `lldb`。Rust 的测试无聊得恰到好处：它就是代码，由同一套工具链编译运行。

### 练习

1. 给一个 `sort` 包装函数加单元测试与文档测试，确认都在 `cargo test` 下运行。
2. 用 `proptest` 验证：对一个 `Vec` 反转两次得到原值。
3. 给函数加 `tracing` span，用 `tracing_subscriber::fmt` 检查输出。
