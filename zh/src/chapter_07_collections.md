# 第7章：集合类型与数据结构

到目前为止，我们持有的值要么在栈上，要么是固定大小的数组。真实程序需要在运行时动态增长数据——任务队列、记录缓存、唯一 ID 集合。Rust 标准库提供了一组精挑细选的**集合类型**来满足这些需求。本章覆盖日常使用最多的三种——`Vec`、`HashMap`、`HashSet`——以及让它们富于表达力的迭代器机制，并通过一个实战项目把这些组件拼装成可运行的应用。

## 学习目标

- 正确使用 `Vec<T>`，理解容量（capacity）与切片。
- 在 `HashMap` 与 `HashSet` 之间做出选择并高效使用。
- 用迭代器与闭包组合出简洁的数据处理流水线。
- 知道何时该用 `BTreeMap`、`VecDeque` 或 `LinkedList`。
- 避开各类结构的常见性能陷阱。

---

## 7.1 `Vec<T>`：动态数组

`Vec` 把值连续地存放在堆上，内部维护三样东西：指向数据的指针、长度（已存元素数）、容量（已分配内存可容纳的元素数）。尾部追加是均摊 O(1)，按下标访问是 O(1)。

```rust
fn main() {
    // 三种创建方式
    let mut a: Vec<i32> = Vec::new();        // 空
    let b = vec![1, 2, 3];                   // 宏
    let mut c = Vec::with_capacity(100);     // 预分配

    a.push(10);
    a.push(20);
    c.extend([1, 2, 3]);

    // 按下标读取（越界会 panic），或用 get 返回 Option（安全）
    let first = b[0];          // 1
    let maybe = b.get(10);     // None
    println!("{first} {maybe:?}");
}
```

### 容量很关键

`Vec` 容量耗尽时会分配一块更大的内存（通常翻倍）并把元素拷过去。如果你知道最终大小，预先分配可以避免反复重分配：

```rust
// 好：只分配一次
let mut squares: Vec<i32> = Vec::with_capacity(1000);
for i in 0..1000 {
    squares.push(i * i);
}
```

`Vec::with_capacity` 是日常 Rust 里杠杆率最高的优化之一。只要大小已知或可估，就用它。

### 访问、修改与切片

```rust
fn main() {
    let mut v = vec![3, 1, 4, 1, 5, 9, 2, 6];

    v[2] = 10;                       // 修改
    v.push(7);                       // 追加
    let popped = v.pop();            // 弹出末尾
    v.insert(0, 0);                  // 插入（O(n)，需后移）
    let removed = v.remove(0);       // 删除（O(n)）

    let slice: &[i32] = &v[1..=3];   // 切片借用，不拷贝
    v.sort();
    v.dedup();                       // 去除连续重复

    // 查找元素位置
    if let Some(idx) = v.iter().position(|&x| x == 5) {
        println!("found 5 at {idx}");
    }
}
```

> **陷阱**：`insert` 与 `remove` 涉及元素后移，是 O(n)。若你只需要“无序增删”，用 `swap_remove` 更快——它把末尾元素换到目标位置再弹出，O(1)。

---

## 7.2 `HashMap<K, V>`：键值查找

`HashMap` 存储键值对，查找、插入、删除平均 O(1)。键必须实现 `Hash` 与 `Eq`。

```rust
use std::collections::HashMap;

fn main() {
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert("alice".into(), 10);
    scores.insert("bob".into(), 7);

    // entry：仅在键不存在时插入默认值，避免二次查找
    scores.entry("alice".into()).or_insert(50);  // 已存在，不覆盖
    scores.entry("carol".into()).or_insert(3);   // 不存在，插入 3

    if let Some(s) = scores.get("alice") {
        println!("alice: {s}");
    }
}
```

**entry API**（`entry().or_insert()`）是“不存在则插入、否则读取/修改”的地道写法，一次查找搞定。用统计词频来体会：

