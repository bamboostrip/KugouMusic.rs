//! 评论控制器 —— 对应 .NET `CommentController`（前缀 `comment`）。
//! 7 端点。这套验证 OfficialAndroid 签名。

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

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MusicCommentQuery {
    #[validate(length(min = 1, message = "mixsongid 不能为空"))]
    pub mixsongid: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}
fn default_page() -> i64 { 1 }
fn default_pagesize() -> i64 { 30 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PlaylistCommentQuery {
    #[validate(length(min = 1, message = "id 不能为空"))]
    pub id: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CommentCountQuery {
    #[serde(default)]
    #[allow(dead_code)]
    pub hash: Option<String>,
    #[serde(default, rename = "special_id")]
    #[allow(dead_code)]
    pub special_id: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct FloorCommentQuery {
    #[serde(default, rename = "special_id")]
    #[allow(dead_code)]
    pub special_id: Option<String>,
    #[validate(length(min = 1, message = "tid 不能为空"))]
    pub tid: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub mixsongid: Option<String>,
    #[serde(default = "default_resource_type", rename = "resource_type")]
    #[allow(dead_code)]
    pub resource_type: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub code: Option<String>,
}
fn default_resource_type() -> String { "song".into() }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ClassifyCommentQuery {
    #[validate(length(min = 1, message = "mixsongid 不能为空"))]
    pub mixsongid: String,
    #[serde(rename = "type_id")]
    #[validate(length(min = 1, message = "type_id 不能为空"))]
    pub type_id: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
    #[serde(default = "default_sort")]
    #[allow(dead_code)]
    pub sort: i64,
}
fn default_sort() -> i64 { 1 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct HotwordCommentQuery {
    #[validate(length(min = 1, message = "mixsongid 不能为空"))]
    pub mixsongid: String,
    #[serde(rename = "hot_word")]
    #[validate(length(min = 1, message = "hot_word 不能为空"))]
    pub hot_word: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}

/// `GET /comment/music` —— 歌曲评论列表。
#[utoipa::path(get, path = "/comment/music", tag = "comment", responses((status = 200, body = Object)))]
async fn comment_music(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<MusicCommentQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::comment::music_comments(&state, &session, &q.mixsongid, q.page, q.pagesize, 1, 1).await?))
}

/// `GET /comment/playlist` —— 歌单评论列表。
#[utoipa::path(get, path = "/comment/playlist", tag = "comment", responses((status = 200, body = Object)))]
async fn comment_playlist(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<PlaylistCommentQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::comment::playlist_comments(&state, &session, &q.id, q.page, q.pagesize, 1, 1).await?))
}

/// `GET /comment/album` —— 专辑评论列表。
#[utoipa::path(get, path = "/comment/album", tag = "comment", responses((status = 200, body = Object)))]
async fn comment_album(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<PlaylistCommentQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::comment::album_comments(&state, &session, &q.id, q.page, q.pagesize, 1, 1).await?))
}

/// `GET /comment/count` —— 评论数量（hash 或 special_id 二选一）。
#[utoipa::path(get, path = "/comment/count", tag = "comment", responses((status = 200, body = Object)))]
async fn comment_count(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<CommentCountQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::comment::comment_count(&state, &session, q.hash.as_deref(), q.special_id.as_deref()).await?))
}

/// `GET /comment/floor` —— 楼层评论（楼中楼）。
#[utoipa::path(get, path = "/comment/floor", tag = "comment", responses((status = 200, body = Object)))]
async fn comment_floor(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<FloorCommentQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::comment::floor_comments(
        &state,
        &session,
        &services::comment::FloorCommentsParams {
            special_id: q.special_id.as_deref(),
            tid: &q.tid,
            mixsongid: q.mixsongid.as_deref(),
            resource_type: &q.resource_type,
            page: q.page,
            pagesize: q.pagesize,
            show_classify: 1,
            show_hotword_list: 1,
            code: q.code.as_deref(),
        },
    )
    .await?))
}

/// `GET /comment/music/classify` —— 歌曲评论分类列表。
#[utoipa::path(get, path = "/comment/music/classify", tag = "comment", responses((status = 200, body = Object)))]
async fn comment_music_classify(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<ClassifyCommentQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::comment::music_comment_classify(
        &state, &session, &q.mixsongid, &q.type_id, q.page, q.pagesize, q.sort,
    ).await?))
}

/// `GET /comment/music/hotword` —— 歌曲评论热词列表。
#[utoipa::path(get, path = "/comment/music/hotword", tag = "comment", responses((status = 200, body = Object)))]
async fn comment_music_hotword(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<HotwordCommentQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::comment::music_comment_hotword(
        &state, &session, &q.mixsongid, &q.hot_word, q.page, q.pagesize,
    ).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(comment_music))
        .routes(routes!(comment_playlist))
        .routes(routes!(comment_album))
        .routes(routes!(comment_count))
        .routes(routes!(comment_floor))
        .routes(routes!(comment_music_classify))
        .routes(routes!(comment_music_hotword))
}
