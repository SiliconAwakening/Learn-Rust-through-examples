# 第四章：结构体与枚举

## 学习目标

通过本章学习，您将掌握：
- Rust中结构体的定义、方法和关联函数
- 枚举的强大功能和模式匹配
- 如何设计灵活的数据结构
- 实现生产级的配置管理系统
- 实战项目：构建一个企业级配置管理工具

## 4.1 引言：结构化数据的重要性

在现实世界中，数据很少是孤立的。应用程序需要处理复杂的、相互关联的数据结构。Rust通过结构体和枚举提供了强大的工具来建模和操作这些复杂数据。

**为什么需要结构体和枚举？**
- **类型安全**：确保数据结构的完整性
- **表达力**：精确建模业务逻辑
- **维护性**：清晰的代码组织
- **性能**：零成本的抽象

## 4.2 结构体基础

### 4.2.1 什么是结构体？

在许多编程语言中，我们经常需要将一系列相关联的数据组织在一起，例如一个“用户”对象，它包含用户名、年龄和电子邮件地址。在 C++ 或 Java 中，我们会使用 class 来实现这一点。

在 Rust 中，结构体（struct）的作用与类非常相似，但概念上更纯粹：它仅仅是一个用于组合多个不同类型数据的复合数据类型（Compound Data Type）。

- 核心用途： 将相关的字段（Fields）打包在一起，形成一个逻辑实体。
- 本质区别 (Struct vs Class)： struct 关注的是数据的组织和结构；而传统的 class 通常将“数据”和“行为”（方法/函数）捆绑在一个单元内。在 Rust 中，我们通常使用 struct 定义数据结构，然后使用关联实现块（impl block）来定义与这个数据相关的行为（方法）。

```rust
struct User {
    name: String,
    email: String,
    age: u32,
    is_active: bool,
}

fn main() {
    let user = User {
        name: String::from("Alice"),
        email: String::from("alice@example.com"),
        age: 25,
        is_active: true,
    };
    
    println!("User: {} ({})", user.name, user.email);
}
```

### 4.2.2 定义和使用结构体


#### 4.2.2.1 基础结构体
使用 `struct` 关键字，后跟结构体名称和字段列表。

结构体本身只负责存储数据。如果我们要让这个数据能够执行某些操作（比如计算距离，或打印格式化信息），我们就需要使用 impl 块 来实现这些功能。

impl 的作用： 为一个特定的类型提供一组关联函数和方法。

**实现方法 (Methods)**

在 impl 块内部定义的函数，它们与结构体实例绑定，并且总是接收一个指向自身数据的引用作为第一个参数（通常命名为 self）。

```rust
// 定义一个点结构体
struct Point {
    x: f64,
    y: f64,
}

// 定义一个矩形结构体
struct Rectangle {
    top_left: Point,
    width: f64,
    height: f64,
}

impl Rectangle {
    // 关联函数（类似静态方法）
    fn new(top_left: Point, width: f64, height: f64) -> Self {
        Self {
            top_left,
            width,
            height,
        }
    }
    
    // 方法
    fn area(&self) -> f64 {
        self.width * self.height
    }
    
    fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.top_left.x
            && point.x <= self.top_left.x + self.width
            && point.y >= self.top_left.y
            && point.y <= self.top_left.y + self.height
    }
    
    fn move_to(&mut self, new_x: f64, new_y: f64) {
        self.top_left.x = new_x;
        self.top_left.y = new_y;
    }
}

// 关联函数vs方法的区别
fn main() {
    // 使用关联函数创建实例
    let rect = Rectangle::new(Point { x: 0.0, y: 0.0 }, 10.0, 5.0);
    
    // 调用方法
    println!("Area: {}", rect.area());
    
    // 只能通过方法修改，因为self是&mut self
    let mut rect = rect; // 需要声明mut
    rect.move_to(5.0, 2.0);
    
    let test_point = Point { x: 3.0, y: 1.0 };
    if rect.contains_point(&test_point) {
        println!("Point is inside rectangle");
    }
}
```
> 定义好结构体后，需要通过其名称来创建实际的数据实例。

##### 💡 理解 `self` 的类型（重要！）

在方法签名中，参数的类型决定了你对数据的访问权限：

| 参数 | 类型 | 含义 | 何时使用？ |
| :--- | :--- | :--- | :--- |
| **`&self`** | 共享引用 (`&T`) | 只读访问实例的数据。无法修改结构体字段的值。 | 只是读取数据，如计算距离、打印信息。 |
| **`&mut self`** | 可变引用 (`&mut T`) | 可读写地访问和修改实例的数据。 | 当方法需要改变结构体的状态时（如 `move_by`）。 |
| **`self`** | 移动所有权 (`T`) | 接收整个结构体的所有权。方法结束后，该实例将无法使用（被“销毁”）。 | 通常用于消耗自身，例如在 Drop 实现中或实现 `into()` 方法时。 |

#### 4.2.2.2 元组结构体
如果你的结构体只是一个占位符，它没有任何命名意义的字段，你不需要为每个字段起名字，而是用括号和逗号来代替字段名称。这常用于代表固定格式的数据（如坐标点）。
元组结构体类似于元组，但每个字段都有类型：

