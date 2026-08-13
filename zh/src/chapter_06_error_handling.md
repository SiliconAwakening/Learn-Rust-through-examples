# 第6章：错误处理

错误处理是 Rust 与主流语言分道扬镳最明显之处，也是它“无畏”名声的来源。Rust 不抛异常，而是让失败的可能性**显式出现在函数签名里**，编译器逼你在编译期决定“出错时怎么办”。代价是更多的类型标注，回报是大量在别处运行时才暴露的崩溃，在这里被挡在了编译期之外。

## 学习目标

- 区分可恢复错误（`Result`）与不可恢复错误（`panic!`）。
- 用 `Option<T>` 表示“值的缺失”。
- 用 `Result<T, E>` 与模式匹配处理预期失败。
- 用 `?` 操作符优雅地传播错误。
- 用 `From` trait 转换错误类型。
- 用 `thiserror` 设计自定义错误类型，在应用层选用 `anyhow`。
- 在异步代码中应用错误处理最佳实践。

---

## 6.1 两类错误

Rust 把错误分成两族，本章一切皆从此而来：

| 类别 | 类型 | 含义 | 示例 |
|------|------|------|------|
| **不可恢复** | `panic!` | bug 或不变量被破坏，程序无法安全继续 | 下标越界、除零、Mutex 中毒 |
| **可恢复** | `Result<T, E>` | 预期内会发生的失败，调用方可反应 | 文件不存在、网络超时、格式错误 |

**心智模型**：`panic` 是程序在说“出了我无法修的问题，立刻停”；`Result` 是函数在说“这事可能失败——给你值，或给你失败原因，你来定”。现实里多数失败是可恢复的，所以你的错误处理大多用 `Result`。

### `panic!`：真出问题了

panic 会展开栈（或中止）并结束当前线程。用它表示正确代码里**绝不应该**发生的情况：

```rust
fn main() {
    let nums = [10, 20, 30];
    let v = nums[5]; // 越界——逻辑 bug，Rust panic
    println!("{v}");
}
```

`unwrap()` 与 `expect()` 是会 panic 的快捷方式。原型与测试里很好用，生产路径里危险——它把可恢复失败变成崩溃：

```rust
let n: i32 = "42".parse().unwrap();                   // 解析失败会 panic
let m: i32 = "abc".parse().expect("输入必须是整数");    // 带上下文
```

> **经验法则**：`unwrap`/`expect` 适合原型和测试。处理用户输入或外部系统时，用 `?` 和 `Result`。

---

## 6.2 `Option<T>`：值的缺失

先看“缺失”。当函数可能合理地返回“无值”（不是失败，就是没有）时，用 `Option<T>`：

```rust
fn find_user(users: &[&str], name: &str) -> Option<&str> {
    for u in users {
        if *u == name { return Some(u); }
    }
    None
}

fn main() {
    let users = ["alice", "bob", "carol"];

    match find_user(&users, "bob") {
        Some(name) => println!("找到 {name}"),
        None => println!("无此用户"),
    }

    // 组合子——简洁且空安全
    let upper = find_user(&users, "alice").map(str::to_uppercase);
    println!("{upper:?}"); // Some("ALICE")

    let display = find_user(&users, "zoe").unwrap_or("guest");
    println!("{display}"); // guest
}
```

常用 `Option` 组合子：`map`、`and_then`、`unwrap_or`、`unwrap_or_default`、`is_some`/`is_none`。优先用组合子而非层层嵌套 `match`。

---

## 6.3 `Result<T, E>`：可恢复失败

`Result` 是 Rust 错误处理的主力，本质是个枚举：

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

会失败的函数返回 `Result` 而非 panic。读文件是经典例子：

```rust
use std::fs;

fn read_config(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

fn main() {
    match read_config("config.toml") {
        Ok(contents) => println!("配置已加载:\n{contents}"),
        Err(e) => eprintln!("读取失败: {e}"),
    }
}
```

错误类型 `io::Error` 具体且信息丰富。`match` 让你按失败种类分支：

```rust
use std::io::{self, fs};

fn main() {
    match fs::read_to_string("missing.txt") {
        Ok(_) => println!("读取成功"),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => eprintln!("文件不存在"),
            io::ErrorKind::PermissionDenied => eprintln!("无权限"),
            _ => eprintln!("其它 io 错误: {e}"),
        },
    }
}
```

---

## 6.4 `?` 操作符：干净的传播

每个错误都 `match` 会很啰嗦。`?` 是**传播**错误的地道写法：“成功就继续；失败就立刻把错误返回给调用者。”

```rust
use std::fs;
use std::io;

fn read_config(path: &str) -> Result<String, io::Error> {
    let contents = fs::read_to_string(path)?; // 出错则传播
    Ok(contents.trim().to_string())
}
```

`?` 对 `Result` 和 `Option` 都适用。

### 串联多个可失败步骤

`?` 让一连串可失败步骤读起来像直线代码：

```rust
use std::fs;
use std::io;

fn load_and_parse(path: &str) -> Result<i32, io::Error> {
    let text = fs::read_to_string(path)?;
    let value: i32 = text.trim().parse().map_err(|e| {
        // 把解析错误转成 io::Error，让签名对齐
        io::Error::new(io::ErrorKind::InvalidData, e)
    })?;
    Ok(value * 2)
}
```

`map_err` 在 `?` 无法自动转换时用来适配错误类型（见下节）。

---

## 6.5 用 `From` 转换错误

`?` 还做一件自动事：若函数的错误类型 `E` 对内部错误实现了 `From`，`?` 会自动转换。这让不同子系统产生的不同错误类型汇聚到一个边界错误类型：

