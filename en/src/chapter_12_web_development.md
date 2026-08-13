# Chapter 12: Web Development

A web service in Rust is typically built on the `tokio` + `axum` stack: Tokio provides the async runtime, and Axum gives routing, extractors, and response handlers with a clean, type-driven API. This chapter builds a small JSON API end-to-end — routing, state, validation, and error responses — so you see how the pieces compose.

## Learning Objectives

- Build an HTTP API with `axum` on top of `tokio`.
- Read request bodies and path/query parameters with extractors.
- Share state across handlers safely.
- Return typed JSON responses and a consistent error format.
- Compose middleware (logging, recovery).

---

## 12.1 A first server

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

A handler is just an `async fn` that returns something implementing `IntoResponse`. `&'static str` becomes a `200 OK` with a text body. Routing maps HTTP methods plus paths to handlers.

---

## 12.2 Path and query parameters

Extractors are Axum's signature feature: the compiler reads a handler's argument types and parses the request for you.

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

The order of extractors matters: `Path` and `Query` are fine anywhere, but the body-consuming extractor (`Json`, `String`) must come **last**.

---

## 12.3 JSON bodies and responses

`Json<T>` both deserializes the request body and serializes the response:

```rust
use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateTodo { title: String }

#[derive(Serialize)]
struct Todo { id: u64, title: String, done: bool }

// Receives {"title":"..."}, returns the created todo as JSON.
async fn create_todo(Json(input): Json<CreateTodo>) -> impl IntoResponse {
    let todo = Todo { id: 1, title: input.title, done: false };
    (axum::http::StatusCode::CREATED, Json(todo))
}
```

If the body fails to parse, Axum returns `400 Bad Request` automatically — you do not write that code.

---

## 12.4 Shared state

Most handlers need a database pool or a cache. Put it in a struct wrapped in `Arc`, pass it to `Router::with_state`, and extract it with `State`:

```rust
use axum::extract::State;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    counter: Arc<std::sync::atomic::AtomicU64>,
}

async fn increment(State(state): State<Arc<AppState>>) -> String {
    let n = state.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    format!("you are visitor {}", n + 1)
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

The state type must be `Clone` (usually via an inner `Arc`), because Axum hands a cheap clone to each request.

---

## 12.5 A consistent error format

Returning `Result` from a handler lets you centralize error handling. Define your error type and an `IntoResponse` impl that maps it to a uniform JSON shape:

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
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found"),
            ApiError::BadRequest(reason) => (StatusCode::BAD_REQUEST, reason.leak()),
        };
        let body = Json(json!({ "error": msg }));
        (status, body).into_response()
    }
}

async fn get_todo(Path(id): Path<u32>) -> Result<String, ApiError> {
    if id == 0 {
        return Err(ApiError::BadRequest("id must be positive".into()));
    }
    if id > 100 {
        return Err(ApiError::NotFound);
    }
    Ok(format!("todo {id}"))
}
```

Now every error response has the same `{"error": "..."}` shape, and handlers stay focused on the happy path.

---

## 12.6 Middleware

Middleware wraps the router to add cross-cutting behavior. `tower_http` ships common layers: logging, CORS, compression, and a catch-all `catch_panic` to turn panics into `500`s.

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

Layers apply in reverse order: the last `.layer` added runs first on the request.

---

## 12.7 Static files and templates

To serve a frontend alongside your API, use `tower_http::services::ServeDir` as a fallback, and render HTML server-side with `askama` (compile-time templates, like Jinja2) or `maud` (HTML as Rust macros). The choice is a matter of taste; both avoid runtime template parsing.

---

## 12.8 Best Practices

1. **Handlers should be thin.** Push logic into the library; the handler parses input, calls a service, and shapes the response.
2. **One error type per API.** Map it with `IntoResponse` so all errors look uniform.
3. **Validate at the boundary.** Reject malformed input before it reaches your domain code — `serde` plus a `validator` crate covers most cases.
4. **Share state through `Arc`, not `static`.** It composes with tests.
5. **Layer observability early.** `TraceLayer` plus `tracing` gives you structured logs you will be grateful for in production.

---

## 12.9 Summary

`axum` turns HTTP into typed Rust: extractors parse the request, `Json` serializes the body, `State` shares resources, and an error type with `IntoResponse` keeps responses uniform. Layered middleware adds logging, CORS, and recovery. The result is a web service that feels like the rest of your statically-checked codebase.

### Exercises

1. Build a `/todos` resource with `GET` (list), `POST` (create), and `GET /:id` (show), backed by an in-memory `Vec` behind a `Mutex`.
2. Add an `ApiError` type and return `404` for unknown ids and `400` for empty titles.
3. Add a `TraceLayer` and a `tracing` subscriber that logs each request with its method, path, and status.
