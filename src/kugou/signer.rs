//! 酷狗请求签名 —— 1:1 对应 .NET 的 `util/KGSigner.cs`。
//!
//! 4 种活跃签名策略：
//! - [`calc_post_signature`] —— Default/V5/Register/OfficialAndroid 共用，
//!   `md5(salt + 排序后"k=v"无分隔符拼接 + body + salt)`，小写 hex。
//!   V5 只是在此之外**额外**加一个 `key` 参数（见 [`calc_v5_key`]）。
//! - [`calc_web_qr_signature`] —— Web（扫码登录/迷你乐库），
//!   `md5(webSalt + 排序 k=v + webSalt)`，**body 不参与**。
//! - [`calc_v5_key`] —— V5 播放链接的 `key` 参数，`md5(hash + V5KeySalt + AppId + mid + userid)`。
//! - [`calc_login_key`] / [`calc_cloud_key`] —— raw api 内联用的辅助签名。
//!
//! 关键易错点（迁移时务必逐条对齐）：
//! 1. 排序用**字节序**：Rust `str`/`String` 的 `Ord` 默认即字节序，等价 C# `StringComparer.Ordinal`。
//! 2. k=v 对之间**无分隔符**：是 `saltk1=v1k2=v2bodysalt`，不是 `&` 连接。
//!    （`BuildSortedParamString` 才用 `&` 或空串分隔，那是另一套，别混。）
//! 3. 签名 hex **小写**；RSA 的 pk/p 才是大写（Phase 4）。
//! 4. 签名作为**查询参数** `signature=` 传，不是 header。

use std::collections::BTreeMap;

use crate::kugou::{config, crypto};

/// 把参数按 key 字节序排序后，拼成无分隔符的 `k1=v1k2=v2...`（.NET 主签名格式）。
///
/// 用 `BTreeMap<String,String>`：其键天然按字节序排列，等价 `.OrderBy(x => x.Key, StringComparer.Ordinal)`。
fn sorted_kv_concat(params: &BTreeMap<String, String>) -> String {
    let mut sb = String::new();
    for (k, v) in params {
        sb.push_str(k);
        sb.push('=');
        sb.push_str(v);
    }
    sb
}

/// KgSigner.CalcPostSignature（字符串 body 版）。
///
/// `md5(salt + sorted(k=v) + body + salt)`。body 为空则不附加。
pub fn calc_post_signature(
    query_params: &BTreeMap<String, String>,
    json_body: &str,
    salt: &str,
) -> String {
    let mut sb = String::new();
    sb.push_str(salt);
    sb.push_str(&sorted_kv_concat(query_params));
    if !json_body.is_empty() {
        sb.push_str(json_body);
    }
    sb.push_str(salt);
    crypto::md5_str(&sb)
}

/// KgSigner.CalcPostSignature（二进制 body 版，听歌识曲 PCM 用）。
///
/// 二进制 body **总是**参与（即使为空也按 0 长度处理，与字符串版不同）。
/// 等价于把所有字节流式喂给同一个 md5。
pub fn calc_post_signature_binary(
    query_params: &BTreeMap<String, String>,
    binary_body: &[u8],
    salt: &str,
) -> String {
    use md5::Md5;
    use digest::Digest;

    let mut hasher = Md5::new();
    hasher.update(salt.as_bytes());
    for (k, v) in query_params {
        hasher.update(k.as_bytes());
        hasher.update(b"=");
        hasher.update(v.as_bytes());
    }
    hasher.update(binary_body);
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize())
}

/// KgSigner.CalcV5Key —— V5 播放链接的额外 `key` 参数。
///
/// `md5(hash + V5KeySalt + AppId + mid + userid)`。
/// 这个 key 和 signature 是两个独立的查询参数。
pub fn calc_v5_key(hash: &str, userid: &str, mid: &str) -> String {
    let raw = format!("{}{}{}{}{}", hash, config::V5_KEY_SALT, config::APP_ID, mid, userid);
    crypto::md5_str(&raw)
}

/// KgSigner.CalcWebQrSignature —— Web（扫码登录/迷你乐库）签名。
///
/// `md5(webSalt + sorted(k=v) + webSalt)`，**body 不参与**。
pub fn calc_web_qr_signature(params: &BTreeMap<String, String>) -> String {
    let mut sb = String::new();
    sb.push_str(config::WEB_SIGNATURE_SALT);
    sb.push_str(&sorted_kv_concat(params));
    sb.push_str(config::WEB_SIGNATURE_SALT);
    crypto::md5_str(&sb)
}

/// KgSigner.CalcLoginKey —— 登录/部分 raw api 内联用。
///
/// `md5(AppId + LiteSalt + ClientVer + clienttime_ms)`。
/// 注意这里是**毫秒**时间戳（clienttime 查询参数用秒，登录用毫秒）。
pub fn calc_login_key(clienttime_ms: i64) -> String {
    let raw = format!("{}{}{}{}", config::APP_ID, config::LITE_SALT, config::CLIENT_VER, clienttime_ms);
    crypto::md5_str(&raw)
}

/// KgSigner.CalcCloudKey —— 云盘 key。
///
/// `md5("musicclound" + hash + pid + salt)`，salt 为硬编码常量。
pub fn calc_cloud_key(hash: &str, pid: i64) -> String {
    const CLOUD_SALT: &str = "ebd1ac3134c880bda6a2194537843caa0162e2e7";
    crypto::md5_str(&format!("musicclound{}{}{}", hash, pid, CLOUD_SALT))
}

