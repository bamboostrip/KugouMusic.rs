//! 排行榜控制器 —— 对应 .NET `RankController`（前缀 `rank`）。

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
pub struct RankListQuery {
    #[serde(default = "default_withsong")]
    #[allow(dead_code)]
    pub withsong: i64,
}
fn default_withsong() -> i64 { 1 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct RankInfoQuery {
    pub rankid: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub rank_cid: Option<i64>,
    #[serde(default = "default_album_img")]
    #[allow(dead_code)]
    pub album_img: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub zone: Option<String>,
}
fn default_album_img() -> i64 { 1 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct RankAudioQuery {
    pub rankid: i64,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub rank_cid: Option<i64>,
}
fn default_page() -> i64 { 1 }
fn default_pagesize() -> i64 { 30 }

/// `GET /rank/list` —— 所有排行榜。
#[utoipa::path(get, path = "/rank/list", tag = "rank", responses((status = 200, body = Object)))]
async fn rank_list(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<RankListQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::rank::rank_list(&state, &session, q.withsong).await?))
}

/// `GET /rank/info` —— 排行榜详情。
#[utoipa::path(get, path = "/rank/info", tag = "rank", responses((status = 200, body = Object)))]
async fn rank_info(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<RankInfoQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::rank::rank_info(
        &state, &session, q.rankid, q.rank_cid.unwrap_or(0), q.album_img, &q.zone.unwrap_or_default(),
    ).await?))
}

/// `GET /rank/audio` —— 排行榜歌曲（分页）。
#[utoipa::path(get, path = "/rank/audio", tag = "rank", responses((status = 200, body = Object)))]
async fn rank_audio(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<RankAudioQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::rank::rank_audio(
        &state, &session, q.rankid, q.rank_cid.unwrap_or(0), q.page, q.pagesize,
    ).await?))
}

/// `GET /rank/top` —— 推荐排行榜。
#[utoipa::path(get, path = "/rank/top", tag = "rank", responses((status = 200, body = Object)))]
async fn rank_top(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
) -> AppResult<Json<Value>> {
    Ok(Json(services::rank::rank_top(&state, &session).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(rank_list))
        .routes(routes!(rank_info))
        .routes(routes!(rank_audio))
        .routes(routes!(rank_top))
}
