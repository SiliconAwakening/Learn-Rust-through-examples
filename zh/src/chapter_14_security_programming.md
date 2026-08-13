# 第14章：安全编程

Rust 在构造层面消灭了一整类漏洞——缓冲区溢出、释放后使用、空指针解引用、数据竞争在安全代码里都是编译期错误，而非运行时漏洞。但内存安全不是全部：一个安全的应用还必须校验输入、管理密钥、认证用户，并抵御针对任何 Web 服务的攻击。本章讲的是在 Rust 保证**之上**叠加的实战安全实践。

## 学习目标

- 理解 Rust 自动防止了什么、没有防止什么。
- 校验与净化不可信输入。
- 正确地哈希与加盐密码。
- 管理密钥而不泄露到日志或源码。
- 应用 TLS 与常见 Web 安全响应头。

**实战项目**：构建一个安全认证服务，覆盖多因素认证、密码管理与安全审计。

---

## 14.1 Rust 防止了什么，没有防止什么

安全代码里，Rust 的所有权模型让以下问题不可能发生：

- **缓冲区溢出**——下标越界是 panic 而非溢出。
- **释放后使用与双重释放**——move/borrow 系统禁止对已释放内存的别名可变访问。
- **空指针解引用**——没有 null；缺失用 `Option<T>`。
- **数据竞争**——`Send`/`Sync` 让无同步的并发可变成为编译错误。

Rust **没有**防止的：

- **逻辑 bug**——内存正确，答案错误。
- **release 构建的整数溢出**（会回绕；要紧时用 `checked_*`/`saturating_*`）。
- **不可信输入导致的 panic**——对攻击者可控数据 `unwrap` 会崩进程。
- **注入**——用原始输入拼 SQL、HTML、shell 命令。
- **泄露密钥**——持有密码的 `String` 只是一段内存，编译器照样会打印它。

因此安全的关键在于你的程序与不可信数据之间的边界。

---

## 14.2 在边界校验输入

第一道防线是在畸形输入进入领域逻辑前就拒掉。`serde` 反序列化已能捕获类型错误；语义规则用 `validator` crate：

```toml
[dependencies]
validator = { version = "0.16", features = ["derive"] }
```

```rust
use validator::Validate;

#[derive(serde::Deserialize, Validate)]
struct Signup {
    #[validate(length(min = 3, max = 32))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password: String,
}

fn handle_signup(input: Signup) -> Result<(), String> {
    input.validate().map_err(|e| e.to_string())?;
    Ok(())
}
```

所有外部数据——HTTP 请求体、查询串、环境变量、文件内容——都应视为不可信，直到被校验。

---

## 14.3 SQL 与命令注入

规则是绝对的：**永远不要把不可信数据插进命令字符串。** SQL 用参数绑定（第 11 章），子进程用类型化参数数组：

```rust
use std::process::Command;

// 好——参数是传进去的，不由 shell 解析
let output = Command::new("ls")
    .arg("-l")
    .arg(user_path)        // 即使含空格或 ";" 也安全
    .output()?;
```

避免 `Command::new("sh").arg("-c").arg(format!("ls {user_path}"))`——那把用户输入交给了 shell，重新打开注入。

---

## 14.4 密码：哈希，永不存储

永远不要明文或可逆密码学方式存储密码。用为密码设计的慢、加盐哈希。`argon2` 是当前标准：

```toml
[dependencies]
argon2 = "0.5"
```

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

fn hash_password(plain: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(plain.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

fn verify_password(plain: &str, stored: &str) -> Result<(), argon2::password_hash::Error> {
    let parsed = PasswordHash::new(stored)?;
    Argon2::default().verify_password(plain.as_bytes(), &parsed)
}
```

存储的字符串内嵌盐与参数，所以验证是一行。永远别自己造哈希。

---

## 14.5 密钥管理

密钥（API key、数据库密码、token）要满足三条：来自环境而非源码；加载一次留在内存；永远不进日志。

```rust
use std::env;

struct Config {
    db_url: String,
    api_key: String,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        Ok(Config {
            db_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL missing")?,
            api_key: env::var("API_KEY").map_err(|_| "API_KEY missing")?,
        })
    }
}
```

实战防护：

- 启动时从环境变量或密钥管理器加载——绝不硬编码、绝不提交进 git。
- 标记密钥字段让日志 crate 跳过（`tracing` 配合 `secrecy` 支持 redact 风格）。
- 用 `secrecy` crate 把密钥包进 `Secret<String>`，它不实现 `Display`，意外 `println!` 会是编译错误。

```toml
[dependencies]
secrecy = "0.8"
```

```rust
use secrecy::Secret;
let api_key: Secret<String> = Secret::new(env::var("API_KEY")?);
// println!("{}", api_key); // 编译不过
```

---

## 14.6 TLS

公共互联网上的明文不可接受。用 `rustls`（纯 Rust TLS 栈）终结 TLS，要么在服务器内，要么在反向代理。客户端 HTTPS，`reqwest` 默认用 rustls：

```rust
let resp = reqwest::get("https://example.com").await?.text().await?;
```

固定到特定 TLS 版本（要求 TLS 1.2+）与精选密码套件；默认值保守且通常正确。

---

## 14.7 Web 安全响应头

几个响应头能阻止整类浏览器攻击：

| 头 | 作用 |
|----|------|
| `Content-Security-Policy` | 限制脚本/样式可从哪加载——击溃多数 XSS |
| `Strict-Transport-Security` | 强制未来访问走 HTTPS（HSTS） |
| `X-Content-Type-Options: nosniff` | 阻止 MIME 嗅探 |
| `X-Frame-Options: DENY` | 防止 iframe 点击劫持 |

在 `axum` 里用 `tower_http::set_header::SetResponseHeaderLayer` 加这些，或用专门的中间件。CSP 最强——严格的策略即使你有渲染 bug 也能挡住反射型 XSS。

---

## 14.8 认证与会话

基于 cookie 的认证：

- 发放随机、不可猜测的会话 token（用 `getrandom` 或 `uuid` v4）。
- 服务端存会话，映射到用户 id，带过期。
- cookie 设 `HttpOnly`（JS 不可访问）、`Secure`（仅 HTTPS）、`SameSite=Lax`（防 CSRF）。
- 权限变更（登录、提权）时轮换 token。

无状态 token（JWT）用强算法（EdDSA 或 HS256 配长密钥），设短过期，敏感操作用服务端 denylist 撤销。

---

## 14.9 最佳实践

1. **所有外部输入都视为敌意**，直到被校验。
2. **绑定，绝不拼接**——SQL、shell、URL 构造都是。
3. **用 Argon2 哈希密码**；永不存储或记录。
4. **密钥从环境加载**，包起来让它无法被打印。
5. **公共互联网处处 TLS。**
6. **设安全头**，尤其 CSP。
7. **保持依赖更新**——`cargo audit` 标记已知漏洞。

---

## 14.10 小结

Rust 移除了内存安全攻击面——这是真实漏洞里很大的一块。剩下的是应用层：校验输入、防注入、哈希密码、保护密钥、终结 TLS。把 Rust 的编译期保证与这些边界纪律结合，你就得到一个比多数技术栈显著更硬的目标。

### 练习

1. 给注册 handler 加 `validator`，拒掉短于 3 字符的用户名与非法邮箱。
2. 用 `argon2` 哈希并验证密码，把哈希串存进 SQLite 行。
3. 把 API key 包进 `secrecy::Secret`，确认编译器拒绝意外的 `println!`。
4. 给 Axum 路由加 `Strict-Transport-Security` 与一个基础 `Content-Security-Policy` 头。