```rust
struct Color(u8, u8, u8);
struct Point3D(f64, f64, f64);

fn main() {
    let red = Color(255, 0, 0);
    let point = Point3D(1.0, 2.0, 3.0);
    
    // 通过索引访问
    println!("Red: {}, Green: {}, Blue: {}", red.0, red.1, red.2);
    println!("Point: x={}, y={}, z={}", point.0, point.1, point.2);
}
```

#### 4.2.2.3 空结构体(Unit Structs)

如果一个结构体不需要存储任何数据，它只是作为一个标记或类型安全的存在。没有字段的结构体，称为空结构体(Unit Structs)或单元结构体：

```rust
struct UnitStruct;

// 主要用于实现trait
impl SomeTrait for UnitStruct {
    // 可以为空
}

fn main() {
    let unit = UnitStruct;
    // unit可以用作标记
}
```

### 4.2.3 结构和操作

#### 4.2.3.1 字段访问

字段访问是指通过点操作符 (.) 直接访问和修改结构体实例内部存储的、具有明确名字的数据变量。你是在直接处理“数据本身”。

**⚙️ 工作原理**
- 语法: instance_name.field_name
- 作用: 读取（只读）或写入（可变）。
- 核心： 你是在操作结构体内存中的一个特定“槽位”的值。

```rust
struct Student {
    name: String,
    student_id: u32,
    gpa: f32,
    subjects: Vec<String>,
}

fn main() {
    let mut student = Student {
        name: String::from("Bob"),
        student_id: 2023001,
        gpa: 3.85,
        subjects: Vec::new(),
    };
    
    // 访问字段
    println!("Student: {} (ID: {})", student.name, student.student_id);
    
    // 修改字段
    student.subjects.push("Rust Programming".to_string());
    student.gpa += 0.1; // 获得额外分数
    
    // 完整更新语法
    let student2 = Student {
        name: String::from("Charlie"),
        student_id: 2023002,
        // ... 复制其他字段
        gpa: 3.75,
        subjects: vec!["Python".to_string()],
    };
    
    let student3 = Student {
        name: String::from("Diana"),
        ..student2 // 复制除name外的其他字段
    };
}
```

#### 4.2.3.2 方法和关联函数

结构体无法只包含数据而不具备任何功能；我们需要 impl 来添加“生命力”。这里的行为分为两种类型：方法 (Method) 和 关联函数 (Associated Function)。

**A. 方法（Methods）**
方法是与特定实例（即特定的数据值）紧密绑定的行为。它们总是需要一个实例作为前提才能执行。

📜 定义
通过 impl 块定义，第一个参数必须是自身的一个引用 (&self, &mut self, 或 self)。

🚀 工作原理
方法接收结构体实例的引用，然后使用这个引用来读取或修改其内部字段，并执行一段逻辑计算或操作。

**B. 关联函数（Associated Functions）**
关联函数（通常用作构造函数 ::new()）是与结构体类型本身绑定的独立行为。它们不需要一个实例来调用，只需要知道“我是什么类型的”。

📜 定义
在 impl 块内定义，但不接收任何参数作为第一个参数 (self)。

🚀 工作原理
最常见的作用是：构造函数（Constructor）。提供一种标准化的、安全的方式来创建并初始化结构体实例。

```rust
struct Calculator {
    result: f64,
    history: Vec<String>,
}

impl Calculator {
    // 关联函数（类似构造器）
    fn new() -> Self {
        Self {
            result: 0.0,
            history: Vec::new(),
        }
    }
    
    fn with_initial_value(value: f64) -> Self {
        Self {
            result: value,
            history: vec![format!("Initial value: {}", value)],
        }
    }
    
    // 方法（接收&self）
    fn get_result(&self) -> f64 {
        self.result
    }
    
    // 方法（接收&mut self）
    fn add(&mut self, value: f64) {
        self.result += value;
        self.history.push(format!("+ {} = {}", value, self.result));
    }
    
    // 方法（接收self，消耗实例）
    fn get_history(self) -> Vec<String> {
        self.history
    }
    
    // 泛型方法
    fn apply_operation<T>(&mut self, value: T, operation: Operation)
    where
        T: Into<f64>,
    {
        let num: f64 = value.into();
        self.perform_operation(num, operation);
    }
    
    fn perform_operation(&mut self, value: f64, operation: Operation) {
        match operation {
            Operation::Add => self.result += value,
            Operation::Subtract => self.result -= value,
            Operation::Multiply => self.result *= value,
            Operation::Divide => {
                if value != 0.0 {
                    self.result /= value;
                }
            }
        }
        self.history.push(format!("{:?} {} = {}", operation, value, self.result));
    }
}

#[derive(Debug)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

fn main() {
    let mut calc = Calculator::new();
    calc.add(10.0);
    calc.add(5.0);
    calc.apply_operation(2.0, Operation::Multiply);
    calc.apply_operation(3.0, Operation::Subtract);
    
    println!("Result: {}", calc.get_result());
    
    // 获取历史记录（消耗calc）
    let history = calc.get_history();
    println!("History: {:?}", history);
}
```
**⚖️ 方法 vs. 关联函数：对比表**

