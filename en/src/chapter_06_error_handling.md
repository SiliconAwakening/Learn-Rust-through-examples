# Chapter 6: Error Handling

Error handling is where Rust diverges most sharply from mainstream languages — and where it earns the reputation of being "fearless." Rust does not throw exceptions. Instead, the type system makes the possibility of failure **visible in function signatures**, so the compiler forces you to decide what happens when something goes wrong. The payoff is dramatic: a large class of crashes that silently happen at runtime in other languages simply cannot occur in shipped Rust code.

This chapter teaches you to think about errors the Rust way: divide them into *recoverable* and *unrecoverable*, model recoverable failures with `Result`, propagate them cleanly with `?`, and design error types that scale from a small script to a large library.

## Learning Objectives

- Distinguish recoverable errors (`Result`) from unrecoverable ones (`panic!`).
- Use `Option<T>` to represent the *absence* of a value.
- Use `Result<T, E>` and pattern matching to handle expected failures.
- Propagate errors concisely with the `?` operator.
- Convert between error types with the `From` trait.
- Design custom error types with `thiserror`, and choose `anyhow` for applications.
- Apply error-handling best practices in `async` code.

---

## 6.1 Two Kinds of Errors

Rust groups errors into two families. The distinction is the foundation of everything in this chapter.

| Kind | Type | Meaning | Example |
|------|------|---------|---------|
| **Unrecoverable** | `panic!` | A bug or a broken invariant — the program cannot safely continue. | Index out of bounds, dividing by zero, a `Mutex` that was poisoned. |
| **Recoverable** | `Result<T, E>` | An expected failure that the caller can react to. | File not found, network timeout, malformed input. |

**Mental model.** A `panic` is the program saying *"something is wrong that I cannot fix; stop now."* A `Result` is a function saying *"this might fail — here is the value, or here is the reason it failed; you decide."* Most real-world failures are recoverable, so most of your error handling will use `Result`.

### 6.1.1 `panic!` — when something is truly wrong

A panic unwinds the stack (or aborts) and ends the current thread. Use it for conditions that should *never* happen in correct code.

```rust
fn main() {
    let numbers = [10, 20, 30];
    // Index 5 is out of bounds — a logic bug, so Rust panics.
    let value = numbers[5];
    println!("{value}");
}
```

You can trigger a panic explicitly with the `panic!` macro, and attach a message:

```rust
fn assert_non_empty<T>(slice: &[T]) {
    if slice.is_empty() {
        panic!("expected a non-empty slice, got an empty one");
    }
}
```

`unwrap()` and `expect()` are shortcuts that panic on failure. They are excellent for quick scripts and tests, but risky in production paths because they turn a recoverable failure into a crash.

```rust
fn main() {
    // `parse` returns Result; `unwrap` panics if parsing fails.
    let n: i32 = "42".parse().unwrap();
    // `expect` lets you attach context — prefer it over bare `unwrap`.
    let m: i32 = "abc".parse().expect("input must be a valid integer");
    println!("{n} {m}");
}
```

> **Rule of thumb.** `unwrap()` / `expect()` are fine in prototypes and tests. In code that handles user input or external systems, reach for `?` and proper `Result` handling instead.

---

## 6.2 `Option<T>` — the absence of a value

Before errors, consider *absence*. When a function can legitimately return "nothing" (not a failure, just no value), use `Option<T>`.

```rust
// The signature itself tells you: this might not find anything.
fn find_user(users: &[&str], name: &str) -> Option<&str> {
    for u in users {
        if *u == name {
            return Some(u);
        }
    }
    None
}

fn main() {
    let users = ["alice", "bob", "carol"];

    match find_user(&users, "bob") {
        Some(name) => println!("found {name}"),
        None => println!("no such user"),
    }

    // Convenience combinators — concise and null-safe.
    let upper = find_user(&users, "alice").map(str::to_uppercase);
    println!("{upper:?}"); // Some("ALICE")

    // Provide a default when the value is missing.
    let display = find_user(&users, "zoe").unwrap_or("guest");
    println!("{display}"); // guest
}
```

Key `Option` combinators:

| Combinator | Returns | Purpose |
|------------|---------|---------|
| `map(f)` | `Option<U>` | Transform the inner value if present. |
| `and_then(f)` | `Option<U>` | Chain operations that themselves return `Option`. |
| `unwrap_or(v)` | `T` | Fall back to `v` when `None`. |
| `unwrap_or_default()` | `T` | Fall back to `T::default()`. |
| `is_some()` / `is_none()` | `bool` | Inspect without consuming. |

