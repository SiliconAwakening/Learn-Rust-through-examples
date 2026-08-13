# 第4章：结构体与枚举

现实世界的数据很少是孤立的。Rust 用**结构体**把相关字段组合成自定义类型，用**枚举**表达“一个值可能是几种形态之一”。配合模式匹配，这两者让你精确建模业务领域，并让非法状态在编译期就无法表达。

## 学习目标

- 定义结构体、方法与关联函数。
- 用枚举建模“多选一”的数据，理解 `Option` 与 `Result` 也是枚举。
- 用 `match` 与 `if let` 做模式匹配。
- 用 `impl` 块给类型附加行为。

---

## 4.1 结构体

结构体把命名字段组合成一个类型。三种形式：具名结构体、元组结构体、单元结构体：

```rust
// 具名结构体——最常用
struct User {
    name: String,
    age: u32,
    active: bool,
}

// 元组结构体——字段无名字，适合轻量包装
struct Color(i32, i32, i32);

// 单元结构体——无字段，常用于 trait
struct Marker;

fn main() {
    let u = User { name: "alice".into(), age: 30, active: true };
    println!("{} {} {}", u.name, u.age, u.active);

    let c = Color(255, 128, 0);
    println!("{} {} {}", c.0, c.1, c.2);
}
```

> **字段私有性**：结构体字段默认私有。在同模块外访问字段需要 `pub`——详见第 8 章。

### 字段简写与更新语法

当变量名与字段名相同时可简写；用 `..` 复制其余字段：

```rust
fn main() {
    let name = String::from("bob");
    let u1 = User { name, age: 25, active: true }; // name 简写
    let u2 = User { age: 26, ..u1 };                // 其余字段复制自 u1
    println!("{} {}", u2.name, u2.age);
}
```

> **注意**：`..u1` 会 move 出 `u1` 的字段。`u1.name` 已被 move，`u1` 整体此后不可用（除非被复制字段都 `Copy`）。

---

## 4.2 方法与关联函数：`impl` 块

用 `impl` 给类型附加行为。`fn` 带 `&self`/`&mut self`/`self` 是方法；不带 `self` 是关联函数（类似静态方法）：

```rust
struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    // 关联函数——构造器，类似 Rectangle::new
    fn new(width: f64, height: f64) -> Self {
        Rectangle { width, height }
    }

    // 方法——借用 self
    fn area(&self) -> f64 {
        self.width * self.height
    }

    // 方法——可变借用
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

`Self` 是当前类型的别名。`impl` 块可以写多个，常用于把方法按功能分组。

---

## 4.3 枚举：多选一的值

枚举表示一个值可能是几种变体之一。Rust 的枚举很强——每个变体可以携带不同类型、不同数量的数据：

```rust
enum Message {
    Quit,                        // 无数据
    Move { x: i32, y: i32 },     // 具名字段
    Write(String),               // 一个值
    ChangeColor(i32, i32, i32),  // 元组
}

fn main() {
    let m = Message::Write("hello".into());
    process(m);
}

fn process(msg: Message) {
    // 必须处理所有变体——编译器强制穷尽
    match msg {
        Message::Quit => println!("quit"),
        Message::Move { x, y } => println!("move to {x},{y}"),
        Message::Write(text) => println!("write: {text}"),
        Message::ChangeColor(r, g, b) => println!("color {r},{g},{b}"),
    }
}
```

> **枚举 vs 结构体**：当一个值“是 A 或 B 或 C”时用枚举；当它“同时有 A 和 B 和 C”时用结构体。

### `Option<T>`：标准库的枚举

`Option` 用枚举表达“有值或无值”，取代了 null：

```rust
enum Option<T> {
    Some(T),
    None,
}
```

Rust 里没有 null——要表示可能缺失，就用 `Option<T>`，编译器强制你处理 `None`。第 6 章会展开它的错误处理用法。

---

## 4.4 模式匹配：`match`

`match` 对枚举做穷尽式分支，是 Rust 最强大的控制流之一：

```rust
fn describe(n: i32) -> &'static str {
    match n {
        0 => "零",
        1..=9 => "个位",
        10 | 20 | 30 => "整十",
        _ if n < 0 => "负数",        // 守卫
        _ => "其它",
    }
}

fn main() {
    println!("{}", describe(0));
    println!("{}", describe(7));
    println!("{}", describe(-3));
}
```

要点：

- **必须穷尽**所有可能，`_` 是通配兜底。
- 分支可以绑定变量（如 `Message::Move { x, y }` 把 `x`/`y` 绑到字段值）。
- 可以加**守卫**（`if 条件`）做额外过滤。

### `if let`：只关心一个分支

只想处理一种情况、忽略其余时，`if let` 比 `match` 简洁：

```rust
fn main() {
    let m = Message::Write("hi".into());
    if let Message::Write(text) = m {
        println!("要写: {text}");
    } else {
        println!("不是 Write");
    }
}
```

`while let` 同理，用于循环里反复解构。

---

## 4.5 实战：一个状态机

用枚举 + 模式匹配建模订单状态机——非法转换在编译期就无法写出：

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
            // 已完成或已取消——没有下一个状态
            OrderState::Delivered | OrderState::Cancelled => self,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            OrderState::Pending => "待支付",
            OrderState::Paid => "已支付",
            OrderState::Shipped => "已发货",
            OrderState::Delivered => "已送达",
            OrderState::Cancelled => "已取消",
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

这个例子体现了枚举的核心价值：**把业务规则编码进类型系统**，让“已送达又变成待支付”这种非法状态根本无法表达。

---

## 4.6 小结

结构体把相关字段组合成自定义类型，枚举表达“多选一”的值，`impl` 块附加行为，`match` 做穷尽式分支。`Option` 用枚举取代了 null，强制你处理缺失。把这些用起来，就能把业务约束编码进类型，让非法状态在编译期无处遁形——这是 Rust 类型安全的设计精髓。

### 练习

1. 定义一个 `Point` 结构体与一个 `Shape` 枚举（`Circle`、`Rectangle`、`Triangle`），用 `match` 为每种形状计算面积。
2. 给 `User` 结构体加一个 `birthday(&mut self)` 方法让 `age` 加 1，并写一个关联函数 `User::new(name, age)`。
3. 用 `Option<i32>` 写一个函数，返回列表中第一个正数；用 `if let` 处理结果。
