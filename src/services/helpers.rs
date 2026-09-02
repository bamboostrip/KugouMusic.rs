//! 服务层通用辅助 —— 会话失效自动刷新与重试。
//!
//! 对齐 Flutter bridage `rust/src/engine.rs:dispatch` 的逻辑：
//! 上游返回 `status:0 + error_code 20000-20100` 视为登录态失效，
//! 自动调用 `refresh_token` 并重放一次原请求，成功则对用户无感。

use serde_json::Value;

use crate::error::AppResult;
use crate::kugou::session::KgSession;
use crate::state::AppState;

/// 判断上游响应是否为“会话失效/需要登录”（与 Flutter 侧一致）。
///
/// 实测：`/song/url` 返回 `{"errcode":20028,"error":"本次请求需要验证","status":0}`，
/// `/user/playlist` 返回 `{"error_code":20017,"status":0}`；20xxx 均为登录类错误。
pub fn is_session_invalid_response(v: &Value) -> bool {
    if v.get("status").and_then(|s| s.as_i64()) != Some(0) {
        return false;
    }
    let code = v
        .get("error_code")
        .and_then(|c| c.as_i64())
        .or_else(|| v.get("errcode").and_then(|c| c.as_i64()));
    matches!(code, Some(c) if (20000..20100).contains(&c))
}

/// 是否为免重试路径（登录/验证码等本身就是鉴权流程，不应触发刷新）。
pub fn is_exempt_path(path: &str) -> bool {
    path.starts_with("/login") || path.starts_with("/captcha") || path.starts_with("/register")
}

/// 带会话自动刷新的包装器：先执行 `f(session.clone())`，若返回会话失效则尝试刷新并用新会话重放一次。
///
/// - `path` 用于日志与免重试判断（如 `"/song/url"`）。
/// - 仅当 `session.is_logged_in()` 且响应为失效时才尝试刷新。
/// - 刷新成功则用 DB 中最新会话重放；失败则返回原始失效响应。
/// - 闭包签名为 `FnMut(KgSession) -> Fut`（按值接收，便于 async move 捕获且避免生命周期问题）。
pub async fn with_auto_retry<F, Fut>(
    state: &AppState,
    session_key: &str,
    session: &KgSession,
    path: &str,
    mut f: F,
) -> AppResult<Value>
where
    F: FnMut(KgSession) -> Fut,
    Fut: std::future::Future<Output = AppResult<Value>>,
{
    let first = f(session.clone()).await?;
    if !is_session_invalid_response(&first) || is_exempt_path(path) || !session.is_logged_in() {
        return Ok(first);
    }
    tracing::info!(path, "检测到会话失效，尝试自动刷新 token");
    match crate::services::login::refresh_token(state, session_key, session).await {
        Ok(_) => {
            // 刷新后从 DB 重载最新会话（包含新 token/t1/vip_type 等）
            let new_session = crate::kugou::session_store::load(&state.db, session_key)
                .await
                .unwrap_or_else(|| session.clone());
            tracing::info!(path, "刷新成功，重试原请求");
            f(new_session).await
        }
        Err(e) => {
            tracing::warn!(path, error = %e, "会话失效且刷新失败，返回原始响应");
            Ok(first)
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn session_invalid_detection() {
        assert!(is_session_invalid_response(&json!({"errcode":20028,"error":"本次请求需要验证","status":0})));
        assert!(is_session_invalid_response(&json!({"error_code":20017,"status":0})));
        assert!(!is_session_invalid_response(&json!({"status":1,"error_code":0,"url":"https://example.com/a.mp3"})));
        assert!(!is_session_invalid_response(&json!({"status":0,"error_code":9001,"err":"缺参"})));
        assert!(!is_session_invalid_response(&json!({"error_code":20017})));
        assert!(!is_session_invalid_response(&json!({"status":2,"error_code":20017})));
        assert!(!is_session_invalid_response(&json!({"status":0,"msg":"服务内部错误"})));
    }

    #[test]
    fn exempt_paths() {
        assert!(is_exempt_path("/login/cellphone"));
        assert!(is_exempt_path("/login/qr/check"));
        assert!(is_exempt_path("/captcha/sent"));
        assert!(!is_exempt_path("/song/url"));
        assert!(!is_exempt_path("/user/playlist"));
    }
}
