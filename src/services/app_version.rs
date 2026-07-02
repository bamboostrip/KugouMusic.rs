//! App 版本业务层 —— 对应 .NET `ApplicationInfo` controller（唯一走数据库的域）。
//!
//! 从 SQLite `app_versions` 表读取版本信息，不经过酷狗。

use serde::Serialize;
use utoipa::ToSchema;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// App 版本 DTO（与 .NET AppVersionDto 一致）。
#[derive(Debug, Serialize, ToSchema)]
pub struct AppVersionDto {
    pub platform: String,
    pub version_name: String,
    pub version_code: i64,
    pub update_content: String,
    pub download_url: String,
    pub force_update: bool,
    pub release_date: String,
}

/// mobile/app/versions —— 所有版本（可按平台过滤，按 version_code 倒序）。
pub async fn list_versions(state: &AppState, platform: Option<&str>) -> AppResult<Vec<AppVersionDto>> {
    let rows: Result<Vec<VersionRow>, sqlx::Error> = if let Some(p) = platform.filter(|s| !s.is_empty()) {
        sqlx::query_as(
            r#"SELECT platform, version_name, version_code, update_content,
                      download_url, force_update, release_date
               FROM app_versions WHERE platform = ?
               ORDER BY version_code DESC"#,
        )
        .bind(p)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as(
            r#"SELECT platform, version_name, version_code, update_content,
                      download_url, force_update, release_date
               FROM app_versions
               ORDER BY version_code DESC"#,
        )
        .fetch_all(&state.db)
        .await
    };

    match rows {
        Ok(list) => Ok(list.into_iter().map(Into::into).collect()),
        Err(e) => Err(AppError::Database(e)),
    }
}

/// mobile/app/versions/latest —— 指定平台的最新版本。
pub async fn latest_version(state: &AppState, platform: &str) -> AppResult<AppVersionDto> {
    let row: Result<Option<VersionRow>, sqlx::Error> = sqlx::query_as(
        r#"SELECT platform, version_name, version_code, update_content,
                  download_url, force_update, release_date
           FROM app_versions WHERE platform = ?
           ORDER BY version_code DESC LIMIT 1"#,
    )
    .bind(platform)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(r)) => Ok(r.into()),
        Ok(None) => Err(AppError::NotFound(format!("未找到平台 {platform} 的版本信息"))),
        Err(e) => Err(AppError::Database(e)),
    }
}

#[derive(Debug, sqlx::FromRow)]
struct VersionRow {
    platform: String,
    version_name: String,
    version_code: i64,
    update_content: String,
    download_url: String,
    force_update: i64,
    release_date: String,
}

impl From<VersionRow> for AppVersionDto {
    fn from(r: VersionRow) -> Self {
        Self {
            platform: r.platform,
            version_name: r.version_name,
            version_code: r.version_code,
            update_content: r.update_content,
            download_url: r.download_url,
            force_update: r.force_update != 0,
            release_date: r.release_date,
        }
    }
}
