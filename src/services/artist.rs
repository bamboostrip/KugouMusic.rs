//! 歌手业务层 —— 对应 .NET 的 `ArtistClient` + `RawArtistApi`。
//!
//! Phase 2 只迁读类端点（lists/singer/videos/detail/audios/albums/honour）。
//! follow/unfollow/newsongs 需 AES+RSA（Phase 4）/登录态，留待后续。

use serde_json::{json, Value};

use crate::error::AppResult;
use crate::kugou::{config, crypto, request::{KgRequest, SignatureType}, session::KgSession, signer, transport};
use crate::state::AppState;

/// 歌手列表。GET /ocean/v6/singer/list
pub async fn artist_lists(
    state: &AppState,
    session: &KgSession,
    musician: i64,
    sextype: i64,
    r#type: i64,
    hotsize: i64,
) -> AppResult<Value> {
    let req = KgRequest::get("/ocean/v6/singer/list")
        .param("musician", musician.to_string())
        .param("sextype", sextype.to_string())
        .param("showtype", "2")
        .param("type", r#type.to_string())
        .param("hotsize", hotsize.to_string())
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 推荐歌手列表。GET /ocean/v6/singer/list（同上，默认参不同）。
pub async fn singer_list(
    state: &AppState,
    session: &KgSession,
    sextype: i64,
    r#type: i64,
    hotsize: i64,
) -> AppResult<Value> {
    let req = KgRequest::get("/ocean/v6/singer/list")
        .param("sextype", sextype.to_string())
        .param("type", r#type.to_string())
        .param("hotsize", hotsize.to_string())
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 歌手 MV。GET https://openapicdn.kugou.com/kmr/v1/author/videos
pub async fn artist_videos(
    state: &AppState,
    session: &KgSession,
    id: &str,
    page: i64,
    pagesize: i64,
    tag: &str,
) -> AppResult<Value> {
    let tag_idx = match tag {
        "official" => "18",
        "live" => "20",
        "fan" => "23",
        "artist" => "42419",
        _ => "",
    };
    let req = KgRequest::get("/kmr/v1/author/videos")
        .base_url("https://openapicdn.kugou.com")
        .param("author_id", id)
        .param("is_fanmade", "")
        .param("tag_idx", tag_idx)
        .param("pagesize", pagesize.to_string())
        .param("page", page.to_string())
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 歌手详情。POST /kmr/v3/author（kg-tid:36）
pub async fn artist_detail(state: &AppState, session: &KgSession, id: &str) -> AppResult<Value> {
    let body = json!({ "author_id": id });
    let req = KgRequest::get("/kmr/v3/author")
        .method(reqwest::Method::POST)
        .router("openapi.kugou.com")
        .json_body(body)
        .custom_header("kg-tid", "36")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 歌手歌曲。POST https://openapi.kugou.com/kmr/v1/audio_group/author（kg-tid:220，body 内嵌 key/mid）
pub async fn artist_audios(
    state: &AppState,
    session: &KgSession,
    id: &str,
    page: i64,
    pagesize: i64,
    sort: &str,
) -> AppResult<Value> {
    let client_time = chrono::Utc::now().timestamp();
    let body = json!({
        "appid": config::APP_ID,
        "clientver": config::CLIENT_VER,
        "mid": crypto::calc_new_mid(&session.dfid),
        "clienttime": client_time,
        "key": signer::calc_login_key(chrono::Utc::now().timestamp_millis()),
        "author_id": id,
        "pagesize": pagesize,
        "page": page,
        "sort": if sort == "hot" { 1 } else { 2 },
        "area_code": "all"
    });
    let req = KgRequest::get("/kmr/v1/audio_group/author")
        .method(reqwest::Method::POST)
        .base_url("https://openapi.kugou.com")
        .router("openapi.kugou.com")
        .json_body(body)
        .custom_header("kg-tid", "220")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 歌手专辑。POST /kmr/v1/author/albums（kg-tid:36）
pub async fn artist_albums(
    state: &AppState,
    session: &KgSession,
    id: &str,
    page: i64,
    pagesize: i64,
    sort: &str,
) -> AppResult<Value> {
    let body = json!({
        "author_id": id,
        "pagesize": pagesize,
        "page": page,
        "sort": if sort == "hot" { 3 } else { 1 },
        "category": 1,
        "area_code": "all"
    });
    let req = KgRequest::get("/kmr/v1/author/albums")
        .method(reqwest::Method::POST)
        .router("openapi.kugou.com")
        .json_body(body)
        .custom_header("kg-tid", "36")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 歌手荣誉。POST http://h5activity.kugou.com/v1/query_singer_honour_detail
pub async fn artist_honour(
    state: &AppState,
    session: &KgSession,
    id: &str,
    page: i64,
    pagesize: i64,
) -> AppResult<Value> {
    let req = KgRequest::get("/v1/query_singer_honour_detail")
        .method(reqwest::Method::POST)
        .base_url("http://h5activity.kugou.com")
        .param("singer_id", id)
        .param("pagesize", pagesize.to_string())
        .param("page", page.to_string())
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}
