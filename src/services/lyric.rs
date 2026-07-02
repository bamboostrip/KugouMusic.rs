//! 歌词业务层 —— 对应 .NET 的 `LyricClient` + `RawLyricApi`。
//!
//! 两个端点：搜索歌词（拿 id+accesskey）、下载歌词（可选 KRC 解码）。
//! 歌词解码逻辑（KRC/base64）在 `kugou::crypto::decode_lyrics`。

use base64::Engine;
use serde_json::{json, Value};

use crate::error::AppResult;
use crate::kugou::{
    crypto,
    request::{KgRequest, SignatureType},
    session::KgSession,
    transport,
};
use crate::state::AppState;

const LYRIC_HOST: &str = "https://lyrics.kugou.com";

/// 搜索歌词（获取 id 和 accesskey）。对应 RawLyricApi.SearchLyricAsync。
pub async fn search_lyric(
    state: &AppState,
    session: &KgSession,
    hash: Option<&str>,
    album_audio_id: Option<&str>,
    keyword: Option<&str>,
    man: Option<&str>,
) -> AppResult<Value> {
    let req = KgRequest::get("/v1/search")
        .base_url(LYRIC_HOST)
        .param("album_audio_id", album_audio_id.unwrap_or("0"))
        .param("duration", "0")
        .param("hash", hash.unwrap_or(""))
        .param("keyword", keyword.unwrap_or(""))
        .param("lrctxt", "1")
        .param("man", man.unwrap_or("no"))
        .signature_type(SignatureType::Default);
    transport::send(&state.http, session, &req).await
}

/// 下载歌词并（可选）解码。对应 LyricClient.GetLyricAsync + RawLyricApi.DownloadLyricAsync。
///
/// 返回形如 `{ raw_content, decoded_content, decoded_translation, raw }`，与 .NET LyricResult 一致。
pub async fn get_lyric(
    state: &AppState,
    session: &KgSession,
    id: &str,
    accesskey: &str,
    fmt: &str,
    decode: bool,
) -> AppResult<Value> {
    let req = KgRequest::get("/download")
        .base_url(LYRIC_HOST)
        .param("ver", "1")
        .param("client", "android")
        .param("id", id)
        .param("accesskey", accesskey)
        .param("fmt", fmt)
        .param("charset", "utf8")
        .signature_type(SignatureType::Default);
    let raw = transport::send(&state.http, session, &req).await?;

    let raw_content = raw.get("content").and_then(|v| v.as_str()).map(|s| s.to_string());
    let (decoded_content, decoded_trans) = if decode {
        // content 解码：fmt=lrc 或 contenttype!=0 → 直接 base64；否则 KRC 解码
        let content_type = raw.get("contenttype").and_then(|v| v.as_i64()).unwrap_or(0);
        let decoded_content = raw_content.as_deref().and_then(|b64| {
            if b64.is_empty() {
                None
            } else if fmt == "lrc" || content_type != 0 {
                base64_decode_utf8(b64).ok()
            } else {
                Some(crypto::decode_lyrics(b64))
            }
        });
        let decoded_trans = raw
            .get("trans")
            .and_then(|v| v.as_str())
            .and_then(|s| base64_decode_utf8(s).ok());
        (decoded_content, decoded_trans)
    } else {
        (None, None)
    };

    Ok(json!({
        "raw_content": raw_content,
        "decoded_content": decoded_content,
        "decoded_translation": decoded_trans,
        "raw": raw,
    }))
}

fn base64_decode_utf8(s: &str) -> Result<String, ()> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(s).map_err(|_| ())?;
    String::from_utf8(bytes).map_err(|_| ())
}
