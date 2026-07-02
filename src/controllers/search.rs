//! 搜索控制器 —— 对应 .NET `SearchController`（路由前缀 `search`）。
//!
//! 只做：入参校验 → 调 service → 把上游透传结果回给前端。
//! 所有端点全匿名（软依赖 userid 的 default 用 session.userid）。

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

/// /search 入参
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SearchQuery {
    #[validate(length(min = 1, message = "关键词不能为空"))]
    pub keywords: String,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    pub pagesize: i64,
    #[serde(default = "default_search_type")]
    #[allow(dead_code)]
    pub r#type: String,
}
fn default_page() -> i64 { 1 }
fn default_pagesize() -> i64 { 30 }
fn default_search_type() -> String { "song".into() }

/// /search/hot 无入参
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchHotQuery {}

/// /search/default 无入参
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchDefaultQuery {}

/// /search/suggest 入参
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SearchSuggestQuery {
    #[validate(length(min = 1, message = "关键词不能为空"))]
    pub keywords: String,
    #[serde(default = "default_album_tip")]
    #[allow(dead_code)]
    pub albumtipcount: i64,
    #[serde(default = "default_correct_tip")]
    #[allow(dead_code)]
    pub correcttipcount: i64,
    #[serde(default = "default_mv_tip")]
    #[allow(dead_code)]
    pub mvtipcount: i64,
    #[serde(default = "default_music_tip")]
    #[allow(dead_code)]
    pub musictipcount: i64,
}
fn default_album_tip() -> i64 { 10 }
fn default_correct_tip() -> i64 { 10 }
fn default_mv_tip() -> i64 { 10 }
fn default_music_tip() -> i64 { 10 }

/// /search/mixed 入参（注意是 keyword 单数）
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SearchMixedQuery {
    #[validate(length(min = 1, message = "关键词不能为空"))]
    pub keyword: String,
}

/// /search/complex 入参
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SearchComplexQuery {
    #[validate(length(min = 1, message = "关键词不能为空"))]
    pub keywords: String,
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_pagesize")]
    #[allow(dead_code)]
    pub pagesize: i64,
}

/// `GET /search` —— 统一搜索。
/// song → 强类型歌曲列表；special/album → 透传；其它 type → 透传。
#[utoipa::path(
    get,
    path = "/search",
    tag = "search",
    params(("keywords" = String, Query, description = "关键词"), ("page" = Option<i64>, Query), ("pagesize" = Option<i64>, Query), ("type" = Option<String>, Query, description = "song/special/album/lyric")),
    responses((status = 200, description = "搜索结果", body = Object))
)]
async fn search(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<SearchQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    let search_type = q.r#type.as_str();
    // song 走类型化，其它透传（与 .NET 一致）
    let v = services::search::search_raw(
        &state, &session, &q.keywords, q.page, q.pagesize, search_type,
    )
    .await?;
    Ok(Json(v))
}

/// `GET /search/hot` —— 热搜。
#[utoipa::path(
    get,
    path = "/search/hot",
    tag = "search",
    responses((status = 200, description = "热搜列表", body = Object))
)]
async fn search_hot(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
) -> AppResult<Json<Value>> {
    Ok(Json(services::search::search_hot(&state, &session).await?))
}

/// `GET /search/default` —— 默认搜索词（软依赖登录态）。
#[utoipa::path(
    get,
    path = "/search/default",
    tag = "search",
    responses((status = 200, description = "默认搜索词", body = Object))
)]
async fn search_default(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
) -> AppResult<Json<Value>> {
    let userid = session.userid.clone();
    let vip_type = if session.vip_type.is_empty() { "65530".to_string() } else { session.vip_type.clone() };
    Ok(Json(services::search::search_default(&state, &session, &userid, &vip_type).await?))
}

/// `GET /search/suggest` —— 搜索建议。
#[utoipa::path(
    get,
    path = "/search/suggest",
    tag = "search",
    params(("keywords" = String, Query)),
    responses((status = 200, description = "搜索建议", body = Object))
)]
async fn search_suggest(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<SearchSuggestQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(
        services::search::search_suggest(
            &state, &session, &q.keywords, q.albumtipcount, q.correcttipcount, q.mvtipcount, q.musictipcount,
        )
        .await?,
    ))
}

/// `GET /search/mixed` —— 混合搜索。
#[utoipa::path(
    get,
    path = "/search/mixed",
    tag = "search",
    params(("keyword" = String, Query)),
    responses((status = 200, description = "混合搜索结果", body = Object))
)]
async fn search_mixed(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<SearchMixedQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(services::search::search_mixed(&state, &session, &q.keyword).await?))
}

/// `GET /search/complex` —— 综合搜索。
#[utoipa::path(
    get,
    path = "/search/complex",
    tag = "search",
    params(("keywords" = String, Query), ("page" = Option<i64>, Query), ("pagesize" = Option<i64>, Query)),
    responses((status = 200, description = "综合搜索结果", body = Object))
)]
async fn search_complex(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<SearchComplexQuery>,
) -> AppResult<Json<Value>> {
    q.validate()?;
    Ok(Json(
        services::search::search_complex(&state, &session, &q.keywords, q.page, q.pagesize).await?,
    ))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(search))
        .routes(routes!(search_hot))
        .routes(routes!(search_default))
        .routes(routes!(search_suggest))
        .routes(routes!(search_mixed))
        .routes(routes!(search_complex))
}
