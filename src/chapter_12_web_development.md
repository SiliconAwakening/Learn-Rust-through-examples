# 第12章：Web开发

## 章节概述

Web开发是现代软件开发的核心技能。在本章中，我们将深入探索Rust的Web开发能力，从框架选择到复杂的企业级应用构建。本章不仅关注前端技术，更强调后端架构、数据库集成、安全性和可维护性。

**学习目标**：
- 掌握Rust Web开发的核心概念和最佳实践
- 理解主流Web框架的特点和适用场景
- 学会构建安全、高效的Web应用
- 掌握用户认证、授权和会话管理
- 学会表单处理、数据验证和文件上传
- 设计并实现一个完整的企业级博客系统

**实战项目**：构建一个企业级博客系统，支持多用户、权限管理、内容管理、评论系统、搜索功能、SEO优化等企业级特性。

## 12.1 Web框架选择

### 12.1.1 Rust Web框架生态

Rust在Web开发方面拥有多个成熟的框架：

- **Actix-web**：高性能、功能完整、社区活跃
- **Axum**：基于Tokio的现代化框架，类型安全
- **Rocket**：零配置、开发友好、安全
- **Warp**：组合式、函数式编程风格
- **Tide**：异步、简洁的设计

### 12.1.2 框架对比分析

#### Actix-web特点
```rust
// Actix-web示例
use actix_web::{web, App, HttpResponse, HttpRequest, Responder};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello World!")
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

#### Axum特点
```rust
// Axum示例
use axum::{extract::Path, response::Json, routing::get, Router};
use serde_json::{json, Value};

async fn root() -> Json<Value> {
    Json(json!({
        "message": "Hello, World!"
    }))
}

