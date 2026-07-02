//! 设备注册业务层 —— 对应 .NET `RegisterClient` + `RawDeviceApi`。
//!
//! 注册风控设备，产出真实 dfid（替换占位 "-"）。用 AES-128(playlist) + RSA PKCS1。

use serde_json::{json, Value};

use crate::error::AppResult;
use crate::kugou::{
    config, crypto,
    request::{KgRequest, SignatureType},
    session::KgSession,
    session_store,
    transport,
};
use crate::state::AppState;

/// 注册设备并写回 dfid/mid/uuid。对应 RegisterClient.RegisterDeviceAsync。
///
/// 返回 true 表示成功拿到 dfid。session 已有有效 dfid 时直接返回 true（不重复注册）。
pub async fn register_device(
    state: &AppState,
    session_key: &str,
    session: &KgSession,
) -> AppResult<bool> {
    if !session.dfid.is_empty() && session.dfid != "-" {
        return Ok(true);
    }

    let client_time = chrono::Utc::now().timestamp();

    // 硬件信息（复刻 .NET dataMap）
    let hardware = json!({
        "availableRamSize": 4983533568_i64, "availableRomSize": 48114719_i64, "availableSDSize": 48114717_i64,
        "basebandVer": "", "batteryLevel": 100, "batteryStatus": 3,
        "brand": "Redmi", "buildSerial": "unknown", "device": "marble",
        "imei": session.install_guid, "imsi": "",
        "manufacturer": "Xiaomi", "uuid": session.install_guid,
        "accelerometer": false, "accelerometerValue": "", "gravity": false, "gravityValue": "",
        "gyroscope": false, "gyroscopeValue": "", "light": false, "lightValue": "",
        "magnetic": false, "magneticValue": "", "orientation": false, "orientationValue": "",
        "pressure": false, "pressureValue": "", "step_counter": false, "step_counterValue": "",
        "temperature": false, "temperatureValue": ""
    });

    // AES-128(playlist) 加密硬件信息
    let aes = crypto::playlist_aes_encrypt(&hardware.to_string());

    // p = RSA PKCS1({aes, uid, token})，大写
    let p_data = json!({ "aes": aes.temp_key, "uid": session.userid, "token": session.token });
    let p = crypto::rsa_encrypt_pkcs1(&p_data.to_string(), true).to_uppercase();

    let req = KgRequest::get("/risk/v2/r_register_dev")
        .method(reqwest::Method::POST)
        .base_url("https://userservice.kugou.com")
        .param("part", "1")
        .param("platid", "1")
        .param("p", p)
        .param("clientver", config::CLIENT_VER)
        .param("clienttime", client_time.to_string())
        .param("appid", config::APP_ID)
        .raw_body(aes.cipher_text) // AES 密文（base64）作为 raw body
        .not_signature()
        .signature_type(SignatureType::Default);

    // 注：注册时还没有 dfid，header dfid 设 "-"
    let req = req.specific_dfid("-");
    let resp = transport::send(&state.http, session, &req).await?;

    // 解密响应：可能是 __raw_base64__ 或字符串
    let encrypted = resp
        .get("__raw_base64__")
        .and_then(|v| v.as_str())
        .or_else(|| resp.as_str());
    if let Some(enc) = encrypted.filter(|s| !s.is_empty())
        && let Ok(decrypted) = serde_json::from_str::<Value>(&crypto::playlist_aes_decrypt(enc, &aes.temp_key))
        && let Some(d5) = decrypted.get("dfid").and_then(|v| v.as_str()).filter(|s| !s.is_empty())
    {
        let mut updated = session.clone();
        updated.dfid = d5.to_string();
        updated.mid = crypto::calc_new_mid(d5);
        updated.uuid = crypto::md5_str(&format!("{}{}", d5, updated.mid));
        session_store::save(&state.db, session_key, &updated).await;
        tracing::info!(dfid = %d5, "[device] 注册成功");
        return Ok(true);
    }
    tracing::warn!("[device] 注册失败，未能解析 dfid");
    Ok(false)
}
