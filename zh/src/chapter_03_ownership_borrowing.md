# 第3章：所有权与借用

所有权是 Rust 最独特的特性，也是它无需垃圾回收却保证内存安全的根基。本章讲清三条所有权规则、借用与借用检查器、生命周期，以及切片。理解了这一章，你就能读懂编译器的报错，并明白它为何这样要求你。

## 学习目标

- 掌握所有权的三条规则与 move 语义。
- 理解借用：共享引用 `&T` 与可变引用 `&mut T`。
- 掌握借用的规则与“同一时刻多个引用”的限制。
- 用生命周期标注让引用之间的关系显式化。
- 用切片 `&[T]` / `&str` 借用一段连续数据。

---

## 3.1 所有权的三条规则

Rust 内存管理的核心是三条规则：

1. **每个值有且仅有一个所有者**——变量。
2. **当所有者离开作用域，值被丢弃**（析构运行，内存释放）。
3. **赋值或传参时，所有权转移（move）**——除非类型实现了 `Copy`。

```rust
fn main() {
    {
        let s = String::from("hello"); // s 是所有者
        println!("{s}");
    } // s 离开作用域，String 的内存自动释放——无需 free

    let s1 = String::from("hello");
    let s2 = s1;            // 所有权从 s1 move 到 s2
    // println!("{s1}");    // 错误：s1 已被 move，不再有效
    println!("{s2}");
}
```

### Move 与 `Copy`

`String` 拥有堆内存，赋值时是 **move**——旧变量失效，避免双重释放。栈上的简单类型（整数、布尔、字符、固定大小数组等）实现了 `Copy` trait，赋值时是**按位拷贝**，旧变量仍可用：

```rust
fn main() {
    let a = 5;
    let b = a;       // i32 是 Copy，a 仍可用
    println!("{a} {b}");

    let s1 = String::from("hi");
    let s2 = s1;     // String 不是 Copy，s1 被 move
    // println!("{s1}"); // 错误
}
```

> **函数传参也是 move**：把 `String` 传给函数后，调用方就不能再用它。想要“借用而不转移所有权”，就用下一节的引用。

---

## 3.2 借用与引用

借用让函数使用值而不获取所有权。`&T` 是共享引用（只读），`&mut T` 是可变引用：

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
} // s 是借用，这里不释放任何东西

fn append(s: &mut String) {
    s.push_str("!");
}

fn main() {
    let mut s = String::from("hello");
    let len = calculate_length(&s);   // 借用，s 仍归 main 所有
    println!("{s} 长度 {len}");

    append(&mut s);
    println!("{s}"); // hello!
}
```

### 借用的两条规则

借用检查器在编译期强制两条规则：

1. **任意时刻，可以有多个共享引用 `&T`，或者一个可变引用 `&mut T`，二者不能并存。**
2. **引用必须始终有效**（不能悬垂）。

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &s;       // OK：多个共享引用
// let r3 = &mut s; // 错误：已有共享引用时不能再借可变
println!("{r1} {r2}");

let mut s = String::from("hi");
let r1 = &mut s;
// let r2 = &mut s; // 错误：同一时刻只能一个可变引用
println!("{r1}");
```

> **为何这么严格？** 多个可变引用混用正是数据竞争与迭代器失效的根源。编译期拒绝它们，就把一整类并发 bug 提前消灭了。

### NLL：非词法生命周期

借用检查器较新（NLL，Non-Lexical Lifetimes）会看引用**实际**最后一次使用的位置，而非作用域结尾：

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &s;
println!("{r1} {r2}");
// r1、r2 此后不再使用
let r3 = &mut s;   // OK：旧的共享引用已不再需要
println!("{r3}");
```

---

## 3.3 悬垂引用

函数不能返回对局部变量的引用——变量离开函数就被释放，引用会悬垂。编译器会拒绝：

```rust
// fn dangle() -> &String {
//     let s = String::from("hi");
//     &s
// } // 错误：s 在此处释放，返回的引用会悬垂

// 正确做法：直接返回 String，转移所有权
fn no_dangle() -> String {
    let s = String::from("hi");
    s
}
```

---

## 3.4 生命周期

当引用来自多个地方、编译器无法推断它们谁活得更久时，需要**生命周期标注**显式说明关系。标注不改变引用实际寿命，只是声明约束。

```rust
// 'a 表示：返回的引用至少和 x、y 中较短的那个一样长
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let s1 = String::from("long string");
    let s2 = String::from("hi");
    let result = longest(s1.as_str(), s2.as_str());
    println!("更长的是: {result}");
}
```

### 函数中的生命周期省略规则

多数情况下不必手写标注。编译器按三条**省略规则**自动补：

1. 每个引用参数各自获得一个生命周期。
2. 若只有一个输入生命周期，它赋给所有输出引用。
3. 若有 `&self`/`&mut self`，`self` 的生命周期赋给所有输出引用。

不满足时编译器会报错，要求你显式标注——这通常意味着你的 API 需要重新考虑。

### 结构体里的生命周期

结构体持有引用时必须标注：

```rust
struct Excerpt<'a> {
    part: &'a str,
}

fn main() {
    let novel = String::from("call me Ishmael. Some years ago...");
    let first = novel.split('.').next().unwrap();
    let e = Excerpt { part: first };
    println!("{:?}", e.part);
}
```

`'a` 表示 `Excerpt` 不能比它借用的字符串活得更久。

---

## 3.5 切片

切片是对连续序列的借用，不拥有数据。`&[T]` 是数组/Vec 的切片，`&str` 是字符串切片：

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
    let word = first_word(&s);   // word 借用 s
    println!("{word}");
    // 若这里修改 s，借用检查器会因 word 仍存活而拒绝
}
```

切片让函数同时适用于 `&String`、`&str`、`&[T]`、`&Vec<T>`——这是 Rust 写出通用代码的关键之一。

---

## 3.6 一个完整例子：所有权流转

把所有权、借用、切片串起来——一个不用拷贝即可统计文本最长行的函数：

```rust
fn longest_line<'a>(lines: &'a [&'a str]) -> Option<&'a str> {
    lines.iter().copied().max_by_key(|l| l.len())
}

fn main() {
    let text = ["short", "a longer line", "mid"];
    if let Some(longest) = longest_line(&text) {
        println!("最长: {longest}");
    }
}
```

`longest_line` 只借用切片、返回借用，没有发生任何堆分配。这正是 Rust 零成本抽象的写照。

---

## 3.7 小结

所有权三规则、move 语义、借用的两条铁律、生命周期标注、切片——这些构成了 Rust 内存安全的骨架。借用检查器看似严苛，实则消灭了空指针、悬垂引用、双重释放与数据竞争。它拒绝的不是你的意图，而是你代码里潜藏的 bug。掌握本章，你就跨过了 Rust 最陡的那道坎。

### 练习

1. 解释为何 `let s2 = s1;`（`s1` 是 `String`）后 `s1` 失效，而 `let b = a;`（`a` 是 `i32`）后 `a` 仍可用。
2. 写一个函数 `fn longest_word(s: &str) -> &str`，返回输入中第一个最长的单词（用空格分隔）。标注需要的生命周期，体会省略规则。
3. 写一个持有 `&str` 的结构体 `Config<'a>`，并构造一个实例，验证它不能比借用的 `String` 活得更久。
