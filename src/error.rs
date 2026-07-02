//! 统一错误类型。
//!
//! 全局返回 [`AppError`]，它实现了 [`IntoResponse`]，会被 Axum 自动转成
//! 形如 `{"status":400,"msg":"...","errorCode":...}` 的 JSON。
//! 业务层（services）只需 `Result<T, AppError>` 即可，错误码集中维护。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    /// 入参校验失败（validator crate）
    #[error("参数校验失败: {0}")]
    Validation(String),

    /// 资源不存在
    #[error("资源不存在: {0}")]
    NotFound(String),

    /// 鉴权 / 会话相关（实现登录态/会话中间件后启用）
    #[error("未授权: {0}")]
    #[allow(dead_code)]
    Unauthorized(String),

    /// 上游酷狗 API 请求失败（实现代理 service 后启用）
    #[error("上游请求失败: {0}")]
    #[allow(dead_code)]
    Upstream(String),

    /// 数据库错误
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    /// 其它内部错误
    #[error("内部错误: {0}")]
    Internal(anyhow::Error),
}

/// 让 anyhow::Error 能用 `?` 优雅转成 AppError::Internal
impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e)
    }
}

/// 透传 validator 的校验错误集合
impl From<validator::ValidationErrors> for AppError {
    fn from(e: validator::ValidationErrors) -> Self {
        AppError::Validation(e.to_string())
    }
}

/// 统一的错误响应体（与 .NET 端 ApiErrorResponse 形状保持一致，前端零改动）
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ApiErrorResponse {
    pub status: u16,
    pub msg: String,
    pub error_code: i32,
}

impl AppError {
    /// 映射到 HTTP 状态码 + 内部 error_code
    fn parts(&self) -> (StatusCode, i32) {
        match self {
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, 400),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, 404),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, 401),
            AppError::Upstream(_) => (StatusCode::BAD_GATEWAY, 502),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, 500),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, 500),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code) = self.parts();

        // 5xx 才记 error 级别日志，4xx 用 warn，避免日志噪音
        if status.is_server_error() {
            tracing::error!(error = %self, code = error_code, "请求处理失败");
        } else {
            tracing::warn!(error = %self, code = error_code, "请求处理失败");
        }

        let body = ApiErrorResponse {
            status: status.as_u16(),
            msg: self.to_string(),
            error_code,
        };

        (status, Json(body)).into_response()
    }
}

/// 业务 handler 的常用返回类型别名
pub type AppResult<T> = Result<T, AppError>;
