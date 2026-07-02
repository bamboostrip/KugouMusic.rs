//! 登录业务层 —— 对应 .NET 的 `LoginClient` + `RawLoginApi`。
//!
//! 含手机号验证码登录（AES-256 t1/t2 + 随机 AES payload + 裸 RSA pk）、
//! 扫码登录（Web 签名）、token 刷新（AES p3）、登出。
//! 登录成功后把 userid/token/viptype/viptoken/t1 写回 session 并持久化。

use serde_json::{json, Value};

use crate::error::{AppError, AppResult};
use crate::kugou::{
    crypto,
    request::{KgRequest, SignatureType},
    session::KgSession,
    session_store,
    signer,
    transport,
};
use crate::state::AppState;

// 登录专用 AES key/iv（RawLoginApi.cs 硬编码）
const LITE_T1_KEY: &str = "5e4ef500e9597fe004bd09a46d8add98";
const LITE_T1_IV: &str = "04bd09a46d8add98";
const LITE_T2_KEY: &str = "fd14b35e3f81af3817a20ae7adae7020";
const LITE_T2_IV: &str = "17a20ae7adae7020";
const T2_FIXED_HASH: &str = "0f607264fc6318a92b9e13c65db7cd3c";
const LITE_APP_KEY: &str = "c24f74ca2820225badc01946dba4fdf7";
const LITE_APP_IV: &str = "adc01946dba4fdf7";

const API_HOST: &str = "http://login.user.kugou.com";
const LOGIN_ROUTER: &str = "login.user.kugou.com";
const LOGIN_RETRY_HOST: &str = "https://loginserviceretry.kugou.com";
const WEB_HOST: &str = "https://login-user.kugou.com";

/// 解密登录响应：若 secu_params 存在，用 aesKey 解密后合并进根节点。
///
/// 对应 RawLoginApi.TryDecryptResponse。
/// **注意**：transport::send 已把 `data` 提升为根节点，所以这里直接在根节点
/// 查 `secu_params`，解密后也直接合并进根节点（不再有 data 子对象）。
fn try_decrypt_response(response: Value, aes_key: Option<&str>) -> Value {
    let Some(aes_key) = aes_key else { return response };
    let secu = match response.get("secu_params").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return response,
    };
    let plain = crypto::aes_decrypt(secu, aes_key);
    let decrypted_json = match serde_json::from_str::<Value>(&plain) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "[login] 解密 secu_params 失败");
            return response;
        }
    };

    // 把解密字段直接合并进根节点（transport 已提升 data → root）
    let mut root = response;
    if let (Some(root_obj), Some(dec_obj)) = (root.as_object_mut(), decrypted_json.as_object()) {
        for (k, v) in dec_obj {
            root_obj.insert(k.clone(), v.clone());
        }
    }
    root
}


