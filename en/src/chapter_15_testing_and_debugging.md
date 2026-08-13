# Chapter 15: Testing & Debugging

Rust's testing story is built into the language: the compiler knows about `#[test]`, the standard library ships the assertions, and `cargo test` runs everything. This chapter covers unit, integration, and doc tests, then moves to the tools that find bugs the compiler cannot — property testing, structured logging, and the debugger.

## Learning Objectives

- Write unit, integration, and doc tests.
- Organize test modules and use the common assertion macros.
- Test async code and external dependencies.
- Generate randomized inputs with property-based testing.
- Debug with `tracing` and `lldb`.

---

## 15.1 Unit tests

A unit test lives next to the code it tests, in a `#[cfg(test)]` module so it is compiled only for `cargo test`:

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
        // Demonstrates checking for an expected panic.
        panic!("overflow");
    }
}
```

Core assertions:

| Macro | Checks |
|-------|--------|
| `assert!(cond)` | Condition is true. |
| `assert_eq!(a, b)` | Two values are equal. |
| `assert_ne!(a, b)` | Two values differ. |
| `should_panic` | The test panics (optionally with a message). |

Keep tests small, focused, and independent — each tests one behavior.

---

## 15.2 Integration tests

Integration tests live in `tests/` and exercise the crate's public API as an external user would. Each file is a separate binary:

```rust
// tests/api.rs
use my_crate::add;

#[test]
fn add_from_outside() {
    assert_eq!(add(3, 4), 7);
}
```

Use integration tests for end-to-end paths and unit tests for internal branches.

---

## 15.3 Doc tests

Code blocks in `///` doc comments are compiled and run by `cargo test`. They double as examples and as a correctness check that the documented API actually works:

```rust
/// Adds two integers.
///
/// ```
/// use my_crate::add;
/// assert_eq!(add(2, 2), 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

If an example should not run, mark it ```` ```no_run ```` or ```` ```ignore ````. Doc tests keep your documentation honest.

---

## 15.4 Testing async code

`tokio` provides a `#[tokio::test]` attribute that wraps the test in a runtime:

```rust
# use tokio;

#[tokio::test]
async fn fetches_a_value() {
    let result = some_async_fn().await;
    assert_eq!(result, 42);
}
```

For code with timers, use `tokio::time::pause` and `advance` to make tests deterministic without real delays.

---

## 15.5 Mocking and dependency injection

Rust has no built-in mocking framework, and that is by design — the idiomatic approach is dependency injection via traits. Define a small trait for the external dependency, write a fake implementation in tests, and pass it in:

```rust
pub trait Clock {
    fn now(&self) -> u64;
}

pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> u64 { std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() }
}

pub fn greet(name: &str, clock: &dyn Clock) -> String {
    let hour = (clock.now() / 3600) % 24;
    if hour < 12 { format!("good morning, {name}") }
    else { format!("hello, {name}") }
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
        assert_eq!(greet("alice", &FixedClock(7 * 3600)), "good morning, alice");
    }
}
```

For heavier mocking, `mockall` generates mock implementations of traits automatically.

---

## 15.6 Property-based testing

Instead of writing one example at a time, state a *property* that should always hold and let the framework search for a counterexample. `proptest` is the standard crate:

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

Property tests find edge cases you would not think to write — empty inputs, maximum values, off-by-ones — by shrinking a failing random case to a minimal reproducer.

---

## 15.7 Debugging with `tracing`

`println!` works, but `tracing` gives you structured, leveled, context-aware logs that work across async tasks:

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
    info!("started processing");
    // ...
}

fn main() {
    tracing_subscriber::fmt::init();
    process("alice");
}
```

Spans attach context (the function, its arguments) to every log line beneath them, which is invaluable when many requests are interleaved.

---

## 15.8 The debugger

When logs are not enough, use `lldb` (or `gdb`, or the IDE's debugger) with a debug build:

```bash
cargo build
lldb -- target/debug/myapp
```

Set breakpoints with `b function_name`, step with `n` / `s`, and inspect with `p variable`. For panics, run with `RUST_BACKTRACE=1` to get a stack trace without a debugger:

```bash
RUST_BACKTRACE=1 cargo run
```

---

## 15.9 Best Practices

1. **Test behavior, not implementation.** A test that reaches into private internals breaks on every refactor.
2. **One assertion per test where possible.** Narrow tests pinpoint the failure.
3. **Keep the fast tests fast.** Move slow integration tests behind a feature flag so `cargo test` stays snappy.
4. **Write the failing test first.** It confirms the bug exists before you fix it.
5. **Property-test pure functions.** They are where `proptest` shines.

---

## 15.10 Summary

`cargo test` runs unit tests in `#[cfg(test)]` modules, integration tests in `tests/`, and doc tests in `///` comments — one command, three kinds of coverage. Inject dependencies via traits to test in isolation, hunt edge cases with `proptest`, and reach for `tracing` and `lldb` when behavior goes wrong. Testing in Rust is boring in the best way: it is just code, compiled and run by the same toolchain.

### Exercises

1. Add unit and doc tests to a `sort` wrapper and confirm both run under `cargo test`.
2. Use `proptest` to verify that reversing a `Vec` twice yields the original.
3. Add a `tracing` span to a function and inspect the output with `tracing_subscriber::fmt`.
