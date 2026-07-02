//! 媒体目录业务层 —— 对应 .NET `RawMediaCatalogApi`（聚合 Video/LongAudio/Ip/Scene/Theme 5 个 client）。
//!
//! 25 个端点，全 Default 签名、匿名可用。部分用 body 内嵌 login_key/mid。

use serde_json::{json, Value};

use crate::error::AppResult;
use crate::kugou::{
    config, crypto,
    request::{KgRequest, SignatureType},
    session::KgSession,
    signer,
    transport,
};
use crate::state::AppState;

// ===== Video =====

/// video/detail —— 视频详情（clear_default_params，body 内嵌 key/mid/uuid）。
pub async fn video_detail(state: &AppState, session: &KgSession, ids: &str) -> AppResult<Value> {
    let now = chrono::Utc::now().timestamp();
    let mid = crypto::calc_new_mid(&session.dfid);
    let data: Vec<Value> = ids.split(',').filter(|s| !s.trim().is_empty())
        .map(|id| json!({ "video_id": id.trim() })).collect();
    let body = json!({
        "appid": config::APP_ID, "clientver": config::CLIENT_VER, "clienttime": now,
        "mid": mid, "uuid": crypto::md5_str(&format!("{}{}", session.dfid, mid)),
        "dfid": session.dfid, "token": session.token,
        "key": signer::calc_login_key(now), "show_resolution": 1, "data": data
    });
    let req = KgRequest::get("/v1/video")
        .method(reqwest::Method::POST)
        .clear_default_params()
        .router("kmr.service.kugou.com")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

// ===== LongAudio =====

/// longaudio/album/detail —— 长音频专辑详情（kg-tid:78）。
pub async fn longaudio_album_detail(state: &AppState, session: &KgSession, album_ids: &str) -> AppResult<Value> {
    let data: Vec<Value> = album_ids.split(',').filter(|s| !s.trim().is_empty())
        .map(|id| json!({ "album_id": id.trim() })).collect();
    let body = json!({
        "data": data, "show_album_tag": 1,
        "fields": "album_name,album_id,category,authors,sizable_cover,intro,author_name,trans_param,album_tag,mix_intro,full_intro,is_publish"
    });
    let req = KgRequest::get("/openapi/v2/broadcast")
        .method(reqwest::Method::POST)
        .custom_header("kg-tid", "78")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// longaudio/album/audios —— 长音频专辑章节（kg-tid:78）。
pub async fn longaudio_album_audios(state: &AppState, session: &KgSession, album_id: &str, page: i64, pagesize: i64) -> AppResult<Value> {
    let body = json!({ "album_id": album_id, "area_code": 1, "tagid": 0, "page": page, "pagesize": pagesize });
    let req = KgRequest::get("/longaudio/v2/album_audios")
        .method(reqwest::Method::POST)
        .router("openapi.kugou.com")
        .custom_header("kg-tid", "78")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// longaudio/daily/recommend。
pub async fn longaudio_daily_recommend(state: &AppState, session: &KgSession, page: i64, pagesize: i64) -> AppResult<Value> {
    let req = KgRequest::get("/longaudio/v1/home_new/daily_recommend")
        .method(reqwest::Method::POST)
        .param("module_id", "1").param("size", pagesize.to_string()).param("page", page.to_string())
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// longaudio/rank/recommend。
pub async fn longaudio_rank_recommend(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/longaudio/v1/home_new/rank_card_recommend")
        .param("platform", "ios")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// longaudio/vip/recommend。
pub async fn longaudio_vip_recommend(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/longaudio/v1/home_new/vip_select_recommend")
        .method(reqwest::Method::POST)
        .param("position", "2").param("clientver", "12329")
        .json_body(json!({ "album_playlist": [] }))
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// longaudio/week/recommend。
pub async fn longaudio_week_recommend(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/longaudio/v1/home_new/week_new_albums_recommend")
        .method(reqwest::Method::POST)
        .param("clientver", "12329")
        .json_body(json!({ "album_playlist": [] }))
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

// ===== IP =====

/// ip —— IP 资源列表（type: audios/albums/videos/author_list）。
pub async fn ip_resources(state: &AppState, session: &KgSession, id: &str, ty: &str, page: i64, pagesize: i64) -> AppResult<Value> {
    let normalized = match ty { "audios" | "albums" | "videos" | "author_list" => ty, _ => "audios" };
    let body = json!({ "is_publish": 1, "ip_id": id, "sort": 3, "page": page, "pagesize": pagesize, "query": 1 });
    let req = KgRequest::get(format!("/openapi/v1/ip/{normalized}"))
        .method(reqwest::Method::POST)
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// ip/detail —— IP 详情。
pub async fn ip_detail(state: &AppState, session: &KgSession, ids: &str) -> AppResult<Value> {
    let data: Vec<Value> = ids.split(',').filter(|s| !s.trim().is_empty())
        .map(|id| json!({ "ip_id": id.trim() })).collect();
    let body = json!({ "data": data, "is_publish": 1 });
    let req = KgRequest::get("/openapi/v1/ip")
        .method(reqwest::Method::POST)
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// ip/playlist —— IP 关联歌单。
pub async fn ip_playlist(state: &AppState, session: &KgSession, id: &str, page: i64, pagesize: i64) -> AppResult<Value> {
    let req = KgRequest::get("/ocean/v6/pubsongs/list_info_for_ip")
        .method(reqwest::Method::POST)
        .param("ip", id).param("page", page.to_string()).param("pagesize", pagesize.to_string())
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// ip/zone。
pub async fn ip_zone(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/v1/zone/index")
        .router("yuekucategory.kugou.com")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// ip/zone/home。
pub async fn ip_zone_home(state: &AppState, session: &KgSession, id: &str) -> AppResult<Value> {
    let req = KgRequest::get("/v1/zone/home")
        .router("yuekucategory.kugou.com")
        .param("id", id).param("share", "0")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

// ===== Scene =====

/// scene/lists。
pub async fn scene_lists(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/scene/v1/scene/list")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// scene/audio/list。
pub async fn scene_audios(state: &AppState, session: &KgSession, scene_id: &str, module_id: &str, tag: &str, page: i64, page_size: i64) -> AppResult<Value> {
    let body = json!({ "appid": config::APP_ID, "clientver": config::CLIENT_VER, "token": session.token, "userid": session.userid });
    let req = KgRequest::get("/scene/v1/scene/audio_list")
        .method(reqwest::Method::POST)
        .param("scene_id", scene_id).param("module_id", module_id).param("tag", tag)
        .param("page", page.to_string()).param("page_size", page_size.to_string())
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// scene/collection/list。
pub async fn scene_collections(state: &AppState, session: &KgSession, tag_id: &str, page: i64, page_size: i64) -> AppResult<Value> {
    let body = json!({
        "appid": config::APP_ID, "clientver": config::CLIENT_VER, "token": session.token,
        "userid": session.userid, "tag_id": tag_id, "page": page, "page_size": page_size, "exposed_data": []
    });
    let req = KgRequest::get("/scene/v1/distribution/collection_list")
        .method(reqwest::Method::POST)
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// scene/lists/v2。
pub async fn scene_lists_v2(state: &AppState, session: &KgSession, scene_id: &str, page: i64, pagesize: i64, sort: &str) -> AppResult<Value> {
    let sort_type = match sort { "hot" => "2", "new" => "3", _ => "1" };
    let req = KgRequest::get("/scene/v1/scene/list_v2")
        .method(reqwest::Method::POST)
        .param("scene_id", scene_id).param("page", page.to_string()).param("pagesize", pagesize.to_string())
        .param("sort_type", sort_type).param("kugouid", &session.userid)
        .json_body(json!({ "exposure": [] }))
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// scene/module。
pub async fn scene_module(state: &AppState, session: &KgSession, scene_id: &str) -> AppResult<Value> {
    let req = KgRequest::get("/scene/v1/scene/module")
        .method(reqwest::Method::POST)
        .param("scene_id", scene_id)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// scene/module/info。
pub async fn scene_module_info(state: &AppState, session: &KgSession, scene_id: &str, module_id: &str) -> AppResult<Value> {
    let req = KgRequest::get("/scene/v1/scene/module_info")
        .param("scene_id", scene_id).param("module_id", module_id)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// scene/music。
pub async fn scene_music(state: &AppState, session: &KgSession, scene_id: &str, page: i64, pagesize: i64) -> AppResult<Value> {
    let req = KgRequest::get("/genesisapi/v1/scene_music/rec_music")
        .method(reqwest::Method::POST)
        .param("scene_id", scene_id).param("page", page.to_string()).param("pagesize", pagesize.to_string())
        .json_body(json!({ "exposure": [] }))
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// scene/video/list。
pub async fn scene_videos(state: &AppState, session: &KgSession, tag_id: &str, page: i64, page_size: i64) -> AppResult<Value> {
    let body = json!({
        "appid": config::APP_ID, "clientver": config::CLIENT_VER, "token": session.token,
        "userid": session.userid, "tag_id": tag_id, "page": page, "page_size": page_size, "exposed_data": []
    });
    let req = KgRequest::get("/scene/v1/distribution/video_list")
        .method(reqwest::Method::POST)
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

// ===== Theme =====

/// theme/music。
pub async fn theme_music(state: &AppState, session: &KgSession, ids: &str) -> AppResult<Value> {
    let body = json!({
        "platform": "android", "clienttime": chrono::Utc::now().timestamp(),
        "show_theme_category_ids": ids, "userid": session.userid, "module_id": 508
    });
    let req = KgRequest::get("/everydayrec.service/v1/mul_theme_category_recommend")
        .method(reqwest::Method::POST)
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// theme/playlist。
pub async fn theme_playlists(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let body = json!({
        "platform": "android", "clientver": config::CLIENT_VER,
        "clienttime": chrono::Utc::now().timestamp_millis(),
        "area_code": 1, "module_id": 1, "userid": session.userid
    });
    let req = KgRequest::get("/v2/getthemelist")
        .method(reqwest::Method::POST)
        .router("everydayrec.service.kugou.com")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// theme/music/detail。
pub async fn theme_music_detail(state: &AppState, session: &KgSession, id: &str) -> AppResult<Value> {
    let body = json!({
        "platform": "android", "clienttime": chrono::Utc::now().timestamp(),
        "theme_category_id": id, "show_theme_category_id": 0,
        "userid": session.userid, "module_id": 508
    });
    let req = KgRequest::get("/everydayrec.service/v1/theme_category_recommend")
        .method(reqwest::Method::POST)
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// theme/playlist/track。
pub async fn theme_playlist_track(state: &AppState, session: &KgSession, theme_id: &str) -> AppResult<Value> {
    let body = json!({
        "platform": "android", "clientver": config::CLIENT_VER,
        "clienttime": chrono::Utc::now().timestamp_millis(),
        "area_code": 1, "module_id": 1, "userid": session.userid, "theme_id": theme_id
    });
    let req = KgRequest::get("/v2/gettheme_songidlist")
        .method(reqwest::Method::POST)
        .router("everydayrec.service.kugou.com")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}