```rust
use std::collections::HashMap;

fn word_count(text: &str) -> HashMap<&str, u32> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        let c = counts.entry(word).or_insert(0);
        *c += 1;
    }
    counts
}

fn main() {
    let counts = word_count("the quick brown fox the lazy dog the");
    println!("{counts:?}"); // {"the": 3, "quick": 1, ...}
}
```

---

## 7.3 `HashSet<T>`：唯一值集合

`HashSet` 是没有值的 `HashMap`——一个唯一元素的集合，操作同样平均 O(1)。

```rust
use std::collections::HashSet;

fn main() {
    let mut seen: HashSet<&str> = HashSet::new();
    for word in ["a", "b", "a", "c", "b"] {
        // insert 在元素已存在时返回 false
        if !seen.insert(word) {
            println!("重复: {word}");
        }
    }
    println!("唯一: {seen:?}");
}
```

集合支持 `union`、`intersection`、`difference`、`symmetric_difference`，均返回惰性迭代器。

---

## 7.4 迭代器与闭包

集合一旦与**迭代器**结合就变得强大。迭代器是惰性的：按需产出元素，且零成本（编译后等价于手写循环）。

```rust
fn main() {
    let nums = vec![1, 2, 3, 4, 5, 6];

    // 流水线：filter -> map -> collect
    let doubled_evens: Vec<i32> = nums
        .iter()
        .filter(|&&n| n % 2 == 0)   // 闭包：保留偶数
        .map(|&n| n * 2)            // 闭包：每个翻倍
        .collect();                 // 物化为 Vec

    println!("{doubled_evens:?}");  // [4, 8, 12]

    let sum: i32 = nums.iter().sum();
    let max = nums.iter().copied().max();
    println!("sum={sum} max={max:?}");
}
```

### 所有权与借用迭代器

- `.iter()` 产出 `&T`——借用。
- `.iter_mut()` 产出 `&mut T`——可变借用。
- `.into_iter()` 产出 `T`——消费集合。

```rust
let v = vec![1, 2, 3];
let borrowed: Vec<&i32> = v.iter().collect();       // v 仍可用
let owned: Vec<i32> = v.into_iter().collect();      // v 被消费
```

> **陷阱**：`|&&n|` 这种双引用模式来自对 `&Vec<i32>` 迭代（迭代器产出 `&i32`，再模式匹配解一层）。看到它不必慌——是借用叠加。

---

## 7.5 其他集合与如何选择

| 需求 | 选用 | 说明 |
|------|------|------|
| 可增长、有序、可下标 | `Vec<T>` | 默认选择，缓存友好 |
| 快速键查找 | `HashMap<K,V>` | 无序，平均 O(1) |
| 唯一值 | `HashSet<T>` | 支持集合运算 |
| 有序键查找 | `BTreeMap<K,V>` | O(log n)，键有序 |
| 双端队列 | `VecDeque<T>` | 两端都快 |
| 栈（LIFO） | `Vec<T>` | 用 `push` / `pop` |
| 双向链表 | `LinkedList<T>` | 在 Rust 里极少是正确选择 |

```rust
use std::collections::BTreeMap;

fn main() {
    // BTreeMap：键有序，适合需要按顺序遍历的场景
    let mut map = BTreeMap::new();
    map.insert("charlie", 3);
    map.insert("alice", 1);
    map.insert("bob", 2);
    for (k, v) in &map {
        println!("{k}: {v}"); // 按 alice / bob / charlie 顺序输出
    }
}
```

**默认用 `Vec`。** 对中等规模数据，凭借缓存局部性它几乎总是最快的。只有真正需要按键访问时才换 `HashMap`/`BTreeMap`。

---

## 7.6 实战项目：Todo 管理器

把上面的组件拼起来——一个命令行待办管理器，用 `Vec<Todo>` 存储条目、`HashMap` 按标签建立索引、`serde` 持久化到 JSON。

