# 第12章：Web开发

Rust 的 Web 服务通常建立在 `tokio` + `axum` 技术栈之上：Tokio 提供异步运行时，Axum 用清晰、类型驱动的 API 提供路由、提取器与响应处理。本章端到端地构建一个小型 JSON API——路由、状态、校验、错误响应——让你看清各组件如何组合。

## 学习目标

- 用 `axum` 基于 `tokio` 构建 HTTP API。
- 用提取器读取请求体与路径/查询参数。
- 在 handler 之间安全地共享状态。
- 返回类型化的 JSON 响应与一致的错误格式。
- 组合中间件（日志、异常恢复）。

**实战项目**：构建一个博客系统，覆盖多用户、权限管理、内容管理与评论。

---

## 12.1 第一个服务器

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust
use axum::{routing::get, Router};

async fn hello() -> &'static str {
    "hello, world"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

handler 就是一个返回实现了 `IntoResponse` 类型的 `async fn`。`&'static str` 变成 `200 OK` + 文本正文。路由把 HTTP 方法与路径映射到 handler。

---

## 12.2 路径与查询参数

提取器（extractor）是 Axum 的招牌特性：编译器读取 handler 的参数类型，自动为你解析请求。

```rust
use axum::extract::Path;

// /users/42  ->  id = 42
async fn show_user(Path(id): Path<u32>) -> String {
    format!("user {id}")
}
```

```rust
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination { page: Option<u32>, size: Option<u32> }

// /items?page=2&size=10
async fn list_items(Query(p): Query<Pagination>) -> String {
    format!("page {:?}, size {:?}", p.page.unwrap_or(1), p.size.unwrap_or(20))
}
```

提取器顺序有讲究：`Path` 与 `Query` 放哪都行，但消费请求体的提取器（`Json`、`String`）必须放在**最后**。

---

## 12.3 JSON 请求体与响应

`Json<T>` 既能反序列化请求体，又能序列化响应：

```rust
use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateTodo { title: String }

#[derive(Serialize)]
struct Todo { id: u64, title: String, done: bool }

// 接收 {"title":"..."}，返回创建的 todo 为 JSON
async fn create_todo(Json(input): Json<CreateTodo>) -> impl IntoResponse {
    let todo = Todo { id: 1, title: input.title, done: false };
    (axum::http::StatusCode::CREATED, Json(todo))
}
```

请求体解析失败时，Axum 自动返回 `400 Bad Request`——你不必写这段代码。

---

## 12.4 共享状态

多数 handler 需要数据库池或缓存。把状态放进一个包了 `Arc` 的结构体，传给 `Router::with_state`，再用 `State` 提取：

```rust
use axum::extract::State;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    counter: Arc<std::sync::atomic::AtomicU64>,
}

async fn increment(State(state): State<Arc<AppState>>) -> String {
    let n = state.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    format!("你是第 {} 位访客", n + 1)
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
    });
    let app = Router::new()
        .route("/visit", get(increment))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

状态类型必须 `Clone`（通常靠内部 `Arc`），因为 Axum 给每个请求一个廉价副本。

---

## 12.5 统一的错误格式

从 handler 返回 `Result` 可以集中处理错误。定义自己的错误类型与 `IntoResponse` 实现，把它映射为统一的 JSON 形状：

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

enum ApiError {
    NotFound,
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            ApiError::BadRequest(reason) => (StatusCode::BAD_REQUEST, reason),
        };
        let body = Json(json!({ "error": msg }));
        (status, body).into_response()
    }
}

async fn get_todo(Path(id): Path<u32>) -> Result<String, ApiError> {
    if id == 0 {
        return Err(ApiError::BadRequest("id 必须为正".into()));
    }
    if id > 100 {
        return Err(ApiError::NotFound);
    }
    Ok(format!("todo {id}"))
}
```

这样每个错误响应都是 `{"error": "..."}` 形状，handler 专注于正常路径。

---

## 12.6 中间件

中间件包裹路由，加入横切行为。`tower_http` 提供常用 layer：日志、CORS、压缩，以及把 panic 转成 `500` 的兜底：

```toml
[dependencies]
tower-http = { version = "0.5", features = ["trace", "cors"] }
tower = "0.4"
tracing-subscriber = "0.3"
```

```rust
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;

let app = Router::new()
    .route("/", get(hello))
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive());
```

layer 按相反顺序应用：最后加的 `.layer` 在请求时最先执行。

---

## 12.7 静态文件与模板

要在 API 旁边服务前端，用 `tower_http::services::ServeDir` 作 fallback；服务端渲染 HTML 可用 `askama`（编译期模板，类 Jinja2）或 `maud`（把 HTML 写成 Rust 宏）。两者都避免运行时模板解析，选哪个看口味。

---

## 12.8 最佳实践

1. **handler 要薄。** 把逻辑推进库；handler 只解析输入、调服务、塑形响应。
2. **每个 API 一个错误类型。** 用 `IntoResponse` 映射，让错误长相统一。
3. **在边界校验。** 在畸形输入进入领域代码前就拒掉——`serde` 加 `validator` crate 覆盖多数情况。
4. **状态走 `Arc`，别用 `static`。** 这样能与测试组合。
5. **早加可观测性。** `TraceLayer` 加 `tracing` 给你结构化日志，生产时会感激。

---

## 12.9 小结

`axum` 把 HTTP 变成有类型的 Rust：提取器解析请求，`Json` 序列化请求体，`State` 共享资源，错误类型配 `IntoResponse` 让响应统一。中间件层加日志、CORS、异常恢复。结果是一个与你静态检查的代码库其余部分观感一致的 Web 服务。

### 练习

1. 用 `GET`（列表）、`POST`（创建）、`GET /:id`（详情）实现 `/todos` 资源，背后用 `Mutex` 包一个内存 `Vec`。
2. 加一个 `ApiError` 类型：未知 id 返回 `404`，空标题返回 `400`。
3. 加 `TraceLayer` 与 `tracing` 订阅器，记录每个请求的方法、路径、状态。
