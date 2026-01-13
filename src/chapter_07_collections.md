# 第7章：集合类型与数据结构

## 目录
1. [引言](#引言)
2. [Vector（Vec<T>）深入理解](#vector深入理解)
3. [HashMap与HashSet详解](#hashmap与hashset详解)
4. [迭代器与闭包的深入应用](#迭代器与闭包的深入应用)
5. [其他重要集合类型](#其他重要集合类型)
6. [实战项目1：Todo管理器](#实战项目1todo管理器)
7. [实战项目2：Web API服务器](#实战项目2web-api服务器)
8. [性能优化与最佳实践](#性能优化与最佳实践)
9. [总结](#总结)

---

## 引言

集合类型是任何编程语言的核心，Rust提供了丰富的集合类型来满足不同的数据存储和操作需求。在本章中，我们将深入学习：

- **Vector（Vec<T>）**：动态数组，支持随机访问和高效追加
- **HashMap与HashSet**：哈希表实现，提供O(1)查找性能
- **迭代器**：函数式编程的核心工具
- **性能考虑**：何时使用哪种集合类型

通过两个实战项目（Todo管理器和Web API服务器），我们将学习如何在实际应用中高效使用这些集合类型。

### 本章学习目标

完成本章学习后，你将能够：
- 熟练使用各种Rust集合类型
- 理解不同集合类型的性能特征
- 设计高效的集合操作策略
- 构建基于集合的复杂应用

---

## Vector深入理解

### Vector基础

Vector（动态数组）是Rust中最常用的集合类型，提供了动态大小的连续内存存储。

```rust
// 创建Vector的多种方式
fn main() {
    // 1. 使用vec!宏创建
    let mut numbers = vec![1, 2, 3, 4, 5];
    
    // 2. 动态创建空Vector并添加元素
    let mut names: Vec<String> = Vec::new();
    names.push("Alice".to_string());
    names.push("Bob".to_string());
    
    // 3. 预分配空间
    let mut buffer = Vec::with_capacity(1000);
    
    // 4. 使用迭代器创建
    let squares: Vec<i32> = (1..=10).map(|x| x * x).collect();
    
    println!("Numbers: {:?}", numbers);
    println!("Names: {:?}", names);
    println!("Buffer capacity: {}", buffer.capacity());
    println!("Squares: {:?}", squares);
}
```

### Vector核心操作

```rust
use std::collections::BTreeMap;

fn vector_operations_demo() {
    let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6];
    
    // 1. 访问元素
    println!("First element: {}", data[0]); // 索引访问
    println!("First element (safe): {:?}", data.get(0)); // 安全访问
    println!("Last element: {}", data[data.len() - 1]);
    
    // 2. 修改元素
    data[2] = 10;
    println!("Modified data: {:?}", data);
    
    // 3. 添加和删除
    data.push(15);         // 添加到末尾
    data.insert(3, 7);     // 插入到指定位置
    let popped = data.pop(); // 从末尾删除
    let removed = data.remove(1); // 删除指定位置的元素
    
    println!("After modifications: {:?}", data);
    println!("Popped: {:?}", popped);
    println!("Removed: {:?}", removed);
    
    // 4. 切片操作
    let slice = &data[2..=5];
    println!("Slice: {:?}", slice);
    
    // 5. 查找元素
    if let Some(&index) = data.iter().position(|&x| x == 10) {
        println!("Found 10 at index: {}", index);
    }
    
    // 6. 排序
    data.sort();
    println!("Sorted: {:?}", data);
    
    // 7. 去重
    data.dedup();
    println!("After dedup: {:?}", data);
}
```

### Vector内存管理

```rust
fn vector_memory_management() {
    // 1. 预分配容量优化
    let start = std::time::Instant::now();
    let mut unoptimized = Vec::new();
    for i in 0..10000 {
        unoptimized.push(i);
    }
    let unoptimized_time = start.elapsed();
    
    // 2. 预分配容量
    let start = std::time::Instant::now();
    let mut optimized = Vec::with_capacity(10000);
    for i in 0..10000 {
        optimized.push(i);
    }
    let optimized_time = start.elapsed();
    
    println!("Unoptimized time: {:?}", unoptimized_time);
    println!("Optimized time: {:?}", optimized_time);
    println!("Capacity: {}, Length: {}", optimized.capacity(), optimized.len());
    
    // 3. 收缩到合适大小
    optimized.shrink_to_fit();
    println!("After shrink_to_fit - Capacity: {}", optimized.capacity());
    
    // 4. 保留指定容量
    optimized.reserve(5000);
    println!("After reserve(5000) - Capacity: {}", optimized.capacity());
}
```

### Vector性能分析

```rust
fn vector_performance_analysis() {
    use std::time::{Duration, Instant};
    
    // 1. 顺序访问性能
    let large_vec: Vec<i32> = (0..1_000_000).collect();
    
    let start = Instant::now();
    for i in 0..large_vec.len() {
        let _ = large_vec[i];
    }
    let sequential_time = start.elapsed();
    
    // 2. 迭代器访问性能
    let start = Instant::now();
    for value in &large_vec {
        let _ = *value;
    }
    let iterator_time = start.elapsed();
    
    println!("Sequential access: {:?}", sequential_time);
    println!("Iterator access: {:?}", iterator_time);
    
    // 3. 预分配vs动态增长
    let iterations = 1000;
    let batch_size = 1000;
    
    // 不预分配
    let start = Instant::now();
    let mut vec1 = Vec::new();
    for _ in 0..iterations {
        for i in 0..batch_size {
            vec1.push(i);
        }
    }
    let no_prealloc_time = start.elapsed();
    
    // 预分配
    let start = Instant::now();
    let mut vec2 = Vec::with_capacity(iterations * batch_size);
    for _ in 0..iterations {
        for i in 0..batch_size {
            vec2.push(i);
        }
    }
    let prealloc_time = start.elapsed();
    
    println!("No pre-allocation: {:?}", no_prealloc_time);
    println!("With pre-allocation: {:?}", prealloc_time);
}
```

---

## HashMap与HashSet详解

### HashMap基础

HashMap是Rust中最重要的键值对存储结构，提供O(1)平均时间复杂度的查找、插入和删除操作。

```rust
use std::collections::HashMap;
use std::collections::HashSet;

fn hashmap_basic_operations() {
    // 1. 创建HashMap
    let mut scores = HashMap::new();
    let mut settings = HashMap::from([
        ("theme", "dark"),
        ("language", "Rust"),
        ("editor", "VSCode")
    ]);
    
    // 2. 插入键值对
    scores.insert("Blue", 10);
    scores.insert("Red", 50);
    scores.insert("Green", 25);
    
    // 3. 访问值
    println!("Blue score: {:?}", scores.get("Blue"));
    println!("All scores: {:?}", scores);
    
    // 4. 批量插入
    let additional_scores = vec![
        ("Yellow", 30),
        ("Purple", 40)
    ];
    scores.extend(additional_scores);
    
    // 5. 检查键是否存在
    if scores.contains_key("Red") {
        println!("Red team exists");
    }
    
    // 6. 获取并更新
    let old_value = scores.insert("Red", 60);
    println!("Old Red value: {:?}", old_value);
    
    // 7. 键值对计数
    println!("Number of teams: {}", scores.len());
}
```

### HashMap高级操作

```rust
fn hashmap_advanced_operations() {
    let mut inventory = HashMap::new();
    
    // 1. 条件插入（仅当键不存在时）
    inventory.entry("widget").or_insert(0);
    inventory.entry("gadget").or_insert_with(|| 10);
    
    // 2. 修改现有值
    {
        let count = inventory.entry("widget").or_insert(0);
        *count += 5;
    }
    
    // 3. 使用Entry API
    for (item, count) in &inventory {
        println!("{}: {}", item, count);
    }
    
    // 4. 移除键值对
    if let Some(removed_value) = inventory.remove("gadget") {
        println!("Removed gadget with count: {}", removed_value);
    }
    
    // 5. 过滤操作
    let high_inventory: HashMap<String, i32> = inventory
        .iter()
        .filter(|(&k, &v)| v > 5)
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    
    println!("High inventory: {:?}", high_inventory);
    
    // 6. 聚合操作
    let total_items: i32 = inventory.values().sum();
    let unique_items = inventory.keys().len();
    
    println!("Total items: {}", total_items);
    println!("Unique items: {}", unique_items);
}
```

### HashSet详解

HashSet是基于HashMap实现的集合类型，用于存储唯一的值。

```rust
fn hashset_operations() {
    // 1. 创建HashSet
    let mut colors: HashSet<String> = HashSet::new();
    let predefined_colors = vec![
        "red".to_string(),
        "green".to_string(),
        "blue".to_string()
    ];
    let mut color_set: HashSet<String> = predefined_colors.into_iter().collect();
    
    // 2. 添加元素
    colors.insert("yellow".to_string());
    colors.insert("red".to_string()); // 不会重复添加
    colors.insert("blue".to_string());
    
    // 3. 集合操作
    let set1: HashSet<i32> = vec![1, 2, 3, 4, 5].into_iter().collect();
    let set2: HashSet<i32> = vec![3, 4, 5, 6, 7].into_iter().collect();
    
    // 并集
    let union: HashSet<i32> = set1.union(&set2).cloned().collect();
    println!("Union: {:?}", union);
    
    // 交集
    let intersection: HashSet<i32> = set1.intersection(&set2).cloned().collect();
    println!("Intersection: {:?}", intersection);
    
    // 差集
    let difference: HashSet<i32> = set1.difference(&set2).cloned().collect();
    println!("Difference: {:?}", difference);
    
    // 对称差集
    let symmetric_difference: HashSet<i32> = 
        set1.symmetric_difference(&set2).cloned().collect();
    println!("Symmetric difference: {:?}", symmetric_difference);
    
    // 4. 集合关系
    println!("set1 is subset of set2: {}", set1.is_subset(&set2));
    println!("set1 is superset of set2: {}", set1.is_superset(&set2));
    println!("set1 is disjoint with set2: {}", set1.is_disjoint(&set2));
}
```

### 自定义类型作为HashMap键

```rust
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Person {
    id: u32,
    name: String,
    email: String,
}

impl Hash for Person {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.email.hash(state);
    }
}

fn custom_type_as_key() {
    let mut people_db = HashMap::new();
    
    let person1 = Person {
        id: 1,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    let person2 = Person {
        id: 2,
        name: "Bob Smith".to_string(),
        email: "bob@example.com".to_string(),
    };
    
    people_db.insert(person1, "Developer");
    people_db.insert(person2.clone(), "Designer");
    
    // 查找
    if let Some(role) = people_db.get(&person2) {
        println!("Bob's role: {}", role);
    }
    
    // 计算哈希值
    let hasher = DefaultHasher::new();
    let hash = person2.hash(hasher);
    println!("Bob's hash: {}", hash);
}
```

---

## 迭代器与闭包的深入应用

### 迭代器基础

```rust
fn iterator_basics() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    // 1. 基本迭代
    for num in numbers.iter() {
        println!("Number: {}", num);
    }
    
    // 2. 消费迭代器
    let sum: i32 = numbers.iter().sum();
    println!("Sum: {}", sum);
    
    // 3. 映射操作
    let squares: Vec<i32> = numbers
        .iter()
        .map(|&x| x * x)
        .collect();
    println!("Squares: {:?}", squares);
    
    // 4. 过滤操作
    let evens: Vec<&i32> = numbers
        .iter()
        .filter(|&&x| x % 2 == 0)
        .collect();
    println!("Even numbers: {:?}", evens);
    
    // 5. 链式操作
    let result: Vec<i32> = numbers
        .iter()
        .filter(|&&x| x > 2)
        .map(|&x| x * 2)
        .collect();
    println!("Doubled and filtered: {:?}", result);
    
    // 6. 查找操作
    if let Some(&first_even) = numbers.iter().find(|&&x| x % 2 == 0) {
        println!("First even number: {}", first_even);
    }
    
    // 7. 位置查找
    if let Some(position) = numbers.iter().position(|&&x| x == 4) {
        println!("4 is at position: {}", position);
    }
}
```

### 复杂迭代器模式

```rust
fn complex_iterator_patterns() {
    let data = vec![
        ("Alice", 25, "Engineer"),
        ("Bob", 30, "Designer"),
        ("Charlie", 35, "Manager"),
        ("Diana", 28, "Engineer"),
    ];
    
    // 1. 元组解构迭代
    let engineers: Vec<&str> = data
        .iter()
        .filter(|(_, _, role)| *role == "Engineer")
        .map(|(name, _, _)| *name)
        .collect();
    println!("Engineers: {:?}", engineers);
    
    // 2. 分组操作
    let mut age_groups = HashMap::new();
    for (name, age, role) in &data {
        age_groups
            .entry(if *age < 30 { "young" } else { "experienced" })
            .or_insert_with(Vec::new)
            .push((name, age, role));
    }
    println!("Age groups: {:?}", age_groups);
    
    // 3. 累积操作
    let total_ages: usize = data
        .iter()
        .map(|(_, age, _)| *age)
        .fold(0, |acc, age| acc + age as usize);
    let average_age = total_ages / data.len();
    println!("Average age: {}", average_age);
    
    // 4. 嵌套迭代
    let combos: Vec<_> = data
        .iter()
        .flat_map(|(name1, _, _)| {
            data.iter()
                .filter(move |(name2, _, _)| name1 != name2)
                .map(move |(name2, _, _)| format!("{} - {}", name1, name2))
        })
        .collect();
    println!("Name combinations: {:?}", combos);
}
```

### 闭包与迭代器

```rust
fn closures_with_iterators() {
    // 1. 捕获环境的闭包
    let threshold = 30;
    let numbers = vec![10, 25, 35, 40, 55];
    
    let above_threshold: Vec<i32> = numbers
        .iter()
        .filter(|&&x| {
            let condition = x > threshold;
            println!("Checking {} > {}: {}", x, threshold, condition);
            condition
        })
        .map(|&x| {
            let processed = x * 2;
            println!("Processing {} -> {}", x, processed);
            processed
        })
        .collect();
    println!("Above threshold (doubled): {:?}", above_threshold);
    
    // 2. 高阶函数模式
    let numbers = vec![1, 2, 3, 4, 5];
    
    // 创建通用的数据处理函数
    let process_data = |data: &Vec<i32>, filter: &dyn Fn(&i32) -> bool, 
                       transform: &dyn Fn(&i32) -> i32| -> Vec<i32> {
        data
            .iter()
            .filter(filter)
            .map(transform)
            .collect()
    };
    
    let evens_squared = process_data(
        &numbers,
        |&x| x % 2 == 0,
        |&x| x * x
    );
    
    let odds_cubed = process_data(
        &numbers,
        |&x| x % 2 == 1,
        |&x| x * x * x
    );
    
    println!("Evens squared: {:?}", evens_squared);
    println!("Odds cubed: {:?}", odds_cubed);
}
```

### 惰性计算与性能

```rust
fn lazy_evaluation_performance() {
    use std::time::Instant;
    
    // 1. 惰性迭代器
    let large_data: Vec<i32> = (1..=1_000_000).collect();
    
    // 计算1到1000000之间所有偶数的平方
    let start = Instant::now();
    let result: Vec<i32> = large_data
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .take(5) // 只取前5个结果
        .collect();
    let lazy_time = start.elapsed();
    
    println!("Lazy evaluation result: {:?}", result);
    println!("Lazy evaluation time: {:?}", lazy_time);
    
    // 2. 早期退出
    let start = Instant::now();
    let first_large_square = large_data
        .iter()
        .filter(|&&x| x % 2 == 0)
        .find(|&&x| x > 1000)
        .map(|&x| x * x);
    let early_exit_time = start.elapsed();
    
    println!("First large square: {:?}", first_large_square);
    println!("Early exit time: {:?}", early_exit_time);
    
    // 3. 链式操作优化
    let start = Instant::now();
    let chain_result: Vec<i32> = large_data
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| {
            // 模拟一个昂贵的操作
            std::thread::sleep(std::time::Duration::from_millis(1));
            x * x
        })
        .take(10)
        .collect();
    let chain_time = start.elapsed();
    
    println!("Chain operation time: {:?}", chain_time);
}
```

---

## 其他重要集合类型

### BTreeMap与BTreeSet

```rust
use std::collections::{BTreeMap, BTreeSet};

fn btree_collections() {
    // 1. BTreeMap - 保持键的排序
    let mut btree_map: BTreeMap<String, i32> = BTreeMap::new();
    
    btree_map.insert("Charlie".to_string(), 35);
    btree_map.insert("Alice".to_string(), 25);
    btree_map.insert("Bob".to_string(), 30);
    
    println!("BTreeMap (sorted by key):");
    for (name, age) in &btree_map {
        println!("  {}: {}", name, age);
    }
    
    // 2. 范围查询
    let range: BTreeMap<String, i32> = btree_map
        .range("Alice".."Charlie")
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    println!("Range Alice..Charlie: {:?}", range);
    
    // 3. BTreeSet
    let mut btree_set: BTreeSet<i32> = BTreeSet::new();
    btree_set.insert(5);
    btree_set.insert(1);
    btree_set.insert(3);
    btree_set.insert(2);
    btree_set.insert(4);
    
    println!("BTreeSet (sorted): {:?}", btree_set);
    
    // 4. 范围查询
    let range_set: BTreeSet<&i32> = btree_set.range(2..=4).collect();
    println!("Range 2..=4: {:?}", range_set);
}
```

### 栈和队列

```rust
use std::collections::VecDeque;

fn stack_queue_operations() {
    // 1. 栈 (Vec)
    let mut stack = Vec::new();
    stack.push(1);
    stack.push(2);
    stack.push(3);
    
    println!("Stack: {:?}", stack);
    println!("Top: {:?}", stack.pop());
    println!("After pop: {:?}", stack);
    
    // 2. 队列 (VecDeque)
    let mut queue = VecDeque::new();
    queue.push_back(1);
    queue.push_back(2);
    queue.push_back(3);
    
    println!("Queue: {:?}", queue);
    println!("Front: {:?}", queue.pop_front());
    println!("Back: {:?}", queue.pop_back());
    println!("After operations: {:?}", queue);
    
    // 3. 双向队列
    let mut deque = VecDeque::new();
    deque.push_front(3);
    deque.push_front(2);
    deque.push_front(1);
    deque.push_back(4);
    deque.push_back(5);
    
    println!("Deque: {:?}", deque);
    
    // 4. 滑动窗口算法
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let window_size = 3;
    
    let windows: Vec<Vec<i32>> = numbers
        .windows(window_size)
        .map(|window| window.to_vec())
        .collect();
    
    println!("Sliding windows: {:?}", windows);
    
    // 5. 移动窗口
    let chunks: Vec<Vec<i32>> = numbers
        .chunks(window_size)
        .map(|chunk| chunk.to_vec())
        .collect();
    
    println!("Fixed chunks: {:?}", chunks);
}
```

### 优先级队列

```rust
use std::cmp::Reverse;
use std::collections::BinaryHeap;

fn priority_queue_demo() {
    // 1. 最大堆（默认）
    let mut max_heap = BinaryHeap::new();
    max_heap.push(10);
    max_heap.push(5);
    max_heap.push(20);
    max_heap.push(15);
    
    println!("Max heap:");
    while let Some(num) = max_heap.pop() {
        println!("  {}", num);
    }
    
    // 2. 最小堆（使用Reverse）
    let mut min_heap = BinaryHeap::new();
    min_heap.push(Reverse(10));
    min_heap.push(Reverse(5));
    min_heap.push(Reverse(20));
    min_heap.push(Reverse(15));
    
    println!("\nMin heap:");
    while let Some(Reverse(num)) = min_heap.pop() {
        println!("  {}", num);
    }
    
    // 3. 任务调度器
    #[derive(Debug, PartialEq, Eq)]
    struct Task {
        priority: u8,
        description: String,
    }
    
    impl PartialOrd for Task {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.priority.cmp(&other.priority).reverse())
        }
    }
    
    impl Ord for Task {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.priority.cmp(&other.priority).reverse()
        }
    }
    
    let mut task_queue = BinaryHeap::new();
    task_queue.push(Task {
        priority: 3,
        description: "Low priority task".to_string(),
    });
    task_queue.push(Task {
        priority: 1,
        description: "High priority task".to_string(),
    });
    task_queue.push(Task {
        priority: 2,
        description: "Medium priority task".to_string(),
    });
    
    println!("\nTask execution order:");
    while let Some(task) = task_queue.pop() {
        println!("  {}: {}", task.priority, task.description);
    }
}
```

---

## 实战项目1：Todo管理器

### 项目概述

我们将构建一个功能完整的Todo管理器，包含以下功能：
- 添加、编辑、删除待办事项
- 标记完成/未完成状态
- 按优先级、截止日期、状态分类
- 数据持久化到JSON文件
- 命令行界面

### 项目结构

```rust
// src/main.rs
use std::env;
use std::fs;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub priority: u8, // 1-5, 5为最高优先级
    pub due_date: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct TodoManager {
    pub todos: Vec<Todo>,
    pub next_id: u32,
    pub file_path: PathBuf,
}

impl TodoManager {
    pub fn new() -> Self {
        let mut manager = TodoManager {
            todos: Vec::new(),
            next_id: 1,
            file_path: PathBuf::from("todos.json"),
        };
        
        // 尝试加载已存在的数据
        if manager.file_path.exists() {
            let _ = manager.load_from_file();
        }
        
        manager
    }
    
    pub fn add_todo(&mut self, title: String, description: Option<String>, 
                    priority: u8, due_date: Option<String>, tags: Vec<String>) {
        let now = chrono::Utc::now().to_rfc3339();
        
        let todo = Todo {
            id: self.next_id,
            title,
            description,
            completed: false,
            priority,
            due_date,
            tags,
            created_at: now.clone(),
            updated_at: now,
        };
        
        self.todos.push(todo);
        self.next_id += 1;
        let _ = self.save_to_file();
    }
    
    pub fn list_todos(&self) {
        if self.todos.is_empty() {
            println!("暂无待办事项");
            return;
        }
        
        println!("\n=== 待办事项列表 ===");
        for todo in &self.todos {
            let status = if todo.completed { "✓" } else { "○" };
            let priority_stars = "★".repeat(todo.priority as usize);
            println!("[{}] {} {} - {} (ID: {})", 
                    status, 
                    priority_stars, 
                    todo.title, 
                    if todo.completed { "已完成" } else { "进行中" },
                    todo.id);
            
            if let Some(ref desc) = todo.description {
                println!("    描述: {}", desc);
            }
            
            if let Some(ref due) = todo.due_date {
                println!("    截止日期: {}", due);
            }
            
            if !todo.tags.is_empty() {
                println!("    标签: {}", todo.tags.join(", "));
            }
            
            println!("    创建: {}, 更新: {}", todo.created_at, todo.updated_at);
            println!();
        }
    }
    
    pub fn complete_todo(&mut self, id: u32) -> Result<(), String> {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = true;
            todo.updated_at = chrono::Utc::now().to_rfc3339();
            let _ = self.save_to_file();
            Ok(())
        } else {
            Err(format!("未找到ID为 {} 的待办事项", id))
        }
    }
    
    pub fn uncomplete_todo(&mut self, id: u32) -> Result<(), String> {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = false;
            todo.updated_at = chrono::Utc::now().to_rfc3339();
            let _ = self.save_to_file();
            Ok(())
        } else {
            Err(format!("未找到ID为 {} 的待办事项", id))
        }
    }
    
    pub fn update_todo(&mut self, id: u32, title: Option<String>, 
                      description: Option<String>, priority: Option<u8>, 
                      due_date: Option<Option<String>>, tags: Option<Vec<String>>) -> Result<(), String> {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            if let Some(new_title) = title {
                todo.title = new_title;
            }
            if let Some(new_desc) = description {
                todo.description = Some(new_desc);
            }
            if let Some(new_priority) = priority {
                todo.priority = new_priority;
            }
            if let Some(new_due_date) = due_date {
                todo.due_date = new_due_date;
            }
            if let Some(new_tags) = tags {
                todo.tags = new_tags;
            }
            
            todo.updated_at = chrono::Utc::now().to_rfc3339();
            let _ = self.save_to_file();
            Ok(())
        } else {
            Err(format!("未找到ID为 {} 的待办事项", id))
        }
    }
    
    pub fn delete_todo(&mut self, id: u32) -> Result<(), String> {
        if let Some(index) = self.todos.iter().position(|t| t.id == id) {
            self.todos.remove(index);
            let _ = self.save_to_file();
            Ok(())
        } else {
            Err(format!("未找到ID为 {} 的待办事项", id))
        }
    }
    
    pub fn filter_by_status(&self, completed: bool) -> Vec<&Todo> {
        self.todos.iter().filter(|t| t.completed == completed).collect()
    }
    
    pub fn filter_by_priority(&self, priority: u8) -> Vec<&Todo> {
        self.todos.iter().filter(|t| t.priority == priority).collect()
    }
    
    pub fn search_by_tag(&self, tag: &str) -> Vec<&Todo> {
        self.todos.iter().filter(|t| t.tags.contains(&tag.to_string())).collect()
    }
    
    pub fn search_by_keyword(&self, keyword: &str) -> Vec<&Todo> {
        let keyword = keyword.to_lowercase();
        self.todos.iter()
            .filter(|t| {
                t.title.to_lowercase().contains(&keyword) ||
                t.description.as_ref().map(|d| d.to_lowercase().contains(&keyword)).unwrap_or(false)
            })
            .collect()
    }
    
    pub fn sort_by_priority(&self) -> Vec<&Todo> {
        let mut sorted = self.todos.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted
    }
    
    pub fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.todos)?;
        fs::write(&self.file_path, json)?;
        Ok(())
    }
    
    pub fn load_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&self.file_path)?;
        self.todos = serde_json::from_str(&content)?;
        
        // 更新下一个ID
        if let Some(max_id) = self.todos.iter().map(|t| t.id).max() {
            self.next_id = max_id + 1;
        }
        
        Ok(())
    }
    
    pub fn export_csv(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(file_path)?;
        
        // 写入表头
        wtr.write_record(&["ID", "标题", "描述", "完成状态", "优先级", "截止日期", "标签", "创建时间", "更新时间"])?;
        
        for todo in &self.todos {
            wtr.write_record(&[
                &todo.id.to_string(),
                &todo.title,
                todo.description.as_deref().unwrap_or(""),
                if todo.completed { "已完成" } else { "进行中" },
                &todo.priority.to_string(),
                todo.due_date.as_deref().unwrap_or(""),
                &todo.tags.join(";"),
                &todo.created_at,
                &todo.updated_at
            ])?;
        }
        
        wtr.flush()?;
        Ok(())
    }
    
    pub fn get_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        stats.insert("总数量".to_string(), self.todos.len());
        stats.insert("已完成".to_string(), 
                    self.todos.iter().filter(|t| t.completed).count());
        stats.insert("进行中".to_string(), 
                    self.todos.iter().filter(|t| !t.completed).count());
        
        // 按优先级统计
        for priority in 1..=5 {
            let count = self.todos.iter().filter(|t| t.priority == priority).count();
            if count > 0 {
                stats.insert(format!("优先级{}", priority), count);
            }
        }
        
        stats
    }
}

fn print_help() {
    println!("\n=== Todo管理器命令 ===");
    println!("add <标题> [描述] - 添加待办事项");
    println!("list - 列出所有待办事项");
    println!("complete <ID> - 标记为已完成");
    println!("uncomplete <ID> - 标记为进行中");
    println!("update <ID> [选项] - 更新待办事项");
    println!("delete <ID> - 删除待办事项");
    println!("filter <状态> - 按状态筛选 (completed/uncompleted)");
    println!("priority <优先级> - 按优先级筛选 (1-5)");
    println!("search <关键词> - 搜索待办事项");
    println!("tag <标签> - 按标签搜索");
    println!("sort - 按优先级排序显示");
    println!("stats - 显示统计信息");
    println!("export <文件名> - 导出为CSV");
    println!("help - 显示帮助信息");
    println!("quit - 退出程序");
    println!("\n示例:");
    println!("  add \"完成项目报告\" \"需要包含Q3数据\" 4 \"2023-12-31\" work,urgent");
    println!("  list");
    println!("  complete 1");
    println!("  filter completed");
    println!("  update 1 --priority 5 --tag urgent");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = TodoManager::new();
    
    println!("欢迎使用Todo管理器！输入 'help' 查看帮助信息。");
    
    loop {
        print!("\n> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let command = parts[0];
        
        match command {
            "quit" | "exit" => {
                println!("再见！");
                break;
            }
            
            "help" => {
                print_help();
            }
            
            "add" => {
                if parts.len() < 2 {
                    println!("用法: add <标题> [描述] [优先级] [截止日期] [标签]");
                    continue;
                }
                
                let title = parts[1].to_string();
                let description = if parts.len() > 2 { 
                    Some(parts[2].to_string()) 
                } else { None };
                
                let priority = if parts.len() > 3 { 
                    parts[3].parse::<u8>().unwrap_or(3) 
                } else { 3 };
                
                let due_date = if parts.len() > 4 { 
                    Some(parts[4].to_string()) 
                } else { None };
                
                let tags = if parts.len() > 5 { 
                    parts[5].split(',').map(|s| s.trim().to_string()).collect()
                } else { Vec::new() };
                
                manager.add_todo(title, description, priority, due_date, tags);
                println!("✅ 已添加待办事项");
            }
            
            "list" => {
                manager.list_todos();
            }
            
            "complete" => {
                if parts.len() < 2 {
                    println!("用法: complete <ID>");
                    continue;
                }
                
                if let Ok(id) = parts[1].parse::<u32>() {
                    match manager.complete_todo(id) {
                        Ok(_) => println!("✅ 已标记为已完成"),
                        Err(e) => println!("❌ {}", e),
                    }
                } else {
                    println!("❌ 无效的ID");
                }
            }
            
            "uncomplete" => {
                if parts.len() < 2 {
                    println!("用法: uncomplete <ID>");
                    continue;
                }
                
                if let Ok(id) = parts[1].parse::<u32>() {
                    match manager.uncomplete_todo(id) {
                        Ok(_) => println!("✅ 已标记为进行中"),
                        Err(e) => println!("❌ {}", e),
                    }
                } else {
                    println!("❌ 无效的ID");
                }
            }
            
            "update" => {
                if parts.len() < 2 {
                    println!("用法: update <ID> [选项]");
                    continue;
                }
                
                if let Ok(id) = parts[1].parse::<u32>() {
                    let mut title = None;
                    let mut description = None;
                    let mut priority = None;
                    let mut due_date = None;
                    let mut tags = None;
                    
                    for i in (2..parts.len()).step_by(2) {
                        if i + 1 < parts.len() {
                            match parts[i] {
                                "--title" => title = Some(parts[i + 1].to_string()),
                                "--description" => description = Some(parts[i + 1].to_string()),
                                "--priority" => {
                                    if let Ok(p) = parts[i + 1].parse::<u8>() {
                                        priority = Some(p);
                                    }
                                }
                                "--due-date" => due_date = Some(Some(parts[i + 1].to_string())),
                                "--tags" => {
                                    let tag_list: Vec<String> = parts[i + 1]
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .collect();
                                    tags = Some(tag_list);
                                }
                                _ => {}
                            }
                        }
                    }
                    
                    match manager.update_todo(id, title, description, priority, due_date, tags) {
                        Ok(_) => println!("✅ 已更新待办事项"),
                        Err(e) => println!("❌ {}", e),
                    }
                } else {
                    println!("❌ 无效的ID");
                }
            }
            
            "delete" => {
                if parts.len() < 2 {
                    println!("用法: delete <ID>");
                    continue;
                }
                
                if let Ok(id) = parts[1].parse::<u32>() {
                    match manager.delete_todo(id) {
                        Ok(_) => println!("✅ 已删除待办事项"),
                        Err(e) => println!("❌ {}", e),
                    }
                } else {
                    println!("❌ 无效的ID");
                }
            }
            
            "filter" => {
                if parts.len() < 2 {
                    println!("用法: filter <completed|uncompleted>");
                    continue;
                }
                
                match parts[1] {
                    "completed" => {
                        let completed_todos = manager.filter_by_status(true);
                        println!("\n=== 已完成的待办事项 ===");
                        for todo in completed_todos {
                            println!("[✓] {} (ID: {})", todo.title, todo.id);
                        }
                    }
                    "uncompleted" => {
                        let uncompleted_todos = manager.filter_by_status(false);
                        println!("\n=== 进行中的待办事项 ===");
                        for todo in uncompleted_todos {
                            println!("[○] {} (ID: {})", todo.title, todo.id);
                        }
                    }
                    _ => {
                        println!("❌ 无效的筛选条件，使用 completed 或 uncompleted");
                    }
                }
            }
            
            "priority" => {
                if parts.len() < 2 {
                    println!("用法: priority <1-5>");
                    continue;
                }
                
                if let Ok(priority) = parts[1].parse::<u8>() {
                    if priority >= 1 && priority <= 5 {
                        let priority_todos = manager.filter_by_priority(priority);
                        println!("\n=== 优先级{}的待办事项 ===", priority);
                        for todo in priority_todos {
                            let stars = "★".repeat(todo.priority as usize);
                            let status = if todo.completed { "✓" } else { "○" };
                            println!("[{}] {} {} (ID: {})", status, stars, todo.title, todo.id);
                        }
                    } else {
                        println!("❌ 优先级必须在1-5之间");
                    }
                } else {
                    println!("❌ 无效的优先级");
                }
            }
            
            "search" => {
                if parts.len() < 2 {
                    println!("用法: search <关键词>");
                    continue;
                }
                
                let keyword = parts[1];
                let results = manager.search_by_keyword(keyword);
                
                if results.is_empty() {
                    println!("未找到包含 '{}' 的待办事项", keyword);
                } else {
                    println!("\n=== 搜索结果: '{}' ===", keyword);
                    for todo in results {
                        let status = if todo.completed { "✓" } else { "○" };
                        println!("[{}] {} (ID: {})", status, todo.title, todo.id);
                        if let Some(ref desc) = todo.description {
                            println!("    描述: {}", desc);
                        }
                    }
                }
            }
            
            "tag" => {
                if parts.len() < 2 {
                    println!("用法: tag <标签>");
                    continue;
                }
                
                let tag = parts[1];
                let results = manager.search_by_tag(tag);
                
                if results.is_empty() {
                    println!("未找到标签为 '{}' 的待办事项", tag);
                } else {
                    println!("\n=== 标签: '{}' ===", tag);
                    for todo in results {
                        let status = if todo.completed { "✓" } else { "○" };
                        println!("[{}] {} (ID: {})", status, todo.title, todo.id);
                        if !todo.tags.is_empty() {
                            println!("    标签: {}", todo.tags.join(", "));
                        }
                    }
                }
            }
            
            "sort" => {
                let sorted_todos = manager.sort_by_priority();
                println!("\n=== 按优先级排序的待办事项 ===");
                for todo in sorted_todos {
                    let status = if todo.completed { "✓" } else { "○" };
                    let stars = "★".repeat(todo.priority as usize);
                    println!("[{}] {} {} - {} (ID: {})", 
                            status, stars, todo.title, 
                            if todo.completed { "已完成" } else { "进行中" },
                            todo.id);
                }
            }
            
            "stats" => {
                let stats = manager.get_statistics();
                println!("\n=== 统计信息 ===");
                for (key, value) in stats {
                    println!("{}: {}", key, value);
                }
            }
            
            "export" => {
                if parts.len() < 2 {
                    println!("用法: export <文件名.csv>");
                    continue;
                }
                
                match manager.export_csv(parts[1]) {
                    Ok(_) => println!("✅ 已导出到 {}", parts[1]),
                    Err(e) => println!("❌ 导出失败: {}", e),
                }
            }
            
            _ => {
                println!("❌ 未知命令 '{}'，输入 'help' 查看帮助", command);
            }
        }
    }
    
    Ok(())
}
```

### Cargo.toml配置

```toml
[package]
name = "todo-manager"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
csv = "1.1"
```

### 项目特色功能

1. **数据持久化**：自动保存到JSON文件
2. **复合查询**：支持按状态、优先级、标签、关键词筛选
3. **数据导出**：支持CSV格式导出
4. **统计分析**：提供详细的统计信息
5. **灵活更新**：支持部分字段更新
6. **标签系统**：多标签管理
7. **优先级管理**：5级优先级系统

---

## 实战项目2：Web API服务器

### 项目概述

构建一个基于Rust的Web API服务器，模拟一个博客系统的后端API，包含：
- 用户管理
- 文章管理
- 评论系统
- 标签管理
- RESTful API设计
- 数据验证
- 错误处理
- 中间件支持

### 项目结构

```rust
// Cargo.toml
[package]
name = "blog-api-server"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
actix-files = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-async-std-rustls", "sqlite"] }
async-trait = "0.1"
validator = { version = "0.16", features = ["derive"] }
bcrypt = "0.15"
jsonwebtoken = "9"
env_logger = "0.10"
log = "0.4"
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
tempfile = "3.0"
```

```rust
// src/main.rs
use actix_web::{App, HttpServer, web};
use log::info;
use std::env;

mod models;
mod handlers;
mod database;
mod middleware;
mod error;

use database::Database;
use error::AppError;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let port = env::args()
        .find_map(|arg| {
            if arg.starts_with("--port=") {
                arg.split('=').nth(1)?.parse().ok()
            } else {
                None
            }
        })
        .unwrap_or(8080);
    
    info!("启动服务器，端口: {}", port);
    info!("API文档: http://localhost:{}/api/docs", port);
    
    // 初始化数据库
    let database = Database::new("blog.db").await?;
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(database.clone()))
            .wrap(middleware::LoggingMiddleware)
            .wrap(middleware::CorsMiddleware)
            .service(
                web::scope("/api")
                    .service(handlers::auth::register)
                    .service(handlers::auth::login)
                    .service(handlers::users::get_users)
                    .service(handlers::users::get_user)
                    .service(handlers::posts::get_posts)
                    .service(handlers::posts::get_post)
                    .service(handlers::posts::create_post)
                    .service(handlers::posts::update_post)
                    .service(handlers::posts::delete_post)
                    .service(handlers::comments::get_comments)
                    .service(handlers::comments::create_comment)
                    .service(handlers::comments::delete_comment)
                    .service(handlers::tags::get_tags)
                    .service(handlers::tags::create_tag)
                    .service(handlers::stats::get_stats)
            )
            .service(
                actix_files::Files::new("/", "./static/")
                    .index_file("index.html")
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
```

```rust
// src/models.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Regular,
    Admin,
    Moderator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub is_published: bool,
    pub view_count: u64,
    pub like_count: u64,
    pub tags: HashSet<String>,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_approved: bool,
    pub like_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub post_count: u64,
}

// DTOs for API requests/responses
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1))]
    pub content: String,
    pub summary: Option<String>,
    pub tags: Vec<String>,
    pub is_published: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommentRequest {
    #[validate(length(min = 1, max = 1000))]
    pub content: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTagRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    pub description: Option<String>,
}

// Response types
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserSummary,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub role: UserRole,
}

#[derive(Debug, Serialize)]
pub struct PostSummary {
    pub id: Uuid,
    pub title: String,
    pub summary: Option<String>,
    pub author: UserSummary,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub is_published: bool,
    pub view_count: u64,
    pub like_count: u64,
    pub comment_count: u64,
    pub tags: Vec<String>,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct CommentSummary {
    pub id: Uuid,
    pub content: String,
    pub author: UserSummary,
    pub created_at: DateTime<Utc>,
    pub like_count: u64,
    pub replies: Vec<CommentSummary>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: None,
            error: Some(message),
        }
    }
}
```

```rust
// src/database.rs
use sqlx::{Sqlite, Pool, Row};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::time::{Duration, timeout};
use crate::models::*;
use std::collections::HashSet;
use std::time::SystemTime;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),
    #[error("Query execution error: {0}")]
    QueryError(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        
        Self::init_tables(&pool).await?;
        
        Ok(Database { pool })
    }
    
    async fn init_tables(pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // Users table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                display_name TEXT,
                bio TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                is_active BOOLEAN DEFAULT 1,
                role TEXT DEFAULT 'regular'
            )
        "#).execute(pool).await.map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        // Posts table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS posts (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                summary TEXT,
                author_id TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                published_at DATETIME,
                is_published BOOLEAN DEFAULT 0,
                view_count INTEGER DEFAULT 0,
                like_count INTEGER DEFAULT 0,
                slug TEXT UNIQUE NOT NULL,
                FOREIGN KEY (author_id) REFERENCES users (id)
            )
        "#).execute(pool).await.map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        // Post tags junction table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS post_tags (
                post_id TEXT NOT NULL,
                tag_name TEXT NOT NULL,
                PRIMARY KEY (post_id, tag_name),
                FOREIGN KEY (post_id) REFERENCES posts (id)
            )
        "#).execute(pool).await.map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        // Comments table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS comments (
                id TEXT PRIMARY KEY,
                post_id TEXT NOT NULL,
                author_id TEXT NOT NULL,
                parent_id TEXT,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                is_approved BOOLEAN DEFAULT 1,
                like_count INTEGER DEFAULT 0,
                FOREIGN KEY (post_id) REFERENCES posts (id),
                FOREIGN KEY (author_id) REFERENCES users (id),
                FOREIGN KEY (parent_id) REFERENCES comments (id)
            )
        "#).execute(pool).await.map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        // Tags table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                description TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(pool).await.map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    // User operations
    pub async fn create_user(&self, user: &RegisterRequest) -> Result<User, DatabaseError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let password_hash = bcrypt::hash(&user.password, bcrypt::DEFAULT_COST)
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        sqlx::query(r#"
            INSERT INTO users (id, username, email, password_hash, display_name, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&password_hash)
        .bind(&user.display_name)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        self.get_user_by_id(&id).await
    }
    
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, DatabaseError> {
        let row = sqlx::query(r#"
            SELECT * FROM users WHERE username = ? AND is_active = 1
        "#)
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        match row {
            Some(row) => Ok(Some(Self::user_from_row(row))),
            None => Ok(None),
        }
    }
    
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DatabaseError> {
        let row = sqlx::query(r#"
            SELECT * FROM users WHERE email = ? AND is_active = 1
        "#)
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        match row {
            Some(row) => Ok(Some(Self::user_from_row(row))),
            None => Ok(None),
        }
    }
    
    pub async fn get_user_by_id(&self, id: &str) -> Result<User, DatabaseError> {
        let row = sqlx::query(r#"
            SELECT * FROM users WHERE id = ?
        "#)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        Ok(Self::user_from_row(row))
    }
    
    pub async fn verify_password(&self, user: &User, password: &str) -> bool {
        bcrypt::verify(password, &user.password_hash).unwrap_or(false)
    }
    
    fn user_from_row(row: sqlx::sqlite::SqliteRow) -> User {
        User {
            id: Uuid::parse_str(row.get("id")).unwrap(),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            display_name: row.get("display_name"),
            bio: row.get("bio"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
            role: match row.get::<String, _>("role").as_str() {
                "admin" => UserRole::Admin,
                "moderator" => UserRole::Moderator,
                _ => UserRole::Regular,
            },
        }
    }
    
    // Post operations
    pub async fn create_post(&self, post: &CreatePostRequest, author_id: Uuid) -> Result<Post, DatabaseError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let slug = Self::generate_slug(&post.title);
        
        sqlx::query(r#"
            INSERT INTO posts (id, title, content, summary, author_id, created_at, updated_at, published_at, is_published, slug)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&post.title)
        .bind(&post.content)
        .bind(&post.summary)
        .bind(author_id.to_string())
        .bind(now)
        .bind(now)
        .bind(if post.is_published { Some(now) } else { None })
        .bind(post.is_published)
        .bind(&slug)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        // Add tags
        for tag_name in &post.tags {
            sqlx::query(r#"
                INSERT OR IGNORE INTO post_tags (post_id, tag_name) VALUES (?, ?)
            "#)
            .bind(&id)
            .bind(tag_name)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        }
        
        self.get_post_by_id(&id).await
    }
    
    pub async fn get_posts(&self, page: u32, per_page: u32, published_only: bool) -> Result<(Vec<Post>, u64), DatabaseError> {
        let offset = (page - 1) * per_page;
        
        let where_clause = if published_only { "WHERE p.is_published = 1" } else { "" };
        
        // Get total count
        let count_row = sqlx::query(&format!(
            "SELECT COUNT(*) as count FROM posts p {}", where_clause
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let total: u64 = count_row.get("count");
        
        // Get posts
        let rows = sqlx::query(&format!(
            r#"
            SELECT p.*, 
                   GROUP_CONCAT(pt.tag_name) as tags
            FROM posts p
            LEFT JOIN post_tags pt ON p.id = pt.post_id
            {}
            GROUP BY p.id
            ORDER BY p.created_at DESC
            LIMIT ? OFFSET ?
            "#, where_clause
        ))
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        let posts = rows.into_iter().map(Self::post_from_row).collect();
        Ok((posts, total))
    }
    
    pub async fn get_post_by_id(&self, id: &str) -> Result<Post, DatabaseError> {
        let row = sqlx::query(r#"
            SELECT p.*, GROUP_CONCAT(pt.tag_name) as tags
            FROM posts p
            LEFT JOIN post_tags pt ON p.id = pt.post_id
            WHERE p.id = ?
            GROUP BY p.id
        "#)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        Ok(Self::post_from_row(row))
    }
    
    pub async fn update_post(&self, id: &str, update: &UpdatePostRequest) -> Result<Post, DatabaseError> {
        let mut set_clauses = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<sqlx::Sqlite> + Send>> = vec![];
        
        if let Some(ref title) = update.title {
            set_clauses.push("title = ?");
            params.push(Box::new(title.clone()));
        }
        
        if let Some(ref content) = update.content {
            set_clauses.push("content = ?");
            params.push(Box::new(content.clone()));
        }
        
        if let Some(ref summary) = update.summary {
            set_clauses.push("summary = ?");
            params.push(Box::new(summary.clone()));
        }
        
        if let Some(is_published) = update.is_published {
            set_clauses.push("is_published = ?");
            params.push(Box::new(is_published));
            if is_published {
                set_clauses.push("published_at = ?");
                params.push(Box::new(Utc::now()));
            }
        }
        
        if let Some(ref tags) = update.tags {
            // Remove existing tags
            sqlx::query("DELETE FROM post_tags WHERE post_id = ?")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            
            // Add new tags
            for tag_name in tags {
                sqlx::query(r#"
                    INSERT OR IGNORE INTO post_tags (post_id, tag_name) VALUES (?, ?)
                "#)
                .bind(id)
                .bind(tag_name)
                .execute(&self.pool)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            }
        }
        
        if !set_clauses.is_empty() {
            set_clauses.push("updated_at = ?");
            params.push(Box::new(Utc::now()));
            
            let query = format!("UPDATE posts SET {} WHERE id = ?", set_clauses.join(", "));
            let mut sql = sqlx::query(&query);
            
            for param in params {
                sql = sql.bind(param);
            }
            
            sql.bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        }
        
        self.get_post_by_id(id).await
    }
    
    pub async fn delete_post(&self, id: &str) -> Result<(), DatabaseError> {
        // Delete related data first
        sqlx::query("DELETE FROM comments WHERE post_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            
        sqlx::query("DELETE FROM post_tags WHERE post_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        // Delete the post
        sqlx::query("DELETE FROM posts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    fn post_from_row(row: sqlx::sqlite::SqliteRow) -> Post {
        let tags_str: Option<String> = row.get("tags");
        let tags: HashSet<String> = tags_str
            .as_ref()
            .and_then(|s| if s.is_empty() { None } else { Some(s.split(',').map(|s| s.to_string()).collect()) })
            .unwrap_or_default();
        
        Post {
            id: Uuid::parse_str(row.get("id")).unwrap(),
            title: row.get("title"),
            content: row.get("content"),
            summary: row.get("summary"),
            author_id: Uuid::parse_str(row.get("author_id")).unwrap(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            published_at: row.get("published_at"),
            is_published: row.get("is_published"),
            view_count: row.get("view_count"),
            like_count: row.get("like_count"),
            tags,
            slug: row.get("slug"),
        }
    }
    
    fn generate_slug(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .map(|c| match c {
                'a'..='z' | '0'..='9' => c,
                ' ' => '-',
                _ => '-',
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }
}
```

```rust
// src/handlers/mod.rs
pub mod auth;
pub mod users;
pub mod posts;
pub mod comments;
pub mod tags;
pub mod stats;
```

```rust
// src/handlers/auth.rs
use actix_web::{web, HttpResponse, Responder};
use crate::models::*;
use crate::database::Database;
use crate::error::AppError;
use bcrypt::verify;
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Duration, Utc};
use std::env;

pub async fn register(
    db: web::Data<Database>,
    user_data: web::Json<RegisterRequest>,
) -> Result<impl Responder, AppError> {
    let user_data = user_data.into_inner();
    user_data.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    // Check if username exists
    if let Some(_) = db.get_user_by_username(&user_data.username).await? {
        return Ok(HttpResponse::Conflict().json(ApiResponse::error("用户名已存在".to_string())));
    }
    
    // Check if email exists
    if let Some(_) = db.get_user_by_email(&user_data.email).await? {
        return Ok(HttpResponse::Conflict().json(ApiResponse::error("邮箱已存在".to_string())));
    }
    
    // Create user
    let user = db.create_user(&user_data).await?;
    let token = generate_jwt_token(&user)?;
    
    let user_summary = UserSummary {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
    };
    
    let auth_response = AuthResponse {
        token,
        user: user_summary,
        expires_at: Utc::now() + Duration::days(30),
    };
    
    Ok(HttpResponse::Created().json(ApiResponse::success(auth_response)))
}

pub async fn login(
    db: web::Data<Database>,
    login_data: web::Json<LoginRequest>,
) -> Result<impl Responder, AppError> {
    let login_data = login_data.into_inner();
    login_data.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    // Try to find user by username or email
    let user = if login_data.username_or_email.contains('@') {
        db.get_user_by_email(&login_data.username_or_email).await?
    } else {
        db.get_user_by_username(&login_data.username_or_email).await?
    };
    
    let user = match user {
        Some(user) => user,
        None => return Ok(HttpResponse::Unauthorized().json(ApiResponse::error("用户不存在".to_string()))),
    };
    
    if !db.verify_password(&user, &login_data.password) {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::error("密码错误".to_string())));
    }
    
    let token = generate_jwt_token(&user)?;
    
    let user_summary = UserSummary {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
    };
    
    let auth_response = AuthResponse {
        token,
        user: user_summary,
        expires_at: Utc::now() + Duration::days(30),
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(auth_response)))
}

fn generate_jwt_token(user: &crate::models::User) -> Result<String, AppError> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    
    #[derive(Serialize)]
    struct Claims {
        sub: String,
        username: String,
        role: String,
        exp: usize,
    }
    
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: match user.role {
            crate::models::UserRole::Admin => "admin".to_string(),
            crate::models::UserRole::Moderator => "moderator".to_string(),
            crate::models::UserRole::Regular => "regular".to_string(),
        },
        exp: (Utc::now() + Duration::days(30)).timestamp() as usize,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::TokenGenerationError(e.to_string()))
}```rust
// src/handlers/users.rs
use actix_web::{web, HttpResponse, Responder};
use crate::models::*;
use crate::database::Database;
use crate::error::AppError;

pub async fn get_users(
    db: web::Data<Database>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, AppError> {
    let page = query.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
    let per_page = query.get("per_page").and_then(|s| s.parse().ok()).unwrap_or(20);
    
    let (users, total) = db.get_users_paginated(page, per_page).await?;
    
    let user_summaries: Vec<UserSummary> = users.into_iter().map(|user| UserSummary {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
    }).collect();
    
    let response = PaginatedResponse {
        items: user_summaries,
        total,
        page,
        per_page,
        total_pages: ((total as f32) / per_page as f32).ceil() as u32,
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

pub async fn get_user(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let user_id = path.into_inner();
    
    match db.get_user_by_id(&user_id).await {
        Ok(user) => {
            let user_summary = UserSummary {
                id: user.id,
                username: user.username,
                display_name: user.display_name,
                role: user.role,
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(user_summary)))
        }
        Err(_) => Ok(HttpResponse::NotFound().json(ApiResponse::error("用户不存在".to_string()))),
    }
}
```

```rust
// src/handlers/posts.rs
use actix_web::{web, HttpResponse, Responder};
use crate::models::*;
use crate::database::Database;
use crate::error::AppError;
use uuid::Uuid;

pub async fn get_posts(
    db: web::Data<Database>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, AppError> {
    let page = query.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
    let per_page = query.get("per_page").and_then(|s| s.parse().ok()).unwrap_or(10);
    let published_only = query.get("published_only")
        .and_then(|s| s.parse().ok())
        .unwrap_or(true);
    
    let (posts, total) = db.get_posts(page, per_page, published_only).await?;
    
    let post_summaries: Vec<PostSummary> = posts.into_iter().map(|post| {
        // In a real implementation, you'd fetch the author info
        PostSummary {
            id: post.id,
            title: post.title,
            summary: post.summary,
            author: UserSummary {
                id: post.author_id,
                username: "unknown".to_string(),
                display_name: Some("Unknown User".to_string()),
                role: UserRole::Regular,
            },
            created_at: post.created_at,
            published_at: post.published_at,
            is_published: post.is_published,
            view_count: post.view_count,
            like_count: post.like_count,
            comment_count: 0, // Would be fetched in real implementation
            tags: post.tags.into_iter().collect(),
            slug: post.slug,
        }
    }).collect();
    
    let response = PaginatedResponse {
        items: post_summaries,
        total,
        page,
        per_page,
        total_pages: ((total as f32) / per_page as f32).ceil() as u32,
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

pub async fn get_post(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let post_id = path.into_inner();
    
    match db.get_post_by_id(&post_id).await {
        Ok(post) => Ok(HttpResponse::Ok().json(ApiResponse::success(post))),
        Err(_) => Ok(HttpResponse::NotFound().json(ApiResponse::error("文章不存在".to_string()))),
    }
}

pub async fn create_post(
    db: web::Data<Database>,
    post_data: web::Json<CreatePostRequest>,
) -> Result<impl Responder, AppError> {
    let post_data = post_data.into_inner();
    post_data.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    // In a real implementation, you'd get the user ID from JWT token
    let author_id = Uuid::new_v4(); // Placeholder
    
    let post = db.create_post(&post_data, author_id).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(post)))
}

pub async fn update_post(
    db: web::Data<Database>,
    path: web::Path<String>,
    post_data: web::Json<UpdatePostRequest>,
) -> Result<impl Responder, AppError> {
    let post_id = path.into_inner();
    let post_data = post_data.into_inner();
    
    let post = db.update_post(&post_id, &post_data).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(post)))
}

pub async fn delete_post(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let post_id = path.into_inner();
    
    db.delete_post(&post_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
```

```rust
// src/handlers/comments.rs
use actix_web::{web, HttpResponse, Responder};
use crate::models::*;
use crate::database::Database;
use crate::error::AppError;
use uuid::Uuid;

pub async fn get_comments(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let post_id = path.into_inner();
    
    let comments = db.get_comments_by_post_id(&post_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(comments)))
}

pub async fn create_comment(
    db: web::Data<Database>,
    path: web::Path<String>,
    comment_data: web::Json<CreateCommentRequest>,
) -> Result<impl Responder, AppError> {
    let post_id = path.into_inner();
    let comment_data = comment_data.into_inner();
    
    comment_data.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    // In a real implementation, you'd get the user ID from JWT token
    let author_id = Uuid::new_v4(); // Placeholder
    
    let comment = db.create_comment(&post_id, &comment_data, author_id).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(comment)))
}

pub async fn delete_comment(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let comment_id = path.into_inner();
    
    db.delete_comment(&comment_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
```

```rust
// src/handlers/tags.rs
use actix_web::{web, HttpResponse, Responder};
use crate::models::*;
use crate::database::Database;
use crate::error::AppError;
use uuid::Uuid;

pub async fn get_tags(
    db: web::Data<Database>,
) -> Result<impl Responder, AppError> {
    let tags = db.get_all_tags().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(tags)))
}

pub async fn create_tag(
    db: web::Data<Database>,
    tag_data: web::Json<CreateTagRequest>,
) -> Result<impl Responder, AppError> {
    let tag_data = tag_data.into_inner();
    tag_data.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    let tag = db.create_tag(&tag_data).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(tag)))
}
```

```rust
// src/handlers/stats.rs
use actix_web::{web, HttpResponse, Responder};
use crate::models::*;
use crate::database::Database;
use crate::error::AppError;
use std::collections::HashMap;

pub async fn get_stats(
    db: web::Data<Database>,
) -> Result<impl Responder, AppError> {
    let stats = db.get_blog_statistics().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(stats)))
}
```

```rust
// src/middleware.rs
use actix_web::{HttpRequest, HttpResponse, Result};
use actix_web::body::EitherBody;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Future};

pub struct LoggingMiddleware;

impl<S, B> actix_web::dev::Service<HttpRequest, Response = HttpResponse<EitherBody<B>>, Error = actix_web::Error> for LoggingMiddleware
where
    S: actix_web::dev::Service<HttpRequest, Response = HttpResponse<EitherBody<B>>, Error = actix_web::Error>,
{
    type Response = HttpResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: HttpRequest) -> Self::Future {
        let start_time = std::time::Instant::now();
        
        let future = self.service.call(req);
        
        Box::pin(async move {
            let result = future.await?;
            let elapsed = start_time.elapsed();
            
            log::info!(
                "Request processed in {:?}ms with status: {}",
                elapsed.as_millis(),
                result.status()
            );
            
            Ok(result)
        })
    }
}

pub struct CorsMiddleware;

impl<S, B> actix_web::dev::Service<HttpRequest, Response = HttpResponse<EitherBody<B>>, Error = actix_web::Error> for CorsMiddleware
where
    S: actix_web::dev::Service<HttpRequest, Response = HttpResponse<EitherBody<B>>, Error = actix_web::Error>,
{
    type Response = HttpResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, mut req: HttpRequest) -> Self::Future {
        let future = self.service.call(req);
        
        Box::pin(async move {
            let mut response = future.await?;
            
            response.headers_mut().insert(
                actix_web::http::header::AccessControlAllowOrigin::ANY,
                actix_web::http::header::HeaderValue::from_static("*"),
            );
            
            response.headers_mut().insert(
                actix_web::http::header::AccessControlAllowMethods,
                actix_web::http::header::HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
            );
            
            response.headers_mut().insert(
                actix_web::http::header::AccessControlAllowHeaders,
                actix_web::http::header::HeaderValue::from_static("Content-Type, Authorization"),
            );
            
            Ok(response)
        })
    }
}
```

```rust
// src/error.rs
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;
use serde_json::json;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Token generation error: {0}")]
    TokenGenerationError(String),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match self {
            AppError::DatabaseError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::AuthError(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::AuthorizationError(_) => actix_web::http::StatusCode::FORBIDDEN,
            AppError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            AppError::TokenGenerationError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalServiceError(_) => actix_web::http::StatusCode::BAD_GATEWAY,
        };
        
        HttpResponse::build(status_code)
            .json(json!({
                "success": false,
                "error": self.to_string()
            }))
    }
}
```

### API使用示例

```bash
# 启动服务器
cargo run -- --port=8080

# API端点测试

# 1. 用户注册
curl -X POST http://localhost:8080/api/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "email": "john@example.com",
    "password": "password123",
    "display_name": "John Doe"
  }'