Prefer combinators over nested `match` — they express intent more clearly.

---

## 6.3 `Result<T, E>` — recoverable failures

`Result` is the workhorse of Rust error handling. It is just an enum:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

A function that can fail returns `Result` instead of panicking. The classic example is reading a file:

```rust
use std::fs;
use std::io;

fn read_config(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path) // already returns Result<String, io::Error>
}

fn main() {
    match read_config("config.toml") {
        Ok(contents) => println!("config loaded:\n{contents}"),
        Err(error) => eprintln!("failed to read config: {error}"),
    }
}
```

The error type `io::Error` is concrete and informative. Pattern matching lets you branch on the kind of failure:

```rust
use std::io;
use std::fs;

fn main() {
    match fs::read_to_string("missing.txt") {
        Ok(_) => println!("read ok"),
        Err(error) => match error.kind() {
            io::ErrorKind::NotFound => eprintln!("file does not exist"),
            io::ErrorKind::PermissionDenied => eprintln!("no permission"),
            _ => eprintln!("other io error: {error}"),
        },
    }
}
```

`Result` shares most combinators with `Option` — `map`, `and_then` (also called `?`-style chaining), `unwrap_or`, etc.

---

## 6.4 The `?` operator — clean propagation

Handling every error with `match` gets verbose. The `?` operator is the idiomatic way to *propagate* an error: "if this succeeded, keep going; if it failed, return the error to the caller immediately."

```rust
use std::fs;
use std::io;

// `?` turns a verbose match into a one-liner.
fn read_config(path: &str) -> Result<String, io::Error> {
    let contents = fs::read_to_string(path)?; // propagate on error
    Ok(contents.trim().to_string())
}
```

`?` works on both `Result` and `Option`. You can even convert between them with the right context.

### Chaining multiple fallible operations

This is where `?` shines — a pipeline of fallible steps reads like straight-line code:

```rust
use std::fs;
use std::io;

fn load_and_parse(path: &str) -> Result<i32, io::Error> {
    let text = fs::read_to_string(path)?;          // io::Error
    let trimmed = text.trim();
    let value: i32 = trimmed.parse().map_err(|e| {
        // Convert the parse error into an io::Error so the signatures line up.
        io::Error::new(io::ErrorKind::InvalidData, e)
    })?;
    Ok(value * 2)
}
```

Notice `map_err` — it adapts the error type when `?` cannot convert it automatically (see the next section).

---

## 6.5 Converting errors with `From`

`?` does one more thing automatically: if the function's error type `E` implements `From` for the inner error, `?` converts it for you. This lets different subsystems that produce different error types flow into a single error type at the boundary.

```rust
use std::fs;
use std::io;
use std::num::ParseIntError;

// One error type that unifies several lower-level errors.
#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Parse(ParseIntError),
}

// These conversions let `?` work without an explicit `map_err`.
impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}
impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        AppError::Parse(err)
    }
}

fn load_number(path: &str) -> Result<i32, AppError> {
    let text = fs::read_to_string(path)?; // io::Error -> AppError automatically
    let n: i32 = text.trim().parse()?;    // ParseIntError -> AppError automatically
    Ok(n)
}
```

Writing `From` impls by hand is mechanical. In practice, a derive macro does it for you — that is the topic of the next section.

---

## 6.6 Custom error types with `thiserror`

