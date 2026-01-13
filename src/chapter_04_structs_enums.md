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

结构体（Struct）是一种复合数据类型，允许将多个相关的数据项组合在一起。与元组不同，结构体为每个字段提供有意义的名称。

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

#### 4.2.2.2 元组结构体

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

#### 4.2.2.3 单元结构体

没有字段的结构体，称为单元结构体：

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

### 4.2.4 高级特性

#### 4.2.4.1 泛型结构体

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

#### 4.2.4.2 生命周期在结构体中

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
    let holder = ReferenceHolder::new(&data, data);
    
    // 引用指向的数据比holder生命周期长
    let _long_lived_ref = holder.get_reference(); // OK
    
    // 所有权数据
    let _owned_data = holder.get_data().to_string(); // 复制
    let (ref_str, data_str) = holder.get_data_string();
    println!("Reference: {}, Data: {}", ref_str, data_str);
}
```

## 4.3 枚举详解

### 4.3.1 基础枚举

枚举允许定义一个类型，其值可以是几个固定选项中的一个：

```rust
// 简单的枚举
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

`Option`是Rust标准库中最重要的枚举：

```rust
enum Option<T> {
    Some(T),
    None,
}

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

#### 4.3.2.2 Result枚举

`Result`用于错误处理：

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}

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
    
    let sum = process_numbers("10", "32")?;
    println!("Sum: {}", sum);
    
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

### 4.3.3 枚举的高级用法

#### 4.3.3.1 枚举作为泛型参数

```rust
enum Either<T, E> {
    Left(T),
    Right(E),
}

enum Nullable<T> {
    Some(T),
    None,
}

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

#### 4.3.3.2 复杂的状态机

```rust
// 状态机模式
enum State {
    Idle,
    Connecting,
    Connected,
    Authenticating,
    Authenticated,
    Error(String),
    Closed,
}

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
    
    match person {
        Person {
            name,
            age,
            address: Address {
                street,
                city,
                ..
            },
        } if age >= 18 => {
            println!("Adult: {} lives in {}", name, city);
        }
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

## 4.5 实战项目：企业级配置管理工具

现在我们来构建一个完整的配置管理工具，展示结构体和枚举的实际应用。

### 4.5.1 项目设计

**项目名称**：`config-manager`

**核心功能**：
1. 多格式配置解析（JSON、YAML、TOML）
2. 类型安全的配置验证
3. 动态配置更新和热重载
4. 配置模板系统
5. 环境特定配置管理

### 4.5.2 项目结构

```
config-manager/
├── src/
│   ├── main.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── value.rs
│   │   ├── manager.rs
│   │   └── validation.rs
│   ├── parsers/
│   │   ├── mod.rs
│   │   ├── json.rs
│   │   ├── yaml.rs
│   │   ├── toml.rs
│   │   └── custom.rs
│   ├── hot_reload/
│   │   ├── mod.rs
│   │   ├── watcher.rs
│   │   └── notifier.rs
│   ├── templates/
│   │   ├── mod.rs
│   │   ├── engine.rs
│   │   └── generator.rs
│   └── utils/
│       ├── mod.rs
│       ├── error.rs
│       └── types.rs
├── examples/
├── tests/
└── configs/
    ├── development.yaml
    ├── production.json
    └── template.toml
```

### 4.5.3 核心实现

#### 4.5.3.1 配置值系统

**src/config/value.rs**

```rust
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// 配置数据类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<DataType>),
    Object,
    Custom(String),
}

impl DataType {
    pub fn is_compatible_with(&self, value: &ConfigValue) -> bool {
        match (self, &value.data_type) {
            (DataType::String, DataType::String) => true,
            (DataType::Integer, DataType::Integer) => true,
            (DataType::Float, DataType::Float) => true,
            (DataType::Boolean, DataType::Boolean) => true,
            (DataType::Array(inner_type), DataType::Array(value_type)) => {
                inner_type.is_compatible_with(&ConfigValue {
                    data_type: *value_type.clone(),
                    value: value.value.clone(),
                    required: false,
                    validation_rules: vec![],
                    description: String::new(),
                })
            }
            (DataType::Object, DataType::Object) => true,
            (DataType::Custom(custom1), DataType::Custom(custom2)) => custom1 == custom2,
            _ => false,
        }
    }
    
