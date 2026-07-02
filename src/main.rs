//! 异步入口：初始化日志 → 读取配置 → 建连接池 → 跑迁移 → 启动服务。
//!
//! 核心逻辑都在库 crate（lib.rs）里；本文件只负责启动编排。

use std::time::Duration;

use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use kugou_web_api::config::Config;
use kugou_web_api::middleware::session_echo_middleware;
use kugou_web_api::routes;
use kugou_web_api::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 0. 安装 rustls 的 ring crypto provider。
    //    reqwest/sqlx 都用 rustls-no-provider，必须由我们在进程启动时
    //    装一个具体后端（这里选 ring），否则首次 HTTPS 握手会 panic。
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("install rustls ring provider");

    // 1. 加载 .env（不存在不报错，CI/容器里通常直接用环境变量）
    let _ = dotenvy::dotenv();

    // 2. 先用默认日志级别初始化 Tracing，确保后续所有启动错误都能被记录
    //    （config 加载失败、DB 连接失败等都需要日志输出才能诊断）
    let pre_cfg_log = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "kugou_web_api=debug,tower_http=info".into());
    init_tracing(&pre_cfg_log);

    // 3. 读配置（失败即退出，启动期错误不可吞）
    let cfg = Config::from_env()?;

    tracing::info!(env = %cfg.env, "启动 KuGou Web API");

    // 4. SQLite 连接池
    let db = init_db(&cfg).await?;

    // 5. 复用的上游 HTTP 客户端（代理酷狗时统一用它）
    let http = reqwest::Client::builder()
        .timeout(cfg.http_timeout)
        .gzip(true)
        .build()?;

    // 6. 组装路由 + Swagger
    let state = AppState::new(db, http);
    let (api_router, api_doc) = routes::app_router(state);

    let app = Router::new()
        .merge(api_router)
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", api_doc),
        )
        // session 回写中间件：放在最内层，handler 的 extensions 它能感知到。
        // 它会把 X-Kg-Session-Id / kg_sid 写到响应，并在请求前注入 SessionKey。
        .layer(axum::middleware::from_fn(session_echo_middleware))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(CatchPanicLayer::new()) // 任一 handler panic 也返回 500，不让进程崩
        .layer(CorsLayer::very_permissive()) // 生产请收紧到具体 origin
        .layer(TraceLayer::new_for_http());

    // 7. 监听
    let addr = format!("{}:{}", cfg.host, cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 Swagger 面板: http://{addr}/swagger-ui/");
    tracing::info!("📖 OpenAPI JSON: http://{addr}/api-docs/openapi.json");
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing(directive: &str) {
    let filter = EnvFilter::try_new(directive).unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
}

async fn init_db(cfg: &Config) -> anyhow::Result<SqlitePool> {
    // 从 DATABASE_URL 中提取数据库文件的父目录并确保它存在。
    // 支持 sqlite://./data/app.db 和 sqlite:///app/data/app.db 两种写法。
    let db_path = cfg.database_url
        .trim_start_matches("sqlite://")
        .split('?')
        .next()
        .unwrap_or("data/app.db");
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!(path = %parent.display(), error = %e, "创建数据库目录失败（可能已存在）");
        } else {
            tracing::debug!(path = %parent.display(), "数据库目录已就绪");
        }
    }

    tracing::info!(url = %cfg.database_url, "正在连接 SQLite 数据库");
    let pool = SqlitePoolOptions::new()
        .max_connections(5) // SQLite 写并发有限，小项目 5 足够
        .connect(&cfg.database_url)
        .await?;

    // 跑迁移（migrations/ 目录）
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("数据库迁移完成");
    Ok(pool)
}
