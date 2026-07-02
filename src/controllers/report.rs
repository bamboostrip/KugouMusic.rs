//! 上报控制器 —— 对应 .NET `ReportController`（无前缀，全 POST）。

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
pub struct PlayHistoryQuery {
    pub mxid: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub time: Option<i64>,
    #[serde(default = "default_pc")]
    #[allow(dead_code)]
    pub pc: i64,
}
fn default_pc() -> i64 { 1 }

#[derive(Debug, Deserialize, ToSchema)]
pub struct LatestQuery {
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}
fn default_pagesize() -> i64 { 30 }

/// `POST /playhistory/upload` —— 上报播放历史。
#[utoipa::path(post, path = "/playhistory/upload", tag = "report", responses((status = 200, body = Object)))]
async fn playhistory_upload(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PlayHistoryQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::report::upload_play_history(&state, &s, q.mxid, q.time, q.pc).await?))
}

/// `POST /lastest/songs/listen` —— 最近听过的歌曲。
#[utoipa::path(post, path = "/lastest/songs/listen", tag = "report", responses((status = 200, body = Object)))]
async fn latest_songs_listen(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<LatestQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::report::latest_songs(&state, &s, q.pagesize).await?))
}

/// `POST /listen/timeadd` —— 上报听歌时长。
#[utoipa::path(post, path = "/listen/timeadd", tag = "report", responses((status = 200, body = Object)))]
async fn listen_timeadd(State(state): State<AppState>, KgReqSession(s): KgReqSession) -> AppResult<Json<Value>> {
    Ok(Json(services::report::listen_time_add(&state, &s).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(playhistory_upload))
        .routes(routes!(latest_songs_listen))
        .routes(routes!(listen_timeadd))
}
