//! [![Agent Client Protocol](https://zed.dev/img/acp/banner-dark.webp)](https://agentclientprotocol.com/)
//!
//! # Agent Client Protocol (ACP)
//!
//! The Agent Client Protocol standardizes communication between code editors
//! (IDEs, text-editors, etc.) and coding agents (programs that use generative AI
//! to autonomously modify code).
//!
//! ## Protocol & Transport
//!
//! ACP is a JSON-RPC based protocol. While clients typically start agents as
//! subprocesses and communicate over stdio (stdin/stdout), this crate is
//! transport-agnostic.
//!
//! You can use any bidirectional stream that implements `AsyncRead` and `AsyncWrite`.
//!
//! ## Core Components
//!
//! - **Agent**: Programs that use generative AI to autonomously modify code
//!   - See: [Agent](https://agentclientprotocol.com/protocol/overview#agent)
//! - **Client**: Code editors that provide the interface between users and agents
//!   - See: [Client](https://agentclientprotocol.com/protocol/overview#client)
//!
//! ## Getting Started
//!
//! To understand the protocol, start by exploring the [`Agent`] and [`Client`] traits,
//! which define the core methods and capabilities of each side of the connection.
//!
//! To see working examples of these traits in action, check out the
//! [agent](https://github.com/agentclientprotocol/rust-sdk/blob/main/examples/agent.rs)
//! and
//! [client](https://github.com/agentclientprotocol/rust-sdk/blob/main/examples/client.rs)
//! example binaries included with this crate.
//!
//! ### Implementation Pattern
//!
//! ACP uses a symmetric design where each participant implements one trait and
//! creates a connection that provides the complementary trait:
//!
//! - **Agent builders** implement the [`Agent`] trait to handle client requests
//!   (like initialization, authentication, and prompts). They pass this implementation
//!   to [`AgentSideConnection::new`], which returns a connection providing [`Client`]
//!   methods for requesting permissions and accessing the file system.
//!
//! - **Client builders** implement the [`Client`] trait to handle agent requests
//!   (like file system operations and permission checks). They pass this implementation
//!   to [`ClientSideConnection::new`], which returns a connection providing [`Agent`]
//!   methods for managing sessions and sending prompts.
//!
//! For the complete protocol specification and documentation, visit:
//! [https://agentclientprotocol.com](https://agentclientprotocol.com)

mod agent;
#[cfg(feature = "unstable_cancel_request")]
mod cancellation;
mod client;
mod content;
mod error;
mod ext;
mod plan;
mod rpc;
mod tool_call;
mod version;

pub use agent::*;
#[cfg(feature = "unstable_cancel_request")]
pub use cancellation::*;
pub use client::*;
pub use content::*;
use derive_more::{Display, From};
pub use error::*;
pub use ext::*;
pub use plan::*;
pub use rpc::*;
pub use serde_json::value::RawValue;
pub use tool_call::*;
pub use version::*;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

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
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// Utility trait for builder methods for optional values.
/// This allows the caller to either pass in the value itself without wrapping it in `Some`,
/// or to just pass in an Option if that is what they have.
pub trait IntoOption<T> {
    fn into_option(self) -> Option<T>;
}

impl<T> IntoOption<T> for Option<T> {
    fn into_option(self) -> Option<T> {
        self
    }
}

impl<T> IntoOption<T> for T {
    fn into_option(self) -> Option<T> {
        Some(self)
    }
}

impl IntoOption<String> for &str {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}

impl IntoOption<String> for &mut str {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}

impl IntoOption<String> for &String {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}

impl IntoOption<String> for Box<str> {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}

impl IntoOption<String> for Cow<'_, str> {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}

impl IntoOption<String> for Arc<str> {
    fn into_option(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl<T: ?Sized + AsRef<OsStr>> IntoOption<PathBuf> for &T {
    fn into_option(self) -> Option<PathBuf> {
        Some(self.into())
    }
}

impl IntoOption<PathBuf> for Box<Path> {
    fn into_option(self) -> Option<PathBuf> {
        Some(self.into())
    }
}

impl IntoOption<PathBuf> for Cow<'_, Path> {
    fn into_option(self) -> Option<PathBuf> {
        Some(self.into())
    }
}

impl IntoOption<serde_json::Value> for &str {
    fn into_option(self) -> Option<serde_json::Value> {
        Some(self.into())
    }
}

impl IntoOption<serde_json::Value> for String {
    fn into_option(self) -> Option<serde_json::Value> {
        Some(self.into())
    }
}

impl IntoOption<serde_json::Value> for Cow<'_, str> {
    fn into_option(self) -> Option<serde_json::Value> {
        Some(self.into())
    }
}
