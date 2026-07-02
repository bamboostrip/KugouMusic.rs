//! kugou_web_api 库入口。
//!
//! 把核心逻辑（config/error/kugou/services/state 等）作为库暴露，原因有二：
//! 1. 集成测试（tests/）只能链接库目标，不能链接 bin 目标；
//! 2. 未来若有 CLI 工具或多入口，可直接复用本库。
//!
//! `main.rs`（bin）只是一个薄入口：初始化日志/配置/连接池，然后调 routes 启动。

pub mod config;
pub mod controllers;
pub mod error;
pub mod kugou;
pub mod middleware;
pub mod routes;
pub mod services;
pub mod state;
