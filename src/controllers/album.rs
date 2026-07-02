//! 专辑控制器 —— 对应 .NET `AlbumController`（前缀 `album`）。

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
pub struct AlbumShopQuery {}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AlbumQuery {
    #[serde(rename = "album_id")]
    #[validate(length(min = 1, message = "album_id 不能为空"))]
    pub album_ids: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub fields: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AlbumDetailQuery {
    #[validate(length(min = 1, message = "id 不能为空"))]
    pub id: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AlbumSongsQuery {
    #[validate(length(min = 1, message = "id 不能为空"))]
    pub id: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}
fn default_page() -> i64 { 1 }
fn default_pagesize() -> i64 { 30 }

/// `GET /album/shop` —— 新专辑上架。
#[utoipa::path(get, path = "/album/shop", tag = "album", responses((status = 200, body = Object)))]
async fn album_shop(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(_q): Query<AlbumShopQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::album::album_shop(&state, &session).await?))
}

/// `GET /album` —— 专辑信息。
#[utoipa::path(get, path = "/album", tag = "album", params(("album_id" = String, Query)), responses((status = 200, body = Object)))]
async fn album_info(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<AlbumQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::album::album_info(&state, &session, &q.album_ids, q.fields.as_deref()).await?))
}

/// `GET /album/detail` —— 专辑详情。
#[utoipa::path(get, path = "/album/detail", tag = "album", params(("id" = String, Query)), responses((status = 200, body = Object)))]
async fn album_detail(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<AlbumDetailQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::album::album_detail(&state, &session, &q.id).await?))
}

/// `GET /album/songs` —— 专辑歌曲。
#[utoipa::path(get, path = "/album/songs", tag = "album", params(("id" = String, Query)), responses((status = 200, body = Object)))]
async fn album_songs(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<AlbumSongsQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::album::album_songs(&state, &session, &q.id, q.page, q.pagesize).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(album_shop))
        .routes(routes!(album_info))
        .routes(routes!(album_detail))
        .routes(routes!(album_songs))
}
