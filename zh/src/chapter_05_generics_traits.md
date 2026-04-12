# 第五章：泛型与特征

## 5.1 章节概述

泛型（Generics）和特征（Traits）是Rust语言中最重要的抽象机制之一。它们允许我们编写既灵活又类型安全的代码，通过抽象出通用的算法和数据结构，而不需要为每种具体类型编写重复的代码。

在本章中，我们将通过构建一个**通用数据处理与分析框架**（dataflow-framework）来深入学习这些概念。这个框架将展示如何在实际项目中应用泛型和特征来创建可扩展、可维护的企业级系统。

### 学习目标

完成本章学习后，您将能够：

- 理解泛型的基本概念和语法
- 掌握特征的定义、实现和使用
- 学会特征边界和泛型约束
- 掌握特征对象和动态分发的概念
- 理解关联类型和泛型关联类型
- 学会如何使用泛型和特征设计可扩展的架构
- 构建一个完整的数据处理框架

### 实战项目预览

本章实战项目将构建一个通用数据处理框架，支持：
- 多种数据源（文件、数据库、API、实时流）
- 灵活的数据处理管道
- 多种输出格式
- 性能优化和并发处理

## 5.2 泛型基础

### 5.2.1 什么是泛型

在 Rust 的宏与泛型体系中，泛型（Generics）是构建 零拷贝、类型安全、高性能程序的核心基石。如果说 Rust 库是“积木”，那么泛型就是赋予这些积木“通用模具”的能力。

**Rust 泛型的本质：**

允许你定义 “带有类型参数”的结构和函数 。编译器会帮你处理类型，你只需要定义逻辑。

> 通俗比喻：
>  - 泛型就像是一个 “万能盒子” 。
> 
>  - 没有泛型：你需要为盒子装“苹果”做一个盒子，再为盒子装“梨子”做一个盒子。
> -  有泛型：你只需要做一个“盒子”，告诉它“盒子可以装苹果，也可以装梨子”，你只需要声明“装的东西”是 T 类型，Rust 编译器会确保你装进去的一定是合法的东西。

Rust 泛型主要分为两个部分：**Type Parameters（类型参数）** 和 **Associated Types（相关类型）**。初学者主要关注前者。

```rust
// 定义一个结构体，其中 T 代表任意类型
struct Box<T> {
    data: T,          // 这里 T 是类型参数
    capacity: usize,
}

// 使用泛型结构体
let my_box_int = Box { data: 10, capacity: 5 };
let my_box_str = Box { data: "Hello".to_string(), capacity: 5 };

// 编译器会确保 data 的类型与定义一致
```
### 5.2.2 泛型函数

让我们从一个简单的泛型函数开始：

```rust
// 泛型函数示例
fn compare<T>(a: T, b: T) -> i32 
where
    T: PartialOrd,
{
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

// 使用泛型函数
fn main() {
    println!("比较整数: {}", compare(5, 3));  // 输出: 1
    println!("比较浮点数: {}", compare(3.14, 2.71));  // 输出: 1
    println!("比较字符串: {}", compare("abc", "xyz"));  // 输出: -1
}
```

在上面的例子中：
- `T` 是类型参数，表示函数可以处理任何类型
- `where T: PartialOrd` 是特征边界，指定T必须实现`PartialOrd`特征
- 这样函数就能对所有实现了比较操作符的类型工作

### 5.2.3 泛型结构体

如果说泛型是 Rust 的“骨架”，那么 泛型结构体和 泛型枚举就是构建这个骨架最核心的两块砖。掌握这两者，你就能写出像 Vec、 Option 这样既灵活又类型安全的 Rust 代码。

想象你在盖房子。

> **非泛型**：你想盖一座“别墅”，就需要专门设计一套图纸；你想盖一套“公寓”，又要另一套图纸。
> **泛型**：你设计了一个“模块化模板”，通过改变模板里的“砖块类型”（T），它可以变成“别墅”，也可以变成“公寓”。

在 Rust 中，泛型结构体允许你在定义时声明一个“类型变量”（通常用 T 表示），让该结构体可以封装任意一种类型。

泛型结构体最强大的地方：你可以为结构体定义通用方法。

