//! Agent Client Protocol version 2 draft types.
//!
//! **EXPERIMENTAL.** This module is gated behind the `unstable_protocol_v2`
//! feature, is not part of the [`unstable`] umbrella, and is **not**
//! advertised by [`crate::ProtocolVersion::LATEST`]. The wire format is
//! currently identical to v1 (the default crate-root types) and the types
//! here exist only as a place to evolve v2 without disturbing the stable v1
//! API. Both the type definitions and the [`conversion`] helpers may change
//! at any time.
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
#[cfg(feature = "unstable_nes")]
mod nes;
mod plan;
#[cfg(feature = "unstable_cancel_request")]
mod protocol_level;
mod tool_call;

pub use crate::rpc::{JsonRpcMessage, Notification, Request, RequestId};
pub use agent::*;
pub use client::*;
pub use content::*;
use derive_more::{Display, From};
#[cfg(feature = "unstable_elicitation")]
pub use elicitation::*;
pub use error::*;
pub use ext::*;
#[cfg(feature = "unstable_nes")]
pub use nes::*;
pub use plan::*;
#[cfg(feature = "unstable_cancel_request")]
pub use protocol_level::*;
pub use serde_json::value::RawValue;
pub use tool_call::*;

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
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}
