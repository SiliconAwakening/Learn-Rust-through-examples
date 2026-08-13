# Chapter 7: Collections & Data Structures

So far, every value we have owned has lived on the stack or in a fixed-size array. Real programs need to grow data at runtime — queues of jobs, caches of records, sets of unique IDs. Rust's standard library provides a small, well-chosen set of **collections** for these jobs. This chapter covers the three you will use every day — `Vec`, `HashMap`, and `HashSet` — plus the iterator machinery that makes them expressive.

## Learning Objectives

- Use `Vec<T>` correctly, including capacity and slicing.
- Choose between `HashMap` and `HashSet` and use them efficiently.
- Combine collections with iterators and closures for concise data pipelines.
- Recognize when to reach for `BTreeMap`, `VecDeque`, or `LinkedList`.
- Avoid the common performance pitfalls of each structure.

---

## 7.1 `Vec<T>` — the growable array

A `Vec` stores values contiguously on the heap. It tracks three things: a pointer to the data, a length (how many elements exist), and a capacity (how much memory is allocated). Appending is amortized O(1); indexing is O(1).

```rust
fn main() {
    // Three ways to create a Vec.
    let mut a: Vec<i32> = Vec::new();        // empty
    let b = vec![1, 2, 3];                   // from a macro
    let mut c = Vec::with_capacity(100);     // preallocated

    a.push(10);
    a.push(20);
    c.extend([1, 2, 3]);

    // Read by index (panics if out of bounds) or by get (returns Option).
    let first = b[0];            // 1
    let maybe = b.get(10);       // None — safe lookup
    println!("{first} {maybe:?}");

    // Iterate by value (consumes the Vec) or by reference.
    for n in &b {
        println!("{n}");
    }
}
```

### Capacity matters

When a `Vec` runs out of capacity, it allocates a larger buffer (usually double) and copies the elements over. If you know the final size, preallocate — it avoids repeated reallocation:

```rust
// Good: one allocation.
let mut squares: Vec<i32> = Vec::with_capacity(1000);
for i in 0..1000 {
    squares.push(i * i);
}
```

`Vec::with_capacity` is one of the highest-leverage optimizations in everyday Rust. Use it whenever the size is known or can be estimated.

---

## 7.2 `HashMap<K, V>` — keyed lookup

A `HashMap` stores key/value pairs with average O(1) lookup, insertion, and removal. Keys must implement `Hash` and `Eq`.

```rust
use std::collections::HashMap;

fn main() {
    let mut scores: HashMap<String, i32> = HashMap::new();

    scores.insert(String::from("alice"), 10);
    scores.insert(String::from("bob"), 7);

    // `entry` inserts a default only if the key is absent — no double lookup.
    scores.entry(String::from("alice")).or_insert(50);
    scores.entry(String::from("carol")).or_insert(3);

    // Read returns Option<&V>.
    if let Some(score) = scores.get("alice") {
        println!("alice: {score}");
    }

    // Iterate over (&K, &V) pairs.
    for (name, score) in &scores {
        println!("{name}: {score}");
    }
}
```

The **entry API** (`entry().or_insert()`) is the idiomatic way to "insert if missing, otherwise read/modify" in a single pass. For counting words:

```rust
use std::collections::HashMap;

fn word_count(text: &str) -> HashMap<&str, u32> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        let count = counts.entry(word).or_insert(0);
        *count += 1;
    }
    counts
}

fn main() {
    let counts = word_count("the quick brown fox the lazy dog the");
    println!("{counts:?}");
}
```

---

## 7.3 `HashSet<T>` — unique values

A `HashSet` is a `HashMap` without values — a set of unique items, again with average O(1) operations.

```rust
use std::collections::HashSet;

fn main() {
    let mut seen: HashSet<&str> = HashSet::new();
    for word in ["a", "b", "a", "c", "b"] {
        // `insert` returns false if the value was already present.
        if !seen.insert(word) {
            println!("duplicate: {word}");
        }
    }
    println!("unique: {seen:?}");
}
```

Sets support the usual set operations — `union`, `intersection`, `difference`, `symmetric_difference` — all returned as lazy iterators.

---

## 7.4 Iterators and closures

Collections become powerful once you combine them with **iterators**. An iterator is a lazy sequence: it produces values on demand and is zero-cost (it compiles down to the same machine code as a hand-written loop).

