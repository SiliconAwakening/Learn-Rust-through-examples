# Chapter 13: Performance Optimization

The first rule of optimization is: **measure, then optimize.** Rust is already fast by default, so most "optimization" is about not throwing that speed away — avoiding needless allocations, laying out data for the cache, and choosing the right concurrency model. This chapter is a toolkit for finding bottlenecks and a catalog of the techniques that matter.

## Learning Objectives

- Establish a performance baseline with benchmarks before changing code.
- Profile CPU and memory to find real bottlenecks, not guesses.
- Reduce allocations and copies, the most common Rust performance wins.
- Lay out data for cache locality.
- Choose between threads and async based on workload.

---

## 13.1 Measure first

Never optimize from intuition. Establish a baseline with [`criterion`](https://docs.rs/criterion), which runs statistical benchmarks and reports noise:

```toml
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "string_join"
harness = false
```

```rust
// benches/string_join.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_join(c: &mut Criterion) {
    let words: Vec<String> = (0..1000).map(|i| i.to_string()).collect();

    c.bench_function("join_with_plus", |b| {
        b.iter(|| {
            let mut s = String::new();
            for w in &words { s += w; }
            s
        })
    });

    c.bench_function("join_with_iter", |b| {
        b.iter(|| words.join(""))
    });
}

criterion_group!(benches, bench_join);
criterion_main!(benches);
```

Run with `cargo bench`. Criterion tells you whether a change is a real win or within noise.

---

## 13.2 Profiling

For a whole program, profile with `perf` (Linux), `Instruments` (macOS), or `cargo flamegraph`:

```bash
cargo install flamegraph
cargo flamegraph --bin myapp
```

A flamegraph shows where CPU time is spent, aggregated across the call tree. Look for surprising hot spots — a `clone` you did not expect, a `format!` in a tight loop, a hash function dominating.

---

## 13.3 Allocations are the usual suspect

Heap allocation is cheap, but thousands per second add up. The biggest wins usually come from removing allocations:

```rust
// Bad: allocates a new String on every call.
fn bad(items: &[i32]) -> String {
    let mut s = String::new();
    for x in items { s += &x.to_string(); }
    s
}

// Better: one allocation, sized up front.
fn good(items: &[i32]) -> String {
    // Each i32 is at most 11 chars; preallocate to avoid regrowth.
    let mut s = String::with_capacity(items.len() * 11);
    for x in items { s.push_str(&x.to_string()); }
    s
}
```

Common allocation patterns to question:

- `clone()` inside a loop — can you borrow instead?
- `to_string()` to compare against a `&str` — compare with `==` directly.
- `format!` for logging that is usually disabled — use `log` / `tracing` macros, which skip formatting when the level is off.
- Collecting into a `Vec` only to iterate it once — stay lazy with iterators.

---

## 13.4 String handling

`String` is heap-allocated and growable; `&str` is a borrowed slice. Prefer `&str` in function arguments. When you must build a string, use `String::with_capacity` or `write!` into a `String`:

```rust
use std::fmt::Write;

let mut out = String::with_capacity(64);
write!(out, "x={}, y={}", 10, 20).unwrap();
```

For ASCII identifiers, `CompactString` or simply an interned `&'static str` can avoid per-call allocation.

---

## 13.5 Cache locality and data layout

Modern CPUs are fast at arithmetic and slow at memory access. Data that is contiguous and accessed sequentially is dramatically faster than pointer-chasing. This is why `Vec` beats `LinkedList` almost always, and why a **struct of arrays** can outperform an **array of structs** when you iterate over one field:

```rust
// Array of structs — natural but touches three cache lines per item.
struct Particle { x: f64, y: f64, v: f64 }
let aos: Vec<Particle> = /* ... */;

// Struct of arrays — iterating `xs` streams one contiguous buffer.
struct Particles { xs: Vec<f64>, ys: Vec<f64>, vs: Vec<f64> }
```

If a benchmark shows you spend time loading data you never use, restructuring into a struct of arrays (or splitting a large struct into hot and cold parts) is often a 2–10× win.

---

## 13.6 Hashing

The default `HashMap` uses SipHash, which is DoS-resistant but slower than alternatives. For trusted, non-adversarial keys, `ahash` or `rustc-hash` (FNV-style) is several times faster:

```toml
[dependencies]
ahash = "0.8"
```

```rust
use ahash::AHashMap;
let mut m: AHashMap<&str, i32> = AHashMap::new();
```

---

## 13.7 Inlining and generics

Generic functions in Rust are **monomorphized** — the compiler generates a separate copy per concrete type, which enables inlining and is usually faster than dynamic dispatch. Prefer generics over `dyn Trait` in hot paths:

```rust
// Generic — monomorphized, inlinable, fast.
fn sum<T: Copy + std::ops::Add<Output = T>>(xs: &[T], zero: T) -> T {
    xs.iter().fold(zero, |a, &b| a + b)
}

// Trait object — one copy, virtual dispatch, harder to inline.
fn sum_dyn(xs: &[Box<dyn Numeric>]) -> f64 { /* ... */ }
```

Use `#[inline]` sparingly — the compiler is good at it; reserve hints for tiny leaf functions in libraries that callers will want inlined across crate boundaries.

---

## 13.8 Async versus threads

- **CPU-bound work:** use threads or `rayon`'s data parallelism. `rayon` turns `iter()` into `par_iter()`:

  ```rust
  use rayon::prelude::*;
  let total: u64 = (0..1_000_000).into_par_iter().map(|i| i * i).sum();
  ```

- **I/O-bound work:** use async. Spawning a task per connection is far cheaper than a thread.
- **Mixing them:** keep blocking CPU work off the async runtime with `tokio::task::spawn_blocking`, so it does not stall the reactor.

---

## 13.9 Best Practices

1. **Benchmark before and after.** A change without a measurement is a guess.
2. **Profile the whole program**, not a micro-benchmark, when you care about end-to-end speed.
3. **Cut allocations first.** They are the easiest big win in idiomatic Rust.
4. **Respect the cache.** Contiguous, sequential, predictable access wins.
5. **Don't fight the optimizer.** Write clear, monomorphized code; `cargo build --release` does the rest.

---

## 13.10 Summary

Performance work in Rust starts with measurement: `criterion` for micro-benchmarks, `flamegraph` for the whole program. The common wins are fewer allocations, better cache layout, and the right concurrency model — threads and `rayon` for CPU work, async for I/O. Most Rust code is already fast; these techniques keep it that way as it grows.

### Exercises

1. Benchmark `Vec::push` with and without `with_capacity`, and report the difference.
2. Rewrite a struct-of-arrays example and benchmark a sum over one field against the array-of-structs version.
3. Replace a `HashMap` with an `AHashMap` in a hot loop and measure the change.
