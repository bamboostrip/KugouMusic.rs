//! 发现/推荐控制器 —— 对应 .NET `DiscoveryController`（空前缀）。

use axum::extract::{Query, State};
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
pub struct PlaylistRecQuery {
    #[serde(default, rename = "category_id")]
    #[allow(dead_code)]
    pub category_id: i64,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
}
fn default_page() -> i64 { 1 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewSongQuery {
    #[serde(default = "default_rank")]
    #[allow(dead_code)]
    pub r#type: i64,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
}
fn default_rank() -> i64 { 21608 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct AiRecQuery {
    #[serde(rename = "album_audio_id")]
    pub album_audio_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}
fn default_pagesize() -> i64 { 30 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct CardQuery {
    #[serde(default = "default_card", rename = "card_id")]
    #[allow(dead_code)]
    pub card_id: i64,
}
fn default_card() -> i64 { 1 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct BrushQuery {
    #[serde(default, rename = "song_pool_id")]
    #[allow(dead_code)]
    pub song_pool_id: i64,
    #[serde(default = "default_mode")]
    #[allow(dead_code)]
    pub mode: String,
}
fn default_mode() -> String { "normal".into() }

#[derive(Debug, Deserialize, ToSchema)]
pub struct HistoryQuery {
    #[serde(default = "default_mode")]
    #[allow(dead_code)]
    pub mode: String,
    #[serde(default = "default_platform")]
    #[allow(dead_code)]
    pub platform: String,
    #[serde(default, rename = "history_name")]
    #[allow(dead_code)]
    pub history_name: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub date: Option<String>,
}
fn default_platform() -> String { "ios".into() }

#[derive(Debug, Deserialize, ToSchema)]
pub struct PersonalFmQuery {
    pub hash: Option<String>,
    pub songid: Option<String>,
    pub playtime: Option<i64>,
    #[serde(default = "default_action")]
    #[allow(dead_code)]
    pub action: String,
    #[serde(default = "default_mode")]
    #[allow(dead_code)]
    pub mode: String,
    #[serde(default, rename = "song_pool_id")]
    #[allow(dead_code)]
    pub song_pool_id: i64,
    #[serde(default, rename = "is_overplay")]
    #[allow(dead_code)]
    pub is_overplay: bool,
    #[serde(default, rename = "remain_song_cnt")]
    #[allow(dead_code)]
    pub remain_song_cnt: i64,
}
fn default_action() -> String { "play".into() }

/// `GET /top/playlist` —— 推荐歌单（按分类）。
#[utoipa::path(get, path = "/top/playlist", tag = "discover", responses((status = 200, body = Object)))]
async fn top_playlist(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PlaylistRecQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::recommend_playlists(&state, &s, q.category_id, q.page).await?))
}

/// `GET /top/song` —— 新歌速递。
#[utoipa::path(get, path = "/top/song", tag = "discover", responses((status = 200, body = Object)))]
async fn top_song(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<NewSongQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::new_songs(&state, &s, q.r#type, q.page).await?))
}

/// `GET /recommend/songs` —— 推荐歌曲。
#[utoipa::path(get, path = "/recommend/songs", tag = "discover", responses((status = 200, body = Object)))]
async fn recommend_songs(State(state): State<AppState>, KgReqSession(s): KgReqSession) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::recommend_songs(&state, &s).await?))
}

/// `GET /everyday/style/recommend` —— 每日推荐风格。
#[utoipa::path(get, path = "/everyday/style/recommend", tag = "discover", responses((status = 200, body = Object)))]
async fn recommend_style(State(state): State<AppState>, KgReqSession(s): KgReqSession) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::recommend_style(&state, &s).await?))
}

/// `GET /ai/recommend` —— AI 推荐（基于种子歌曲）。
#[utoipa::path(get, path = "/ai/recommend", tag = "discover", responses((status = 200, body = Object)))]
async fn ai_recommend(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<AiRecQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::ai_recommend(&state, &s, &q.album_audio_id).await?))
}

/// `GET /yueku` —— 乐库首页。
#[utoipa::path(get, path = "/yueku", tag = "discover", responses((status = 200, body = Object)))]
async fn yueku(State(state): State<AppState>, KgReqSession(s): KgReqSession) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::yueku(&state, &s).await?))
}

/// `GET /yueku/banner` —— 乐库 banner。
#[utoipa::path(get, path = "/yueku/banner", tag = "discover", responses((status = 200, body = Object)))]
async fn yueku_banner(State(state): State<AppState>, KgReqSession(s): KgReqSession) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::yueku_banner(&state, &s).await?))
}

/// `GET /yueku/fm` —— 乐库电台。
#[utoipa::path(get, path = "/yueku/fm", tag = "discover", responses((status = 200, body = Object)))]
async fn yueku_fm(State(state): State<AppState>, KgReqSession(s): KgReqSession) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::yueku_fm(&state, &s).await?))
}

/// `GET /top/album` —— 新碟上架（分页）。
#[utoipa::path(get, path = "/top/album", tag = "discover", responses((status = 200, body = Object)))]
async fn top_album(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::top_album(&state, &s, q.page, q.pagesize).await?))
}

/// `GET /top/card` —— 卡片榜单。
#[utoipa::path(get, path = "/top/card", tag = "discover", responses((status = 200, body = Object)))]
async fn top_card(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<CardQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::top_card(&state, &s, q.card_id).await?))
}

/// `GET /top/ip` —— IP 榜单。
#[utoipa::path(get, path = "/top/ip", tag = "discover", responses((status = 200, body = Object)))]
async fn top_ip(State(state): State<AppState>, KgReqSession(s): KgReqSession) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::top_ip(&state, &s).await?))
}

/// `GET /pc/diantai` —— PC 电台。
#[utoipa::path(get, path = "/pc/diantai", tag = "discover", responses((status = 200, body = Object)))]
async fn pc_diantai(State(state): State<AppState>, KgReqSession(s): KgReqSession) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::pc_diantai(&state, &s).await?))
}

/// `GET /brush` —— 听歌打卡（刷歌曲池）。
#[utoipa::path(get, path = "/brush", tag = "discover", responses((status = 200, body = Object)))]
async fn brush(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<BrushQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::brush(&state, &s, q.song_pool_id, &q.mode).await?))
}

/// `POST /everyday/history` —— 每日听歌历史。
#[utoipa::path(post, path = "/everyday/history", tag = "discover", responses((status = 200, body = Object)))]
async fn everyday_history(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<HistoryQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::everyday_history(&state, &s, &q.mode, &q.platform, q.history_name.as_deref(), q.date.as_deref()).await?))
}

/// `GET /personal/fm` —— 私人 FM（基于播放行为反馈）。
#[utoipa::path(get, path = "/personal/fm", tag = "discover", responses((status = 200, body = Object)))]
async fn personal_fm(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PersonalFmQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::discover::personal_fm(&state, &s, q.hash.as_deref(), q.songid.as_deref(), q.playtime, &q.action, &q.mode, q.song_pool_id, q.is_overplay, q.remain_song_cnt).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(top_playlist))
        .routes(routes!(top_song))
        .routes(routes!(recommend_songs))
        .routes(routes!(recommend_style))
        .routes(routes!(ai_recommend))
        .routes(routes!(yueku))
        .routes(routes!(yueku_banner))
        .routes(routes!(yueku_fm))
        .routes(routes!(top_album))
        .routes(routes!(top_card))
        .routes(routes!(top_ip))
        .routes(routes!(pc_diantai))
        .routes(routes!(brush))
        .routes(routes!(everyday_history))
        .routes(routes!(personal_fm))
}
