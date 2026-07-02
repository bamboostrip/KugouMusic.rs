//! 媒体目录控制器 —— 对应 .NET `MediaCatalogController`（无前缀）。25 端点。

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

fn default_page() -> i64 { 1 }
fn default_pagesize() -> i64 { 30 }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct IdsQuery { #[validate(length(min = 1, message = "id 不能为空"))] #[allow(dead_code)] pub id: String }

#[derive(Debug, Deserialize, ToSchema)]
pub struct PageQuery {
    #[serde(default = "default_page")] #[allow(dead_code)] pub page: i64,
    #[serde(default = "default_pagesize")] #[allow(dead_code)] pub pagesize: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct IpResQuery {
    #[allow(dead_code)] pub id: String,
    #[serde(default = "default_type")] #[allow(dead_code)] pub r#type: String,
    #[serde(default = "default_page")] #[allow(dead_code)] pub page: i64,
    #[serde(default = "default_pagesize")] #[allow(dead_code)] pub pagesize: i64,
}
fn default_type() -> String { "audios".into() }

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SceneAudioQuery {
    #[allow(dead_code)] pub id: String,
    #[serde(default, rename = "module_id")] #[allow(dead_code)] pub module_id: Option<String>,
    #[serde(default)] #[allow(dead_code)] pub tag: Option<String>,
    #[serde(default = "default_page")] #[allow(dead_code)] pub page: i64,
    #[serde(default = "default_pagesize", rename = "pagesize")] #[allow(dead_code)] pub pagesize: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SceneCollectionQuery {
    #[allow(dead_code)] #[serde(rename = "tag_id")] pub tag_id: String,
    #[serde(default = "default_page")] #[allow(dead_code)] pub page: i64,
    #[serde(default = "default_pagesize", rename = "pagesize")] #[allow(dead_code)] pub pagesize: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SceneListV2Query {
    #[allow(dead_code)] pub id: String,
    #[serde(default = "default_page")] #[allow(dead_code)] pub page: i64,
    #[serde(default = "default_pagesize")] #[allow(dead_code)] pub pagesize: i64,
    #[serde(default = "default_sort")] #[allow(dead_code)] pub sort: String,
}
fn default_sort() -> String { "rec".into() }

#[derive(Debug, Deserialize, ToSchema)]
pub struct ModuleQuery { #[allow(dead_code)] pub id: String, #[allow(dead_code)] #[serde(rename = "module_id")] pub module_id: String }

/// 无参端点生成宏：自动生成 `#[utoipa::path]` + doc 注释（含方法、路径、描述）。
macro_rules! noarg_ep {
    ($name:ident, $method:ident, $path:expr, $svc:path, $desc:expr) => {
        #[doc = concat!("`", stringify!($method), " ", $path, "` —— ", $desc)]
        #[utoipa::path($method, path = $path, tag = "media", responses((status = 200, body = Object)))]
        async fn $name(State(state): State<AppState>, KgReqSession(s): KgReqSession) -> AppResult<Json<Value>> {
            Ok(Json($svc(&state, &s).await?))
        }
    };
}

// ===== Video =====
/// `GET /video/detail` —— 视频详情。
#[utoipa::path(get, path = "/video/detail", tag = "media", responses((status = 200, body = Object)))]
async fn video_detail(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IdsQuery>) -> AppResult<Json<Value>> {
    q.validate()?; Ok(Json(services::media_catalog::video_detail(&state, &s, &q.id).await?))
}
/// `GET /video/url` —— 视频播放地址。
#[utoipa::path(get, path = "/video/url", tag = "media", responses((status = 200, body = Object)))]
async fn video_url(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IdsQuery>) -> AppResult<Json<Value>> {
    q.validate()?; Ok(Json(services::media_catalog::video_detail(&state, &s, &q.id).await?))
}

// ===== LongAudio =====
/// `GET /longaudio/album/detail` —— 长音频专辑详情。
#[utoipa::path(get, path = "/longaudio/album/detail", tag = "media", responses((status = 200, body = Object)))]
async fn longaudio_album_detail(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IdsQuery>) -> AppResult<Json<Value>> {
    q.validate()?; Ok(Json(services::media_catalog::longaudio_album_detail(&state, &s, &q.id).await?))
}
/// `GET /longaudio/album/audios` —— 长音频专辑声音列表（分页）。
#[utoipa::path(get, path = "/longaudio/album/audios", tag = "media", responses((status = 200, body = Object)))]
async fn longaudio_album_audios(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<SceneListV2Query>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::longaudio_album_audios(&state, &s, &q.id, q.page, q.pagesize).await?))
}
/// `GET /longaudio/daily/recommend` —— 长音频每日推荐。
#[utoipa::path(get, path = "/longaudio/daily/recommend", tag = "media", responses((status = 200, body = Object)))]
async fn longaudio_daily_recommend(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::longaudio_daily_recommend(&state, &s, q.page, q.pagesize).await?))
}
noarg_ep!(longaudio_rank_recommend, get, "/longaudio/rank/recommend", services::media_catalog::longaudio_rank_recommend, "长音频榜单推荐。");
noarg_ep!(longaudio_vip_recommend, get, "/longaudio/vip/recommend", services::media_catalog::longaudio_vip_recommend, "长音频 VIP 推荐。");
noarg_ep!(longaudio_week_recommend, get, "/longaudio/week/recommend", services::media_catalog::longaudio_week_recommend, "长音频每周推荐。");