```toml
# Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust
use std::collections::HashMap;
use std::env;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
    tags: Vec<String>,
}

struct TodoStore {
    todos: Vec<Todo>,
    next_id: u32,
    path: String,
}

impl TodoStore {
    fn open(path: &str) -> Self {
        let todos: Vec<Todo> = fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let next_id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        TodoStore { todos, next_id, path: path.into() }
    }

    fn add(&mut self, title: &str, tags: &[String]) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.todos.push(Todo {
            id, title: title.into(), done: false, tags: tags.to_vec(),
        });
        id
    }

    fn complete(&mut self, id: u32) -> bool {
        if let Some(t) = self.todos.iter_mut().find(|t| t.id == id) {
            t.done = true;
            true
        } else {
            false
        }
    }

    fn list(&self) {
        for t in &self.todos {
            let mark = if t.done { "[x]" } else { "[ ]" };
            println!("{} #{} {} {}", mark, t.id, t.title, t.tags.join(","));
        }
    }

    /// 用 HashMap 建立标签 -> 待办条目下标的倒排索引。
    fn tag_index(&self) -> HashMap<&str, Vec<u32>> {
        let mut index: HashMap<&str, Vec<u32>> = HashMap::new();
        for t in &self.todos {
            for tag in &t.tags {
                index.entry(tag).or_default().push(t.id);
            }
        }
        index
    }

    fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.todos)?;
        fs::write(&self.path, json)?;
        Ok(())
    }
}

fn main() {
    let mut store = TodoStore::open("todos.json");
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("add") => {
            let title = args.get(2).cloned().unwrap_or_default();
            let tags: Vec<String> = args[3..].to_vec();
            let id = store.add(&title, &tags);
            println!("added #{id}");
        }
        Some("done") => {
            let id: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            if !store.complete(id) {
                eprintln!("no todo #{id}");
            }
        }
        Some("list") => store.list(),
        Some("tags") => {
            for (tag, ids) in store.tag_index() {
                println!("{tag}: {ids:?}");
            }
        }
        _ => {
            eprintln!("usage: todo [add <title> <tags...> | done <id> | list | tags]");
            return;
        }
    }
    store.save().expect("failed to save");
}
```

用法：

```bash
cargo run -- add "写周报" work urgent
cargo run -- add "买牛奶" life
cargo run -- done 1
cargo run -- list
cargo run -- tags
```

这个项目把本章的组件串了起来：`Vec` 存主数据、`HashMap` 建倒排索引、`serde` 做持久化、迭代器与闭包做查找。它也是后续章节（错误处理、模块化、测试）的现成素材。

---

## 7.7 最佳实践

1. **默认 `Vec`。** 简单、连续、缓存友好；需要按键访问时再换 map。
2. **能预分配就预分配。** `Vec::with_capacity` 是最便宜的优化。
3. **能用迭代器组合子就别写显式循环。** `filter`/`map`/`collect` 更清晰，且零成本。
4. **只迭代一次就别 collect。** 结果只是循环用，就保持惰性。
5. **栈/队列用 `Vec`/`VecDeque`，别用 `LinkedList`。** 链表在 Rust 里既慢又难用。

---

## 7.8 小结

`Vec` 是默认集合——可增长、连续、缓存友好。`HashMap` 与 `HashSet` 提供平均 O(1) 的键查找与去重。迭代器与闭包把这些结构变成富于表达力、零成本的数据流水线。选最简单的、够用的结构，能预分配就预分配，让借用检查器引导你走向正确的访问方式。

### 练习

1. 实现 `dedup_preserve_order<T: Eq + Hash + Clone>(v: &[T]) -> Vec<T>`，用 `HashSet` 记录已见元素以保持顺序去重。
2. 用纯迭代器组合子（无显式循环）实现：输入 `Vec<i32>`，返回其中正数平方之和。
3. 用 entry API 构建一个 `HashMap<String, Vec<String>>`，按首字母分组单词。