| 特性 (Feature) | 💡 方法 (Methods) | ✨ 关联函数 (Associated Functions) |
| :--- | :--- | :--- |
| **核心概念** | 操作 **对象实例** 的行为。 | 操作 **类型本身** 的能力（静态行为）。 |
| **接收器 (Receiver)** | 必须接受一个明确的接收器参数：`&self`, `&mut self`, 或 `self`。 | 不接受显式的接收器，直接通过类型名调用。 |
| **作用域/目的** | 用于读取、修改或基于实例状态执行操作。 | 常用于构造函数 (`::new`)、工厂模式，或实现需要类型级的工具方法。 |
| **语法定义** | `fn method_name(&self, ...)` (在 `impl` 块内) | `fn associated_func(...)` (在 `impl` 块内) |
| **调用方式** | 需要一个实例：`instance.method_name(...)` | 通过类型名（Scope Resolution）：`Type::associated_function(...)` |

### 4.2.4 高级特性

#### 4.2.4.1 泛型结构体

**核心定义**：泛型是什么？ (What)

泛结构体 指的是在使用 类型参数（Type Parameters） 来定义的结构体，而不是使用固定的数据类型（如 i32 或 String）。

它允许你创建的结构体具有“通用性”。你可以把一个结构体看作是一个占位符模板，这个占位符等待外部传入具体的类型来“实例化”自己。

```rust
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
        } else {
            panic!("Container is full");
        }
    }
    
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
    
    fn len(&self) -> usize {
        self.items.len()
    }
    
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
    
    fn get_all(&self) -> &[T] {
        &self.items
    }
    
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }
    
    fn clear(&mut self) {
        self.items.clear();
    }
}

fn main() {
    // 字符串容器
    let mut string_container = Container::new(3);
    string_container.push("Hello".to_string());
    string_container.push("World".to_string());
    string_container.push("Rust".to_string());
    
    println!("String container length: {}", string_container.len());
    
    for item in string_container.iter() {
        println!("Item: {}", item);
    }
    
    // 数字容器
    let mut number_container = Container::new(5);
    number_container.push(1.0);
    number_container.push(2.5);
    number_container.push(3.7);
    
    for num in number_container.get_all() {
        println!("Number: {}", num);
    }
}
```
✅ 带来的两大核心好处：

代码复用性 (Code Reusability)： 只写一套逻辑，服务于无限多种数据类型。

类型安全 (Type Safety)（最重要）： 与使用 Any 指针或 void* 进行的运行时类型转换不同，Rust 的泛型是在 编译时 

强制检查类型的。这保证了程序在执行前就能发现潜在的数据不匹配错误，极大地提高了代码的健壮性。

> 泛结构体让你的代码拥有了高度的通用性和极高的编译时安全保障。它让 Rust 的代码库既高效又健壮，是编写框架、库和高性能工具的首选模式。


#### 4.2.4.2 生命周期在结构体中

当我们在结构体（Struct）中存储**引用（References）**时，就必须引入生命周期。因为结构体本身是数据的“容器”，如果它内部的引用的数据源在结构体存在的时间内被销毁了，那么这个结构体就会成为一个包含“悬垂指针”的危险容器。

```rust
struct ReferenceHolder<'a> {
    reference: &'a str,
    data: String,
}

impl<'a> ReferenceHolder<'a> {
    fn new(reference: &'a str, data: String) -> Self {
        Self {
            reference,
            data,
        }
    }
    
    fn get_reference(&self) -> &'a str {
        self.reference
    }
    
    fn get_data(&self) -> &str {
        &self.data
    }
    
    // 返回生命周期较短的引用
    fn get_data_mut(&mut self) -> &mut str {
        &mut self.data
    }
    
    fn get_data_string(self) -> (String, String) {
        (self.reference.to_string(), self.data)
    }
}

fn main() {
    let data = String::from("Hello World");
    let holder = ReferenceHolder::new(&data, data.clone());
    
    // 引用指向的数据比holder生命周期长
    let _long_lived_ref = holder.get_reference(); // OK
    
    // 所有权数据
    let _owned_data = holder.get_data().to_string(); // 复制
    let (ref_str, data_str) = holder.get_data_string();
    println!("Reference: {}, Data: {}", ref_str, data_str);
}
```

## 4.3 枚举详解

枚举是 Rust 类型系统中最重要、最强大的特性之一，它让开发者能够表达更复杂、类型安全且易于调试的数据结构。与其他语言（如 C 或 Java）相比，Rust 的枚举不仅在结构设计上更灵活，还在类型安全和内存布局上提供诸多优势。 


**什么是枚举？**

枚举是一种预定义的类型，其值由一组常量或 tag/value 结构组成，类似于“状态枚举”，用于表达“这是一件事”或“这是一个状态”的语义。Rust 的枚举是专门为表达能力强、类型安全、无运行时开销而设计的。

**三大核心能力（Highlights）**

