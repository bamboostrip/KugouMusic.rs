//! 排行榜业务层 —— 对应 .NET 的 `RankClient` + `RawRankApi`。4 个读类端点，全 Default 签名。

use serde_json::{json, Value};

use crate::error::AppResult;
use crate::kugou::request::{KgRequest, SignatureType};
use crate::kugou::session::KgSession;
use crate::kugou::transport;
use crate::state::AppState;

/// 排行榜列表。GET /ocean/v6/rank/list
pub async fn rank_list(state: &AppState, session: &KgSession, withsong: i64) -> AppResult<Value> {
    let req = KgRequest::get("/ocean/v6/rank/list")
        .param("plat", "2")
        .param("withsong", withsong.to_string())
        .param("parentid", "0")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 排行榜详情。GET /ocean/v6/rank/info
pub async fn rank_info(
    state: &AppState,
    session: &KgSession,
    rankid: i64,
    rank_cid: i64,
    album_img: i64,
    zone: &str,
) -> AppResult<Value> {
    let req = KgRequest::get("/ocean/v6/rank/info")
        .param("rank_cid", rank_cid.to_string())
        .param("rankid", rankid.to_string())
        .param("with_album_img", album_img.to_string())
        .param("zone", zone)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 排行榜歌曲。POST /openapi/kmr/v2/rank/audio（带 kg-tid:369）
pub async fn rank_audio(
    state: &AppState,
    session: &KgSession,
    rankid: i64,
    rank_cid: i64,
    page: i64,
    pagesize: i64,
) -> AppResult<Value> {
    let body = json!({
        "show_portrait_mv": 1,
        "show_type_total": 1,
        "filter_original_remarks": 1,
        "area_code": 1,
        "pagesize": pagesize,
        "rank_cid": rank_cid,
        "type": 1,
        "page": page,
        "rank_id": rankid
    });
    let req = KgRequest::get("/openapi/kmr/v2/rank/audio")
        .method(reqwest::Method::POST)
        .json_body(body)
        .custom_header("kg-tid", "369")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 推荐排行榜。GET /mobileservice/api/v5/rank/rec_rank_list
pub async fn rank_top(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/mobileservice/api/v5/rank/rec_rank_list")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}
