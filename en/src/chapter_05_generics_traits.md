# Chapter 5: Generics & Traits

Generics and traits are Rust's most important abstraction mechanisms: generics let you write one piece of code for many types, and traits define what a type can do. Together they yield code that is both flexible and type-safe — and, thanks to monomorphization, with zero runtime overhead.

## Learning Objectives

- Write type-agnostic code with generic functions and structs.
- Define and implement traits, and understand default methods.
- Constrain generic types with trait bounds.
- Distinguish static dispatch (generics) from dynamic dispatch (trait objects).
- Design interfaces that fit a domain using associated types.

---

## 5.1 Generics

A generic uses a placeholder type `T` in place of a concrete type, filled in at the call site. A single `largest` works for any slice of comparable items:

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut biggest = &list[0];
    for item in &list[1..] {
        if item > biggest {
            biggest = item;
        }
    }
    biggest
}

fn main() {
    let nums = vec![3, 1, 4, 1, 5, 9, 2, 6];
    println!("largest: {}", largest(&nums));

    let chars = vec!['a', 'z', 'm'];
    println!("largest: {}", largest(&chars));
}
```

### Generic structs & enums

```rust
struct Pair<T> {
    first: T,
    second: T,
}

impl<T> Pair<T> {
    fn new(first: T, second: T) -> Self {
        Pair { first, second }
    }
}

fn main() {
    let p = Pair::new(1, 2);
    println!("{} {}", p.first, p.second);
}
```

`Option<T>`, `Result<T, E>`, and `Vec<T>` are themselves generic enums/structs.

> **Zero cost**: generics are **monomorphized** at compile time — the compiler generates a dedicated copy for each concrete type. `largest::<i32>` and `largest::<char>` are two separate functions, each inlinable, with no dispatch overhead at runtime.

---

## 5.2 Traits: What a Type Can Do

A trait defines a set of method signatures; a type provides an implementation with `impl`, declaring "I can do these things":

```rust
trait Summary {
    fn summarize(&self) -> String;

    // Default method — implementers may skip it
    fn preview(&self) -> String {
        let s = self.summarize();
        let n = s.len().min(20);
        format!("{}...", &s[..n])
    }
}

struct Article {
    title: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}: {}", self.title, self.content)
    }
}

fn main() {
    let a = Article {
        title: "Rust released".into(),
        content: "Rust 2021 edition is stable".into(),
    };
    println!("{}", a.summarize());
    println!("{}", a.preview()); // default implementation
}
```

Traits can have default implementations that implementers override as needed.

---

## 5.3 Trait Bounds: Constraining Generics

A generic `T` can do almost nothing by default. To call its methods, declare the traits it must implement with a **trait bound**:

```rust
// T: Summary + Display — T must implement both
fn report<T: Summary>(item: &T) {
    println!("report: {}", item.summarize());
}
```

### `where` clauses

With many bounds, a `where` clause is clearer:

```rust
fn merge<T, U>(a: &T, b: &U) -> String
where
    T: Summary,
    U: Summary,
{
    format!("{} | {}", a.summarize(), b.summarize())
}
```

### `impl Trait` syntax

Parameters and return values can use `impl Trait` as shorthand:

```rust
// Parameter: accept any type implementing Summary
fn report(item: &impl Summary) { /* ... */ }

// Return: return some type implementing Summary (caller need not know which)
fn make() -> impl Summary {
    Article { title: "x".into(), content: "y".into() }
}
```

> **Returning `impl Trait`**: you may return only a single concrete type. To return one of several types, use a trait object (next section).

---

## 5.4 Static vs Dynamic Dispatch

Generics with trait bounds are **static dispatch**: monomorphized at compile time, one copy per concrete type, calls are direct and inlinable. The trade-off is slightly larger binaries.

When you need to hold values of "several different types" at runtime (e.g. a `Vec` of various `Summary`), use **dynamic dispatch** — a trait object:

```rust
fn main() {
    // &dyn Summary is a trait object: dispatched via a vtable at runtime
    let items: Vec<Box<dyn Summary>> = vec![
        Box::new(Article { title: "a".into(), content: "b".into() }),
    ];
    for it in &items {
        println!("{}", it.summarize());
    }
}
```

| Form | Dispatch | Overhead | Holds many types? |
|------|----------|----------|-------------------|
| Generic `T: Trait` | static (monomorphized) | none | no (one per type) |
| `&dyn Trait` / `Box<dyn Trait>` | dynamic (vtable) | one indirect call | yes |

**Rule of thumb**: prefer generics when you can (faster); reach for `dyn` only when you need runtime polymorphism.

---

## 5.5 Associated Types

An associated type lets a trait carry a "type decided by the implementer," which often fits a domain better than a generic parameter. `Iterator` is the classic example:

```rust
trait Iterator {
    type Item;                       // associated type
    fn next(&mut self) -> Option<Self::Item>;
}

struct Counter { count: u32 }

impl Iterator for Counter {
    type Item = u32;                 // Counter yields u32
    fn next(&mut self) -> Option<u32> {
        self.count += 1;
        if self.count <= 5 { Some(self.count) } else { None }
    }
}

fn main() {
    for n in Counter { count: 0 } {
        println!("{n}");
    }
}
```

The difference from a generic parameter: a type can have only one `impl` of a trait with an associated type (the type is fixed), whereas a generic trait can have several `impl`s (one per set of type parameters). `Iterator` uses an associated type because "what an iterator yields" is fixed for a given iterator.

---

## 5.6 Summary

Generics write type-agnostic code that is monomorphized and zero-cost at compile time; traits define "what a type can do" and constrain generics via trait bounds. Static dispatch (generics) is fast but cannot be polymorphic at runtime; dynamic dispatch (`dyn Trait`) is flexible but has a vtable cost. Associated types make a trait's interface fit the domain. Together these are how Rust abstracts without losing performance.

### Exercises

1. Write a generic `fn first<T>(v: &[T]) -> Option<&T>` returning the first element of a slice.
2. Define a `Drawable` trait (`fn draw(&self)`), implement it for two different structs, and hold them in a `Vec<Box<dyn Drawable>>` to iterate.
3. Add a `take_n` method to the `Counter` above (returning `impl Iterator`) and observe how the associated type propagates.
