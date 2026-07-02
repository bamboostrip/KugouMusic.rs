//! Golden test —— 签名正确性的"最终裁决"。
//!
//! 这里的测试**默认 `#[ignore]`**，因为它们的期望值必须来自 .NET 端的**真实请求**，
//! 而不是手算。等你有真实样例后，按下面的格式填入并取消 ignore。
//!
//! ## 怎么抓真实样例（在 .NET 项目里）
//! 1. 在 `KgSignatureHandler.SendAsync` 签名计算之后、`base.SendAsync` 之前打断点，
//!    或者临时 `Console.Error.WriteLine($"[GOLDEN] path={...} params={...} signature={...}")`。
//! 2. 跑一遍 search（验证 Default）和 song/url（验证 V5）。
//! 3. 把抓到的 params（全量 key=value）+ 最终 signature 抄进下面的用例。
//!
//! ## 怎么跑
//! ```
//! cargo test --test signature_golden -- --ignored
//! ```
//! 全绿 = Rust 签名与 .NET 完全一致。这是进入 Phase 1（接真实端点）前的关键闸门。

use kugou_web_api::kugou::{config, crypto, signer};
use std::collections::BTreeMap;

/// 模板：Default 签名 golden（来自 .NET 真实请求）。
/// 填好后把 `# [ignore]` 行删掉。
#[test]
#[ignore = "待填入 .NET 真实样例：params 全量 + 期望 signature"]
fn golden_default_signature() {
    // TODO: 从 .NET 抓一条 /v3/search/song 的真实请求填进来。
    // 例：params 含 keywords/page/pagesize/appid/clientver/dfid/mid/uuid/userid/clienttime
    let mut params = BTreeMap::new();
    params.insert("keywords".into(), "周杰伦".into());
    params.insert("page".into(), "1".into());
    // ...其它参数...
    let json_body = ""; // GET 请求 body 为空

    let sig = signer::calc_post_signature(&params, json_body, config::LITE_SALT);

    // TODO: 把 .NET 抓到的真实 signature 填进来
    let expected = "TODO_PASTE_DOTNET_SIGNATURE_HERE";
    assert_eq!(sig, expected, "Default 签名与 .NET 不一致");
}

/// 模板：V5 签名 golden（来自 .NET /v5/url 真实请求）。
#[test]
#[ignore = "待填入 .NET 真实样例"]
fn golden_v5_signature_and_key() {
    // V5 端点：Default 签名 + 额外 key 参数。
    // 需要同时验证 signature 和 key 两个值都对。
    let mut params = BTreeMap::new();
    params.insert("hash".into(), "TODO_REAL_HASH".into());
    // ...其它参数...

    let sig = signer::calc_post_signature(&params, "", config::LITE_SALT);
    let key = signer::calc_v5_key("TODO_REAL_HASH", "TODO_REAL_USERID", "TODO_REAL_MID");

    assert_eq!(sig, "TODO_PASTE_DOTNET_SIGNATURE");
    assert_eq!(key, "TODO_PASTE_DOTNET_KEY");
}

/// 模板：Web 签名 golden（来自 .NET 扫码登录 /v2/qrcode）。
#[test]
#[ignore = "待填入 .NET 真实样例"]
fn golden_web_qr_signature() {
    let mut params = BTreeMap::new();
    params.insert("qrcode".into(), "TODO".into());
    // ...

    let sig = signer::calc_web_qr_signature(&params);
    assert_eq!(sig, "TODO_PASTE_DOTNET_SIGNATURE");
}

/// 非 ignore 的 sanity：确认签名函数可被外部调用 + md5 空串怪癖对外可见。
#[test]
fn sanity_signer_accessible_from_integration_test() {
    assert_eq!(crypto::md5_str(""), "");
    let mut p = BTreeMap::new();
    p.insert("a".into(), "1".into());
    let s = signer::calc_post_signature(&p, "", config::LITE_SALT);
    assert_eq!(s.len(), 32);
}
