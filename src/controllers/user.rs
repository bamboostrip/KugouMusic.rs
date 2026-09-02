//! 用户控制器 —— 对应 .NET `UserController`（前缀 `user`）。

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::error::AppResult;
use crate::middleware::{KgReqSession, KgSessionKey};
use crate::services::{self, helpers};
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}
fn default_page() -> i64 { 1 }
fn default_pagesize() -> i64 { 30 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListenQuery {
    #[serde(default = "default_listen_type")]
    #[allow(dead_code)]
    pub r#type: i64,
}
fn default_listen_type() -> i64 { 1 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct FollowMessageQuery {
    pub id: String,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CloudUrlQuery {
    pub hash: String,
    #[serde(default, rename = "album_audio_id")]
    #[allow(dead_code)]
    pub album_audio_id: Option<String>,
    #[serde(default, rename = "audio_id")]
    #[allow(dead_code)]
    pub audio_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FavoriteCountQuery {
    pub mixsongids: String,
}

/// `GET /user/detail` —— 用户详情。
#[utoipa::path(get, path = "/user/detail", tag = "user", responses((status = 200, body = Object)))]
async fn user_detail(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey) -> AppResult<Json<Value>> {
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/detail", |sess| {
        let state = state.clone();
        async move { services::user::user_detail(&state, &sess).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /user/vip/detail` —— 用户 VIP 信息。
#[utoipa::path(get, path = "/user/vip/detail", tag = "user", responses((status = 200, body = Object)))]
async fn user_vip_detail(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey) -> AppResult<Json<Value>> {
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/vip/detail", |sess| {
        let state = state.clone();
        async move { services::user::user_vip_detail(&state, &sess).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /user/playlist` —— 用户创建/收藏歌单。
#[utoipa::path(get, path = "/user/playlist", tag = "user", responses((status = 200, body = Object)))]
async fn user_playlist(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    let page = q.page;
    let pagesize = q.pagesize;
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/playlist", |sess| {
        let state = state.clone();
        async move { services::user::user_playlist(&state, &sess, page, pagesize).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /user/history` —— 最近播放历史。
#[utoipa::path(get, path = "/user/history", tag = "user", responses((status = 200, body = Object)))]
async fn user_history(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey) -> AppResult<Json<Value>> {
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/history", |sess| {
        let state = state.clone();
        async move { services::user::user_history(&state, &sess).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /user/listen` —— 听歌统计。
#[utoipa::path(get, path = "/user/listen", tag = "user", responses((status = 200, body = Object)))]
async fn user_listen(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<ListenQuery>) -> AppResult<Json<Value>> {
    let t = q.r#type;
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/listen", |sess| {
        let state = state.clone();
        async move { services::user::user_listen(&state, &sess, t).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /user/follow` —— 关注列表。
#[utoipa::path(get, path = "/user/follow", tag = "user", responses((status = 200, body = Object)))]
async fn user_follow(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey) -> AppResult<Json<Value>> {
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/follow", |sess| {
        let state = state.clone();
        async move { services::user::user_follow(&state, &sess).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /user/cloud` —— 云盘歌曲列表。
#[utoipa::path(get, path = "/user/cloud", tag = "user", responses((status = 200, body = Object)))]
async fn user_cloud(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    let page = q.page;
    let pagesize = q.pagesize;
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/cloud", |sess| {
        let state = state.clone();
        async move { services::user::user_cloud(&state, &sess, page, pagesize).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /user/cloud/url` —— 云盘歌曲播放地址。
#[utoipa::path(get, path = "/user/cloud/url", tag = "user", params(("hash" = String, Query)), responses((status = 200, body = Object)))]
async fn user_cloud_url(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<CloudUrlQuery>) -> AppResult<Json<Value>> {
    let hash = q.hash.clone();
    let album_audio_id = q.album_audio_id.clone();
    let audio_id = q.audio_id.clone();
    let name = q.name.clone();
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/cloud/url", |sess| {
        let state = state.clone();
        let hash = hash.clone();
        let album_audio_id = album_audio_id.clone();
        let audio_id = audio_id.clone();
        let name = name.clone();
        async move { services::user::user_cloud_url(&state, &sess, &hash, album_audio_id.as_deref(), audio_id.as_deref(), name.as_deref()).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /user/follow/message` —— 关注消息（动态）。
#[utoipa::path(get, path = "/user/follow/message", tag = "user", params(("id" = String, Query)), responses((status = 200, body = Object)))]
async fn user_follow_message(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<FollowMessageQuery>) -> AppResult<Json<Value>> {
    let id = q.id.clone();
    let pagesize = q.pagesize;
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/follow/message", |sess| {
        let state = state.clone();
        let id = id.clone();
        async move { services::user::user_follow_message(&state, &sess, &id, pagesize).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /user/video/collect` —— 收藏的视频。
#[utoipa::path(get, path = "/user/video/collect", tag = "user", responses((status = 200, body = Object)))]
async fn user_video_collect(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    let page = q.page;
    let pagesize = q.pagesize;
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/video/collect", |sess| {
        let state = state.clone();
        async move { services::user::user_video_collect(&state, &sess, page, pagesize).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /user/video/love` —— 点赞的视频。
#[utoipa::path(get, path = "/user/video/love", tag = "user", responses((status = 200, body = Object)))]
async fn user_video_love(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    let pagesize = q.pagesize;
    let v = helpers::with_auto_retry(&state, &k, &s, "/user/video/love", |sess| {
        let state = state.clone();
        async move { services::user::user_video_love(&state, &sess, pagesize).await }
    }).await?;
    Ok(Json(v))
}

/// `GET /favorite/count` —— 收藏数量（按 mixsongid 批量）。
#[utoipa::path(get, path = "/favorite/count", tag = "user", params(("mixsongids" = String, Query)), responses((status = 200, body = Object)))]
async fn favorite_count(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey, Query(q): Query<FavoriteCountQuery>) -> AppResult<Json<Value>> {
    let mixsongids = q.mixsongids.clone();
    let v = helpers::with_auto_retry(&state, &k, &s, "/favorite/count", |sess| {
        let state = state.clone();
        let mixsongids = mixsongids.clone();
        async move { services::user::favorite_count(&state, &sess, &mixsongids).await }
    }).await?;
    Ok(Json(v))
}

/// `POST /server/now` —— 服务器当前时间。
#[utoipa::path(post, path = "/server/now", tag = "user", responses((status = 200, body = Object)))]
async fn server_now(State(state): State<AppState>, KgReqSession(s): KgReqSession, KgSessionKey(k): KgSessionKey) -> AppResult<Json<Value>> {
    let v = helpers::with_auto_retry(&state, &k, &s, "/server/now", |sess| {
        let state = state.clone();
        async move { services::user::server_now(&state, &sess).await }
    }).await?;
    Ok(Json(v))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(user_detail))
        .routes(routes!(user_vip_detail))
        .routes(routes!(user_playlist))
        .routes(routes!(user_history))
        .routes(routes!(user_listen))
        .routes(routes!(user_follow))
        .routes(routes!(user_cloud))
        .routes(routes!(user_cloud_url))
        .routes(routes!(user_follow_message))
        .routes(routes!(user_video_collect))
        .routes(routes!(user_video_love))
        .routes(routes!(favorite_count))
        .routes(routes!(server_now))
}