你需要使用 impl<T> 来为泛型结构体实现方法。
```rust
// 泛型结构体
#[derive(Debug, Clone)]
struct Container<T> {
    items: Vec<T>,
    capacity: usize,
}

impl<T> Container<T> {
    fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    fn push(&mut self, item: T) {
        if self.items.len() < self.capacity {
            self.items.push(item);
        }
    }
    
    fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
    
    fn len(&self) -> usize {
        self.items.len()
    }
}

// 泛型结构体的方法
impl<T: std::fmt::Display> Container<T> {
    fn print_all(&self) {
        for item in &self.items {
            println!("{}", item);
        }
    }
}

fn main() {
    let mut int_container = Container::new(3);
    int_container.push(1);
    int_container.push(2);
    int_container.push(3);
    
    println!("整数容器内容: {:?}", int_container.items);
    println!("容器大小: {}", int_container.len());
    
    let mut string_container = Container::new(2);
    string_container.push("hello");
    string_container.push("world");
    string_container.print_all();  // 需要Display trait
}
```

> **⚠️ 注意点**：
> - 泛型方法中的 `T` 必须与结构体的 `T` 一致。
> - 如果结构体中有字段是 `Option<T>` 或 `&T`，泛型方法中仍然可以使用 `T`。
> - Rust 不允许像 Java 那样直接通过 `Pair<T>` 调用，必须在 `impl` 块中显式实现。

### 5.2.4 泛型枚举

枚举（Enum）是 Rust 中定义“多种可能状态”的最佳方式。
- **非泛型**：`enum Status { Active, Disabled }`
- **泛型**：`enum Status<T> { Active, Disabled }` —— 这里 `Active` 和 `Disabled` 的具体含义依赖于类型 `T`。

最常见的泛型枚举是 `Option<T>`（`Some` 包含 T，`None` 不）。
### 🛠 定义与使用

```rust
// 泛型枚举示例
#[derive(Debug, Clone)]
enum Result<T, E> {
    Ok(T),
    Err(E),
}

#[derive(Debug, Clone)]
enum Option<T> {
    Some(T),
    None,
}

// 实用函数
impl<T, E> Result<T, E> {
    fn is_ok(&self) -> bool {
        matches!(self, Ok(_))
    }
    
    fn is_err(&self) -> bool {
        matches!(self, Err(_))
    }
}

impl<T> Option<T> {
    fn unwrap(self) -> T {
        match self {
            Some(value) => value,
            None => panic!("Called Option::unwrap() on a None value"),
        }
    }
    
    fn unwrap_or(self, default: T) -> T {
        match self {
            Some(value) => value,
            None => default,
        }
    }
}

```
> **⚠️ 注意点**：
> - **`impl` 位置**：泛型枚举的 impl 块不能放在枚举定义的内部，而必须放在枚举定义的外部（模块层级）。当为泛型枚举实现方法时，需要显式写出 impl<T>（或带有约束的 impl<T: Bound>），以声明该实现适用于所有类型 T（或满足约束的 T）。
> - **`Associated Types`**：像 `Option<T>` 中的 `Some`，虽然它只包含一个 `T`，但 `Option` 本身是一个泛型类型。
> - **`Result` 的变种**：`Result<T, E>` 是一个更复杂的泛型枚举，它有两个类型参数 `T` 和 `E`。


### 5.2.5 泛型结构体 vs 泛型枚举

| 特性 | 泛型结构体 (Struct) | 泛型枚举 (Enum) |
| :--- | :--- | :--- |
| **用途** | 数据容器、封装逻辑 | 状态机、多态模式、分支逻辑 |
| **灵活性** | 适合封装对象属性 | 适合表达“不同情况下的不同类型” |
| **实现方法** | `impl<T> StructName` | `impl<T> EnumName` (需考虑所有变体) |
| **典型例子** | `Box<T>`, `Vec<T>` (内部结构) | `Option<T>`, `Result<T, E>` |

### 📝 什么时候该用哪个？

1.  **选泛型结构体**：
    - 当你想要封装一组**属性**，且这些属性都**同类型**时（例如 `Pair<T>` 的 `first` 和 `second` 都是 `T`）。
    - 当你需要类似 Java 的 `Class` 那种封装数据的感觉。

2.  **选泛型枚举**：
    - 当你有**多种模式**，且这些模式代表了**不同的行为**时（例如 `Option` 是“有值”或“无值”，`Result` 是“成功”或“失败”）。
    - 枚举允许你**在定义时即可定义类型**，这是 Rust 泛型中最强大的特性之一。



