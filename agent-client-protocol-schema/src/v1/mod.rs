//! Agent Client Protocol version 1 types.

mod agent;
mod client;
mod content;
mod elicitation;
mod error;
mod ext;
#[cfg(feature = "unstable_mcp_over_acp")]
mod mcp;
mod plan;
mod protocol_level;
#[cfg(feature = "unstable_proxy_chains")]
mod proxy;
mod rpc;
mod tool_call;

pub use crate::rpc::{JsonRpcBatch, JsonRpcMessage, Notification};
pub use agent::*;
pub use client::*;
pub use content::*;
use derive_more::{Display, From};
pub use elicitation::*;
pub use error::*;
pub use ext::*;
#[cfg(feature = "unstable_mcp_over_acp")]
pub use mcp::*;
pub use plan::*;
pub use protocol_level::*;
#[cfg(feature = "unstable_proxy_chains")]
pub use proxy::*;
pub use rpc::*;
pub use serde_json::value::RawValue;
pub use tool_call::*;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A unique identifier for a conversation session between a client and agent.
///
/// Each session has its own context, conversation history, and state, so one
/// agent can run several independent sessions.
///
/// See protocol docs: [Session ID](https://agentclientprotocol.com/protocol/v1/session-setup#session-id)
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display, From)]
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