For libraries, define a dedicated error enum and derive the boilerplate (`Debug`, `Display`, `From`) with [`thiserror`](https://docs.rs/thiserror).

```rust
// Cargo.toml
// [dependencies]
// thiserror = "1"
```

```rust
use std::io;
use std::num::ParseIntError;
use thiserror::Error;

/// All the ways our config loader can fail.
#[derive(Debug, Error)]
enum ConfigError {
    #[error("could not read file: {0}")]
    Io(#[from] io::Error),

    #[error("invalid number in config: {0}")]
    Parse(#[from] ParseIntError),

    #[error("missing required key: {key}")]
    Missing { key: String },
}

fn load_port(path: &str) -> Result<u16, ConfigError> {
    let text = std::fs::read_to_string(path)?;   // io::Error auto-converts
    let port: u16 = text.trim().parse()?;         // ParseIntError auto-converts
    if port == 0 {
        return Err(ConfigError::Missing { key: "port".into() });
    }
    Ok(port)
}
```

The `#[from]` attribute generates the `From` impl, so `?` just works. The `#[error("...")]` attribute provides the human-readable `Display` message. This is the recommended way to model errors in any code intended for reuse.

---

## 6.7 `thiserror` vs `anyhow` — library vs application

A common source of confusion: *which* error type should I use? The answer depends on whether your code is a **library** (called by others) or an **application** (the top-level program).

- **Libraries** should return a *specific, structured* error type so callers can match on it and react. Use **`thiserror`**.
- **Applications** mostly just want to bundle any error with context and report it at the top. Use **[`anyhow`](https://docs.rs/anyhow)**, which provides a single `anyhow::Error` that can hold any error and a `.context(...)` method to attach a human-readable message.

```rust
// Cargo.toml
// [dependencies]
// anyhow = "1"
```

```rust
use anyhow::{Context, Result};
use std::fs;

fn load_port(path: &str) -> Result<u16> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {path:?}"))?;
    let port: u16 = text.trim().parse()
        .with_context(|| format!("port in {path:?} is not a valid number"))?;
    Ok(port)
}

fn main() -> Result<()> {
    let port = load_port("config.toml")?;
    println!("listening on {port}");
    Ok(())
}
```

If `load_port` fails, `anyhow` prints a chain like:

```
Error: port in "config.toml" is not a valid number

Caused by:
    invalid digit found in string
```

**Guideline.** Return `thiserror`-based errors from libraries; use `anyhow::Result` in binaries, tests, and glue code. The two compose perfectly: an `anyhow::Error` can wrap any error that implements `std::error::Error`, which `thiserror` types do.

---

## 6.8 Error handling in async code

In `async` functions, `?` works exactly the same way — but the error travels out through a `Future` rather than a direct return. The only subtlety is selecting an error type that is `Send` when the future is sent across threads (e.g. with a multi-threaded Tokio runtime).

```rust
// Cargo.toml
// [dependencies]
// tokio = { version = "1", features = ["full"] }
// anyhow = "1"
```

```rust
use anyhow::{Context, Result};
use tokio::fs;
use tokio::io::AsyncReadExt;

async fn read_head(path: &str) -> Result<String> {
    let mut file = fs::File::open(path)
        .await
        .with_context(|| format!("open {path:?}"))?;
    let mut buf = [0u8; 64];
    let n = file.read(&mut buf)
        .await
        .context("read first bytes")?;
    Ok(String::from_utf8_lossy(&buf[..n]).into_owned())
}

#[tokio::main]
async fn main() -> Result<()> {
    let head = read_head("README.md").await?;
    println!("{head}");
    Ok(())
}
```

The pattern is identical to the synchronous case: each fallible `.await` is followed by `?` or `.context(...)`. Treat async error handling as ordinary `Result` handling that happens to be interrupted by `.await` points.

---

## 6.9 Best Practices

1. **Model failure in the type system.** Prefer `Result` over `panic!` for anything a caller might want to handle.
2. **Let `?` propagate.** Resist the urge to `match` every error — `?` is clearer and shorter.
3. **Attach context early.** Use `.context()` / `.with_context()` so the top-level error message explains *what* you were trying to do, not just the low-level cause.
4. **Libraries: structured errors.** Use `thiserror` and expose a public `Error` enum so callers can match on variants.
5. **Applications: `anyhow`.** Use `anyhow::Result` for top-level orchestration and glue code.
6. **Avoid `unwrap`/`expect` in production paths.** Keep them for tests, examples, and truly impossible states.
7. **Don't discard errors.** Never write `let _ = fallible();` unless you genuinely intend to ignore the outcome — and even then, add a comment.
8. **Errors are values.** Log, wrap, retry, or convert them — but do so explicitly.

---

## 6.10 Summary

Rust treats errors as values. Unrecoverable bugs become `panic!`; expected failures become `Result`. The `?` operator makes propagation concise, the `From` trait (often via `thiserror`) makes conversion automatic, and `anyhow` keeps application code tidy. The result is error handling that is explicit enough to reason about, yet ergonomic enough to actually use everywhere.

### Exercises

1. Write a function `parse_pair(s: &str) -> Result<(i32, i32), ParseIntError>` that parses `"3,4"` into `(3, 4)`. Extend it with a custom error that also reports a missing comma.
2. Build a `thiserror`-based `WeatherError` enum with variants for network and parse failures, then write an async function that fetches and parses a JSON-like string using `?`.
3. Rewrite a function that currently uses `unwrap()` three times to use `?` and `anyhow::Result`, adding a `.context()` to each fallible step.