1. 编译期穷举检查 (Exhaustiveness Check)
这是 Rust 枚举的“杀手锏”。当你使用 `match` 遍历枚举时，编译器会检查所有 **变体（Variant）** 是否都被覆盖。
*   如果漏掉了一个分支（比如你的枚举有 `Success` 和 `Err`，你只写了 `Success`），编译器会直接报 **Error**，而不是等到运行时崩溃。
*   **专家提示**：这是防止“空指针”、“逻辑分支遗漏”的第一道防线。

2. 强大的模式匹配 (Pattern Matching)
Rust 的 `match` 不仅仅是 `if-else` 的替代，它支持 **解构（Deconstruction）**。
*   你可以直接对变量进行“拆解”并匹配。
*   例如：`match result { Some(x) => ... }`。`x` 在这里直接是值，无需先取 `result.unwrap()`，这避免了运行时空指针异常。

3. 零成本的内存布局 (Zero-Cost Memory)
Rust 枚举的内存布局非常高效：
*   **Unit 变体**（如 `Red`）：只占用一个字节（标签位），没有额外数据。
*   **Struct 变体**：如果其结构体有内存占用，枚举变体本身不增加额外开销，只是指向数据。
*   这使得它们非常适合用于 **状态机（State Machine）** 和 **错误处理（Error Handling）**，且几乎不占用堆内存（Heap）。

### 4.3.1 基础枚举

`基础枚举`既可以包含 标签（如 Red, Green），也可以包含 数据（如 Green(255)）。
Rust 允许通过 `match` 对枚举进行解构，提取内部数据。

**适用场景：**

状态机（State Machine）：定义有限的状态流转。

聚合类型：表示一个变量可能是多种具体类型之一（如 u8, u16, u32 等）。

无法用 Option/Result 表达的逻辑分支：例如“红色”、“绿色”、“蓝色”这三个状态，不能表示“无数据”或“错误”。



```rust
// 简单的枚举
#[derive(Debug)]
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

impl TrafficLight {
    fn time(&self) -> u32 {
        match self {
            TrafficLight::Red => 30,
            TrafficLight::Yellow => 5,
            TrafficLight::Green => 45,
        }
    }
    
    fn next(&self) -> TrafficLight {
        match self {
            TrafficLight::Red => TrafficLight::Green,
            TrafficLight::Yellow => TrafficLight::Red,
            TrafficLight::Green => TrafficLight::Yellow,
        }
    }
}

// 携带数据的枚举
enum WebEvent {
    PageLoad,
    PageUnload,
    Click { x: i32, y: i32 },
    KeyPress(char),
    Paste(String),
    Scroll { delta_x: f32, delta_y: f32 },
    Resize { width: u32, height: u32 },
}

fn main() {
    let light = TrafficLight::Red;
    println!("Light time: {} seconds", light.time());
    println!("Next light: {:?}", light.next());
    
    let click = WebEvent::Click { x: 50, y: 100 };
    let paste = WebEvent::Paste("Hello Rust!".to_string());
    let resize = WebEvent::Resize { width: 1920, height: 1080 };
    
    process_event(click);
    process_event(paste);
    process_event(resize);
}

fn process_event(event: WebEvent) {
    match event {
        WebEvent::PageLoad => println!("Page loaded"),
        WebEvent::PageUnload => println!("Page unloaded"),
        WebEvent::Click { x, y } => println!("Click at ({}, {})", x, y),
        WebEvent::KeyPress(c) => println!("Key pressed: {}", c),
        WebEvent::Paste(text) => println!("Pasted: {}", text),
        WebEvent::Scroll { delta_x, delta_y } => {
            println!("Scrolled: ({}, {})", delta_x, delta_y);
        }
        WebEvent::Resize { width, height } => {
            println!("Window resized: {}x{}", width, height);
        }
    }
}
```

### 4.3.2 复杂的枚举

#### 4.3.2.1 Option枚举

`Option`是Rust标准库中最重要的枚举，它表示一个值 可能存在 (Some)，也可能 不存在 (None)。


`Option<T>` 是 Rust 标准库中泛型枚举的封装。表示 非空值，即“如果没有则返回 None”。

`Option<T>` 的内存开销仅 1 字节（标签位），非常轻量。


官方标准库定义：

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub enum Option<T> {
    /// No value
    None,
    
    /// Some value `T`
    Some(T),
}

```

**适用场景：**

查找可能为空的值：如 `if let Some(user) = users.get(id)`。当一个函数返回 Option<T> 时，表示调用可能成功也可能失败。


```rust

fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

fn find_user(id: u32) -> Option<User> {
    if id == 1 {
        Some(User { name: "Alice".to_string(), id })
    } else {
        None
    }
}

#[derive(Debug, Clone)]
struct User {
    name: String,
    id: u32,
}

