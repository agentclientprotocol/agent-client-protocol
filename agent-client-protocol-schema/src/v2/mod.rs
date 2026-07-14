//! Agent Client Protocol version 2 draft types.
//!
//! **EXPERIMENTAL.** This module is gated behind the `unstable_protocol_v2`
//! feature, is not part of the [`unstable`] umbrella, and must be selected
//! explicitly with [`crate::ProtocolVersion::V2`]. The wire format is
//! currently identical to v1 (the default crate-root types) and the types here
//! exist only as a place to evolve v2 without disturbing the stable v1 API. The
//! wire format intentionally diverges from v1 as draft v2 RFDs land. Both the
//! type definitions and the [`conversion`] helpers may change at any time.
//!
//! [`unstable`]: https://docs.rs/crate/agent-client-protocol-schema/latest/features

mod agent;
mod client;
mod content;
pub mod conversion;
#[cfg(feature = "unstable_elicitation")]
mod elicitation;
mod error;
mod ext;
#[cfg(feature = "unstable_mcp_over_acp")]
mod mcp;
#[cfg(feature = "unstable_nes")]
mod nes;
mod plan;
mod protocol_level;
pub(crate) mod schema_util;
mod terminal;
mod tool_call;

pub use crate::rpc::{JsonRpcBatch, JsonRpcMessage, Notification, Request, RequestId};
pub use agent::*;
pub use client::*;
pub use content::*;
use derive_more::{Display, From};
#[cfg(feature = "unstable_elicitation")]
pub use elicitation::*;
pub use error::*;
pub use ext::*;
#[cfg(feature = "unstable_mcp_over_acp")]
pub use mcp::*;
#[cfg(feature = "unstable_nes")]
pub use nes::*;
pub use plan::*;
pub use protocol_level::*;
pub use serde_json::value::RawValue;
pub use terminal::*;
pub use tool_call::*;

/// JSON-RPC response envelope using this protocol version's error type.
pub type Response<Result> = crate::rpc::Response<Result, Error>;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A unique identifier for a conversation session between a client and agent.
///
/// Sessions maintain their own context, conversation history, and state,
/// allowing multiple independent interactions with the same agent.
///
/// See protocol docs: [Session ID](https://agentclientprotocol.com/protocol/session-setup#session-id)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct SessionId(pub Arc<str>);

impl SessionId {
    /// Wraps a protocol string as a typed [`SessionId`].
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}
