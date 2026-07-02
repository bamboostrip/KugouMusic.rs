//! 全局共享状态：会被 clone 进每个请求的 handler。
//!
//! `AppState` 内部都是 `Arc`/连接池，clone 开销极低。
//! 这是 Axum 推荐的 State 模式（`Router::with_state`）。

use sqlx::SqlitePool;

/// 应用级共享状态。
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    /// 复用连接池的 HTTPS 客户端，代理上游酷狗时统一用它（transport 层消费）
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(db: SqlitePool, http: reqwest::Client) -> Self {
        Self { db, http }
    }
}
