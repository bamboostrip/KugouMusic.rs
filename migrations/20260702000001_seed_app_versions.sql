-- 种子数据：App 版本下发（对应 .NET AppVersions 表的初始数据）
-- 各平台填写实际 APK/IPA 发布信息；下载地址按需修改。

INSERT OR IGNORE INTO app_versions
    (platform, version_name, version_code, update_content, download_url, force_update, release_date)
VALUES
    ('android', '1.0.0', 1000000, '首次发布', '', 0, date('now')),
    ('ios',     '1.0.0', 1000000, '首次发布', '', 0, date('now'));
