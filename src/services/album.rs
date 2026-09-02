//! 专辑业务层 —— 对应 .NET 的 `AlbumClient` + `RawAlbumApi`。4 端点，全 Default 签名。

use serde_json::{json, Value};

use crate::error::AppResult;
use crate::kugou::{config, crypto, request::{KgRequest, SignatureType}, session::KgSession, signer, transport};
use crate::state::AppState;

/// 新专辑上架。GET /zhuanjidata/v3/album_shop_v2/get_classify_data
pub async fn album_shop(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/zhuanjidata/v3/album_shop_v2/get_classify_data")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 专辑信息。POST http://kmr.service.kugou.com/v1/album
///
/// body 内嵌 key(login_key, 毫秒)/mid/d5="-"（与 .NET 一致，用固定 "-"）。
pub async fn album_info(
    state: &AppState,
    session: &KgSession,
    album_ids: &str,
    fields: Option<&str>,
) -> AppResult<Value> {
    let client_time_ms = chrono::Utc::now().timestamp_millis();
    let data: Vec<Value> = album_ids
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|id| json!({ "album_id": id.trim(), "album_name": "", "author_name": "" }))
        .collect();

    let body = json!({
        "appid": config::APP_ID,
        "clienttime": client_time_ms,
        "clientver": config::CLIENT_VER,
        "data": data,
        "dfid": "-",
        "fields": fields.unwrap_or(""),
        "key": signer::calc_login_key(client_time_ms),
        "mid": crypto::calc_new_mid("-"),
    });

    let req = KgRequest::get("/v1/album")
        .method(reqwest::Method::POST)
        .base_url("http://kmr.service.kugou.com")
        .router("kmr.service.kugou.com")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 专辑详情。POST /kmr/v2/albums（kg-tid:255）
pub async fn album_detail(state: &AppState, session: &KgSession, album_id: &str) -> AppResult<Value> {
    let body = json!({
        "data": [{ "album_id": album_id }],
        "is_buy": 0,
        "fields": "album_id,album_name,publish_date,sizable_cover,intro,language,is_publish,heat,type,quality,authors,exclusive,author_name,trans_param"
    });
    let req = KgRequest::get("/kmr/v2/albums")
        .method(reqwest::Method::POST)
        .router("openapi.kugou.com")
        .json_body(body)
        .custom_header("kg-tid", "255")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 专辑歌曲。POST /v1/album_audio/lite（kg-tid:255）
///
/// 上游单页上限由 `ALBUM_MAX_PAGESIZE` 控制（默认 50，酷狗超过返回 invalid param），
/// 对齐 Flutter 侧 `albumSongs` 按页循环拉取全量的前置约束（e621ac3）。
pub async fn album_songs(
    state: &AppState,
    session: &KgSession,
    album_id: &str,
    page: i64,
    pagesize: i64,
) -> AppResult<Value> {
    let max = state.config.album_max_pagesize;
    let orig_pagesize = pagesize;
    let pagesize = pagesize.clamp(1, max);
    let page = if page < 1 { 1 } else { page };
    if orig_pagesize != pagesize {
        tracing::debug!(requested_pagesize = orig_pagesize, clamped = pagesize, max, "专辑歌曲 pagesize 已夹逼至上限");
    }
    let body = json!({
        "album_id": album_id,
        "is_buy": 0,
        "page": page,
        "pagesize": pagesize
    });
    let req = KgRequest::get("/v1/album_audio/lite")
        .method(reqwest::Method::POST)
        .router("openapi.kugou.com")
        .json_body(body)
        .custom_header("kg-tid", "255")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}
