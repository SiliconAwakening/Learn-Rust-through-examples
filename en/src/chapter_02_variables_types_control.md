# Chapter 2: Variables, Data Types & Control Flow

This chapter is the grammar of Rust: how to declare variables, what data types exist, and how to organize logic with control flow and functions. These are the bedrock for every later chapter — especially the "immutable by default" decision, which runs through every line of Rust you will write.

## Learning Objectives

- Declare variables with `let`, understand `mut` vs immutable, and shadowing.
- Master scalar types (integers, floats, booleans, chars) and compound types (tuples, arrays).
- Understand strings: the difference between `String` and `&str`.
- Use `if`/`loop`/`while`/`for` and pattern matching for control flow.
- Define functions and understand expression semantics and return values.

---

## 2.1 Variables & Mutability

Rust declares variables with `let`, **immutable by default**:

```rust
fn main() {
    let x = 5;
    // x = 6; // error: x is immutable
    println!("{x}");

    let mut y = 5;
    y = 6;       // OK: y declared mut
    println!("{y}");
}
```

Immutability by default is deliberate: it makes code predictable and lets the compiler optimize more. When you do need to change a value, write `mut` explicitly — a signal that "state changes here."

### Shadowing

You can re-declare a variable with the same name; the new one shadows the old. Shadowing can even change the type:

```rust
fn main() {
    let x = 5;
    let x = x + 1;        // compute from the old value
    let x = x * 2;        // {x} = 12

    let spaces = "   ";   // &str
    let spaces = spaces.len(); // usize — type changed too
    println!("{x} {spaces}");
}
```

> **`mut` vs shadowing**: `mut` changes the same variable's value and cannot change its type; shadowing creates a new variable and can change the type. Turning a string into its length is natural with shadowing and impossible with `mut`.

### Constants

`const` differs from an immutable variable: it is evaluated at compile time, requires a type annotation, is uppercase, and can be declared in any scope:

```rust
const MAX_POINTS: u32 = 100_000;
```

---

## 2.2 Scalar Types

| Type | Meaning | Example |
|------|---------|---------|
| `i8`…`i128`, `isize` | signed integer | `-5`, `42` |
| `u8`…`u128`, `usize` | unsigned integer | `0`, `255` |
| `f32`, `f64` | float | `3.14`, `2.0` |
| `bool` | boolean | `true`, `false` |
| `char` | Unicode scalar value (4 bytes) | `'A'`, `'中'`, `'🦀'` |

```rust
fn main() {
    let a: i32 = -42;
    let b: u64 = 1_000_000;   // underscores for readability
    let c: f64 = 2.71828;
    let flag: bool = true;
    let heart: char = '🦀';
    println!("{a} {b} {c} {flag} {heart}");
}
```

> **Integer literals**: `42` defaults to `i32`. Annotate when the context needs another type: `let n: u8 = 42;`. Integer overflow panics in debug builds and wraps in release — use `checked_*`, `wrapping_*`, or `saturating_*` methods to handle it explicitly when it matters.

---

## 2.3 Compound Types: Tuples & Arrays

A **tuple** groups values of different types, fixed length:

```rust
fn main() {
    let tup: (i32, f64, &str) = (500, 6.4, "hello");
    let (x, _, s) = tup;       // destructure
    println!("{x} {s}");
    println!("{}", tup.0);     // index access
}
```

An **array** is fixed-length, same-type, contiguous on the stack:

```rust
fn main() {
    let arr = [1, 2, 3, 4, 5];
    let zeros = [0; 10];       // ten 0s
    println!("first = {}, len = {}", arr[0], arr.len());

    // Out-of-bounds access panics at runtime (debug build) —
    // it does not read past the end like C would.
    // let oob = arr[10]; // panic
}
```

> **Arrays vs `Vec`**: arrays have a compile-time-fixed length and suit small, known collections; for runtime-growable data use `Vec` (Chapter 7).

---

## 2.4 Strings: `String` vs `&str`

Rust has two string types that trip up beginners:

- **`&str`**: a string slice — a borrow of UTF-8 bytes somewhere. A literal `"hello"` is a `&'static str`.
- **`String`**: heap-allocated, growable, owned.

```rust
fn main() {
    let literal: &str = "hello";            // borrowed, immutable
    let mut owned = String::from("hello");  // heap, growable
    owned.push_str(", world");
    owned.push('!');

    // Conversions
    let from_slice: String = literal.to_string();
    let to_slice: &str = &owned;

    println!("{owned}  {from_slice}  {to_slice}");
}
```

**Rule of thumb**: prefer `&str` for function parameters (accepts both `&str` and `&String`); use `String` when you need to own, mutate, or return it.

---

## 2.5 Control Flow

### `if` is an expression

`if` yields a value; all branches must have the same type:

```rust
fn main() {
    let n = 7;
    let label = if n % 2 == 0 { "even" } else { "odd" };
    println!("{label}");

    if n > 10 {
        println!("big");
    } else if n > 3 {
        println!("medium");
    } else {
        println!("small");
    }
}
```

### Loops: `loop`, `while`, `for`

```rust
fn main() {
    // loop: infinite loop, break can return a value
    let mut count = 0;
    let result = loop {
        count += 1;
        if count == 10 { break count * 2; }
    };
    println!("{result}"); // 20

    // while: conditional loop
    let mut n = 3;
    while n > 0 { n -= 1; }

    // for: iterate a collection — the most common
    for x in [1, 2, 3] {
        println!("{x}");
    }
    for i in 0..5 { print!("{i} "); }      // 0 1 2 3 4
    for i in (1..=3).rev() { print!("{i} "); } // 3 2 1
}
```

Ranges come as `a..b` (half-open) and `a..=b` (inclusive). Indexed `while` loops are rare in Rust — iterators are safer and clearer.

---

## 2.6 Functions

Functions are defined with `fn`; parameters need type annotations. Rust is an **expression language**: without `return`, the last expression (no semicolon) is the return value:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b          // expression — the return value
}

fn greet(name: &str) {   // no -> means returns the unit type ()
    println!("hi, {name}");
}

fn abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }   // an if expression as the return value
}

fn main() {
    greet("alice");
    println!("{} {}", add(2, 3), abs(-7));
}
```

> **Statements vs expressions**: `let x = 5;` is a statement (no value); `x + 1` is an expression (has a value). Adding a semicolon turns an expression into a statement — and drops its value. The common "missing return value" error is usually a stray semicolon.

### Diverging functions

Functions that never return are typed `-> !`:

```rust
fn forever() -> ! {
    loop {}
}
```

---

## 2.7 Summary

Rust variables are immutable by default; use `mut` when you need to change them, and shadowing to reuse a name or even change its type. Scalars and compound types are the foundation; for strings, distinguish owned `String` from borrowed `&str`. `if` and `loop` are expressions, and a function returns its last semicolon-free expression. These rules are simple yet underpin every later topic — ownership, generics, error handling.

### Exercises

1. Write `fn fizzbuzz(n: u32)` that prints 1 to n by the classic FizzBuzz rules.
2. Return both quotient and remainder from one function: `fn divmod(a: i32, b: i32) -> (i32, i32)`.
3. Sum the integers 1 to 100 with a `for` and a range, and note why an indexed `while` is unnecessary.
