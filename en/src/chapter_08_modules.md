# Chapter 8: Modules & Project Organization

Once a program grows beyond a single screen, organization matters as much as correctness. Rust gives you a module system that controls **visibility**, resolves **paths**, and splits code across **crates** and **workspaces**. This chapter shows how to structure a project so that it scales from a few hundred lines to a large codebase without becoming a tangle.

## Learning Objectives

- Declare modules and submodules with `mod`.
- Control what is public with `pub` and `pub(crate)`.
- Bring items into scope with `use`, including aliases and re-exports.
- Split a crate across files following Rust's path conventions.
- Organize a multi-crate project with a Cargo workspace.

---

## 8.1 Modules: the basics

A module groups related items and gives them a namespace. Declare one with `mod`:

```rust
mod network {
    pub fn connect(host: &str) {
        println!("connecting to {host}");
        configure();
    }

    fn configure() {
        // private — only visible inside `network`
        println!("configuring socket");
    }
}

fn main() {
    network::connect("example.com");
    // network::configure(); // error: `configure` is private
}
```

Items are **private by default**. `pub` makes them visible outside their module. This default is the opposite of many languages and is a deliberate safety feature: you must explicitly opt in to exposing an API.

---

## 8.2 Paths and `use`

To reach an item you qualify it with a path: `crate::network::connect`, or from another module `network::connect`. The `use` declaration shortens this:

```rust
mod network {
    pub mod tcp {
        pub fn listen(port: u16) {
            println!("listening on {port}");
        }
    }
}

use network::tcp::listen; // bring `listen` into scope

fn main() {
    listen(8080); // no need to qualify
}
```

Two useful `use` forms:

```rust
// Group several paths from the same module.
use std::io::{self, Read, Write};

// Re-export so callers see a shorter path.
pub use network::tcp::listen as tcp_listen;
```

When two imported items have the same name, you can alias one: `use std::fmt::Result as FmtResult;`.

---

## 8.3 Splitting code into files

Rust lets a module's body live in another file. The convention:

```
src/
├── main.rs
├── network.rs        // contents of `mod network` declared in main.rs
└── network/
    └── tcp.rs        // contents of `mod tcp` declared in network.rs
```

In `main.rs` you declare the modules without a body, and Rust finds the corresponding file:

```rust
// src/main.rs
mod network;

fn main() {
    network::tcp::listen(8080);
}
```

```rust
// src/network.rs
pub mod tcp; // Rust looks for src/network/tcp.rs
```

```rust
// src/network/tcp.rs
pub fn listen(port: u16) {
    println!("listening on {port}");
}
```

The rules: `mod foo;` with no body tells Rust to look for `foo.rs` or `foo/mod.rs`. Declare submodules in the file that corresponds to their parent.

---

## 8.4 Visibility in depth

Visibility controls who may *name* an item.

| Visibility | Accessible from |
|------------|-----------------|
| (default) `private` | The current module and its descendants only. |
| `pub` | Any module that can name it. |
| `pub(crate)` | Anything in the current crate. |
| `pub(super)` | The parent module. |
| `pub(in path)` | A specific ancestor module. |

`pub(crate)` is the workhorse for library internals that several modules share but that you do not want to expose to users of the crate:

```rust
pub(crate) fn internal_cache_key(s: &str) -> String {
    format!("cache:{s}")
}
```

A subtle rule: making a struct `pub` does not make its fields public. You must mark each field `pub` individually:

```rust
pub struct User {
    pub name: String,    // public
    created_at: u64,     // private — callers cannot read or write it
}
```

---

## 8.5 Crates and packages

A **crate** is the unit of compilation. A **package** is a directory with a `Cargo.toml` that contains one or more crates. A binary crate has a `main` function; a library crate does not.

A common layout for a package that ships both a library and a binary:

```
src/
├── lib.rs     // library crate root
├── main.rs    // binary crate root — uses the library
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
use my_crate::greet; // the binary depends on its own library

fn main() {
    greet("world");
}
```

Keeping the real logic in the library and `main.rs` thin makes your code testable: tests can link the library directly.

---

## 8.6 Workspaces

When a project contains several crates that evolve together, a **workspace** shares a single `target/` directory and `Cargo.lock`:

```toml
# Cargo.toml at the workspace root
[workspace]
members = ["core", "cli", "server"]
```

Each member is its own crate with its own `Cargo.toml`. They depend on each other by path:

```toml
# cli/Cargo.toml
[dependencies]
core = { path = "../core" }
```

Workspaces keep build times reasonable (one `target/` dir) and let you version and test the crates together while keeping their boundaries clean.

---

## 8.7 The standard library prelude

Some items are always in scope without a `use` — `Vec`, `String`, `Option`, `Result`, `println!`. This is the **prelude**, a small set of the most common types re-exported by the standard library. You never need to import them.

---

## 8.8 Best Practices

1. **Start flat, extract modules as patterns emerge.** Do not pre-build a deep folder tree for a tiny program.
2. **Re-export a clean public API at the crate root.** Users should `use your_crate::Thing`, not navigate your internal module tree.
3. **Prefer `pub(crate)` over `pub` for internals.** Keep your public surface small.
4. **Put logic in the library, not `main.rs`.** It pays off the first time you write a test.
5. **Group related constants and types in a module** rather than letting them float at the crate root.

---

## 8.9 Summary

Rust's module system is about *controlled visibility*: everything is private by default, and you expose exactly what callers need. `use` brings paths into scope, `mod` (with files) splits code across the filesystem, and crates plus workspaces scale the structure across teams. Keep the public API narrow, the library fat, and `main.rs` thin.

### Exercises

1. Take a single-file program with three concerns (parsing, processing, output) and split it into three modules in separate files.
2. Add a `pub(crate)` helper used by two modules, and verify that an external user cannot name it.
3. Convert a single-crate package into a workspace with a `core` library and a `cli` binary that depends on it.
