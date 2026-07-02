//! 会话持久化 —— 对应 .NET 的 `ISessionPersistence` + `KgWebSessionPersistence` +
//! `KgSessionEntity`。
//!
//! 按 `session_key` 从 SQLite `kg_sessions` 表加载/保存一个 [`KgSession`]。
//!
//! 行为对齐 .NET：**所有 DB 操作吞掉异常**（失败时 load 返回 None/默认值，save
//! 静默 no-op），让 DB 故障绝不打断业务请求。这与 .NET 的 try/catch 语义一致。

use sqlx::SqlitePool;

use crate::kugou::session::KgSession;

/// 按 session_key 加载会话；不存在或出错时返回 None（上层会用 default + normalize）。
pub async fn load(pool: &SqlitePool, session_key: &str) -> Option<KgSession> {
    // 用 sqlx::query_as + FromRow 风格；表列名与 KgSession 字段一一对应。
    let row: Result<Option<SessionRow>, sqlx::Error> = sqlx::query_as(
        r#"SELECT session_key, user_id, token, vip_type, vip_token,
                  dfid, mid, uuid, install_dev, install_mac, install_guid, t1
           FROM kg_sessions WHERE session_key = ?"#,
    )
    .bind(session_key)
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some(r)) => Some(KgSession {
            userid: r.user_id,
            token: r.token,
            vip_type: r.vip_type,
            vip_token: r.vip_token,
            dfid: r.dfid,
            mid: r.mid,
            uuid: r.uuid,
            install_dev: r.install_dev,
            install_mac: r.install_mac,
            install_guid: r.install_guid,
            t1: r.t1,
        }),
        Ok(None) => None,
        Err(e) => {
            // 与 .NET 一致：吞掉错误，仅记日志
            tracing::warn!(error = %e, session_key, "加载会话失败，回退到默认会话");
            None
        }
    }
}

/// 保存（upsert）会话。失败时静默（仅记日志）。
pub async fn save(pool: &SqlitePool, session_key: &str, s: &KgSession) {
    let res = sqlx::query(
        r#"INSERT INTO kg_sessions
             (session_key, user_id, token, vip_type, vip_token,
              dfid, mid, uuid, install_dev, install_mac, install_guid, t1, updated_at_utc)
           VALUES (?,?,?,?,?,?,?,?,?,?,?,?, datetime('now'))
           ON CONFLICT(session_key) DO UPDATE SET
             user_id=excluded.user_id, token=excluded.token, vip_type=excluded.vip_type,
             vip_token=excluded.vip_token, dfid=excluded.dfid, mid=excluded.mid,
             uuid=excluded.uuid, install_dev=excluded.install_dev,
             install_mac=excluded.install_mac, install_guid=excluded.install_guid,
             t1=excluded.t1, updated_at_utc=datetime('now')"#,
    )
    .bind(session_key)
    .bind(&s.userid)
    .bind(&s.token)
    .bind(&s.vip_type)
    .bind(&s.vip_token)
    .bind(&s.dfid)
    .bind(&s.mid)
    .bind(&s.uuid)
    .bind(&s.install_dev)
    .bind(&s.install_mac)
    .bind(&s.install_guid)
    .bind(&s.t1)
    .execute(pool)
    .await;

    if let Err(e) = res {
        tracing::warn!(error = %e, session_key, "保存会话失败（已忽略）");
    }
}

/// 删除会话（登出用）。失败时静默。
pub async fn clear(pool: &SqlitePool, session_key: &str) {
    let res = sqlx::query("DELETE FROM kg_sessions WHERE session_key = ?")
        .bind(session_key)
        .execute(pool)
        .await;
    if let Err(e) = res {
        tracing::warn!(error = %e, session_key, "删除会话失败（已忽略）");
    }
}

/// DB 行映射（字段顺序与 SELECT 一致）。sqlx 的 query_as（非宏版）按名解码。
#[derive(Debug, Default, sqlx::FromRow)]
struct SessionRow {
    #[allow(dead_code)]
    session_key: String,
    user_id: String,
    token: String,
    vip_type: String,
    vip_token: String,
    dfid: String,
    mid: String,
    uuid: String,
    install_dev: String,
    install_mac: String,
    install_guid: String,
    t1: String,
}
