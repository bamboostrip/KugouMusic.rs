//! 登录控制器 —— 对应 .NET `LoginController`（前缀 `login`）+ `CaptchaController`。

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::error::AppResult;
use crate::middleware::{KgReqSession, KgSessionKey};
use crate::services;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MobileLoginRequest {
    #[validate(length(min = 1, message = "mobile 不能为空"))]
    #[allow(dead_code)]
    pub mobile: String,
    #[validate(length(equal = 6, message = "code 须为 6 位"))]
    #[allow(dead_code)]
    pub code: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub userid: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CaptchaSentQuery {
    #[allow(dead_code)]
    pub mobile: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct QrCheckQuery {
    pub key: String,
}

/// `POST /login/cellphone` —— 手机号验证码登录。
#[utoipa::path(post, path = "/login/cellphone", tag = "login", responses((status = 200, body = Object)))]
async fn login_cellphone(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    KgSessionKey(session_key): KgSessionKey,
    Json(req): Json<MobileLoginRequest>,
) -> AppResult<Json<Value>> {
    req.validate()?;
    Ok(Json(
        services::login::login_by_mobile(
            &state, &session_key, &session, &req.mobile, &req.code, req.userid.as_ref().map(|i| i.to_string()).as_deref(),
        )
        .await?,
    ))
}

/// `POST /captcha/sent` —— 发送验证码。
#[utoipa::path(post, path = "/captcha/sent", tag = "login", responses((status = 200, body = Object)))]
async fn captcha_sent(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    Query(q): Query<CaptchaSentQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::login::send_sms_code(&state, &session, &q.mobile).await?))
}

/// `GET /login/qr/key` —— 获取扫码登录二维码。
#[utoipa::path(get, path = "/login/qr/key", tag = "login", responses((status = 200, body = Object)))]
async fn login_qr_key(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
) -> AppResult<Json<Value>> {
    Ok(Json(services::login::get_qr_key(&state, &session).await?))
}

/// `GET /login/qr/check` —— 轮询扫码状态（成功后自动写回 token）。
#[utoipa::path(get, path = "/login/qr/check", tag = "login", params(("key" = String, Query)), responses((status = 200, body = Object)))]
async fn login_qr_check(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    KgSessionKey(session_key): KgSessionKey,
    Query(q): Query<QrCheckQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(services::login::check_qr_status(&state, &session_key, &session, &q.key).await?))
}

/// `POST /login/token` —— 刷新 token（保活）。
#[utoipa::path(post, path = "/login/token", tag = "login", responses((status = 200, body = Object)))]
async fn login_token(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    KgSessionKey(session_key): KgSessionKey,
) -> AppResult<Json<Value>> {
    Ok(Json(services::login::refresh_token(&state, &session_key, &session).await?))
}

/// `POST /login/logout` —— 登出。
#[utoipa::path(post, path = "/login/logout", tag = "login", responses((status = 200, body = Object)))]
async fn login_logout(
    State(state): State<AppState>,
    KgReqSession(session): KgReqSession,
    KgSessionKey(session_key): KgSessionKey,
) -> AppResult<Json<Value>> {
    services::login::logout(&state, &session_key, &session).await;
    Ok(Json(serde_json::json!({ "status": 1, "msg": "已登出" })))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(login_cellphone))
        .routes(routes!(captcha_sent))
        .routes(routes!(login_qr_key))
        .routes(routes!(login_qr_check))
        .routes(routes!(login_token))
        .routes(routes!(login_logout))
}
