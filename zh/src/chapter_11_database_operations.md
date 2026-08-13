# 第11章：数据库操作

大多数应用最终都会把数据存进数据库。Rust 的数据库方案以 **`sqlx`** 为核心——一个异步、编译期校验 SQL 的工具包——配合 `serde` 在行与结构体之间搬运。本章覆盖连接、查询、连接池、事务与迁移，示例使用 SQLite，无需额外服务即可在本机运行。

## 学习目标

- 用 `sqlx` 连接数据库并执行查询。
- 把行映射为结构体，并用编译期 SQL 校验。
- 管理连接池，理解其调优参数。
- 用事务保证多语句更新的原子性。
- 编写并执行数据库迁移。

**实战项目**：构建一个任务管理系统，支持团队协作、任务分配、进度跟踪。

---

## 11.1 准备工作

`sqlx` 支持 PostgreSQL、MySQL/MariaDB、SQLite、MSSQL。本章用 SQLite，让每个示例都能在本机直接跑。

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "macros"] }
serde = { version = "1", features = ["derive"] }
```

SQLite 的连接串就是一个文件路径：

```text
sqlite://todos.db?mode=rwc
```

`mode=rwc` 表示文件不存在则创建。

---

## 11.2 连接与查询

`SqlitePool` 是连接池；查询时会从中取出连接、执行、归还。

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

### 用参数绑定插入

永远把用户提供的值作为参数绑定——绝不要拼进 SQL 字符串，那等于 inviting 注入：

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

## 11.3 把行映射为结构体

用 `query_as` 配合派生 `FromRow` 的结构体：

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

`fetch_all` 把所有行装入 `Vec`；`fetch_one` 返回单行；`fetch` 返回 `Stream`，适合大结果集。

---

## 11.4 编译期 SQL 校验

`sqlx::query!`（与 `query_as!`）宏会在**编译期**对照真实数据库（或保存的 schema）解析并类型检查你的 SQL。拼错列名，构建直接失败：

```rust
async fn titles(pool: &SqlitePool) -> sqlx::Result<Vec<String>> {
    // 构建期对照 todos 表校验
    let rows = sqlx::query!("SELECT title FROM todos WHERE done = 0")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| r.title).collect())
}
```

要用这些宏，要么设 `DATABASE_URL` 让宏在构建时连库，要么运行 `cargo sqlx prepare` 生成 `.sqlx/` 缓存签入版本控制——CI 无库环境必需。

---

## 11.5 连接池

池保持一批热连接，避免每次查询重连。三个调优旋钮：

- `max_connections`——活跃连接上限。
- `min_connections`——保持打开的下限。
- `acquire_timeout`——连接全忙时的等待时长。

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

常见误区是把 `max_connections` 调得很大——数据库自身有上限，超订只会引发争用而非提速。

---

## 11.6 事务

事务让一组语句原子化：要么全部提交，要么全部回滚。这是账户间转账、插入父记录与子记录、更新必须一致的计数器的唯一正确做法。

```rust
async fn transfer(pool: &SqlitePool, from: i64, to: i64, amount: i64) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE accounts SET balance = balance - ? WHERE id = ?")
        .bind(amount).bind(from)
        .execute(&mut *tx).await?;

    sqlx::query("UPDATE accounts SET balance = balance + ? WHERE id = ?")
        .bind(amount).bind(to)
        .execute(&mut *tx).await?;

    tx.commit().await?; // 两步一起生效——否则回滚
    Ok(())
}
```

任一语句失败，`?` 提前返回，`tx` 被 drop 时自动回滚。需要主动中止时可显式 `tx.rollback().await`。

---

## 11.7 迁移

schema 会演进。`sqlx::migrate!` 把迁移文件打包，启动时应用待执行的：

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

`sqlx` 在 `_sqlx_migrations` 表里记录已应用的迁移，所以重复运行二进制只会补上缺失的。

---

## 11.8 最佳实践

1. **绑定，绝不拼接。** 参数既安全又常常更快（驱动可缓存预编译语句）。
2. **池只建一次，处处共享。** 启动时建一个池，把 `Pool` 句柄（内部是 `Arc`）克隆进各 handler。
3. **事务保持短小。** 长事务持锁，拖累吞吐。
4. **部署期迁移，而非每次请求。** 迁移作为启动步骤或独立命令运行。
5. **用 `query_as!` 拿类型安全。** 编译期校验免费消除一大类 bug。

---

## 11.9 小结

`sqlx` 给 Rust 提供异步、类型检查的数据库访问。用参数绑定防注入，用 `FromRow` 映射行，全程序共享一个连接池，多步写操作用事务守护，用迁移演进 schema。结果是与你程序其余部分一样经过静态检查的数据库代码。

### 练习

1. 写一个 CLI，对 SQLite 中的待办事项进行新增、列表、完成操作。
2. 增加 `users` 与 `posts` 两张表，用事务原子地创建用户与其首篇帖子。
3. 把 `query_as` 调用改为 `query_as!` 宏，并用 `cargo sqlx prepare` 为 CI 生成缓存。
