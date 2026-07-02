//! 电台控制器 —— 对应 .NET `FmController`（前缀 `fm`）。4 端点。

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::error::AppResult;
use crate::middleware::KgReqSession;
use crate::services;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct FmEmptyQuery {}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct FmSongsQuery {
    #[validate(length(min = 1, message = "fmid 不能为空"))]
    pub fmid: String,
    #[serde(default = "default_fmtype")]
    #[allow(dead_code)]
    pub r#type: i64,
    #[serde(default = "default_offset")]
    #[allow(dead_code)]
    pub offset: i64,
    #[serde(default = "default_size")]
    #[allow(dead_code)]
    pub size: i64,
}
fn default_fmtype() -> i64 { 2 }
fn default_offset() -> i64 { -1 }
fn default_size() -> i64 { 20 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct FmImageQuery {
    #[validate(length(min = 1, message = "fmid 不能为空"))]
    pub fmid: String,
}

/// `GET /fm/recommend` —— 推荐电台。
#[utoipa::path(get, path = "/fm/recommend", tag = "fm", responses((status = 200, body = Object)))]
async fn fm_recommend(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(_q): Query<FmEmptyQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::fm::fm_recommend(&state, &session).await?))
}

/// `GET /fm/songs` —— 电台歌曲。
#[utoipa::path(get, path = "/fm/songs", tag = "fm", params(("fmid" = String, Query)), responses((status = 200, body = Object)))]
async fn fm_songs(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<FmSongsQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::fm::fm_songs(&state, &session, &q.fmid, q.r#type, q.offset, q.size).await?))
}

/// `GET /fm/class` —— 电台分类。
#[utoipa::path(get, path = "/fm/class", tag = "fm", responses((status = 200, body = Object)))]
async fn fm_class(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(_q): Query<FmEmptyQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::fm::fm_class(&state, &session).await?))
}

/// `GET /fm/image` —— 电台图片。
#[utoipa::path(get, path = "/fm/image", tag = "fm", params(("fmid" = String, Query)), responses((status = 200, body = Object)))]
async fn fm_image(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<FmImageQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::fm::fm_image(&state, &session, &q.fmid).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(fm_recommend))
        .routes(routes!(fm_songs))
        .routes(routes!(fm_class))
        .routes(routes!(fm_image))
}
