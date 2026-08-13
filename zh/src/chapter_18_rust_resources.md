# 第18章：Rust 进阶资源与官方书导读

你已经读到了本书的末尾，但 Rust 是一门庞大的语言，生态也在快速演进。本章是一张经过筛选的地图：接下来该读哪些权威资料、该装哪些工具、该常备哪些参考，以及一条从“能写 Rust”到“熟练驾驭 Rust”的进阶路径。

## 学习目标

- 了解官方文档体系，知道何时查阅哪一份。
- 用 `rustup`、`cargo`、`clippy`、`rustfmt` 搭建日常工具链。
- 在 crate 生态中导航并评估质量。
- 沿一条 deliberate 的路径走向熟练。

---

## 18.1 官方文档

Rust 项目维护着一批免费且互相交叉引用的书籍，各司其职：

| 资源 | 网址 | 用途 |
|------|------|------|
| **The Rust Programming Language**（“the Book”） | doc.rust-lang.org/book | 带项目实战的引导式入门，最权威的起点 |
| **Rust by Example** | doc.rust-lang.org/rust-by-example | 按主题组织、可直接运行的代码片段，速查用 |
| **The Rust Reference** | doc.rust-lang.org/reference | 精确、权威的语言语义（非教程） |
| **The Rustonomicon** | doc.rust-lang.org/nomicon | “黑魔法”：`unsafe`、FFI、底层内存 |
| **The Async Book** | rust-lang.github.io/async-book | async/await 底层原理 |
| **The Cargo Book** | doc.rust-lang.org/cargo | 构建系统与打包的一切 |
| **The Edition Guide** | doc.rust-lang.org/edition-guide | 2015、2018、2021 edition 之间的差异 |
| **The API Guidelines** | rust-lang.github.io/api-guidelines | 如何设计地道的 Rust API |
| **std API 文档** | doc.rust-lang.org/std | 标准库参考 |

一个好习惯：写代码时常开 `doc.rust-lang.org/std`，遇到常用的类型就读读它的源码——标准库本身就是典范级的 Rust 代码。

---

## 18.2 工具集

每位 Rust 开发者都应把以下工具接入编辑器与 CI：

- **`rustup`**——管理工具链与目标。`rustup update` 保持最新；`rustup component add` 添加组件。
- **`cargo`**——构建、测试、生成文档、发布。`cargo check` 是快速反馈循环；`cargo build --release` 用于交付。
- **`rustfmt`**——官方格式化器。运行 `cargo fmt`，让格式永远不必成为代码评审的话题。
- **`clippy`**——lint 工具。`cargo clippy` 能揪出一长串常见错误与不地道写法。认真对待它的告警，其中不少就是真 bug。
- **`cargo doc --open`**——为你的 crate 及其依赖生成并启动文档服务。读自己生成的文档是评估 API 的好办法。

```bash
rustup component add rustfmt clippy
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --open
```

---

## 18.3 crate 生态

有些 crate 用得如此之广，几乎算语言的一部分。认识它们能省去重复造轮子：

| 领域 | crate | 用途 |
|------|-------|------|
| 序列化 | `serde`、`serde_json` | 通用（反）序列化层 |
| 错误处理 | `thiserror`、`anyhow` | 库与应用的错误类型 |
| 异步运行时 | `tokio` | 主流异步运行时 |
| HTTP 服务 | `axum`、`actix-web` | Web 框架 |
| HTTP 客户端 | `reqwest` | 高层阻塞/异步客户端 |
| 数据库 | `sqlx` | 异步、编译期校验的 SQL |
| 日志 | `tracing`、`tracing-subscriber` | 结构化日志与 span |
| 随机数 | `rand` | 随机数生态 |
| 正则 | `regex` | Perl 风格正则 |
| CLI 解析 | `clap` | 带 derive 宏的参数解析 |
| 日期时间 | `chrono`、`time` | 日期时间运算 |
| 并行 | `rayon` | 数据并行迭代器 |

**评估一个 crate：** 在 `crates.io` 看下载量与近期版本日期，读 README，扫一眼未关闭的 issue，优先选维护活跃、文档完善的。一个两年没更新的 crate 是负债。

---

## 18.4 走向熟练的路径

1. **通读 the Book。** 相比其覆盖面，它并不长，而且边讲边搭一个 `grep` 克隆项目。
2. **做 `rustlings`。** 一组小练习，补上 the Book 留给读者练习的空档。
3. **做个真实项目。** 一个 CLI 工具、一个小 Web 服务、一个小游戏——自有项目会暴露教程预见不到的问题。
4. **读优秀的代码。** 标准库、`serde`、`tokio`、`axum` 都写得很好，值得学习。
5. **最后再写 `unsafe`。** 多数 Rust 程序员很少需要它；真要用时，先读 Rustonomicon。
6. **融入社区。** `rust-users` 论坛、Discord、本地 meetup 都很友好且底蕴深厚。

---

## 18.5 保持同步

Rust 每六周发布一次——稳定的发布列车，而非漫长的大版本间隔。多数发布是增量式的。留意偶尔出现的 edition（一次引入小幅语言便利而不破坏生态的机会），以及年度 Rust 调查，了解社区走向。

把 `rustup update` 放进日常，浏览发布说明，把小幅风格变化交给 `clippy` 与 `rustfmt` 吸收即可。

---

## 18.6 结语

Rust 的承诺是：你可以写出快速、底层的代码，而不必背负通常随之而来的恐惧。编译器很严格，但正是这份严格，让你能放心地重构大型代码库、上线一个不会因空指针崩溃的服务、发布一段不会缓冲区溢出的固件。学习投入是真实的，回报同样真实。

让标准库文档常开，每天写一点代码，让借用检查器来教你。欢迎来到 Rust 的世界。