    pub fn from_json_value(value: &JsonValue) -> Self {
        match value {
            JsonValue::String(_) => DataType::String,
            JsonValue::Number(n) if n.is_i64() => DataType::Integer,
            JsonValue::Number(n) if n.is_f64() => DataType::Float,
            JsonValue::Bool(_) => DataType::Boolean,
            JsonValue::Array(_) => DataType::Object,
            JsonValue::Object(_) => DataType::Object,
            _ => DataType::String,
        }
    }
}

/// 配置验证规则
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationRule {
    MinValue(i64),
    MaxValue(i64),
    MinLength(usize),
    MaxLength(usize),
    Pattern(String), // 正则表达式
    Required,
    Custom(String), // 自定义验证脚本
}

/// 配置值结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigValue {
    pub value: JsonValue,
    pub data_type: DataType,
    pub required: bool,
    pub validation_rules: Vec<ValidationRule>,
    pub description: String,
    pub default_value: Option<JsonValue>,
    pub env_override: Option<String>,
    pub depends_on: Option<String>, // 依赖的另一个配置项
}

impl ConfigValue {
    pub fn new(
        value: JsonValue,
        data_type: DataType,
        required: bool,
    ) -> Self {
        Self {
            value: value.clone(),
            data_type,
            required,
            validation_rules: vec![],
            description: String::new(),
            default_value: None,
            env_override: None,
            depends_on: None,
        }
    }
    
    /// 验证配置值
    pub fn validate(&self) -> Result<(), ValidationError> {
        // 检查必需值
        if self.required && self.value.is_null() {
            if let Some(default) = &self.default_value {
                return Ok(()); // 使用默认值
            }
            return Err(ValidationError::Required(
                "Required configuration value is missing".to_string()
            ));
        }
        
        // 检查数据类型
        if !self.data_type.is_compatible_with(self) {
            return Err(ValidationError::TypeMismatch(format!(
                "Expected {:?}, got {:?}",
                self.data_type, self.value
            )));
        }
        
        // 应用验证规则
        for rule in &self.validation_rules {
            rule.apply(&self.value)?;
        }
        
        Ok(())
    }
    
    /// 获取实际值（考虑环境变量覆盖）
    pub fn get_actual_value(&self) -> Result<JsonValue, ConfigError> {
        // 检查环境变量覆盖
        if let Some(env_var) = &self.env_override {
            if let Ok(env_value) = std::env::var(env_var) {
                return Ok(JsonValue::String(env_value));
            }
        }
        
        // 返回实际值或默认值
        if self.value.is_null() {
            self.default_value.clone()
                .ok_or(ConfigError::MissingValue(self.description.clone()))
        } else {
            Ok(self.value.clone())
        }
    }
}

impl ValidationRule {
    pub fn apply(&self, value: &JsonValue) -> Result<(), ValidationError> {
        match self {
            ValidationRule::MinValue(min) => {
                if let Some(num) = value.as_i64() {
                    if num < *min {
                        return Err(ValidationError::MinValue(*min, num));
                    }
                }
            }
            ValidationRule::MaxValue(max) => {
                if let Some(num) = value.as_i64() {
                    if num > *max {
                        return Err(ValidationError::MaxValue(*max, num));
                    }
                }
            }
            ValidationRule::MinLength(min) => {
                if let Some(text) = value.as_str() {
                    if text.len() < *min {
                        return Err(ValidationError::MinLength(*min, text.len()));
                    }
                }
            }
            ValidationRule::MaxLength(max) => {
                if let Some(text) = value.as_str() {
                    if text.len() > *max {
                        return Err(ValidationError::MaxLength(*max, text.len()));
                    }
                }
            }
            ValidationRule::Pattern(pattern) => {
                if let Some(text) = value.as_str() {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|e| ValidationError::InvalidPattern(e.to_string()))?;
                    if !regex.is_match(text) {
                        return Err(ValidationError::PatternMismatch(pattern.clone(), text.to_string()));
                    }
                }
            }
            ValidationRule::Required => {
                if value.is_null() {
                    return Err(ValidationError::Required("Value is required".to_string()));
                }
            }
            ValidationRule::Custom(_) => {
                // 自定义验证逻辑
                // 这里可以实现更复杂的验证脚本
            }
        }
        
        Ok(())
    }
}

/// 配置错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required key: {0}")]
    MissingValue(String),
    
    #[error("Configuration key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Type conversion error: {0}")]
    TypeConversionError(String),
    
    #[error("Environment variable not set: {0}")]
    EnvNotSet(String),
    
    #[error("File I/O error: {0}")]
    IOError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    
    #[error("TOML error: {0}")]
    TomlError(#[from] toml::de::Error),
}

