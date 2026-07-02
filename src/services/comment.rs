//! 评论业务层 —— 对应 .NET 的 `CommentClient` + `RawCommentApi`。
//!
//! **这套端点验证第 4 种签名策略 OfficialAndroid**（用 OfficialSalt + 官方 appid/clientver）。
//! .NET 在 params 里显式设置 appid=1005/clientver=20489，transport 的 TryAdd 会尊重它们。
//! Rust transport 的 entry().or_insert() 同样尊重显式值。

use std::collections::BTreeMap;

use serde_json::Value;

use crate::error::AppResult;
use crate::kugou::{config, request::{KgRequest, SignatureType}, session::KgSession, transport};
use crate::state::AppState;

/// 评论业务 code 常量
const CODE_SONG: &str = config::COMMENT_SONG_CODE;
const CODE_PLAYLIST: &str = config::COMMENT_PLAYLIST_CODE;
const CODE_ALBUM: &str = config::COMMENT_ALBUM_CODE;

/// 给 params 注入官方 appid/clientver（对应 .NET UseOfficialApp）。
fn official(mut params: BTreeMap<String, String>) -> BTreeMap<String, String> {
    params.insert("appid".into(), config::OFFICIAL_APP_ID.into());
    params.insert("clientver".into(), config::OFFICIAL_CLIENT_VER.into());
    params
}

/// 把 [(k,v)] 构造成 BTreeMap<String,String>（链式 .param 太长，这里用宏辅助）。
macro_rules! params {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut m: BTreeMap<String, String> = BTreeMap::new();
        $( m.insert(($k).into(), ($v).into()); )*
        m
    }};
}

/// 歌曲评论。POST /mcomment/v1/cmtlist
pub async fn music_comments(
    state: &AppState,
    session: &KgSession,
    mixsongid: &str,
    page: i64,
    pagesize: i64,
    show_classify: i64,
    show_hotword_list: i64,
) -> AppResult<Value> {
    let params = official(params! {
        "mixsongid" => mixsongid,
        "need_show_image" => "1",
        "p" => page.to_string(),
        "pagesize" => pagesize.to_string(),
        "show_classify" => show_classify.to_string(),
        "show_hotword_list" => show_hotword_list.to_string(),
        "extdata" => "0",
        "code" => CODE_SONG,
    });
    send_comment_list(state, session, "/mcomment/v1/cmtlist", params).await
}

/// 歌单评论。POST /m.comment.service/v1/cmtlist
pub async fn playlist_comments(
    state: &AppState,
    session: &KgSession,
    id: &str,
    page: i64,
    pagesize: i64,
    show_classify: i64,
    show_hotword_list: i64,
) -> AppResult<Value> {
    let params = official(params! {
        "childrenid" => id,
        "need_show_image" => "1",
        "p" => page.to_string(),
        "pagesize" => pagesize.to_string(),
        "show_classify" => show_classify.to_string(),
        "show_hotword_list" => show_hotword_list.to_string(),
        "code" => CODE_PLAYLIST,
        "content_type" => "0",
        "tag" => "5",
    });
    send_comment_list(state, session, "/m.comment.service/v1/cmtlist", params).await
}

/// 专辑评论。POST /m.comment.service/v1/cmtlist
pub async fn album_comments(
    state: &AppState,
    session: &KgSession,
    id: &str,
    page: i64,
    pagesize: i64,
    show_classify: i64,
    show_hotword_list: i64,
) -> AppResult<Value> {
    let params = official(params! {
        "childrenid" => id,
        "need_show_image" => "1",
        "p" => page.to_string(),
        "pagesize" => pagesize.to_string(),
        "show_classify" => show_classify.to_string(),
        "show_hotword_list" => show_hotword_list.to_string(),
        "code" => CODE_ALBUM,
    });
    send_comment_list(state, session, "/m.comment.service/v1/cmtlist", params).await
}

/// 评论数量。GET /index.php（SignatureType::Web，路由 sum.comment.service）
///
/// 注意这里用 **Web 签名**（不是 OfficialAndroid），appid/clientver 也是官方值。
pub async fn comment_count(
    state: &AppState,
    session: &KgSession,
    hash: Option<&str>,
    special_id: Option<&str>,
) -> AppResult<Value> {
    let mut params = params! {
        "appid" => config::OFFICIAL_APP_ID,
        "clientver" => config::OFFICIAL_CLIENT_VER,
        "r" => "comments/getcommentsnum",
        "code" => CODE_SONG,
    };
    if let Some(h) = hash.filter(|s| !s.trim().is_empty()) {
        params.insert("hash".into(), h.into());
    } else if let Some(s) = special_id.filter(|s| !s.trim().is_empty()) {
        params.insert("childrenid".into(), s.into());
    }
    let req = KgRequest::get("/index.php")
        .router("sum.comment.service.kugou.com")
        .signature_type(SignatureType::Web);
    let mut req = req;
    for (k, v) in params {
        req = req.param(k, v);
    }
    transport::send(&state.http, session, &req).await
}

