//! Methods and notifications the agent handles/receives.
//!
//! This module defines the Agent trait and all associated types for implementing
//! an AI coding agent that follows the Agent Client Protocol (ACP).

use std::{path::PathBuf, sync::Arc};

use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ClientCapabilities, ContentBlock, ExtNotification, ExtRequest, ExtResponse, IntoOption, Meta,
    ProtocolVersion, SessionId,
};

// Initialize

/// Request parameters for the initialize method.
///
/// Sent by the client to establish connection and negotiate capabilities.
///
/// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = INITIALIZE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct InitializeRequest {
    /// The latest protocol version supported by the client.
    pub protocol_version: ProtocolVersion,
    /// Capabilities supported by the client.
    #[serde(default)]
    pub client_capabilities: ClientCapabilities,
    /// Information about the Client name and version sent to the Agent.
    ///
    /// Note: in future versions of the protocol, this will be required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_info: Option<Implementation>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl InitializeRequest {
    #[must_use]
    pub fn new(protocol_version: ProtocolVersion) -> Self {
        Self {
            protocol_version,
            client_capabilities: ClientCapabilities::default(),
            client_info: None,
            meta: None,
        }
    }

    /// Capabilities supported by the client.
    #[must_use]
    pub fn client_capabilities(mut self, client_capabilities: ClientCapabilities) -> Self {
        self.client_capabilities = client_capabilities;
        self
    }

    /// Information about the Client name and version sent to the Agent.
    #[must_use]
    pub fn client_info(mut self, client_info: impl IntoOption<Implementation>) -> Self {
        self.client_info = client_info.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Response to the `initialize` method.
///
/// Contains the negotiated protocol version and agent capabilities.
///
/// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = INITIALIZE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct InitializeResponse {
    /// The protocol version the client specified if supported by the agent,
    /// or the latest protocol version supported by the agent.
    ///
    /// The client should disconnect, if it doesn't support this version.
    pub protocol_version: ProtocolVersion,
    /// Capabilities supported by the agent.
    #[serde(default)]
    pub agent_capabilities: AgentCapabilities,
    /// Authentication methods supported by the agent.
    #[serde(default)]
    pub auth_methods: Vec<AuthMethod>,
    /// Information about the Agent name and version sent to the Client.
    ///
    /// Note: in future versions of the protocol, this will be required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_info: Option<Implementation>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl InitializeResponse {
    #[must_use]
    pub fn new(protocol_version: ProtocolVersion) -> Self {
        Self {
            protocol_version,
            agent_capabilities: AgentCapabilities::default(),
            auth_methods: vec![],
            agent_info: None,
            meta: None,
        }
    }

    /// Capabilities supported by the agent.
    #[must_use]
    pub fn agent_capabilities(mut self, agent_capabilities: AgentCapabilities) -> Self {
        self.agent_capabilities = agent_capabilities;
        self
    }

    /// Authentication methods supported by the agent.
    #[must_use]
    pub fn auth_methods(mut self, auth_methods: Vec<AuthMethod>) -> Self {
        self.auth_methods = auth_methods;
        self
    }

    /// Information about the Agent name and version sent to the Client.
    #[must_use]
    pub fn agent_info(mut self, agent_info: impl IntoOption<Implementation>) -> Self {
        self.agent_info = agent_info.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Metadata about the implementation of the client or agent.
/// Describes the name and version of an MCP implementation, with an optional
/// title for UI representation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Implementation {
    /// Intended for programmatic or logical use, but can be used as a display
    /// name fallback if title isn’t present.
    pub name: String,
    /// Intended for UI and end-user contexts — optimized to be human-readable
    /// and easily understood.
    ///
    /// If not provided, the name should be used for display.
    pub title: Option<String>,
    /// Version of the implementation. Can be displayed to the user or used
    /// for debugging or metrics purposes. (e.g. "1.0.0").
    pub version: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Implementation {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            title: None,
            version: version.into(),
            meta: None,
        }
    }

    /// Intended for UI and end-user contexts — optimized to be human-readable
    /// and easily understood.
    ///
    /// If not provided, the name should be used for display.
    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// Authentication

/// Request parameters for the authenticate method.
///
/// Specifies which authentication method to use.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = AUTHENTICATE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AuthenticateRequest {
    /// The ID of the authentication method to use.
    /// Must be one of the methods advertised in the initialize response.
    pub method_id: AuthMethodId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl AuthenticateRequest {
    #[must_use]
    pub fn new(method_id: impl Into<AuthMethodId>) -> Self {
        Self {
            method_id: method_id.into(),
            meta: None,
        }
    }

    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Response to the `authenticate` method.
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "agent", "x-method" = AUTHENTICATE_METHOD_NAME))]
#[non_exhaustive]
pub struct AuthenticateResponse {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl AuthenticateResponse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct AuthMethodId(pub Arc<str>);

impl AuthMethodId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// Describes an available authentication method.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AuthMethod {
    /// Unique identifier for this authentication method.
    pub id: AuthMethodId,
    /// Human-readable name of the authentication method.
    pub name: String,
    /// Optional description providing more details about this authentication method.
    pub description: Option<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl AuthMethod {
    pub fn new(id: impl Into<AuthMethodId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            meta: None,
        }
    }

    /// Optional description providing more details about this authentication method.
    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// New session

/// Request parameters for creating a new session.
///
/// See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_NEW_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NewSessionRequest {
    /// The working directory for this session. Must be an absolute path.
    pub cwd: PathBuf,
    /// List of MCP (Model Context Protocol) servers the agent should connect to.
    pub mcp_servers: Vec<McpServer>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl NewSessionRequest {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self {
            cwd: cwd.into(),
            mcp_servers: vec![],
            meta: None,
        }
    }

    /// List of MCP (Model Context Protocol) servers the agent should connect to.
    #[must_use]
    pub fn mcp_servers(mut self, mcp_servers: Vec<McpServer>) -> Self {
        self.mcp_servers = mcp_servers;
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Response from creating a new session.
///
/// See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_NEW_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NewSessionResponse {
    /// Unique identifier for the created session.
    ///
    /// Used in all subsequent requests for this conversation.
    pub session_id: SessionId,
    /// Initial mode state if supported by the Agent
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modes: Option<SessionModeState>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial model state if supported by the Agent
    #[cfg(feature = "unstable_session_model")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<SessionModelState>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial session configuration options if supported by the Agent.
    #[cfg(feature = "unstable_session_config_options")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_options: Option<Vec<SessionConfigOption>>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl NewSessionResponse {
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>) -> Self {
        Self {
            session_id: session_id.into(),
            modes: None,
            #[cfg(feature = "unstable_session_model")]
            models: None,
            #[cfg(feature = "unstable_session_config_options")]
            config_options: None,
            meta: None,
        }
    }

