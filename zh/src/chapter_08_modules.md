# 第8章：模块系统与工程化

当程序规模超过一屏，组织方式就和正确性同样重要。Rust 的模块系统控制着**可见性**、解析**路径**，并把代码拆分到 **crate** 与 **workspace** 之中。本章讲清楚如何组织一个项目，使其从几百行平稳扩展到大型代码库而不致沦为乱麻。

## 学习目标

- 用 `mod` 声明模块与子模块。
- 用 `pub` 与 `pub(crate)` 精确控制可见性。
- 用 `use` 把条目引入作用域，包括别名与再导出。
- 按 Rust 的路径约定把一个 crate 拆分到多个文件。
- 用 Cargo workspace 组织多 crate 项目。

---

## 8.1 模块基础

模块把相关条目分组并给它们一个命名空间。用 `mod` 声明：

```rust
mod network {
    pub fn connect(host: &str) {
        println!("connecting to {host}");
        configure();
    }

    fn configure() {
        // 私有——只在 network 内部可见
        println!("configuring socket");
    }
}

fn main() {
    network::connect("example.com");
    // network::configure(); // 错误：configure 是私有的
}
```

条目**默认私有**。`pub` 让它对模块外可见。这个默认与许多语言相反，是有意为之的安全特性：你必须显式选择暴露 API。

---

## 8.2 路径与 `use`

引用条目时用路径限定：`crate::network::connect`，或从其他模块 `network::connect`。`use` 声明用来缩短它：

```rust
mod network {
    pub mod tcp {
        pub fn listen(port: u16) {
            println!("listening on {port}");
        }
    }
}

use network::tcp::listen; // 把 listen 引入作用域

fn main() {
    listen(8080); // 无需限定
}
```

两个常用的 `use` 形式：

```rust
// 从同一模块批量引入
use std::io::{self, Read, Write};

// 再导出，让调用方看到更短的路径
pub use network::tcp::listen as tcp_listen;
```

两个导入同名时，给其中一个起别名：`use std::fmt::Result as FmtResult;`。

---

## 8.3 把代码拆分到文件

Rust 允许把模块体放到另一个文件里。约定如下：

```
src/
├── main.rs
├── network.rs        // main.rs 里 `mod network` 对应的内容
└── network/
    └── tcp.rs        // network.rs 里 `mod tcp` 对应的内容
```

在 `main.rs` 里声明模块（不带体），Rust 会找到对应文件：

```rust
// src/main.rs
mod network;

fn main() {
    network::tcp::listen(8080);
}
```

```rust
// src/network.rs
pub mod tcp; // Rust 查找 src/network/tcp.rs
```

```rust
// src/network/tcp.rs
pub fn listen(port: u16) {
    println!("listening on {port}");
}
```

规则：不带体的 `mod foo;` 告诉 Rust 去找 `foo.rs` 或 `foo/mod.rs`。子模块在对应父模块的文件里声明。

---

## 8.4 可见性进阶

可见性控制的是**谁能命名**一个条目。

| 可见性 | 可访问范围 |
|--------|-----------|
|（默认）私有 | 仅当前模块及其后代 |
| `pub` | 任何能命名它的模块 |
| `pub(crate)` | 当前 crate 内任意位置 |
| `pub(super)` | 父模块 |
| `pub(in path)` | 指定的祖先模块 |

`pub(crate)` 是库内部多个模块共享、但不想暴露给 crate 用户的利器：

```rust
pub(crate) fn internal_cache_key(s: &str) -> String {
    format!("cache:{s}")
}
```

一个微妙的规则：把结构体设为 `pub` 并**不会**让其字段公开。每个字段要单独标 `pub`：

```rust
pub struct User {
    pub name: String,    // 公开
    created_at: u64,     // 私有——调用方既不能读也不能写
}
```

---

## 8.5 Crate 与包

**crate** 是编译单元。**包**（package）是带 `Cargo.toml`、包含一个或多个 crate 的目录。二进制 crate 有 `main` 函数；库 crate 没有。

一个同时交付库与二进制的常见布局：

```
src/
├── lib.rs     // 库 crate 根
├── main.rs    // 二进制 crate 根——使用这个库
└── ...
```

```rust
// src/lib.rs
pub fn greet(name: &str) {
    println!("hello, {name}");
}
```

```rust
// src/main.rs
use my_crate::greet; // 二进制依赖自己的库

fn main() {
    greet("world");
}
```

把真实逻辑放进库、让 `main.rs` 保持纤薄，代码就可测——测试能直接链接库。

---

## 8.6 工作空间（Workspace）

当一个项目包含多个一起演进的 crate，**workspace** 共享一个 `target/` 目录与一份 `Cargo.lock`：

```toml
# 工作空间根的 Cargo.toml
[workspace]
members = ["core", "cli", "server"]
```

每个成员是独立 crate，有自己的 `Cargo.toml`，彼此按路径依赖：

```toml
# cli/Cargo.toml
[dependencies]
core = { path = "../core" }
```

工作空间让构建时间可控（一个 `target/`），又能一起版本化与测试，同时保持边界清晰。

---

## 8.7 标准库 prelude

有些条目无需 `use` 就始终在作用域里——`Vec`、`String`、`Option`、`Result`、`println!`。这就是 **prelude**：标准库再导出的一小撮最常用类型。你永远不必导入它们。

---

## 8.8 实战：把单文件程序拆成模块

把一个包含“解析、处理、输出”三件事的单文件程序拆成三个模块文件：

```
src/
├── main.rs
├── parser.rs
├── processor.rs
└── output.rs
```

```rust
// src/main.rs
mod parser;
mod processor;
mod output;

fn main() {
    let raw = "1,2,3";
    let parsed = parser::parse(raw);     // &str -> Vec<i32>
    let processed = processor::double(&parsed);
    output::print(&processed);
}
```

```rust
// src/parser.rs
pub fn parse(s: &str) -> Vec<i32> {
    s.split(',').filter_map(|t| t.trim().parse().ok()).collect()
}
```

```rust
// src/processor.rs
pub fn double(v: &[i32]) -> Vec<i32> {
    v.iter().map(|x| x * 2).collect()
}
```

```rust
// src/output.rs
pub fn print(v: &[i32]) {
    println!("{v:?}");
}
```

`main.rs` 只做编排，每个模块职责单一。这正是把第 7 章的 Todo 管理器拆成 `store`/`cli`/`persistence` 模块时该用的结构。

---

## 8.9 最佳实践

1. **从扁平开始，等模式浮现再抽模块。** 别为小程序预先搭一棵深目录树。
2. **在 crate 根再导出干净的公开 API。** 用户应当 `use your_crate::Thing`，而不是钻进你的内部模块树。
3. **内部用 `pub(crate)` 而非 `pub`。** 让公开面保持小。
4. **逻辑放进库，而非 `main.rs`。** 第一次写测试时就会尝到甜头。
5. **把相关常量与类型归到一个模块**，而不是让它们漂在 crate 根。

---

## 8.10 小结

Rust 模块系统的核心是**受控可见性**：一切默认私有，你只暴露调用方需要的部分。`use` 把路径引入作用域，`mod`（配合文件）把代码拆到文件系统，crate 与 workspace 把结构扩展到跨团队。保持公开 API 窄、库厚、`main.rs` 薄。

### 练习

1. 把一个含三件事（解析、处理、输出）的单文件程序拆成三个文件中的模块。
2. 加一个被两个模块共用的 `pub(crate)` 辅助函数，验证外部用户无法命名它。
3. 把一个单 crate 包改造成 workspace：一个 `core` 库 + 一个依赖它的 `cli` 二进制。