## 5.3 特征

### 5.3.1 什么是特征

特征（Trait）定义了一组可以由不同类型实现的方法。它们类似于其他语言中的接口，但功能更强大。

**Rust 的 Trait 本质**：
它定义了一组**行为（方法）的集合**。如果一个类型实现了这个 Trait，它就获得了这些行为的能力。

> **Rust 的 Trait 与 Java/C++ 的区别**：
> Java/C++ 的接口（Interface）通常指向运行时多态（动态类型）。
> Rust 的 Trait 指向**编译期多态**（静态类型）。
> - **Java**: 运行时多态（`obj` 是运行时类型，`obj.method()` 调用接口）。
> - **Rust**: 编译期多态（`trait` 是模板，编译器为每个类型生成具体代码，无运行时开销）。

#### 🧩 为什么 Rust 需要 Trait？
1. **抽象化**：将不同结构体的行为统一。
2. **类型安全**：确保 `T` 只有在满足某些能力（如 `Copy` 或 `Send`）时才能使用。
3. **无拷贝（Zero-Cost）**：Trait 的约束在编译期解决，运行时没有虚表开销（Virtual Table Overhead）。

### 5.3.2. 基础语法：定义与使用

Rust 的 Trait 分为两部分：**定义（Definition）** 和 **实现（Implementation）**。

#### 5.3.2.1 定义 Trait
使用 `trait` 关键字，定义一组方法。
```rust
// 定义一个名为 'Clone' 的 Trait（继承自 std）
trait Clone {
    fn clone(&self) -> Self;
}

// 定义 'Display' 特质，用于打印到标准输出
trait Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>;
}
```

#### 5.3.2.2 实现 Trait
为某个类型（Struct 或 Enum）实现 Trait 方法。
```rust
struct Boxed <T> {
    data: T,
}

// 为 Boxed<T> 实现 Clone 方法
impl<T> Clone for Boxed<T> {
    fn clone(&self) -> Boxed<T> {
        // 实际逻辑：复制内部数据
        Boxed {
            data: self.data.clone(),
        }
    }
}
```

> **💡 注意**：
> - 实现 Trait 时，必须使用 `impl TraitName for Type` 的语法。
> - 如果类型是泛型（如 `Boxed<T>`），实现时必须包含泛型参数 `<T>`。


```rust
// 定义一个特征
pub trait Drawable {
    fn draw(&self) -> String;
    
    // 默认实现
    fn area(&self) -> f64 {
        0.0  // 默认面积为0
    }
    
    // 可以有其他方法
    fn is_visible(&self) -> bool {
        true  // 默认可见
    }
}

// 实现特征的类型
struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

struct Triangle {
    base: f64,
    height: f64,
}

// 为每个类型实现Drawable特征
impl Drawable for Circle {
    fn draw(&self) -> String {
        format!("画一个半径为 {} 的圆形", self.radius)
    }
    
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl Drawable for Rectangle {
    fn draw(&self) -> String {
        format!("画一个 {}x{} 的矩形", self.width, self.height)
    }
    
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl Drawable for Triangle {
    fn draw(&self) -> String {
        format!("画一个底边 {}，高 {} 的三角形", self.base, self.height)
    }
    
    fn area(&self) -> f64 {
        (self.base * self.height) / 2.0
    }
}

// 函数接受实现了特征的类型
fn draw_shape<T: Drawable>(shape: &T) {
    println!("{}", shape.draw());
    println!("面积: {:.2}", shape.area());
    println!("可见: {}", shape.is_visible());
    println!("---");
}

fn main() {
    let circle = Circle { radius: 5.0 };
    let rectangle = Rectangle { width: 4.0, height: 6.0 };
    let triangle = Triangle { base: 3.0, height: 4.0 };
    
    draw_shape(&circle);
    draw_shape(&rectangle);
    draw_shape(&triangle);
}
```

### 5.3.3 特征作为参数

 **“Trait 作为参数” (Trait as Parameter)**，其实并不是指直接传递一个 `Trait` 定义本身（Rust 不支持这个），而是指**如何在一个函数中利用 Trait 的能力**。

主要有两种场景：
1.  **泛型约束 (`where T: Trait`)**：编译期检查，类型安全。
2.  **Trait 对象 (`&dyn Trait`)**：运行时多态，类型擦除。

