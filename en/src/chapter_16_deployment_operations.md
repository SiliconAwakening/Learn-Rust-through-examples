# Chapter 16: Deployment & Operations

Writing the code is half the job; shipping and running it is the other half. This chapter covers the practical lifecycle of a Rust service in production: release builds, container images, configuration, health checks, graceful shutdown, observability, and zero-downtime updates. Rust's static, single-binary output makes most of this remarkably easy.

## Learning Objectives

- Produce optimized release binaries and understand what `--release` does.
- Package an application as a minimal Docker image.
- Configure with environment variables and files, for twelve-factor deployment.
- Implement health checks and graceful shutdown.
- Observe a running service with logs, metrics, and traces.

---

## 16.1 Release builds

Debug builds are for development; production runs `cargo build --release`. The release profile turns on optimizations (`opt-level = 3`), disables debug assertions, and uses the system allocator tuned for throughput. For a service, consider tightening it:

```toml
# Cargo.toml
[profile.release]
lto = "thin"          # link-time optimization across crates
codegen-units = 1     # better optimization, slower compile
strip = true          # remove debug symbols for a smaller binary
panic = "abort"       # smaller binary, no unwinding overhead
```

`panic = "abort"` is a trade-off: smaller, faster binaries, but no stack unwinding — a panicking thread tears down the whole process, which is often what you want in a service supervised by a restart policy.

---

## 16.2 A minimal Docker image

Rust produces a statically linked (or near-static) binary, so the runtime image can be tiny. A multi-stage build compiles in a full image and copies the binary into `scratch` or `debian:bookworm-slim`:

```dockerfile
# Build stage
FROM rust:1.78 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/myapp /usr/local/bin/myapp
RUN useradd -r -s /bin/false appuser
USER appuser
EXPOSE 8080
ENTRYPOINT ["myapp"]
```

The result is an image of tens of megabytes, not gigabytes, with no toolchain inside — a smaller attack surface and faster pulls.

---

## 16.3 Configuration

Keep configuration in the environment, following the [twelve-factor](https://12factor.net) methodology. A typical setup reads environment variables, with a local file for development:

```rust
use std::env;

struct Config {
    port: u16,
    database_url: String,
    log_level: String,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        Ok(Config {
            port: env::var("PORT").unwrap_or_else(|_| "8080".into()).parse().map_err(|_| "PORT not a number")?,
            database_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL missing")?,
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()),
        })
    }
}
```

Never bake secrets into the image. Inject them at runtime from a secret manager or orchestration platform (Kubernetes secrets, Docker secrets, cloud secret managers).

---

## 16.4 Health checks and readiness

Orchestrators need to know whether your service is alive and ready. Expose two endpoints:

- **`/health`** (liveness) — "the process is up." Returns `200` unconditionally; used to decide whether to restart the container.
- **`/ready`** (readiness) — "I can handle traffic." Returns `200` only when the database is connected and warm-up is complete; used to decide whether to route traffic.

```rust
use axum::{routing::get, Router, http::StatusCode};

let app = Router::new()
    .route("/health", get(|| async { StatusCode::OK }))
    .route("/ready", get(|| async { StatusCode::OK }));
```

A failing dependency should make `/ready` return `503`, not crash the process.

---

## 16.5 Graceful shutdown

When a deployment rolls, the orchestrator sends `SIGTERM` and waits a short grace period before `SIGKILL`. Your service should stop accepting new connections, finish in-flight requests, and exit. `axum::serve` supports this directly:

```rust
use axum::{routing::get, Router};

async fn handler() -> &'static str { "ok" }

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async { signal::ctrl_c().await.expect("install ctrl-c handler"); };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install terminate handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    println!("shutdown signal received");
}
```

Combined with the readiness check, this gives zero-downtime rolling updates: traffic drains before the process exits.

---

## 16.6 Observability

A production service needs three signals: **logs**, **metrics**, and **traces**.

- **Logs** — structured, via `tracing`. Emit JSON so a log aggregator can index it.
- **Metrics** — counters and histograms via `prometheus` or `metrics` crate, exposed at `/metrics` for scraping.
- **Traces** — distributed spans via `tracing-opentelemetry`, so you can follow a request across services.

```rust
tracing_subscriber::fmt()
    .json()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```

The single most useful metric is the RED set: **R**ate, **E**rrors, **D**uration of requests. Track those per route and you have most of what operations needs.

---

## 16.7 Best Practices

1. **Build once, run anywhere.** A release binary that reads its config from the environment works in Docker, systemd, or Kubernetes unchanged.
2. **Fail fast on config.** If a required variable is missing, exit with a clear message at startup — do not limp along.
3. **Keep the runtime image small.** `scratch` or a slim base, no toolchain, no source.
4. **Implement both health endpoints.** Liveness is not readiness.
5. **Handle `SIGTERM`.** Graceful shutdown is what makes rolling updates safe.
6. **Observe from day one.** Retrofitting logs and metrics is painful.

---

## 16.8 Summary

A Rust service ships as a single, optimized binary in a small container, configured by environment variables, supervised via health checks, and drained gracefully on `SIGTERM`. The release profile and a multi-stage Docker image are the mechanical core; liveness/readiness endpoints, graceful shutdown, and structured observability are what make it operable in production. Rust's output is unusually easy to deploy — spend that ease on good operational hygiene.

### Exercises

1. Configure the release profile with `lto`, `strip`, and `panic = "abort"`, and compare binary size and startup time.
2. Write a multi-stage Dockerfile that builds your Axum service and runs it as a non-root user.
3. Add `/health` and `/ready` endpoints and a `SIGTERM` graceful-shutdown handler.
4. Emit JSON logs with `tracing_subscriber` and filter them by level through `RUST_LOG`.