fn main() {
    let result = divide(10.0, 2.0);
    match result {
        Some(quotient) => println!("10 / 2 = {}", quotient),
        None => println!("Cannot divide by zero"),
    }
    
    // 使用if let进行条件检查
    if let Some(quotient) = divide(10.0, 0.0) {
        println!("Result: {}", quotient);
    } else {
        println!("Division by zero");
    }
    
    // unwrap 方法
    let value = result.unwrap(); // 可能 panic!
    
    // unwrap_or 提供默认值
    let value = divide(10.0, 0.0).unwrap_or(0.0);
    
    // 链式操作
    let user = find_user(1)
        .and_then(|user| find_user(2).map(|user2| (user, user2)))
        .unwrap_or((
            User { name: "Anonymous".to_string(), id: 0 },
            User { name: "Anonymous".to_string(), id: 0 },
        ));
    
    println!("Found users: {:?}", user);
}
```
> **unwrap() 陷阱**：unwrap() 会直接 Panic 如果值是 None。
>
>**if let** 是处理 Option 的推荐方式，比 match 更简洁，且在 IDE 中更容易处理。
>
>**Option 不是 Result**：Option 用于表示“可能没有数据”，Result 用于表示“错误”。

#### 4.3.2.2 Result枚举

Result (Result 枚举) 表示一个函数调用 **成功 (Ok)**，或者 **失败 (Err)**。

`Result<T, E>` 是 Rust 中用于错误处理的 **标准库枚举**。表示 **成功** 还是 **失败**。与 `Option` 类似，只占用 1 字节（标签位）。

官方标准库定义

```rust
// Rust 标准库 (std::result) 中的真实定义
#[derive(Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
#[must_use]
pub enum Result<T, E> {
    /// Contains the success value
    Ok(T),

    /// Contains the error value
    Err(E),
}
```

**适用场景：**

   **函数返回值**：当函数可能抛出错误时，使用 `Result<T, E>`。
   **`?` 运算符**：这是 Rust 处理 `Result` 的核心糖衣，会自动向上抛错。

>   **`?` 运算符**：它是 Rust 中处理错误传播的标准方式。如果函数返回 `Result`，使用 `?` 可以将错误向上抛出，代码更简洁。
>
>   **`Err` 类型**：`E` 类型通常是一个结构体（如 `MyError`），而不是直接使用 `String`，因为 `String` 在错误传播中效率低。
>
>   **`Result` 不是 `Option`**：`Result` 用于表示错误处理，`Option` 用于表示空值。



```rust

fn parse_number(s: &str) -> Result<i32, String> {
    match s.parse::<i32>() {
        Ok(n) => Ok(n),
        Err(_) => Err(format!("'{}' is not a number", s)),
    }
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

fn main() {
    match parse_number("42") {
        Ok(n) => println!("Number: {}", n),
        Err(e) => println!("Error: {}", e),
    }
    
    if let Ok(n) = parse_number("42") {
        println!("Parsed: {}", n);
    }
    
    // 错误传播
    fn process_numbers(a: &str, b: &str) -> Result<i32, String> {
        let num1 = parse_number(a)?; // 传播错误
        let num2 = parse_number(b)?;
        Ok(num1 + num2)
    }
    
    let sum = process_numbers("10", "32");
    println!("Sum: {:?}", sum);
    
    // 组合多个Result
    let results = vec!["1", "2", "3", "4"];
    let numbers: Result<Vec<i32>, _> = results.iter()
        .map(|s| parse_number(s))
        .collect();
    
    match numbers {
        Ok(nums) => println!("All numbers: {:?}", nums),
        Err(e) => println!("Failed to parse: {}", e),
    }
}
```

#### 4.3.2.3 自定义错误类型

为了更精确的错误描述，使用自己的错误类型结构。

Rust 的 Result 的 E 类型可以是自定义的 struct。现代 Rust 推荐使用 Box<dyn std::error::Error> 或实现 std::error::Error trait。


**适用场景：**

业务逻辑错误：如“用户未登录”、“余额不足”。

系统级错误：如“文件未找到”、“网络超时”。

调试信息：使用自定义结构体可以携带 source、message、code 等元数据。

```rust
// 自定义错误类型
#[derive(Debug)]
enum ConfigError {
    FileNotFound(String),
    InvalidFormat(String),
    MissingKey(String),
    ValidationFailed(String),
    IOError(std::io::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "Configuration file not found: {}", path),
            ConfigError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ConfigError::MissingKey(key) => write!(f, "Missing required key: {}", key),
            ConfigError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ConfigError::IOError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::IOError(e)
    }
}

// 错误处理函数
fn load_config(path: &str) -> Result<Config, ConfigError> {
    if !std::path::Path::new(path).exists() {
        return Err(ConfigError::FileNotFound(path.to_string()));
    }
    
    let content = std::fs::read_to_string(path)?;
    parse_config(&content)
}

fn parse_config(content: &str) -> Result<Config, ConfigError> {
    if content.trim().is_empty() {
        return Err(ConfigError::InvalidFormat("Empty content".to_string()));
    }
    
    // 解析逻辑...
    Ok(Config::new())
}

struct Config {
    settings: std::collections::HashMap<String, String>,
}

