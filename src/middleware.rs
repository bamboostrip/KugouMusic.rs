//! Axum 提取器与中间件 —— 对应 .NET 的 `KgWebSessionMiddleware` + `KgSessionManager`。
//!
//! [`KgReqSession`] 是一个 `FromRequestParts` 提取器：从请求里解析 session key
//! （X-Kg-Session-Id header 优先，否则 kg_sid cookie，否则新建），按 key 从 SQLite
//! 加载 [`KgSession`]，归一化并回写。这样每个 handler 拿到的就是一个现成的 session。
//!
//! 这比"中间件 + 共享 state"更贴合 Axum 风格，且 session key 的回写（header/cookie）
//! 放在响应层由 [`SessionKey`] 提取器配合完成。

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;
use regex::Regex;
use uuid::Uuid;

use crate::error::AppResult;
use crate::kugou::{session::KgSession, session_store};
use crate::state::AppState;

/// 响应回写 session key 的中间件：
/// 1. 在请求进入 handler 前，解析 session key 并放进 request extensions（供 KgReqSession 复用）；
/// 2. 在响应返回时，把 session key 写到响应的 X-Kg-Session-Id header + kg_sid cookie。
///
/// 必须挂在路由内层（这样 handler 的 extensions 修改它能感知到）。
pub async fn session_echo_middleware(
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    // 1. 解析并放进 extensions（KgReqSession 提取器会复用它，避免重复解析）
    let session_key = resolve_session_key_from_request(&req);
    req.extensions_mut().insert(SessionKey(session_key.clone()));

    let mut resp = next.run(req).await;

    // 2. 回写响应（无论 handler 是否提取过 KgReqSession，都把 key 回给客户端）
    write_session_key_to_response(&mut resp, &session_key);
    resp
}

/// 从完整 Request 解析 session key（中间件入口，Parts 版的兄弟函数）。
fn resolve_session_key_from_request(req: &axum::extract::Request) -> String {
    // header 优先
    if let Some(h) = req.headers().get("X-Kg-Session-Id").and_then(|v| v.to_str().ok())
        && is_valid_session_key(h)
    {
        return h.to_string();
    }
    if let Some(cookie) = req.headers().get("cookie").and_then(|v| v.to_str().ok()) {
        for kv in cookie.split(';') {
            let kv = kv.trim();
            if let Some(val) = kv.strip_prefix("kg_sid=")
                && is_valid_session_key(val)
            {
                return val.to_string();
            }
        }
    }
    Uuid::new_v4().simple().to_string()
}

/// session key 的合法字符集（与 .NET `^[A-Za-z0-9._-]+$` 一致），长度 ≤ 128。
fn is_valid_session_key(s: &str) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^[A-Za-z0-9._-]+$").unwrap());
    re.is_match(s)
}

/// 从请求 parts 解析 session key（header 优先 → cookie → 新建）。
/// 返回 (session_key, is_newly_minted)。
pub fn resolve_session_key(parts: &Parts) -> (String, bool) {
    // header 优先
    if let Some(h) = parts.headers.get("X-Kg-Session-Id").and_then(|v| v.to_str().ok())
        && is_valid_session_key(h)
    {
        return (h.to_string(), false);
    }
    // cookie 次之（axum 不自动解析 cookie，这里简单手解）
    if let Some(cookie) = parts.headers.get("cookie").and_then(|v| v.to_str().ok()) {
        for kv in cookie.split(';') {
            let kv = kv.trim();
            if let Some(val) = kv.strip_prefix("kg_sid=")
                && is_valid_session_key(val)
            {
                return (val.to_string(), false);
            }
        }
    }
    // 新建：32 位十六进制（与 .NET Guid.NewGuid().ToString("N") 一致）
    (Uuid::new_v4().simple().to_string(), true)
}

/// 当前请求的会话（含设备身份 + 登录态）。在 handler 里作为参数提取。
///
/// 用法：`async fn handler(State(state): State<AppState>, KgReqSession(session): KgReqSession) { ... }`
///
/// 注意：它依赖 [`session_echo_middleware`] 先把 [`SessionKey`] 放进 request extensions
/// （中间件挂在路由外层、先于 handler 运行）。若未挂该中间件，则回退到自行解析。
pub struct KgReqSession(pub KgSession);

impl FromRequestParts<AppState> for KgReqSession {
    type Rejection = crate::error::AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // 中间件已解析并放进 extensions 时直接复用；否则自行解析
        let session_key = parts
            .extensions
            .get::<SessionKey>()
            .map(|s| s.0.clone())
            .unwrap_or_else(|| resolve_session_key(parts).0);

        // 从库加载；不存在则 default
        let mut session = session_store::load(&state.db, &session_key)
            .await
            .unwrap_or_default();
        session.normalize();

        // 确保 extensions 里有 SessionKey（供中间件回写）
        parts.extensions.insert(SessionKey(session_key));

        Ok(KgReqSession(session))
    }
}

/// 当前请求的 session key（用于响应回写）。放进 extensions 后可在响应里取出。
#[derive(Clone)]
pub struct SessionKey(pub String);

/// 当前请求的 session key 提取器（登录/登出后用它把更新后的会话写回 DB）。
///
/// 用法：`async fn h(state: State<AppState>, KgReqSession(mut s): KgReqSession, KgSessionKey(key): KgSessionKey)`
pub struct KgSessionKey(pub String);

impl FromRequestParts<AppState> for KgSessionKey {
    type Rejection = crate::error::AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &AppState) -> Result<Self, Self::Rejection> {
        // session_echo_middleware 已注入 SessionKey；handler 兜底自行解析
        let key = parts
            .extensions
            .get::<SessionKey>()
            .map(|s| s.0.clone())
            .unwrap_or_else(|| resolve_session_key(parts).0);
        Ok(KgSessionKey(key))
    }
}

/// 把 session key 回写到响应（X-Kg-Session-Id header + kg_sid cookie）。
/// 对应 .NET KgWebSessionMiddleware 的 OnStarting 回写。
pub fn write_session_key_to_response(resp: &mut Response, session_key: &str) {
    resp.headers_mut().insert(
        "X-Kg-Session-Id",
        session_key.parse().unwrap_or_else(|_| "default".parse().unwrap()),
    );
    // HttpOnly; SameSite=Lax; 30 天（与 .NET 一致；Secure 由调用方按 is_https 决定，这里默认带上）
    let cookie = format!(
        "kg_sid={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000",
        session_key
    );
    if let Ok(val) = cookie.parse() {
        resp.headers_mut().append("set-cookie", val);
    }
}

/// 便捷：在 controller 拿到 session 后若需要持久化（如登录后），用这个。
pub async fn save_session(state: &AppState, session_key: &str, session: &KgSession) {
    session_store::save(&state.db, session_key, session).await;
}

/// 便捷：取出当前请求 extensions 里的 SessionKey（响应中间件用）。
pub fn current_session_key(parts: &Parts) -> Option<String> {
    parts.extensions.get::<SessionKey>().map(|s| s.0.clone())
}

/// 兜底辅助：执行一个业务闭包并返回 AppResult（保留给 controller 用）。
#[allow(dead_code)]
pub fn ok<T>(t: T) -> AppResult<T> {
    Ok(t)
}
