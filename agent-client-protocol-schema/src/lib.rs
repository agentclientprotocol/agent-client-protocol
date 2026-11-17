#![cfg_attr(docsrs, feature(doc_cfg))]

//! [![Agent Client Protocol](https://zed.dev/img/acp/banner-dark.webp)](https://agentclientprotocol.com/)
//!
//! # Agent Client Protocol Schema
//!
//! Strongly-typed Rust definitions of the Agent Client Protocol (ACP) wire
//! format. ACP is a JSON-RPC based protocol that standardizes communication
//! between code editors (IDEs, text-editors, etc.) and coding agents
//! (programs that use generative AI to autonomously modify code).
//!
//! This crate is **only** the schema: the request, response, and
//! notification types, plus serde plumbing and optional JSON Schema generation.
//! For the runtime pieces (transport, connection setup, the `Agent` / `Client`
//! traits, etc.) use the higher-level [`agent-client-protocol`] crate, which
//! builds on top of these types.
//!
//! [`agent-client-protocol`]: https://crates.io/crates/agent-client-protocol
//!
//! ## What's in this crate
//!
//! - Versioned wire-format types for every ACP method: request, response, and
//!   notification structs grouped by which side handles them, currently under
//!   the [`v1`] module.
//! - JSON-RPC envelope and routing types: [`v1::JsonRpcMessage`],
//!   [`rpc::JsonRpcBatch`], [`v1::Request`], [`v1::Response`],
//!   [`v1::Notification`], [`v1::RequestId`], [`v1::Error`].
//! - Aggregated routing enums: [`v1::AgentRequest`], [`v1::AgentResponse`],
//!   [`v1::AgentNotification`], and the matching client-side trio used by SDK
//!   crates to dispatch incoming JSON-RPC messages.
//!
//! ## Parsing aggregate enums
//!
//! Do not deserialize an aggregate routing enum directly from a wire payload.
//! Its untagged `Deserialize` implementation selects by payload shape, not the
//! JSON-RPC method, and can silently select an unrelated variant.
//! Use the enum's `from_method_and_params` or `from_method_and_result` method
//! instead. Individual payload structs can be deserialized directly once the
//! method is known.
//!
//! These helpers only select and deserialize payloads. Use the SDK for
//! connections, request tracking, and JSON-RPC error handling.
//!
//! ```
//! use agent_client_protocol_schema::v1::ClientRequest;
//! use serde_json::json;
//!
//! let params = json!({"cwd": "/work", "mcpServers": []});
//! let request = ClientRequest::from_method_and_params("session/new", &params)?;
//! assert!(matches!(request, ClientRequest::NewSessionRequest(_)));
//! # Ok::<(), agent_client_protocol_schema::MethodDispatchError>(())
//! ```
//!
//! ## Cargo features
//!
//! The `schemars` feature implements `schemars::JsonSchema` for the protocol
//! types. It is enabled by default to preserve the existing default API.
//! Consumers that only need serialization can disable default features to omit
//! the dependency and those trait implementations.
//!
//! The `uuid` feature adds `IdempotencyKey::generate` to the `v2` module, which
//! creates a key from a random UUID. It is off by default.
//!
//! ## Versioning
//!
//! Stable protocol types are exposed through explicit version modules. For
//! example, use `agent_client_protocol_schema::v1::SessionId` for ACP protocol
//! version 1 types.
//!
//! For the complete protocol specification and documentation, visit
//! <https://agentclientprotocol.com>.

#[macro_use]
mod dispatch;
pub mod rpc;
mod serde_util;
pub mod v1;
#[cfg(feature = "unstable_protocol_v2")]
pub mod v2;
mod version;

// The `uuid` feature only adds API to the `v2` module. With `uuid` on and
// `unstable_protocol_v2` off, the dependency is enabled and nothing uses it.
// That combination is valid, so reference the crate to say so. Remove this
// once the `v2` module is no longer behind `unstable_protocol_v2`.
#[cfg(all(feature = "uuid", not(feature = "unstable_protocol_v2")))]
use uuid as _;

pub use dispatch::MethodDispatchError;
pub use serde_util::{IntoMaybeUndefined, IntoOption, MaybeUndefined};
pub use version::*;

#[cfg(test)]
mod serde_json_feature_tests {
    #[cfg(feature = "schemars")]
    use schemars::JsonSchema;
    use serde_json::Value;

    #[cfg(feature = "schemars")]
    #[test]
    fn protocol_types_implement_json_schema_when_enabled() {
        fn assert_json_schema<T: JsonSchema>() {}

        assert_json_schema::<crate::ProtocolVersion>();
        assert_json_schema::<crate::v1::InitializeRequest>();
    }

    #[test]
    fn serde_json_values_preserve_object_key_order() {
        let Value::Object(object) =
            serde_json::from_str::<Value>(r#"{"z":1,"a":2,"m":3}"#).unwrap()
        else {
            panic!("expected JSON object");
        };

        let keys = object.keys().map(String::as_str).collect::<Vec<_>>();
        assert_eq!(keys, ["z", "a", "m"]);
    }
}