/// 发送短信验证码。对应 RawLoginApi.SendSmsCodeAsync（无加密，Default 签名）。
pub async fn send_sms_code(state: &AppState, session: &KgSession, mobile: &str) -> AppResult<Value> {
    let body = json!({ "businessid": 5, "mobile": mobile, "plat": 3 });
    let req = KgRequest::get("/v7/send_mobile_code")
        .method(reqwest::Method::POST)
        .base_url(API_HOST)
        .router(LOGIN_ROUTER)
        .json_body(body)
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 手机号验证码登录。对应 RawLoginApi.LoginByMobileAsync。
///
/// 成功（status==1 且有 token）时把凭证写回 session 并持久化。
/// 返回 (登录响应, 是否登录成功)。
pub async fn login_by_mobile(
    state: &AppState,
    session_key: &str,
    session: &KgSession,
    mobile: &str,
    code: &str,
    userid: Option<&str>,
) -> AppResult<Value> {
    let date_ms = chrono::Utc::now().timestamp_millis();

    // t1 = "|{ms}"，AES-256 显式 key
    let t1_raw = format!("|{date_ms}");
    let t1_enc = crypto::aes_encrypt(&t1_raw, Some(LITE_T1_KEY), Some(LITE_T1_IV)).cipher_text;

    // t2 = "{install_guid}|{fixed_hash}|{install_mac}|{install_dev}|{ms}"
    let t2_raw = format!("{}|{T2_FIXED_HASH}|{}|{}|{date_ms}",
        session.install_guid, session.install_mac, session.install_dev);
    let t2_enc = crypto::aes_encrypt(&t2_raw, Some(LITE_T2_KEY), Some(LITE_T2_IV)).cipher_text;

    // payload = {mobile, code}，随机 AES key（用于解密响应）
    let aes_payload = json!({ "mobile": mobile, "code": code });
    let payload_enc = crypto::aes_encrypt(&aes_payload.to_string(), None, None);

    // pk = 裸 RSA({clienttime_ms, key})，大写 hex
    let pk_data = json!({ "clienttime_ms": date_ms, "key": payload_enc.temp_key });
    let pk = crypto::rsa_encrypt_no_padding(&pk_data.to_string(), true).to_uppercase();

    // 手机号脱敏
    let masked = if mobile.chars().count() > 10 {
        let chars: Vec<char> = mobile.chars().collect();
        format!("{}*****{}", chars[..2].iter().collect::<String>(), chars[10])
    } else {
        mobile.to_string()
    };

    let mut body = json!({
        "plat": 1,
        "support_multi": 1,
        "t1": t1_enc,
        "t2": t2_enc,
        "clienttime_ms": date_ms,
        "mobile": masked,
        "key": signer::calc_login_key(date_ms),
        "pk": pk,
        "params": payload_enc.cipher_text,
        "dfid": "-",
        "dev": session.install_dev,
        "gitversion": "5f0b7c4"
    });

    let login_userid = userid.filter(|s| !s.is_empty()).unwrap_or(&session.userid);
    if !login_userid.is_empty() && login_userid != "0" {
        body["userid"] = json!(login_userid.parse::<i64>().unwrap_or(0));
    }

    let req = KgRequest::get("/v7/login_by_verifycode")
        .method(reqwest::Method::POST)
        .base_url(LOGIN_RETRY_HOST)
        .router(LOGIN_ROUTER)
        .json_body(body)
        .custom_header("support-calm", "1")
        .signature_type(SignatureType::Default);

    let resp = transport::send(&state.http, session, &req).await?;
    let resp = try_decrypt_response(resp, Some(&payload_enc.temp_key));

    // 成功则写回 session
    if let Some(uid) = resp.get("userid").and_then(|v| v.as_i64())
        && let Some(token) = resp.get("token").and_then(|v| v.as_str())
        && !token.is_empty()
        && uid != 0
    {
        let t1 = resp.get("t1").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mut updated = session.clone();
        updated.update_auth(uid.to_string(), token, "0", "", t1);
        session_store::save(&state.db, session_key, &updated).await;
    }

    Ok(resp)
}

/// 获取扫码登录二维码。对应 RawLoginApi.GetQrKeyAsync（Web 签名，无加密）。
pub async fn get_qr_key(state: &AppState, session: &KgSession) -> AppResult<Value> {
    let req = KgRequest::get("/v2/qrcode")
        .base_url(WEB_HOST)
        .param("appid", "1001")
        .param("clientver", "11040")
        .param("type", "1")
        .param("plat", "4")
        .param("srcappid", "2919")
        .param("qrcode_txt", "https://h5.kugou.com/apps/loginQRCode/html/index.html?appid=3116&")
        .signature_type(SignatureType::Web);
    transport::send(&state.http, session, &req).await
}

/// 检查扫码状态。对应 RawLoginApi.CheckQrStatusAsync（Web 签名）。
/// 成功时写回 session。
pub async fn check_qr_status(
    state: &AppState,
    session_key: &str,
    session: &KgSession,
    key: &str,
) -> AppResult<Value> {
    let req = KgRequest::get("/v2/get_userinfo_qrcode")
        .base_url(WEB_HOST)
        .param("plat", "4")
        .param("appid", "3116")
        .param("srcappid", "2919")
        .param("qrcode", key)
        .signature_type(SignatureType::Web);
    let resp = transport::send(&state.http, session, &req).await?;

    // .NET: status == 4 (QrLoginStatus.Success) 表示扫码确认登录成功
    // QR 状态码：1=已扫码，2=过期，0=等待，4/5=已确认
    let status = resp.get("status").and_then(|v| v.as_i64());
    let status_with_sdk = resp.get("status_with_sdk").and_then(|v| v.as_i64());
    let sdk_ok = status == Some(4) || status == Some(5) || status_with_sdk == Some(1);

    if sdk_ok
        && let (Some(uid), Some(token)) = (
            resp.get("userid").and_then(|v| v.as_i64()),
            resp.get("token").and_then(|v| v.as_str()),
        )
        && !token.is_empty()
        && uid != 0
    {
        let mut updated = session.clone();
        updated.update_auth(uid.to_string(), token, "0", "", "");
        session_store::save(&state.db, session_key, &updated).await;
    }
    Ok(resp)
}

/// 刷新 token（保活）。对应 RawLoginApi.RefreshTokenAsync。
///
/// 无 token/userid 时直接返回错误。
pub async fn refresh_token(
    state: &AppState,
    session_key: &str,
    session: &KgSession,
) -> AppResult<Value> {
    if session.token.is_empty() || session.userid == "0" {
        return Err(AppError::Unauthorized("本地无有效 Token，无法刷新".into()));
    }

    let date_ms = chrono::Utc::now().timestamp_millis();
    let clienttime_sec = date_ms / 1000;

    // t1：无 lastT1 时 "|{ms}"，否则 "{lastT1}|{ms}"
    let t1_raw = if session.t1.is_empty() {
        format!("|{date_ms}")
    } else {
        format!("{}|{date_ms}", session.t1)
    };
    let t1_enc = crypto::aes_encrypt(&t1_raw, Some(LITE_T1_KEY), Some(LITE_T1_IV)).cipher_text;

    // t2
    let t2_raw = format!("{}|{T2_FIXED_HASH}|{}|{}|{date_ms}",
        session.install_guid, session.install_mac, session.install_dev);
    let t2_enc = crypto::aes_encrypt(&t2_raw, Some(LITE_T2_KEY), Some(LITE_T2_IV)).cipher_text;

    // p3 = AES({clienttime, token})，用 LiteAppKey/IV
    let p3_data = json!({ "clienttime": clienttime_sec, "token": session.token });
    let p3_enc = crypto::aes_encrypt(&p3_data.to_string(), Some(LITE_APP_KEY), Some(LITE_APP_IV)).cipher_text;

    // params = AES("{}")，随机 key（用于解密响应 secu_params）
    let params_enc = crypto::aes_encrypt("{}", None, None);

    // pk = 裸 RSA({clienttime_ms, key})，大写
    let pk_data = json!({ "clienttime_ms": date_ms, "key": params_enc.temp_key });
    let pk = crypto::rsa_encrypt_no_padding(&pk_data.to_string(), true).to_uppercase();

    let body = json!({
        "dfid": "-",
        "p3": p3_enc,
        "plat": 1,
        "t1": t1_enc,
        "t2": t2_enc,
        "t3": "MCwwLDAsMCwwLDAsMCwwLDA=",
        "pk": pk,
        "params": params_enc.cipher_text,
        "userid": session.userid,
        "clienttime_ms": date_ms,
        "dev": session.install_dev
    });

    let req = KgRequest::get("/v5/login_by_token")
        .method(reqwest::Method::POST)
        .base_url(API_HOST)
        .router(LOGIN_ROUTER)
        .json_body(body)
        .signature_type(SignatureType::Default);

    let resp = transport::send(&state.http, session, &req).await?;
    let resp = try_decrypt_response(resp, Some(&params_enc.temp_key));

    // 成功则写回（含 viptype/t1）
    if resp.get("status").and_then(|v| v.as_i64()) == Some(1)
        && let Some(token) = resp.get("token").and_then(|v| v.as_str())
        && !token.is_empty()
    {
        let uid = resp.get("userid").and_then(|v| v.as_i64()).unwrap_or(0).to_string();
        let vip = resp.get("is_vip").and_then(|v| v.as_i64()).unwrap_or(0).to_string();
        let t1 = resp.get("t1").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mut updated = session.clone();
        updated.update_auth(uid, token, &vip, "", t1);
        session_store::save(&state.db, session_key, &updated).await;
    }
    Ok(resp)
}

/// 登出：清空 session 凭证并删除 DB 记录。对应 LoginClient.LogOutAsync。
pub async fn logout(state: &AppState, session_key: &str, session: &KgSession) {
    let mut updated = session.clone();
    updated.logout();
    session_store::save(&state.db, session_key, &updated).await;
}
