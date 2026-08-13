# Chapter 1: Rust Overview & Setup

Rust is a systems language that aims for "safety without sacrificing performance": no garbage collector, yet whole classes of memory bugs are eliminated at compile time. This chapter explains why Rust exists, what its core features are, and how to set up a working environment and run your first program.

## Learning Objectives

- Understand Rust's design philosophy and core features.
- Install and manage the Rust toolchain with `rustup`.
- Create, build, and run projects with `cargo`.
- Understand the difference between debug and release builds.

---

## 1.1 Why Rust

Rust began at Mozilla (2006, public in 2010) with a goal: the performance and control of C++ without the memory bugs. Its three pillars are:

- **Memory safety**: ownership, borrowing, and lifetimes are checked at compile time, preventing null pointers, dangling references, buffer overflows, and data races — without a garbage collector or manual `free`.
- **Zero-cost abstractions**: high-level abstractions (iterators, generics, traits) compile down to code as fast as hand-written low-level code.
- **Fearless concurrency**: the same ownership rules also prevent data races at compile time, so you can write multithreaded code with confidence.

The trade-off is a learning curve: the borrow checker will at first "reject" your code, but what it rejects is real bugs. Once it clicks, the constraints become a reliable safety net for refactoring.

---

## 1.2 Installing the Toolchain

Rust is managed with `rustup`. On macOS/Linux:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Windows users download `rustup-init.exe`. After install, restart your shell and verify:

```bash
rustc --version
cargo --version
```

`rustup` lets you switch toolchains, add cross-compilation targets, and install components:

```bash
rustup update                       # update to latest stable
rustup component add clippy rustfmt
rustup target add wasm32-unknown-unknown   # add a WebAssembly target
```

> **Tip**: the `stable` channel is fine for everyday work. Try `nightly` for cutting-edge features, but don't depend on it in production.

---

## 1.3 Hello, Cargo

`cargo` is Rust's build tool and package manager; nearly every Rust project starts with it:

```bash
cargo new hello_rust
cd hello_rust
```

The layout it generates:

```
hello_rust/
├── Cargo.toml    # project manifest (dependencies, metadata)
└── src/
    └── main.rs   # source entry point
```

`src/main.rs` by default:

```rust
fn main() {
    println!("Hello, world!");
}
```

Build and run:

```bash
cargo run
# prints: Hello, world!
```

`cargo run` compiles then runs. `cargo build` compiles without running; `cargo check` does only type checking without producing a binary — the fastest feedback loop during development.

---

## 1.4 Cargo Basics

`Cargo.toml` is the project manifest:

```toml
[package]
name = "hello_rust"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
```

- `edition`: the language edition (2015/2018/2021). New projects use `2021`.
- `[dependencies]`: declares dependencies; `cargo` fetches them from [crates.io](https://crates.io) and pins them in `Cargo.lock`.

Common commands:

| Command | Purpose |
|---------|---------|
| `cargo new <name>` | new binary project |
| `cargo new --lib <name>` | new library project |
| `cargo build` | compile (debug build) |
| `cargo build --release` | optimized build, for release/benchmarks |
| `cargo run` | compile and run |
| `cargo check` | type-check only (fastest) |
| `cargo test` | run all tests |
| `cargo fmt` | format code |
| `cargo clippy` | run lints |
| `cargo doc --open` | generate and open docs |

> **Debug vs release**: the default `cargo build` is a debug build (`opt-level = 0`, fast to compile, includes debug info). For benchmarks or deployment you must use `--release`, or the results are not representative.

---

## 1.5 A Slightly Bigger Example

A taste of Rust's style — explicit types, expression semantics, zero-cost abstraction:

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];

    // Iterator combinators: filter evens, double, sum
    let result: i32 = numbers
        .iter()
        .filter(|&&n| n % 2 == 0)
        .map(|&n| n * 2)
        .sum();

    println!("sum of doubled evens = {result}"); // 4 + 8 + 12 = 24
}
```

This reads like a math formula yet compiles to the same machine code as a hand-written loop. That is "zero-cost abstraction" made concrete — later chapters unpack each mechanism.

---

## 1.6 Toolchain & Ecosystem

- **rust-analyzer**: the IDE backend that powers VS Code / Vim / Emacs with completion, jump-to-definition, inline types. Install it and the dev experience transforms.
- **rustfmt**: the official formatter; ends style debates.
- **clippy**: the linter; catches a long list of common mistakes.
- **crates.io**: the package registry. `cargo add <crate>` adds a dependency.
- **docs.rs**: auto-generated docs for every crate published to crates.io.

---

## 1.7 Summary

Rust guarantees memory safety and concurrency safety at compile time via ownership, and offers zero-cost abstractions so high-level code does not sacrifice performance. `rustup` manages the toolchain, `cargo` manages projects and dependencies, and `cargo check`/`run`/`test` are the daily trio. Add `rust-analyzer`, `rustfmt`, and `clippy`, and you have a capable environment.

### Exercises

1. Create a project with `cargo new`, write a function returning the first N Fibonacci numbers, and verify with `cargo run` and `cargo test`.
2. Add a dependency (e.g. `rand`) and inspect the generated docs with `cargo doc --open`.
3. Write code that triggers a `clippy` warning (e.g. a needless `return`), run `cargo clippy`, and fix it.
