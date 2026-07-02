//! 环境变量 → 配置结构体映射。
//!
//! 启动时由 [`Config::from_env`] 一次性读取，失败即 panic（启动期错误用 anyhow 聚合）。
//! 不引入额外的 `config` crate：用 std + dotenvy 足够，少一个依赖、少一份体积。

use std::env;
use std::time::Duration;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub env: String,
    /// tracing `EnvFilter` 直接表达式，例如 `kugou_web_api=debug,tower_http=info`
    pub rust_log: String,
    pub database_url: String,
    /// 上游酷狗网关（代理请求时由 service 层读取）
    #[allow(dead_code)]
    pub kugou_gateway_url: String,
    pub http_timeout: Duration,
}

impl Config {
    /// 从环境变量构造。调用前应已执行 [`dotenvy::dotenv`]。
    pub fn from_env() -> Result<Self> {
        let port = env::var("APP_PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse::<u16>()
            .context("APP_PORT 不是合法的端口号")?;

        let timeout_secs = env::var("HTTP_TIMEOUT_SECS")
            .unwrap_or_else(|_| "10".into())
            .parse::<u64>()
            .context("HTTP_TIMEOUT_SECS 不是合法的秒数")?;

        Ok(Self {
            host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port,
            env: env::var("APP_ENV").unwrap_or_else(|_| "development".into()),
            rust_log: env::var("RUST_LOG")
                .unwrap_or_else(|_| "kugou_web_api=debug,tower_http=info".into()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://data/app.db?mode=rwc".into()),
            kugou_gateway_url: env::var("KUGOU_GATEWAY_URL")
                .unwrap_or_else(|_| "https://gateway.kugou.com".into()),
            http_timeout: Duration::from_secs(timeout_secs),
        })
    }

    /// 按生产/开发环境切换行为（中间件、CORS 等）。脚手架阶段暂未接线。
    #[allow(dead_code)]
    pub fn is_production(&self) -> bool {
        self.env.eq_ignore_ascii_case("production")
    }
}
