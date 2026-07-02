//! 健康检查接口。
//! 不碰数据库 / 上游，保证容器探针总是能拿到稳定结果。

use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

/// `GET /health` —— 存活探针。
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "服务存活", body = HealthResponse)
    )
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "kugou_web_api",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// 返回本模块路由（尚未绑定 state，由上层 `with_state` 注入）。
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(health))
}
