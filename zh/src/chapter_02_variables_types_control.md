# 第2章：变量、数据类型与控制流

本章是 Rust 语法的基本功：如何声明变量、有哪些数据类型、如何用控制流与函数组织逻辑。这些是后面所有章节的基石——尤其是“默认不可变”这一设计决定，它会贯穿你写的每一行 Rust。

## 学习目标

- 用 `let` 声明变量，理解可变（`mut`）与不可变、以及变量遮蔽。
- 掌握标量类型（整数、浮点、布尔、字符）与复合类型（元组、数组）。
- 理解字符串：`String` 与 `&str` 的区别。
- 用 `if`/`loop`/`while`/`for` 与模式匹配写控制流。
- 定义函数，理解表达式语义与返回值。

---

## 2.1 变量与可变性

Rust 用 `let` 声明变量，**默认不可变**：

```rust
fn main() {
    let x = 5;
    // x = 6; // 错误：x 不可变
    println!("{x}");

    let mut y = 5;
    y = 6;       // OK：y 用 mut 声明，可变
    println!("{y}");
}
```

默认不可变是有意为之：它让代码更可预测，编译器也能据此做更多优化。当你确实需要修改时，显式写 `mut`——它是一个“这里状态会变”的信号。

### 变量遮蔽（shadowing）

可以用同一个名字重新声明变量，新变量会遮蔽旧变量。遮蔽还能改变类型：

```rust
fn main() {
    let x = 5;
    let x = x + 1;        // 用旧值算新值
    let x = x * 2;        // {x} = 12

    let spaces = "   ";   // &str
    let spaces = spaces.len(); // usize——类型也变了
    println!("{x} {spaces}");
}
```

> **`mut` vs 遮蔽**：`mut` 改的是同一个变量的值，类型不能变；遮蔽是新建一个变量，可以变类型。把字符串转成长度时用遮蔽很自然，用 `mut` 做不到。

### 常量

`const` 与不可变变量不同：它编译期求值、必须标注类型、全大写、可在任意作用域声明：

```rust
const MAX_POINTS: u32 = 100_000;
```

---

## 2.2 标量类型

| 类型 | 含义 | 示例 |
|------|------|------|
| `i8`…`i128`, `isize` | 有符号整数 | `-5`, `42` |
| `u8`…`u128`, `usize` | 无符号整数 | `0`, `255` |
| `f32`, `f64` | 浮点数 | `3.14`, `2.0` |
| `bool` | 布尔 | `true`, `false` |
| `char` | Unicode 标量值（4 字节） | `'A'`, `'中'`, `'🦀'` |

```rust
fn main() {
    let a: i32 = -42;
    let b: u64 = 1_000_000;   // 下划线分隔，提升可读性
    let c: f64 = 2.71828;
    let flag: bool = true;
    let heart: char = '🦀';
    println!("{a} {b} {c} {flag} {heart}");
}
```

> **整数字面量的类型**：`42` 默认推断为 `i32`。若上下文需要别的类型，标注即可：`let n: u8 = 42;`。整数溢出在调试构建会 panic，在 release 构建会回绕——要紧时用 `checked_*`、`wrapping_*`、`saturating_*` 方法显式处理。

---

## 2.3 复合类型：元组与数组

**元组**（tuple）把多个不同类型的值固定在一起，长度不可变：

```rust
fn main() {
    let tup: (i32, f64, &str) = (500, 6.4, "hello");
    let (x, _, s) = tup;       // 解构
    println!("{x} {s}");
    println!("{}", tup.0);     // 索引访问
}
```

**数组**（array）是固定长度、同类型、栈上连续存储：

```rust
fn main() {
    let arr = [1, 2, 3, 4, 5];
    let zeros = [0; 10];       // 10 个 0
    println!("first = {}, len = {}", arr[0], arr.len());

    // 越界访问在运行时 panic（调试构建），不会像 C 那样读越界内存
    // let oob = arr[10]; // panic
}
```

