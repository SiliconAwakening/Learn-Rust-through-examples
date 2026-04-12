# 第四章检查报告

## 严重问题

### 1. 第 585 行 - `main` 函数中使用 `?` 操作符
```rust
let sum = process_numbers("10", "32")?;  // ❌ 错误：main 函数返回 ()
```
**问题**：`main` 函数没有返回 `Result`，不能直接使用 `?` 操作符。
**状态**：可能是故意错误，用于教学示范（展示 ? 的错误用法）

**修复方案**：
```rust
fn main() -> Result<(), String> {
    // ...
    let sum = process_numbers("10", "32")?;
    println!("Sum: {}", sum);
    Ok(())
}
```

---

## 🟡 中等问题

### 2. 第 1741 行 - `unwatch` 参数错误
```rust
let _ = watcher.unwatch(&std::path::Path::new("."));
```
**问题**：`Path::new(".")` 返回 `&Path`，不需要再取引用。

### 3. 第 1747 行 - `subscribe()` 方法不存在
```rust
self.change_sender.as_ref().map(|sender| sender.subscribe())
```
**问题**：`crossbeam::channel::Sender` 没有 `subscribe()` 方法。

---

## 🟢 轻微问题

### 4. 第 395 行 - 生命周期示例逻辑
```rust
let data = String::from("Hello World");
let holder = ReferenceHolder::new(&data, data);
```
**说明**：虽然能编译，但逻辑上容易让读者困惑（先借用再 move）。

### 5. 第 1204 行 - `Array` 类型判断
```rust
JsonValue::Array(_) => DataType::Object,
```
**问题**：数组被映射为 `Object`，与类型定义 `DataType::Array(Box<DataType>)` 不符。

### 6. 第 530 行 - `unwrap()` 可能 panic
```rust
let value = result.unwrap(); // 可能 panic!
```
**说明**：可能是故意演示 `unwrap()` 的风险。

---

## 待确认

- 第 742 行 `State` 枚举：使用了 `{:?}` 格式化，需要确认是否有 `#[derive(Debug)]`
- 第 778 行 `old_state.clone()`：需要确认 `State` 是否实现了 `Clone`

---

## 总结

| 优先级 | 数量 | 说明 |
|--------|------|------|
| 🔴 高 | 1 | main 函数 ? 操作符（可能故意） |
| 🟡 中 | 2 | unwatch 参数、subscribe() |
| 🟢 低 | 3 | 生命周期示例、Array类型、unwrap |

大部分代码示例是正确的，适合教学使用。发现的问题中有一些可能是故意设计的错误示范，用于帮助读者理解常见的 Rust 陷阱。