/// 验证错误类型
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Required value missing: {0}")]
    Required(String),
    
    #[error("Expected value >= {0}, got {1}")]
    MinValue(i64, i64),
    
    #[error("Expected value <= {0}, got {1}")]
    MaxValue(i64, i64),
    
    #[error("Expected length >= {0}, got {1}")]
    MinLength(usize, usize),
    
    #[error("Expected length <= {0}, got {1}")]
    MaxLength(usize, usize),
    
    #[error("Pattern mismatch: expected {0}, got {1}")]
    PatternMismatch(String, String),
    
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
}

/// 配置监听器
pub trait ConfigWatcher: Send + Sync {
    fn on_config_change(&self, key: &str, new_value: &ConfigValue);
    fn on_config_removed(&self, key: &str);
    fn on_validation_error(&self, key: &str, error: &ValidationError);
}

/// 配置监听器实现
pub struct LoggingWatcher {
    logger: slog::Logger,
}

impl LoggingWatcher {
    pub fn new(logger: slog::Logger) -> Self {
        Self { logger }
    }
}

impl ConfigWatcher for LoggingWatcher {
    fn on_config_change(&self, key: &str, new_value: &ConfigValue) {
        info!(self.logger, "Configuration changed: {} = {:?}", key, new_value.value);
    }
    
    fn on_config_removed(&self, key: &str) {
        warn!(self.logger, "Configuration removed: {}", key);
    }
    
    fn on_validation_error(&self, key: &str, error: &ValidationError) {
        error!(self.logger, "Configuration validation error: {} - {}", key, error);
    }
}
```

#### 4.5.3.2 配置管理器

**src/config/manager.rs**

```rust
use crate::config::value::{ConfigValue, ConfigError, ValidationError, ConfigWatcher, DataType};
use crate::parsers::{load_config_file, ConfigFormat};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::Path;
use std::fs;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, Event};
use crossbeam::channel::{unbounded, Receiver, Sender};
use rayon::prelude::*;

pub struct ConfigManager {
    configs: Arc<RwLock<HashMap<String, ConfigValue>>>,
    watchers: Arc<RwLock<Vec<Box<dyn ConfigWatcher>>>>,
    change_sender: Option<Sender<ConfigChangeEvent>>,
    watcher: Option<RecommendedWatcher>,
    logger: slog::Logger,
}

#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    pub key: String,
    pub old_value: Option<ConfigValue>,
    pub new_value: Option<ConfigValue>,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone)]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
}

