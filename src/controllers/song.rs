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
use crate::middleware::{KgReqSession, KgSessionKey};
use crate::services::{self, helpers};
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
///
/// 说明：VIP 态由 transport 层注入的 `Cookie: vip_type/vip_token` 携带到上游
/// （对齐 .NET `WebApiCookieContainerHandler` + `KgSessionManager.SyncCookies`），
/// 因此这里不再需要额外的"播放守卫"。
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
    KgSessionKey(session_key): KgSessionKey,
    Query(q): Query<SongUrlQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    let hash = q.hash.clone();
    let quality = q.quality.clone();
    let album_id = q.album_id.clone();
    let album_audio_id = q.album_audio_id.clone();
    let free_part = q.free_part.unwrap_or(false);
    let v = helpers::with_auto_retry(&state, &session_key, &session, "/song/url", |sess| {
        let state = state.clone();
        let hash = hash.clone();
        let quality = quality.clone();
        let album_id = album_id.clone();
        let album_audio_id = album_audio_id.clone();
        async move {
            services::song::get_play_url(
                &state,
                &sess,
                &hash,
                Some(&quality),
                album_id.as_deref(),
                album_audio_id.as_deref(),
                free_part,
            )
            .await
        }
    })
    .await?;
    Ok(Json(v))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_song_url))
}