    /// Initial mode state if supported by the Agent
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    #[must_use]
    pub fn modes(mut self, modes: impl IntoOption<SessionModeState>) -> Self {
        self.modes = modes.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial model state if supported by the Agent
    #[cfg(feature = "unstable_session_model")]
    #[must_use]
    pub fn models(mut self, models: impl IntoOption<SessionModelState>) -> Self {
        self.models = models.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial session configuration options if supported by the Agent.
    #[cfg(feature = "unstable_session_config_options")]
    #[must_use]
    pub fn config_options(
        mut self,
        config_options: impl IntoOption<Vec<SessionConfigOption>>,
    ) -> Self {
        self.config_options = config_options.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// Load session

/// Request parameters for loading an existing session.
///
/// Only available if the Agent supports the `loadSession` capability.
///
/// See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_LOAD_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LoadSessionRequest {
    /// List of MCP servers to connect to for this session.
    pub mcp_servers: Vec<McpServer>,
    /// The working directory for this session.
    pub cwd: PathBuf,
    /// The ID of the session to load.
    pub session_id: SessionId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl LoadSessionRequest {
    pub fn new(session_id: impl Into<SessionId>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            mcp_servers: vec![],
            cwd: cwd.into(),
            session_id: session_id.into(),
            meta: None,
        }
    }

    /// List of MCP servers to connect to for this session.
    #[must_use]
    pub fn mcp_servers(mut self, mcp_servers: Vec<McpServer>) -> Self {
        self.mcp_servers = mcp_servers;
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Response from loading an existing session.
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_LOAD_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LoadSessionResponse {
    /// Initial mode state if supported by the Agent
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modes: Option<SessionModeState>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial model state if supported by the Agent
    #[cfg(feature = "unstable_session_model")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models: Option<SessionModelState>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial session configuration options if supported by the Agent.
    #[cfg(feature = "unstable_session_config_options")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_options: Option<Vec<SessionConfigOption>>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl LoadSessionResponse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initial mode state if supported by the Agent
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    #[must_use]
    pub fn modes(mut self, modes: impl IntoOption<SessionModeState>) -> Self {
        self.modes = modes.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial model state if supported by the Agent
    #[cfg(feature = "unstable_session_model")]
    #[must_use]
    pub fn models(mut self, models: impl IntoOption<SessionModelState>) -> Self {
        self.models = models.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial session configuration options if supported by the Agent.
    #[cfg(feature = "unstable_session_config_options")]
    #[must_use]
    pub fn config_options(
        mut self,
        config_options: impl IntoOption<Vec<SessionConfigOption>>,
    ) -> Self {
        self.config_options = config_options.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// Fork session

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for forking an existing session.
///
/// Creates a new session based on the context of an existing one, allowing
/// operations like generating summaries without affecting the original session's history.
///
/// Only available if the Agent supports the `session.fork` capability.
#[cfg(feature = "unstable_session_fork")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_FORK_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ForkSessionRequest {
    /// The ID of the session to fork.
    pub session_id: SessionId,
    /// The working directory for this session.
    pub cwd: PathBuf,
    /// List of MCP servers to connect to for this session.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServer>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_fork")]
impl ForkSessionRequest {
    pub fn new(session_id: impl Into<SessionId>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            session_id: session_id.into(),
            cwd: cwd.into(),
            mcp_servers: vec![],
            meta: None,
        }
    }

    /// List of MCP servers to connect to for this session.
    #[must_use]
    pub fn mcp_servers(mut self, mcp_servers: Vec<McpServer>) -> Self {
        self.mcp_servers = mcp_servers;
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response from forking an existing session.
#[cfg(feature = "unstable_session_fork")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_FORK_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ForkSessionResponse {
    /// Unique identifier for the newly created forked session.
    pub session_id: SessionId,
    /// Initial mode state if supported by the Agent
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modes: Option<SessionModeState>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial model state if supported by the Agent
    #[cfg(feature = "unstable_session_model")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<SessionModelState>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial session configuration options if supported by the Agent.
    #[cfg(feature = "unstable_session_config_options")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_options: Option<Vec<SessionConfigOption>>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_fork")]
impl ForkSessionResponse {
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>) -> Self {
        Self {
            session_id: session_id.into(),
            modes: None,
            #[cfg(feature = "unstable_session_model")]
            models: None,
            #[cfg(feature = "unstable_session_config_options")]
            config_options: None,
            meta: None,
        }
    }

    /// Initial mode state if supported by the Agent
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    #[must_use]
    pub fn modes(mut self, modes: impl IntoOption<SessionModeState>) -> Self {
        self.modes = modes.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial model state if supported by the Agent
    #[cfg(feature = "unstable_session_model")]
    #[must_use]
    pub fn models(mut self, models: impl IntoOption<SessionModelState>) -> Self {
        self.models = models.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial session configuration options if supported by the Agent.
    #[cfg(feature = "unstable_session_config_options")]
    #[must_use]
    pub fn config_options(
        mut self,
        config_options: impl IntoOption<Vec<SessionConfigOption>>,
    ) -> Self {
        self.config_options = config_options.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// Resume session

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for resuming an existing session.
///
/// Resumes an existing session without returning previous messages (unlike `session/load`).
/// This is useful for agents that can resume sessions but don't implement full session loading.
///
/// Only available if the Agent supports the `session.resume` capability.
#[cfg(feature = "unstable_session_resume")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_RESUME_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResumeSessionRequest {
    /// The ID of the session to resume.
    pub session_id: SessionId,
    /// The working directory for this session.
    pub cwd: PathBuf,
    /// List of MCP servers to connect to for this session.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServer>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_resume")]
impl ResumeSessionRequest {
    pub fn new(session_id: impl Into<SessionId>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            session_id: session_id.into(),
            cwd: cwd.into(),
            mcp_servers: vec![],
            meta: None,
        }
    }

    /// List of MCP servers to connect to for this session.
    #[must_use]
    pub fn mcp_servers(mut self, mcp_servers: Vec<McpServer>) -> Self {
        self.mcp_servers = mcp_servers;
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response from resuming an existing session.
#[cfg(feature = "unstable_session_resume")]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_RESUME_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResumeSessionResponse {
    /// Initial mode state if supported by the Agent
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modes: Option<SessionModeState>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial model state if supported by the Agent
    #[cfg(feature = "unstable_session_model")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models: Option<SessionModelState>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial session configuration options if supported by the Agent.
    #[cfg(feature = "unstable_session_config_options")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_options: Option<Vec<SessionConfigOption>>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_resume")]
impl ResumeSessionResponse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initial mode state if supported by the Agent
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    #[must_use]
    pub fn modes(mut self, modes: impl IntoOption<SessionModeState>) -> Self {
        self.modes = modes.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial model state if supported by the Agent
    #[cfg(feature = "unstable_session_model")]
    #[must_use]
    pub fn models(mut self, models: impl IntoOption<SessionModelState>) -> Self {
        self.models = models.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial session configuration options if supported by the Agent.
    #[cfg(feature = "unstable_session_config_options")]
    #[must_use]
    pub fn config_options(
        mut self,
        config_options: impl IntoOption<Vec<SessionConfigOption>>,
    ) -> Self {
        self.config_options = config_options.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// List sessions

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for listing existing sessions.
///
/// Only available if the Agent supports the `listSessions` capability.
#[cfg(feature = "unstable_session_list")]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_LIST_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ListSessionsRequest {
    /// Filter sessions by working directory. Must be an absolute path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    /// Opaque cursor token from a previous response's nextCursor field for cursor-based pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_list")]
impl ListSessionsRequest {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter sessions by working directory. Must be an absolute path.
    #[must_use]
    pub fn cwd(mut self, cwd: impl IntoOption<PathBuf>) -> Self {
        self.cwd = cwd.into_option();
        self
    }

    /// Opaque cursor token from a previous response's nextCursor field for cursor-based pagination
    #[must_use]
    pub fn cursor(mut self, cursor: impl IntoOption<String>) -> Self {
        self.cursor = cursor.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response from listing sessions.
#[cfg(feature = "unstable_session_list")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_LIST_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ListSessionsResponse {
    /// Array of session information objects
    pub sessions: Vec<SessionInfo>,
    /// Opaque cursor token. If present, pass this in the next request's cursor parameter
    /// to fetch the next page. If absent, there are no more results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_list")]
impl ListSessionsResponse {
    #[must_use]
    pub fn new(sessions: Vec<SessionInfo>) -> Self {
        Self {
            sessions,
            next_cursor: None,
            meta: None,
        }
    }

    #[must_use]
    pub fn next_cursor(mut self, next_cursor: impl IntoOption<String>) -> Self {
        self.next_cursor = next_cursor.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Information about a session returned by session/list
#[cfg(feature = "unstable_session_list")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionInfo {
    /// Unique identifier for the session
    pub session_id: SessionId,
    /// The working directory for this session. Must be an absolute path.
    pub cwd: PathBuf,
    /// Human-readable title for the session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// ISO 8601 timestamp of last activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_list")]
impl SessionInfo {
    pub fn new(session_id: impl Into<SessionId>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            session_id: session_id.into(),
            cwd: cwd.into(),
            title: None,
            updated_at: None,
            meta: None,
        }
    }

    /// Human-readable title for the session
    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
        self
    }

    /// ISO 8601 timestamp of last activity
    #[must_use]
    pub fn updated_at(mut self, updated_at: impl IntoOption<String>) -> Self {
        self.updated_at = updated_at.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// Session modes

/// The set of modes and the one currently active.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionModeState {
    /// The current mode the Agent is in.
    pub current_mode_id: SessionModeId,
    /// The set of modes that the Agent can operate in
    pub available_modes: Vec<SessionMode>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionModeState {
    #[must_use]
    pub fn new(
        current_mode_id: impl Into<SessionModeId>,
        available_modes: Vec<SessionMode>,
    ) -> Self {
        Self {
            current_mode_id: current_mode_id.into(),
            available_modes,
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// A mode the agent can operate in.
///
/// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionMode {
    pub id: SessionModeId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionMode {
    pub fn new(id: impl Into<SessionModeId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            meta: None,
        }
    }

    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Unique identifier for a Session Mode.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, From, Display)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct SessionModeId(pub Arc<str>);

impl SessionModeId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// Request parameters for setting a session mode.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_SET_MODE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SetSessionModeRequest {
    /// The ID of the session to set the mode for.
    pub session_id: SessionId,
    /// The ID of the mode to set.
    pub mode_id: SessionModeId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SetSessionModeRequest {
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, mode_id: impl Into<SessionModeId>) -> Self {
        Self {
            session_id: session_id.into(),
            mode_id: mode_id.into(),
            meta: None,
        }
    }

    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Response to `session/set_mode` method.
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_SET_MODE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SetSessionModeResponse {
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SetSessionModeResponse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// Session config options

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Unique identifier for a session configuration option.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, From, Display)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct SessionConfigId(pub Arc<str>);

#[cfg(feature = "unstable_session_config_options")]
impl SessionConfigId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Unique identifier for a session configuration option value.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, From, Display)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct SessionConfigValueId(pub Arc<str>);

#[cfg(feature = "unstable_session_config_options")]
impl SessionConfigValueId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Unique identifier for a session configuration option value group.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, From, Display)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct SessionConfigGroupId(pub Arc<str>);

#[cfg(feature = "unstable_session_config_options")]
impl SessionConfigGroupId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A possible value for a session configuration option.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionConfigSelectOption {
    /// Unique identifier for this option value.
    pub value: SessionConfigValueId,
    /// Human-readable label for this option value.
    pub name: String,
    /// Optional description for this option value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_config_options")]
impl SessionConfigSelectOption {
    #[must_use]
    pub fn new(value: impl Into<SessionConfigValueId>, name: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            name: name.into(),
            description: None,
            meta: None,
        }
    }

    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A group of possible values for a session configuration option.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionConfigSelectGroup {
    /// Unique identifier for this group.
    pub group: SessionConfigGroupId,
    /// Human-readable label for this group.
    pub name: String,
    /// The set of option values in this group.
    pub options: Vec<SessionConfigSelectOption>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_config_options")]
impl SessionConfigSelectGroup {
    #[must_use]
    pub fn new(
        group: impl Into<SessionConfigGroupId>,
        name: impl Into<String>,
        options: Vec<SessionConfigSelectOption>,
    ) -> Self {
        Self {
            group: group.into(),
            name: name.into(),
            options,
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Possible values for a session configuration option.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum SessionConfigSelectOptions {
    /// A flat list of options with no grouping.
    Ungrouped(Vec<SessionConfigSelectOption>),
    /// A list of options grouped under headers.
    Grouped(Vec<SessionConfigSelectGroup>),
}

#[cfg(feature = "unstable_session_config_options")]
impl From<Vec<SessionConfigSelectOption>> for SessionConfigSelectOptions {
    fn from(options: Vec<SessionConfigSelectOption>) -> Self {
        SessionConfigSelectOptions::Ungrouped(options)
    }
}

#[cfg(feature = "unstable_session_config_options")]
impl From<Vec<SessionConfigSelectGroup>> for SessionConfigSelectOptions {
    fn from(groups: Vec<SessionConfigSelectGroup>) -> Self {
        SessionConfigSelectOptions::Grouped(groups)
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A single-value selector (dropdown) session configuration option payload.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionConfigSelect {
    /// The currently selected value.
    pub current_value: SessionConfigValueId,
    /// The set of selectable options.
    pub options: SessionConfigSelectOptions,
}

#[cfg(feature = "unstable_session_config_options")]
impl SessionConfigSelect {
    #[must_use]
    pub fn new(
        current_value: impl Into<SessionConfigValueId>,
        options: impl Into<SessionConfigSelectOptions>,
    ) -> Self {
        Self {
            current_value: current_value.into(),
            options: options.into(),
        }
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Semantic category for a session configuration option.
///
/// This is intended to help Clients distinguish broadly common selectors (e.g. model selector vs
/// session mode selector vs thought/reasoning level) for UX purposes (keyboard shortcuts, icons,
/// placement). It MUST NOT be required for correctness. Clients MUST handle missing or unknown
/// categories gracefully (treat as `Other`).
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SessionConfigOptionCategory {
    /// Session mode selector.
    Mode,
    /// Model selector.
    Model,
    /// Thought/reasoning level selector.
    ThoughtLevel,
    /// Unknown / uncategorized selector.
    #[serde(other)]
    Other,
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Type-specific session configuration option payload.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(extend("discriminator" = {"propertyName": "type"}))]
#[non_exhaustive]
pub enum SessionConfigKind {
    /// Single-value selector (dropdown).
    Select(SessionConfigSelect),
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A session configuration option selector and its current state.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionConfigOption {
    /// Unique identifier for the configuration option.
    pub id: SessionConfigId,
    /// Human-readable label for the option.
    pub name: String,
    /// Optional description for the Client to display to the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional semantic category for this option (UX only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<SessionConfigOptionCategory>,
    /// Type-specific fields for this configuration option.
    #[serde(flatten)]
    pub kind: SessionConfigKind,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_config_options")]
impl SessionConfigOption {
    #[must_use]
    pub fn new(
        id: impl Into<SessionConfigId>,
        name: impl Into<String>,
        kind: SessionConfigKind,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            category: None,
            kind,
            meta: None,
        }
    }

    #[must_use]
    pub fn select(
        id: impl Into<SessionConfigId>,
        name: impl Into<String>,
        current_value: impl Into<SessionConfigValueId>,
        options: impl Into<SessionConfigSelectOptions>,
    ) -> Self {
        Self::new(
            id,
            name,
            SessionConfigKind::Select(SessionConfigSelect::new(current_value, options)),
        )
    }

    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    #[must_use]
    pub fn category(mut self, category: impl IntoOption<SessionConfigOptionCategory>) -> Self {
        self.category = category.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for setting a session configuration option.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_SET_CONFIG_OPTION_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SetSessionConfigOptionRequest {
    /// The ID of the session to set the configuration option for.
    pub session_id: SessionId,
    /// The ID of the configuration option to set.
    pub config_id: SessionConfigId,
    /// The ID of the configuration option value to set.
    pub value: SessionConfigValueId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_config_options")]
impl SetSessionConfigOptionRequest {
    #[must_use]
    pub fn new(
        session_id: impl Into<SessionId>,
        config_id: impl Into<SessionConfigId>,
        value: impl Into<SessionConfigValueId>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            config_id: config_id.into(),
            value: value.into(),
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response to `session/set_config_option` method.
#[cfg(feature = "unstable_session_config_options")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_SET_CONFIG_OPTION_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SetSessionConfigOptionResponse {
    /// The full set of configuration options and their current values.
    pub config_options: Vec<SessionConfigOption>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_config_options")]
impl SetSessionConfigOptionResponse {
    #[must_use]
    pub fn new(config_options: Vec<SessionConfigOption>) -> Self {
        Self {
            config_options,
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// MCP

/// Configuration for connecting to an MCP (Model Context Protocol) server.
///
/// MCP servers provide tools and context that the agent can use when
/// processing prompts.
///
/// See protocol docs: [MCP Servers](https://agentclientprotocol.com/protocol/session-setup#mcp-servers)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum McpServer {
    /// HTTP transport configuration
    ///
    /// Only available when the Agent capabilities indicate `mcp_capabilities.http` is `true`.
    Http(McpServerHttp),
    /// SSE transport configuration
    ///
    /// Only available when the Agent capabilities indicate `mcp_capabilities.sse` is `true`.
    Sse(McpServerSse),
    /// Stdio transport configuration
    ///
    /// All Agents MUST support this transport.
    #[serde(untagged)]
    Stdio(McpServerStdio),
}

/// HTTP transport configuration for MCP.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct McpServerHttp {
    /// Human-readable name identifying this MCP server.
    pub name: String,
    /// URL to the MCP server.
    pub url: String,
    /// HTTP headers to set when making requests to the MCP server.
    pub headers: Vec<HttpHeader>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl McpServerHttp {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            headers: Vec::new(),
            meta: None,
        }
    }

    /// HTTP headers to set when making requests to the MCP server.
    #[must_use]
    pub fn headers(mut self, headers: Vec<HttpHeader>) -> Self {
        self.headers = headers;
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// SSE transport configuration for MCP.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct McpServerSse {
    /// Human-readable name identifying this MCP server.
    pub name: String,
    /// URL to the MCP server.
    pub url: String,
    /// HTTP headers to set when making requests to the MCP server.
    pub headers: Vec<HttpHeader>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl McpServerSse {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            headers: Vec::new(),
            meta: None,
        }
    }

    /// HTTP headers to set when making requests to the MCP server.
    #[must_use]
    pub fn headers(mut self, headers: Vec<HttpHeader>) -> Self {
        self.headers = headers;
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Stdio transport configuration for MCP.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct McpServerStdio {
    /// Human-readable name identifying this MCP server.
    pub name: String,
    /// Path to the MCP server executable.
    pub command: PathBuf,
    /// Command-line arguments to pass to the MCP server.
    pub args: Vec<String>,
    /// Environment variables to set when launching the MCP server.
    pub env: Vec<EnvVariable>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl McpServerStdio {
    pub fn new(name: impl Into<String>, command: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            args: Vec::new(),
            env: Vec::new(),
            meta: None,
        }
    }

    /// Command-line arguments to pass to the MCP server.
    #[must_use]
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Environment variables to set when launching the MCP server.
    #[must_use]
    pub fn env(mut self, env: Vec<EnvVariable>) -> Self {
        self.env = env;
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// An environment variable to set when launching an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EnvVariable {
    /// The name of the environment variable.
    pub name: String,
    /// The value to set for the environment variable.
    pub value: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl EnvVariable {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// An HTTP header to set when making requests to the MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct HttpHeader {
    /// The name of the HTTP header.
    pub name: String,
    /// The value to set for the HTTP header.
    pub value: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl HttpHeader {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// Prompt

/// Request parameters for sending a user prompt to the agent.
///
/// Contains the user's message and any additional context.
///
/// See protocol docs: [User Message](https://agentclientprotocol.com/protocol/prompt-turn#1-user-message)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_PROMPT_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PromptRequest {
    /// The ID of the session to send this user message to
    pub session_id: SessionId,
    /// The blocks of content that compose the user's message.
    ///
    /// As a baseline, the Agent MUST support [`ContentBlock::Text`] and [`ContentBlock::ResourceLink`],
    /// while other variants are optionally enabled via [`PromptCapabilities`].
    ///
    /// The Client MUST adapt its interface according to [`PromptCapabilities`].
    ///
    /// The client MAY include referenced pieces of context as either
    /// [`ContentBlock::Resource`] or [`ContentBlock::ResourceLink`].
    ///
    /// When available, [`ContentBlock::Resource`] is preferred
    /// as it avoids extra round-trips and allows the message to include
    /// pieces of context from sources the agent may not have access to.
    pub prompt: Vec<ContentBlock>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl PromptRequest {
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, prompt: Vec<ContentBlock>) -> Self {
        Self {
            session_id: session_id.into(),
            prompt,
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Response from processing a user prompt.
///
/// See protocol docs: [Check for Completion](https://agentclientprotocol.com/protocol/prompt-turn#4-check-for-completion)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_PROMPT_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PromptResponse {
    /// Indicates why the agent stopped processing the turn.
    pub stop_reason: StopReason,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl PromptResponse {
    #[must_use]
    pub fn new(stop_reason: StopReason) -> Self {
        Self {
            stop_reason,
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Reasons why an agent stops processing a prompt turn.
///
/// See protocol docs: [Stop Reasons](https://agentclientprotocol.com/protocol/prompt-turn#stop-reasons)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum StopReason {
    /// The turn ended successfully.
    EndTurn,
    /// The turn ended because the agent reached the maximum number of tokens.
    MaxTokens,
    /// The turn ended because the agent reached the maximum number of allowed
    /// agent requests between user turns.
    MaxTurnRequests,
    /// The turn ended because the agent refused to continue. The user prompt
    /// and everything that comes after it won't be included in the next
    /// prompt, so this should be reflected in the UI.
    Refusal,
    /// The turn was cancelled by the client via `session/cancel`.
    ///
    /// This stop reason MUST be returned when the client sends a `session/cancel`
    /// notification, even if the cancellation causes exceptions in underlying operations.
    /// Agents should catch these exceptions and return this semantically meaningful
    /// response to confirm successful cancellation.
    Cancelled,
}

// Model

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// The set of models and the one currently active.
#[cfg(feature = "unstable_session_model")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionModelState {
    /// The current model the Agent is in.
    pub current_model_id: ModelId,
    /// The set of models that the Agent can use
    pub available_models: Vec<ModelInfo>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_model")]
impl SessionModelState {
    #[must_use]
    pub fn new(current_model_id: impl Into<ModelId>, available_models: Vec<ModelInfo>) -> Self {
        Self {
            current_model_id: current_model_id.into(),
            available_models,
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A unique identifier for a model.
#[cfg(feature = "unstable_session_model")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct ModelId(pub Arc<str>);

#[cfg(feature = "unstable_session_model")]
impl ModelId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Information about a selectable model.
#[cfg(feature = "unstable_session_model")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ModelInfo {
    /// Unique identifier for the model.
    pub model_id: ModelId,
    /// Human-readable name of the model.
    pub name: String,
    /// Optional description of the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_model")]
impl ModelInfo {
    pub fn new(model_id: impl Into<ModelId>, name: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            name: name.into(),
            description: None,
            meta: None,
        }
    }

    /// Optional description of the model.
    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for setting a session model.
#[cfg(feature = "unstable_session_model")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_SET_MODEL_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SetSessionModelRequest {
    /// The ID of the session to set the model for.
    pub session_id: SessionId,
    /// The ID of the model to set.
    pub model_id: ModelId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_model")]
impl SetSessionModelRequest {
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, model_id: impl Into<ModelId>) -> Self {
        Self {
            session_id: session_id.into(),
            model_id: model_id.into(),
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response to `session/set_model` method.
#[cfg(feature = "unstable_session_model")]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_SET_MODEL_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SetSessionModelResponse {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_model")]
impl SetSessionModelResponse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// Capabilities

/// Capabilities supported by the agent.
///
/// Advertised during initialization to inform the client about
/// available features and content types.
///
/// See protocol docs: [Agent Capabilities](https://agentclientprotocol.com/protocol/initialization#agent-capabilities)
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AgentCapabilities {
    /// Whether the agent supports `session/load`.
    #[serde(default)]
    pub load_session: bool,
    /// Prompt capabilities supported by the agent.
    #[serde(default)]
    pub prompt_capabilities: PromptCapabilities,
    /// MCP capabilities supported by the agent.
    #[serde(default)]
    pub mcp_capabilities: McpCapabilities,
    #[serde(default)]
    pub session_capabilities: SessionCapabilities,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl AgentCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the agent supports `session/load`.
    #[must_use]
    pub fn load_session(mut self, load_session: bool) -> Self {
        self.load_session = load_session;
        self
    }

    /// Prompt capabilities supported by the agent.
    #[must_use]
    pub fn prompt_capabilities(mut self, prompt_capabilities: PromptCapabilities) -> Self {
        self.prompt_capabilities = prompt_capabilities;
        self
    }

    /// MCP capabilities supported by the agent.
    #[must_use]
    pub fn mcp_capabilities(mut self, mcp_capabilities: McpCapabilities) -> Self {
        self.mcp_capabilities = mcp_capabilities;
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Session capabilities supported by the agent.
///
/// As a baseline, all Agents **MUST** support `session/new`, `session/prompt`, `session/cancel`, and `session/update`.
///
/// Optionally, they **MAY** support other session methods and notifications by specifying additional capabilities.
///
/// Note: `session/load` is still handled by the top-level `load_session` capability. This will be unified in future versions of the protocol.
///
/// See protocol docs: [Session Capabilities](https://agentclientprotocol.com/protocol/initialization#session-capabilities)
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[non_exhaustive]
pub struct SessionCapabilities {
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Whether the agent supports `session/list`.
    #[cfg(feature = "unstable_session_list")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<SessionListCapabilities>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Whether the agent supports `session/fork`.
    #[cfg(feature = "unstable_session_fork")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork: Option<SessionForkCapabilities>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Whether the agent supports `session/resume`.
    #[cfg(feature = "unstable_session_resume")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<SessionResumeCapabilities>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "unstable_session_list")]
    /// Whether the agent supports `session/list`.
    #[must_use]
    pub fn list(mut self, list: impl IntoOption<SessionListCapabilities>) -> Self {
        self.list = list.into_option();
        self
    }

    #[cfg(feature = "unstable_session_fork")]
    /// Whether the agent supports `session/fork`.
    #[must_use]
    pub fn fork(mut self, fork: impl IntoOption<SessionForkCapabilities>) -> Self {
        self.fork = fork.into_option();
        self
    }

    #[cfg(feature = "unstable_session_resume")]
    /// Whether the agent supports `session/resume`.
    #[must_use]
    pub fn resume(mut self, resume: impl IntoOption<SessionResumeCapabilities>) -> Self {
        self.resume = resume.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Capabilities for the `session/list` method.
///
/// By supplying `{}` it means that the agent supports listing of sessions.
///
/// Further capabilities can be added in the future for other means of filtering or searching the list.
#[cfg(feature = "unstable_session_list")]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[non_exhaustive]
pub struct SessionListCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_list")]
impl SessionListCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Capabilities for the `session/fork` method.
///
/// By supplying `{}` it means that the agent supports forking of sessions.
#[cfg(feature = "unstable_session_fork")]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[non_exhaustive]
pub struct SessionForkCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_fork")]
impl SessionForkCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Capabilities for the `session/resume` method.
///
/// By supplying `{}` it means that the agent supports resuming of sessions.
#[cfg(feature = "unstable_session_resume")]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[non_exhaustive]
pub struct SessionResumeCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_resume")]
impl SessionResumeCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Prompt capabilities supported by the agent in `session/prompt` requests.
///
/// Baseline agent functionality requires support for [`ContentBlock::Text`]
/// and [`ContentBlock::ResourceLink`] in prompt requests.
///
/// Other variants must be explicitly opted in to.
/// Capabilities for different types of content in prompt requests.
///
/// Indicates which content types beyond the baseline (text and resource links)
/// the agent can process.
///
/// See protocol docs: [Prompt Capabilities](https://agentclientprotocol.com/protocol/initialization#prompt-capabilities)
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PromptCapabilities {
    /// Agent supports [`ContentBlock::Image`].
    #[serde(default)]
    pub image: bool,
    /// Agent supports [`ContentBlock::Audio`].
    #[serde(default)]
    pub audio: bool,
    /// Agent supports embedded context in `session/prompt` requests.
    ///
    /// When enabled, the Client is allowed to include [`ContentBlock::Resource`]
    /// in prompt requests for pieces of context that are referenced in the message.
    #[serde(default)]
    pub embedded_context: bool,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl PromptCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Agent supports [`ContentBlock::Image`].
    #[must_use]
    pub fn image(mut self, image: bool) -> Self {
        self.image = image;
        self
    }

    /// Agent supports [`ContentBlock::Audio`].
    #[must_use]
    pub fn audio(mut self, audio: bool) -> Self {
        self.audio = audio;
        self
    }

    /// Agent supports embedded context in `session/prompt` requests.
    ///
    /// When enabled, the Client is allowed to include [`ContentBlock::Resource`]
    /// in prompt requests for pieces of context that are referenced in the message.
    #[must_use]
    pub fn embedded_context(mut self, embedded_context: bool) -> Self {
        self.embedded_context = embedded_context;
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// MCP capabilities supported by the agent
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct McpCapabilities {
    /// Agent supports [`McpServer::Http`].
    #[serde(default)]
    pub http: bool,
    /// Agent supports [`McpServer::Sse`].
    #[serde(default)]
    pub sse: bool,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl McpCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Agent supports [`McpServer::Http`].
    #[must_use]
    pub fn http(mut self, http: bool) -> Self {
        self.http = http;
        self
    }

    /// Agent supports [`McpServer::Sse`].
    #[must_use]
    pub fn sse(mut self, sse: bool) -> Self {
        self.sse = sse;
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// Method schema

/// Names of all methods that agents handle.
///
/// Provides a centralized definition of method names used in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct AgentMethodNames {
    /// Method for initializing the connection.
    pub initialize: &'static str,
    /// Method for authenticating with the agent.
    pub authenticate: &'static str,
    /// Method for creating a new session.
    pub session_new: &'static str,
    /// Method for loading an existing session.
    pub session_load: &'static str,
    /// Method for setting the mode for a session.
    pub session_set_mode: &'static str,
    /// Method for setting a configuration option for a session.
    #[cfg(feature = "unstable_session_config_options")]
    pub session_set_config_option: &'static str,
    /// Method for sending a prompt to the agent.
    pub session_prompt: &'static str,
    /// Notification for cancelling operations.
    pub session_cancel: &'static str,
    /// Method for selecting a model for a given session.
    #[cfg(feature = "unstable_session_model")]
    pub session_set_model: &'static str,
    /// Method for listing existing sessions.
    #[cfg(feature = "unstable_session_list")]
    pub session_list: &'static str,
    /// Method for forking an existing session.
    #[cfg(feature = "unstable_session_fork")]
    pub session_fork: &'static str,
    /// Method for resuming an existing session.
    #[cfg(feature = "unstable_session_resume")]
    pub session_resume: &'static str,
}

/// Constant containing all agent method names.
pub const AGENT_METHOD_NAMES: AgentMethodNames = AgentMethodNames {
    initialize: INITIALIZE_METHOD_NAME,
    authenticate: AUTHENTICATE_METHOD_NAME,
    session_new: SESSION_NEW_METHOD_NAME,
    session_load: SESSION_LOAD_METHOD_NAME,
    session_set_mode: SESSION_SET_MODE_METHOD_NAME,
    #[cfg(feature = "unstable_session_config_options")]
    session_set_config_option: SESSION_SET_CONFIG_OPTION_METHOD_NAME,
    session_prompt: SESSION_PROMPT_METHOD_NAME,
    session_cancel: SESSION_CANCEL_METHOD_NAME,
    #[cfg(feature = "unstable_session_model")]
    session_set_model: SESSION_SET_MODEL_METHOD_NAME,
    #[cfg(feature = "unstable_session_list")]
    session_list: SESSION_LIST_METHOD_NAME,
    #[cfg(feature = "unstable_session_fork")]
    session_fork: SESSION_FORK_METHOD_NAME,
    #[cfg(feature = "unstable_session_resume")]
    session_resume: SESSION_RESUME_METHOD_NAME,
};

/// Method name for the initialize request.
pub(crate) const INITIALIZE_METHOD_NAME: &str = "initialize";
/// Method name for the authenticate request.
pub(crate) const AUTHENTICATE_METHOD_NAME: &str = "authenticate";
/// Method name for creating a new session.
pub(crate) const SESSION_NEW_METHOD_NAME: &str = "session/new";
/// Method name for loading an existing session.
pub(crate) const SESSION_LOAD_METHOD_NAME: &str = "session/load";
/// Method name for setting the mode for a session.
pub(crate) const SESSION_SET_MODE_METHOD_NAME: &str = "session/set_mode";
/// Method name for setting a configuration option for a session.
#[cfg(feature = "unstable_session_config_options")]
pub(crate) const SESSION_SET_CONFIG_OPTION_METHOD_NAME: &str = "session/set_config_option";
/// Method name for sending a prompt.
pub(crate) const SESSION_PROMPT_METHOD_NAME: &str = "session/prompt";
/// Method name for the cancel notification.
pub(crate) const SESSION_CANCEL_METHOD_NAME: &str = "session/cancel";
/// Method name for selecting a model for a given session.
#[cfg(feature = "unstable_session_model")]
pub(crate) const SESSION_SET_MODEL_METHOD_NAME: &str = "session/set_model";
/// Method name for listing existing sessions.
#[cfg(feature = "unstable_session_list")]
pub(crate) const SESSION_LIST_METHOD_NAME: &str = "session/list";
/// Method name for forking an existing session.
#[cfg(feature = "unstable_session_fork")]
pub(crate) const SESSION_FORK_METHOD_NAME: &str = "session/fork";
/// Method name for resuming an existing session.
#[cfg(feature = "unstable_session_resume")]
pub(crate) const SESSION_RESUME_METHOD_NAME: &str = "session/resume";

/// All possible requests that a client can send to an agent.
///
/// This enum is used internally for routing RPC requests. You typically won't need
/// to use this directly - instead, use the methods on the [`Agent`] trait.
///
/// This enum encompasses all method calls from client to agent.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[non_exhaustive]
pub enum ClientRequest {
    /// Establishes the connection with a client and negotiates protocol capabilities.
    ///
    /// This method is called once at the beginning of the connection to:
    /// - Negotiate the protocol version to use
    /// - Exchange capability information between client and agent
    /// - Determine available authentication methods
    ///
    /// The agent should respond with its supported protocol version and capabilities.
    ///
    /// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
    InitializeRequest(InitializeRequest),
    /// Authenticates the client using the specified authentication method.
    ///
    /// Called when the agent requires authentication before allowing session creation.
    /// The client provides the authentication method ID that was advertised during initialization.
    ///
    /// After successful authentication, the client can proceed to create sessions with
    /// `new_session` without receiving an `auth_required` error.
    ///
    /// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
    AuthenticateRequest(AuthenticateRequest),
    /// Creates a new conversation session with the agent.
    ///
    /// Sessions represent independent conversation contexts with their own history and state.
    ///
    /// The agent should:
    /// - Create a new session context
    /// - Connect to any specified MCP servers
    /// - Return a unique session ID for future requests
    ///
    /// May return an `auth_required` error if the agent requires authentication.
    ///
    /// See protocol docs: [Session Setup](https://agentclientprotocol.com/protocol/session-setup)
    NewSessionRequest(NewSessionRequest),
    /// Loads an existing session to resume a previous conversation.
    ///
    /// This method is only available if the agent advertises the `loadSession` capability.
    ///
    /// The agent should:
    /// - Restore the session context and conversation history
    /// - Connect to the specified MCP servers
    /// - Stream the entire conversation history back to the client via notifications
    ///
    /// See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
    LoadSessionRequest(LoadSessionRequest),
    #[cfg(feature = "unstable_session_list")]
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Lists existing sessions known to the agent.
    ///
    /// This method is only available if the agent advertises the `listSessions` capability.
    ///
    /// The agent should return metadata about sessions with optional filtering and pagination support.
    ListSessionsRequest(ListSessionsRequest),
    #[cfg(feature = "unstable_session_fork")]
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Forks an existing session to create a new independent session.
    ///
    /// This method is only available if the agent advertises the `session.fork` capability.
    ///
    /// The agent should create a new session with the same conversation context as the
    /// original, allowing operations like generating summaries without affecting the
    /// original session's history.
    ForkSessionRequest(ForkSessionRequest),
    #[cfg(feature = "unstable_session_resume")]
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Resumes an existing session without returning previous messages.
    ///
    /// This method is only available if the agent advertises the `session.resume` capability.
    ///
    /// The agent should resume the session context, allowing the conversation to continue
    /// without replaying the message history (unlike `session/load`).
    ResumeSessionRequest(ResumeSessionRequest),
    /// Sets the current mode for a session.
    ///
    /// Allows switching between different agent modes (e.g., "ask", "architect", "code")
    /// that affect system prompts, tool availability, and permission behaviors.
    ///
    /// The mode must be one of the modes advertised in `availableModes` during session
    /// creation or loading. Agents may also change modes autonomously and notify the
    /// client via `current_mode_update` notifications.
    ///
    /// This method can be called at any time during a session, whether the Agent is
    /// idle or actively generating a response.
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    SetSessionModeRequest(SetSessionModeRequest),
    #[cfg(feature = "unstable_session_config_options")]
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Sets the current value for a session configuration option.
    SetSessionConfigOptionRequest(SetSessionConfigOptionRequest),
    /// Processes a user prompt within a session.
    ///
    /// This method handles the whole lifecycle of a prompt:
    /// - Receives user messages with optional context (files, images, etc.)
    /// - Processes the prompt using language models
    /// - Reports language model content and tool calls to the Clients
    /// - Requests permission to run tools
    /// - Executes any requested tool calls
    /// - Returns when the turn is complete with a stop reason
    ///
    /// See protocol docs: [Prompt Turn](https://agentclientprotocol.com/protocol/prompt-turn)
    PromptRequest(PromptRequest),
    #[cfg(feature = "unstable_session_model")]
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Select a model for a given session.
    SetSessionModelRequest(SetSessionModelRequest),
    /// Handles extension method requests from the client.
    ///
    /// Extension methods provide a way to add custom functionality while maintaining
    /// protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    ExtMethodRequest(ExtRequest),
}

impl ClientRequest {
    /// Returns the corresponding method name of the request.
    #[must_use]
    pub fn method(&self) -> &str {
        match self {
            Self::InitializeRequest(_) => AGENT_METHOD_NAMES.initialize,
            Self::AuthenticateRequest(_) => AGENT_METHOD_NAMES.authenticate,
            Self::NewSessionRequest(_) => AGENT_METHOD_NAMES.session_new,
            Self::LoadSessionRequest(_) => AGENT_METHOD_NAMES.session_load,
            #[cfg(feature = "unstable_session_list")]
            Self::ListSessionsRequest(_) => AGENT_METHOD_NAMES.session_list,
            #[cfg(feature = "unstable_session_fork")]
            Self::ForkSessionRequest(_) => AGENT_METHOD_NAMES.session_fork,
            #[cfg(feature = "unstable_session_resume")]
            Self::ResumeSessionRequest(_) => AGENT_METHOD_NAMES.session_resume,
            Self::SetSessionModeRequest(_) => AGENT_METHOD_NAMES.session_set_mode,
            #[cfg(feature = "unstable_session_config_options")]
            Self::SetSessionConfigOptionRequest(_) => AGENT_METHOD_NAMES.session_set_config_option,
            Self::PromptRequest(_) => AGENT_METHOD_NAMES.session_prompt,
            #[cfg(feature = "unstable_session_model")]
            Self::SetSessionModelRequest(_) => AGENT_METHOD_NAMES.session_set_model,
            Self::ExtMethodRequest(ext_request) => &ext_request.method,
        }
    }
}

/// All possible responses that an agent can send to a client.
///
/// This enum is used internally for routing RPC responses. You typically won't need
/// to use this directly - the responses are handled automatically by the connection.
///
/// These are responses to the corresponding `ClientRequest` variants.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[non_exhaustive]
pub enum AgentResponse {
    InitializeResponse(InitializeResponse),
    AuthenticateResponse(#[serde(default)] AuthenticateResponse),
    NewSessionResponse(NewSessionResponse),
    LoadSessionResponse(#[serde(default)] LoadSessionResponse),
    #[cfg(feature = "unstable_session_list")]
    ListSessionsResponse(ListSessionsResponse),
    #[cfg(feature = "unstable_session_fork")]
    ForkSessionResponse(ForkSessionResponse),
    #[cfg(feature = "unstable_session_resume")]
    ResumeSessionResponse(#[serde(default)] ResumeSessionResponse),
    SetSessionModeResponse(#[serde(default)] SetSessionModeResponse),
    #[cfg(feature = "unstable_session_config_options")]
    SetSessionConfigOptionResponse(SetSessionConfigOptionResponse),
    PromptResponse(PromptResponse),
    #[cfg(feature = "unstable_session_model")]
    SetSessionModelResponse(SetSessionModelResponse),
    ExtMethodResponse(ExtResponse),
}

/// All possible notifications that a client can send to an agent.
///
/// This enum is used internally for routing RPC notifications. You typically won't need
/// to use this directly - use the notification methods on the [`Agent`] trait instead.
///
/// Notifications do not expect a response.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[non_exhaustive]
pub enum ClientNotification {
    /// Cancels ongoing operations for a session.
    ///
    /// This is a notification sent by the client to cancel an ongoing prompt turn.
    ///
    /// Upon receiving this notification, the Agent SHOULD:
    /// - Stop all language model requests as soon as possible
    /// - Abort all tool call invocations in progress
    /// - Send any pending `session/update` notifications
    /// - Respond to the original `session/prompt` request with `StopReason::Cancelled`
    ///
    /// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
    CancelNotification(CancelNotification),
    /// Handles extension notifications from the client.
    ///
    /// Extension notifications provide a way to send one-way messages for custom functionality
    /// while maintaining protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    ExtNotification(ExtNotification),
}

impl ClientNotification {
    /// Returns the corresponding method name of the notification.
    #[must_use]
    pub fn method(&self) -> &str {
        match self {
            Self::CancelNotification(_) => AGENT_METHOD_NAMES.session_cancel,
            Self::ExtNotification(ext_notification) => &ext_notification.method,
        }
    }
}

/// Notification to cancel ongoing operations for a session.
///
/// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_CANCEL_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CancelNotification {
    /// The ID of the session to cancel operations for.
    pub session_id: SessionId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl CancelNotification {
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>) -> Self {
        Self {
            session_id: session_id.into(),
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

#[cfg(test)]
mod test_serialization {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_server_stdio_serialization() {
        let server = McpServer::Stdio(
            McpServerStdio::new("test-server", "/usr/bin/server")
                .args(vec!["--port".to_string(), "3000".to_string()])
                .env(vec![EnvVariable::new("API_KEY", "secret123")]),
        );

        let json = serde_json::to_value(&server).unwrap();
        assert_eq!(
            json,
            json!({
                "name": "test-server",
                "command": "/usr/bin/server",
                "args": ["--port", "3000"],
                "env": [
                    {
                        "name": "API_KEY",
                        "value": "secret123"
                    }
                ]
            })
        );

        let deserialized: McpServer = serde_json::from_value(json).unwrap();
        match deserialized {
            McpServer::Stdio(McpServerStdio {
                name,
                command,
                args,
                env,
                meta: _,
            }) => {
                assert_eq!(name, "test-server");
                assert_eq!(command, PathBuf::from("/usr/bin/server"));
                assert_eq!(args, vec!["--port", "3000"]);
                assert_eq!(env.len(), 1);
                assert_eq!(env[0].name, "API_KEY");
                assert_eq!(env[0].value, "secret123");
            }
            _ => panic!("Expected Stdio variant"),
        }
    }

    #[test]
    fn test_mcp_server_http_serialization() {
        let server = McpServer::Http(
            McpServerHttp::new("http-server", "https://api.example.com").headers(vec![
                HttpHeader::new("Authorization", "Bearer token123"),
                HttpHeader::new("Content-Type", "application/json"),
            ]),
        );

        let json = serde_json::to_value(&server).unwrap();
        assert_eq!(
            json,
            json!({
                "type": "http",
                "name": "http-server",
                "url": "https://api.example.com",
                "headers": [
                    {
                        "name": "Authorization",
                        "value": "Bearer token123"
                    },
                    {
                        "name": "Content-Type",
                        "value": "application/json"
                    }
                ]
            })
        );

        let deserialized: McpServer = serde_json::from_value(json).unwrap();
        match deserialized {
            McpServer::Http(McpServerHttp {
                name,
                url,
                headers,
                meta: _,
            }) => {
                assert_eq!(name, "http-server");
                assert_eq!(url, "https://api.example.com");
                assert_eq!(headers.len(), 2);
                assert_eq!(headers[0].name, "Authorization");
                assert_eq!(headers[0].value, "Bearer token123");
                assert_eq!(headers[1].name, "Content-Type");
                assert_eq!(headers[1].value, "application/json");
            }
            _ => panic!("Expected Http variant"),
        }
    }

    #[test]
    fn test_mcp_server_sse_serialization() {
        let server = McpServer::Sse(
            McpServerSse::new("sse-server", "https://sse.example.com/events")
                .headers(vec![HttpHeader::new("X-API-Key", "apikey456")]),
        );

        let json = serde_json::to_value(&server).unwrap();
        assert_eq!(
            json,
            json!({
                "type": "sse",
                "name": "sse-server",
                "url": "https://sse.example.com/events",
                "headers": [
                    {
                        "name": "X-API-Key",
                        "value": "apikey456"
                    }
                ]
            })
        );

        let deserialized: McpServer = serde_json::from_value(json).unwrap();
        match deserialized {
            McpServer::Sse(McpServerSse {
                name,
                url,
                headers,
                meta: _,
            }) => {
                assert_eq!(name, "sse-server");
                assert_eq!(url, "https://sse.example.com/events");
                assert_eq!(headers.len(), 1);
                assert_eq!(headers[0].name, "X-API-Key");
                assert_eq!(headers[0].value, "apikey456");
            }
            _ => panic!("Expected Sse variant"),
        }
    }
}