/// 楼层评论参数（避免函数参数过多，clippy too_many_arguments）。
pub struct FloorCommentsParams<'a> {
    pub special_id: Option<&'a str>,
    pub tid: &'a str,
    pub mixsongid: Option<&'a str>,
    pub resource_type: &'a str,
    pub page: i64,
    pub pagesize: i64,
    pub show_classify: i64,
    pub show_hotword_list: i64,
    pub code: Option<&'a str>,
}

/// 楼层评论。POST /mcomment/v1/hot_replylist 或 /m.comment.service/v1/hot_replylist
pub async fn floor_comments(
    state: &AppState,
    session: &KgSession,
    p: &FloorCommentsParams<'_>,
) -> AppResult<Value> {
    let normalized = p.resource_type.to_lowercase();
    let resolved_code = p.code.filter(|s| !s.trim().is_empty()).unwrap_or(match normalized.as_str() {
        "playlist" => CODE_PLAYLIST,
        "album" => CODE_ALBUM,
        _ => CODE_SONG,
    });
    let use_service = normalized == "playlist"
        || normalized == "album"
        || resolved_code == CODE_PLAYLIST
        || resolved_code == CODE_ALBUM;
    let path = if use_service {
        "/m.comment.service/v1/hot_replylist"
    } else {
        "/mcomment/v1/hot_replylist"
    };

    let mut params = official(params! {
        "childrenid" => p.special_id.unwrap_or(""),
        "need_show_image" => "1",
        "p" => p.page.to_string(),
        "pagesize" => p.pagesize.to_string(),
        "show_classify" => p.show_classify.to_string(),
        "show_hotword_list" => p.show_hotword_list.to_string(),
        "code" => resolved_code,
        "tid" => p.tid,
    });
    if let Some(m) = p.mixsongid.filter(|s| !s.trim().is_empty()) {
        params.insert("mixsongid".into(), m.into());
    }

    let mut req = KgRequest::get(path)
        .method(reqwest::Method::POST)
        .signature_type(SignatureType::OfficialAndroid);
    for (k, v) in params {
        req = req.param(k, v);
    }
    transport::send(&state.http, session, &req).await
}

/// 歌曲分类评论。POST /mcomment/v1/cmt_classify_list
pub async fn music_comment_classify(
    state: &AppState,
    session: &KgSession,
    mixsongid: &str,
    type_id: &str,
    page: i64,
    pagesize: i64,
    sort: i64,
) -> AppResult<Value> {
    let params = official(params! {
        "mixsongid" => mixsongid,
        "need_show_image" => "1",
        "page" => page.to_string(),
        "pagesize" => pagesize.to_string(),
        "type_id" => type_id,
        "extdata" => "0",
        "code" => CODE_SONG,
        "sort_method" => if sort == 2 { "2" } else { "1" },
    });
    send_comment_list(state, session, "/mcomment/v1/cmt_classify_list", params).await
}

/// 歌曲热词评论。POST /mcomment/v1/get_hot_word
pub async fn music_comment_hotword(
    state: &AppState,
    session: &KgSession,
    mixsongid: &str,
    hot_word: &str,
    page: i64,
    pagesize: i64,
) -> AppResult<Value> {
    let params = official(params! {
        "mixsongid" => mixsongid,
        "need_show_image" => "1",
        "p" => page.to_string(),
        "pagesize" => pagesize.to_string(),
        "hot_word" => hot_word,
        "extdata" => "0",
        "code" => CODE_SONG,
    });
    send_comment_list(state, session, "/mcomment/v1/get_hot_word", params).await
}

/// 通用评论列表发送（POST + OfficialAndroid）。
async fn send_comment_list(
    state: &AppState,
    session: &KgSession,
    path: &str,
    params: BTreeMap<String, String>,
) -> AppResult<Value> {
    let mut req = KgRequest::get(path)
        .method(reqwest::Method::POST)
        .signature_type(SignatureType::OfficialAndroid);
    for (k, v) in params {
        req = req.param(k, v);
    }
    transport::send(&state.http, session, &req).await
}
