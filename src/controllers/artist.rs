//! 歌手控制器 —— 对应 .NET `ArtistController`（前缀 `artist`）的读类端点。
//! follow/unfollow/newsongs 需 AES+RSA/登录态，留待 Phase 4。

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
pub struct ArtistListsQuery {
    #[serde(default = "default_zero")]
    #[allow(dead_code)]
    pub musician: i64,
    #[serde(default = "default_zero", rename = "sextypes")]
    #[allow(dead_code)]
    pub sextype: i64,
    #[serde(default = "default_zero")]
    #[allow(dead_code)]
    pub r#type: i64,
    #[serde(default = "default_hotsize")]
    #[allow(dead_code)]
    pub hotsize: i64,
}
fn default_zero() -> i64 { 0 }
fn default_hotsize() -> i64 { 30 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct SingerListQuery {
    #[serde(default = "default_zero", rename = "sextype")]
    #[allow(dead_code)]
    pub sextype: i64,
    #[serde(default = "default_zero")]
    #[allow(dead_code)]
    pub r#type: i64,
    #[serde(default = "default_singer_hotsize")]
    #[allow(dead_code)]
    pub hotsize: i64,
}
fn default_singer_hotsize() -> i64 { 200 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ArtistIdQuery {
    #[validate(length(min = 1, message = "id 不能为空"))]
    pub id: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ArtistVideosQuery {
    #[validate(length(min = 1, message = "id 不能为空"))]
    pub id: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
    #[serde(default = "default_tag")]
    #[allow(dead_code)]
    pub tag: String,
}
fn default_page() -> i64 { 1 }
fn default_pagesize() -> i64 { 30 }
fn default_tag() -> String { "all".into() }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ArtistAudiosQuery {
    #[validate(length(min = 1, message = "id 不能为空"))]
    pub id: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
    #[serde(default = "default_sort")]
    #[allow(dead_code)]
    pub sort: String,
}
fn default_sort() -> String { "new".into() }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ArtistHonourQuery {
    #[validate(length(min = 1, message = "id 不能为空"))]
    pub id: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}

/// `GET /artist/lists` —— 歌手列表。
#[utoipa::path(get, path = "/artist/lists", tag = "artist", responses((status = 200, body = Object)))]
async fn artist_lists(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<ArtistListsQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::artist::artist_lists(
        &state, &session, q.musician, q.sextype, q.r#type, q.hotsize,
    ).await?))
}

/// `GET /singer/list` —— 推荐歌手列表。
#[utoipa::path(get, path = "/singer/list", tag = "artist", responses((status = 200, body = Object)))]
async fn singer_list(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<SingerListQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::artist::singer_list(&state, &session, q.sextype, q.r#type, q.hotsize).await?))
}

/// `GET /artist/videos` —— 歌手 MV。
#[utoipa::path(get, path = "/artist/videos", tag = "artist", params(("id" = String, Query)), responses((status = 200, body = Object)))]
async fn artist_videos(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<ArtistVideosQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::artist::artist_videos(&state, &session, &q.id, q.page, q.pagesize, &q.tag).await?))
}

/// `GET /artist/detail` —— 歌手详情。
#[utoipa::path(get, path = "/artist/detail", tag = "artist", params(("id" = String, Query)), responses((status = 200, body = Object)))]
async fn artist_detail(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<ArtistIdQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::artist::artist_detail(&state, &session, &q.id).await?))
}

/// `GET /artist/audios` —— 歌手歌曲。
#[utoipa::path(get, path = "/artist/audios", tag = "artist", params(("id" = String, Query)), responses((status = 200, body = Object)))]
async fn artist_audios(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<ArtistAudiosQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::artist::artist_audios(&state, &session, &q.id, q.page, q.pagesize, &q.sort).await?))
}

/// `GET /artist/albums` —— 歌手专辑。
#[utoipa::path(get, path = "/artist/albums", tag = "artist", params(("id" = String, Query)), responses((status = 200, body = Object)))]
async fn artist_albums(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<ArtistAudiosQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::artist::artist_albums(&state, &session, &q.id, q.page, q.pagesize, &q.sort).await?))
}

/// `GET /artist/honour` —— 歌手荣誉（.NET 是 POST artist/honour）。
#[utoipa::path(get, path = "/artist/honour", tag = "artist", params(("id" = String, Query)), responses((status = 200, body = Object)))]
async fn artist_honour(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<ArtistHonourQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::artist::artist_honour(&state, &session, &q.id, q.page, q.pagesize).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(artist_lists))
        .routes(routes!(singer_list))
        .routes(routes!(artist_videos))
        .routes(routes!(artist_detail))
        .routes(routes!(artist_audios))
        .routes(routes!(artist_albums))
        .routes(routes!(artist_honour))
}
