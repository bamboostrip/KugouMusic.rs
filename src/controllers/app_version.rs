//! App 版本控制器 —— 对应 .NET `ApplicationInfo`（前缀 mobile/app）。唯一走数据库的域。

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::error::AppResult;
use crate::services::{self, app_version::AppVersionDto};
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct VersionsQuery {
    #[serde(default)]
    #[allow(dead_code)]
    pub platform: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LatestQuery {
    pub platform: String,
}

/// `GET /mobile/app/versions` —— App 版本列表。
#[utoipa::path(get, path = "/mobile/app/versions", tag = "app", responses((status = 200, body = [AppVersionDto])))]
async fn versions(State(state): State<AppState>, Query(q): Query<VersionsQuery>) -> AppResult<Json<Vec<AppVersionDto>>> {
    Ok(Json(services::app_version::list_versions(&state, q.platform.as_deref()).await?))
}

/// `GET /mobile/app/versions/latest` —— 指定平台的最新版本。
#[utoipa::path(get, path = "/mobile/app/versions/latest", tag = "app", responses((status = 200, body = AppVersionDto)))]
async fn latest(State(state): State<AppState>, Query(q): Query<LatestQuery>) -> AppResult<Json<AppVersionDto>> {
    Ok(Json(services::app_version::latest_version(&state, &q.platform).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(versions))
        .routes(routes!(latest))
}