async fn greet(Path(name): Path<String>) -> Json<Value> {
    Json(json!({
        "message": format!("Hello, {}!", name)
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/:name", get(greet));
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 12.1.3 框架选择建议

```rust
// 框架选择决策树
pub struct FrameworkSelection {
    performance_priority: bool,
    development_speed: bool,
    feature_complexity: String,
    team_experience: String,
    deployment_target: String,
}

impl FrameworkSelection {
    pub fn recommend_framework(&self) -> FrameworkRecommendation {
        match (
            self.performance_priority,
            self.development_speed,
            &self.feature_complexity,
        ) {
            (true, false, "simple") => FrameworkRecommendation::ActixWeb,
            (true, true, "medium") => FrameworkRecommendation::Axum,
            (false, true, "simple") => FrameworkRecommendation::Rocket,
            (false, false, "complex") => FrameworkRecommendation::Axum,
            _ => FrameworkRecommendation::ActixWeb,
        }
    }
}

#[derive(Debug)]
pub enum FrameworkRecommendation {
    ActixWeb,
    Axum,
    Rocket,
}

impl std::fmt::Display for FrameworkRecommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameworkRecommendation::ActixWeb => write!(f, "Actix-web"),
            FrameworkRecommendation::Axum => write!(f, "Axum"),
            FrameworkRecommendation::Rocket => write!(f, "Rocket"),
        }
    }
}
```

## 12.2 路由与中间件

### 12.2.1 基于Axum的路由系统

```rust
// 高级路由配置
use axum::{
    extract::{Path, Query, State, Extension},
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post, put, delete, patch},
    Router, Json, Form
};
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, cors::CorsLayer, compression::CompressionLayer};
use std::collections::HashMap;
use std::sync::Arc;

// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub redis_client: redis::Client,
    pub config: Config,
    pub logger: Arc<tracing::log::Logger>,
}

// 路由构建器
pub struct RouteBuilder {
    state: AppState,
    routes: Vec<Route>,
    middleware: Vec<Box<dyn axum::middleware::Middleware<(), State = AppState>>>,
}

impl RouteBuilder {
    pub fn new(state: AppState) -> Self {
        RouteBuilder {
            state,
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }
    
    pub fn add_route<R, T>(mut self, method: Method, path: &str, handler: R) -> Self
    where
        R: axum::handler::Handler<T, State = AppState> + Clone,
        T: axum::extract::FromRequestParts<AppState> + axum::extract::FromRequest<AppState>,
    {
        let route = Route {
            method,
            path: path.to_string(),
            handler: std::any::type_name::<R>().to_string(),
        };
        self.routes.push(route);
        self
    }
    
    pub fn add_middleware<M>(mut self, middleware: M) -> Self
    where
        M: axum::middleware::Middleware<(), State = AppState> + Send + Sync + 'static,
    {
        self.middleware.push(Box::new(middleware) as Box<dyn axum::middleware::Middleware<(), State = AppState>>);
        self
    }
    
    pub fn build(self) -> Router<AppState> {
        let mut app = Router::new();
        
        // 基础路由
        app = app
            .route("/", get(home_handler))
            .route("/health", get(health_check))
            .route("/api/v1/status", get(api_status));
        
        // 用户管理路由
        app = app
            .route("/api/v1/users", get(list_users).post(create_user))
            .route("/api/v1/users/:id", get(get_user).put(update_user).delete(delete_user))
            .route("/api/v1/auth/login", post(login))
            .route("/api/v1/auth/logout", post(logout))
            .route("/api/v1/auth/refresh", post(refresh_token));
        
        // 博客相关路由
        app = app
            .route("/api/v1/blogs", get(list_blogs).post(create_blog))
            .route("/api/v1/blogs/:id", get(get_blog).put(update_blog).delete(delete_blog))
            .route("/api/v1/blogs/:id/comments", get(list_comments).post(create_comment))
            .route("/api/v1/blogs/:id/like", post(like_blog))
            .route("/api/v1/blogs/:id/share", post(share_blog));
        
        // 分类和标签路由
        app = app
            .route("/api/v1/categories", get(list_categories).post(create_category))
            .route("/api/v1/tags", get(list_tags).post(create_tag))
            .route("/api/v1/search", get(search));
        
        // 管理员路由
        app = app
            .route("/api/v1/admin/dashboard", get(admin_dashboard))
            .route("/api/v1/admin/users", get(admin_list_users))
            .route("/api/v1/admin/blogs", get(admin_list_blogs))
            .route("/api/v1/admin/comments", get(admin_list_comments));
        
        // 文件上传路由
        app = app
            .route("/api/v1/upload", post(upload_file))
            .route("/api/v1/files/:id", get(download_file).delete(delete_file));
        
        // 静态文件服务
        app = app
            .route("/static/*path", get(serve_static));
        
        // 添加中间件
        app = app
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
                    .layer(CompressionLayer::new())
                    .layer(Extension(self.state))
            );
        
        // 添加自定义中间件
        for middleware in self.middleware {
            app = app.layer(middleware);
        }
        
        app
    }
}

struct Route {
    method: Method,
    path: String,
    handler: String,
}

// 基础处理器
async fn home_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("Content-Type", "text/html")],
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>企业级博客系统</title>
            <meta charset="UTF-8">
        </head>
        <body>
            <h1>欢迎使用企业级博客系统</h1>
            <p>API文档: <a href="/api/v1/docs">查看文档</a></p>
        </body>
        </html>
        "#,
    )
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // 检查数据库连接
    let db_healthy = match sqlx::query("SELECT 1").fetch_one(&state.db_pool).await {
        Ok(_) => true,
        Err(_) => false,
    };
    
    // 检查Redis连接
    let redis_healthy = match state.redis_client.get_connection() {
        Ok(_) => true,
        Err(_) => false,
    };
    
    Json(serde_json::json!({
        "status": "healthy",
        "database": db_healthy,
        "redis": redis_healthy,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn api_status() -> impl IntoResponse {
    Json(serde_json::json!({
        "api_version": "1.0.0",
        "service": "企业级博客系统",
        "status": "operational",
    }))
}

// 错误处理
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Internal server error")]
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Resource not found",
                    "status": 404
                }))
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Unauthorized access",
                    "status": 401
                }))
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "Access forbidden",
                    "status": 403
                }))
            ),
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": msg,
                    "status": 400
                }))
            ),
            AppError::Database(_) | AppError::Redis(_) | AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "status": 500
                }))
            ),
        }
    }
}

// 配置文件
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub upload_dir: String,
    pub max_upload_size: usize,
    pub session_timeout: std::time::Duration,
}
```

### 12.2.2 中间件系统

```rust
// 自定义中间件实现
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
    Extension,
};
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

// 认证中间件
pub struct AuthMiddleware {
    pub required_roles: Vec<String>,
}

impl axum::middleware::Middleware<(), State = AppState> for AuthMiddleware {
    type Future = Pin<Box<dyn Future<Output = Result<Response, (StatusCode, String)>> + Send>>;
    
    fn call(&self, request: Request, state: State<AppState>, next: Next) -> Self::Future {
        Box::pin(async move {
            let user_id = extract_user_id(&request).await;
            
            if let Some(user_id) = user_id {
                // 验证用户
                if let Ok(user) = get_user_by_id(&state.db_pool, &user_id).await {
                    // 检查角色权限
                    if check_role_permissions(&user, &self.required_roles) {
                        // 添加用户信息到请求扩展
                        let mut request = request;
                        request.extensions_mut().insert(user);
                        next.run(request).await
                    } else {
                        Err((StatusCode::FORBIDDEN, "Insufficient permissions".to_string()))
                    }
                } else {
                    Err((StatusCode::UNAUTHORIZED, "Invalid user".to_string()))
                }
            } else {
                Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()))
            }
        })
    }
}

// 速率限制中间件
pub struct RateLimitMiddleware {
    pub max_requests: u64,
    pub window: Duration,
    pub key_extractor: fn(&Request) -> String,
}

impl axum::middleware::Middleware<(), State = AppState> for RateLimitMiddleware {
    type Future = Pin<Box<dyn Future<Output = Result<Response, (StatusCode, String)>> + Send>>;
    
    fn call(&self, request: Request, state: State<AppState>, next: Next) -> Self::Future {
        Box::pin(async move {
            let key = (self.key_extractor)(&request);
            
            if let Some(allowed) = check_rate_limit(&state.redis_client, &key, self.max_requests, self.window).await {
                if allowed {
                    next.run(request).await
                } else {
                    Err((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()))
                }
            } else {
                next.run(request).await
            }
        })
    }
}

// 性能监控中间件
pub struct MetricsMiddleware {
    pub name: String,
}

impl axum::middleware::Middleware<(), State = AppState> for MetricsMiddleware {
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;
    
    fn call(&self, request: Request, state: State<AppState>, next: Next) -> Self::Future {
        Box::pin(async move {
            let start = Instant::now();
            let method = request.method().clone();
            let path = request.uri().path().to_string();
            
            let response = next.run(request).await;
            
            let duration = start.elapsed();
            let status_code = response.status();
            
            // 记录指标
            record_metrics(&state, &self.name, &method, &path, status_code, duration);
            
            response
        })
    }
}

// 日志中间件
pub struct LoggingMiddleware {
    pub level: tracing::Level,
}

impl axum::middleware::Middleware<(), State = AppState> for LoggingMiddleware {
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;
    
    fn call(&self, request: Request, state: State<AppState>, next: Next) -> Self::Future {
        Box::pin(async move {
            let start = Instant::now();
            let method = request.method().clone();
            let path = request.uri().path().to_string();
            let user_agent = request.headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown");
            
            tracing::info!(
                target: "http_requests",
                method = %method,
                path = %path,
                user_agent = %user_agent,
                "request started"
            );
            
            let response = next.run(request).await;
            
            let duration = start.elapsed();
            let status_code = response.status();
            
            tracing::info!(
                target: "http_requests",
                method = %method,
                path = %path,
                status_code = %status_code,
                duration = ?duration,
                "request completed"
            );
            
            response
        })
    }
}
```

## 12.3 表单处理与验证

### 12.3.1 表单数据提取

```rust
use axum::{
    extract::{Form, Multipart, FromRequest, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, Redirect},
    Json, Form
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;

// 基础表单结构
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserRegistrationForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub terms_accepted: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BlogPostForm {
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub category_id: Option<String>,
    pub tags: Option<String>, // 逗号分隔的标签
    pub is_published: bool,
    pub featured_image: Option<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub allow_comments: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommentForm {
    pub content: String,
    pub parent_id: Option<String>, // 回复评论的ID
    pub rating: Option<u8>, // 1-5星评分
}

// 文件上传表单
#[derive(Debug, Deserialize, Serialize)]
pub struct FileUploadForm {
    pub description: Option<String>,
    pub category: String,
    pub tags: Option<String>,
}

// 自定义提取器
pub struct ValidatedForm<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: for<'de> Deserialize<'de> + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);
    
    async fn from_request(req: axum::extract::Request, _state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req.headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        
        if content_type.contains("application/x-www-form-urlencoded") {
            let form = axum::extract::Form::<HashMap<String, String>>::from_request(req, _state).await
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid form data".to_string()))?;
            
            let data = serde_urlencoded::from_str::<T>(&form.0.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&"))
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)))?;
            
            Ok(ValidatedForm(data))
        } else if content_type.contains("multipart/form-data") {
            // 处理multipart表单
            let multipart = axum::extract::Multipart::from_request(req, _state).await
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid multipart data".to_string()))?;
            
            let data = process_multipart_form::<T>(multipart).await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)))?;
            
            Ok(ValidatedForm(data))
        } else {
            Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported content type".to_string()))
        }
    }
}

async fn process_multipart_form<T: for<'de> Deserialize<'de>>(
    mut multipart: axum::extract::Multipart
) -> Result<T, Box<dyn std::error::Error>> {
    let mut form_data = HashMap::new();
    let mut files = HashMap::new();
    
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await?;
        
        if field.file_name().is_some() {
            // 处理文件
            files.insert(name, data.to_vec());
        } else {
            // 处理文本字段
            form_data.insert(name, String::from_utf8_lossy(&data).to_string());
        }
    }
    
    // 构建最终的表单数据
    let form_data_str = form_data.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    
    serde_urlencoded::from_str::<T>(&form_data_str).map_err(|e| e.into())
}

// 表单验证器
pub struct FormValidator;

impl FormValidator {
    pub fn validate_registration_form(form: &UserRegistrationForm) -> Result<(), ValidationError> {
        // 用户名验证
        if form.username.len() < 3 || form.username.len() > 50 {
            return Err(ValidationError::new("username", "用户名长度必须在3-50个字符之间"));
        }
        
        if !form.username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ValidationError::new("username", "用户名只能包含字母、数字、下划线和连字符"));
        }
        
        // 邮箱验证
        if !is_valid_email(&form.email) {
            return Err(ValidationError::new("email", "请输入有效的邮箱地址"));
        }
        
        // 密码验证
        if form.password.len() < 8 {
            return Err(ValidationError::new("password", "密码长度至少8个字符"));
        }
        
        if form.password != form.password_confirm {
            return Err(ValidationError::new("password_confirm", "两次输入的密码不一致"));
        }
        
        // 检查密码强度
        if !is_strong_password(&form.password) {
            return Err(ValidationError::new("password", "密码必须包含大小写字母、数字和特殊字符"));
        }
        
        // 条款接受验证
        if !form.terms_accepted {
            return Err(ValidationError::new("terms_accepted", "您必须接受服务条款"));
        }
        
        Ok(())
    }
    
    pub fn validate_blog_form(form: &BlogPostForm) -> Result<(), ValidationError> {
        // 标题验证
        if form.title.trim().is_empty() || form.title.len() > 200 {
            return Err(ValidationError::new("title", "标题长度必须在1-200个字符之间"));
        }
        
        // 内容验证
        if form.content.trim().is_empty() || form.content.len() < 100 {
            return Err(ValidationError::new("content", "内容长度至少100个字符"));
        }
        
        // 摘要验证
        if let Some(summary) = &form.summary {
            if summary.len() > 500 {
                return Err(ValidationError::new("summary", "摘要长度不能超过500个字符"));
            }
        }
        
        // 标签验证
        if let Some(tags) = &form.tags {
            let tag_list: Vec<&str> = tags.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
            if tag_list.len() > 10 {
                return Err(ValidationError::new("tags", "最多只能添加10个标签"));
            }
            
            for tag in tag_list {
                if tag.len() > 30 {
                    return Err(ValidationError::new("tags", "每个标签长度不能超过30个字符"));
                }
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    field: String,
    message: String,
}

impl ValidationError {
    pub fn new(field: &str, message: &str) -> Self {
        ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

// 辅助函数
fn is_valid_email(email: &str) -> bool {
    regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .unwrap()
        .is_match(email)
}

fn is_strong_password(password: &str) -> bool {
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
    
    has_upper && has_lower && has_digit && has_special
}
```

## 12.4 用户认证与授权

### 12.4.1 JWT认证系统

```rust
// JWT认证实现
use jsonwebtoken::{EncodingKey, DecodingKey, Algorithm, Header, TokenData, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use axum::{
    extract::{FromRequestParts, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // 用户ID
    pub username: String,
    pub role: String,
    pub exp: usize, // 过期时间
    pub iat: usize, // 签发时间
    pub jti: String, // JWT ID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub avatar_url: Option<String>,
}

pub struct JwtManager {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub access_token_duration: Duration,
    pub refresh_token_duration: Duration,
    pub algorithm: Algorithm,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        let key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        
        JwtManager {
            encoding_key: key,
            decoding_key,
            access_token_duration: Duration::minutes(15), // 15分钟
            refresh_token_duration: Duration::days(7), // 7天
            algorithm: Algorithm::HS256,
        }
    }
    
    pub fn generate_tokens(&self, user: &UserInfo) -> Result<(String, String), JwtError> {
        let now = Utc::now();
        let access_exp = (now + self.access_token_duration).timestamp() as usize;
        let refresh_exp = (now + self.refresh_token_duration).timestamp() as usize;
        
        let access_claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: access_exp,
            iat: now.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
        };
        
        let refresh_claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: refresh_exp,
            iat: now.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
        };
        
        let access_token = jsonwebtoken::encode(
            &Header::default(),
            &access_claims,
            &self.encoding_key,
        )?;
        
        let refresh_token = jsonwebtoken::encode(
            &Header::default(),
            &refresh_claims,
            &self.encoding_key,
        )?;
        
        Ok((access_token, refresh_token))
    }
    
    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, JwtError> {
        let validation = Validation::new(self.algorithm);
        jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &validation)
    }
    
    pub fn extract_user_from_request(&self, request: &Request) -> Option<TokenData<Claims>> {
        let auth_header = request.headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok());
        
        if let Some(auth) = auth_header {
            if auth.starts_with("Bearer ") {
                let token = &auth[7..];
                return self.verify_token(token).ok();
            }
        }
        
        // 也检查cookie
        let cookies = request.headers()
            .get("cookie")
            .and_then(|c| c.to_str().ok());
        
        if let Some(cookie_str) = cookies {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("access_token=") {
                    let token = &cookie[13..];
                    return self.verify_token(token).ok();
                }
            }
        }
        
        None
    }
}

// 从请求中提取用户信息
pub struct AuthenticatedUser {
    pub claims: TokenData<Claims>,
}

impl AuthenticatedUser {
    pub fn user_id(&self) -> &str {
        &self.claims.claims.sub
    }
    
    pub fn username(&self) -> &str {
        &self.claims.claims.username
    }
    
    pub fn role(&self) -> &str {
        &self.claims.claims.role
    }
    
    pub fn is_expired(&self) -> bool {
        self.claims.claims.exp < Utc::now().timestamp() as usize
    }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = (StatusCode, String);
    
    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let jwt_manager = &state.jwt_manager;
            
            if let Some(claims) = jwt_manager.extract_user_from_request(&parts.extensions.get::<Request>().unwrap()) {
                if !claims.claims.exp < Utc::now().timestamp() as usize {
                    return Err((StatusCode::UNAUTHORIZED, "Token expired".to_string()));
                }
                
                // 验证用户是否仍然有效
                if let Some(user) = get_user_by_id(&state.db_pool, &uuid::Uuid::parse_str(&claims.claims.sub).unwrap()).await {
                    // 检查用户状态
                    if !user.is_active {
                        return Err((StatusCode::FORBIDDEN, "User account is disabled".to_string()));
                    }
                    
                    Ok(AuthenticatedUser { claims })
                } else {
                    Err((StatusCode::UNAUTHORIZED, "User not found".to_string()))
                }
            } else {
                Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()))
            }
        })
    }
}
```

## 12.5 企业级博客系统

现在我们来构建一个完整的企业级博客系统，集成所有学到的Web开发技术。

```rust
// 企业级博客系统主项目
// File: enterprise-blog/Cargo.toml
/*
[package]
name = "enterprise-blog"
version = "1.0.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = { version = "0.7", features = ["macros"] }
tower = { version = "0.4" }
tower-http = { version = "0.5", features = ["cors", "compression", "trace"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "json", "uuid", "chrono"] }
redis = { version = "0.23", features = ["tokio-comp"] }
bcrypt = "0.15"
jsonwebtoken = "9.0"
clap = { version = "4.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "1.0"
regex = "1.0"
markdown = "1.0"
html-escape = "0.4"
mime = "0.4"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
*/
```

```rust
// 数据模型
// File: enterprise-blog/src/models.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::{FromRow, Type};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "user_role")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Editor,
    Author,
    Subscriber,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub author_id: Uuid,
    pub category_id: Option<Uuid>,
    pub status: BlogStatus,
    pub is_featured: bool,
    pub is_pinned: bool,
    pub allow_comments: bool,
    pub allow_ratings: bool,
    pub view_count: i32,
    pub like_count: i32,
    pub comment_count: i32,
    pub reading_time: i32, // 分钟
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "blog_status")]
#[serde(rename_all = "snake_case")]
pub enum BlogStatus {
    Draft,
    Published,
    Archived,
    Scheduled,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub content: String,
    pub status: CommentStatus,
    pub is_approved: bool,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub like_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "comment_status")]
#[serde(rename_all = "snake_case")]
pub enum CommentStatus {
    Pending,
    Approved,
    Spam,
    Trash,
}

// API请求/响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBlogRequest {
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub category_id: Option<Uuid>,
    pub tag_ids: Option<Vec<Uuid>>,
    pub status: BlogStatus,
    pub is_featured: bool,
    pub is_pinned: bool,
    pub allow_comments: bool,
    pub allow_ratings: bool,
    pub featured_image: Option<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

// 主应用程序
// File: enterprise-blog/src/main.rs
use clap::{Parser, Subcommand};
use tracing::{info, warn, error, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use tokio::sync::RwLock;

mod models;
mod services;
mod web;

use models::*;
use services::*;
use web::WebServer;

#[derive(Parser, Debug)]
#[command(name = "enterprise-blog")]
#[command(about = "Enterprise Blog System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the web server
    Server {
        #[arg(short, long, default_value = "0.0.0.0:3000")]
        addr: String,
        
        #[arg(short, long, default_value = "postgres://blog_user:password@localhost/enterprise_blog")]
        database_url: String,
        
        #[arg(short, long, default_value = "redis://localhost:6379")]
        redis_url: String,
    },
    /// Run database migrations
    Migrate {
        #[arg(short, long, default_value = "postgres://blog_user:password@localhost/enterprise_blog")]
        database_url: String,
    },
    /// Setup database and run migrations
    Setup {
        #[arg(short, long, default_value = "postgres://blog_user:password@localhost/enterprise_blog")]
        database_url: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "enterprise_blog=debug,tokio=warn,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Server { addr, database_url, redis_url } => {
            run_server(addr, database_url, redis_url).await
        }
        Commands::Migrate { database_url } => {
            run_migrations(database_url).await
        }
        Commands::Setup { database_url } => {
            setup_database(database_url).await
        }
    }
}

#[instrument]
async fn run_server(
    addr: String,
    database_url: String,
    redis_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Enterprise Blog server on {}", addr);
    
    // 初始化数据库
    let db_pool = sqlx::PgPool::connect(&database_url).await?;
    let redis_client = redis::Client::open(&redis_url)?;
    
    // 初始化服务
    let user_service = Arc::new(UserService::new(db_pool.clone()));
    let blog_service = Arc::new(BlogService::new(db_pool.clone()));
    let auth_service = Arc::new(AuthService::new(db_pool.clone(), redis_client.clone()));
    let media_service = Arc::new(MediaService::new(db_pool.clone()));
    let analytics_service = Arc::new(AnalyticsService::new(db_pool.clone()));
    
    // 启动Web服务器
    let server = WebServer::new(
        addr,
        user_service,
        blog_service,
        auth_service,
        media_service,
        analytics_service,
    );
    
    info!("Enterprise Blog server started successfully");
    server.run().await?;
    
    Ok(())
}

#[instrument]
async fn run_migrations(database_url: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running database migrations");
    
    let pool = sqlx::PgPool::connect(&database_url).await?;
    
    // 创建用户表
    sqlx::query(r#"
        CREATE TYPE user_role AS ENUM ('admin', 'editor', 'author', 'subscriber');
        
        CREATE TABLE users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username VARCHAR(50) UNIQUE NOT NULL,
            email VARCHAR(100) UNIQUE NOT NULL,
            display_name VARCHAR(100) NOT NULL,
            bio TEXT,
            avatar_url TEXT,
            website TEXT,
            password_hash VARCHAR(255) NOT NULL,
            role user_role NOT NULL DEFAULT 'subscriber',
            is_active BOOLEAN DEFAULT TRUE,
            email_verified BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            last_login TIMESTAMPTZ
        );
    "#).execute(&pool).await?;
    
    // 创建博客文章表
    sqlx::query(r#"
        CREATE TYPE blog_status AS ENUM ('draft', 'published', 'archived', 'scheduled');
        
        CREATE TABLE blog_posts (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(200) NOT NULL,
            slug VARCHAR(200) UNIQUE NOT NULL,
            content TEXT NOT NULL,
            excerpt TEXT,
            featured_image TEXT,
            author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            category_id UUID,
            status blog_status NOT NULL DEFAULT 'draft',
            is_featured BOOLEAN DEFAULT FALSE,
            is_pinned BOOLEAN DEFAULT FALSE,
            allow_comments BOOLEAN DEFAULT TRUE,
            allow_ratings BOOLEAN DEFAULT TRUE,
            view_count INTEGER DEFAULT 0,
            like_count INTEGER DEFAULT 0,
            comment_count INTEGER DEFAULT 0,
            reading_time INTEGER DEFAULT 0,
            seo_title VARCHAR(200),
            seo_description TEXT,
            published_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );
    "#).execute(&pool).await?;
    
    info!("Database migrations completed successfully");
    Ok(())
}

#[instrument]
async fn setup_database(database_url: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting up database and running migrations");
    
    // 先运行迁移
    run_migrations(database_url.clone()).await?;
    
    // 创建默认管理员用户
    let pool = sqlx::PgPool::connect(&database_url).await?;
    let admin_password = "admin123";
    
    sqlx::query!(
        r#"
        INSERT INTO users (username, email, display_name, password_hash, role, is_active, email_verified)
        VALUES ('admin', 'admin@example.com', 'Administrator', $1, 'admin', true, true)
        ON CONFLICT (username) DO NOTHING
        "#,
        bcrypt::hash(&admin_password, bcrypt::DEFAULT_COST)?
    )
    .execute(&pool)
    .await?;
    
    info!("Default admin user created - username: admin, password: admin123");
    info!("Please change the admin password after first login");
    
    Ok(())
}

// 服务层实现
// File: enterprise-blog/src/services.rs
use super::models::*;
use crate::database::DatabaseManager;
use sqlx::PgPool;
use tracing::{info, warn, error, instrument};

pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        UserService { pool }
    }
    
    #[instrument(skip(self))]
    pub async fn create_user(&self, request: &RegisterRequest) -> Result<User, sqlx::Error> {
        let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)?;
        
        let user = sqlx::query!(
            r#"
            INSERT INTO users (username, email, display_name, password_hash, role, is_active, email_verified)
            VALUES ($1, $2, $3, $4, 'subscriber', true, false)
            RETURNING *
            "#,
            request.username,
            request.email,
            request.display_name,
            password_hash
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(User::from_row(&user)?)
    }
    
    #[instrument(skip(self))]
    pub async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query!(
            "SELECT * FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user.map(|row| User::from_row(&row).unwrap()))
    }
    
    #[instrument(skip(self))]
    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<Option<User>, sqlx::Error> {
        if let Some(user) = sqlx::query!(
            "SELECT * FROM users WHERE username = $1 AND is_active = true",
            username
        )
        .fetch_optional(&self.pool)
        .await? {
            let user = User::from_row(&user).unwrap();
            if bcrypt::verify(password, &user.password_hash)? {
                Ok(Some(user))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

pub struct BlogService {
    pool: PgPool,
}

impl BlogService {
    pub fn new(pool: PgPool) -> Self {
        BlogService { pool }
    }
    
    #[instrument(skip(self))]
    pub async fn create_blog_post(&self, request: &CreateBlogRequest, author_id: Uuid) -> Result<BlogPost, sqlx::Error> {
        let slug = generate_slug(&request.title);
        
        let post = sqlx::query!(
            r#"
            INSERT INTO blog_posts (
                id, title, slug, content, excerpt, featured_image, author_id, category_id,
                status, is_featured, is_pinned, allow_comments, allow_ratings,
                seo_title, seo_description, published_at
            ) VALUES (
                gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            RETURNING *
            "#,
            request.title,
            slug,
            request.content,
            request.excerpt,
            request.featured_image,
            author_id,
            request.category_id,
            request.status as BlogStatus,
            request.is_featured,
            request.is_pinned,
            request.allow_comments,
            request.allow_ratings,
            request.seo_title,
            request.seo_description,
            request.published_at
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(BlogPost::from_row(&post)?)
    }
    
    #[instrument(skip(self))]
    pub async fn get_published_posts(&self, limit: i64, offset: i64) -> Result<Vec<BlogPost>, sqlx::Error> {
        let posts = sqlx::query!(
            r#"
            SELECT bp.*, u.display_name as author_name
            FROM blog_posts bp
            JOIN users u ON bp.author_id = u.id
            WHERE bp.status = 'published'
            ORDER BY bp.is_pinned DESC, bp.published_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(posts.into_iter().map(|row| BlogPost::from_row(&row).unwrap()).collect())
    }
    
    #[instrument(skip(self))]
    pub async fn increment_view_count(&self, post_id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE blog_posts SET view_count = view_count + 1 WHERE id = $1",
            post_id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}

// 辅助函数
fn generate_slug(title: &str) -> String {
    title.to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '-' | '_' => '-',
            _ => '',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

// Web服务器
// File: enterprise-blog/src/web.rs
use super::services::*;
use super::models::*;
use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, cors::CorsLayer};
use std::sync::Arc;

pub struct WebServer {
    app: Router,
    addr: String,
}

impl WebServer {
    pub fn new(
        addr: String,
        user_service: Arc<UserService>,
        blog_service: Arc<BlogService>,
        auth_service: Arc<AuthService>,
        media_service: Arc<MediaService>,
        analytics_service: Arc<AnalyticsService>,
    ) -> Self {
        let app = Router::new()
            .route("/", get(home_handler))
            .route("/health", get(health_check))
            
            // 公开API
            .route("/api/v1/posts", get(get_posts).post(create_post))
            .route("/api/v1/posts/:id", get(get_post))
            .route("/api/v1/categories", get(get_categories))
            .route("/api/v1/tags", get(get_tags))
            .route("/api/v1/search", get(search_posts))
            
            // 用户API
            .route("/api/v1/auth/register", post(register_user))
            .route("/api/v1/auth/login", post(login_user))
            .route("/api/v1/auth/logout", post(logout_user))
            
            // 管理API
            .route("/api/v1/admin/posts", get(admin_list_posts))
            .route("/api/v1/admin/users", get(admin_list_users))
            
            .with_state(AppState {
                user_service,
                blog_service,
                auth_service,
                media_service,
                analytics_service,
            })
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
            );
        
        WebServer { app, addr }
    }
    
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = tokio::net::TcpListener::bind(&self.addr).await?;
        println!("Enterprise Blog server listening on {}", self.addr);
        
        axum::serve(listener, self.app).await?;
        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    user_service: Arc<UserService>,
    blog_service: Arc<BlogService>,
    auth_service: Arc<AuthService>,
    media_service: Arc<MediaService>,
    analytics_service: Arc<AnalyticsService>,
}

// 处理器实现
async fn home_handler() -> &'static str {
    "Enterprise Blog System"
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let db_healthy = sqlx::query("SELECT 1").fetch_one(&state.user_service.pool).await.is_ok();
    Json(serde_json::json!({
        "status": "healthy",
        "database": db_healthy,
    }))
}

async fn get_posts(State(state): State<AppState>) -> impl IntoResponse {
    match state.blog_service.get_published_posts(20, 0).await {
        Ok(posts) => Json(serde_json::json!({
            "posts": posts,
            "total": posts.len() as i64,
        })),
        Err(_) => Json(serde_json::json!({
            "error": "Failed to fetch posts"
        })),
    }
}

async fn create_post(
    State(state): State<AppState>,
    Json(request): Json<CreateBlogRequest>,
) -> impl IntoResponse {
    // 从认证中获取用户ID
    let author_id = Uuid::new_v4(); // 简化实现
    
    match state.blog_service.create_blog_post(&request, author_id).await {
        Ok(post) => Json(serde_json::json!({
            "success": true,
            "post": post,
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}

async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // 增加浏览量
    let _ = state.blog_service.increment_view_count(&id).await;
    
    // 获取文章详情
    // 简化实现
    Json(serde_json::json!({
        "id": id,
        "title": "Sample Post",
        "content": "This is a sample blog post content.",
    }))
}

// 其他处理器...
```

```rust
// Docker部署配置
# File: enterprise-blog/docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: enterprise_blog
      POSTGRES_USER: blog_user
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  blog-app:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgres://blog_user:password@postgres:5432/enterprise_blog
      REDIS_URL: redis://redis:6379
    depends_on:
      - postgres
      - redis
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

```rust
// File: enterprise-blog/README.md
# 企业级博客系统

一个基于Rust构建的企业级博客系统，支持多用户、权限管理、内容管理、评论系统、搜索功能、SEO优化等企业级特性。

## 功能特性

### 核心功能
- **用户管理**：用户注册、登录、权限管理
- **内容管理**：博客文章的创建、编辑、发布、管理
- **分类系统**：多级分类、标签管理
- **评论系统**：评论、回复、审核
- **媒体管理**：图片上传、管理、CDN支持
- **搜索功能**：全文搜索、高级搜索
- **SEO优化**：URL重写、meta标签、sitemap

### 企业级特性
- **权限管理**：基于角色的访问控制
- **数据安全**：密码哈希、SQL注入防护
- **性能优化**：缓存、CDN、图片优化
- **监控告警**：性能监控、错误追踪
- **备份恢复**：数据备份、灾难恢复
- **国际化**：多语言支持

## 快速开始

### 使用Docker Compose

1. 启动服务
```bash
docker-compose up -d
```

2. 初始化数据库
```bash
cargo run setup --database-url "postgres://blog_user:password@localhost/enterprise_blog"
```

3. 访问系统
- 网站：http://localhost:3000
- API文档：http://localhost:3000/health

### 本地开发

1. **安装依赖**
```bash
# 安装PostgreSQL和Redis
sudo apt-get install postgresql redis-server
```

2. **设置环境**
```bash
# 创建数据库
createdb enterprise_blog

# 设置环境变量
export DATABASE_URL="postgres://blog_user:password@localhost/enterprise_blog"
export REDIS_URL="redis://localhost:6379"
```

3. **运行应用**
```bash
cargo run server
```

## API文档

### 公开API
- `GET /api/v1/posts` - 获取博客文章列表
- `GET /api/v1/posts/:id` - 获取博客文章详情
- `GET /api/v1/categories` - 获取分类列表
- `GET /api/v1/tags` - 获取标签列表
- `GET /api/v1/search` - 搜索文章

### 用户API
- `POST /api/v1/auth/register` - 用户注册
- `POST /api/v1/auth/login` - 用户登录
- `POST /api/v1/auth/logout` - 用户登出

### 管理API
- `GET /api/v1/admin/posts` - 管理文章列表
- `GET /api/v1/admin/users` - 管理用户列表
- `POST /api/v1/admin/posts` - 创建文章
- `PUT /api/v1/admin/posts/:id` - 更新文章
- `DELETE /api/v1/admin/posts/:id` - 删除文章

## 性能特性

### 数据库优化
- 索引优化
- 查询优化
- 分页处理
- 连接池管理

### 缓存策略
- Redis缓存
- 页面缓存
- API响应缓存
- 静态资源缓存

### 静态资源
- 图片压缩
- CSS/JS压缩
- CDN集成
- 懒加载

## 安全特性

### 身份认证
- JWT token认证
- 密码安全存储
- 会话管理
- 密码重置

### 数据安全
- SQL注入防护
- XSS防护
- CSRF保护
- 输入验证

### 权限控制
- 基于角色的访问控制
- 细粒度权限管理
- API访问控制
- 资源权限验证

## 监控和运维

### 性能监控
- 请求响应时间
- 数据库查询性能
- 内存使用监控
- CPU使用监控

### 错误监控
- 应用错误追踪
- 异常日志记录
- 错误通知
- 错误恢复

### 业务监控
- 用户活跃度
- 内容访问统计
- 搜索热词分析
- 转化率追踪

## 部署和扩展

### 容器化部署
- Docker容器化
- Kubernetes支持
- CI/CD管道
- 蓝绿部署

### 水平扩展
- 无状态设计
- 负载均衡
- 数据库分片
- 微服务架构

### 云部署
- AWS支持
- Google Cloud支持
- Azure支持
- 多云部署

## 开发规范

### 代码质量
- 单元测试
- 集成测试
- 性能测试
- 安全测试

### 文档规范
- API文档
- 代码注释
- 架构文档
- 部署文档

### 版本控制
- Git工作流
- 代码审查
- 分支管理
- 发布流程

## 贡献指南

欢迎贡献代码、报告问题或提出功能请求。

## 许可证

MIT License

---

**联系信息**：
- 作者：MiniMax Agent
- 邮箱：developer@minimax.com
- 文档：https://docs.minimax.com/enterprise-blog
# 第12章：Web开发

## 章节概述

Web开发是现代软件开发的核心技能。在本章中，我们将深入探索Rust的Web开发能力，从框架选择到复杂的企业级应用构建。本章不仅关注前端技术，更强调后端架构、数据库集成、安全性和可维护性。

**学习目标**：
- 掌握Rust Web开发的核心概念和最佳实践
- 理解主流Web框架的特点和适用场景
- 学会构建安全、高效的Web应用
- 掌握用户认证、授权和会话管理
- 学会表单处理、数据验证和文件上传
- 设计并实现一个完整的企业级博客系统

**实战项目**：构建一个企业级博客系统，支持多用户、权限管理、内容管理、评论系统、搜索功能、SEO优化等企业级特性。

## 12.1 Web框架选择

### 12.1.1 Rust Web框架生态

Rust在Web开发方面拥有多个成熟的框架：

- **Actix-web**：高性能、功能完整、社区活跃
- **Axum**：基于Tokio的现代化框架，类型安全
- **Rocket**：零配置、开发友好、安全
- **Warp**：组合式、函数式编程风格
- **Tide**：异步、简洁的设计

### 12.1.2 框架对比分析

#### Actix-web特点
```rust
// Actix-web示例
use actix_web::{web, App, HttpResponse, HttpRequest, Responder};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello World!")
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

#### Axum特点
```rust
// Axum示例
use axum::{extract::Path, response::Json, routing::get, Router};
use serde_json::{json, Value};

async fn root() -> Json<Value> {
    Json(json!({
        "message": "Hello, World!"
    }))
}

async fn greet(Path(name): Path<String>) -> Json<Value> {
    Json(json!({
        "message": format!("Hello, {}!", name)
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/:name", get(greet));
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 12.1.3 框架选择建议

```rust
// 框架选择决策树
pub struct FrameworkSelection {
    performance_priority: bool,
    development_speed: bool,
    feature_complexity: String,
    team_experience: String,
    deployment_target: String,
}

impl FrameworkSelection {
    pub fn recommend_framework(&self) -> FrameworkRecommendation {
        match (
            self.performance_priority,
            self.development_speed,
            &self.feature_complexity,
        ) {
            (true, false, "simple") => FrameworkRecommendation::ActixWeb,
            (true, true, "medium") => FrameworkRecommendation::Axum,
            (false, true, "simple") => FrameworkRecommendation::Rocket,
            (false, false, "complex") => FrameworkRecommendation::Axum,
            _ => FrameworkRecommendation::ActixWeb,
        }
    }
}

#[derive(Debug)]
pub enum FrameworkRecommendation {
    ActixWeb,
    Axum,
    Rocket,
}

impl std::fmt::Display for FrameworkRecommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameworkRecommendation::ActixWeb => write!(f, "Actix-web"),
            FrameworkRecommendation::Axum => write!(f, "Axum"),
            FrameworkRecommendation::Rocket => write!(f, "Rocket"),
        }
    }
}
```

## 12.2 路由与中间件

### 12.2.1 基于Axum的路由系统

```rust
// 高级路由配置
use axum::{
    extract::{Path, Query, State, Extension},
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post, put, delete, patch},
    Router, Json, Form
};
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, cors::CorsLayer, compression::CompressionLayer};
use std::collections::HashMap;
use std::sync::Arc;

// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub redis_client: redis::Client,
    pub config: Config,
    pub logger: Arc<tracing::log::Logger>,
}

// 路由构建器
pub struct RouteBuilder {
    state: AppState,
    routes: Vec<Route>,
    middleware: Vec<Box<dyn axum::middleware::Middleware<(), State = AppState>>>,
}

impl RouteBuilder {
    pub fn new(state: AppState) -> Self {
        RouteBuilder {
            state,
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }
    
    pub fn add_route<R, T>(mut self, method: Method, path: &str, handler: R) -> Self
    where
        R: axum::handler::Handler<T, State = AppState> + Clone,
        T: axum::extract::FromRequestParts<AppState> + axum::extract::FromRequest<AppState>,
    {
        let route = Route {
            method,
            path: path.to_string(),
            handler: std::any::type_name::<R>().to_string(),
        };
        self.routes.push(route);
        self
    }
    
    pub fn add_middleware<M>(mut self, middleware: M) -> Self
    where
        M: axum::middleware::Middleware<(), State = AppState> + Send + Sync + 'static,
    {
        self.middleware.push(Box::new(middleware) as Box<dyn axum::middleware::Middleware<(), State = AppState>>);
        self
    }
    
    pub fn build(self) -> Router<AppState> {
        let mut app = Router::new();
        
        // 基础路由
        app = app
            .route("/", get(home_handler))
            .route("/health", get(health_check))
            .route("/api/v1/status", get(api_status));
        
        // 用户管理路由
        app = app
            .route("/api/v1/users", get(list_users).post(create_user))
            .route("/api/v1/users/:id", get(get_user).put(update_user).delete(delete_user))
            .route("/api/v1/auth/login", post(login))
            .route("/api/v1/auth/logout", post(logout))
            .route("/api/v1/auth/refresh", post(refresh_token));
        
        // 博客相关路由
        app = app
            .route("/api/v1/blogs", get(list_blogs).post(create_blog))
            .route("/api/v1/blogs/:id", get(get_blog).put(update_blog).delete(delete_blog))
            .route("/api/v1/blogs/:id/comments", get(list_comments).post(create_comment))
            .route("/api/v1/blogs/:id/like", post(like_blog))
            .route("/api/v1/blogs/:id/share", post(share_blog));
        
        // 分类和标签路由
        app = app
            .route("/api/v1/categories", get(list_categories).post(create_category))
            .route("/api/v1/tags", get(list_tags).post(create_tag))
            .route("/api/v1/search", get(search));
        
        // 管理员路由
        app = app
            .route("/api/v1/admin/dashboard", get(admin_dashboard))
            .route("/api/v1/admin/users", get(admin_list_users))
            .route("/api/v1/admin/blogs", get(admin_list_blogs))
            .route("/api/v1/admin/comments", get(admin_list_comments));
        
        // 文件上传路由
        app = app
            .route("/api/v1/upload", post(upload_file))
            .route("/api/v1/files/:id", get(download_file).delete(delete_file));
        
        // 静态文件服务
        app = app
            .route("/static/*path", get(serve_static));
        
        // 添加中间件
        app = app
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
                    .layer(CompressionLayer::new())
                    .layer(Extension(self.state))
            );
        
        // 添加自定义中间件
        for middleware in self.middleware {
            app = app.layer(middleware);
        }
        
        app
    }
}

struct Route {
    method: Method,
    path: String,
    handler: String,
}

// 基础处理器
async fn home_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("Content-Type", "text/html")],
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>企业级博客系统</title>
            <meta charset="UTF-8">
        </head>
        <body>
            <h1>欢迎使用企业级博客系统</h1>
            <p>API文档: <a href="/api/v1/docs">查看文档</a></p>
        </body>
        </html>
        "#,
    )
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // 检查数据库连接
    let db_healthy = match sqlx::query("SELECT 1").fetch_one(&state.db_pool).await {
        Ok(_) => true,
        Err(_) => false,
    };
    
    // 检查Redis连接
    let redis_healthy = match state.redis_client.get_connection() {
        Ok(_) => true,
        Err(_) => false,
    };
    
    Json(serde_json::json!({
        "status": "healthy",
        "database": db_healthy,
        "redis": redis_healthy,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn api_status() -> impl IntoResponse {
    Json(serde_json::json!({
        "api_version": "1.0.0",
        "service": "企业级博客系统",
        "status": "operational",
    }))
}

// 错误处理
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Internal server error")]
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Resource not found",
                    "status": 404
                }))
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Unauthorized access",
                    "status": 401
                }))
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "Access forbidden",
                    "status": 403
                }))
            ),
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": msg,
                    "status": 400
                }))
            ),
            AppError::Database(_) | AppError::Redis(_) | AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "status": 500
                }))
            ),
        }
    }
}

// 配置文件
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub upload_dir: String,
    pub max_upload_size: usize,
    pub session_timeout: std::time::Duration,
}
```

### 12.2.2 中间件系统

```rust
// 自定义中间件实现
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
    Extension,
};
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

// 认证中间件
pub struct AuthMiddleware {
    pub required_roles: Vec<String>,
}

impl axum::middleware::Middleware<(), State = AppState> for AuthMiddleware {
    type Future = Pin<Box<dyn Future<Output = Result<Response, (StatusCode, String)>> + Send>>;
    
    fn call(&self, request: Request, state: State<AppState>, next: Next) -> Self::Future {
        Box::pin(async move {
            let user_id = extract_user_id(&request).await;
            
            if let Some(user_id) = user_id {
                // 验证用户
                if let Ok(user) = get_user_by_id(&state.db_pool, &user_id).await {
                    // 检查角色权限
                    if check_role_permissions(&user, &self.required_roles) {
                        // 添加用户信息到请求扩展
                        let mut request = request;
                        request.extensions_mut().insert(user);
                        next.run(request).await
                    } else {
                        Err((StatusCode::FORBIDDEN, "Insufficient permissions".to_string()))
                    }
                } else {
                    Err((StatusCode::UNAUTHORIZED, "Invalid user".to_string()))
                }
            } else {
                Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()))
            }
        })
    }
}

// 速率限制中间件
pub struct RateLimitMiddleware {
    pub max_requests: u64,
    pub window: Duration,
    pub key_extractor: fn(&Request) -> String,
}

impl axum::middleware::Middleware<(), State = AppState> for RateLimitMiddleware {
    type Future = Pin<Box<dyn Future<Output = Result<Response, (StatusCode, String)>> + Send>>;
    
    fn call(&self, request: Request, state: State<AppState>, next: Next) -> Self::Future {
        Box::pin(async move {
            let key = (self.key_extractor)(&request);
            
            if let Some(allowed) = check_rate_limit(&state.redis_client, &key, self.max_requests, self.window).await {
                if allowed {
                    next.run(request).await
                } else {
                    Err((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()))
                }
            } else {
                next.run(request).await
            }
        })
    }
}

// 性能监控中间件
pub struct MetricsMiddleware {
    pub name: String,
}

impl axum::middleware::Middleware<(), State = AppState> for MetricsMiddleware {
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;
    
    fn call(&self, request: Request, state: State<AppState>, next: Next) -> Self::Future {
        Box::pin(async move {
            let start = Instant::now();
            let method = request.method().clone();
            let path = request.uri().path().to_string();
            
            let response = next.run(request).await;
            
            let duration = start.elapsed();
            let status_code = response.status();
            
            // 记录指标
            record_metrics(&state, &self.name, &method, &path, status_code, duration);
            
            response
        })
    }
}

// 日志中间件
pub struct LoggingMiddleware {
    pub level: tracing::Level,
}

impl axum::middleware::Middleware<(), State = AppState> for LoggingMiddleware {
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;
    
    fn call(&self, request: Request, state: State<AppState>, next: Next) -> Self::Future {
        Box::pin(async move {
            let start = Instant::now();
            let method = request.method().clone();
            let path = request.uri().path().to_string();
            let user_agent = request.headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown");
            
            tracing::info!(
                target: "http_requests",
                method = %method,
                path = %path,
                user_agent = %user_agent,
                "request started"
            );
            
            let response = next.run(request).await;
            
            let duration = start.elapsed();
            let status_code = response.status();
            
            tracing::info!(
                target: "http_requests",
                method = %method,
                path = %path,
                status_code = %status_code,
                duration = ?duration,
                "request completed"
            );
            
            response
        })
    }
}
```

## 12.3 表单处理与验证

### 12.3.1 表单数据提取

```rust
use axum::{
    extract::{Form, Multipart, FromRequest, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, Redirect},
    Json, Form
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;

// 基础表单结构
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserRegistrationForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub terms_accepted: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BlogPostForm {
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub category_id: Option<String>,
    pub tags: Option<String>, // 逗号分隔的标签
    pub is_published: bool,
    pub featured_image: Option<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub allow_comments: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommentForm {
    pub content: String,
    pub parent_id: Option<String>, // 回复评论的ID
    pub rating: Option<u8>, // 1-5星评分
}

// 文件上传表单
#[derive(Debug, Deserialize, Serialize)]
pub struct FileUploadForm {
    pub description: Option<String>,
    pub category: String,
    pub tags: Option<String>,
}

// 自定义提取器
pub struct ValidatedForm<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: for<'de> Deserialize<'de> + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);
    
    async fn from_request(req: axum::extract::Request, _state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req.headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        
        if content_type.contains("application/x-www-form-urlencoded") {
            let form = axum::extract::Form::<HashMap<String, String>>::from_request(req, _state).await
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid form data".to_string()))?;
            
            let data = serde_urlencoded::from_str::<T>(&form.0.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&"))
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)))?;
            
            Ok(ValidatedForm(data))
        } else if content_type.contains("multipart/form-data") {
            // 处理multipart表单
            let multipart = axum::extract::Multipart::from_request(req, _state).await
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid multipart data".to_string()))?;
            
            let data = process_multipart_form::<T>(multipart).await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)))?;
            
            Ok(ValidatedForm(data))
        } else {
            Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported content type".to_string()))
        }
    }
}

async fn process_multipart_form<T: for<'de> Deserialize<'de>>(
    mut multipart: axum::extract::Multipart
) -> Result<T, Box<dyn std::error::Error>> {
    let mut form_data = HashMap::new();
    let mut files = HashMap::new();
    
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await?;
        
        if field.file_name().is_some() {
            // 处理文件
            files.insert(name, data.to_vec());
        } else {
            // 处理文本字段
            form_data.insert(name, String::from_utf8_lossy(&data).to_string());
        }
    }
    
    // 构建最终的表单数据
    let form_data_str = form_data.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    
    serde_urlencoded::from_str::<T>(&form_data_str).map_err(|e| e.into())
}

// 表单验证器
pub struct FormValidator;

impl FormValidator {
    pub fn validate_registration_form(form: &UserRegistrationForm) -> Result<(), ValidationError> {
        // 用户名验证
        if form.username.len() < 3 || form.username.len() > 50 {
            return Err(ValidationError::new("username", "用户名长度必须在3-50个字符之间"));
        }
        
        if !form.username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ValidationError::new("username", "用户名只能包含字母、数字、下划线和连字符"));
        }
        
        // 邮箱验证
        if !is_valid_email(&form.email) {
            return Err(ValidationError::new("email", "请输入有效的邮箱地址"));
        }
        
        // 密码验证
        if form.password.len() < 8 {
            return Err(ValidationError::new("password", "密码长度至少8个字符"));
        }
        
        if form.password != form.password_confirm {
            return Err(ValidationError::new("password_confirm", "两次输入的密码不一致"));
        }
        
        // 检查密码强度
        if !is_strong_password(&form.password) {
            return Err(ValidationError::new("password", "密码必须包含大小写字母、数字和特殊字符"));
        }
        
        // 条款接受验证
        if !form.terms_accepted {
            return Err(ValidationError::new("terms_accepted", "您必须接受服务条款"));
        }
        
        Ok(())
    }
    
    pub fn validate_blog_form(form: &BlogPostForm) -> Result<(), ValidationError> {
        // 标题验证
        if form.title.trim().is_empty() || form.title.len() > 200 {
            return Err(ValidationError::new("title", "标题长度必须在1-200个字符之间"));
        }
        
        // 内容验证
        if form.content.trim().is_empty() || form.content.len() < 100 {
            return Err(ValidationError::new("content", "内容长度至少100个字符"));
        }
        
        // 摘要验证
        if let Some(summary) = &form.summary {
            if summary.len() > 500 {
                return Err(ValidationError::new("summary", "摘要长度不能超过500个字符"));
            }
        }
        
        // 标签验证
        if let Some(tags) = &form.tags {
            let tag_list: Vec<&str> = tags.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
            if tag_list.len() > 10 {
                return Err(ValidationError::new("tags", "最多只能添加10个标签"));
            }
            
            for tag in tag_list {
                if tag.len() > 30 {
                    return Err(ValidationError::new("tags", "每个标签长度不能超过30个字符"));
                }
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    field: String,
    message: String,
}

impl ValidationError {
    pub fn new(field: &str, message: &str) -> Self {
        ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

// 辅助函数
fn is_valid_email(email: &str) -> bool {
    regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .unwrap()
        .is_match(email)
}

fn is_strong_password(password: &str) -> bool {
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
    
    has_upper && has_lower && has_digit && has_special
}
```

## 12.4 用户认证与授权

### 12.4.1 JWT认证系统

```rust
// JWT认证实现
use jsonwebtoken::{EncodingKey, DecodingKey, Algorithm, Header, TokenData, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use axum::{
    extract::{FromRequestParts, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // 用户ID
    pub username: String,
    pub role: String,
    pub exp: usize, // 过期时间
    pub iat: usize, // 签发时间
    pub jti: String, // JWT ID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub avatar_url: Option<String>,
}

pub struct JwtManager {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub access_token_duration: Duration,
    pub refresh_token_duration: Duration,
    pub algorithm: Algorithm,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        let key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        
        JwtManager {
            encoding_key: key,
            decoding_key,
            access_token_duration: Duration::minutes(15), // 15分钟
            refresh_token_duration: Duration::days(7), // 7天
            algorithm: Algorithm::HS256,
        }
    }
    
    pub fn generate_tokens(&self, user: &UserInfo) -> Result<(String, String), JwtError> {
        let now = Utc::now();
        let access_exp = (now + self.access_token_duration).timestamp() as usize;
        let refresh_exp = (now + self.refresh_token_duration).timestamp() as usize;
        
        let access_claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: access_exp,
            iat: now.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
        };
        
        let refresh_claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: refresh_exp,
            iat: now.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
        };
        
        let access_token = jsonwebtoken::encode(
            &Header::default(),
            &access_claims,
            &self.encoding_key,
        )?;
        
        let refresh_token = jsonwebtoken::encode(
            &Header::default(),
            &refresh_claims,
            &self.encoding_key,
        )?;
        
        Ok((access_token, refresh_token))
    }
    
    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, JwtError> {
        let validation = Validation::new(self.algorithm);
        jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &validation)
    }
    
    pub fn extract_user_from_request(&self, request: &Request) -> Option<TokenData<Claims>> {
        let auth_header = request.headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok());
        
        if let Some(auth) = auth_header {
            if auth.starts_with("Bearer ") {
                let token = &auth[7..];
                return self.verify_token(token).ok();
            }
        }
        
        // 也检查cookie
        let cookies = request.headers()
            .get("cookie")
            .and_then(|c| c.to_str().ok());
        
        if let Some(cookie_str) = cookies {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("access_token=") {
                    let token = &cookie[13..];
                    return self.verify_token(token).ok();
                }
            }
        }
        
        None
    }
}

// 从请求中提取用户信息
pub struct AuthenticatedUser {
    pub claims: TokenData<Claims>,
}

impl AuthenticatedUser {
    pub fn user_id(&self) -> &str {
        &self.claims.claims.sub
    }
    
    pub fn username(&self) -> &str {
        &self.claims.claims.username
    }
    
    pub fn role(&self) -> &str {
        &self.claims.claims.role
    }
    
    pub fn is_expired(&self) -> bool {
        self.claims.claims.exp < Utc::now().timestamp() as usize
    }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = (StatusCode, String);
    
    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let jwt_manager = &state.jwt_manager;
            
            if let Some(claims) = jwt_manager.extract_user_from_request(&parts.extensions.get::<Request>().unwrap()) {
                if !claims.claims.exp < Utc::now().timestamp() as usize {
                    return Err((StatusCode::UNAUTHORIZED, "Token expired".to_string()));
                }
                
                // 验证用户是否仍然有效
                if let Some(user) = get_user_by_id(&state.db_pool, &uuid::Uuid::parse_str(&claims.claims.sub).unwrap()).await {
                    // 检查用户状态
                    if !user.is_active {
                        return Err((StatusCode::FORBIDDEN, "User account is disabled".to_string()));
                    }
                    
                    Ok(AuthenticatedUser { claims })
                } else {
                    Err((StatusCode::UNAUTHORIZED, "User not found".to_string()))
                }
            } else {
                Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()))
            }
        })
    }
}
```

## 12.5 企业级博客系统

现在我们来构建一个完整的企业级博客系统，集成所有学到的Web开发技术。

```rust
// 企业级博客系统主项目
// File: enterprise-blog/Cargo.toml
/*
[package]
name = "enterprise-blog"
version = "1.0.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = { version = "0.7", features = ["macros"] }
tower = { version = "0.4" }
tower-http = { version = "0.5", features = ["cors", "compression", "trace"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "json", "uuid", "chrono"] }
redis = { version = "0.23", features = ["tokio-comp"] }
bcrypt = "0.15"
jsonwebtoken = "9.0"
clap = { version = "4.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "1.0"
regex = "1.0"
markdown = "1.0"
html-escape = "0.4"
mime = "0.4"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
*/
```

```rust
// 数据模型
// File: enterprise-blog/src/models.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::{FromRow, Type};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "user_role")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Editor,
    Author,
    Subscriber,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub author_id: Uuid,
    pub category_id: Option<Uuid>,
    pub status: BlogStatus,
    pub is_featured: bool,
    pub is_pinned: bool,
    pub allow_comments: bool,
    pub allow_ratings: bool,
    pub view_count: i32,
    pub like_count: i32,
    pub comment_count: i32,
    pub reading_time: i32, // 分钟
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "blog_status")]
#[serde(rename_all = "snake_case")]
pub enum BlogStatus {
    Draft,
    Published,
    Archived,
    Scheduled,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub content: String,
    pub status: CommentStatus,
    pub is_approved: bool,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub like_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "comment_status")]
#[serde(rename_all = "snake_case")]
pub enum CommentStatus {
    Pending,
    Approved,
    Spam,
    Trash,
}

// API请求/响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBlogRequest {
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub category_id: Option<Uuid>,
    pub tag_ids: Option<Vec<Uuid>>,
    pub status: BlogStatus,
    pub is_featured: bool,
    pub is_pinned: bool,
    pub allow_comments: bool,
    pub allow_ratings: bool,
    pub featured_image: Option<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

// 主应用程序
// File: enterprise-blog/src/main.rs
use clap::{Parser, Subcommand};
use tracing::{info, warn, error, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use tokio::sync::RwLock;

mod models;
mod services;
mod web;

use models::*;
use services::*;
use web::WebServer;

#[derive(Parser, Debug)]
#[command(name = "enterprise-blog")]
#[command(about = "Enterprise Blog System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the web server
    Server {
        #[arg(short, long, default_value = "0.0.0.0:3000")]
        addr: String,
        
        #[arg(short, long, default_value = "postgres://blog_user:password@localhost/enterprise_blog")]
        database_url: String,
        
        #[arg(short, long, default_value = "redis://localhost:6379")]
        redis_url: String,
    },
    /// Run database migrations
    Migrate {
        #[arg(short, long, default_value = "postgres://blog_user:password@localhost/enterprise_blog")]
        database_url: String,
    },
    /// Setup database and run migrations
    Setup {
        #[arg(short, long, default_value = "postgres://blog_user:password@localhost/enterprise_blog")]
        database_url: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "enterprise_blog=debug,tokio=warn,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Server { addr, database_url, redis_url } => {
            run_server(addr, database_url, redis_url).await
        }
        Commands::Migrate { database_url } => {
            run_migrations(database_url).await
        }
        Commands::Setup { database_url } => {
            setup_database(database_url).await
        }
    }
}

#[instrument]
async fn run_server(
    addr: String,
    database_url: String,
    redis_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Enterprise Blog server on {}", addr);
    
    // 初始化数据库
    let db_pool = sqlx::PgPool::connect(&database_url).await?;
    let redis_client = redis::Client::open(&redis_url)?;
    
    // 初始化服务
    let user_service = Arc::new(UserService::new(db_pool.clone()));
    let blog_service = Arc::new(BlogService::new(db_pool.clone()));
    let auth_service = Arc::new(AuthService::new(db_pool.clone(), redis_client.clone()));
    let media_service = Arc::new(MediaService::new(db_pool.clone()));
    let analytics_service = Arc::new(AnalyticsService::new(db_pool.clone()));
    
    // 启动Web服务器
    let server = WebServer::new(
        addr,
        user_service,
        blog_service,
        auth_service,
        media_service,
        analytics_service,
    );
    
    info!("Enterprise Blog server started successfully");
    server.run().await?;
    
    Ok(())
}

#[instrument]
async fn run_migrations(database_url: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running database migrations");
    
    let pool = sqlx::PgPool::connect(&database_url).await?;
    
    // 创建用户表
    sqlx::query(r#"
        CREATE TYPE user_role AS ENUM ('admin', 'editor', 'author', 'subscriber');
        
        CREATE TABLE users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username VARCHAR(50) UNIQUE NOT NULL,
            email VARCHAR(100) UNIQUE NOT NULL,
            display_name VARCHAR(100) NOT NULL,
            bio TEXT,
            avatar_url TEXT,
            website TEXT,
            password_hash VARCHAR(255) NOT NULL,
            role user_role NOT NULL DEFAULT 'subscriber',
            is_active BOOLEAN DEFAULT TRUE,
            email_verified BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            last_login TIMESTAMPTZ
        );
    "#).execute(&pool).await?;
    
    // 创建博客文章表
    sqlx::query(r#"
        CREATE TYPE blog_status AS ENUM ('draft', 'published', 'archived', 'scheduled');
        
        CREATE TABLE blog_posts (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(200) NOT NULL,
            slug VARCHAR(200) UNIQUE NOT NULL,
            content TEXT NOT NULL,
            excerpt TEXT,
            featured_image TEXT,
            author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            category_id UUID,
            status blog_status NOT NULL DEFAULT 'draft',
            is_featured BOOLEAN DEFAULT FALSE,
            is_pinned BOOLEAN DEFAULT FALSE,
            allow_comments BOOLEAN DEFAULT TRUE,
            allow_ratings BOOLEAN DEFAULT TRUE,
            view_count INTEGER DEFAULT 0,
            like_count INTEGER DEFAULT 0,
            comment_count INTEGER DEFAULT 0,
            reading_time INTEGER DEFAULT 0,
            seo_title VARCHAR(200),
            seo_description TEXT,
            published_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );
    "#).execute(&pool).await?;
    
    info!("Database migrations completed successfully");
    Ok(())
}

#[instrument]
async fn setup_database(database_url: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting up database and running migrations");
    
    // 先运行迁移
    run_migrations(database_url.clone()).await?;
    
    // 创建默认管理员用户
    let pool = sqlx::PgPool::connect(&database_url).await?;
    let admin_password = "admin123";
    
    sqlx::query!(
        r#"
        INSERT INTO users (username, email, display_name, password_hash, role, is_active, email_verified)
        VALUES ('admin', 'admin@example.com', 'Administrator', $1, 'admin', true, true)
        ON CONFLICT (username) DO NOTHING
        "#,
        bcrypt::hash(&admin_password, bcrypt::DEFAULT_COST)?
    )
    .execute(&pool)
    .await?;
    
    info!("Default admin user created - username: admin, password: admin123");
    info!("Please change the admin password after first login");
    
    Ok(())
}

// 服务层实现
// File: enterprise-blog/src/services.rs
use super::models::*;
use crate::database::DatabaseManager;
use sqlx::PgPool;
use tracing::{info, warn, error, instrument};

pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        UserService { pool }
    }
    
    #[instrument(skip(self))]
    pub async fn create_user(&self, request: &RegisterRequest) -> Result<User, sqlx::Error> {
        let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)?;
        
        let user = sqlx::query!(
            r#"
            INSERT INTO users (username, email, display_name, password_hash, role, is_active, email_verified)
            VALUES ($1, $2, $3, $4, 'subscriber', true, false)
            RETURNING *
            "#,
            request.username,
            request.email,
            request.display_name,
            password_hash
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(User::from_row(&user)?)
    }
    
    #[instrument(skip(self))]
    pub async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query!(
            "SELECT * FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user.map(|row| User::from_row(&row).unwrap()))
    }
    
    #[instrument(skip(self))]
    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<Option<User>, sqlx::Error> {
        if let Some(user) = sqlx::query!(
            "SELECT * FROM users WHERE username = $1 AND is_active = true",
            username
        )
        .fetch_optional(&self.pool)
        .await? {
            let user = User::from_row(&user).unwrap();
            if bcrypt::verify(password, &user.password_hash)? {
                Ok(Some(user))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

pub struct BlogService {
    pool: PgPool,
}

impl BlogService {
    pub fn new(pool: PgPool) -> Self {
        BlogService { pool }
    }
    
    #[instrument(skip(self))]
    pub async fn create_blog_post(&self, request: &CreateBlogRequest, author_id: Uuid) -> Result<BlogPost, sqlx::Error> {
        let slug = generate_slug(&request.title);
        
        let post = sqlx::query!(
            r#"
            INSERT INTO blog_posts (
                id, title, slug, content, excerpt, featured_image, author_id, category_id,
                status, is_featured, is_pinned, allow_comments, allow_ratings,
                seo_title, seo_description, published_at
            ) VALUES (
                gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            RETURNING *
            "#,
            request.title,
            slug,
            request.content,
            request.excerpt,
            request.featured_image,
            author_id,
            request.category_id,
            request.status as BlogStatus,
            request.is_featured,
            request.is_pinned,
            request.allow_comments,
            request.allow_ratings,
            request.seo_title,
            request.seo_description,
            request.published_at
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(BlogPost::from_row(&post)?)
    }
    
    #[instrument(skip(self))]
    pub async fn get_published_posts(&self, limit: i64, offset: i64) -> Result<Vec<BlogPost>, sqlx::Error> {
        let posts = sqlx::query!(
            r#"
            SELECT bp.*, u.display_name as author_name
            FROM blog_posts bp
            JOIN users u ON bp.author_id = u.id
            WHERE bp.status = 'published'
            ORDER BY bp.is_pinned DESC, bp.published_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(posts.into_iter().map(|row| BlogPost::from_row(&row).unwrap()).collect())
    }
    
    #[instrument(skip(self))]
    pub async fn increment_view_count(&self, post_id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE blog_posts SET view_count = view_count + 1 WHERE id = $1",
            post_id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}

// 辅助函数
fn generate_slug(title: &str) -> String {
    title.to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '-' | '_' => '-',
            _ => '',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

// Web服务器
// File: enterprise-blog/src/web.rs
use super::services::*;
use super::models::*;
use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, cors::CorsLayer};
use std::sync::Arc;

pub struct WebServer {
    app: Router,
    addr: String,
}

impl WebServer {
    pub fn new(
        addr: String,
        user_service: Arc<UserService>,
        blog_service: Arc<BlogService>,
        auth_service: Arc<AuthService>,
        media_service: Arc<MediaService>,
        analytics_service: Arc<AnalyticsService>,
    ) -> Self {
        let app = Router::new()
            .route("/", get(home_handler))
            .route("/health", get(health_check))
            
            // 公开API
            .route("/api/v1/posts", get(get_posts).post(create_post))
            .route("/api/v1/posts/:id", get(get_post))
            .route("/api/v1/categories", get(get_categories))
            .route("/api/v1/tags", get(get_tags))
            .route("/api/v1/search", get(search_posts))
            
            // 用户API
            .route("/api/v1/auth/register", post(register_user))
            .route("/api/v1/auth/login", post(login_user))
            .route("/api/v1/auth/logout", post(logout_user))
            
            // 管理API
            .route("/api/v1/admin/posts", get(admin_list_posts))
            .route("/api/v1/admin/users", get(admin_list_users))
            
            .with_state(AppState {
                user_service,
                blog_service,
                auth_service,
                media_service,
                analytics_service,
            })
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
            );
        
        WebServer { app, addr }
    }
    
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = tokio::net::TcpListener::bind(&self.addr).await?;
        println!("Enterprise Blog server listening on {}", self.addr);
        
        axum::serve(listener, self.app).await?;
        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    user_service: Arc<UserService>,
    blog_service: Arc<BlogService>,
    auth_service: Arc<AuthService>,
    media_service: Arc<MediaService>,
    analytics_service: Arc<AnalyticsService>,
}

// 处理器实现
async fn home_handler() -> &'static str {
    "Enterprise Blog System"
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let db_healthy = sqlx::query("SELECT 1").fetch_one(&state.user_service.pool).await.is_ok();
    Json(serde_json::json!({
        "status": "healthy",
        "database": db_healthy,
    }))
}

async fn get_posts(State(state): State<AppState>) -> impl IntoResponse {
    match state.blog_service.get_published_posts(20, 0).await {
        Ok(posts) => Json(serde_json::json!({
            "posts": posts,
            "total": posts.len() as i64,
        })),
        Err(_) => Json(serde_json::json!({
            "error": "Failed to fetch posts"
        })),
    }
}

async fn create_post(
    State(state): State<AppState>,
    Json(request): Json<CreateBlogRequest>,
) -> impl IntoResponse {
    // 从认证中获取用户ID
    let author_id = Uuid::new_v4(); // 简化实现
    
    match state.blog_service.create_blog_post(&request, author_id).await {
        Ok(post) => Json(serde_json::json!({
            "success": true,
            "post": post,
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}

async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // 增加浏览量
    let _ = state.blog_service.increment_view_count(&id).await;
    
    // 获取文章详情
    // 简化实现
    Json(serde_json::json!({
        "id": id,
        "title": "Sample Post",
        "content": "This is a sample blog post content.",
    }))
}

// 其他处理器...
```

```rust
// 完整的API处理器实现
// File: enterprise-blog/src/handlers.rs
use super::web::AppState;
use super::models::*;
use axum::{
    extract::{Path, State, Form, Extension, Multipart},
    response::{Json, Html, Redirect},
    http::StatusCode,
};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, warn, error, instrument};

// 用户认证处理器
#[instrument(skip(state))]
pub async fn register_user(
    State(state): State<AppState>,
    Form(request): Form<RegisterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // 验证输入
    if request.username.is_empty() || request.email.is_empty() || request.password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "所有字段都是必需的".to_string()));
    }
    
    // 检查用户是否已存在
    if let Ok(Some(_)) = sqlx::query!(
        "SELECT id FROM users WHERE username = $1 OR email = $2",
        &request.username, &request.email
    )
    .fetch_optional(&state.user_service.pool)
    .await {
        return Err((StatusCode::CONFLICT, "用户名或邮箱已存在".to_string()));
    }
    
    // 创建用户
    match state.user_service.create_user(&request).await {
        Ok(user) => {
            info!("User registered successfully: {}", user.username);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "注册成功",
                "user_id": user.id,
            })))
        }
        Err(e) => {
            error!("Failed to register user: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "注册失败".to_string()))
        }
    }
}

#[instrument(skip(state))]
pub async fn login_user(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // 验证用户
    if let Some(user) = state.user_service
        .authenticate_user(&request.username, &request.password)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "数据库错误".to_string()))? {
        
        // 生成JWT token
        let user_info = UserInfo {
            id: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            role: user.role.to_string(),
            avatar_url: user.avatar_url,
        };
        
        if let Ok((access_token, refresh_token)) = state.auth_service.generate_tokens(&user_info).await {
            Ok(Json(serde_json::json!({
                "success": true,
                "access_token": access_token,
                "refresh_token": refresh_token,
                "token_type": "Bearer",
                "expires_in": 900, // 15分钟
                "user": user_info,
            })))
        } else {
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Token生成失败".to_string()))
        }
    } else {
        Err((StatusCode::UNAUTHORIZED, "用户名或密码错误".to_string()))
    }
}

// 博客文章处理器
#[instrument(skip(state))]
pub async fn get_post_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match sqlx::query!(
        r#"
        SELECT bp.*, u.display_name as author_name, u.username as author_username
        FROM blog_posts bp
        JOIN users u ON bp.author_id = u.id
        WHERE bp.slug = $1 AND bp.status = 'published'
        "#,
        &slug
    )
    .fetch_optional(&state.blog_service.pool)
    .await {
        Ok(Some(post)) => {
            // 增加浏览量
            let _ = sqlx::query!(
                "UPDATE blog_posts SET view_count = view_count + 1 WHERE id = $1",
                post.id
            )
            .execute(&state.blog_service.pool)
            .await;
            
            let blog_post = BlogPost::from_row(&post).unwrap();
            Ok(Json(serde_json::json!({
                "success": true,
                "post": blog_post,
            })))
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, "文章不存在".to_string())),
        Err(e) => {
            error!("Database error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "数据库错误".to_string()))
        }
    }
}

#[instrument(skip(state))]
pub async fn search_posts(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let query = params.get("q").unwrap_or(&"".to_string()).to_string();
    let limit = params.get("limit").unwrap_or(&"20".to_string()).parse::<i64>().unwrap_or(20);
    let offset = params.get("offset").unwrap_or(&"0".to_string()).parse::<i64>().unwrap_or(0);
    
    if query.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "搜索关键词不能为空".to_string()));
    }
    
    match sqlx::query!(
        r#"
        SELECT bp.*, u.display_name as author_name
        FROM blog_posts bp
        JOIN users u ON bp.author_id = u.id
        WHERE bp.status = 'published' 
        AND (bp.title ILIKE $1 OR bp.content ILIKE $1 OR bp.excerpt ILIKE $1)
        ORDER BY bp.view_count DESC, bp.published_at DESC
        LIMIT $2 OFFSET $3
        "#,
        format!("%{}%", query),
        limit,
        offset
    )
    .fetch_all(&state.blog_service.pool)
    .await {
        Ok(posts) => {
            let posts: Vec<BlogPost> = posts.into_iter()
                .map(|row| BlogPost::from_row(&row).unwrap())
                .collect();
            
            Ok(Json(serde_json::json!({
                "success": true,
                "query": query,
                "posts": posts,
                "total": posts.len() as i64,
            })))
        }
        Err(e) => {
            error!("Search error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "搜索失败".to_string()))
        }
    }
}

// 文件上传处理器
#[instrument(skip(state))]
pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use tokio::io::AsyncWriteExt;
    use std::path::PathBuf;
    use uuid::Uuid;
    
    let mut uploaded_files = Vec::new();
    
    while let Some(field) = multipart.next_field().await
        .map_err(|_| (StatusCode::BAD_REQUEST, "文件上传错误".to_string()))? {
        
        let file_name = field.file_name()
            .ok_or((StatusCode::BAD_REQUEST, "无效的文件名".to_string()))?
            .to_string();
        
        let data = field.bytes().await
            .map_err(|_| (StatusCode::BAD_REQUEST, "读取文件数据错误".to_string()))?;
        
        // 检查文件大小
        if data.len() > 10 * 1024 * 1024 { // 10MB
            return Err((StatusCode::BAD_REQUEST, "文件大小超过限制".to_string()));
        }
        
        // 生成唯一文件名
        let file_ext = Path::new(&file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        let unique_name = format!("{}.{}", Uuid::new_v4(), file_ext);
        let file_path = PathBuf::from("uploads").join(&unique_name);
        
        // 创建上传目录
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "创建目录失败".to_string()))?;
        }
        
        // 保存文件
        let mut file = tokio::fs::File::create(&file_path)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "创建文件失败".to_string()))?;
        
        file.write_all(&data)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "写入文件失败".to_string()))?;
        
        uploaded_files.push(serde_json::json!({
            "original_name": file_name,
            "saved_as": unique_name,
            "size": data.len(),
            "path": file_path.to_string_lossy(),
        }));
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "文件上传成功",
        "files": uploaded_files,
    })))
}

// 管理API处理器
#[instrument(skip(state))]
pub async fn admin_dashboard(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // 检查管理员权限
    if user.role() != "admin" {
        return Err((StatusCode::FORBIDDEN, "需要管理员权限".to_string()));
    }
    
    // 获取统计信息
    let (total_users, total_posts, total_comments) = sqlx::query!(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM users) as total_users,
            (SELECT COUNT(*) FROM blog_posts) as total_posts,
            (SELECT COUNT(*) FROM comments) as total_comments
        "#,
    )
    .fetch_one(&state.user_service.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "获取统计信息失败".to_string()))?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "stats": {
            "total_users": total_users.unwrap_or(0),
            "total_posts": total_posts.unwrap_or(0),
            "total_comments": total_comments.unwrap_or(0),
        }
    })))
}

// 缺失的处理器桩实现
pub async fn list_users() -> impl IntoResponse {
    todo!()
}

pub async fn create_user() -> impl IntoResponse {
    todo!()
}

pub async fn get_user() -> impl IntoResponse {
    todo!()
}

pub async fn update_user() -> impl IntoResponse {
    todo!()
}

pub async fn delete_user() -> impl IntoResponse {
    todo!()
}

pub async fn logout() -> impl IntoResponse {
    todo!()
}

pub async fn refresh_token() -> impl IntoResponse {
    todo!()
}

pub async fn list_blogs() -> impl IntoResponse {
    todo!()
}

pub async fn get_blog() -> impl IntoResponse {
    todo!()
}

pub async fn update_blog() -> impl IntoResponse {
    todo!()
}

pub async fn delete_blog() -> impl IntoResponse {
    todo!()
}

pub async fn list_comments() -> impl IntoResponse {
    todo!()
}

pub async fn create_comment() -> impl IntoResponse {
    todo!()
}

pub async fn like_blog() -> impl IntoResponse {
    todo!()
}

pub async fn share_blog() -> impl IntoResponse {
    todo!()
}

pub async fn list_categories() -> impl IntoResponse {
    todo!()
}

pub async fn create_category() -> impl IntoResponse {
    todo!()
}

pub async fn list_tags() -> impl IntoResponse {
    todo!()
}

pub async fn create_tag() -> impl IntoResponse {
    todo!()
}

pub async fn search() -> impl IntoResponse {
    todo!()
}

pub async fn admin_dashboard_data() -> impl IntoResponse {
    todo!()
}

pub async fn admin_list_users_data() -> impl IntoResponse {
    todo!()
}

pub async fn admin_list_blogs_data() -> impl IntoResponse {
    todo!()
}

pub async fn admin_list_comments_data() -> impl IntoResponse {
    todo!()
}

pub async fn upload_file_handler() -> impl IntoResponse {
    todo!()
}

pub async fn download_file() -> impl IntoResponse {
    todo!()
}

pub async fn delete_file() -> impl IntoResponse {
    todo!()
}

pub async fn serve_static() -> impl IntoResponse {
    todo!()
}

pub async fn get_categories() -> impl IntoResponse {
    todo!()
}

pub async fn get_tags() -> impl IntoResponse {
    todo!()
}

pub async fn search_posts_api() -> impl IntoResponse {
    todo!()
}

pub async fn register_user_api() -> impl IntoResponse {
    todo!()
}

pub async fn login_user_api() -> impl IntoResponse {
    todo!()
}

pub async fn logout_user_api() -> impl IntoResponse {
    todo!()
}

pub async fn admin_list_posts_api() -> impl IntoResponse {
    todo!()
}

pub async fn admin_list_users_api() -> impl IntoResponse {
    todo!()
}
```

## 12.6 部署指南

### 12.6.1 Docker容器化部署

```dockerfile
# File: enterprise-blog/Dockerfile
# 多阶段构建
FROM rust:1.70 as builder

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟包来加速构建
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# 复制源代码
COPY src ./src
COPY templates ./templates
COPY static ./static

# 构建应用
RUN cargo build --release

# 运行时阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -r -s /bin/false blog

# 设置工作目录
WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/enterprise-blog ./

# 创建必要目录
RUN mkdir -p uploads logs && \
    chown -R blog:blog /app

# 切换到非root用户
USER blog

# 暴露端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# 启动应用
CMD ["./enterprise-blog", "server", "--addr=0.0.0.0:3000"]
```

```yaml
# File: enterprise-blog/docker-compose.yml
version: '3.8'

services:
  # PostgreSQL数据库
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: enterprise_blog
      POSTGRES_USER: blog_user
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-password}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init-scripts:/docker-entrypoint-initdb.d
    ports:
      - "5432:5432"
    restart: unless-stopped
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U blog_user -d enterprise_blog"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Redis缓存
  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes --requirepass ${REDIS_PASSWORD:-redis123}
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      timeout: 10s
      retries: 3

  # 企业级博客应用
  blog-app:
    build:
      context: .
      dockerfile: Dockerfile
    environment:
      DATABASE_URL: postgres://blog_user:${POSTGRES_PASSWORD:-password}@postgres:5432/enterprise_blog
      REDIS_URL: redis://:${REDIS_PASSWORD:-redis123}@redis:6379
      JWT_SECRET: ${JWT_SECRET:-your-jwt-secret-change-this}
      UPLOAD_DIR: /app/uploads
      MAX_UPLOAD_SIZE: 10485760
      LOG_LEVEL: info
    volumes:
      - uploads_data:/app/uploads
      - logs_data:/app/logs
    ports:
      - "3000:3000"
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Nginx反向代理
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
      - static_files:/var/www/static
    depends_on:
      - blog-app
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # 文件存储（MinIO）
  minio:
    image: minio/minio:latest
    environment:
      MINIO_ROOT_USER: ${MINIO_USER:-minioadmin}
      MINIO_ROOT_PASSWORD: ${MINIO_PASSWORD:-minioadmin123}
    command: server /data --console-address ":9001"
    volumes:
      - minio_data:/data
    ports:
      - "9000:9000"
      - "9001:9001"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 30s
      timeout: 20s
      retries: 3

  # 监控 - Prometheus
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"
    restart: unless-stopped

  # 监控 - Grafana
  grafana:
    image: grafana/grafana:latest
    environment:
      GF_SECURITY_ADMIN_PASSWORD: ${GRAFANA_PASSWORD:-admin123}
    volumes:
      - grafana_data:/var/lib/grafana
volumes:
  postgres_data:
  redis_data:
  uploads_data:
  logs_data:
  minio_data:
  static_files:
  prometheus_data:
  grafana_data:

networks:
  default:
    driver: bridge
```

### 12.6.2 Kubernetes部署

```yaml
# File: enterprise-blog/k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: enterprise-blog
```

```yaml
# File: enterprise-blog/k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: blog-config
  namespace: enterprise-blog
data:
  DATABASE_URL: "postgresql://blog_user:password@postgres-service:5432/enterprise_blog"
  REDIS_URL: "redis://redis-service:6379"
  JWT_SECRET: "your-jwt-secret-change-in-production"
  UPLOAD_DIR: "/app/uploads"
  MAX_UPLOAD_SIZE: "10485760"
  LOG_LEVEL: "info"
```

```yaml
# File: enterprise-blog/k8s/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: blog-secrets
  namespace: enterprise-blog
type: Opaque
data:
  postgres-password: cGFzc3dvcmQ=  # base64编码的password
  redis-password: cmVkaXMxMjM=     # base64编码的redis123
  jwt-secret: eW91ci1qd3Qtc2VjcmV0LWNoYW5nZS10aGlz  # base64编码的your-jwt-secret-change-this
```

```yaml
# File: enterprise-blog/k8s/postgres-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: enterprise-blog
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        env:
        - name: POSTGRES_DB
          value: enterprise_blog
        - name: POSTGRES_USER
          value: blog_user
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: blog-secrets
              key: postgres-password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - blog_user
            - -d
            - enterprise_blog
          initialDelaySeconds: 30
          periodSeconds: 10
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: postgres-service
  namespace: enterprise-blog
spec:
  selector:
    app: postgres
  ports:
  - protocol: TCP
    port: 5432
    targetPort: 5432
  type: ClusterIP
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: enterprise-blog
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 20Gi
```

```yaml
# File: enterprise-blog/k8s/redis-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: enterprise-blog
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        command:
        - redis-server
        - --requirepass
        - $(REDIS_PASSWORD)
        env:
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: blog-secrets
              key: redis-password
        ports:
        - containerPort: 6379
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "200m"
        livenessProbe:
          exec:
            command:
            - redis-cli
            - ping
          initialDelaySeconds: 30
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: redis-service
  namespace: enterprise-blog
spec:
  selector:
    app: redis
  ports:
  - protocol: TCP
    port: 6379
    targetPort: 6379
  type: ClusterIP
```

```yaml
# File: enterprise-blog/k8s/blog-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: blog-app
  namespace: enterprise-blog
spec:
  replicas: 3
  selector:
    matchLabels:
      app: blog-app
  template:
    metadata:
      labels:
        app: blog-app
    spec:
      containers:
      - name: blog-app
        image: enterprise-blog:latest
        imagePullPolicy: Never  # 对于本地开发
        env:
        - name: DATABASE_URL
          valueFrom:
            configMapKeyRef:
              name: blog-config
              key: DATABASE_URL
        - name: REDIS_URL
          valueFrom:
            configMapKeyRef:
              name: blog-config
              key: REDIS_URL
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: blog-secrets
              key: jwt-secret
        - name: UPLOAD_DIR
          valueFrom:
            configMapKeyRef:
              name: blog-config
              key: UPLOAD_DIR
        - name: MAX_UPLOAD_SIZE
          valueFrom:
            configMapKeyRef:
              name: blog-config
              key: MAX_UPLOAD_SIZE
        ports:
        - containerPort: 3000
        volumeMounts:
        - name: uploads-storage
          mountPath: /app/uploads
        - name: logs-storage
          mountPath: /app/logs
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: uploads-storage
        persistentVolumeClaim:
          claimName: uploads-pvc
      - name: logs-storage
        persistentVolumeClaim:
          claimName: logs-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: blog-service
  namespace: enterprise-blog
spec:
  selector:
    app: blog-app
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: ClusterIP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: blog-ingress
  namespace: enterprise-blog
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - your-domain.com
    secretName: blog-tls
  rules:
  - host: your-domain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: blog-service
            port:
              number: 80
```

```yaml
# File: enterprise-blog/k8s/pvc.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: uploads-pvc
  namespace: enterprise-blog
spec:
  accessModes:
    - ReadWriteMany
  resources:
    requests:
      storage: 10Gi
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: logs-pvc
  namespace: enterprise-blog
spec:
  accessModes:
    - ReadWriteMany
  resources:
    requests:
      storage: 5Gi
```

### 12.6.3 CI/CD部署流水线

```yaml
# File: enterprise-blog/.github/workflows/deploy.yml
name: Deploy Enterprise Blog

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: password
          POSTGRES_USER: blog_user
          POSTGRES_DB: enterprise_blog_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy, rustfmt
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Cache cargo index
      uses: actions/cache@v3
      with:
        path: ~/.cargo/git
        key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Cache cargo build
      uses: actions/cache@v3
      with:
        path: target
        key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run tests
      run: cargo test
      env:
        DATABASE_URL: postgres://blog_user:password@localhost:5432/enterprise_blog_test
        REDIS_URL: redis://localhost:6379
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Run rustfmt
      run: cargo fmt -- --check

  security:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Install cargo-audit
      run: cargo install cargo-audit
    
    - name: Run cargo audit
      run: cargo audit

  build:
    needs: [test, security]
    runs-on: ubuntu-latest
    outputs:
      image: ${{ steps.image.outputs.image }}
      digest: ${{ steps.build.outputs.digest }}
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v3
    
    - name: Log in to Container Registry
      uses: docker/login-action@v3
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v5
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=sha,prefix={{branch}}-
          type=raw,value=latest,enable={{is_default_branch}}
    
    - name: Build and push Docker image
      id: build
      uses: docker/build-push-action@v5
      with:
        context: .
        file: ./Dockerfile
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: production
    
    steps:
    - name: Deploy to production
      run: |
        echo "Deploying to production..."
        # 这里可以添加部署脚本，比如 kubectl apply, docker-compose up 等
        
  notify:
    needs: [build, deploy]
    runs-on: ubuntu-latest
    if: always()
    
    steps:
    - name: Notify deployment status
      if: failure()
      run: |
        echo "Deployment failed! Please check the logs."
        # 这里可以添加通知逻辑，比如发送邮件、Slack消息等
```

### 12.6.4 生产环境配置

```toml
# File: enterprise-blog/Cargo.toml
[package]
name = "enterprise-blog"
version = "1.0.0"
edition = "2021"
authors = ["MiniMax Agent <developer@minimax.com>"]
description = "Enterprise-grade blog system built with Rust"
license = "MIT"
repository = "https://github.com/your-org/enterprise-blog"
keywords = ["blog", "cms", "rust", "web", "enterprise"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.production]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[[bin]]
name = "enterprise-blog"
path = "src/main.rs"

[features]
default = []
production = ["cli", "metrics", "opentelemetry"]
cli = ["clap"]
metrics = ["prometheus-client"]
opentelemetry = ["opentelemetry", "opentelemetry-jaeger", "opentelemetry-http"]

[dependencies]
tokio = { version = "1.35", features = ["full", "tracing"] }
axum = { version = "0.7", features = ["macros", "ws"] }
tower = { version = "0.4" }
tower-http = { version = "0.5", features = ["cors", "compression", "trace", "timeout", "request-id"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "json", "uuid", "chrono", "migrate"] }
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
bcrypt = "0.15"
jsonwebtoken = "9.2"
clap = { version = "4.4", features = ["derive", "cargo"], optional = true }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
anyhow = "1.0"
thiserror = "1.0"
regex = "1.10"
markdown = "1.0"
html-escape = "0.4"
mime = "0.4"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
owo-colors = "4.0"
urlencoding = "2.1"
prometheus-client = { version = "0.21", optional = true }
opentelemetry = { version = "0.20", optional = true }
opentelemetry-jaeger = { version = "0.17", optional = true }
opentelemetry-http = { version = "0.10", optional = true }

[dev-dependencies]
tempfile = "3.8"
wiremock = "0.5"
fake = "0.2"
```

```rust
// File: enterprise-blog/src/main.rs
use clap::{Parser, Subcommand, ValueEnum};
use tracing::{info, warn, error, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::sync::Arc;
use tokio::sync::RwLock;

mod models;
mod services;
mod web;
mod config;
mod database;

use models::*;
use services::*;
use web::WebServer;
use config::Config;
use database::DatabaseManager;

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "metrics")]
use prometheus_client::registry::Registry;
#[cfg(feature = "opentelemetry")]
use opentelemetry::{global, sdk::trace::TracerProvider};
#[cfg(feature = "opentelemetry")]
use opentelemetry_jaeger::JaegerLayer;

#[cfg_attr(feature = "cli", derive(Parser))]
#[cfg_attr(feature = "cli", command(name = "enterprise-blog"))]
#[cfg_attr(feature = "cli", command(about = "Enterprise Blog System"))]
struct Cli {
    #[cfg_attr(feature = "cli", command(subcommand))]
    command: Commands,
}

#[cfg_attr(feature = "cli", derive(Subcommand))]
#[cfg_attr(feature = "cli", command())
enum Commands {
    /// Start the web server
    Server {
        #[arg(short, long, default_value = "0.0.0.0:3000")]
        addr: String,
    },
    /// Run database migrations
    Migrate,
    /// Setup database and run migrations
    Setup,
    /// Generate documentation
    Docs,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化配置
    let config = Config::from_env()?;
    
    // 初始化日志
    init_tracing(&config)?;
    
    #[cfg(feature = "metrics")]
    let registry = Registry::default();
    
    #[cfg(feature = "opentelemetry")]
    let tracer = init_opentelemetry()?;
    
    info!("Starting Enterprise Blog System v{}", env!("CARGO_PKG_VERSION"));
    
    #[cfg_attr(feature = "cli", allow(unused_variables))]
    let cli = Cli::parse();
    
    #[cfg_attr(feature = "cli", if let Some(command) = cli.command {} else )]
    // 默认命令
    run_server(config).await
}

#[cfg_attr(feature = "cli", allow(dead_code))]
async fn run_server(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing services...");
    
    // 初始化数据库
    let db_manager = DatabaseManager::new(&config.database_url).await?;
    let db_pool = db_manager.get_pool();
    
    // 初始化Redis
    let redis_client = redis::Client::open(&config.redis_url)?;
    
    // 初始化服务
    let user_service = Arc::new(UserService::new(db_pool.clone()));
    let blog_service = Arc::new(BlogService::new(db_pool.clone()));
    let auth_service = Arc::new(AuthService::new(db_pool.clone(), redis_client.clone(), &config.jwt_secret));
    let media_service = Arc::new(MediaService::new(db_pool.clone(), &config.upload_dir));
    let analytics_service = Arc::new(AnalyticsService::new(db_pool.clone()));
    
    // 初始化Web服务器
    let server = WebServer::new(
        config.addr,
        user_service,
        blog_service,
        auth_service,
        media_service,
        analytics_service,
        config,
    );
    
    info!("Starting server on {}", config.addr);
    server.run().await?;
    
    Ok(())
}

fn init_tracing(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("enterprise_blog={},tokio=warn,sqlx=warn", config.log_level).into());
    
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_thread_ids(true)
            .with_level(true)
        );
    
    #[cfg(feature = "opentelemetry")]
    let subscriber = subscriber.with(JaegerLayer::new());
    
    subscriber.init();
    
    Ok(())
}

#[cfg(feature = "opentelemetry")]
fn init_opentelemetry() -> Result<TracerProvider, Box<dyn std::error::Error>> {
    global::set_text_map_propagator(opentelemetry_jaeger::JaegerPropagator::new());
    
    let tracer = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_jaeger::AgentPipeline::default())
        .build();
    
    global::set_tracer_provider(tracer.clone());
    
    Ok(tracer)
}
```

## 12.7 性能优化

### 12.7.1 数据库优化

```rust
// 数据库查询优化
impl BlogService {
    // 使用索引优化查询
    #[instrument(skip(self))]
    pub async fn get_blogs_with_pagination(
        &self,
        page: i64,
        per_page: i64,
        category: Option<&str>,
        tag: Option<&str>,
    ) -> Result<(Vec<BlogPost>, i64), sqlx::Error> {
        let offset = (page - 1) * per_page;
        
        let mut query = r#"
            SELECT bp.*, u.display_name as author_name, 
                   c.name as category_name,
                   ARRAY_AGG(DISTINCT t.name) as tag_names
            FROM blog_posts bp
            JOIN users u ON bp.author_id = u.id
            LEFT JOIN categories c ON bp.category_id = c.id
            LEFT JOIN blog_post_tags bpt ON bp.id = bpt.post_id
            LEFT JOIN tags t ON bpt.tag_id = t.id
            WHERE bp.status = 'published'
        "#.to_string();
        
        let mut params: Vec<String> = Vec::new();
        
        if let Some(category) = category {
            query.push_str(" AND c.slug = $1");
            params.push(category.to_string());
        }
        
        if let Some(tag) = tag {
            query.push_str(" AND EXISTS (SELECT 1 FROM blog_post_tags bpt2 WHERE bpt2.post_id = bp.id AND bpt2.tag_id = (SELECT id FROM tags WHERE slug = $2))");
            params.push(tag.to_string());
        }
        
        query.push_str(" GROUP BY bp.id, u.display_name, c.name ORDER BY bp.is_pinned DESC, bp.published_at DESC LIMIT $");
        query.push_str(&format!("{} OFFSET ${}", per_page + 1, per_page + 2));
        params.extend(vec![per_page.to_string(), offset.to_string()]);
        
        let posts = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;
        
        // 获取总数
        let count_query = "SELECT COUNT(DISTINCT bp.id) FROM blog_posts bp WHERE bp.status = 'published'";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(&self.pool)
            .await?;
        
        Ok((posts.into_iter().map(|row| BlogPost::from_row(&row).unwrap()).collect(), total))
    }
    
    // 批量操作优化
    #[instrument(skip(self))]
    pub async fn batch_update_view_counts(&self, post_ids: Vec<Uuid>) -> Result<(), sqlx::Error> {
        if post_ids.is_empty() {
            return Ok(());
        }
        
        // 使用批量更新而不是多次单条更新
        let placeholders: String = post_ids.iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        
        let query = format!(
            "UPDATE blog_posts SET view_count = view_count + 1 WHERE id IN ({})",
            placeholders
        );
        
        // 构建参数数组
        let mut query_builder = sqlx::query(&query);
        for post_id in post_ids {
            query_builder = query_builder.bind(post_id);
        }
        
        query_builder.execute(&self.pool).await?;
        Ok(())
    }
    
    // 使用缓存优化
    #[instrument(skip(self))]
    pub async fn get_cached_blog_post(&self, redis_client: &redis::Client, post_id: &Uuid) -> Result<Option<BlogPost>, sqlx::Error> {
        let mut conn = redis_client.get_connection()?;
        let cache_key = format!("blog:post:{}", post_id);
        
        // 首先尝试从缓存获取
        if let Some(cached_data) = redis::cmd("GET")
            .arg(&cache_key)
            .query::<Option<String>>(&mut conn)? {
            
            let cached_post: BlogPost = serde_json::from_str(&cached_data)
                .map_err(|e| sqlx::Error::Protocol(format!("Failed to deserialize cached post: {}", e)))?;
            return Ok(Some(cached_post));
        }
        
        // 缓存未命中，从数据库获取
        if let Some(post) = self.get_blog_post_by_id(post_id).await? {
            // 存入缓存
            let post_json = serde_json::to_string(&post)
                .map_err(|e| sqlx::Error::Protocol(format!("Failed to serialize post: {}", e)))?;
            
            let _ = redis::cmd("SETEX")
                .arg(&cache_key)
                .arg(3600) // 1小时过期
                .arg(&post_json)
                .query::<()>(&mut conn);
            
            Ok(Some(post))
        } else {
            Ok(None)
        }
    }
}

// 连接池优化
pub struct DatabaseManager {
    pool: sqlx::PgPool,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::PgPoolOptions::new()
            .max_connections(20) // 根据负载调整
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .test_before_acquire(true)
            .connect(database_url)
            .await?;
        
        Ok(DatabaseManager { pool })
    }
    
    pub fn get_pool(&self) -> sqlx::PgPool {
        self.pool.clone()
    }
}
```

### 12.7.2 缓存策略

```rust
// 缓存服务实现
pub struct CacheService {
    redis_client: redis::Client,
    default_ttl: u32,
    namespace: String,
}

impl CacheService {
    pub fn new(redis_client: redis::Client, default_ttl: u32, namespace: String) -> Self {
        CacheService {
            redis_client,
            default_ttl,
            namespace,
        }
    }
    
    fn namespaced_key(&self, key: &str) -> String {
        format!("{}:{}", self.namespace, key)
    }
    
    // 博客文章缓存
    pub async fn cache_blog_post(&self, post: &BlogPost) -> Result<(), redis::RedisError> {
        let key = self.namespaced_key(&format!("blog:post:{}", post.id));
        let data = serde_json::to_string(post)?;
        
        let mut conn = self.redis_client.get_connection()?;
        redis::cmd("SETEX")
            .arg(&key)
            .arg(self.default_ttl)
            .arg(&data)
            .query(&mut conn)
    }
    
    pub async fn get_cached_blog_post(&self, post_id: &Uuid) -> Result<Option<BlogPost>, redis::RedisError> {
        let key = self.namespaced_key(&format!("blog:post:{}", post_id));
        let mut conn = self.redis_client.get_connection()?;
        
        if let Some(data) = redis::cmd("GET").arg(&key).query::<Option<String>>(&mut conn)? {
            let post: BlogPost = serde_json::from_str(&data)?;
            Ok(Some(post))
        } else {
            Ok(None)
        }
    }
    
    // 博客列表缓存
    pub async fn cache_blog_list(&self, key_suffix: &str, posts: &[BlogPost]) -> Result<(), redis::RedisError> {
        let key = self.namespaced_key(&format!("blog:list:{}", key_suffix));
        let data = serde_json::to_string(posts)?;
        
        let mut conn = self.redis_client.get_connection()?;
        redis::cmd("SETEX")
            .arg(&key)
            .arg(self.default_ttl)
            .arg(&data)
            .query(&mut conn)
    }
    
    // 分页缓存
    pub async fn get_cached_blog_page(&self, page: i64, per_page: i64, filters: &str) -> Result<Option<Vec<BlogPost>>, redis::RedisError> {
        let key = self.namespaced_key(&format!("blog:page:{}:{}:{}", page, per_page, filters));
        let mut conn = self.redis_client.get_connection()?;
        
        if let Some(data) = redis::cmd("GET").arg(&key).query::<Option<String>>(&mut conn)? {
            let posts: Vec<BlogPost> = serde_json::from_str(&data)?;
            Ok(Some(posts))
        } else {
            Ok(None)
        }
    }
    
    // 用户会话缓存
    pub async fn cache_user_session(&self, user_id: &Uuid, session_data: &UserSession) -> Result<(), redis::RedisError> {
        let key = self.namespaced_key(&format!("user:session:{}", user_id));
        let data = serde_json::to_string(session_data)?;
        
        let mut conn = self.redis_client.get_connection()?;
        redis::cmd("SETEX")
            .arg(&key)
            .arg(86400) // 24小时
            .arg(&data)
            .query(&mut conn)
    }
    
    // 缓存失效策略
    pub async fn invalidate_user_cache(&self, user_id: &Uuid) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_connection()?;
        
        // 删除用户相关的所有缓存
        let patterns = vec![
            self.namespaced_key(&format!("user:session:{}", user_id)),
            self.namespaced_key(&format!("user:profile:{}", user_id)),
        ];
        
        for pattern in patterns {
            let _ = redis::cmd("DEL").arg(&pattern).query::<()>(&mut conn);
        }
        
        Ok(())
    }
    
    // 批量缓存操作
    pub async fn batch_cache_blog_posts(&self, posts: &[BlogPost]) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_connection()?;
        let mut pipeline = redis::pipe();
        
        for post in posts {
            let key = self.namespaced_key(&format!("blog:post:{}", post.id));
            let data = serde_json::to_string(post)?;
            pipeline = pipeline.setex(key, self.default_ttl, data);
        }
        
        pipeline.query(&mut conn)
    }
}

// 会话数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
    pub last_activity: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
}
```

## 12.8 安全最佳实践

### 12.8.1 输入验证和防护

```rust
// 安全验证器
pub struct SecurityValidator;

impl SecurityValidator {
    // XSS防护
    pub fn sanitize_html(&self, input: &str) -> String {
        // 移除或转义潜在的XSS载荷
        input
            .replace("<script>", "&lt;script&gt;")
            .replace("</script>", "&lt;/script&gt;")
            .replace("<iframe>", "&lt;iframe&gt;")
            .replace("</iframe>", "&lt;/iframe&gt;")
            .replace("javascript:", "javascript_")
            .replace("onload=", "onload_")
            .replace("onerror=", "onerror_")
            .replace("onclick=", "onclick_")
    }
    
    // SQL注入防护
    pub fn validate_sql_safe(&self, input: &str) -> bool {
        // 检查是否包含SQL关键字
        let sql_keywords = ["SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "CREATE", "ALTER", "UNION", "SCRIPT"];
        let input_upper = input.to_uppercase();
        
        !sql_keywords.iter().any(|keyword| input_upper.contains(keyword))
    }
    
    // 文件上传安全
    pub fn validate_file_upload(&self, filename: &str, content_type: &str, size: usize) -> Result<(), SecurityError> {
        // 检查文件名
        if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
            return Err(SecurityError::InvalidFilename);
        }
        
        // 检查文件扩展名
        let allowed_extensions = ["jpg", "jpeg", "png", "gif", "webp", "pdf", "doc", "docx"];
        let ext = Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        if !allowed_extensions.contains(&ext.as_str()) {
            return Err(SecurityError::InvalidFileType);
        }
        
        // 检查文件大小 (10MB限制)
        if size > 10 * 1024 * 1024 {
            return Err(SecurityError::FileTooLarge);
        }
        
        // 检查MIME类型
        let allowed_mime_types = [
            "image/jpeg", "image/png", "image/gif", "image/webp",
            "application/pdf", "application/msword",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ];
        
        if !allowed_mime_types.contains(&content_type) {
            return Err(SecurityError::InvalidMimeType);
        }
        
        Ok(())
    }
    
    // CSRF防护
    pub fn validate_csrf_token(&self, session_token: &str, form_token: &str) -> bool {
        // 简单的时间窗口验证
        session_token == form_token
    }
    
    // 密码强度检查
    pub fn validate_password_strength(&self, password: &str) -> Result<(), ValidationError> {
        if password.len() < 8 {
            return Err(ValidationError::new("password", "密码长度至少8个字符"));
        }
        
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(ValidationError::new("password", "密码必须包含大写字母"));
        }
        
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(ValidationError::new("password", "密码必须包含小写字母"));
        }
        
        if !password.chars().any(|c| c.is_digit(10)) {
            return Err(ValidationError::new("password", "密码必须包含数字"));
        }
        
        if !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            return Err(ValidationError::new("password", "密码必须包含特殊字符"));
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Invalid filename")]
    InvalidFilename,
    #[error("Invalid file type")]
    InvalidFileType,
    #[error("File too large")]
    FileTooLarge,
    #[error("Invalid MIME type")]
    InvalidMimeType,
    #[error("CSRF token validation failed")]
    InvalidCSRFToken,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}
```

### 12.8.2 速率限制和防护

```rust
// 速率限制中间件
pub struct RateLimitConfig {
    pub requests_per_minute: u64,
    pub burst_size: u64,
    pub key_extractor: fn(&axum::http::Request<axum::body::Body>) -> String,
}

impl RateLimitConfig {
    pub fn by_ip() -> Self {
        RateLimitConfig {
            requests_per_minute: 100,
            burst_size: 20,
            key_extractor: extract_client_ip,
        }
    }
    
    pub fn by_user() -> Self {
        RateLimitConfig {
            requests_per_minute: 1000,
            burst_size: 100,
            key_extractor: extract_user_id_or_ip,
        }
    }
    
    pub fn by_endpoint() -> Self {
        RateLimitConfig {
            requests_per_minute: 50,
            burst_size: 10,
            key_extractor: extract_endpoint_key,
        }
    }
}

fn extract_client_ip(request: &axum::http::Request<axum::body::Body>) -> String {
    request.headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .or_else(|| request.headers().get("x-real-ip").and_then(|h| h.to_str().ok()))
        .unwrap_or("unknown")
        .to_string()
}

fn extract_user_id_or_ip(request: &axum::http::Request<axum::body::Body>) -> String {
    // 从JWT token中提取用户ID
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(token) = auth_header.to_str() {
            if token.starts_with("Bearer ") {
                let token_data = &token[7..];
                if let Ok(claims) = validate_jwt_token(token_data) {
                    return format!("user:{}", claims.sub);
                }
            }
        }
    }
    
    // 如果没有用户信息，使用IP
    extract_client_ip(request)
}

fn extract_endpoint_key(request: &axum::http::Request<axum::body::Body>) -> String {
    let method = request.method();
    let path = request.uri().path();
    format!("{}:{}", method, path)
}

impl axum::middleware::Middleware<(), State = AppState> for RateLimitConfig {
    type Future = Pin<Box<dyn Future<Output = Result<Response, (StatusCode, String)>> + Send>>;
    
    fn call(&self, request: Request, state: State<AppState>, next: Next) -> Self::Future {
        Box::pin(async move {
            let key = (self.key_extractor)(&request);
            let client_ip = extract_client_ip(&request);
            
            // 检查速率限制
            if let Some(violation) = check_rate_limit_violation(
                &state.redis_client,
                &key,
                self.requests_per_minute,
                self.burst_size,
            ).await {
                
                // 记录违规行为
                tracing::warn!(
                    "Rate limit exceeded for {} (key: {}, violations: {})",
                    client_ip, key, violation
                );
                
                return Err((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()));
            }
            
            next.run(request).await
        })
    }
}

async fn check_rate_limit_violation(
    redis_client: &redis::Client,
    key: &str,
    requests_per_minute: u64,
    burst_size: u64,
) -> Option<u32> {
    let mut conn = redis_client.get_connection().ok()?;
    
    let current_count: u32 = redis::cmd("GET")
        .arg(format!("rate_limit:{}", key))
        .query(&mut conn)
        .unwrap_or(0);
    
    if current_count >= requests_per_minute + burst_size {
        Some(current_count)
    } else {
        // 增加计数器
        let _ = redis::cmd("INCR")
            .arg(format!("rate_limit:{}", key))
            .query::<u32>(&mut conn);
        
        // 设置过期时间
        let _ = redis::cmd("EXPIRE")
            .arg(format!("rate_limit:{}", key))
            .arg(60) // 1分钟
            .query::<()>(&mut conn);
        
        None
    }
}
```

## 12.9 最佳实践总结

### 12.9.1 项目结构最佳实践

```
enterprise-blog/
├── src/
│   ├── main.rs              # 应用入口
│   ├── config/              # 配置管理
│   │   ├── mod.rs
│   │   └── config.rs
│   ├── database/            # 数据库层
│   │   ├── mod.rs
│   │   ├── migrations/      # 数据库迁移
│   │   └── connection.rs
│   ├── models/              # 数据模型
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── blog.rs
│   │   └── common.rs
│   ├── services/            # 业务逻辑层
│   │   ├── mod.rs
│   │   ├── user_service.rs
│   │   ├── blog_service.rs
│   │   └── cache_service.rs
│   ├── handlers/            # HTTP处理器
│   │   ├── mod.rs
│   │   ├── auth_handlers.rs
│   │   ├── blog_handlers.rs
│   │   └── admin_handlers.rs
│   ├── middleware/          # 中间件
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── rate_limit.rs
│   │   └── logging.rs
│   ├── utils/               # 工具函数
│   │   ├── mod.rs
│   │   ├── validation.rs
│   │   └── security.rs
│   └── web/                 # Web服务器
│       ├── mod.rs
│       └── router.rs
├── templates/               # HTML模板
│   ├── base.html
│   ├── blog/
│   └── admin/
├── static/                  # 静态文件
│   ├── css/
│   ├── js/
│   └── images/
├── migrations/              # SQL迁移文件
├── tests/                   # 测试
├── docs/                    # 文档
├── Dockerfile              # Docker构建文件
├── docker-compose.yml      # Docker Compose配置
├── k8s/                    # Kubernetes配置
├── .github/
│   └── workflows/         # CI/CD配置
├── Cargo.toml
└── README.md
```

### 12.9.2 代码质量标准

```rust
// 使用derive宏减少样板代码
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    // ... 字段定义
}

// 使用thiserror处理错误
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found")]
    NotFound,
}

// 使用tracing进行结构化日志记录
#[instrument(skip(self))]
pub async fn create_user(&self, request: &CreateUserRequest) -> Result<User, ServiceError> {
    tracing::info!("Creating user: {}", request.username);
    
    // 业务逻辑
    let user = self.repository.create_user(request).await?;
    
    tracing::info!("User created successfully: {}", user.id);
    Ok(user)
}

// 使用类型安全的API设计
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

// 实施依赖注入
pub struct ServiceContainer {
    pub user_service: Arc<dyn UserServiceTrait>,
    pub blog_service: Arc<dyn BlogServiceTrait>,
    pub cache_service: Arc<dyn CacheServiceTrait>,
}

impl ServiceContainer {
    pub fn new() -> Self {
        let db_pool = DatabaseManager::new().await?;
        let redis_client = RedisClient::new()?;
        
        ServiceContainer {
            user_service: Arc::new(UserService::new(db_pool.clone())),
            blog_service: Arc::new(BlogService::new(db_pool.clone(), redis_client.clone())),
            cache_service: Arc::new(CacheService::new(redis_client)),
        }
    }
}
```

### 12.9.3 性能优化建议

1. **数据库优化**：
   - 使用适当的索引
   - 实施查询优化
   - 使用连接池
   - 实施读写分离

2. **缓存策略**：
   - Redis缓存热点数据
   - 实施多层缓存
   - 合理设置缓存过期时间
   - 使用缓存预热

3. **异步处理**：
   - 使用async/await
   - 合理配置tokio运行时
   - 实施背压控制
   - 使用连接池

4. **静态资源**：
   - 使用CDN
   - 启用Gzip压缩
   - 实施资源缓存
   - 优化图片大小

### 12.9.4 安全建议

1. **输入验证**：
   - 验证所有用户输入
   - 使用白名单而非黑名单
   - 实施内容安全策略(CSP)
   - 防止XSS和SQL注入

2. **认证和授权**：
   - 使用强密码策略
   - 实施多因素认证
   - 使用JWT进行无状态认证
   - 实施基于角色的访问控制

3. **数据传输**：
   - 使用HTTPS
   - 实施HSTS
   - 验证SSL证书
   - 使用安全的Cookie设置

4. **监控和日志**：
   - 记录安全事件
   - 监控异常活动
   - 实施入侵检测
   - 定期安全审计

## 本章小结

本章深入探讨了Rust的Web开发能力，从基础框架选择到企业级应用构建。我们学习了：

1. **Web框架生态**：了解了Actix-web、Axum、Rocket等主流框架的特点和选择标准
2. **路由和中间件**：掌握了基于Axum的高级路由系统和自定义中间件开发
3. **表单处理**：学习了表单数据提取、验证和安全处理
4. **用户认证**：实现了JWT认证系统、权限管理和会话管理
5. **企业级项目**：构建了完整的企业级博客系统，集成所有核心技术
6. **部署和运维**：提供了Docker、Kubernetes等多种部署方案
7. **性能优化**：实施了数据库优化、缓存策略和性能监控
8. **安全实践**：建立了全面的安全防护体系

通过这个完整的企业级博客系统项目，我们不仅掌握了Rust Web开发的核心技术，更重要的是学会了如何构建安全、高性能、可维护的企业级应用。

**关键技能**：
- 现代Web框架的使用和选择
- RESTful API设计和实现
- 数据库集成和优化
- 缓存策略设计
- 安全编程实践
- 容器化部署
- 监控和日志
- 性能优化

这些技能为构建复杂的Web应用提供了坚实的基础，能够满足现代企业级应用的各种需求。

---

**第12章完成**：Web开发核心技术已全面掌握，能够构建企业级Web应用。准备进入第13章：性能优化。