impl Config {
    fn new() -> Self {
        Self {
            settings: std::collections::HashMap::new(),
        }
    }
}
```
为了便于记忆，我整理了一个快速对照表：

| 类型 | 定义 | 内存开销 | 典型用法 | 是否推荐 |
| :--- | :--- | :--- | :--- | :--- |
| **Basic Enum** | 标签 + 数据 | 无 (Tag) | 状态机、类型选择 | ✅ 推荐 |
| **Option** | 有值 (T) 或 无值 (None) | 1 字节 (Tag) | 表示“可能没有数据” | ✅ 推荐 |
| **Result** | 成功 (Ok) 或 失败 (Err) | 1 字节 (Tag) | 函数返回成功或错误 | ✅ 推荐 |
| **Custom Error** | 描述具体错误信息 | 可变 (Data) | 业务逻辑错误、调试信息 | ✅ 推荐 |

### 4.3.3 枚举的高级用法

#### 4.3.3.1 枚举作为泛型参数

在 Rust 中，将枚举作为泛型参数（Generic Parameter）的主要目的是：

**确保类型安全性**：通过枚举的变体覆盖来保证类型安全。

**实现类型级别编程**：利用枚举实现类型级别的行为。

**支持策略模式**：通过枚举实现策略模式。

**实现类型推导**：利用枚举实现类型推导。

```rust
#[derive(Debug)]
enum Either<T, E> {
    Left(T),
    Right(E),
}
#[derive(Debug)]
enum Nullable<T> {
    Some(T),
    None,
}
#[derive(Debug)]
enum ResultOr<T, E> {
    Success(T),
    Failure(E),
}

// 模式匹配与泛型
impl<T, E> Either<T, E> {
    fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }
    
    fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }
    
    fn as_ref(&self) -> Either<&T, &E> {
        match self {
            Either::Left(value) => Either::Left(value),
            Either::Right(error) => Either::Right(error),
        }
    }
    
    fn map<U, F>(self, f: F) -> Either<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Either::Left(value) => Either::Left(f(value)),
            Either::Right(error) => Either::Right(error),
        }
    }
    
    fn unwrap_or(self, default: T) -> T {
        match self {
            Either::Left(value) => value,
            Either::Right(_) => default,
        }
    }
}

fn main() {
    let result: Either<i32, String> = Either::Left(42);
    let error: Either<i32, String> = Either::Right("Error".to_string());
    
    if result.is_left() {
        println!("Got a value");
    }
    
    if error.is_right() {
        println!("Got an error");
    }
    
    let mapped = result.map(|x| x * 2);
    println!("Mapped result: {:?}", mapped);
}
```
> Rust 中，枚举作为泛型参数 是实现 类型安全 和 类型推导 的重要工具。通过 T 作为泛型参数，可以灵活地实现 类型级别编程 和 策略模式。

#### 4.3.3.2 复杂的状态机

Rust 中，枚举（enum） 是实现复杂状态机（State Machine）最优雅、最 idiomatic 的方式之一。它比传统的面向对象“状态模式”（用 trait + struct）更简洁、安全，且编译器能提供强力保障。

**为什么枚举特别适合做状态机？**

- 每个状态可以是枚举的一个 variant（变体）。
- 不同状态可以携带 不同类型的数据（payload）。
- 通过 match 表达式处理状态转换，编译器会强制你覆盖所有可能的状态（穷尽性检查）。
- 可以轻松实现类型安全的状态转换（消耗旧状态，返回新状态，避免无效状态）。
- 性能优秀（通常和 C 的 enum + switch 差不多）。





```rust
// 状态机模式
#[derive(Debug,Clone)]
enum State {
    Idle,
    Connecting,
    Connected,
    Authenticating,
    Authenticated,
    Error(String),
    Closed,
}
#[derive(Debug)]
enum Event {
    Connect,
    Disconnect,
    DataReceived(String),
    Error(String),
    Timeout,
    AuthSuccess,
    AuthFailed,
}

struct Connection {
    state: State,
    retry_count: u32,
    max_retries: u32,
}

impl Connection {
    fn new(max_retries: u32) -> Self {
        Self {
            state: State::Idle,
            retry_count: 0,
            max_retries,
        }
    }
    
    fn handle_event(&mut self, event: Event) -> Result<(), String> {
        let old_state = self.state.clone();
        
        self.state = match (self.state.clone(), event) {
            (State::Idle, Event::Connect) => {
                self.retry_count = 0;
                State::Connecting
            }
            
            (State::Connecting, Event::DataReceived(_)) => State::Authenticating,
            (State::Connecting, Event::Timeout) => {
                self.retry_count += 1;
                if self.retry_count >= self.max_retries {
                    return Err("Max retries exceeded".to_string());
                }
                State::Connecting
            }
            (State::Connecting, Event::Error(e)) => State::Error(e),
            
            (State::Authenticating, Event::AuthSuccess) => State::Authenticated,
            (State::Authenticating, Event::AuthFailed) => {
                self.retry_count += 1;
                if self.retry_count >= self.max_retries {
                    return Err("Authentication failed after max retries".to_string());
                }
                State::Connecting
            }
            (State::Authenticating, Event::Error(e)) => State::Error(e),
            
            (State::Authenticated, Event::Disconnect) => State::Closed,
            (State::Authenticated, Event::Error(e)) => State::Error(e),
            
            (State::Error(_), Event::Connect) => {
                self.retry_count = 0;
                State::Connecting
            }
            
            (State::Error(_), Event::Disconnect) => State::Closed,
            
            (_, Event::Disconnect) => State::Closed,
            
            (s, e) => {
                println!("Unhandled transition: {:?} -> {:?}", s, e);
                s
            }
        };
        
        println!("State transition: {:?} -> {:?}", old_state, self.state);
        Ok(())
    }
    