// ===== IP =====
/// `GET /ip` —— IP 资源（按 type 区分 audios/playlist 等）。
#[utoipa::path(get, path = "/ip", tag = "media", responses((status = 200, body = Object)))]
async fn ip(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IpResQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::ip_resources(&state, &s, &q.id, &q.r#type, q.page, q.pagesize).await?))
}
/// `GET /ip/detail` —— IP 详情。
#[utoipa::path(get, path = "/ip/detail", tag = "media", responses((status = 200, body = Object)))]
async fn ip_detail(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IdsQuery>) -> AppResult<Json<Value>> {
    q.validate()?; Ok(Json(services::media_catalog::ip_detail(&state, &s, &q.id).await?))
}
/// `GET /ip/playlist` —— IP 关联歌单。
#[utoipa::path(get, path = "/ip/playlist", tag = "media", responses((status = 200, body = Object)))]
async fn ip_playlist(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<SceneListV2Query>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::ip_playlist(&state, &s, &q.id, q.page, q.pagesize).await?))
}
noarg_ep!(ip_zone, get, "/ip/zone", services::media_catalog::ip_zone, "IP 圈子列表。");
/// `GET /ip/zone/home` —— IP 圈子首页。
#[utoipa::path(get, path = "/ip/zone/home", tag = "media", responses((status = 200, body = Object)))]
async fn ip_zone_home(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IdsQuery>) -> AppResult<Json<Value>> {
    q.validate()?; Ok(Json(services::media_catalog::ip_zone_home(&state, &s, &q.id).await?))
}

