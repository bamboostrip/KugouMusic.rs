//! Youth（概念版/听歌）业务层 —— 对应 .NET `UserClient` 的 Youth 部分 + `RawUserApi` 的
//! `/youth/*`、`/v1/get_union_vip`。
//!
//! 这一组端点覆盖酷狗"概念版"功能：频道、动态、每日 VIP 领取/升级、联合 VIP 查询等。
//! 与 .NET `YouthController`（路由前缀 `youth`）一一对应，便于前端在 .NET / Rust 后端
//! 之间无感切换。所有端点均为 Default 签名；除 `union_vip` 走 `kugouvip.kugou.com` 外，
//! 其余走默认 gateway。
//!
//! 注意：本模块只做透传代理（与 .NET web API 一致），**不**做"启动期自动领取 VIP"
//! 那种编排——那是 .NET 桌面端 `LoginInitializationService` 的职责，web API 没有这个。

use serde_json::{json, Value};

use crate::error::AppResult;
use crate::kugou::{
    request::{KgRequest, SignatureType},
    session::KgSession,
    transport,
};
use crate::state::AppState;

fn require_login(session: &KgSession) -> AppResult<()> {
    if !session.is_logged_in() {
        return Err(crate::error::AppError::Unauthorized("此接口需要登录".into()));
    }
    Ok(())
}

