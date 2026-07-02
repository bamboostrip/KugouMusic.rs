//! 歌单读类控制器 —— 对应 .NET `PlayListController` 的读类端点（前缀 playlist/sheet）。

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
pub struct PositionQuery {
    #[serde(default = "default_position")]
    #[allow(dead_code)]
    pub position: i64,
}
fn default_position() -> i64 { 2 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CollectionDetailQuery {
    #[serde(rename = "collection_id")]
    #[validate(length(min = 1, message = "collection_id 不能为空"))]
    #[allow(dead_code)]
    pub collection_id: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
}
fn default_page() -> i64 { 1 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct IdSourceQuery {
    #[validate(length(min = 1, message = "id 不能为空"))]
    #[allow(dead_code)]
    pub id: String,
    #[validate(length(min = 1, message = "source 不能为空"))]
    #[allow(dead_code)]
    pub source: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OpernTypeQuery {
    #[serde(default = "default_opern", rename = "opern_type")]
    #[allow(dead_code)]
    pub opern_type: i64,
}
fn default_opern() -> i64 { 1 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SheetListQuery {
    #[serde(rename = "album_audio_id")]
    #[validate(length(min = 1, message = "album_audio_id 不能为空"))]
    #[allow(dead_code)]
    pub album_audio_id: String,
    #[serde(default, rename = "opern_type")]
    #[allow(dead_code)]
    pub opern_type: i64,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}
fn default_pagesize() -> i64 { 30 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PlaylistIdQuery {
    #[serde(rename = "ids")]
    #[validate(length(min = 1, message = "ids 不能为空"))]
    #[allow(dead_code)]
    pub ids: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ListIdQuery {
    #[serde(default, rename = "listid")]
    #[allow(dead_code)]
    pub listid: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
    #[serde(rename = "id")]
    #[allow(dead_code)]
    pub id: Option<String>,
}

/// `GET /sheet/collection` —— 歌单合集列表。
#[utoipa::path(get, path = "/sheet/collection", tag = "playlist", responses((status = 200, body = Object)))]
async fn sheet_collection(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PositionQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::playlist::sheet_collection(&state, &s, q.position).await?))
}

/// `GET /sheet/collection/detail` —— 歌单合集详情。
#[utoipa::path(get, path = "/sheet/collection/detail", tag = "playlist", responses((status = 200, body = Object)))]
async fn sheet_collection_detail(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<CollectionDetailQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::playlist::sheet_collection_detail(&state, &s, &q.collection_id, q.page).await?))
}

/// `GET /sheet/detail` —— 歌单详情（按 id + source）。
#[utoipa::path(get, path = "/sheet/detail", tag = "playlist", responses((status = 200, body = Object)))]
async fn sheet_detail(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IdSourceQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::playlist::sheet_detail(&state, &s, &q.id, &q.source).await?))
}

/// `GET /sheet/hot` —— 热门歌单。
#[utoipa::path(get, path = "/sheet/hot", tag = "playlist", responses((status = 200, body = Object)))]
async fn sheet_hot(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<OpernTypeQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::playlist::sheet_hot(&state, &s, q.opern_type).await?))
}

/// `GET /sheet/list` —— 歌单列表（按歌曲推荐）。
#[utoipa::path(get, path = "/sheet/list", tag = "playlist", responses((status = 200, body = Object)))]
async fn sheet_list(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<SheetListQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::playlist::sheet_list(&state, &s, &q.album_audio_id, q.opern_type, q.page, q.pagesize).await?))
}

/// `GET /playlist/detail` —— 歌单详情。
#[utoipa::path(get, path = "/playlist/detail", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_detail(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PlaylistIdQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::playlist::playlist_info(&state, &s, &q.ids).await?))
}

/// `GET /playlist/tags` —— 歌单标签列表。
#[utoipa::path(get, path = "/playlist/tags", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_tags(State(state): State<AppState>, KgReqSession(s): KgReqSession) -> AppResult<Json<Value>> {
    Ok(Json(services::playlist::playlist_tags(&state, &s).await?))
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PlaylistTrackQuery {
    #[validate(length(min = 1, message = "id 不能为空"))]
    pub id: String,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    pub pagesize: i64,
}

/// `GET /playlist/track/all` —— 歌单全部歌曲。
#[utoipa::path(
    get, path = "/playlist/track/all", tag = "playlist",
    params(("id" = String, Query), ("page" = Option<i64>, Query), ("pagesize" = Option<i64>, Query)),
    responses((status = 200, body = Object))
)]
async fn playlist_track_all(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
    Query(q): Query<PlaylistTrackQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    let begin_idx = (q.page - 1) * q.pagesize;
    Ok(Json(services::playlist::playlist_tracks(&state, &s, &q.id, begin_idx, q.pagesize).await?))
}

/// `GET /playlist/track/all/new` —— 歌单全部歌曲（新版接口）。
#[utoipa::path(get, path = "/playlist/track/all/new", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_track_all_new(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<ListIdQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::playlist::playlist_tracks_new(&state, &s, &q.listid, q.page, q.pagesize).await?))
}

/// `GET /playlist/similar` —— 相似歌单。
#[utoipa::path(get, path = "/playlist/similar", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_similar(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PlaylistIdQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::playlist::playlist_similar(&state, &s, &q.ids).await?))
}

/// `GET /playlist/effect` —— 音效歌单。
#[utoipa::path(get, path = "/playlist/effect", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_effect(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<ListIdQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::playlist::playlist_effect(&state, &s, q.page, q.pagesize).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(sheet_collection))
        .routes(routes!(sheet_collection_detail))
        .routes(routes!(sheet_detail))
        .routes(routes!(sheet_hot))
        .routes(routes!(sheet_list))
        .routes(routes!(playlist_detail))
        .routes(routes!(playlist_tags))
        .routes(routes!(playlist_track_all))
        .routes(routes!(playlist_track_all_new))
        .routes(routes!(playlist_similar))
        .routes(routes!(playlist_effect))
}
