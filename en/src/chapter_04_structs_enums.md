# Chapter 4: Structs & Enums

Real-world data is rarely isolated. Rust uses **structs** to bundle related fields into a custom type and **enums** to express "a value may be one of several forms." Paired with pattern matching, these let you model a domain precisely — and make illegal states unrepresentable at compile time.

## Learning Objectives

- Define structs, methods, and associated functions.
- Model "one-of" data with enums, and understand `Option` and `Result` as enums.
- Use `match` and `if let` for pattern matching.
- Attach behavior to types with `impl` blocks.

---

## 4.1 Structs

A struct groups named fields into a type. Three forms: named-field, tuple, and unit:

```rust
// Named-field — most common
struct User {
    name: String,
    age: u32,
    active: bool,
}

// Tuple struct — fields unnamed, lightweight wrapper
struct Color(i32, i32, i32);

// Unit struct — no fields, often used with traits
struct Marker;

fn main() {
    let u = User { name: "alice".into(), age: 30, active: true };
    println!("{} {} {}", u.name, u.age, u.active);

    let c = Color(255, 128, 0);
    println!("{} {} {}", c.0, c.1, c.2);
}
```

> **Field privacy**: struct fields are private by default; accessing them outside their module requires `pub` (Chapter 8).

### Field shorthand & update syntax

When a variable name matches a field name you can shorthand; `..` copies the rest:

```rust
fn main() {
    let name = String::from("bob");
    let u1 = User { name, age: 25, active: true }; // name shorthand
    let u2 = User { age: 26, ..u1 };                // rest copied from u1
    println!("{} {}", u2.name, u2.age);
}
```

> **Note**: `..u1` moves fields out of `u1`. Since `u1.name` is moved, `u1` as a whole is no longer usable (unless all copied fields are `Copy`).

---

## 4.2 Methods & Associated Functions: `impl`

Use `impl` to attach behavior. An `fn` with `&self`/`&mut self`/`self` is a method; without `self` it is an associated function (like a static method):

```rust
struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    // Associated function — a constructor, like Rectangle::new
    fn new(width: f64, height: f64) -> Self {
        Rectangle { width, height }
    }

    // Method — borrows self
    fn area(&self) -> f64 {
        self.width * self.height
    }

    // Method — mutable borrow
    fn scale(&mut self, factor: f64) {
        self.width *= factor;
        self.height *= factor;
    }
}

fn main() {
    let mut r = Rectangle::new(3.0, 4.0);
    println!("area = {}", r.area()); // 12
    r.scale(2.0);
    println!("area = {}", r.area()); // 48
}
```

`Self` is an alias for the current type. You may write multiple `impl` blocks, often to group methods by concern.

---

## 4.3 Enums: One-of Values

An enum represents a value that may be one of several variants. Rust's enums are powerful — each variant can carry different types and amounts of data:

```rust
enum Message {
    Quit,                        // no data
    Move { x: i32, y: i32 },     // named fields
    Write(String),               // one value
    ChangeColor(i32, i32, i32),  // tuple
}

fn main() {
    let m = Message::Write("hello".into());
    process(m);
}

fn process(msg: Message) {
    // Must handle every variant — the compiler enforces exhaustiveness
    match msg {
        Message::Quit => println!("quit"),
        Message::Move { x, y } => println!("move to {x},{y}"),
        Message::Write(text) => println!("write: {text}"),
        Message::ChangeColor(r, g, b) => println!("color {r},{g},{b}"),
    }
}
```

> **Enum vs struct**: use an enum when a value "is A or B or C"; use a struct when it "has A and B and C."

### `Option<T>`: a standard-library enum

`Option` uses an enum to express "a value or nothing," replacing null:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

There is no null in Rust — to represent possible absence, use `Option<T>`, and the compiler forces you to handle `None`. Chapter 6 expands on its role in error handling.

---

## 4.4 Pattern Matching: `match`

`match` does exhaustive branching on an enum — one of Rust's most powerful control-flow constructs:

```rust
fn describe(n: i32) -> &'static str {
    match n {
        0 => "zero",
        1..=9 => "single digit",
        10 | 20 | 30 => "round ten",
        _ if n < 0 => "negative",        // guard
        _ => "other",
    }
}

fn main() {
    println!("{}", describe(0));
    println!("{}", describe(7));
    println!("{}", describe(-3));
}
```

Key points:

- **Must be exhaustive**; `_` is the catch-all.
- Arms can bind variables (e.g. `Message::Move { x, y }` binds `x`/`y` to the field values).
- You can add a **guard** (`if condition`) for extra filtering.

### `if let`: caring about one arm

When you want to handle only one case and ignore the rest, `if let` is more concise than `match`:

```rust
fn main() {
    let m = Message::Write("hi".into());
    if let Message::Write(text) = m {
        println!("writing: {text}");
    } else {
        println!("not Write");
    }
}
```

`while let` works similarly, for repeated deconstruction in a loop.

---

## 4.5 Worked Example: A State Machine

Model an order state machine with an enum and pattern matching — an illegal transition cannot even be written:

```rust
enum OrderState {
    Pending,
    Paid,
    Shipped,
    Delivered,
    Cancelled,
}

impl OrderState {
    fn next(self) -> OrderState {
        match self {
            OrderState::Pending => OrderState::Paid,
            OrderState::Paid => OrderState::Shipped,
            OrderState::Shipped => OrderState::Delivered,
            // Delivered or Cancelled — no next state
            OrderState::Delivered | OrderState::Cancelled => self,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            OrderState::Pending => "pending",
            OrderState::Paid => "paid",
            OrderState::Shipped => "shipped",
            OrderState::Delivered => "delivered",
            OrderState::Cancelled => "cancelled",
        }
    }
}

fn main() {
    let mut s = OrderState::Pending;
    for _ in 0..4 {
        println!("{}", s.label());
        s = s.next();
    }
}
```

This shows the core value of enums: **encode business rules into the type system**, so "delivered then back to pending" cannot be expressed at all.

---

## 4.6 Summary

Structs bundle related fields into custom types; enums express "one-of" values; `impl` blocks attach behavior; `match` does exhaustive branching. `Option` replaces null and forces you to handle absence. Used together, they encode business constraints into types — making illegal states unrepresentable, which is the heart of Rust's type-safe design.

### Exercises

1. Define a `Point` struct and a `Shape` enum (`Circle`, `Rectangle`, `Triangle`); use `match` to compute each shape's area.
2. Add a `birthday(&mut self)` method to `User` that increments `age`, and an associated function `User::new(name, age)`.
3. Write a function returning the first positive number from a list as `Option<i32>`, and handle the result with `if let`.
