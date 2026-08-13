# 第5章：泛型与特征

泛型与特征是 Rust 最重要的抽象机制：泛型让你写一份代码适用于多种类型，特征定义类型能做什么。二者结合，带来“既灵活又类型安全”的代码，并且——得益于单态化——零运行时开销。

## 学习目标

- 用泛型函数、泛型结构体写出类型无关的代码。
- 定义与实现特征，理解默认方法。
- 用特征边界约束泛型类型。
- 区分静态分发（泛型）与动态分发（trait object）。
- 用关联类型设计更贴合领域的接口。

---

## 5.1 泛型

泛型用一个占位类型 `T` 代替具体类型，调用时再填入。一个 `largest` 函数对任何“可比较的切片”都适用：

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
    println!("最大: {}", largest(&nums));

    let chars = vec!['a', 'z', 'm'];
    println!("最大: {}", largest(&chars));
}
```

### 泛型结构体与枚举

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

`Option<T>`、`Result<T, E>`、`Vec<T>` 本质都是泛型枚举/结构体。

> **零成本**：泛型在编译期**单态化**——编译器为每个具体类型生成一份专用代码。`largest::<i32>` 与 `largest::<char>` 是两个独立函数，各自可内联，运行时无分发开销。

---

## 5.2 特征：类型能做什么

特征定义一组方法签名，类型通过 `impl` 提供实现，声明“我能做这些事”：

```rust
trait Summary {
    fn summarize(&self) -> String;

    // 默认方法——实现者可不重写
    fn preview(&self) -> String {
        format!("{}...", &self.summarize()[..self.summarize().len().min(20)])
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
        title: "Rust 发布".into(),
        content: "Rust 2021 edition 已稳定".into(),
    };
    println!("{}", a.summarize());
    println!("{}", a.preview()); // 用默认实现
}
```

特征可以有默认实现，实现者按需覆盖。

---

## 5.3 特征边界：约束泛型

泛型 `T` 默认什么都能做（几乎）。要调用其方法，就用**特征边界**声明 `T` 必须实现的 trait：

```rust
// T: Summary + Display —— T 必须同时实现这两个 trait
fn report<T: Summary>(item: &T) {
    println!("报告: {}", item.summarize());
}
```

### `where` 子句

边界多时，`where` 子句更清晰：

```rust
fn merge<T, U>(a: &T, b: &U) -> String
where
    T: Summary,
    U: Summary,
{
    format!("{} | {}", a.summarize(), b.summarize())
}
```

### `impl Trait` 语法

参数与返回值可用 `impl Trait` 简写：

```rust
// 参数：接任何实现了 Summary 的类型
fn report(item: &impl Summary) { /* ... */ }

// 返回：返回某个实现了 Summary 的类型（调用方无需知道具体类型）
fn make() -> impl Summary {
    Article { title: "x".into(), content: "y".into() }
}
```

> **返回 `impl Trait` 的限制**：只能返回单一具体类型。想返回多种类型要用 trait object（下节）。

---

## 5.4 静态分发 vs 动态分发

泛型 + 特征边界是**静态分发**：编译期单态化，每个具体类型一份代码，调用直接、可内联。代价是二进制体积略大。

当你需要在运行时持有“多种不同类型”的值（如一个 `Vec` 装不同种 `Summary`），就要**动态分发**——trait object：

```rust
fn main() {
    // &dyn Summary 是 trait object：运行时通过虚表分发
    let items: Vec<Box<dyn Summary>> = vec![
        Box::new(Article { title: "a".into(), content: "b".into() }),
    ];
    for it in &items {
        println!("{}", it.summarize());
    }
}
```

| 方式 | 分发 | 开销 | 能否装多种类型 |
|------|------|------|----------------|
| 泛型 `T: Trait` | 静态（单态化） | 无 | 否（一类型一份） |
| `&dyn Trait` / `Box<dyn Trait>` | 动态（虚表） | 一次间接调用 | 是 |

**经验法则**：能用泛型就用泛型（更快）；必须运行时多态才用 `dyn`。

---

## 5.5 关联类型

关联类型让 trait 持有一个“由实现者决定”的类型位，比泛型参数更贴合领域语义。`Iterator` 是经典例子：

```rust
trait Iterator {
    type Item;                       // 关联类型
    fn next(&mut self) -> Option<Self::Item>;
}

struct Counter { count: u32 }

impl Iterator for Counter {
    type Item = u32;                 // Counter 产出 u32
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

关联类型与泛型参数的区别：一个类型对一个 trait 只能有一份 `impl`（关联类型固定），而泛型 trait 可以有多份 `impl`（每套类型参数一份）。`Iterator` 用关联类型，因为“一个迭代器产出什么”是确定的。

---

## 5.6 小结

泛型用占位类型写出类型无关代码，编译期单态化、零成本；特征定义“类型能做什么”，用特征边界约束泛型。静态分发（泛型）快但不能运行时多态，动态分发（`dyn Trait`）灵活但有虚表开销。关联类型让 trait 的接口更贴合领域。这套机制是 Rust 既能抽象又不损失性能的关键。

### 练习

1. 写一个泛型函数 `fn first<T>(v: &[T]) -> Option<&T>`，返回切片首元素。
2. 定义一个 `Drawable` trait（`fn draw(&self)`），为两个不同结构体实现它，用 `Vec<Box<dyn Drawable>>` 持有并遍历。
3. 给上面的 `Counter` 加一个 `take_n` 方法（用 `impl Iterator` 返回值），体会关联类型如何随迭代器传播。