impl ConfigManager {
    pub fn new(logger: slog::Logger) -> Self {
        let (change_sender, change_receiver) = unbounded();
        
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            watchers: Arc::new(RwLock::new(Vec::new())),
            change_sender: Some(change_sender),
            watcher: None,
            logger,
        }
    }
    
    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ConfigError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        
        // 检测文件格式
        let format = ConfigFormat::from_file_extension(path)
            .ok_or_else(|| ConfigError::InvalidFormat(
                format!("Unsupported file extension: {:?}", path.extension())
            ))?;
        
        let configs: HashMap<String, ConfigValue> = match format {
            ConfigFormat::Json => serde_json::from_str(&content)?,
            ConfigFormat::Yaml => serde_yaml::from_str(&content)?,
            ConfigFormat::Toml => toml::from_str(&content)?,
        };
        
        self.update_configurations(configs)?;
        self.start_file_watcher(path)?;
        
        Ok(())
    }
    
    /// 更新配置集合
    fn update_configurations(&self, new_configs: HashMap<String, ConfigValue>) -> Result<(), ConfigError> {
        let mut current_configs = self.configs.write().unwrap();
        
        // 验证所有新配置
        for (key, config) in &new_configs {
            config.validate()
                .map_err(|e| ConfigError::InvalidFormat(format!("Validation error in {}: {}", key, e)))?;
        }
        
        // 检测变化
        let changes = self.detect_changes(&current_configs, &new_configs);
        
        // 更新配置
        *current_configs = new_configs;
        
        // 发送变化通知
        if let Some(ref sender) = self.change_sender {
            for change in changes {
                if let Err(e) = sender.send(change) {
                    error!(self.logger, "Failed to send change event: {}", e);
                }
            }
        }
        
        // 通知所有监听器
        self.notify_watchers(&current_configs)?;
        
        Ok(())
    }
    
    /// 检测配置变化
    fn detect_changes(
        &self,
        current: &HashMap<String, ConfigValue>,
        new: &HashMap<String, ConfigValue>,
    ) -> Vec<ConfigChangeEvent> {
        let mut changes = Vec::new();
        
        // 检查新增和修改的键
        for (key, new_value) in new {
            match current.get(key) {
                Some(old_value) => {
                    if old_value != new_value {
                        changes.push(ConfigChangeEvent {
                            key: key.clone(),
                            old_value: Some(old_value.clone()),
                            new_value: Some(new_value.clone()),
                            change_type: ChangeType::Modified,
                        });
                    }
                }
                None => {
                    changes.push(ConfigChangeEvent {
                        key: key.clone(),
                        old_value: None,
                        new_value: Some(new_value.clone()),
                        change_type: ChangeType::Added,
                    });
                }
            }
        }
        
        // 检查删除的键
        for key in current.keys() {
            if !new.contains_key(key) {
                changes.push(ConfigChangeEvent {
                    key: key.clone(),
                    old_value: current.get(key).cloned(),
                    new_value: None,
                    change_type: ChangeType::Removed,
                });
            }
        }
        
        changes
    }
    
    /// 获取配置值
    pub fn get<T>(&self, key: &str) -> Result<T, ConfigError>
    where
        T: serde::de::DeserializeOwned,
    {
        let configs = self.configs.read().unwrap();
        let config_value = configs.get(key)
            .ok_or(ConfigError::KeyNotFound(key.to_string()))?;
        
        // 获取实际值（考虑环境变量覆盖）
        let actual_value = config_value.get_actual_value()?;
        let parsed_value: T = serde_json::from_value(actual_value)
            .map_err(|e| ConfigError::TypeConversionError(e.to_string()))?;
        
        Ok(parsed_value)
    }
    
    /// 获取所有配置键
    pub fn keys(&self) -> Vec<String> {
        let configs = self.configs.read().unwrap();
        configs.keys().cloned().collect()
    }
    
    /// 检查配置是否存在
    pub fn has(&self, key: &str) -> bool {
        let configs = self.configs.read().unwrap();
        configs.contains_key(key)
    }
    
    /// 设置配置值
    pub fn set(&mut self, key: String, value: ConfigValue) -> Result<(), ConfigError> {
        value.validate()?;
        
        let mut configs = self.configs.write().unwrap();
        let old_value = configs.get(&key).cloned();
        
        configs.insert(key.clone(), value.clone());
        
        // 发送变化通知
        if let Some(ref sender) = self.change_sender {
            let change = ConfigChangeEvent {
                key: key.clone(),
                old_value,
                new_value: Some(value),
                change_type: if old_value.is_some() { ChangeType::Modified } else { ChangeType::Added },
            };
            
            if let Err(e) = sender.send(change) {
                error!(self.logger, "Failed to send change event: {}", e);
            }
        }
        
        // 通知监听器
        if let Some(watcher) = configs.get(&key) {
            self.notify_single_watcher(&key, watcher)?;
        }
        
        Ok(())
    }
    
    /// 移除配置
    pub fn remove(&mut self, key: &str) -> Result<Option<ConfigValue>, ConfigError> {
        let mut configs = self.configs.write().unwrap();
        let removed_value = configs.remove(key);
        
        if let Some(ref removed) = removed_value {
            // 发送变化通知
            if let Some(ref sender) = self.change_sender {
                let change = ConfigChangeEvent {
                    key: key.to_string(),
                    old_value: Some(removed.clone()),
                    new_value: None,
                    change_type: ChangeType::Removed,
                };
                
                if let Err(e) = sender.send(change) {
                    error!(self.logger, "Failed to send change event: {}", e);
                }
            }
            
            // 通知监听器
            self.notify_watcher_removed(key)?;
        }
        
        Ok(removed_value)
    }
    
    /// 添加监听器
    pub fn add_watcher(&mut self, watcher: Box<dyn ConfigWatcher>) {
        let mut watchers = self.watchers.write().unwrap();
        watchers.push(watcher);
    }
    
    /// 移除监听器
    pub fn remove_watcher(&mut self, index: usize) {
        let mut watchers = self.watchers.write().unwrap();
        if index < watchers.len() {
            watchers.remove(index);
        }
    }
    
    /// 启动文件监视
    fn start_file_watcher<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ConfigError> {
        let path = path.as_ref().to_path_buf();
        let logger = self.logger.clone();
        
        let (tx, rx) = crossbeam::channel::unbounded();
        
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                if let Ok(event) = result {
                    if event.kind.is_modify() {
                        let _ = tx.send(event);
                    }
                }
            },
            notify::Config::default(),
        )?;
        
        watcher.watch(path.parent().unwrap(), RecursiveMode::NonRecursive)?;
        
        // 启动异步处理
        std::thread::spawn(move || {
            for event in rx {
                info!(logger, "File change detected: {:?}", event.paths);
                
                // 重新加载配置
                // 这里可以添加重试逻辑和错误处理
            }
        });
        
        self.watcher = Some(watcher);
        Ok(())
    }
    
    /// 停止文件监视
    pub fn stop_file_watcher(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            let _ = watcher.unwatch(&std::path::Path::new("."));
        }
    }
    
    /// 获取变化事件接收器
    pub fn get_change_receiver(&self) -> Option<Receiver<ConfigChangeEvent>> {
        self.change_sender.as_ref().map(|sender| sender.subscribe())
    }
    
    /// 通知所有监听器
    fn notify_watchers(&self, configs: &HashMap<String, ConfigValue>) -> Result<(), ConfigError> {
        let watchers = self.watchers.read().unwrap();
        let notify_tasks: Vec<_> = watchers
            .par_iter()
            .map(|watcher| {
                for (key, config) in configs {
                    watcher.on_config_change(key, config);
                }
                Ok::<(), ConfigError>(())
            })
            .collect();
        
        for result in notify_tasks {
            result?;
        }
        
        Ok(())
    }
    
    /// 通知单个监听器
    fn notify_single_watcher(&self, key: &str, config: &ConfigValue) -> Result<(), ConfigError> {
        let watchers = self.watchers.read().unwrap();
        for watcher in watchers.iter() {
            watcher.on_config_change(key, config);
        }
        Ok(())
    }
    
    /// 通知监听器配置被移除
    fn notify_watcher_removed(&self, key: &str) -> Result<(), ConfigError> {
        let watchers = self.watchers.read().unwrap();
        for watcher in watchers.iter() {
            watcher.on_config_removed(key);
        }
        Ok(())
    }
    
    /// 导出配置为JSON
    pub fn export_json(&self) -> Result<String, ConfigError> {
        let configs = self.configs.read().unwrap();
        let export_data: HashMap<String, JsonValue> = configs
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect();
        
        Ok(serde_json::to_string_pretty(&export_data)?)
    }
    
    /// 验证所有配置
    pub fn validate_all(&self) -> Result<(), ValidationError> {
        let configs = self.configs.read().unwrap();
        for (key, config) in configs {
            if let Err(error) = config.validate() {
                return Err(error);
            }
        }
        Ok(())
    }
}

