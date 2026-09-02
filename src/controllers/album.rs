//! 专辑控制器 —— 对应 .NET `AlbumController`（前缀 `album`）。

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::error::AppResult;
use crate::middleware::{KgReqSession, KgSessionKey};
use crate::services::{self, helpers};
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
    // Dart 侧契约原为 id（与 Flutter 侧一致），兼容旧参数名 album_id / albumId
    #[serde(alias = "album_id", alias = "albumId")]
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
    KgSessionKey(k): KgSessionKey,
    Query(_q): Query<AlbumShopQuery>,
) -> AppResult<Json<Value>> {
    let v = helpers::with_auto_retry(&state, &k, &session, "/album/shop", |sess| {
        let state = state.clone();
        async move { services::album::album_shop(&state, &sess).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /album` —— 专辑信息。
#[utoipa::path(get, path = "/album", tag = "album", params(("album_id" = String, Query)), responses((status = 200, body = Object)))]
async fn album_info(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    KgSessionKey(k): KgSessionKey,
    Query(q): Query<AlbumQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    let ids = q.album_ids.clone();
    let fields = q.fields.clone();
    let v = helpers::with_auto_retry(&state, &k, &session, "/album", |sess| {
        let state = state.clone();
        let ids = ids.clone();
        let fields = fields.clone();
        async move { services::album::album_info(&state, &sess, &ids, fields.as_deref()).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /album/detail` —— 专辑详情。
#[utoipa::path(get, path = "/album/detail", tag = "album", params(("id" = String, Query)), responses((status = 200, body = Object)))]
async fn album_detail(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    KgSessionKey(k): KgSessionKey,
    Query(q): Query<AlbumDetailQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    let id = q.id.clone();
    let v = helpers::with_auto_retry(&state, &k, &session, "/album/detail", |sess| {
        let state = state.clone();
        let id = id.clone();
        async move { services::album::album_detail(&state, &sess, &id).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /album/songs` —— 专辑歌曲。
#[utoipa::path(get, path = "/album/songs", tag = "album", params(("id" = String, Query)), responses((status = 200, body = Object)))]
async fn album_songs(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    KgSessionKey(k): KgSessionKey,
    Query(q): Query<AlbumSongsQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    let id = q.id.clone();
    let page = q.page;
    let pagesize = q.pagesize;
    let v = helpers::with_auto_retry(&state, &k, &session, "/album/songs", |sess| {
        let state = state.clone();
        let id = id.clone();
        async move { services::album::album_songs(&state, &sess, &id, page, pagesize).await }
    }).await?;
    Ok(Json(v))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(album_shop))
        .routes(routes!(album_info))
        .routes(routes!(album_detail))
        .routes(routes!(album_songs))
}