# 2. 用户登录
curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{
    "username_or_email": "john_doe",
    "password": "password123"
  }'

# 3. 创建文章
curl -X POST http://localhost:8080/api/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "title": "My First Rust Blog Post",
    "content": "This is a great post about Rust programming...",
    "summary": "A brief summary of my Rust experience",
    "tags": ["rust", "programming", "tutorial"],
    "is_published": true
  }'

# 4. 获取文章列表
curl -X GET "http://localhost:8080/api/posts?page=1&per_page=10&published_only=true"

# 5. 获取文章详情
curl -X GET http://localhost:8080/api/posts/POST_ID

# 6. 创建评论
curl -X POST http://localhost:8080/api/comments/POST_ID \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "content": "Great post! Very informative.",
    "parent_id": null
  }'

# 7. 获取标签
curl -X GET http://localhost:8080/api/tags

# 8. 创建标签
curl -X POST http://localhost:8080/api/tags \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "name": "web-development",
    "description": "Web development related content"
  }'

# 9. 获取统计信息
curl -X GET http://localhost:8080/api/stats
```

### 数据库操作完善

为了完成Database实现，我们还需要添加一些缺失的方法：

```rust
// 在database.rs中添加的方法
impl Database {
    // 补充Database实现
    pub async fn get_users_paginated(&self, page: u32, per_page: u32) -> Result<(Vec<User>, u64), DatabaseError> {
        let offset = (page - 1) * per_page;
        
        // Get total count
        let count_row = sqlx::query("SELECT COUNT(*) as count FROM users WHERE is_active = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let total: u64 = count_row.get("count");
        
        // Get users
        let rows = sqlx::query("SELECT * FROM users WHERE is_active = 1 LIMIT ? OFFSET ?")
            .bind(per_page as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        let users = rows.into_iter().map(Self::user_from_row).collect();
        Ok((users, total))
    }
    
    pub async fn get_comments_by_post_id(&self, post_id: &str) -> Result<Vec<Comment>, DatabaseError> {
        let rows = sqlx::query(r#"
            SELECT c.*, u.username, u.display_name, u.role
            FROM comments c
            JOIN users u ON c.author_id = u.id
            WHERE c.post_id = ? AND c.is_approved = 1
            ORDER BY c.created_at ASC
        "#)
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        Ok(rows.into_iter().map(Self::comment_from_row).collect())
    }
    
    pub async fn create_comment(&self, post_id: &str, comment_data: &CreateCommentRequest, author_id: Uuid) -> Result<Comment, DatabaseError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        sqlx::query(r#"
            INSERT INTO comments (id, post_id, author_id, parent_id, content, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(post_id)
        .bind(author_id.to_string())
        .bind(comment_data.parent_id.as_ref().map(|id| id.to_string()))
        .bind(&comment_data.content)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        self.get_comment_by_id(&id).await
    }
    
    pub async fn get_comment_by_id(&self, id: &str) -> Result<Comment, DatabaseError> {
        let row = sqlx::query(r#"
            SELECT c.*, u.username, u.display_name, u.role
            FROM comments c
            JOIN users u ON c.author_id = u.id
            WHERE c.id = ?
        "#)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        Ok(Self::comment_from_row(row))
    }
    
    pub async fn delete_comment(&self, id: &str) -> Result<(), DatabaseError> {
        // First delete all replies to this comment
        sqlx::query("DELETE FROM comments WHERE parent_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        // Then delete the comment itself
        sqlx::query("DELETE FROM comments WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, DatabaseError> {
        let rows = sqlx::query(r#"
            SELECT t.*, COUNT(pt.post_id) as post_count
            FROM tags t
            LEFT JOIN post_tags pt ON t.name = pt.tag_name
            GROUP BY t.id, t.name
            ORDER BY t.name ASC
        "#)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        Ok(rows.into_iter().map(Self::tag_from_row).collect())
    }
    
    pub async fn create_tag(&self, tag_data: &CreateTagRequest) -> Result<Tag, DatabaseError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        sqlx::query(r#"
            INSERT INTO tags (id, name, description, created_at)
            VALUES (?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&tag_data.name)
        .bind(&tag_data.description)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        self.get_tag_by_id(&id).await
    }
    
    pub async fn get_tag_by_id(&self, id: &str) -> Result<Tag, DatabaseError> {
        let row = sqlx::query(r#"
            SELECT t.*, COUNT(pt.post_id) as post_count
            FROM tags t
            LEFT JOIN post_tags pt ON t.id = pt.tag_name
            WHERE t.id = ?
            GROUP BY t.id
        "#)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        Ok(Self::tag_from_row(row))
    }
    
    pub async fn get_blog_statistics(&self) -> Result<std::collections::HashMap<String, serde_json::Value>, DatabaseError> {
        let mut stats = std::collections::HashMap::new();
        
        // Total users
        let user_count = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_active = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        stats.insert("total_users".to_string(), serde_json::Value::from(user_count::<i64>()));
        
        // Total posts
        let post_count = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        stats.insert("total_posts".to_string(), serde_json::Value::from(post_count::<i64>()));
        
        // Published posts
        let published_count = sqlx::query_scalar("SELECT COUNT(*) FROM posts WHERE is_published = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        stats.insert("published_posts".to_string(), serde_json::Value::from(published_count::<i64>()));
        
        // Total comments
        let comment_count = sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE is_approved = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        stats.insert("total_comments".to_string(), serde_json::Value::from(comment_count::<i64>()));
        
        // Total tags
        let tag_count = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        stats.insert("total_tags".to_string(), serde_json::Value::from(tag_count::<i64>()));
        
        Ok(stats)
    }
    
    fn comment_from_row(row: sqlx::sqlite::SqliteRow) -> Comment {
        Comment {
            id: Uuid::parse_str(row.get("id")).unwrap(),
            post_id: Uuid::parse_str(row.get("post_id")).unwrap(),
            author_id: Uuid::parse_str(row.get("author_id")).unwrap(),
            parent_id: row.get::<Option<String>, _>("parent_id")
                .and_then(|s| Uuid::parse_str(&s).ok()),
            content: row.get("content"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_approved: row.get("is_approved"),
            like_count: row.get("like_count"),
        }
    }
    
    fn tag_from_row(row: sqlx::sqlite::SqliteRow) -> Tag {
        Tag {
            id: Uuid::parse_str(row.get("id")).unwrap(),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            post_count: row.get("post_count"),
        }
    }
}
```

---

## 性能优化与最佳实践

### 1. 集合选择指南

```rust
fn collection_selection_guide() {
    // 何时使用Vec<T>
    // - 需要随机访问 (O(1))
    // - 频繁在末尾添加/删除元素
    // - 需要索引访问
    let mut numbers = Vec::new();
    numbers.push(42); // O(1) 追加
    let first = numbers[0]; // O(1) 访问
    
    // 何时使用HashMap<K, V>
    // - 需要根据键快速查找 (O(1))
    // - 需要根据键插入/删除 (O(1))
    // - 键的唯一性很重要
    let mut user_cache = HashMap::new();
    user_cache.insert("user_id", "user_data"); // O(1) 插入
    
    // 何时使用BTreeMap<K, V>
    // - 需要按键排序遍历
    // - 需要范围查询
    // - 键的大小比较有意义
    let mut sorted_users = BTreeMap::new();
    sorted_users.insert("Bob", "data1");
    sorted_users.insert("Alice", "data2"); // 自动按键排序
    
    // 何时使用HashSet<T>
    // - 需要快速检查成员关系 (O(1))
    // - 需要集合操作 (并集、交集、差集)
    let mut tags = HashSet::new();
    tags.insert("rust");
    tags.insert("programming");
    let has_rust = tags.contains("rust"); // O(1) 检查
    
    // 何时使用VecDeque<T>
    // - 需要高效的在两端添加/删除
    // - 实现队列或双端队列
    let mut queue = VecDeque::new();
    queue.push_back(1); // O(1) 队尾插入
    let front = queue.pop_front(); // O(1) 队头删除
}
```

### 2. 内存优化策略

```rust
fn memory_optimization() {
    // 1. 预分配容量
    let mut large_vec = Vec::with_capacity(10000);
    for i in 0..10000 {
        large_vec.push(i);
    }
    // 避免重复分配内存
    
    // 2. 压缩数据结构
    use std::mem;
    
    #[repr(C)]
    struct CompactUser {
        id: u32,        // 4字节
        age: u8,        // 1字节
        active: bool,   // 1字节
        // 编译器会在这些字段之间添加填充字节
    }
    
    // 3. 使用引用而不是复制
    let data = vec![1, 2, 3, 4, 5];
    let slice = &data[1..4]; // 只借用数据，而不是复制
    println!("Slice: {:?}", slice);
    
    // 4. 避免不必要的分配
    let mut result = String::new();
    for i in 0..1000 {
        // 不推荐：每次都分配
        let temp = format!("Item {}", i);
        result.push_str(&temp);
    }
    
    // 推荐：重用缓冲区
    let mut buffer = String::with_capacity(8000);
    let mut temp = String::new();
    for i in 0..1000 {
        temp.clear();
        temp.push_str("Item ");
        temp.push_str(&i.to_string());
        buffer.push_str(&temp);
    }
    
    // 5. 懒加载
    struct LazyData<T> {
        data: Option<T>,
        init: Box<dyn Fn() -> T>,
    }
    
    impl<T> LazyData<T> {
        fn new<F: Fn() -> T + 'static>(init: F) -> Self {
            Self {
                data: None,
                init: Box::new(init),
            }
        }
        
        fn get(&mut self) -> &T {
            if self.data.is_none() {
                self.data = Some((self.init)());
            }
            self.data.as_ref().unwrap()
        }
    }
}
```

### 3. 性能测试

```rust
fn performance_benchmarking() {
    use std::time::{Duration, Instant};
    
    // 1. Vector vs LinkedList vs ArrayDeque
    let iterations = 100000;
    
    // Vector测试
    let start = Instant::now();
    let mut vec = Vec::new();
    for i in 0..iterations {
        vec.push(i);
    }
    let vec_time = start.elapsed();
    
    // 查找测试
    let start = Instant::now();
    for i in 0..10000 {
        let _ = vec.iter().find(|&&x| x == i);
    }
    let vec_search_time = start.elapsed();
    
    println!("Vector operations:");
    println!("  Push: {:?}", vec_time);
    println!("  Search: {:?}", vec_search_time);
    
    // HashMap vs BTreeMap
    let start = Instant::now();
    let mut hash_map = HashMap::new();
    for i in 0..iterations {
        hash_map.insert(i, format!("value_{}", i));
    }
    let hashmap_build_time = start.elapsed();
    
    let start = Instant::now();
    for i in 0..10000 {
        let _ = hash_map.get(&i);
    }
    let hashmap_search_time = start.elapsed();
    
    println!("HashMap operations:");
    println!("  Build: {:?}", hashmap_build_time);
    println!("  Search: {:?}", hashmap_search_time);
    
    // BTreeMap测试
    let start = Instant::now();
    let mut btree_map = BTreeMap::new();
    for i in 0..iterations {
        btree_map.insert(i, format!("value_{}", i));
    }
    let btree_build_time = start.elapsed();
    
    let start = Instant::now();
    for i in 0..10000 {
        let _ = btree_map.get(&i);
    }
    let btree_search_time = start.elapsed();
    
    println!("BTreeMap operations:");
    println!("  Build: {:?}", btree_build_time);
    println!("  Search: {:?}", btree_search_time);
}
```

### 4. 并发安全集合

```rust
use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use std::time::Duration;

fn concurrent_collections() {
    // 1. 线程安全的HashMap
    let map = Arc::new(RwLock::new(HashMap::new()));
    
    let handles: Vec<_> = (0..10).map(|i| {
        let map_clone = Arc::clone(&map);
        thread::spawn(move || {
            for j in 0..1000 {
                let key = format!("key_{}_{}", i, j);
                let value = format!("value_{}_{}", i, j);
                
                {
                    let mut map = map_clone.write().unwrap();
                    map.insert(key, value);
                }
                
                thread::sleep(Duration::from_micros(1));
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_map = map.read().unwrap();
    println!("Concurrent map size: {}", final_map.len());
    
    // 2. 线程安全的Vec
    let vec = Arc::new(Mutex::new(Vec::new()));
    
    let handles: Vec<_> = (0..5).map(|i| {
        let vec_clone = Arc::clone(&vec);
        thread::spawn(move || {
            for j in 0..100 {
                let mut vec = vec_clone.lock().unwrap();
                vec.push(i * 100 + j);
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_vec = vec.lock().unwrap();
    println!("Concurrent vec length: {}", final_vec.len());
    println!("Final vec sum: {}", final_vec.iter().sum::<i32>());
}
```

---

## 总结

### 本章要点回顾

1. **Vector（Vec<T>）**：
   - 动态数组，支持随机访问
   - O(1) 追加操作，适合频繁添加元素
   - 预分配容量可以提升性能
   - 切片操作提供安全访问

2. **HashMap与HashSet**：
   - O(1) 平均查找性能
   - 适合键值对存储和集合操作
   - 需要实现Hash trait用于自定义类型
   - Entry API提供高效的条件操作

3. **迭代器与闭包**：
   - 惰性计算，避免不必要的计算
   - 链式操作提高代码可读性
   - 高阶函数模式支持
   - 性能优化通过短路操作

4. **其他集合类型**：
   - BTreeMap/BTreeSet：有序集合，适合范围查询
   - VecDeque：双端队列，支持高效两端操作
   - BinaryHeap：优先级队列

5. **实战项目成果**：
   - **Todo管理器**：完整的命令行应用，支持数据持久化和复杂查询
   - **Web API服务器**：生产级博客后端，包含用户管理、文章系统、评论功能

### 学习成果检验

完成本章后，你应该能够：
- [ ] 熟练使用各种Rust集合类型
- [ ] 理解不同集合的性能特征和适用场景
- [ ] 设计高效的数据存储和查询策略
- [ ] 构建基于集合的复杂应用
- [ ] 进行性能优化和内存管理

### 下章预告

第8章将深入学习**模块系统与工程化**，包括：
- Rust模块系统的深入理解
- Crate和Package管理
- 依赖管理最佳实践
- 代码组织结构设计
- 企业级项目架构

### 实践建议

1. **扩展Todo管理器**：
   - 添加日历视图功能
   - 实现同步到云服务
   - 增加团队协作功能

2. **增强Web API**：
   - 添加缓存层
   - 实现全文搜索
   - 添加实时通知功能
   - 集成第三方服务（邮件、短信等）

3. **性能测试**：
   - 使用criterion进行基准测试
   - 分析内存使用情况
   - 进行压力测试

Rust的集合类型为你提供了构建高效、可靠应用程序的强大工具。通过这些基础组件，你可以构建出企业级的复杂系统。在下一章中，我们将学习如何组织这些组件以构建更大的应用程序。