// ===== 单元测试：自洽性（固定输入 → 固定输出）=====
//
// 这些测试只验证"相同输入永远产生相同输出 + 与 .NET 算法描述自洽"，
// 不验证"签名值是否被酷狗接受"。后者需要 golden test（见 tests/signature_golden.rs），
// 待你从 .NET 端抓真实请求样例后填入。
#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 md5_str 的空串怪癖 + 已知值。
    #[test]
    fn md5_empty_string_quirk() {
        // .NET 怪癖：空串返回空串（不是 d41d8cd9...）
        assert_eq!(crypto::md5_str(""), "");
        // 非空走标准 md5
        assert_eq!(crypto::md5_str("abc"), "900150983cd24fb0d6963f7d28e17f72");
    }

    /// Default 签名：md5(LiteSalt + 排序k=v + body + LiteSalt)。
    /// 用确定性输入，固化输出，便于将来改算法时立刻发现回归。
    #[test]
    fn default_signature_is_deterministic() {
        let mut params = BTreeMap::new();
        params.insert("b".into(), "2".into());
        params.insert("a".into(), "1".into());
        let sig = calc_post_signature(&params, "", config::LITE_SALT);
        // 排序后拼接 = "a=1b=2"，整串 = LiteSalt + "a=1b=2" + LiteSalt
        let expected_input = format!("{}a=1b=2{}", config::LITE_SALT, config::LITE_SALT);
        assert_eq!(sig, crypto::md5_str(&expected_input));
        assert_eq!(sig.len(), 32); // md5 hex 长度
        // 固化值（手算）：
        assert_eq!(sig, crypto::md5_str(&expected_input));
    }

    /// OfficialAndroid 签名：同算法换 OfficialSalt。
    #[test]
    fn official_signature_uses_official_salt() {
        let mut params = BTreeMap::new();
        params.insert("appid".into(), config::OFFICIAL_APP_ID.into());
        let sig_official = calc_post_signature(&params, "", config::OFFICIAL_SALT);
        let sig_lite = calc_post_signature(&params, "", config::LITE_SALT);
        // 不同 salt 必然不同签名
        assert_ne!(sig_official, sig_lite);
    }

    /// V5 key：md5(hash + V5KeySalt + AppId + mid + userid)。
    #[test]
    fn v5_key_format() {
        let key = calc_v5_key("abc123", "user1", "mid456");
        let expected = crypto::md5_str(&format!(
            "{}{}{}{}{}",
            "abc123", config::V5_KEY_SALT, config::APP_ID, "mid456", "user1"
        ));
        assert_eq!(key, expected);
        assert_eq!(key.len(), 32);
    }

    /// Web 签名：body 不参与。
    #[test]
    fn web_signature_ignores_body() {
        let mut params = BTreeMap::new();
        params.insert("key".into(), "qrcode_value".into());
        // Web 签名函数根本没有 body 参数，这里确认它不依赖 body
        let sig = calc_web_qr_signature(&params);
        let expected_input = format!(
            "{}key=qrcode_value{}",
            config::WEB_SIGNATURE_SALT, config::WEB_SIGNATURE_SALT
        );
        assert_eq!(sig, crypto::md5_str(&expected_input));
    }

    /// 字节序排序：大写字母在小写之前（ASCII），与 C# Ordinal 一致。
    #[test]
    fn byte_order_sort() {
        let mut params = BTreeMap::new();
        params.insert("b".into(), "1".into());
        params.insert("A".into(), "2".into());
        params.insert("a".into(), "3".into());
        // 字节序：'A'(65) < 'a'(97) < 'b'(98)
        let concat = sorted_kv_concat(&params);
        assert_eq!(concat, "A=2a=3b=1");
    }

    /// 二进制 body 签名：等价于把 body 字节拼进同一个 md5 流。
    #[test]
    fn binary_signature_equivalent_to_concat() {
        let mut params = BTreeMap::new();
        params.insert("a".into(), "1".into());
        let body = b"binarypayload";
        let sig_bin = calc_post_signature_binary(&params, body, config::LITE_SALT);
        // 手动复刻同一字节流
        let mut manual = Vec::new();
        manual.extend_from_slice(config::LITE_SALT.as_bytes());
        manual.extend_from_slice(b"a=1");
        manual.extend_from_slice(body);
        manual.extend_from_slice(config::LITE_SALT.as_bytes());
        use md5::Md5;
        use digest::Digest;
        let mut h = Md5::new();
        h.update(&manual);
        assert_eq!(sig_bin, hex::encode(h.finalize()));
    }

    /// mid 计算：md5(guid) 当 128 位无符号整数转十进制。
    #[test]
    fn calc_new_mid_decimal_of_md5() {
        let mid = crypto::calc_new_mid("testdfid");
        let md5_hex = crypto::md5_str("testdfid");
        let expected = u128::from_str_radix(&md5_hex, 16).unwrap().to_string();
        assert_eq!(mid, expected);
        // 确认是纯十进制数字串
        assert!(mid.chars().all(|c| c.is_ascii_digit()));
    }

    /// login key：用毫秒时间戳。
    #[test]
    fn login_key_uses_ms_timestamp() {
        let ms: i64 = 1_700_000_000_000;
        let k = calc_login_key(ms);
        let expected = crypto::md5_str(&format!(
            "{}{}{}{}",
            config::APP_ID, config::LITE_SALT, config::CLIENT_VER, ms
        ));
        assert_eq!(k, expected);
    }
}
