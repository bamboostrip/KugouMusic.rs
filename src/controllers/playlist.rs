//! 歌单写控制器 —— 对应 .NET `PlayListController` 的写类端点（前缀 `playlist`）。

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::error::AppResult;
use crate::middleware::{KgReqSession, KgSessionKey};
use crate::services::{self, helpers, playlist::AddSongItem};
use crate::state::AppState;

fn flex_string_query<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    Ok(match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => String::new(),
    })
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PlaylistAddQuery {
    #[validate(length(min = 1, message = "name 不能为空"))]
    #[allow(dead_code)]
    pub name: String,
    #[serde(alias = "listCreateGid", alias = "global_collection_id", alias = "list_create_gid")]
    #[validate(length(min = 1, message = "list_create_gid 不能为空"))]
    #[allow(dead_code)]
    pub list_create_gid: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PlaylistCreateQuery {
    #[validate(length(min = 1, message = "name 不能为空"))]
    #[allow(dead_code)]
    pub name: String,
    #[serde(default = "default_type", alias = "is_pri", alias = "isPri")]
    #[allow(dead_code)]
    pub r#type: i64,
}
fn default_type() -> i64 { 0 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PlaylistDelQuery {
    #[serde(alias = "listId", alias = "list_id")]
    #[validate(length(min = 1, message = "listid 不能为空"))]
    #[allow(dead_code)]
    pub listid: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AddTracksRequest {
    #[serde(alias = "listId", alias = "listid", alias = "ListId", alias = "list_id", deserialize_with = "flex_string_query", default)]
    #[validate(length(min = 1, message = "ListId 不能为空"))]
    #[allow(dead_code)]
    pub list_id: String,
    #[validate(length(min = 1, message = "Songs 不能为空"))]
    #[allow(dead_code)]
    pub songs: Vec<AddSongItem>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RemoveTracksQuery {
    #[serde(alias = "listId", alias = "list_id")]
    #[validate(length(min = 1, message = "listid 不能为空"))]
    #[allow(dead_code)]
    pub listid: String,
    #[serde(alias = "fileIds", alias = "file_ids", alias = "fileIdsStr")]
    #[validate(length(min = 1, message = "fileids 不能为空"))]
    #[allow(dead_code)]
    pub fileids: String,
}

/// `POST /playlist/add` —— 收藏歌单。
#[utoipa::path(post, path = "/playlist/add", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_add(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<PlaylistAddQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    let name = q.name.clone();
    let gid = q.list_create_gid.clone();
    let v = helpers::with_auto_retry(&state, &k, &s, "/playlist/add", |sess| {
        let state = state.clone();
        let name = name.clone();
        let gid = gid.clone();
        async move { services::playlist::collect_playlist(&state, &sess, &name, &gid).await }
    }).await?;
    Ok(Json(v))
}

/// `POST /playlist/create` —— 新建歌单。
#[utoipa::path(post, path = "/playlist/create", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_create(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<PlaylistCreateQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    let name = q.name.clone();
    let t = q.r#type;
    let v = helpers::with_auto_retry(&state, &k, &s, "/playlist/create", |sess| {
        let state = state.clone();
        let name = name.clone();
        async move { services::playlist::create_playlist(&state, &sess, &name, t).await }
    }).await?;
    Ok(Json(v))
}

/// `POST /playlist/del` —— 删除歌单。
#[utoipa::path(post, path = "/playlist/del", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_del(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<PlaylistDelQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    let listid = q.listid.clone();
    let v = helpers::with_auto_retry(&state, &k, &s, "/playlist/del", |sess| {
        let state = state.clone();
        let listid = listid.clone();
        async move { services::playlist::delete_playlist(&state, &sess, &listid).await }
    }).await?;
    Ok(Json(v))
}

/// `POST /playlist/tracks/add` —— 添加歌曲到歌单。
#[utoipa::path(post, path = "/playlist/tracks/add", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_tracks_add(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Json(req): Json<AddTracksRequest>) -> AppResult<Json<Value>> {
    req.validate()?;
    let list_id = req.list_id.clone();
    let songs = req.songs.clone();
    let v = helpers::with_auto_retry(&state, &k, &s, "/playlist/tracks/add", |sess| {
        let state = state.clone();
        let list_id = list_id.clone();
        let songs = songs.clone();
        async move { services::playlist::add_tracks(&state, &sess, &list_id, &songs).await }
    }).await?;
    Ok(Json(v))
}

/// `POST /playlist/tracks/del` —— 从歌单删除歌曲。
#[utoipa::path(post, path = "/playlist/tracks/del", tag = "playlist", responses((status = 200, body = Object)))]
async fn playlist_tracks_del(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<RemoveTracksQuery>) -> AppResult<Json<Value>> {
    q.validate()?;
    let listid = q.listid.clone();
    let fileids_raw = q.fileids.clone();
    let file_ids: Vec<i64> = fileids_raw.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    let v = helpers::with_auto_retry(&state, &k, &s, "/playlist/tracks/del", |sess| {
        let state = state.clone();
        let listid = listid.clone();
        let file_ids = file_ids.clone();
        async move { services::playlist::remove_tracks(&state, &sess, &listid, &file_ids).await }
    }).await?;
    Ok(Json(v))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(playlist_add))
        .routes(routes!(playlist_create))
        .routes(routes!(playlist_del))
        .routes(routes!(playlist_tracks_add))
        .routes(routes!(playlist_tracks_del))
}