> **数组 vs `Vec`**：数组长度编译期固定，适合小而确定的集合；运行时可增长用 `Vec`（第 7 章）。

---

## 2.4 字符串：`String` 与 `&str`

Rust 有两种字符串，初学者常被绊倒：

- **`&str`**：字符串切片，是对某处 UTF-8 字节序列的借用。字面量 `"hello"` 是 `&'static str`。
- **`String`**：堆分配、可增长、 owned 的字符串。

```rust
fn main() {
    let literal: &str = "hello";        // 借用，不可变
    let mut owned = String::from("hello"); // 堆上，可增长
    owned.push_str(", world");
    owned.push('!');

    // 互转
    let from_slice: String = literal.to_string();
    let to_slice: &str = &owned;

    println!("{owned}  {from_slice}  {to_slice}");
}
```

**经验法则**：函数参数优先用 `&str`（既能接 `&str` 也能接 `&String`）；需要拥有、修改或返回时用 `String`。

---

## 2.5 控制流

### `if` 是表达式

`if` 有返回值，所有分支类型必须一致：

```rust
fn main() {
    let n = 7;
    let label = if n % 2 == 0 { "偶" } else { "奇" };
    println!("{label}");

    if n > 10 {
        println!("大");
    } else if n > 3 {
        println!("中");
    } else {
        println!("小");
    }
}
```

### 循环：`loop`、`while`、`for`

```rust
fn main() {
    // loop：无限循环，可用 break 返回值
    let mut count = 0;
    let result = loop {
        count += 1;
        if count == 10 { break count * 2; }
    };
    println!("{result}"); // 20

    // while：条件循环
    let mut n = 3;
    while n > 0 { n -= 1; }

    // for：迭代集合，最常用
    for x in [1, 2, 3] {
        println!("{x}");
    }
    for i in 0..5 { print!("{i} "); }   // 0 1 2 3 4
    for i in (1..=3).rev() { print!("{i} "); } // 3 2 1
}
```

`for` 配合范围 `a..b`（半开）与 `a..=b`（闭区间）使用。Rust 里几乎不用 `while` 索引循环——用迭代器更安全。

---

## 2.6 函数

函数用 `fn` 定义，参数需标注类型。Rust 是**表达式语言**：不写 `return` 时，最后一个表达式（无分号）就是返回值：

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b          // 表达式，是返回值
}

fn greet(name: &str) {   // 无 -> 表示返回单元类型 ()
    println!("hi, {name}");
}

fn abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }   // if 表达式作为返回值
}

fn main() {
    greet("alice");
    println!("{} {}", add(2, 3), abs(-7));
}
```

> **语句 vs 表达式**：`let x = 5;` 是语句（无值）；`x + 1` 是表达式（有值）。函数体里加分号就把表达式变成了语句——返回值就丢了。初学者常见的“漏掉返回值”错误，多半是多写了一个分号。

### 语句与发散函数

不返回值的函数返回单元类型 `()`。永不返回的函数标 `-> !`（发散）：

```rust
fn forever() -> ! {
    loop {}
}
```

---

## 2.7 小结

Rust 变量默认不可变，需要变时显式 `mut`；遮蔽允许复用名字甚至改类型。标量与复合类型是基础，字符串要分清 `String`（拥有）与 `&str`（借用）。`if` 与 `loop` 是表达式，函数以最后一个无分号表达式作返回值。这些规则简单，却支撑了后面所有权、泛型、错误处理等所有主题。

### 练习

1. 写一个函数 `fizzbuzz(n: u32)`，按经典 FizzBuzz 规则打印 1 到 n。
2. 用元组从一个函数同时返回商与余数：`fn divmod(a: i32, b: i32) -> (i32, i32)`。
3. 用 `for` 与范围计算 1 到 100 的整数和，体会为何不必用 `while` 索引循环。