    fn get_state(&self) -> &State {
        &self.state
    }
    
    fn is_connected(&self) -> bool {
        matches!(self.state, State::Authenticated)
    }
}

fn main() {
    let mut conn = Connection::new(3);
    
    // 连接流程
    conn.handle_event(Event::Connect).unwrap();
    conn.handle_event(Event::DataReceived("response".to_string())).unwrap();
    conn.handle_event(Event::AuthSuccess).unwrap();
    
    println!("Connected: {}", conn.is_connected());
    
    // 断开连接
    conn.handle_event(Event::Disconnect).unwrap();
    println!("State: {:?}", conn.get_state());
}
```

## 4.4 模式匹配

**什么是模式匹配？**

Rust 的 **模式匹配（Pattern Matching）** 是一种强大的控制流程结构，允许根据变量的值、类型或结构体字段来分支执行代码。相比传统 `if-else`，Rust 的模式匹配更类型安全，支持更精细的控制。

Rust 提供了几种主要的模式匹配方式：

- `match`：用于分情况处理变量。
- `if let`：用于简化布尔分支。
- `match on &mut`：用于解构引用。
- `match` on 复杂类型（如枚举、选项类型等）。
- `match` on 结构体字段。
- `match` on 可变引用（`&mut`）等。

### 4.4.1 基础模式匹配

```rust
fn main() {
    let value = 42;
    
    match value {
        0 => println!("Zero"),
        1 => println!("One"),
        2..=10 => println!("Between 2 and 10"),
        11..=100 => println!("Between 11 and 100"),
        _ => println!("Something else: {}", value),
    }
    
    // if let 语法
    if let 42 = value {
        println!("Found 42!");
    }
    
    // while let
    let mut option: Option<i32> = Some(5);
    while let Some(x) = option {
        println!("Processing: {}", x);
        option = if x > 0 {
            Some(x - 1)
        } else {
            None
        };
    }
    
    // 匹配Option
    let maybe_number = Some(42);
    if let Some(n) = maybe_number {
        println!("Number: {}", n);
    } else {
        println!("No number");
    }
}
```

### 4.4.2 高级模式匹配

#### 4.4.2.1 解构结构体

结构体解构允许你通过模式匹配直接解构结构体的字段，而不需要显式引用。它常用于访问结构体的数据并生成类型安全的代码。
```rust
struct Point {
    x: i32,
    y: i32,
}

struct Person {
    name: String,
    age: i32,
    address: Address,
}

struct Address {
    street: String,
    city: String,
    zip_code: String,
}

fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        address: Address {
            street: "123 Main St".to_string(),
            city: "Anytown".to_string(),
            zip_code: "12345".to_string(),
        },
    };
    
 // 创建一个 Person 实例

    match person {
    // 第一个匹配分支（带守卫）
        Person {
            name,                    // 直接绑定 name 字段
            age,                     // 直接绑定 age 字段
            address: Address {       // 对 address 字段进行嵌套解构
                street,              // 绑定 street
                city,                // 绑定 city
                ..                   // 忽略 zip_code（使用 .. 表示剩余字段全部忽略）
            },
        } if age >= 18 => {          // 匹配守卫（guard）
            println!("Adult: {} lives in {}", name, city);
        }

        // 第二个匹配分支（通配）
        Person { name, age, .. } => {
            println!("Minor: {} is {} years old", name, age);
        }
}
    
    // 简单解构
    let point = Point { x: 10, y: 20 };
    let Point { x, y } = point;
    println!("Point: ({}, {})", x, y);
    
    // 在let语句中使用模式
    let Point { x: x1, y: y1 } = point;
    println!("x1: {}, y1: {}", x1, y1);
}
```

#### 4.4.2.2 守卫条件

守卫条件允许你在模式匹配中增加额外的条件判断，用于更精细地控制分支逻辑。它通常用于处理特定数据，避免不必要的错误匹配。

```rust
#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
    SetVolume(i32),
}

fn main() {
    let msg = Message::ChangeColor(255, 0, 0);
    
    match msg {
        Message::Move { x, y } if x == y => {
            println!("Diagonal move: {}, {}", x, y);
        }
        Message::Move { x, y } if x == 0 || y == 0 => {
            println!("Axis-aligned move: {}, {}", x, y);
        }
        Message::Move { x, y } => {
            println!("General move: {}, {}", x, y);
        }
        Message::Write(text) if text.len() > 10 => {
            println!("Long message: {}", text);
        }
        Message::Write(text) => {
            println!("Short message: {}", text);
        }
        Message::ChangeColor(r, g, b) if r == g && g == b => {
            println!("Grayscale: ({}, {}, {})", r, g, b);
        }
        Message::ChangeColor(r, g, b) if r == 255 && g == 0 && b == 0 => {
            println!("Pure red color");
        }
        Message::ChangeColor(r, g, b) => {
            println!("Color: ({}, {}, {})", r, g, b);
        }
        Message::SetVolume(volume) if volume > 100 => {
            println!("Volume too high: {}", volume);
        }
        Message::SetVolume(volume) if volume == 0 => {
            println!("Muted");
        }
        Message::SetVolume(volume) => {
            println!("Volume: {}", volume);
        }
        Message::Quit => {
            println!("Quitting");
        }
    }
}
```

### 4.4.3 模式匹配最佳实践

#### 4.4.3.1 穷尽性检查

Rust 的 match 表达式要求所有可能的值都被匹配到，确保没有遗漏。如果存在未覆盖的情况，编译器会报错。

```rust
enum Color {
    Red,
    Green,
    Blue,
    Alpha(f32),
}