```rust
// 使用特征作为函数参数
// 定义一个 Trait
trait Addable<T> {
    fn add(&self, other: &T) -> T;
}

// 定义一个泛型函数
fn process<T: Addable>(value: T) {
    // 这里 T 必须实现 Addable
    println!("Value is: {}", value);
}
```
**💡 关键点**
- **`where` 子句**：你也可以使用 `where` 子句来添加约束。
    ```rust
    fn process<T>(value: T) where T: Addable { ... }
    ```
- **类型检查**：编译器会检查 `T` 是否实现了 `Addable`。如果 `T` 不是 `i32` 或 `String`，而是任意类型，编译器会报错。
- **性能**：这是**零开销**的（Zero-Cost）。编译器会生成具体的代码版本，没有运行时开销。



## 5.4 特征边界高级用法

### 5.4.1 多个特征约束（Multiple Trait Bounds）

Rust 允许一个类型 `T` 同时实现多个 Trait。在函数参数或结构体中，你可以使用 `+` 运算符组合多个 Trait 约束。

**语法格式：**
```rust
fn process<T: TraitA + TraitB + TraitC>(item: T) { ... }
```

### 5.4.2. 为什么需要组合约束？
单一约束不够用。例如，你需要一个既能被 `Clone` 复制，又能被 `Debug` 打印，且是 `Send` 安全的类型。

```rust
trait Cloneable { fn clone(&self) -> Self; }
trait Printable { fn print(&self); }
trait ThreadSafe { /* ... */ }

// ❌ 错误写法：T 被要求同时满足多个约束，编译器会报错
// fn process<T: Cloneable + Printable + ThreadSafe>(item: T) ... 

// ✅ 正确写法：组合约束
fn process<T: Cloneable + Printable + ThreadSafe>(item: T) {
    // item 自动拥有 Clone, Print, ThreadSafe 行为
    let _clone = item.clone();
    item.print();
    // item 必须是线程安全的
}
```

### 5.4.3. 约束的顺序有影响吗？
**没有。** Rust 的 trait bounds 是**集合运算**（Set Union），顺序不影响编译。
```rust
fn foo<T: Clone + Debug>(x: T) { } // 等价于 fn foo<T: Debug + Clone>(x: T) { }
```

### 5.4.4. 特殊情况：超 Traits 与 `Where` 子句
虽然 `TraitA + TraitB` 很直观，但在复杂场景下，`Where` 子句更灵活。
```rust
fn generic_function<T: Clone>() {
    // 无法直接在这里加约束，需要 Where
    where T: Clone {} 
}
```
**注意**：`where` 子句允许你在函数内部动态添加约束，适用于某些无法在定义时确定的场景。

```rust
// 定义多个特征
trait Printable {
    fn print(&self);
}

trait Cloneable {
    fn clone_me(&self) -> Self;
}

trait Validatable {
    fn is_valid(&self) -> bool;
}

// 使用多个特征约束
fn process_item<T>(item: &T) 
where
    T: Printable + Cloneable + Validatable,
{
    if item.is_valid() {
        item.print();
        let cloned = item.clone_me();
        cloned.print();
    }
}

// 或者使用 + 语法
fn process_item_shorthand<T: Printable + Cloneable + Validatable>(item: &T) {
    // 同样的实现
}

// 复杂约束示例
fn complex_processing<T, U, V>(item1: T, item2: U, item3: V) 
where
    T: std::fmt::Display + Cloneable,
    U: Printable + Validatable,
    V: Cloneable + Validatable + std::fmt::Debug,
{
    println!("项目1: {}", item1);
    if item2.is_valid() {
        item2.print();
    }
    println!("项目3: {:?}", item3);
}
```

### 5.4.2 特征对象（Trait Objects）：动态多态

`dyn Trait` 表示一个动态多态的对象。Rust 允许通过 `&dyn Trait` 或 `Box<dyn Trait>` 来持有 Trait 对象。

**语法格式：**
```rust
fn accept_trait_object<T: Trait>(item: T) { ... }

fn accept_dyn_trait(item: &dyn Trait) { ... }
```




