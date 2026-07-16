//! Youth 控制器 —— 1:1 对应 .NET `YouthController`（路由前缀 `youth`）。
//!
//! 16 个端点的路径、HTTP 方法、查询参数名都与 .NET 完全一致，前端在 .NET / Rust
//! 后端之间切换 URL 即可无感使用。VIP 领取类端点（day/vip、day/vip/upgrade、
//! month/vip/record）沿用 .NET 的 `FromKgStatus` 语义：上游 status 为空或 1 → 200，
//! 否则 → 400（响应体不变）。

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::error::AppResult;
use crate::middleware::KgReqSession;
use crate::services;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PageQuery {
    #[serde(default = "default_one")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_thirty")]
    #[allow(dead_code)]
    pub pagesize: i64,
}
fn default_one() -> i64 { 1 }
fn default_thirty() -> i64 { 30 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct GlobalIdQuery {
    #[serde(rename = "global_collection_id")]
    #[allow(dead_code)]
    pub global_collection_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChannelIdQuery {
    #[serde(rename = "channel_id")]
    #[allow(dead_code)]
    pub channel_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChannelSongDetailQuery {
    #[serde(rename = "global_collection_id")]
    #[allow(dead_code)]
    pub global_collection_id: String,
    #[allow(dead_code)]
    pub fileid: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChannelSubQuery {
    #[serde(rename = "global_collection_id")]
    #[allow(dead_code)]
    pub global_collection_id: String,
    #[serde(default = "default_one")]
    #[allow(dead_code)]
    pub t: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListenSongQuery {
    #[serde(default = "default_mixsongid")]
    #[allow(dead_code)]
    pub mixsongid: i64,
}
fn default_mixsongid() -> i64 { 666075191 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct UserSongQuery {
    #[serde(default)]
    #[allow(dead_code)]
    pub userid: Option<String>,
    #[serde(default = "default_one")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_thirty")]
    #[allow(dead_code)]
    pub pagesize: i64,
    #[serde(default)]
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub list_type: i64,
}

/// 对齐 .NET `FromKgStatus`：上游 status 为空或 ==1 时 200，否则 400（body 不变）。
fn from_kg_status(v: Value) -> (StatusCode, Json<Value>) {
    let status = v.get("status").and_then(|s| s.as_i64());
    let code = if status.is_none() || status == Some(1) {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    (code, Json(v))
}

/// `GET /youth/channel/all` —— 用户所有订阅频道。
#[utoipa::path(get, path = "/youth/channel/all", tag = "youth", params(("page"=Option<i64>,Query),("pagesize"=Option<i64>,Query)), responses((status = 200, body = Object)))]
async fn channel_all(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
    Query(q): Query<PageQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::channel_all(&state, &s, q.page, q.pagesize).await?))
}

/// `GET /youth/channel/amway` —— 频道安利。
#[utoipa::path(get, path = "/youth/channel/amway", tag = "youth", params(("global_collection_id"=String,Query)), responses((status = 200, body = Object)))]
async fn channel_amway(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
    Query(q): Query<GlobalIdQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::channel_amway(&state, &s, &q.global_collection_id).await?))
}

/// `POST /youth/channel/detail` —— 频道详情。
#[utoipa::path(post, path = "/youth/channel/detail", tag = "youth", params(("global_collection_id"=String,Query)), responses((status = 200, body = Object)))]
async fn channel_detail(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
    Query(q): Query<GlobalIdQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::channel_detail(&state, &s, &q.global_collection_id).await?))
}

/// `POST /youth/channel/similar` —— 相似频道。
#[utoipa::path(post, path = "/youth/channel/similar", tag = "youth", params(("channel_id"=String,Query)), responses((status = 200, body = Object)))]
async fn channel_similar(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
    Query(q): Query<ChannelIdQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::channel_similar(&state, &s, &q.channel_id).await?))
}

/// `GET /youth/channel/song` —— 频道音乐故事。
#[utoipa::path(get, path = "/youth/channel/song", tag = "youth", params(("global_collection_id"=String,Query),("page"=Option<i64>,Query),("pagesize"=Option<i64>,Query)), responses((status = 200, body = Object)))]
async fn channel_song(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
    Query(q): Query<GlobalIdSongQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::channel_songs(&state, &s, &q.global_collection_id, q.page, q.pagesize).await?))
}

/// `GET /youth/channel/song/detail` —— 音乐故事详情。
#[utoipa::path(get, path = "/youth/channel/song/detail", tag = "youth", params(("global_collection_id"=String,Query),("fileid"=String,Query)), responses((status = 200, body = Object)))]
async fn channel_song_detail(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
    Query(q): Query<ChannelSongDetailQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::channel_song_detail(&state, &s, &q.global_collection_id, &q.fileid).await?))
}

