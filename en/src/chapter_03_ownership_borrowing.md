# Chapter 3: Ownership & Borrowing

Ownership is Rust's most distinctive feature and the foundation of its memory safety without a garbage collector. This chapter covers the three ownership rules, borrowing and the borrow checker, lifetimes, and slices. Once you understand it, you can read the compiler's errors and see why it asks what it asks.

## Learning Objectives

- Master the three rules of ownership and move semantics.
- Understand borrowing: shared references `&T` and mutable references `&mut T`.
- Know the borrowing rules and the "many references at once" restriction.
- Use lifetime annotations to make reference relationships explicit.
- Use slices `&[T]` / `&str` to borrow a span of contiguous data.

---

## 3.1 The Three Rules of Ownership

Rust's memory management rests on three rules:

1. **Each value has a single owner** — a variable.
2. **When the owner goes out of scope, the value is dropped** (its destructor runs, memory is freed).
3. **Assignment or passing to a function moves ownership** — unless the type implements `Copy`.

```rust
fn main() {
    {
        let s = String::from("hello"); // s owns it
        println!("{s}");
    } // s goes out of scope; the String's memory is freed — no free needed

    let s1 = String::from("hello");
    let s2 = s1;            // ownership moves from s1 to s2
    // println!("{s1}");    // error: s1 was moved, no longer valid
    println!("{s2}");
}
```

### Move vs `Copy`

`String` owns heap memory, so assignment is a **move** — the old variable is invalidated, avoiding a double free. Stack types (integers, booleans, chars, fixed-size arrays) implement the `Copy` trait, so assignment is a **bitwise copy** and the old variable stays usable:

```rust
fn main() {
    let a = 5;
    let b = a;       // i32 is Copy; a still usable
    println!("{a} {b}");

    let s1 = String::from("hi");
    let s2 = s1;     // String is not Copy; s1 is moved
    // println!("{s1}"); // error
}
```

> **Passing to a function is a move too**: after you pass a `String` to a function, the caller can no longer use it. To "lend without transferring ownership," use references (next section).

---

## 3.2 Borrowing & References

Borrowing lets a function use a value without taking ownership. `&T` is a shared reference (read-only); `&mut T` is a mutable reference:

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
} // s is a borrow; nothing is freed here

fn append(s: &mut String) {
    s.push_str("!");
}

fn main() {
    let mut s = String::from("hello");
    let len = calculate_length(&s);   // borrow; s still owned by main
    println!("{s} length {len}");

    append(&mut s);
    println!("{s}"); // hello!
}
```

### The Two Borrowing Rules

The borrow checker enforces two rules at compile time:

1. **At any moment, you may have either several shared references `&T`, or exactly one mutable reference `&mut T` — not both.**
2. **References must always be valid** (never dangling).

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &s;       // OK: several shared references
// let r3 = &mut s; // error: cannot borrow mutably while shared refs exist
println!("{r1} {r2}");

let mut s = String::from("hi");
let r1 = &mut s;
// let r2 = &mut s; // error: only one mutable reference at a time
println!("{r1}");
```

> **Why so strict?** Mixing mutable references is exactly what causes data races and iterator invalidation. Rejecting them at compile time eliminates a whole class of concurrency bugs up front.

### NLL: Non-Lexical Lifetimes

The modern borrow checker (NLL) looks at where a reference is **actually** last used, not the end of its scope:

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &s;
println!("{r1} {r2}");
// r1, r2 are no longer used after this point
let r3 = &mut s;   // OK: the old shared refs are no longer needed
println!("{r3}");
```

---

## 3.3 Dangling References

A function cannot return a reference to a local variable — the variable is freed when the function returns, leaving the reference dangling. The compiler rejects it:

```rust
// fn dangle() -> &String {
//     let s = String::from("hi");
//     &s
// } // error: s is freed here; the returned reference would dangle

// Correct: return the String, transferring ownership
fn no_dangle() -> String {
    let s = String::from("hi");
    s
}
```

---

## 3.4 Lifetimes

When references come from several places and the compiler cannot infer which lives longest, **lifetime annotations** spell out the relationship. They do not change how long a reference lives; they only declare constraints.

```rust
// 'a means: the returned reference lives at least as long as the shorter of x and y
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let s1 = String::from("long string");
    let s2 = String::from("hi");
    let result = longest(s1.as_str(), s2.as_str());
    println!("longer: {result}");
}
```

### Lifetime elision

Most of the time you need not write annotations. The compiler applies three **elision rules** automatically:

1. Each reference parameter gets its own lifetime.
2. If there is exactly one input lifetime, it is assigned to all output references.
3. If there is a `&self`/`&mut self`, `self`'s lifetime is assigned to all output references.

When these do not apply, the compiler errors and asks you to annotate explicitly — usually a sign your API needs rethinking.

### Lifetimes in structs

A struct holding a reference must annotate it:

```rust
struct Excerpt<'a> {
    part: &'a str,
}

fn main() {
    let novel = String::from("call me Ishmael. some years ago...");
    let first = novel.split('.').next().unwrap();
    let e = Excerpt { part: first };
    println!("{:?}", e.part);
}
```

`'a` says an `Excerpt` cannot outlive the string it borrows.

---

## 3.5 Slices

A slice is a borrow of a contiguous sequence, without ownership. `&[T]` is a slice of an array/Vec; `&str` is a string slice:

```rust
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b' ' { return &s[..i]; }
    }
    &s[..]
}

fn main() {
    let s = String::from("hello world");
    let word = first_word(&s);   // word borrows s
    println!("{word}");
    // If you mutated s here, the borrow checker would reject it because word is still alive.
}
```

Slices let one function work for `&String`, `&str`, `&[T]`, and `&Vec<T>` — a key to writing general Rust code.

---

## 3.6 Putting It Together

Tying ownership, borrowing, and slices together — a function that finds the longest line with no allocation:

```rust
fn longest_line<'a>(lines: &'a [&'a str]) -> Option<&'a str> {
    lines.iter().copied().max_by_key(|l| l.len())
}

fn main() {
    let text = ["short", "a longer line", "mid"];
    if let Some(longest) = longest_line(&text) {
        println!("longest: {longest}");
    }
}
```

`longest_line` only borrows the slice and returns a borrow — no heap allocation at all. That is zero-cost abstraction in action.

---

## 3.7 Summary

The three ownership rules, move semantics, the two borrowing rules, lifetime annotations, and slices form the skeleton of Rust's memory safety. The borrow checker is strict, but it eliminates null pointers, dangling references, double frees, and data races. It rejects not your intent but the bugs hiding in your code. Master this chapter and you have crossed Rust's steepest hill.

### Exercises

1. Explain why `let s2 = s1;` (`s1` is `String`) invalidates `s1`, while `let b = a;` (`a` is `i32`) leaves `a` usable.
2. Write `fn longest_word(s: &str) -> &str` returning the first longest word (split on spaces). Annotate any lifetimes needed and observe the elision rules.
3. Write a struct `Config<'a>` holding a `&str` and construct an instance; verify it cannot outlive the `String` it borrows.
