
# Chapter 4: Structs and Enums

## Learning Objectives

By the end of this chapter, you will have mastered:

- Defining structs, methods, and associated functions in Rust
- The powerful features of enums and pattern matching
- How to design flexible data structures
- Building a production-grade configuration management system
- **Hands-on Project**: Building an enterprise-level configuration management tool

## 4.1 Introduction: The Importance of Structured Data

In the real world, data is rarely isolated. Applications need to handle complex, interrelated data structures. Rust provides powerful tools—**structs** and **enums**—to model and manipulate this kind of data.

**Why do we need structs and enums?**

- **Type Safety**: Ensures the integrity of data structures
- **Expressiveness**: Allows precise modeling of business logic
- **Maintainability**: Results in clean, well-organized code
- **Performance**: Zero-cost abstractions

## 4.2 Struct Basics

### 4.2.1 What Is a Struct?

In many programming languages, we often need to group related pieces of data together. For example, a "User" object might contain a username, age, and email address. In C++ or Java, we would typically use a `class` for this.

In Rust, a **struct** (short for structure) serves a similar purpose, but with a purer concept: it is simply a **compound data type** that combines multiple values of different types into one logical unit.

- **Core Purpose**: To bundle related **fields** together to form a meaningful entity.
- **Key Difference (Struct vs Class)**: A struct focuses purely on organizing data. In Rust, we define the data in a `struct`, then use `impl` blocks to attach behavior (methods and functions) to it.

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

### 4.2.2 Defining and Using Structs

#### 4.2.2.1 Basic Structs

A struct is defined using the `struct` keyword, followed by the struct name and a list of fields.

A struct by itself only stores data. To give it behavior (such as calculating values or printing formatted information), we use an `impl` block.

**The Role of `impl`**: It attaches associated functions and methods to a specific type.

**Methods**

Methods defined inside an `impl` block are bound to instances of the struct. They always take a reference to the instance itself as the first parameter (usually named `self`).

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
**Understanding `self` Types (Very Important!)**

The type of `self` in a method determines what you can do with the data:

| Parameter     | Type                  | Meaning                                      | When to Use? |
|---------------|-----------------------|----------------------------------------------|--------------|
| **`&self`**   | Shared reference (`&T`) | Read-only access. Cannot modify fields.     | For calculations, printing, reading data |
| **`&mut self`** | Mutable reference (`&mut T`) | Read and write access. Can modify fields. | When the method needs to change the struct's state |
| **`self`**    | Ownership (`T`)       | Takes ownership of the entire struct. The instance is consumed. | When you want to consume the struct (e.g., into conversion, Drop) |

#### 4.2.2.2 Tuple Structs

If your struct is just a simple wrapper without meaningful field names, you can use a **tuple struct**. Fields are accessed by index (`.0`, `.1`, etc.). This is useful for types like colors or coordinates.
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

#### 4.2.2.3 Unit Structs (Empty Structs)

A struct with no fields is called a **unit struct**. It is useful as a marker type or for implementing traits when no data needs to be stored.
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

### 4.2.3 Struct Operations

#### 4.2.3.1 Field Access

You can access and modify struct fields using the dot operator (`.`).

- Syntax: `instance_name.field_name`
- You can read fields directly or modify them if the instance is mutable.
- Rust also supports **struct update syntax** (`..other`) to copy remaining fields from another instance.
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

#### 4.2.3.2 Methods and Associated Functions

Structs gain behavior through `impl` blocks. There are two main types:

**A. Methods**  
Methods operate on a specific instance of the struct. They must take `self`, `&self`, or `&mut self` as the first parameter.

**B. Associated Functions**  
These are functions attached to the type itself (not to an instance). They are called using `Type::function_name()`. The most common use is as constructors (e.g., `::new()`).

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

**Methods vs Associated Functions – Comparison**

| Feature                  | Methods                          | Associated Functions                  |
|--------------------------|----------------------------------|---------------------------------------|
| **Core Concept**         | Operate on an **instance**       | Operate on the **type** itself        |
| **Receiver**             | Must have `self`, `&self`, or `&mut self` | No receiver (`self`)                  |
| **Purpose**              | Read/modify instance state       | Constructors, factory methods, utilities |
| **Calling Syntax**       | `instance.method()`              | `Type::associated_function()`         |

### 4.2.4 Advanced Features

#### 4.2.4.1 Generic Structs

A **generic struct** uses type parameters (`<T>`) so that the same struct definition can work with many different concrete types.

**Main Benefits**:
- **Code Reuse**: Write logic once and use it with any data type.
- **Type Safety**: All type checking happens at compile time, preventing runtime errors.

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

---

✅ **Two Core Benefits:**

**1. Code Reusability**:  
You write a single piece of logic that can work with an infinite variety of data types.

**2. Type Safety** (The Most Important One):  
Unlike runtime type conversions using `Any` pointers or `void*` in other languages, Rust’s generics perform all type checking at **compile time**. This ensures that potential data type mismatches are caught before the program even runs, greatly improving the robustness and reliability of your code.

