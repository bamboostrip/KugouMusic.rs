//! 歌曲业务层 —— 对应 .NET 的 `SongClient` + `RawSongApi`。
//!
//! Phase 1 先实现播放链接（song/url，V5 签名）这一最关键端点，验证最难的签名链路。
//! 其余歌曲端点（audio/related、climax、ranking 等）随后续 Phase 补。

use serde_json::Value;

use crate::error::AppResult;
use crate::kugou::{
    crypto,
    models::PlayUrlData,
    request::{KgRequest, SignatureType},
    session::KgSession,
    transport,
};
use crate::state::AppState;

/// 特殊音质前缀（与 .NET GetUrlAsync 的 normalizedQuality 逻辑一致）。
const MAGIC_QUALITIES: &[&str] = &["piano", "acappella", "subwoofer", "ancient", "dj", "surnay"];

/// 归一化音质：特殊类型加 `magic_` 前缀，否则原样（空则默认 128）。
fn normalize_quality(quality: Option<&str>) -> String {
    match quality {
        Some(q) if MAGIC_QUALITIES.contains(&q) => format!("magic_{q}"),
        Some(q) if !q.is_empty() => q.to_string(),
        _ => "128".to_string(),
    }
}

/// 获取播放链接（song/url）。对应 RawSongApi.GetUrlAsync。
///
/// 关键：SignatureType::V5 → transport 自动注入 `key` 参数；
/// 当 session dfid 为 `-` 时生成临时 24 字符 dfid（SpecificDfid）。
pub async fn get_play_url(
    state: &AppState,
    session: &KgSession,
    hash: &str,
    quality: Option<&str>,
    album_id: Option<&str>,
    album_audio_id: Option<&str>,
    free_part: bool,
) -> AppResult<Value> {
    // dfid：session 没有真实 dfid 时临时生成 24 字符（与 .NET 一致）
    let dfid = if session.dfid.trim().is_empty() || session.dfid == "-" {
        crypto::random_string(24)
    } else {
        session.dfid.clone()
    };
    let normalized_quality = normalize_quality(quality);

    let req = KgRequest::get("/v5/url")
        .param("album_id", album_id.unwrap_or("0"))
        .param("area_code", "1")
        .param("hash", hash.to_lowercase())
        .param("ssa_flag", "is_fromtrack")
        .param("version", "11430")
        .param("page_id", "967177915")
        .param("quality", normalized_quality)
        .param("album_audio_id", album_audio_id.unwrap_or("0"))
        .param("behavior", "play")
        .param("pid", "411")
        .param("cmd", "26")
        .param("pidversion", "3001")
        .param("IsFreePart", if free_part { "1" } else { "0" })
        .param("ppage_id", "356753938,823673182,967485191")
        .param("cdnBackup", "1")
        .param("module", "")
        .param("clientver", "11430") // 注意这里是 11430，与全局 CLIENT_VER(11440) 不同
        .router("trackercdn.kugou.com")
        .signature_type(SignatureType::V5)
        .specific_dfid(dfid);

    transport::send(&state.http, session, &req).await
}

/// 获取播放链接并类型化为 PlayUrlData。对应 SongClient.GetPlayInfoAsync。
/// 失败（status != 1）时上游错误信息已在 transport 里被记 warn，这里返回解包后的 data。
pub async fn get_play_info(
    state: &AppState,
    session: &KgSession,
    hash: &str,
    quality: Option<&str>,
    album_id: Option<&str>,
    album_audio_id: Option<&str>,
    free_part: bool,
) -> AppResult<PlayUrlData> {
    let v = get_play_url(
        state, session, hash, quality, album_id, album_audio_id, free_part,
    )
    .await?;
    let data: PlayUrlData = serde_json::from_value(v).map_err(|e| {
        crate::error::AppError::Internal(anyhow::anyhow!("解析播放链接结果失败: {e}"))
    })?;
    Ok(data)
}