/// `POST /youth/channel/sub` —— 订阅/取消订阅频道（t=1 订阅，0 取消）。
#[utoipa::path(post, path = "/youth/channel/sub", tag = "youth", params(("global_collection_id"=String,Query),("t"=Option<i64>,Query)), responses((status = 200, body = Object)))]
async fn channel_sub(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
    Query(q): Query<ChannelSubQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::channel_subscription(&state, &s, &q.global_collection_id, q.t != 0).await?))
}

/// `GET /youth/dynamic` —— 动态。
#[utoipa::path(get, path = "/youth/dynamic", tag = "youth", responses((status = 200, body = Object)))]
async fn dynamic(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::dynamic(&state, &s).await?))
}

/// `GET /youth/dynamic/recent` —— 最常访问。
#[utoipa::path(get, path = "/youth/dynamic/recent", tag = "youth", responses((status = 200, body = Object)))]
async fn dynamic_recent(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::dynamic_recent(&state, &s).await?))
}

/// `POST /youth/listen/song` —— 上报听歌。
#[utoipa::path(post, path = "/youth/listen/song", tag = "youth", params(("mixsongid"=Option<i64>,Query)), responses((status = 200, body = Object)))]
async fn listen_song(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
    Query(q): Query<ListenSongQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::report_listen_song(&state, &s, q.mixsongid).await?))
}

/// `GET /youth/union/vip` —— 联合 VIP 信息。
#[utoipa::path(get, path = "/youth/union/vip", tag = "youth", responses((status = 200, body = Object)))]
async fn union_vip(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::union_vip(&state, &s).await?))
}

/// `GET /youth/user/song` —— 用户公开音乐。
#[utoipa::path(get, path = "/youth/user/song", tag = "youth", params(("userid"=Option<String>,Query),("page"=Option<i64>,Query),("pagesize"=Option<i64>,Query),("type"=Option<i64>,Query)), responses((status = 200, body = Object)))]
async fn user_song(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
    Query(q): Query<UserSongQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        services::youth::user_songs(&state, &s, q.userid.as_deref(), q.page, q.pagesize, q.list_type).await?,
    ))
}

/// `POST /youth/vip` —— 领取 VIP（看广告上报）。
#[utoipa::path(post, path = "/youth/vip", tag = "youth", responses((status = 200, body = Object)))]
async fn vip(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
) -> AppResult<Json<Value>> {
    Ok(Json(services::youth::report_vip_ad_play(&state, &s).await?))
}

/// `GET /youth/day/vip` —— 领取当天 VIP（每日一次）。
#[utoipa::path(get, path = "/youth/day/vip", tag = "youth", responses((status = 200, body = Object), (status = 400, body = Object)))]
async fn day_vip(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
) -> AppResult<(StatusCode, Json<Value>)> {
    let v = services::youth::receive_one_day_vip(&state, &s).await?;
    Ok(from_kg_status(v))
}

/// `GET /youth/day/vip/upgrade` —— 升级到概念版 VIP。
#[utoipa::path(get, path = "/youth/day/vip/upgrade", tag = "youth", responses((status = 200, body = Object), (status = 400, body = Object)))]
async fn day_vip_upgrade(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
) -> AppResult<(StatusCode, Json<Value>)> {
    let v = services::youth::upgrade_vip(&state, &s).await?;
    Ok(from_kg_status(v))
}

/// `GET /youth/month/vip/record` —— 当月 VIP 领取记录。
#[utoipa::path(get, path = "/youth/month/vip/record", tag = "youth", responses((status = 200, body = Object), (status = 400, body = Object)))]
async fn month_vip_record(
    State(state): State<AppState>,
    KgReqSession(s): KgReqSession,
) -> AppResult<(StatusCode, Json<Value>)> {
    let v = services::youth::month_vip_record(&state, &s).await?;
    Ok(from_kg_status(v))
}

// channel/song 复用 GlobalId + 分页
#[derive(Debug, Deserialize, ToSchema)]
pub struct GlobalIdSongQuery {
    #[serde(rename = "global_collection_id")]
    #[allow(dead_code)]
    pub global_collection_id: String,
    #[serde(default = "default_one")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_thirty")]
    #[allow(dead_code)]
    pub pagesize: i64,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(channel_all))
        .routes(routes!(channel_amway))
        .routes(routes!(channel_detail))
        .routes(routes!(channel_similar))
        .routes(routes!(channel_song))
        .routes(routes!(channel_song_detail))
        .routes(routes!(channel_sub))
        .routes(routes!(dynamic))
        .routes(routes!(dynamic_recent))
        .routes(routes!(listen_song))
        .routes(routes!(union_vip))
        .routes(routes!(user_song))
        .routes(routes!(vip))
        .routes(routes!(day_vip))
        .routes(routes!(day_vip_upgrade))
        .routes(routes!(month_vip_record))
}