```rust
use std::fs;
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Parse(ParseIntError),
}

// 这些转换让 `?` 无需 map_err 即可工作
impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self { AppError::Io(err) }
}
impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self { AppError::Parse(err) }
}

fn load_number(path: &str) -> Result<i32, AppError> {
    let text = fs::read_to_string(path)?;   // io::Error 自动转 AppError
    let n: i32 = text.trim().parse()?;      // ParseIntError 自动转 AppError
    Ok(n)
}
```

手写 `From` 很机械。实践中用 derive 宏代劳——下节。

---

## 6.6 用 `thiserror` 设计自定义错误

库应定义专门的错误枚举，用 [`thiserror`](https://docs.rs/thiserror) 派生样板（`Debug`、`Display`、`From`）：

```toml
# Cargo.toml
[dependencies]
thiserror = "1"
```

```rust
use std::io;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, Error)]
enum ConfigError {
    #[error("读取文件失败: {0}")]
    Io(#[from] io::Error),

    #[error("配置中数字非法: {0}")]
    Parse(#[from] ParseIntError),

    #[error("缺少必需的键: {key}")]
    Missing { key: String },
}

fn load_port(path: &str) -> Result<u16, ConfigError> {
    let text = std::fs::read_to_string(path)?;   // 自动转换
    let port: u16 = text.trim().parse()?;
    if port == 0 {
        return Err(ConfigError::Missing { key: "port".into() });
    }
    Ok(port)
}
```

`#[from]` 生成 `From` impl，`?` 直接可用；`#[error("...")]` 提供人类可读的 `Display`。这是任何打算复用的代码建模错误的推荐方式。

---

## 6.7 `thiserror` vs `anyhow`：库 vs 应用

常见困惑：该用哪种错误类型？取决于你是**库**（被别人调用）还是**应用**（顶层程序）。

- **库**应返回*具体、结构化*的错误类型，让调用方能 match 并反应。用 **`thiserror`**。
- **应用**多数时候只想把任何错误打包加上上下文，在顶层报告。用 **[`anyhow`](https://docs.rs/anyhow)**——它提供一个能装任何错误的 `anyhow::Error`，与 `.context(...)` 方法附加上下文：

```toml
[dependencies]
anyhow = "1"
```

```rust
use anyhow::{Context, Result};
use std::fs;

fn load_port(path: &str) -> Result<u16> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("读取配置文件 {path:?} 失败"))?;
    let port: u16 = text.trim().parse()
        .with_context(|| format!("{path:?} 里的 port 不是合法数字"))?;
    Ok(port)
}

fn main() -> Result<()> {
    let port = load_port("config.toml")?;
    println!("监听 {port}");
    Ok(())
}
```

失败时 `anyhow` 打印一条链：

```
Error: "config.toml" 里的 port 不是合法数字

Caused by:
    invalid digit found in string
```

**准则**：库返回 `thiserror` 错误；二进制、测试、胶水代码用 `anyhow::Result`。两者完美组合——`anyhow::Error` 能包任何实现了 `std::error::Error` 的错误，`thiserror` 类型都满足。

---

## 6.8 异步代码里的错误处理

`async` 函数里 `?` 的行为完全一样——只是错误经由 `Future` 传出而非直接返回。唯一要留意的是：当 future 跨线程发送时（多线程 Tokio 运行时），错误类型需 `Send`。

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

```rust
use anyhow::{Context, Result};
use tokio::fs;
use tokio::io::AsyncReadExt;

async fn read_head(path: &str) -> Result<String> {
    let mut file = fs::File::open(path)
        .await
        .with_context(|| format!("打开 {path:?}"))?;
    let mut buf = [0u8; 64];
    let n = file.read(&mut buf).await.context("读取首字节")?;
    Ok(String::from_utf8_lossy(&buf[..n]).into_owned())
}

#[tokio::main]
async fn main() -> Result<()> {
    let head = read_head("README.md").await?;
    println!("{head}");
    Ok(())
}
```

模式与同步版一致：每个可失败 `.await` 后跟 `?` 或 `.context(...)`。把异步错误处理当成“恰好被 `.await` 打断”的普通 `Result` 处理即可。

---

## 6.9 最佳实践

1. **把失败建模进类型系统。** 调用方可能想处理的，用 `Result` 而非 `panic!`。
2. **让 `?` 传播。** 别每个错误都 `match`——`?` 更清晰更短。
3. **尽早附加上下文。** 用 `.context()`，让顶层错误说明“你在做什么”，而非只有底层原因。
4. **库用结构化错误。** `thiserror` 暴露公开 `Error` 枚举，让调用方能 match。
5. **应用用 `anyhow`。** 顶层编排与胶水代码用 `anyhow::Result`。
6. **生产路径别用 `unwrap`/`expect`。** 留给测试、示例、真正不可能的状态。
7. **别吞错误。** 永不写 `let _ = fallible();`，除非真要忽略——即便如此也加注释。

---

## 6.10 小结

Rust 把错误当值。不可恢复 bug 成 `panic!`，预期失败成 `Result`。`?` 让传播简洁，`From` trait（常经 `thiserror`）让转换自动，`anyhow` 让应用代码整洁。结果是一种显式到可推理、又顺手到处处可用的错误处理。

### 练习

1. 写一个 `fn parse_pair(s: &str) -> Result<(i32, i32), ParseIntError>`，把 `"3,4"` 解析成 `(3, 4)`；再扩展为自定义错误，能报告“缺少逗号”。
2. 用 `thiserror` 定义 `WeatherError`（网络/解析两个变体），写一个异步函数用 `?` 拉取并解析类 JSON 字符串。
3. 把一个用了三次 `unwrap()` 的函数改写成用 `?` 与 `anyhow::Result`，每个可失败步骤加 `.context()`。
