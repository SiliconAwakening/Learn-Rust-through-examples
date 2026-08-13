# 第16章：部署与运维

写完代码只是一半工作，发布与运行是另一半。本章覆盖一个 Rust 服务在生产中的实际生命周期：release 构建、容器镜像、配置、健康检查、优雅关闭、可观测性，以及滚动更新。Rust 静态、单一二进制的产出让这一切异常轻松。

## 学习目标

- 产出优化的 release 二进制，理解 `--release` 做了什么。
- 把应用打包成最小 Docker 镜像。
- 用环境变量与文件做配置，符合十二要素（twelve-factor）部署。
- 实现健康检查与优雅关闭。
- 用日志、指标、链路追踪观测一个运行中的服务。

**实战项目**：部署一个微服务架构系统，覆盖服务拆分、容器编排与 CI/CD。

---

## 16.1 Release 构建

调试构建用于开发；生产跑 `cargo build --release`。release profile 开启优化（`opt-level = 3`）、关闭调试断言，并使用为吞吐调优的系统分配器。对服务，考虑收紧它：

```toml
# Cargo.toml
[profile.release]
lto = "thin"          # 跨 crate 链接时优化
codegen-units = 1     # 更好的优化，更慢的编译
strip = true          # 去掉调试符号，更小的二进制
panic = "abort"       # 更小二进制，无 unwind 开销
```

`panic = "abort"` 是权衡：更小更快的二进制，但没有栈展开——panic 的线程会拖垮整个进程，这对由重启策略监督的服务通常正是你想要的。

---

## 16.2 最小 Docker 镜像

Rust 产出静态（或近静态）链接的二进制，所以运行时镜像可以极小。多阶段构建在完整镜像里编译，再把二进制拷进 `scratch` 或 `debian:bookworm-slim`：

```dockerfile
# 构建阶段
FROM rust:1.78 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/myapp /usr/local/bin/myapp
RUN useradd -r -s /bin/false appuser
USER appuser
EXPOSE 8080
ENTRYPOINT ["myapp"]
```

结果是一个几十兆、内部无工具链的镜像——更小的攻击面、更快的拉取。

---

## 16.3 配置

按 [twelve-factor](https://12factor.net) 方法论，把配置放在环境里。典型做法读环境变量，开发时用本地文件：

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
            port: env::var("PORT").unwrap_or_else(|_| "8080".into()).parse().map_err(|_| "PORT 不是数字")?,
            database_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL missing")?,
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()),
        })
    }
}
```

绝不要把密钥烤进镜像。运行时从密钥管理器或编排平台（Kubernetes secrets、Docker secrets、云密钥管理器）注入。

---

## 16.4 健康检查与就绪

编排器需要知道你的服务是否存活、是否就绪。暴露两个端点：

- **`/health`**（liveness）——“进程还在”。无条件返回 `200`，用于决定是否重启容器。
- **`/ready`**（readiness）——“我能接流量”。仅当数据库已连、预热完成时返回 `200`，用于决定是否路由流量。

```rust
use axum::{routing::get, Router, http::StatusCode};

let app = Router::new()
    .route("/health", get(|| async { StatusCode::OK }))
    .route("/ready", get(|| async { StatusCode::OK }));
```

依赖出问题应让 `/ready` 返回 `503`，而不是崩进程。

---

## 16.5 优雅关闭

部署滚动时，编排器发 `SIGTERM`，等一个短暂宽限期再 `SIGKILL`。你的服务应停止接新连接、完成在途请求、再退出。`axum::serve` 直接支持：

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
    println!("收到关闭信号");
}
```

配合就绪检查，这带来零停机滚动更新：流量在进程退出前排干。

---

## 16.6 可观测性

生产服务需要三个信号：**日志、指标、链路追踪**。

- **日志**——结构化，经 `tracing`。输出 JSON 让日志聚合器索引。
- **指标**——计数器与直方图，经 `prometheus` 或 `metrics` crate，在 `/metrics` 暴露供抓取。
- **追踪**——分布式 span，经 `tracing-opentelemetry`，让你跨服务跟踪一个请求。

```rust
tracing_subscriber::fmt()
    .json()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```

最有用的指标是 RED 三件套：请求的 **R**ate、**E**rrors、**D**uration。按路由跟踪它们，运营需要的大半就齐了。

---

## 16.7 最佳实践

1. **一次构建，处处运行。** 一个从环境读配置的 release 二进制，在 Docker、systemd、Kubernetes 里都不用改。
2. **配置缺失就快速失败。** 启动时若必需变量缺失，带着清晰消息退出——别带病运行。
3. **保持运行时镜像小。** `scratch` 或 slim 基础镜像，无工具链、无源码。
4. **两个健康端点都实现。** liveness 不等于 readiness。
5. **处理 `SIGTERM`。** 优雅关闭是滚动更新安全的保证。
6. **从第一天就可观测。** 后补日志与指标很痛苦。

---

## 16.8 小结

一个 Rust 服务以单个优化过的二进制形式交付，装在小容器里，由环境变量配置，经健康检查监督，在 `SIGTERM` 时排干。release profile 与多阶段 Docker 镜像是机械核心；liveness/readiness 端点、优雅关闭、结构化可观测性是让它能在生产运营的关键。Rust 的产出异常易部署——把这份轻松花在良好的运维卫生上即可。

### 练习

1. 配置 release profile（`lto`、`strip`、`panic = "abort"`），对比二进制大小与启动时间。
2. 写一个多阶段 Dockerfile，构建你的 Axum 服务并以非 root 用户运行。
3. 加 `/health` 与 `/ready` 端点，以及 `SIGTERM` 优雅关闭处理。
4. 用 `tracing_subscriber` 输出 JSON 日志，并通过 `RUST_LOG` 按级别过滤。
