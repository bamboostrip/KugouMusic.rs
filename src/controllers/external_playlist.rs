//! 外部歌单控制器 —— 对应 .NET `ExternalPlaylistController`（前缀 playlist/external）。

use axum::extract::State;
use axum::Json;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::error::AppResult;
use crate::services::{self, external_playlist::{ExternalPlaylistResult, ParseRequest}};
use crate::state::AppState;

/// POST /playlist/external/parse —— 解析网易云/QQ 音乐歌单链接。
#[utoipa::path(
    post, path = "/playlist/external/parse", tag = "external",
    request_body = ParseRequest,
    responses((status = 200, description = "解析结果", body = ExternalPlaylistResult))
)]
async fn parse(State(state): State<AppState>, Json(req): Json<ParseRequest>) -> AppResult<Json<ExternalPlaylistResult>> {
    Ok(Json(services::external_playlist::parse(&state, &req.source_text).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(parse))
}
