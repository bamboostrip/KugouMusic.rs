//! 歌曲控制器 —— 对应 .NET `SongController` 的 song/url 端点（Phase 1 样板）。
//!
//! song/url 是最关键的播放链接端点（V5 签名）。其余歌曲端点随后续 Phase 补。

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::error::AppResult;
use crate::kugou::models::PlayUrlData;
use crate::middleware::KgReqSession;
use crate::services;
use crate::state::AppState;

/// /song/url 入参。对应 .NET SongController.GetUrl。
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SongUrlQuery {
    #[validate(length(min = 1, message = "hash 不能为空"))]
    pub hash: String,
    #[serde(default = "default_quality")]
    pub quality: String,
    #[serde(default)]
    pub album_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub album_audio_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub free_part: Option<bool>,
}
fn default_quality() -> String { "128".into() }

/// `GET /song/url` —— 获取播放地址（V5 签名）。
///
/// 返回透传的上游 JSON（已做 data 提升）。前端可直接用其中的 `url` 字段播放。
#[utoipa::path(
    get,
    path = "/song/url",
    tag = "song",
    params(
        ("hash" = String, Query, description = "歌曲 hash"),
        ("quality" = Option<String>, Query, description = "音质：128/320/flac，或特殊 piano/acappella 等"),
        ("album_id" = Option<String>, Query),
        ("album_audio_id" = Option<String>, Query),
        ("free_part" = Option<bool>, Query),
    ),
    responses(
        (status = 200, description = "播放地址", body = PlayUrlData),
    )
)]
async fn get_song_url(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<SongUrlQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(
        services::song::get_play_url(
            &state,
            &session,
            &q.hash,
            Some(&q.quality),
            q.album_id.as_deref(),
            q.album_audio_id.as_deref(),
            q.free_part.unwrap_or(false),
        )
        .await?,
    ))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_song_url))
}