// ===== Scene =====
noarg_ep!(scene_lists, get, "/scene/lists", services::media_catalog::scene_lists, "场景列表。");
/// `GET /scene/audio/list` —— 场景音频列表。
#[utoipa::path(get, path = "/scene/audio/list", tag = "media", responses((status = 200, body = Object)))]
async fn scene_audio_list(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<SceneAudioQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::scene_audios(&state, &s, &q.id, &q.module_id.clone().unwrap_or_default(), &q.tag.clone().unwrap_or_default(), q.page, q.pagesize).await?))
}
/// `GET /scene/collection/list` —— 场景合集列表。
#[utoipa::path(get, path = "/scene/collection/list", tag = "media", responses((status = 200, body = Object)))]
async fn scene_collection_list(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<SceneCollectionQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::scene_collections(&state, &s, &q.tag_id, q.page, q.pagesize).await?))
}
/// `GET /scene/lists/v2` —— 场景列表 V2（带排序）。
#[utoipa::path(get, path = "/scene/lists/v2", tag = "media", responses((status = 200, body = Object)))]
async fn scene_lists_v2(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<SceneListV2Query>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::scene_lists_v2(&state, &s, &q.id, q.page, q.pagesize, &q.sort).await?))
}
/// `GET /scene/module` —— 场景模块。
#[utoipa::path(get, path = "/scene/module", tag = "media", responses((status = 200, body = Object)))]
async fn scene_module(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IdsQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::scene_module(&state, &s, &q.id).await?))
}
/// `GET /scene/module/info` —— 场景模块详情。
#[utoipa::path(get, path = "/scene/module/info", tag = "media", responses((status = 200, body = Object)))]
async fn scene_module_info(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<ModuleQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::scene_module_info(&state, &s, &q.id, &q.module_id).await?))
}
/// `GET /scene/music` —— 场景音乐列表。
#[utoipa::path(get, path = "/scene/music", tag = "media", responses((status = 200, body = Object)))]
async fn scene_music(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<SceneListV2Query>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::scene_music(&state, &s, &q.id, q.page, q.pagesize).await?))
}
/// `GET /scene/video/list` —— 场景视频列表。
#[utoipa::path(get, path = "/scene/video/list", tag = "media", responses((status = 200, body = Object)))]
async fn scene_video_list(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<SceneCollectionQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::scene_videos(&state, &s, &q.tag_id, q.page, q.pagesize).await?))
}

// ===== Theme =====
/// `GET /theme/music` —— 主题音乐列表。
#[utoipa::path(get, path = "/theme/music", tag = "media", responses((status = 200, body = Object)))]
async fn theme_music(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IdsQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::theme_music(&state, &s, &q.id).await?))
}
noarg_ep!(theme_playlist, get, "/theme/playlist", services::media_catalog::theme_playlists, "主题歌单列表。");
/// `GET /theme/music/detail` —— 主题音乐详情。
#[utoipa::path(get, path = "/theme/music/detail", tag = "media", responses((status = 200, body = Object)))]
async fn theme_music_detail(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IdsQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::theme_music_detail(&state, &s, &q.id).await?))
}
/// `GET /theme/playlist/track` —— 主题歌单歌曲。
#[utoipa::path(get, path = "/theme/playlist/track", tag = "media", responses((status = 200, body = Object)))]
async fn theme_playlist_track(State(state): State<AppState>, KgReqSession(s): KgReqSession, Query(q): Query<IdsQuery>) -> AppResult<Json<Value>> {
    Ok(Json(services::media_catalog::theme_playlist_track(&state, &s, &q.id).await?))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(video_detail))
        .routes(routes!(video_url))
        .routes(routes!(longaudio_album_detail))
        .routes(routes!(longaudio_album_audios))
        .routes(routes!(longaudio_daily_recommend))
        .routes(routes!(longaudio_rank_recommend))
        .routes(routes!(longaudio_vip_recommend))
        .routes(routes!(longaudio_week_recommend))
        .routes(routes!(ip))
        .routes(routes!(ip_detail))
        .routes(routes!(ip_playlist))
        .routes(routes!(ip_zone))
        .routes(routes!(ip_zone_home))
        .routes(routes!(scene_lists))
        .routes(routes!(scene_audio_list))
        .routes(routes!(scene_collection_list))
        .routes(routes!(scene_lists_v2))
        .routes(routes!(scene_module))
        .routes(routes!(scene_module_info))
        .routes(routes!(scene_music))
        .routes(routes!(scene_video_list))
        .routes(routes!(theme_music))
        .routes(routes!(theme_playlist))
        .routes(routes!(theme_music_detail))
        .routes(routes!(theme_playlist_track))
}
