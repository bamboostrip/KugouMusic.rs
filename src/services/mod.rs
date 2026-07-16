//! 业务层：写具体的酷狗业务逻辑，不接触 HTTP（不解析 axum 请求）。
//! controller 只调用这里的函数，传 AppState + session。
//!
//! 每个函数 = 一个端点的核心：构造 KgRequest → transport → 解包。

pub mod album;
pub mod app_version;
pub mod artist;
pub mod comment;
pub mod discover;
pub mod external_playlist;
pub mod fm;
pub mod login;
pub mod lyric;
pub mod media_catalog;
pub mod playlist;
pub mod rank;
pub mod register;
pub mod report;
pub mod search;
pub mod song;
pub mod user;
pub mod youth;
