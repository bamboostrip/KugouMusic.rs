//! 酷狗协议核心库 —— 对应 .NET 的 `KuGou.Net`。
//!
//! 这是整个代理的"引擎"，与 Web 框架解耦：它只负责"如何签名、如何注入会话、
//! 如何把请求发给上游酷狗、如何解包响应"。Web 层（controllers/services）只调用本库。
//!
//! 模块对应关系：
//! - [`config`]        ← `util/KuGouConfig.cs` + `util/Constants.cs`（硬编码常量）
//! - [`crypto`]        ← `util/KGUtils.cs` + `util/KGCrypto.cs`（md5/AES/KRC 等）
//! - [`signer`]        ← `util/KGSigner.cs`（4 种签名策略）
//! - [`session`]       ← `Protocol/Session/KgSession.cs` + `KgSessionManager`
//! - [`request`]       ← `Protocol/Transport/KgRequest.cs`（请求描述 + SignatureType）
//! - [`transport`]     ← `KgHttpTransport.cs` + `SignatureHandler.cs`（发送 + 注入）
//! - [`api_response`]  ← `Adapters/Common/KgApiResponseParser.cs`（status/data 解包）
//! - [`session_store`] ← `ISessionPersistence` + `KgSessionEntity`（SQLite 持久化）

pub mod api_response;
pub mod config;
pub mod crypto;
pub mod models;
pub mod request;
pub mod session;
pub mod session_store;
pub mod signer;
pub mod transport;
