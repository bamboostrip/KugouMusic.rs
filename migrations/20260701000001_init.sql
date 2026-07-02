-- 初始化：对应 .NET 项目里 KgWebApiDbContext 的两张表
-- 说明：绝大多数业务（搜索/歌曲/歌词/歌单...）只是代理上游酷狗，不落库。
--       数据库只承担两件事：1) 按 session key 存放酷狗登录凭证  2) App 版本下发。

-- 会话凭证（对应 .NET 的 KgSessions / KgSessionEntity 表）
CREATE TABLE IF NOT EXISTS kg_sessions (
    session_key     TEXT PRIMARY KEY NOT NULL,           -- X-Kg-Session-Id / kg_sid
    user_id         TEXT NOT NULL DEFAULT '0',
    token           TEXT NOT NULL DEFAULT '',
    vip_type        TEXT NOT NULL DEFAULT '0',
    vip_token       TEXT NOT NULL DEFAULT '',
    dfid            TEXT NOT NULL DEFAULT '-',
    mid             TEXT NOT NULL DEFAULT '-',
    uuid            TEXT NOT NULL DEFAULT '-',
    install_dev     TEXT NOT NULL DEFAULT '',
    install_mac     TEXT NOT NULL DEFAULT '',
    install_guid    TEXT NOT NULL DEFAULT '',
    t1              TEXT NOT NULL DEFAULT '',
    updated_at_utc  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- App 版本下发（对应 .NET 的 AppVersions 表）
CREATE TABLE IF NOT EXISTS app_versions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    platform        TEXT NOT NULL,                       -- android / ios / ...
    version_name    TEXT NOT NULL,
    version_code    INTEGER NOT NULL,
    update_content  TEXT NOT NULL DEFAULT '',
    download_url    TEXT NOT NULL DEFAULT '',
    force_update    INTEGER NOT NULL DEFAULT 0,          -- 0/1
    release_date    TEXT NOT NULL DEFAULT '',
    created_at_utc  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_app_versions_platform
    ON app_versions (platform, version_code DESC);