```rust
trait Drawable {
    fn draw(&self);
}

// ✅ 使用 Box<dyn Drawable>
struct Circle {
    r: f64,
}

struct Square {
    side: f64,
}

struct Shape {
    shape: Box<dyn Drawable>,
}

// 动态多态调用
impl Drawable for Circle {
    fn draw(&self) { println!("Drawing Circle"); }
}
impl Drawable for Square {
    fn draw(&self) { println!("Drawing Square"); }
}

fn main() {
    let circle = Circle { r: 5.0 };
    let square = Square { side: 5.0 };
    
    let shape: Box<dyn Drawable> = Box::new(circle);
    println!("Shape: {:?}", shape.draw()); // 动态调用
}
```


```rust
// 特征对象允许我们使用不同类型的相同特征
fn demonstrate_trait_objects() {
    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Rectangle { width: 2.0, height: 3.0 }),
        Box::new(Triangle { base: 4.0, height: 5.0 }),
    ];
    
    // 动态分派 - 运行时决定调用哪个方法
    for shape in &shapes {
        println!("{}", shape.draw());
        println!("面积: {:.2}", shape.area());
    }
}

// 特征对象的返回类型
fn create_shape(shape_type: &str) -> Box<dyn Drawable> {
    match shape_type {
        "circle" => Box::new(Circle { radius: 2.0 }),
        "rectangle" => Box::new(Rectangle { width: 3.0, height: 4.0 }),
        "triangle" => Box::new(Triangle { base: 5.0, height: 6.0 }),
        _ => Box::new(Circle { radius: 1.0 }),
    }
}

// 特征对象作为参数
fn draw_all_shapes(shapes: &[Box<dyn Drawable>]) {
    for (i, shape) in shapes.iter().enumerate() {
        println!("形状 {}: {}", i + 1, shape.draw());
    }
}
```

### 5.4.3 动态多态 vs 泛型约束



这是理解的关键点：

| 特性 | 泛型约束 (`T: Trait`) | 动态多态 (`dyn Trait`) |
| :--- | :--- | :--- |
| **类型推断** | 编译时确定 | 运行时确定 |
| **性能** | 编译时调用（无偏移） | 动态调用（有偏移/表） |
| **使用场景** | 类型已知 | 类型未知 |
| **安全性** | 类型安全 | 类型安全（但无法 `self`） |

```rust
// 泛型方式 - 编译时分派，性能更好
fn draw_shapes_generic<T>(shapes: &[T])
where
    T: Drawable,
{
    for shape in shapes {
        shape.draw();
    }
}

// 特征对象方式 - 运行时动态分派，更灵活
fn draw_shapes_trait_object(shapes: &[Box<dyn Drawable>]) {
    for shape in shapes {
        shape.draw();
    }
}

// 使用泛型
fn main() {
    let circles = vec![Circle { radius: 1.0 }, Circle { radius: 2.0 }];
    // draw_shapes_generic(&circles);  // 只处理同一种类型
    
    let mixed_shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Rectangle { width: 2.0, height: 3.0 }),
    ];
    // draw_shapes_trait_object(&mixed_shapes);  // 可以处理不同类型
}
```

### 5.13 本章总结

在本章中，我们系统学习了 Rust 中两大核心抽象机制——**泛型（Generics）** 和 **特征（Traits）**，并通过构建通用数据处理框架的思路，深入理解了如何利用它们设计灵活、高性能、可扩展的企业级代码。

**主要知识点回顾：**

- **泛型基础**：掌握了类型参数 `T` 的使用，学会了定义泛型函数、泛型结构体（`Container<T>`）和泛型枚举（`Option<T>`、`Result<T, E>`）。理解了 `impl<T>` 和带约束的 `impl<T: Bound>` 的写法。
- **特征（Traits）**：学会了特征的定义、默认实现、为具体类型实现特征，以及使用特征作为函数参数。
- **特征边界与约束**：掌握了 `T: Trait`、`where` 子句、多特征约束（`TraitA + TraitB`）的使用，能够灵活限制泛型的行为。
- **特征对象与动态多态**：理解了 `&dyn Trait` 和 `Box<dyn Trait>` 的作用，区分了**编译时静态分派（泛型）** 与 **运行时动态分派（特征对象）** 的区别、性能差异及适用场景。
- **设计思想**：泛型提供零成本抽象，特征提供行为抽象，二者结合可以构建高度可复用且类型安全的代码架构。

