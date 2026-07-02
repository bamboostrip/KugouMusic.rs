//! 歌词控制器 —— 对应 .NET `LyricController`（无前缀，绝对路由）。
//! 端点：GET /search/lyric、GET /lyric。

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
pub struct SearchLyricQuery {
    pub hash: Option<String>,
    #[serde(default, rename = "album_audio_id")]
    #[allow(dead_code)]
    pub album_audio_id: Option<String>,
    pub keywords: Option<String>,
    pub man: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LyricQuery {
    #[validate(length(min = 1, message = "id 不能为空"))]
    pub id: String,
    #[validate(length(min = 1, message = "accesskey 不能为空"))]
    pub accesskey: String,
    #[serde(default = "default_fmt")]
    pub fmt: String,
    #[serde(default = "default_decode")]
    pub decode: bool,
}
fn default_fmt() -> String { "krc".into() }
fn default_decode() -> bool { true }

/// `GET /search/lyric` —— 搜索歌词（拿 id+accesskey）。
#[utoipa::path(
    get, path = "/search/lyric", tag = "lyric",
    params(("hash" = Option<String>, Query), ("album_audio_id" = Option<String>, Query), ("keywords" = Option<String>, Query), ("man" = Option<String>, Query)),
    responses((status = 200, description = "歌词候选列表", body = Object))
)]
async fn search_lyric(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<SearchLyricQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        services::lyric::search_lyric(
            &state, &session, q.hash.as_deref(), q.album_audio_id.as_deref(),
            q.keywords.as_deref(), q.man.as_deref(),
        )
        .await?,
    ))
}

/// `GET /lyric` —— 下载并解码歌词。
#[utoipa::path(
    get, path = "/lyric", tag = "lyric",
    params(("id" = String, Query), ("accesskey" = String, Query), ("fmt" = Option<String>, Query), ("decode" = Option<bool>, Query)),
    responses((status = 200, description = "歌词内容（含解码后的 KRC）", body = Object))
)]
async fn get_lyric(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<LyricQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(
        services::lyric::get_lyric(&state, &session, &q.id, &q.accesskey, &q.fmt, q.decode).await?,
    ))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(search_lyric))
        .routes(routes!(get_lyric))
}
