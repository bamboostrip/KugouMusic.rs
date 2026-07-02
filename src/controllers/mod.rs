//! 视图层：只负责入参校验（validator）和调用 service。
//!
//! 每个子模块暴露：
//! - `router()` —— 返回挂好本模块 handler 的 `OpenApiRouter`（带 state 占位）；
//! - 各 handler —— 用 `#[utoipa::path]` 标注，自动进 Swagger。

pub mod album;
pub mod app_version;
pub mod artist;
pub mod comment;
pub mod discover;
pub mod external_playlist;
pub mod fm;
pub mod health;
pub mod login;
pub mod lyric;
pub mod media_catalog;
pub mod playlist;
pub mod playlist_read;
pub mod rank;
pub mod register;
pub mod report;
pub mod search;
pub mod song;
pub mod user;
