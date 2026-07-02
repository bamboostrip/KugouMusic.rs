//! 注册控制器 —— 对应 .NET `RegisterController`（前缀 `register`）。

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::error::AppResult;
use crate::middleware::{KgReqSession, KgSessionKey};
use crate::services;
use crate::state::AppState;

#[derive(Debug, ToSchema)]
pub struct RegisterDevResponse {
    pub success: bool,
}

/// `GET /register/dev` —— 注册设备（产出真实 dfid）。匿名可用。
#[utoipa::path(get, path = "/register/dev", tag = "register", responses((status = 200, body = RegisterDevResponse)))]
async fn register_dev(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    KgSessionKey(session_key): KgSessionKey,
) -> AppResult<Json<Value>> {
    let ok = services::register::register_device(&state, &session_key, &session).await?;
    Ok(Json(json!({ "success": ok })))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(register_dev))
}
