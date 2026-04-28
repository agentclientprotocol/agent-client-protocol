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
//! notification types, plus serde plumbing and JSON Schema generation. For
//! the runtime pieces (transport, connection setup, the `Agent` / `Client`
//! traits, etc.) use the higher-level [`agent-client-protocol`] crate, which
//! builds on top of these types.
//!
//! [`agent-client-protocol`]: https://crates.io/crates/agent-client-protocol
//!
//! ## What's in this crate
//!
//! - Wire-format types for every ACP method: request, response, and
//!   notification structs grouped by which side handles them.
//! - JSON-RPC envelope and routing types: [`JsonRpcMessage`], [`Request`],
//!   [`Response`], [`Notification`], [`RequestId`], [`Error`].
//! - Aggregated routing enums: [`AgentRequest`], [`AgentResponse`],
//!   [`AgentNotification`], and the matching client-side trio used by SDK
//!   crates to dispatch incoming JSON-RPC messages.
//! - The `generate` binary that emits the published `schema.json`,
//!   `meta.json`, and the accompanying mdx documentation consumed by the
//!   protocol website and registry.
//!
//! ## Versioning
//!
//! The default surface re-exports the v1 (current stable) protocol types
//! directly at the crate root, so most consumers can write
//! `agent_client_protocol_schema::SessionId` (and so on) without thinking
//! about versions.
//!
//! For the complete protocol specification and documentation, visit
//! <https://agentclientprotocol.com>.

mod serde_util;
mod v1;
#[cfg(feature = "unstable_protocol_v2")]
pub mod v2;
mod version;

pub use serde_util::*;
pub use v1::*;
pub use version::*;
