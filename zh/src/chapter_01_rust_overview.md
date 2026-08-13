# 第1章：Rust 概述与环境搭建

Rust 是一门追求“安全与性能兼得”的系统编程语言：没有垃圾回收，却能在编译期杜绝整类内存错误。本章带你了解 Rust 为何存在、它的核心特性，以及如何搭建开发环境并跑通第一个程序。

## 学习目标

- 理解 Rust 的设计理念与核心特性。
- 用 `rustup` 安装并管理 Rust 工具链。
- 用 `cargo` 创建、构建、运行项目。
- 了解编译与发布构建的区别。

---

## 1.1 为什么是 Rust

Rust 诞生于 Mozilla（2006 年起步，2010 年公开），目标是同时拥有 C++ 的性能与控制力，又不再被内存 bug 折磨。它的三大支柱是：

- **内存安全**：所有权（ownership）、借用（borrowing）、生命周期在编译期检查，杜绝空指针、悬垂引用、缓冲区溢出、数据竞争——无需垃圾回收，也无需手动 `free`。
- **零成本抽象**：高层抽象（迭代器、泛型、trait）编译后与手写底层代码一样快。
- **无畏并发**：同一套所有权规则也在编译期防止数据竞争，让你放心写多线程代码。

代价是学习曲线——借用检查器起初会“拒绝”你写的代码，但它拒绝的正是真实 bug。掌握之后，这套约束会变成可靠的重构保障。

---

## 1.2 安装工具链

Rust 官方用 `rustup` 管理工具链。在 macOS/Linux 上：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Windows 用户下载 `rustup-init.exe` 即可。安装后重启 shell，验证：

```bash
rustc --version
cargo --version
```

`rustup` 让你轻松切换工具链、添加交叉编译目标、安装组件（如 `rustfmt`、`clippy`）：

```bash
rustup update              # 更新到最新稳定版
rustup component add clippy rustfmt
rustup target add wasm32-unknown-unknown   # 添加 WebAssembly 目标
```

> **提示**：日常开发用 `stable` 通道即可。想尝鲜新特性可用 `nightly`，但不要在生产依赖它。

---

## 1.3 第一个程序：Hello, Cargo

`cargo` 是 Rust 的构建工具与包管理器，几乎所有 Rust 项目都从它开始：

```bash
cargo new hello_rust
cd hello_rust
```

生成的目录结构：

```
hello_rust/
├── Cargo.toml    # 项目清单（依赖、元数据）
└── src/
    └── main.rs   # 源码入口
```

`src/main.rs` 默认内容：

```rust
fn main() {
    println!("Hello, world!");
}
```

构建并运行：

```bash
cargo run
# 输出：Hello, world!
```

`cargo run` 会先编译再运行。`cargo build` 只编译不运行；`cargo check` 只做类型检查不生成二进制——开发时反馈最快的命令。

---

## 1.4 Cargo 基础

`Cargo.toml` 是项目的清单：

```toml
[package]
name = "hello_rust"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
```

- `edition`：语言版本（2015/2018/2021）。新项目用 `2021`。
- `[dependencies]`：声明依赖。`cargo` 会从 [crates.io](https://crates.io) 拉取并写进 `Cargo.lock` 锁定版本。

常用命令一览：

| 命令 | 作用 |
|------|------|
| `cargo new <name>` | 新建二进制项目 |
| `cargo new --lib <name>` | 新建库项目 |
| `cargo build` | 编译（调试构建） |
| `cargo build --release` | 优化构建，用于发布/基准测试 |
| `cargo run` | 编译并运行 |
| `cargo check` | 仅类型检查（最快） |
| `cargo test` | 运行所有测试 |
| `cargo fmt` | 格式化代码 |
| `cargo clippy` | 运行 lint |
| `cargo doc --open` | 生成并打开文档 |

> **调试 vs 发布**：默认 `cargo build` 是调试构建（`opt-level = 0`，编译快、含调试信息）。基准测试或部署必须用 `--release`，否则结果没有代表性。

---

## 1.5 一个稍大的例子

用一个函数体会 Rust 的风格——显式类型、表达式语义、零成本抽象：

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];

    // 用迭代器组合子：过滤偶数、翻倍、求和
    let result: i32 = numbers
        .iter()
        .filter(|&&n| n % 2 == 0)
        .map(|&n| n * 2)
        .sum();

    println!("偶数翻倍之和 = {result}"); // 4 + 8 + 12 = 24
}
```

这段代码读起来像数学公式，编译后却与手写循环一样快。这就是“零成本抽象”的直观体现——后面章节会拆解每个机制。

---

## 1.6 工具链与生态一览

- **rust-analyzer**：IDE 后端，给 VS Code / Vim / Emacs 提供补全、跳转、内联类型提示。装上它，Rust 的开发体验会有质变。
- **rustfmt**：官方格式化器，消除代码风格争论。
- **clippy**：lint 工具，捕捉一长串常见错误与不地道写法。
- **crates.io**：包仓库。`cargo add <crate>` 即可引入依赖。
- **docs.rs**：每个发布到 crates.io 的 crate 都有自动生成的文档。

---

## 1.7 小结

Rust 用所有权在编译期保证内存安全与并发安全，用零成本抽象让高层代码不牺牲性能。`rustup` 管理工具链，`cargo` 管理项目与依赖，`cargo check`/`run`/`test` 是日常三件套。装好 `rust-analyzer`、`rustfmt`、`clippy`，你就有了顺手的开发环境。

### 练习

1. 用 `cargo new` 创建一个项目，写一个函数返回斐波那契数列前 N 项，用 `cargo run` 与 `cargo test` 验证。
2. 给项目加一个依赖（如 `rand`），用 `cargo doc --open` 查看生成的文档。
3. 故意写一段会被 `clippy` 警告的代码（如多余的 `return`），运行 `cargo clippy` 并修正。