/// 本地时区的今天（`yyyy-MM-dd`）。对应 .NET `DateTime.Today.ToString("yyyy-MM-dd")`。
fn today_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// youth/channel/all —— 用户所有订阅频道。对应 `RawUserApi.GetYouthChannelAllAsync`。
pub async fn channel_all(state: &AppState, session: &KgSession, page: i64, pagesize: i64) -> AppResult<Value> {
    let req = KgRequest::get("/youth/v2/channel/channel_all_list")
        .param("page", page.to_string())
        .param("pagesize", pagesize.to_string())
        .param("type", "1")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/channel/amway —— 频道安利。对应 `RawUserApi.GetYouthChannelAmwayAsync`。
pub async fn channel_amway(state: &AppState, session: &KgSession, global_collection_id: &str) -> AppResult<Value> {
    let req = KgRequest::get("/youth/api/amway/v2/index")
        .param("global_collection_id", global_collection_id)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/channel/detail —— 频道详情（POST body 带 data 数组）。对应 `GetYouthChannelDetailAsync`。
pub async fn channel_detail(state: &AppState, session: &KgSession, global_collection_ids: &str) -> AppResult<Value> {
    // body = { "data": [ { "global_collection_id": "<id>" }, ... ] }
    let data: Vec<Value> = global_collection_ids
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|id| json!({ "global_collection_id": id }))
        .collect();
    let req = KgRequest::get("/youth/api/channel/v1/channel_list_by_id")
        .method(reqwest::Method::POST)
        .json_body(json!({ "data": data }))
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/channel/similar —— 相似频道（POST body 带 vip_type）。对应 `GetYouthChannelSimilarAsync`。
pub async fn channel_similar(state: &AppState, session: &KgSession, channel_id: &str) -> AppResult<Value> {
    let vip_type: i64 = session.vip_type.parse().unwrap_or(0);
    let req = KgRequest::get("/youth/v1/channel/get_friendly_channel")
        .method(reqwest::Method::POST)
        .param("channel_id", channel_id)
        .json_body(json!({
            "area_code": 1,
            "playlist_ver": 2,
            "vip_type": vip_type,
            "platform": "ios",
        }))
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/channel/song —— 频道音乐故事。对应 `GetYouthChannelSongsAsync`。
pub async fn channel_songs(
    state: &AppState,
    session: &KgSession,
    global_collection_id: &str,
    page: i64,
    pagesize: i64,
) -> AppResult<Value> {
    let req = KgRequest::get("/youth/api/channel/v1/channel_get_song_audit_passed")
        .param("global_collection_id", global_collection_id)
        .param("pagesize", pagesize.to_string())
        .param("page", page.to_string())
        .param("is_filter", "0")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/channel/song/detail —— 音乐故事详情。对应 `GetYouthChannelSongDetailAsync`。
pub async fn channel_song_detail(
    state: &AppState,
    session: &KgSession,
    global_collection_id: &str,
    fileid: &str,
) -> AppResult<Value> {
    let req = KgRequest::get("/youth/v2/post/get_song_detail")
        .param("global_collection_id", global_collection_id)
        .param("fileid", fileid)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/channel/sub —— 订阅/取消订阅频道。对应 `SetYouthChannelSubscriptionAsync`。
/// `subscribe=true` → POST `/youth/v1/channel_subscribe`；否则 DELETE `/youth/v1/channel_unsubscribe`。
pub async fn channel_subscription(
    state: &AppState,
    session: &KgSession,
    global_collection_id: &str,
    subscribe: bool,
) -> AppResult<Value> {
    let path = if subscribe {
        "/youth/v1/channel_subscribe"
    } else {
        "/youth/v1/channel_unsubscribe"
    };
    let mut req = KgRequest::get(path);
    req.method = if subscribe {
        reqwest::Method::POST
    } else {
        reqwest::Method::DELETE
    };
    let req = req
        .param("global_collection_id", global_collection_id)
        .param("source", "1")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/dynamic —— 动态。对应 `GetYouthDynamicAsync`。
pub async fn dynamic(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/youth/v3/user/get_dynamic").signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/dynamic/recent —— 最常访问。对应 `GetYouthRecentDynamicAsync`。
pub async fn dynamic_recent(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/youth/v3/user/recent_dynamic").signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/listen/song —— 上报听歌（自定义 UA + clientver=10566）。对应 `ReportYouthListenSongAsync`。
pub async fn report_listen_song(state: &AppState, session: &KgSession, mixsongid: i64) -> AppResult<Value> {
    let req = KgRequest::get("/youth/v2/report/listen_song")
        .method(reqwest::Method::POST)
        .param("clientver", "10566")
        .json_body(json!({ "mixsongid": mixsongid }))
        .custom_header(
            "user-agent",
            "Android13-1070-10566-201-0-ReportPlaySongToServerProtocol-wifi",
        )
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/union/vip —— 联合 VIP 信息（走 kugouvip 域）。对应 `GetYouthUnionVipAsync`。
pub async fn union_vip(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/v1/get_union_vip")
        .base_url("https://kugouvip.kugou.com")
        .param("busi_type", "concept")
        .param("opt_product_types", "dvip,qvip")
        .param("product_type", "svip")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/user/song —— 用户公开音乐。对应 `GetYouthUserSongsAsync`。
/// `userid` 为 None 时用当前登录用户（与 .NET UserClient 的默认行为一致）。
pub async fn user_songs(
    state: &AppState,
    session: &KgSession,
    userid: Option<&str>,
    page: i64,
    pagesize: i64,
    list_type: i64,
) -> AppResult<Value> {
    let uid = userid.filter(|s| !s.is_empty()).unwrap_or(&session.userid);
    let req = KgRequest::get("/youth/v1/get_user_song_public")
        .param("filter_video", "0")
        .param("type", list_type.to_string())
        .param("userid", uid)
        .param("pagesize", pagesize.to_string())
        .param("page", page.to_string())
        .param("is_filter", "0")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/vip —— 领取 VIP（看广告上报）。对应 `ReportYouthVipAdPlayAsync`。
pub async fn report_vip_ad_play(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let time_ms = chrono::Utc::now().timestamp_millis();
    let req = KgRequest::get("/youth/v1/ad/play_report")
        .method(reqwest::Method::POST)
        .json_body(json!({
            "ad_id": 12307537187_i64,
            "play_end": time_ms,
            "play_start": time_ms - 30000,
        }))
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/day/vip —— 领取当天 VIP（每日一次）。对应 `GetOneDayVipAsync`。
pub async fn receive_one_day_vip(state: &AppState, session: &KgSession) -> AppResult<Value> {
    require_login(session)?;
    let req = KgRequest::get("/youth/v1/recharge/receive_vip_listen_song")
        .method(reqwest::Method::POST)
        .param("source_id", "90139")
        .param("receive_day", today_str())
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/day/vip/upgrade —— 升级到概念版 VIP。对应 `UpgradeVipAsync`。
pub async fn upgrade_vip(state: &AppState, session: &KgSession) -> AppResult<Value> {
    require_login(session)?;
    let req = KgRequest::get("/youth/v1/listen_song/upgrade_vip_reward")
        .method(reqwest::Method::POST)
        .param("kugouid", &session.userid)
        .param("ad_type", "1")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// youth/month/vip/record —— 当月 VIP 领取记录。对应 `GetVipRecordAsync`。
pub async fn month_vip_record(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/youth/v1/activity/get_month_vip_record")
        .param("latest_limit", "100")
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}