impl Drop for ConfigManager {
    fn drop(&mut self) {
        self.stop_file_watcher();
    }
}
```

## 4.6 本章总结

本章深入探讨了Rust中结构体和枚举的强大功能，这是构建复杂应用程序的基础。通过本章的学习，您已经：

1. **掌握了结构体基础**：定义了各种类型的结构体，包括元组结构体和泛型结构体
2. **学会了方法设计**：区分了关联函数和方法的用法
3. **了解了枚举威力**：从简单的枚举到复杂的携带数据的枚举
4. **掌握了模式匹配**：学会了使用match表达式进行复杂的模式匹配
5. **构建了实用项目**：开发了企业级配置管理工具

结构体和枚举为Rust提供了强大的数据建模能力，使得开发者能够创建类型安全、表达力强的代码。这些概念在实际的Rust开发中无处不在，是掌握Rust编程的必备知识。

## 4.7 验收标准

完成本章后，您应该能够：

- [ ] 设计合理的结构体来建模业务数据
- [ ] 实现结构体的方法和关联函数
- [ ] 使用枚举精确建模状态和选项
- [ ] 编写复杂的模式匹配代码
- [ ] 实现生产级的配置管理系统
- [ ] 设计可扩展的数据验证框架

## 4.8 练习题

1. **设计Employee结构体**：创建一个Employee结构体，包含姓名、职位、薪资等字段
2. **实现状态机**：使用枚举实现一个游戏状态机
3. **配置验证器**：为配置系统添加更多验证规则
4. **模式匹配优化**：重构代码以使用更简洁的模式匹配
5. **性能对比测试**：比较不同数据结构实现的性能差异

## 4.9 扩展阅读

- [Rust官方文档：结构体](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [Rust官方文档：枚举和模式匹配](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [Rust与模式匹配](https://doc.rust-lang.org/book/ch18-00-patterns.html)