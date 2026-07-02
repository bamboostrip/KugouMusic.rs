//! 歌单写控制器 —— 对应 .NET `PlayListController` 的写类端点（前缀 `playlist`）。

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::error::AppResult;
use crate::middleware::KgReqSession;
use crate::services::{self, playlist::AddSongItem};
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PlaylistAddQuery {
    #[validate(length(min = 1, message = "name 不能为空"))]
    #[allow(dead_code)]
    pub name: String,
    #[validate(length(min = 1, message = "list_create_gid 不能为空"))]
    #[allow(dead_code)]
    pub list_create_gid: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PlaylistCreateQuery {
    #[validate(length(min = 1, message = "name 不能为空"))]
    #[allow(dead_code)]
    pub name: String,
    #[serde(default = "default_type")]
    #[allow(dead_code)]
    pub r#type: i64,
}
fn default_type() -> i64 { 0 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PlaylistDelQuery {
    #[validate(length(min = 1, message = "listid 不能为空"))]
    #[allow(dead_code)]
    pub listid: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AddTracksRequest {
    #[serde(rename = "ListId")]
    #[validate(length(min = 1, message = "ListId 不能为空"))]
    #[allow(dead_code)]
    pub list_id: String,
    #[validate(length(min = 1, message = "Songs 不能为空"))]
    #[allow(dead_code)]
    pub songs: Vec<AddSongItem>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RemoveTracksQuery {
    #[validate(length(min = 1, message = "listid 不能为空"))]
    #[allow(dead_code)]
    pub listid: String,
    #[validate(length(min = 1, message = "fileids 不能为空"))]
    #[allow(dead_code)]
    pub fileids: String,
}

/// `POST /playlist/add` —— 收藏歌单。
#[utoipa::path(post, path = "/playlist/add", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_add(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PlaylistAddQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::playlist::collect_playlist(&state, &s, &q.name, &q.list_create_gid).await?))
}

/// `POST /playlist/create` —— 新建歌单。
#[utoipa::path(post, path = "/playlist/create", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_create(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PlaylistCreateQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::playlist::create_playlist(&state, &s, &q.name, q.r#type).await?))
}

/// `POST /playlist/del` —— 删除歌单。
#[utoipa::path(post, path = "/playlist/del", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_del(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PlaylistDelQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::playlist::delete_playlist(&state, &s, &q.listid).await?))
}

/// `POST /playlist/tracks/add` —— 添加歌曲到歌单。
#[utoipa::path(post, path = "/playlist/tracks/add", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_tracks_add(State(state): State<AppState>, KgReqSession(s): KgReqSession, Json(req): Json<AddTracksRequest>) -> AppResult<Json<Value>> {
    req.validate()?;
    Ok(Json(services::playlist::add_tracks(&state, &s, &req.list_id, &req.songs).await?))
}

/// `POST /playlist/tracks/del` —— 从歌单删除歌曲。
#[utoipa::path(post, path = "/playlist/tracks/del", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_tracks_del(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<RemoveTracksQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    let file_ids: Vec<i64> = q.fileids.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    Ok(Json(services::playlist::remove_tracks(&state, &s, &q.listid, &file_ids).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(playlist_add))
        .routes(routes!(playlist_create))
        .routes(routes!(playlist_del))
        .routes(routes!(playlist_tracks_add))
        .routes(routes!(playlist_tracks_del))
}
