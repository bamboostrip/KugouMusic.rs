//! 路由与 OpenAPI(Swagger) 聚合。
//!
//! 这里用 utoipa-axum 0.2 的 [`OpenApiRouter`]：注册一条路由的同时，
//! 它就会自动出现在 Swagger 文档里，无需重复声明。
//! 各业务模块通过 `controllers::xxx::router()` 暴露自己的 `OpenApiRouter` 片段，
//! 本文件只负责把它们 `.nest()` 到一起。

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::controllers::{album, app_version, artist, comment, discover, external_playlist, fm, health, login, lyric, media_catalog, playlist, playlist_read, rank, register, report, search, song, user, youth};
use crate::state::AppState;

/// 顶层 OpenAPI 描述：全局信息 + 安全方案(Bearer) 都挂在这里。
#[derive(OpenApi)]
#[openapi(
    info(
        title = "KuGou Web API",
        version = "0.1.0",
        description = "酷狗音乐 Web API（Rust 重写版）—— 代理上游酷狗、给前端访问",
        license(name = "MIT", identifier = "MIT"),
    ),
    modifiers(&SecurityAddon),
    components(schemas(crate::error::ApiErrorResponse)),
    tags(
        (name = "health", description = "健康检查"),
        (name = "search", description = "搜索"),
        (name = "song", description = "歌曲"),
        (name = "lyric", description = "歌词"),
        (name = "rank", description = "排行榜"),
        (name = "album", description = "专辑"),
        (name = "artist", description = "歌手"),
        (name = "comment", description = "评论"),
        (name = "fm", description = "电台"),
        (name = "login", description = "登录/验证码"),
        (name = "register", description = "设备注册"),
        (name = "user", description = "用户"),
        (name = "youth", description = "概念版（频道/动态/每日VIP）"),
        (name = "playlist", description = "歌单"),
        (name = "discover", description = "发现/推荐"),
        (name = "report", description = "上报"),
        (name = "media", description = "媒体目录"),
        (name = "app", description = "App 版本"),
        (name = "external", description = "外部歌单解析"),
    )
)]
struct ApiDoc;

/// 预留：将来接酷狗登录 token / JWT 时，在这里给 Swagger 配全局 Bearer。
struct SecurityAddon;
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, _openapi: &mut utoipa::openapi::OpenApi) {
        // 示例：需要时取消下面的注释，给所有受保护接口加上 Bearer 认证项
        // use utoipa::openapi::security::{SecurityScheme, Http, HttpAuthScheme};
        // let scheme = SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer));
        // openapi.components.as_mut().unwrap().add_security_scheme("bearer", scheme);
    }
}

/// 组装最终应用路由 + OpenAPI 文档。
///
/// 返回的元组里第二个元素是合并好所有 path 的 `OpenApi`，
/// 由 `main.rs` 交给 `SwaggerUi::url(...)` 渲染。
pub fn app_router(state: AppState) -> (axum::Router, utoipa::openapi::OpenApi) {
    // `split_for_parts` 一步产出 (axum::Router, OpenApi)：前者直接挂进应用，
    // 后者交给 SwaggerUi。这是 utoipa-axum 0.2 的官方推荐用法。
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        // 各业务模块自带 #[utoipa::path] 标注的 handler，挂上来即可。
        .merge(health::router())
        .merge(search::router())
        .merge(song::router())
        .merge(lyric::router())
        .merge(rank::router())
        .merge(album::router())
        .merge(artist::router())
        .merge(comment::router())
        .merge(fm::router())
        .merge(login::router())
        .merge(register::router())
        .merge(user::router())
        .merge(youth::router())
        .merge(playlist::router())
        .merge(playlist_read::router())
        .merge(discover::router())
        .merge(report::router())
        .merge(media_catalog::router())
        .merge(app_version::router())
        .merge(external_playlist::router())
        .with_state(state)
        .split_for_parts()
}
