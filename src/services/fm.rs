//! 电台业务层 —— 对应 .NET 的 `FmClient` + `RawFmApi`。4 端点，Default 签名。
//!
//! 注意 FM 端点用 .NET 的 `SignedBody` 模式：把 appid/clienttime(毫秒)/clientver/
//! key(login_key)/mid 注入到 **JSON body** 内（而非走 transport 默认参数）。
//! 这里用 [`fm_signed_body`] 复刻。

use serde_json::{json, Value};

use crate::error::AppResult;
use crate::kugou::{config, crypto, request::{KgRequest, SignatureType}, session::KgSession, signer, transport};
use crate::state::AppState;

/// 复刻 .NET RawFmApi.SignedBody：往 body 注入 appid/clienttime/clientver/key/mid。
/// 返回新 body（保留原字段，注入/覆盖这 5 个）。
fn fm_signed_body(session: &KgSession, now_ms: i64, mut body: Value) -> Value {
    if let Value::Object(ref mut map) = body {
        map.insert("appid".into(), json!(config::APP_ID));
        map.insert("clienttime".into(), json!(now_ms));
        map.insert("clientver".into(), json!(config::CLIENT_VER));
        map.insert("key".into(), json!(signer::calc_login_key(now_ms)));
        map.insert("mid".into(), json!(crypto::calc_new_mid(&session.dfid)));
    }
    body
}

/// 推荐电台。POST /v1/rcmd_list（路由 fm.service.kugou.com）
pub async fn fm_recommend(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let body = fm_signed_body(session, now_ms, json!({
        "rcmdsongcount": 1,
        "level": 0,
        "area_code": 1,
        "get_tracker": 1,
        "uid": 0
    }));
    let req = KgRequest::get("/v1/rcmd_list")
        .method(reqwest::Method::POST)
        .router("fm.service.kugou.com")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 电台歌曲。POST /v1/app_song_list_offset
pub async fn fm_songs(
    state: &AppState,
    session: &KgSession,
    fm_ids: &str,
    fmtype: i64,
    offset: i64,
    size: i64,
) -> AppResult<Value> {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let data: Vec<Value> = fm_ids
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|id| {
            json!({
                "fmid": id.trim(),
                "fmtype": fmtype,
                "offset": offset,
                "size": size,
                "singername": ""
            })
        })
        .collect();

    let body = fm_signed_body(session, now_ms, json!({
        "area_code": 1,
        "data": data,
        "get_tracker": 1,
        "uid": session.userid
    }));
    let req = KgRequest::get("/v1/app_song_list_offset")
        .method(reqwest::Method::POST)
        .router("fm.service.kugou.com")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 电台分类。POST /v1/class_fm_song
pub async fn fm_class(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let userid = session.userid.clone();
    let body = fm_signed_body(session, now_ms, json!({
        "kguid": userid,
        "platform": "android",
        "uid": session.userid,
        "get_tracker": 1
    }));
    let req = KgRequest::get("/v1/class_fm_song")
        .method(reqwest::Method::POST)
        .router("fm.service.kugou.com")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 电台图片。POST /v1/fm_info
pub async fn fm_image(state: &AppState, session: &KgSession, fm_ids: &str) -> AppResult<Value> {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let data: Vec<Value> = fm_ids
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|id| {
            json!({
                "fields": "imgUrl100,imgUrl50",
                "fmid": id.trim(),
                "fmtype": 2
            })
        })
        .collect();

    let body = fm_signed_body(session, now_ms, json!({
        "data": data,
        "dfid": session.dfid
    }));
    let req = KgRequest::get("/v1/fm_info")
        .method(reqwest::Method::POST)
        .router("fm.service.kugou.com")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}
