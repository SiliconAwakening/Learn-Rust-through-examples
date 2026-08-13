# Chapter 11: Database Operations

Most applications eventually store data in a database. Rust's database story centers on **`sqlx`** — an async, compile-time-checked SQL toolkit — and **`serde`** for moving rows to and from structs. This chapter covers connecting, querying, pooling, transactions, and migrations, with SQLite for examples that run on your machine without a server.

## Learning Objectives

- Connect to a database and run queries with `sqlx`.
- Map rows to structs and use compile-time SQL checking.
- Manage a connection pool and understand its tuning knobs.
- Use transactions for atomic, multi-statement updates.
- Run and author database migrations.

---

## 11.1 Setting up

`sqlx` supports PostgreSQL, MySQL/MariaDB, SQLite, and MSSQL. For this chapter we use SQLite so every example runs locally.

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "macros"] }
serde = { version = "1", features = ["derive"] }
```

A connection string for SQLite is just a file path:

```text
sqlite://todos.db?mode=rwc
```

`mode=rwc` creates the file if it does not exist.

---

## 11.2 Connecting and querying

`sqlx::SqlitePool` is a pool of connections; `acquire` or an implicit query will check one out, run, and return it.

```rust
use sqlx::sqlite::SqlitePool;

async fn create_table(pool: &SqlitePool) -> sqlx::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS todos (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            done  INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let pool = SqlitePool::connect("sqlite:todos.db?mode=rwc").await?;
    create_table(&pool).await?;
    Ok(())
}
```

### Insert with bound parameters

Always bind user-supplied values as parameters — never interpolate them into the SQL string, which invites injection:

```rust
async fn add_todo(pool: &SqlitePool, title: &str) -> sqlx::Result<i64> {
    let result = sqlx::query("INSERT INTO todos (title) VALUES (?)")
        .bind(title)
        .execute(pool)
        .await?;
    Ok(result.last_insert_rowid())
}
```

---

## 11.3 Mapping rows to structs

Use `query_as` with a struct that derives `FromRow`:

```rust
use sqlx::FromRow;

#[derive(Debug, FromRow)]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

async fn list_todos(pool: &SqlitePool) -> sqlx::Result<Vec<Todo>> {
    sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos ORDER BY id")
        .fetch_all(pool)
        .await
}
```

`fetch_all` loads every row into a `Vec`; `fetch_one` returns a single row; `fetch` returns a `Stream` for large result sets.

---

## 11.4 Compile-time SQL checking

`sqlx::query!` (and `query_as!`) macros parse and type-check your SQL **at compile time** against a live database (or a saved schema). If you typo a column, the build fails.

```rust
async fn titles(pool: &SqlitePool) -> sqlx::Result<Vec<String>> {
    // Verified at build time against the `todos` table.
    let rows = sqlx::query!("SELECT title FROM todos WHERE done = 0")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| r.title).collect())
}
```

To use the macros you either set `DATABASE_URL` so the macro connects at build time, or run `cargo sqlx prepare` to generate a `.sqlx/` cache checked into version control — essential for CI without a database.

---

## 11.5 Connection pools

A pool keeps a set of connections warm, avoiding the cost of reconnecting per query. Tune three knobs:

- `max_connections` — ceiling on live connections.
- `min_connections` — floor kept open and ready.
- `acquire_timeout` — how long to wait when all are busy.

```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;

let pool = SqlitePoolOptions::new()
    .max_connections(10)
    .min_connections(2)
    .acquire_timeout(Duration::from_secs(5))
    .connect("sqlite:todos.db?mode=rwc")
    .await?;
```

A common mistake is making `max_connections` huge — databases have their own limits, and oversubscribing causes contention rather than speed.

---

## 11.6 Transactions

A transaction makes a group of statements atomic: either all commit, or none do. This is the only correct way to move money between accounts, insert a parent and its children, or update a counter that must stay consistent.

```rust
async fn transfer(pool: &SqlitePool, from: i64, to: i64, amount: i64) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE accounts SET balance = balance - ? WHERE id = ?")
        .bind(amount).bind(from)
        .execute(&mut *tx).await?;

    sqlx::query("UPDATE accounts SET balance = balance + ? WHERE id = ?")
        .bind(amount).bind(to)
        .execute(&mut *tx).await?;

    tx.commit().await?; // apply both — or roll back on error
    Ok(())
}
```

If any statement fails, the `?` returns early and the transaction is automatically rolled back when `tx` is dropped. Explicit `tx.rollback().await` is available when you want to abort on purpose.

---

## 11.7 Migrations

Schema evolves. `sqlx::migrate!` bundles migration files and applies the pending ones at startup.

```
migrations/
├── 20240101000000_init.sql
└── 20240201000000_add_index.sql
```

```sql
-- migrations/20240101000000_init.sql
CREATE TABLE todos (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    done  INTEGER NOT NULL DEFAULT 0
);
```

```rust
async fn main() -> sqlx::Result<()> {
    let pool = SqlitePool::connect("sqlite:todos.db?mode=rwc").await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(())
}
```

`sqlx` records applied migrations in a `_sqlx_migrations` table, so re-running the binary only applies what is missing.

---

## 11.8 Best Practices

1. **Bind, never interpolate.** Parameters are both safer and often faster (the driver can cache the prepared statement).
2. **Pool once, share everywhere.** Create one pool at startup and clone the `Pool` handle (it is just an `Arc` inside) into handlers.
3. **Keep transactions short.** Long-running transactions hold locks and hurt throughput.
4. **Migrate at deploy time, not per request.** Run migrations as a startup step or a separate command.
5. **Use `query_as!` for type safety.** Compile-time checking catches a large class of bugs for free.

---

## 11.9 Summary

`sqlx` gives Rust async, type-checked database access. Bind parameters to prevent injection, map rows with `FromRow`, manage a single connection pool for the whole program, guard multi-step writes with transactions, and evolve the schema with migrations. The result is database code that is as statically checked as the rest of your program.

### Exercises

1. Build a CLI that adds, lists, and completes todos in a SQLite database.
2. Add a `users` and a `posts` table, and write a transaction that creates a user and their first post atomically.
3. Convert the `query_as` calls to `query_as!` macros and set up `cargo sqlx prepare` for CI.
