# Chapter 9: Concurrency

Rust's promise of "fearless concurrency" rests on one fact: the same ownership and borrowing rules that prevent memory errors also prevent **data races** at compile time. If two threads share data, the compiler insists that the sharing is safe — either immutable, or guarded by a synchronization primitive. This chapter covers the two mainstream models, **threads with shared state** and **message passing**, and the traits (`Send`, `Sync`) that make them safe.

## Learning Objectives

- Spawn threads and join them.
- Share data safely with `Arc`, `Mutex`, and `RwLock`.
- Communicate between threads with channels (`mpsc`).
- Understand `Send` and `Sync` and why they matter.
- Avoid the common deadlocks and pitfalls of shared-state concurrency.

---

## 9.1 Spawning threads

`std::thread::spawn` starts an OS thread and returns a `JoinHandle`. Call `.join()` to wait for it to finish.

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        for i in 0..5 {
            println!("child says {i}");
        }
    });

    for i in 0..3 {
        println!("main says {i}");
    }

    handle.join().unwrap(); // wait for the child thread
}
```

### Closures and `move`

A spawned thread cannot borrow local variables unless they live long enough — and `main` might return before the thread finishes. The fix is `move`, which transfers ownership of captured variables into the closure:

```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3];

    let handle = thread::spawn(move || {
        // `data` is now owned by this thread.
        println!("got {} items", data.len());
    });

    handle.join().unwrap();
    // println!("{:?}", data); // error: `data` was moved into the thread
}
```

---

## 9.2 `Send` and `Sync`

These two marker traits are the foundation of thread safety. You do not implement them yourself; the compiler derives them automatically when all fields are `Send`/`Sync`.

- **`Send`**: a type `T` is `Send` if it is safe to *move* a `T` to another thread.
- **`Sync`**: a type `T` is `Sync` if it is safe for multiple threads to hold `&T` simultaneously (i.e., `&T` is `Send`).

Most types are both. The exceptions are types with interior mutability without synchronization — `Rc<T>` is the classic example: it is **not** `Send` or `Sync`, because its reference counting is not atomic. Use `Arc<T>` (atomic reference count) instead when sharing across threads.

---

## 9.3 Shared state: `Arc` + `Mutex`

To share *mutable* data between threads, combine:

- **`Arc<T>`** — an atomically reference-counted pointer, so several threads can own the same allocation.
- **`Mutex<T>`** — a lock that guarantees exclusive access to the inner value.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Shared, mutable counter protected by a mutex, shared via Arc.
    let counter = Arc::new(Mutex::new(0));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                let mut num = counter.lock().unwrap();
                *num += 1;
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    println!("final = {}", *counter.lock().unwrap()); // 10
}
```

`.lock().unwrap()` deserves a word: a `Mutex` can be **poisoned** if a thread panics while holding the lock. `.lock()` then returns `Err`. Calling `.unwrap()` propagates the panic, which is usually the right thing — a poisoned lock means the data may be in an inconsistent state.

### `RwLock` — many readers, one writer

When reads vastly outnumber writes, `RwLock` allows multiple simultaneous readers:

```rust
use std::sync::RwLock;

let cache = RwLock::new(0);

// Multiple readers can hold read locks at once.
{
    let r1 = cache.read().unwrap();
    let r2 = cache.read().unwrap();
    println!("reads: {} {}", *r1, *r2);
}

// A writer gets exclusive access.
{
    let mut w = cache.write().unwrap();
    *w += 1;
}
```

---

## 9.4 Message passing: channels

The other concurrency model is to share data by *communicating*, not by sharing memory. Rust's `std::sync::mpsc` (multi-producer, single-consumer) channel does this.

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    let sender = thread::spawn(move || {
        let messages = ["hi", "from", "the", "thread"];
        for m in messages {
            tx.send(m).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    });

    // `rx` is an iterator that yields received values until the sender drops.
    for received in rx {
        println!("got: {received}");
    }

    sender.join().unwrap();
}
```

- `send` returns `Result` because the receiver might have been dropped.
- Multiple producers: clone `tx` with `tx.clone()` and move each into a different thread.

Channels decouple the threads: the sender does not need to know who reads, and the locking is hidden inside the channel implementation.

---

## 9.5 A tiny worker pool

Putting the pieces together — a pool of worker threads consuming jobs from a shared queue:

```rust
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send>>();
    let rx = Arc::new(Mutex::new(rx));

    // Spawn four workers that share the receiver.
    let mut workers = Vec::new();
    for _ in 0..4 {
        let rx = Arc::clone(&rx);
        workers.push(thread::spawn(move || loop {
            let job = {
                let lock = rx.lock().unwrap();
                lock.recv()
            };
            match job {
                Ok(task) => task(),
                Err(_) => break, // all senders dropped — shut down
            }
        }));
    }

    // Send a few jobs.
    for i in 0..8 {
        tx.send(Box::new(move || {
            println!("job {i} on {:?}", thread::current().id());
        }))
        .unwrap();
    }

    drop(tx); // close the channel so workers can exit
    for w in workers {
        w.join().unwrap();
    }
}
```

The `Mutex` around the receiver is necessary because `mpsc::Receiver` is not `Sync` — only one thread may call `recv` at a time.

---

## 9.6 Pitfalls

1. **Deadlock.** Acquiring two locks in different orders in different threads deadlocks. Acquire locks in a consistent global order, or use a single lock that guards both resources.
2. **Holding a lock across `.await` in async code.** A `std::sync::Mutex` guard is not designed to span `.await` points. In async code, use `tokio::sync::Mutex`, or scope the guard so it drops before the `.await`.
3. **Using `Rc` across threads.** The compiler rejects it (`Rc` is not `Send`). Switch to `Arc`.
4. **Forgetting to `join`.** Detached threads may outlive the data they reference — but since Rust forces `move` or `'static` borrows, this is caught at compile time.
5. **Too much locking.** If every operation takes a global mutex, you have serialized the work. Prefer finer-grained locks, sharded data, or channels.

---

## 9.7 Summary

Rust makes data races impossible by construction: shared mutable state requires a `Mutex`, reference counting across threads requires `Arc`, and the `Send`/`Sync` traits are checked at compile time. Use channels when threads communicate; use `Arc<Mutex<T>>` (or `RwLock`) when they share mutable data. The borrow checker turns concurrency bugs that are Heisenbergian in other languages into compile errors here.

### Exercises

1. Spawn ten threads, each incrementing a shared `Arc<Mutex<i32>>` a thousand times, and print the final value.
2. Rewrite the previous exercise using a channel instead of shared state: each thread sends its increment count to a receiver that sums them.
3. Build a pipeline where one thread generates numbers, a second squares them, and a third prints them, connected by two channels.