```rust
fn main() {
    let nums = vec![1, 2, 3, 4, 5, 6];

    // A pipeline: filter -> map -> collect.
    let doubled_evens: Vec<i32> = nums
        .iter()
        .filter(|&&n| n % 2 == 0)   // closure: keep evens
        .map(|&n| n * 2)            // closure: double each
        .collect();                 // materialize into a Vec

    println!("{doubled_evens:?}");  // [4, 8, 12]

    // Aggregations without explicit loops.
    let sum: i32 = nums.iter().sum();
    let max = nums.iter().copied().max();
    println!("sum={sum} max={max:?}");
}
```

Closures capture their environment by reference (`|n|`, `move |n|`). The `&` patterns like `|&&n|` come from iterating over `&Vec<i32>` (an iterator of `&i32`, then pattern-matched).

### Owning vs borrowing iterators

- `.iter()` yields `&T` — borrow.
- `.iter_mut()` yields `&mut T` — mutable borrow.
- `.into_iter()` yields `T` — consumes the collection.

```rust
let v = vec![1, 2, 3];

let borrowed: Vec<&i32> = v.iter().collect();        // v still usable
let owned: Vec<i32> = v.into_iter().collect();       // v consumed
```

---

## 7.5 Choosing a collection

| Need | Use | Notes |
|------|-----|-------|
| Growable, ordered, indexable | `Vec<T>` | Default choice. Cache-friendly. |
| Fast keyed lookup | `HashMap<K,V>` | No ordering. Average O(1). |
| Unique values | `HashSet<T>` | Set algebra available. |
| Sorted keyed lookup | `BTreeMap<K,V>` | O(log n), keys in order. |
| Double-ended queue | `VecDeque<T>` | Fast push/pop at both ends. |
| Queue, FIFO | `VecDeque<T>` | Prefer over `LinkedList`. |
| Stack (LIFO) | `Vec<T>` | Just use `push` / `pop`. |
| Doubly-linked list | `LinkedList<T>` | Rarely the right choice in Rust. |

**Default to `Vec`.** It is almost always the fastest structure for moderate sizes thanks to cache locality. Reach for a map only when you genuinely need keyed access.

---

## 7.6 Worked example: an in-memory index

Putting the pieces together — a tiny index from a word to the line numbers where it appears, demonstrating `HashMap`, `Vec`, `entry`, and iterators.

```rust
use std::collections::HashMap;

fn build_index(lines: &[&str]) -> HashMap<&str, Vec<usize>> {
    let mut index: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, line) in lines.iter().enumerate() {
        for word in line.split_whitespace() {
            index.entry(word).or_default().push(i);
        }
    }
    index
}

fn main() {
    let text = [
        "the quick brown fox",
        "the lazy dog",
        "quick brown dog",
    ];
    let index = build_index(&text);

    // Print sorted by word using BTreeMap for deterministic order.
    let sorted: std::collections::BTreeMap<_, _> = index.into_iter().collect();
    for (word, lines) in sorted {
        println!("{word:>8}: {lines:?}");
    }
}
```

`or_default()` works because `Vec` implements `Default` (an empty vec), giving us a one-liner that inserts a new list or appends to an existing one.

---

## 7.7 Common pitfalls

1. **Repeated `push` without preallocation.** Use `Vec::with_capacity` when the size is predictable.
2. **`HashMap` with a poor hasher for trusted inputs.** The default hasher (SipHash) is DoS-resistant but slower. For trusted, non-adversarial data, consider `ahash` or `fnv`.
3. **Collecting when you only need to iterate.** If you only loop over the result, skip `.collect()` and stay lazy.
4. **Holding `&mut` to a `Vec` while reading an index.** Borrowing rules prevent this; restructure to compute the index first.
5. **Using `LinkedList` for a queue.** `VecDeque` is almost always faster and friendlier to the borrow checker.

---

## 7.8 Summary

`Vec` is the default collection — growable, contiguous, cache-friendly. `HashMap` and `HashSet` give average O(1) keyed lookup and uniqueness. Iterators and closures turn these structures into expressive, zero-cost data pipelines. Choose the simplest structure that fits, preallocate when you can, and let the borrow checker guide you toward correct access patterns.

### Exercises

1. Implement `dedup_preserve_order<T: Eq + Hash + Clone>(v: &[T]) -> Vec<T>` using a `HashSet` for seen-tracking.
2. Write a function that takes a `Vec<i32>` and returns the sum of squares of its positive values, using only iterator combinators (no explicit loops).
3. Build a `HashMap<String, Vec<String>>` grouping words by their first letter, using the entry API.
