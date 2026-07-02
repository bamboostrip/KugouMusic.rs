//! 搜索业务层 —— 对应 .NET 的 `SearchClient` + `RawSearchApi`。
//!
//! 每个 pub async fn 对应一个搜索端点：构造 [`KgRequest`] → 调 transport → 解包。
//! 透传端点返回 `serde_json::Value`（对应 .NET JsonElement），强类型端点返回 model。

use serde_json::json;
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::kugou::{
    crypto,
    models::{SearchResultData, SongInfo},
    request::{KgRequest, SignatureType},
    session::KgSession,
    transport,
};
use crate::state::AppState;

/// 统一搜索。type=song→强类型 List<SongInfo>；special/album 由各自端点处理；
/// 其它 type 走透传。对应 SearchController.Search + SearchClient.SearchAsync/SearchRawAsync。
///
/// 这里统一返回透传 Value（controller 层对 song 再做类型化），保持简洁。
pub async fn search_raw(
    state: &AppState,
    session: &KgSession,
    keyword: &str,
    page: i64,
    pagesize: i64,
    search_type: &str,
) -> AppResult<Value> {
    let req = KgRequest::get(format!(
        "/{}/search/{}",
        if search_type == "song" { "v3" } else { "v1" },
        search_type
    ))
    .param("keyword", keyword)
    .param("page", page.to_string())
    .param("pagesize", pagesize.to_string())
    .param("platform", "AndroidFilter")
    .param("iscorrection", "1")
    .router("complexsearch.kugou.com")
    .signature_type(SignatureType::Default);

    transport::send(&state.http, session, &req).await
}

/// song 类型搜索 → 类型化歌曲列表。对应 SearchClient.SearchAsync。
pub async fn search_songs(
    state: &AppState,
    session: &KgSession,
    keyword: &str,
    page: i64,
    pagesize: i64,
) -> AppResult<Vec<SongInfo>> {
    let v = search_raw(state, session, keyword, page, pagesize, "song").await?;
    // transport 已做 data 提升，这里直接把 Value 反序列化为 SearchResultData
    let data: SearchResultData = serde_json::from_value(v)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("解析搜索结果失败: {e}")))?;
    Ok(data.songs)
}

/// 热搜。对应 RawSearchApi.SearchHotAsync。
pub async fn search_hot(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/api/v3/search/hot_tab")
        .param("navid", "1")
        .param("plat", "2")
        .router("msearch.kugou.com")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 默认搜索词。对应 RawSearchApi.SearchDefaultAsync（POST + JSON body，软依赖 userid/viptype）。
pub async fn search_default(
    state: &AppState,
    session: &KgSession,
    userid: &str,
    vip_type: &str,
) -> AppResult<Value> {
    let uid: i64 = userid.parse().unwrap_or(0);
    let body = json!({
        "plat": 0,
        "userid": uid,
        "tags": "{}",
        "vip_type": vip_type,
        "m_type": 0,
        "own_ads": {},
        "ability": "3",
        "sources": [],
        "bitmap": 2,
        "mode": "normal"
    });
    let req = KgRequest::get("/searchnofocus/v1/search_no_focus_word")
        .method(reqwest::Method::POST)
        .param("clientver", "12329")
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 搜索建议。对应 RawSearchApi.SearchSuggestAsync。
#[allow(clippy::too_many_arguments)]
pub async fn search_suggest(
    state: &AppState,
    session: &KgSession,
    keyword: &str,
    album_tip: i64,
    correct_tip: i64,
    mv_tip: i64,
    music_tip: i64,
) -> AppResult<Value> {
    let req = KgRequest::get("/v2/getSearchTip")
        .param("keyword", keyword)
        .param("AlbumTipCount", album_tip.to_string())
        .param("CorrectTipCount", correct_tip.to_string())
        .param("MVTipCount", mv_tip.to_string())
        .param("MusicTipCount", music_tip.to_string())
        .param("radiotip", "1")
        .router("searchtip.kugou.com")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 混合搜索。对应 RawSearchApi.SearchMixedAsync。
/// 注意 requestid 用了一个特殊盐的 md5。
pub async fn search_mixed(state: &AppState, session: &KgSession, keyword: &str) -> AppResult<Value> {
    let time_ms = chrono::Utc::now().timestamp_millis();
    let requestid = format!(
        "{}_0",
        crypto::md5_str(&format!(
            "bdaa53d04e7475feb9024164a47032f9{}",
            time_ms
        ))
    );

    let req = KgRequest::get("/v3/search/mixed")
        .param("ab_tag", "0")
        .param("ability", "511")
        .param("albumhide", "0")
        .param("apiver", "22")
        .param("area_code", "1")
        .param("clientver", "20125")
        .param("cursor", "0")
        .param("is_gpay", "0")
        .param("iscorrection", "1")
        .param("keyword", keyword)
        .param("nocollect", "0")
        .param("osversion", "16.5")
        .param("platform", "IOSFilter")
        .param("recver", "2")
        .param("req_ai", "1")
        .param("requestid", requestid)
        .param("search_ability", "3")
        .param("sec_aggre", "1")
        .param("sec_aggre_bitmap", "0")
        .param("style_type", "3")
        .param("tag", "em")
        .router("complexsearch.kugou.com")
        .signature_type(SignatureType::Default);

    let req = req.custom_header("kg-clienttimems", time_ms.to_string());
    transport::send(&state.http, session, &req).await
}

/// 综合搜索。对应 RawSearchApi.SearchComplexAsync（BaseUrl 直连 complexsearch）。
pub async fn search_complex(
    state: &AppState,
    session: &KgSession,
    keyword: &str,
    page: i64,
    pagesize: i64,
) -> AppResult<Value> {
    let req = KgRequest::get("/v6/search/complex")
        .base_url("https://complexsearch.kugou.com")
        .param("platform", "AndroidFilter")
        .param("keyword", keyword)
        .param("page", page.to_string())
        .param("pagesize", pagesize.to_string())
        .param("cursor", "0")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}