> Generic structs give your code high reusability and extremely strong compile-time safety guarantees. They make Rust codebases both efficient and robust, making them the preferred choice when writing frameworks, libraries, and high-performance tools.

---

#### 4.2.4.2 Lifetimes in Structs

When a struct stores **references**, you must specify **lifetimes** (`'a`). This tells the compiler that the referenced data must live at least as long as the struct itself, preventing dangling references.
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
---

## 4.3 Enums in Depth

Enums are one of Rust’s most powerful and distinctive features. They allow you to express complex, type-safe data structures with excellent performance and safety guarantees.

**What Is an Enum?**

An enum (enumeration) is a type that can have one of several predefined variants. Each variant can optionally carry associated data.

**Three Core Strengths**:

1. **Exhaustiveness Checking** at compile time — The compiler forces you to handle every possible variant.
2. **Powerful Pattern Matching** — You can destructure data directly inside `match`.
3. **Zero-Cost Memory Layout** — Very efficient; unit variants use almost no memory.

### 4.3.1 Basic Enums

Enums can be simple (just labels) or complex (carrying data of different types).

---

**Usage Scenarios:**

- **State Machines**: Defining a finite set of states and their transitions.
- **Sum Types / Algebraic Data Types**: Representing a value that can be one of several distinct types (for example, `u8`, `u16`, `u32`, etc.).
- **Logical Branches That Cannot Be Expressed with `Option` or `Result`**: For instance, the three states “Red”, “Green”, and “Blue”. These represent mutually exclusive possibilities that cannot be adequately modeled as “no data” or “an error”.

---

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




### 4.3.2 Important Standard Enums

#### 4.3.2.1 The `Option` Enum

`Option<T>` represents a value that may or may not be present (`Some(T)` or `None`).  
It is the idiomatic way in Rust to handle “possibly missing” values without using null pointers.
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
**Usage Scenarios:**

- **Looking up values that may be missing**: for example, `if let Some(user) = users.get(id)`.
- When a function returns `Option<T>`, it indicates that the operation **may or may not** produce a value — the call can succeed with a result or return nothing (no value).

In short: Use `Option` whenever a value is **optional** — it might exist, or it might not. This is Rust’s safe and idiomatic way to handle “possibly absent” data.

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

> **The `unwrap()` Trap**:  
> Calling `.unwrap()` on an `Option` will cause the program to **panic** immediately if the value is `None`.

> **Recommended Way to Handle `Option`**:  
> Use `if let` — it is generally the preferred and more concise way to handle `Option` values compared to a full `match` expression. It is also easier for IDEs to analyze and provide better code suggestions.

> **Important Distinction**:  
> `Option` is **not** the same as `Result`.  
> - `Option<T>` is used when a value **may or may not exist** (it represents the possible absence of data).  
> - `Result<T, E>` is used when an operation **may succeed or fail** with an error.



#### 4.3.2.2 The `Result` Enum

`Result<T, E>` represents success (`Ok(T)`) or failure (`Err(E)`).  
It is Rust’s standard mechanism for error handling.

**Key Difference**:
- `Option` → “Value may be absent”
- `Result` → “Operation may fail with an error”

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



**Usage Scenarios:**

- **Function Return Values**: Use `Result<T, E>` when a function may fail and needs to return an error.
- **`?` Operator**: This is Rust’s powerful syntactic sugar for error handling. It automatically propagates errors upward.

> **`?` Operator**:  
> It is the standard and idiomatic way to handle error propagation in Rust. If your function returns a `Result`, using `?` allows you to propagate the error automatically, making the code much cleaner and more concise.

> **`Err` Type Recommendation**:  
> The error type `E` should usually be a custom struct (such as `MyError`) rather than a plain `String`. Using `String` for errors is less efficient during error propagation.

> **`Result` vs `Option`**:  
> `Result<T, E>` is used for **error handling** — when an operation can succeed or fail.  
> `Option<T>` is used to represent **optional values** — when data may or may not be present (the absence of a value).

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



#### 4.3.2.3 Custom Error Types

For better error handling in real applications, define your own error enum or struct that implements the `std::error::Error` trait. This provides clear, meaningful error messages and allows proper error chaining.

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



---

To make it easier to remember, here’s a quick comparison table:

| Type              | Definition                              | Memory Overhead       | Typical Use Cases                          | Recommended? |
|-------------------|-----------------------------------------|-----------------------|--------------------------------------------|--------------|
| **Basic Enum**    | Tag + optional data                     | None (just a tag)     | State machines, type selection             | ✅ Recommended |
| **Option**        | A value (`Some(T)`) or no value (`None`) | 1 byte (tag)          | Representing data that may or may not exist | ✅ Recommended |
| **Result**        | Success (`Ok(T)`) or failure (`Err(E)`) | 1 byte (tag)          | Functions that can succeed or return an error | ✅ Recommended |
| **Custom Error**  | Describes specific error details        | Variable (depends on data) | Business logic errors, detailed debugging info | ✅ Recommended |

---



### 4.3.3 Advanced Enum Usage

#### 4.3.3.1 Enums as Generic Parameters

Enums can themselves be generic, allowing flexible and type-safe abstractions (e.g., `Either<T, E>`, custom `ResultOr<T, E>`).