通过本章的学习，你已经具备了使用泛型和特征编写通用、可扩展代码的能力，这也是从“能写 Rust”迈向“会设计 Rust 系统”的重要一步。

**核心 takeaway**：
> **泛型让代码“通用”，特征让代码“有行为”，两者结合让 Rust 代码既安全又灵活**。

---

### 5.13 验收标准（学习自检）

完成本章后，你应该能够自信地完成以下任务：

1. **基础验收**：
   - 正确定义泛型函数、泛型结构体和泛型枚举，并为其实现方法。
   - 为自定义结构体实现至少一个标准特征（如 `Debug`、`Clone`、`Display`）和一个自定义特征。
   - 使用 `T: Trait` 或 `where` 子句为泛型添加约束。

2. **进阶验收**：
   - 能说明泛型（静态分派）与特征对象（动态分派）的区别，并举例说明各自的优缺点和适用场景。
   - 熟练使用多个特征约束（`+` 运算符或 `where` 子句）。
   - 设计一个包含关联类型的特征，并正确使用它。

3. **项目验收**（数据处理框架相关）：
   - 能够定义一个通用的 `Processor<T>` 特征，用于不同类型数据的处理。
   - 使用泛型实现一个可配置的数据管道（Pipeline），支持多种数据源和输出格式。
   - 使用特征对象实现一个支持动态添加处理器的插件系统。

**自检方式**：尝试独立完成下面练习题中的中级和高级题目，如果能顺利通过，则说明你已较好掌握本章内容。

---

### 5.14 练习题

#### 基础练习（巩固语法）

1. 定义一个泛型结构体 `Pair<T>`，包含 `first` 和 `second` 两个字段。为它实现 `new()` 方法和 `swap()` 方法（交换两个值）。

2. 定义一个特征 `Summable`，包含一个方法 `sum(&self) -> i32`。分别为 `Vec<i32>` 和自定义的 `Point { x: i32, y: i32 }` 实现该特征（`Point` 的 `sum` 返回 `x + y`）。

3. 编写一个泛型函数 `print_if_large<T: PartialOrd + Display>(value: T, threshold: T)`，当 `value > threshold` 时打印该值。

#### 中级练习（综合应用）

4. 实现一个泛型枚举 `Either<L, R>`，包含 `Left(L)` 和 `Right(R)` 两个变体。为它实现一个方法 `unwrap_left(self) -> L`，如果不是 `Left` 则 panic。

5. 定义一个特征 `Processable`：
   ```rust
   trait Processable {
       type Output;
       fn process(&self) -> Self::Output;
   }
   ```
   为 `String` 和 `i32` 分别实现该特征（`String` 返回其长度，`i32` 返回其平方）。然后编写一个泛型函数，使用关联类型接收 `Processable` 类型并打印处理结果。

6. 创建一个 `DataPipeline<T>` 结构体，使用特征对象 `Vec<Box<dyn Processor>>` 存储多个处理器，实现一个支持动态添加处理器的管道系统。

#### 高级练习（接近实战）

7. 设计一个简单的日志系统：
   - 定义特征 `Logger`（包含 `log(&self, message: &str)` 方法）。
   - 实现 `ConsoleLogger` 和 `FileLogger` 两个结构体。
   - 编写一个泛型函数 `log_all<T: Logger>(loggers: &[T])`（静态分派）和一个使用特征对象的版本 `log_all_dyn(loggers: &[Box<dyn Logger>])`（动态分派）。
   - 对比两者在性能和灵活性上的差异（可通过基准测试思考）。

8. 扩展本章的数据处理框架：
   - 定义 `DataSource` 特征（支持 `read()` 方法，返回 `Vec<u8>`）。
   - 定义 `DataTransformer<T>` 特征（使用关联类型）。
   - 定义 `DataSink` 特征（支持 `write()` 方法）。
   - 使用泛型构建一个 `Pipeline<S: DataSource, T: DataTransformer, K: DataSink>` 结构体，实现端到端的通用数据处理流程。

---

### 5.15 扩展阅读

为了更深入理解泛型与特征，推荐以下资源：

1. **官方文档（强烈推荐）**
   - [The Rust Book - Generics](https://doc.rust-lang.org/book/ch10-00-generics.html)
   - [The Rust Book - Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
   - [The Rust Reference - Trait Objects](https://doc.rust-lang.org/reference/types/trait-object.html)
