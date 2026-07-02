//! 上游响应解包 —— 对应 .NET 的 `Adapters/Common/KgApiResponseParser.cs`。
//!
//! 酷狗响应形如：
//! ```json
//! { "status": 1, "error_code": 0, "data": { ...业务字段... } }
//! ```
//! 解包规则（与 .NET 严格一致）：
//! 1. 成功 = `status==1` **且** (`error_code` 不存在 **或** ==0)（`errcode` 作为 `error_code` 的别名）。
//! 2. 成功时把 `data` 提升为根节点（透传/反序列化都基于提升后的值）。

use serde_json::Value;

/// 解包结果：成功时给出提升后的 `data`（无 data 则原样），失败时给出错误码+消息。
#[derive(Debug)]
pub enum ParsedResponse {
    /// 成功：携带提升后的节点（成功且有 data 则是 data，否则是整个 root）
    Success(Value),
    /// 失败：root 的 status/error_code 与原始 root（便于上层报错）
    Failure { status: Option<i64>, err_code: Option<i64>, root: Value },
}

/// 解析酷狗响应（对应 KgApiResponseParser.Parse，但只做透传语义）。
///
/// 业务层若需要强类型，可拿到 `Success(Value)` 后自行 `serde_json::from_value`。
pub fn parse(root: Value) -> ParsedResponse {
    let root_status = root.get("status").and_then(|v| v.as_i64());
    let root_err_code = root
        .get("error_code")
        .and_then(|v| v.as_i64())
        .or_else(|| root.get("errcode").and_then(|v| v.as_i64()));

    let is_success =
        root_status == Some(1) && matches!(root_err_code, None | Some(0));

    if is_success {
        // 成功：提升 data
        if let Some(data) = root.get("data")
            && !data.is_null()
        {
            return ParsedResponse::Success(data.clone());
        }
        ParsedResponse::Success(root)
    } else {
        ParsedResponse::Failure {
            status: root_status,
            err_code: root_err_code,
            root,
        }
    }
}

/// 判断响应是否成功（便捷封装）。
pub fn is_success(root: &Value) -> bool {
    let status = root.get("status").and_then(|v| v.as_i64());
    let err = root
        .get("error_code")
        .and_then(|v| v.as_i64())
        .or_else(|| root.get("errcode").and_then(|v| v.as_i64()));
    status == Some(1) && matches!(err, None | Some(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn success_promotes_data() {
        let root = json!({ "status": 1, "error_code": 0, "data": { "lists": [1, 2, 3] } });
        match parse(root) {
            ParsedResponse::Success(v) => assert_eq!(v, json!({ "lists": [1, 2, 3] })),
            _ => panic!("应成功"),
        }
    }

    #[test]
    fn success_without_data_returns_root() {
        let root = json!({ "status": 1 });
        match parse(root) {
            ParsedResponse::Success(v) => assert_eq!(v["status"], json!(1)),
            _ => panic!("应成功"),
        }
    }

    #[test]
    fn failure_with_error_code() {
        let root = json!({ "status": 0, "error_code": 9001, "err": "缺参" });
        match parse(root) {
            ParsedResponse::Failure { status, err_code, .. } => {
                assert_eq!(status, Some(0));
                assert_eq!(err_code, Some(9001));
            }
            _ => panic!("应失败"),
        }
    }

    #[test]
    fn errcode_alias_recognized() {
        // status==1 但 errcode!=0 => 失败
        let root = json!({ "status": 1, "errcode": 5 });
        assert!(matches!(parse(root), ParsedResponse::Failure { .. }));
    }

    #[test]
    fn status_one_err_absent_is_success() {
        let root = json!({ "status": 1, "data": "x" });
        assert!(matches!(parse(root), ParsedResponse::Success(_)));
    }
}