In Rust, using **enums as generic parameters** serves several important purposes:

- **Ensuring Type Safety**: The enum’s variants allow the compiler to enforce complete type safety through exhaustive pattern matching.
- **Type-Level Programming**: Enums enable you to express and implement behavior directly at the type level.
- **Supporting the Strategy Pattern**: They provide an elegant and type-safe way to implement the strategy pattern.
- **Improving Type Inference**: Enums help the compiler infer types more effectively in complex scenarios.



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
> In Rust, treating enums as generic parameters is an important and elegant tool for achieving type safety and supporting type-level programming.  By introducing a generic type parameter like T, you can create highly flexible abstractions. This approach makes it easy to implement design patterns such as the strategy pattern while maintaining full compile-time type safety.



#### 4.3.3.2 Complex State Machines

Enums are the most elegant and idiomatic way to implement **state machines** in Rust. Each state is a variant, and `match` expressions ensure all transitions are handled safely at compile time.

---

**Why are enums particularly well-suited for implementing state machines?**

- Each state can be represented as a **variant** of the enum.
- Different states can carry **different types of data** (payload).
- State transitions are handled using `match` expressions, and the compiler **forces** you to cover all possible states (exhaustive checking).
- You can easily implement **type-safe state transitions** by consuming the old state and returning a new one, preventing invalid or unreachable states.
- Excellent performance — typically comparable to a C `enum` + `switch` statement.

---



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
---

## 4.4 Pattern Matching

**What Is Pattern Matching?**


Rust’s **pattern matching** is a powerful control flow construct that allows you to branch and execute code based on a variable’s value, type, or internal structure (such as struct fields). Compared to traditional `if-else` chains, Rust’s pattern matching is more **type-safe** and provides much finer-grained control.

Rust offers several main ways to perform pattern matching:

- **`match`**: Used for handling multiple cases of a value in a comprehensive way.
- **`if let`**: A concise way to simplify boolean-like conditional branches.
- **`match` on `&mut`**: For destructuring mutable references.
- **`match` on complex types**: Such as enums, `Option`, `Result`, and other algebraic data types.
- **`match` on struct fields**: Directly destructuring and matching fields inside structs.
- **`match` on mutable references** (`&mut`): And other advanced patterns.

---



### 4.4.1 Basic Pattern Matching
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

### 4.4.2 Advanced Pattern Matching

#### 4.4.2.1 Destructuring Structs

You can break structs apart directly in `match` or `let` statements.
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

#### 4.4.2.2 Match Guards

Guards (`if condition`) allow you to add extra boolean conditions to a pattern.

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

### 4.4.3 Best Practices for Pattern Matching

#### 4.4.3.1 Exhaustiveness Checking

The compiler ensures every possible case is handled. Missing a variant causes a compile error.
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

#### 4.4.3.2 `@` Bindings

The `@` operator lets you bind a value to a variable while also matching its internal structure.


**Use Cases**

- ✅ Destructuring complex struct fields  
- ✅ Extracting and reusing a value without repeated bindings  
- ✅ Combining with `if let` to simplify logic  

---


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

---

| Feature              | @ Binding                                      | Exhaustive Matching                              |
|----------------------|------------------------------------------------|--------------------------------------------------|
| **Purpose**          | Extracts a value and binds it to a new variable | Ensures all possible cases are covered           |
| **Use Cases**        | Destructuring structs and extracting field values | Branch handling for enums, `Option`, `Result`, etc. |
| **Type Safety**      | ✅ Improves code readability and maintainability | ✅ Prevents omissions; enforced by the compiler   |
| **Best Practice**    | Avoid repeated bindings; use `_` as a catch-all | All branches must be covered; avoid missing cases |

---


## 4.5 Chapter Summary

In this chapter, you have explored the powerful capabilities of **structs** and **enums** in Rust — the foundation for building complex, real-world applications.

You have learned:
1. How to define and use various kinds of structs (named, tuple, unit, generic)
2. How to attach behavior using methods and associated functions
3. The full power of enums, from simple variants to complex data-carrying ones
4. How to use pattern matching effectively and safely

Structs and enums give Rust its expressive power and strong type safety, making them essential tools for any serious Rust developer.

## 4.6 Acceptance Criteria

After completing this chapter, you should be able to:

- [ ] Design appropriate structs to model business data
- [ ] Implement methods and associated functions on structs
- [ ] Use enums to model states and optional values precisely
- [ ] Write complex pattern matching code
- [ ] Design extensible data validation frameworks

## 4.7 Exercises

1. **Design an Employee struct**: Include fields such as name, position, salary, etc.
2. **Implement a state machine**: Use enums to build a game state machine.
3. **Configuration Validator**: Add more validation rules to the configuration system.
4. **Optimize Pattern Matching**: Refactor code to use cleaner, more concise patterns.
5. **Performance Comparison**: Test and compare the performance of different data structure designs.

## 4.8 Further Reading

- [The Rust Book – Structs](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [The Rust Book – Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [The Rust Book – Patterns](https://doc.rust-lang.org/book/ch18-00-patterns.html)