fn match_color(color: Color) -> String {
    // Rust会检查是否穷尽了所有情况
    match color {
        Color::Red => "Red".to_string(),
        Color::Green => "Green".to_string(),
        Color::Blue => "Blue".to_string(),
        // 必须处理Alpha变体
        Color::Alpha(a) => format!("Alpha: {}", a),
    }
}

// 如果我们忘记处理某个变体，编译器会报错：
fn bad_match_color(color: Color) -> String {
    match color {
        Color::Red => "Red".to_string(),
        Color::Green => "Green".to_string(),
        // 错误：未处理Blue和Alpha
        _ => "Unknown".to_string(), // 使用通配符但会丢失信息
    }
}

// 更好的做法：明确处理所有变体
fn better_match_color(color: Color) -> String {
    match color {
        Color::Red => "Red".to_string(),
        Color::Green => "Green".to_string(),
        Color::Blue => "Blue".to_string(),
        Color::Alpha(a) => format!("Alpha: {}", a),
    }
}
```

#### 4.4.3.2 @绑定

@ 是一个占位符，用于将模式匹配的结果绑定到一个新变量中，常用于提取值而不直接赋值给现有变量。

**使用场景**

- ✅ 解构复杂结构体字段
- ✅ 提取值并复用，避免重复绑定
- ✅ 与 if let 结合，简化逻辑

```rust
#[derive(Debug)]
enum Message {
    Move { x: i32, y: i32 },
    Say(String),
    Other,
}

fn main() {
    let msg = Message::Move { x: 5, y: 10 };
    
    match msg {
        // 绑定整个值到m，同时解构字段
        m @ Message::Move { x, y } => {
            println!("Message: {:?} has coordinates ({}, {})", m, x, y);
        }
        // 绑定字符串到s
        s @ Message::Say(_) => {
            println!("Say message: {:?}", s);
        }
        // 绑定到other
        other => {
            println!("Other message: {:?}", other);
        }
    }
    
    // 使用@绑定进行复杂模式匹配
    let point = (1, 2);
    match point {
        (x, y) if x == y => {
            println!("Equal point: ({}, {})", x, y);
        }
        pt @ (x, y) if x > y => {
            println!("Diagonal point: {:?}", pt);
        }
        pt => {
            println!("Other point: {:?}", pt);
        }
    }
}
```


| 特性         | @ 绑定                    | 穷尽匹配                    |
|--------------|---------------------------|-----------------------------|
| 作用         | 提取值并绑定到新变量       | 确保所有情况都被覆盖         |
| 使用场景     | 解构结构体、提取字段值      | 枚举、选项类型等分支处理    |
| 类型安全     | ✅ 提升代码可读性与维护性   | ✅ 防止遗漏，编译器强制检查  |
| 最佳实践     | 避免重复绑定，使用 `_` 兜底 | 所有分支必须覆盖，避免遗漏 |

## 4.5 本章总结

本章深入探讨了Rust中结构体和枚举的强大功能，这是构建复杂应用程序的基础。通过本章的学习，您已经：

1. **掌握了结构体基础**：定义了各种类型的结构体，包括元组结构体和泛型结构体
2. **学会了方法设计**：区分了关联函数和方法的用法
3. **了解了枚举威力**：从简单的枚举到复杂的携带数据的枚举
4. **掌握了模式匹配**：学会了使用match表达式进行复杂的模式匹配

结构体和枚举为Rust提供了强大的数据建模能力，使得开发者能够创建类型安全、表达力强的代码。这些概念在实际的Rust开发中无处不在，是掌握Rust编程的必备知识。

## 4.6 验收标准

完成本章后，您应该能够：

- [ ] 设计合理的结构体来建模业务数据
- [ ] 实现结构体的方法和关联函数
- [ ] 使用枚举精确建模状态和选项
- [ ] 编写复杂的模式匹配代码
- [ ] 设计可扩展的数据验证框架

## 4.7练习题

1. **设计Employee结构体**：创建一个Employee结构体，包含姓名、职位、薪资等字段
2. **实现状态机**：使用枚举实现一个游戏状态机
3. **配置验证器**：为配置系统添加更多验证规则
4. **模式匹配优化**：重构代码以使用更简洁的模式匹配
5. **性能对比测试**：比较不同数据结构实现的性能差异

## 4.8 扩展阅读

- [Rust官方文档：结构体](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [Rust官方文档：枚举和模式匹配](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [Rust与模式匹配](https://doc.rust-lang.org/book/ch18-00-patterns.html)
