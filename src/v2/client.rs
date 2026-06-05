//! Methods and notifications the client handles/receives.
//!
//! This module defines the Client trait and all associated types for implementing
//! a client that interacts with AI coding agents via the Agent Client Protocol (ACP).

use std::{collections::BTreeMap, sync::Arc};

use derive_more::{Display, From};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, VecSkipError, serde_as, skip_serializing_none};

#[cfg(feature = "unstable_plan_operations")]
use super::PlanRemoved;
#[cfg(feature = "unstable_elicitation")]
use super::{
    CompleteElicitationNotification, CreateElicitationRequest, CreateElicitationResponse,
    ElicitationCapabilities,
};
use super::{
    ContentBlock, ExtNotification, ExtRequest, ExtResponse, Meta, PlanUpdate, SessionConfigOption,
    SessionId, ToolCall, ToolCallUpdate,
};
use crate::{IntoMaybeUndefined, IntoOption, MaybeUndefined, SkipListener};

#[cfg(feature = "unstable_mcp_over_acp")]
use super::mcp::{
    ConnectMcpRequest, ConnectMcpResponse, DisconnectMcpRequest, DisconnectMcpResponse,
    MCP_CONNECT_METHOD_NAME, MCP_DISCONNECT_METHOD_NAME, MCP_MESSAGE_METHOD_NAME,
    MessageMcpNotification, MessageMcpRequest, MessageMcpResponse,
};

#[cfg(feature = "unstable_nes")]
use super::{ClientNesCapabilities, PositionEncodingKind};

// Session updates

/// Notification containing a session update from the agent.
///
/// Used to stream real-time progress and results during prompt processing.
///
/// See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[schemars(extend("x-side" = "client", "x-method" = SESSION_UPDATE_NOTIFICATION))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionNotification {
    /// The ID of the session this update pertains to.
    pub session_id: SessionId,
    /// The actual update content.
    pub update: SessionUpdate,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionNotification {
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, update: SessionUpdate) -> Self {
        Self {
            session_id: session_id.into(),
            update,
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

/// Different types of updates that can be sent during session processing.
///
/// These updates provide real-time feedback about the agent's progress.
///
/// See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "sessionUpdate", rename_all = "snake_case")]
#[schemars(extend("discriminator" = {"propertyName": "sessionUpdate"}))]
#[non_exhaustive]
pub enum SessionUpdate {
    /// A chunk of the user's message being streamed.
    UserMessageChunk(ContentChunk),
    /// A chunk of the agent's response being streamed.
    AgentMessageChunk(ContentChunk),
    /// A chunk of the agent's internal reasoning being streamed.
    AgentThoughtChunk(ContentChunk),
    /// Notification that a new tool call has been initiated.
    ToolCall(ToolCall),
    /// Update on the status or results of a tool call.
    ToolCallUpdate(ToolCallUpdate),
    /// A content update for a plan identified by ID.
    /// See protocol docs: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)
    PlanUpdate(PlanUpdate),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Removal notice for a plan identified by ID.
    #[cfg(feature = "unstable_plan_operations")]
    PlanRemoved(PlanRemoved),
    /// Available commands are ready or have changed
    AvailableCommandsUpdate(AvailableCommandsUpdate),
    /// Session configuration options have been updated.
    ConfigOptionUpdate(ConfigOptionUpdate),
    /// Session metadata has been updated (title, timestamps, custom metadata)
    SessionInfoUpdate(SessionInfoUpdate),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Context window and cost update for the session.
    #[cfg(feature = "unstable_session_usage")]
    UsageUpdate(UsageUpdate),
    /// Custom or future session update.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    ///
    /// Receivers that do not understand this update type should preserve the
    /// raw payload when storing, replaying, proxying, or forwarding session
    /// history, and otherwise ignore it or display it generically.
    #[serde(untagged)]
    Other(OtherSessionUpdate),
}

/// Custom or future session update payload.
///
/// This preserves the unknown `sessionUpdate` discriminator and the rest of the
/// update object for clients that store, replay, proxy, or forward session
/// history.
#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq)]
#[schemars(inline)]
#[schemars(transform = other_session_update_schema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OtherSessionUpdate {
    /// Custom or future session update type.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    #[serde(rename = "sessionUpdate")]
    pub session_update: String,
    /// Additional fields from the unknown update payload.
    #[serde(flatten)]
    pub fields: BTreeMap<String, serde_json::Value>,
}

impl OtherSessionUpdate {
    #[must_use]
    pub fn new(
        session_update: impl Into<String>,
        mut fields: BTreeMap<String, serde_json::Value>,
    ) -> Self {
        fields.remove("sessionUpdate");
        Self {
            session_update: session_update.into(),
            fields,
        }
    }
}

impl<'de> Deserialize<'de> for OtherSessionUpdate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut fields = BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
        let session_update = fields
            .remove("sessionUpdate")
            .ok_or_else(|| serde::de::Error::missing_field("sessionUpdate"))?;
        let serde_json::Value::String(session_update) = session_update else {
            return Err(serde::de::Error::custom("`sessionUpdate` must be a string"));
        };

        if is_known_session_update(&session_update) {
            return Err(serde::de::Error::custom(format!(
                "known session update `{session_update}` did not match its schema"
            )));
        }

        Ok(Self {
            session_update,
            fields,
        })
    }
}

fn is_known_session_update(session_update: &str) -> bool {
    match session_update {
        "user_message_chunk"
        | "agent_message_chunk"
        | "agent_thought_chunk"
        | "tool_call"
        | "tool_call_update"
        | "plan_update"
        | "available_commands_update"
        | "config_option_update"
        | "session_info_update" => true,
        #[cfg(feature = "unstable_session_usage")]
        "usage_update" => true,
        _ => false,
    }
}

fn other_session_update_schema(schema: &mut Schema) {
    super::schema_util::reject_known_string_discriminators(
        schema,
        "sessionUpdate",
        &[
            "user_message_chunk",
            "agent_message_chunk",
            "agent_thought_chunk",
            "tool_call",
            "tool_call_update",
            "plan_update",
            "available_commands_update",
            "config_option_update",
            "session_info_update",
            #[cfg(feature = "unstable_plan_operations")]
            "plan_removed",
            #[cfg(feature = "unstable_session_usage")]
            "usage_update",
        ],
    );
}

/// Session configuration options have been updated.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ConfigOptionUpdate {
    /// The full set of configuration options and their current values.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, SkipListener>>")]
    #[schemars(extend("x-deserialize-default-on-error" = true, "x-deserialize-skip-invalid-items" = true))]
    pub config_options: Vec<SessionConfigOption>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ConfigOptionUpdate {
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

/// Update to session metadata. All fields are optional to support partial updates.
///
/// Agents send this notification to update session information like title or custom metadata.
/// This allows clients to display dynamic session names and track session state changes.
#[skip_serializing_none]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionInfoUpdate {
    /// Human-readable title for the session. Set to null to clear.
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub title: MaybeUndefined<String>,
    /// ISO 8601 timestamp of last activity. Set to null to clear.
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub updated_at: MaybeUndefined<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionInfoUpdate {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Human-readable title for the session. Set to null to clear.
    #[must_use]
    pub fn title(mut self, title: impl IntoMaybeUndefined<String>) -> Self {
        self.title = title.into_maybe_undefined();
        self
    }

    /// ISO 8601 timestamp of last activity. Set to null to clear.
    #[must_use]
    pub fn updated_at(mut self, updated_at: impl IntoMaybeUndefined<String>) -> Self {
        self.updated_at = updated_at.into_maybe_undefined();
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
/// Context window and cost update for a session.
#[cfg(feature = "unstable_session_usage")]
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UsageUpdate {
    /// Tokens currently in context.
    pub used: u64,
    /// Total context window size in tokens.
    pub size: u64,
    /// Cumulative session cost (optional).
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[schemars(extend("x-deserialize-default-on-error" = true))]
    #[serde(default)]
    pub cost: Option<Cost>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_usage")]
impl UsageUpdate {
    #[must_use]
    pub fn new(used: u64, size: u64) -> Self {
        Self {
            used,
            size,
            cost: None,
            meta: None,
        }
    }

    /// Cumulative session cost (optional).
    #[must_use]
    pub fn cost(mut self, cost: impl IntoOption<Cost>) -> Self {
        self.cost = cost.into_option();
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
/// Cost information for a session.
#[cfg(feature = "unstable_session_usage")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Cost {
    /// Total cumulative cost for session.
    pub amount: f64,
    /// ISO 4217 currency code (e.g., "USD", "EUR").
    pub currency: String,
}

#[cfg(feature = "unstable_session_usage")]
impl Cost {
    #[must_use]
    pub fn new(amount: f64, currency: impl Into<String>) -> Self {
        Self {
            amount,
            currency: currency.into(),
        }
    }
}

/// A streamed item of content
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ContentChunk {
    /// A single item of content
    pub content: ContentBlock,
    /// A unique identifier for the message this chunk belongs to.
    ///
    /// All chunks belonging to the same message share the same `messageId`.
    /// A change in `messageId` indicates a new message has started.
    pub message_id: MessageId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ContentChunk {
    #[must_use]
    pub fn new(content: ContentBlock, message_id: impl Into<MessageId>) -> Self {
        Self {
            content,
            message_id: message_id.into(),
            meta: None,
        }
    }

    /// A unique identifier for the message this chunk belongs to.
    ///
    /// All chunks belonging to the same message share the same `messageId`.
    /// A change in `messageId` indicates a new message has started.
    #[must_use]
    pub fn message_id(mut self, message_id: impl Into<MessageId>) -> Self {
        self.message_id = message_id.into();
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

/// Unique identifier for a message within a session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct MessageId(pub Arc<str>);

impl MessageId {
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// Available commands are ready or have changed
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AvailableCommandsUpdate {
    /// Commands the agent can execute
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, SkipListener>>")]
    #[schemars(extend("x-deserialize-default-on-error" = true, "x-deserialize-skip-invalid-items" = true))]
    pub available_commands: Vec<AvailableCommand>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl AvailableCommandsUpdate {
    #[must_use]
    pub fn new(available_commands: Vec<AvailableCommand>) -> Self {
        Self {
            available_commands,
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

/// Information about a command.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AvailableCommand {
    /// Command name (e.g., `create_plan`, `research_codebase`).
    pub name: String,
    /// Human-readable description of what the command does.
    pub description: String,
    /// Input for the command if required
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[schemars(extend("x-deserialize-default-on-error" = true))]
    #[serde(default)]
    pub input: Option<AvailableCommandInput>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl AvailableCommand {
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input: None,
            meta: None,
        }
    }

    /// Input for the command if required
    #[must_use]
    pub fn input(mut self, input: impl IntoOption<AvailableCommandInput>) -> Self {
        self.input = input.into_option();
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

/// The input specification for a command.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged, rename_all = "camelCase")]
#[non_exhaustive]
pub enum AvailableCommandInput {
    /// All text that was typed after the command name is provided as input.
    Unstructured(UnstructuredCommandInput),
    /// Custom or future command input specification.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    ///
    /// Clients that do not understand this input type should preserve the raw
    /// payload when storing, replaying, proxying, or forwarding command
    /// metadata, and otherwise ignore the input specification or display the
    /// command without structured input.
    Other(OtherAvailableCommandInput),
}

/// Custom or future command input specification.
#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(inline)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OtherAvailableCommandInput {
    /// Custom or future command input type.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    #[serde(rename = "type")]
    pub type_: String,
    /// Additional fields from the unknown command input payload.
    #[serde(flatten)]
    pub fields: BTreeMap<String, serde_json::Value>,
}

impl OtherAvailableCommandInput {
    #[must_use]
    pub fn new(type_: impl Into<String>, mut fields: BTreeMap<String, serde_json::Value>) -> Self {
        fields.remove("type");
        Self {
            type_: type_.into(),
            fields,
        }
    }
}

impl<'de> Deserialize<'de> for OtherAvailableCommandInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut fields = BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
        let type_ = fields
            .remove("type")
            .ok_or_else(|| serde::de::Error::missing_field("type"))?;
        let serde_json::Value::String(type_) = type_ else {
            return Err(serde::de::Error::custom("`type` must be a string"));
        };

        Ok(Self { type_, fields })
    }
}

/// All text that was typed after the command name is provided as input.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(transform = unstructured_command_input_schema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UnstructuredCommandInput {
    /// A hint to display when the input hasn't been provided yet
    pub hint: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl UnstructuredCommandInput {
    #[must_use]
    pub fn new(hint: impl Into<String>) -> Self {
        Self {
            hint: hint.into(),
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

impl<'de> Deserialize<'de> for UnstructuredCommandInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RawUnstructuredCommandInput {
            hint: String,
            #[serde(rename = "_meta")]
            meta: Option<Meta>,
            #[serde(flatten)]
            fields: BTreeMap<String, serde_json::Value>,
        }

        let raw = RawUnstructuredCommandInput::deserialize(deserializer)?;
        if raw.fields.contains_key("type") {
            return Err(serde::de::Error::custom(
                "unstructured command input cannot include a `type` field",
            ));
        }

        Ok(Self {
            hint: raw.hint,
            meta: raw.meta,
        })
    }
}

fn unstructured_command_input_schema(schema: &mut Schema) {
    super::schema_util::reject_property(schema, "type");
}

// Permission

/// Request for user permission to execute a tool call.
///
/// Sent when the agent needs authorization before performing a sensitive operation.
///
/// See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[schemars(extend("x-side" = "client", "x-method" = SESSION_REQUEST_PERMISSION_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RequestPermissionRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// Details about the tool call requiring permission.
    pub tool_call: ToolCallUpdate,
    /// Available permission options for the user to choose from.
    pub options: Vec<PermissionOption>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl RequestPermissionRequest {
    #[must_use]
    pub fn new(
        session_id: impl Into<SessionId>,
        tool_call: ToolCallUpdate,
        options: Vec<PermissionOption>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            tool_call,
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

/// An option presented to the user when requesting permission.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PermissionOption {
    /// Unique identifier for this permission option.
    pub option_id: PermissionOptionId,
    /// Human-readable label to display to the user.
    pub name: String,
    /// Hint about the nature of this permission option.
    pub kind: PermissionOptionKind,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl PermissionOption {
    #[must_use]
    pub fn new(
        option_id: impl Into<PermissionOptionId>,
        name: impl Into<String>,
        kind: PermissionOptionKind,
    ) -> Self {
        Self {
            option_id: option_id.into(),
            name: name.into(),
            kind,
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

/// Unique identifier for a permission option.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct PermissionOptionId(pub Arc<str>);

impl PermissionOptionId {
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// The type of permission option being presented to the user.
///
/// Helps clients choose appropriate icons and UI treatment.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PermissionOptionKind {
    /// Allow this operation only this time.
    AllowOnce,
    /// Allow this operation and remember the choice.
    AllowAlways,
    /// Reject this operation only this time.
    RejectOnce,
    /// Reject this operation and remember the choice.
    RejectAlways,
    /// Custom or future permission option kind.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    #[serde(untagged)]
    Other(String),
}

/// Response to a permission request.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "client", "x-method" = SESSION_REQUEST_PERMISSION_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RequestPermissionResponse {
    /// The user's decision on the permission request.
    // This extra-level is unfortunately needed because the output must be an object
    pub outcome: RequestPermissionOutcome,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl RequestPermissionResponse {
    #[must_use]
    pub fn new(outcome: RequestPermissionOutcome) -> Self {
        Self {
            outcome,
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

/// The outcome of a permission request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "snake_case")]
#[schemars(extend("discriminator" = {"propertyName": "outcome"}))]
#[non_exhaustive]
pub enum RequestPermissionOutcome {
    /// The prompt turn was cancelled before the user responded.
    ///
    /// When a client sends a `session/cancel` notification to cancel an ongoing
    /// prompt turn, it MUST respond to all pending `session/request_permission`
    /// requests with this `Cancelled` outcome.
    ///
    /// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
    Cancelled,
    /// The user selected one of the provided options.
    #[serde(rename_all = "camelCase")]
    Selected(SelectedPermissionOutcome),
}

/// The user selected one of the provided options.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SelectedPermissionOutcome {
    /// The ID of the option the user selected.
    pub option_id: PermissionOptionId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SelectedPermissionOutcome {
    #[must_use]
    pub fn new(option_id: impl Into<PermissionOptionId>) -> Self {
        Self {
            option_id: option_id.into(),
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

// Capabilities

/// Capabilities supported by the client.
///
/// Advertised during initialization to inform the agent about
/// available features and methods.
///
/// See protocol docs: [Client Capabilities](https://agentclientprotocol.com/protocol/initialization#client-capabilities)
#[serde_as]
#[skip_serializing_none]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ClientCapabilities {
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Authentication capabilities supported by the client.
    /// Determines which authentication method types the agent may include
    /// in its `InitializeResponse`.
    #[cfg(feature = "unstable_auth_methods")]
    #[serde(default)]
    pub auth: AuthCapabilities,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Elicitation capabilities supported by the client.
    /// Determines which elicitation modes the agent may use.
    #[cfg(feature = "unstable_elicitation")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[schemars(extend("x-deserialize-default-on-error" = true))]
    #[serde(default)]
    pub elicitation: Option<ElicitationCapabilities>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// NES (Next Edit Suggestions) capabilities supported by the client.
    #[cfg(feature = "unstable_nes")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[schemars(extend("x-deserialize-default-on-error" = true))]
    #[serde(default)]
    pub nes: Option<ClientNesCapabilities>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// The position encodings supported by the client, in order of preference.
    #[cfg(feature = "unstable_nes")]
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, SkipListener>>")]
    #[schemars(extend("x-deserialize-default-on-error" = true, "x-deserialize-skip-invalid-items" = true))]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub position_encodings: Vec<PositionEncodingKind>,

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ClientCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Authentication capabilities supported by the client.
    /// Determines which authentication method types the agent may include
    /// in its `InitializeResponse`.
    #[cfg(feature = "unstable_auth_methods")]
    #[must_use]
    pub fn auth(mut self, auth: AuthCapabilities) -> Self {
        self.auth = auth;
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Elicitation capabilities supported by the client.
    /// Determines which elicitation modes the agent may use.
    #[cfg(feature = "unstable_elicitation")]
    #[must_use]
    pub fn elicitation(mut self, elicitation: impl IntoOption<ElicitationCapabilities>) -> Self {
        self.elicitation = elicitation.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// NES (Next Edit Suggestions) capabilities supported by the client.
    #[cfg(feature = "unstable_nes")]
    #[must_use]
    pub fn nes(mut self, nes: impl IntoOption<ClientNesCapabilities>) -> Self {
        self.nes = nes.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// The position encodings supported by the client, in order of preference.
    #[cfg(feature = "unstable_nes")]
    #[must_use]
    pub fn position_encodings(mut self, position_encodings: Vec<PositionEncodingKind>) -> Self {
        self.position_encodings = position_encodings;
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
/// Authentication capabilities supported by the client.
///
/// Advertised during initialization to inform the agent which authentication
/// method types the client can handle. This governs opt-in types that require
/// additional client-side support.
#[cfg(feature = "unstable_auth_methods")]
#[serde_as]
#[skip_serializing_none]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AuthCapabilities {
    /// Whether the client supports `terminal` authentication methods.
    ///
    /// Optional. Omitted or `null` both mean the client does not advertise support.
    /// Supplying `{}` means the agent may include `terminal` entries in its authentication methods.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[schemars(extend("x-deserialize-default-on-error" = true))]
    #[serde(default)]
    pub terminal: Option<TerminalAuthCapabilities>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_auth_methods")]
impl AuthCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the client supports `terminal` authentication methods.
    ///
    /// Omitted or `null` both mean the client does not advertise support.
    /// Supplying `{}` means the agent may include `AuthMethod::Terminal` entries in its authentication methods.
    #[must_use]
    pub fn terminal(mut self, terminal: impl IntoOption<TerminalAuthCapabilities>) -> Self {
        self.terminal = terminal.into_option();
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
/// Capabilities for terminal authentication methods.
///
/// Supplying `{}` means the client supports terminal authentication methods.
#[cfg(feature = "unstable_auth_methods")]
#[skip_serializing_none]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[non_exhaustive]
pub struct TerminalAuthCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_auth_methods")]
impl TerminalAuthCapabilities {
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

// Method schema

/// Names of all methods that clients handle.
///
/// Provides a centralized definition of method names used in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct ClientMethodNames {
    /// Method for requesting permission from the user.
    pub session_request_permission: &'static str,
    /// Notification for session updates.
    pub session_update: &'static str,
    /// Method for opening an MCP-over-ACP connection.
    #[cfg(feature = "unstable_mcp_over_acp")]
    pub mcp_connect: &'static str,
    /// Method for exchanging MCP-over-ACP messages.
    #[cfg(feature = "unstable_mcp_over_acp")]
    pub mcp_message: &'static str,
    /// Method for closing an MCP-over-ACP connection.
    #[cfg(feature = "unstable_mcp_over_acp")]
    pub mcp_disconnect: &'static str,
    /// Method for elicitation.
    #[cfg(feature = "unstable_elicitation")]
    pub elicitation_create: &'static str,
    /// Notification for elicitation completion.
    #[cfg(feature = "unstable_elicitation")]
    pub elicitation_complete: &'static str,
}

/// Constant containing all client method names.
pub const CLIENT_METHOD_NAMES: ClientMethodNames = ClientMethodNames {
    session_update: SESSION_UPDATE_NOTIFICATION,
    session_request_permission: SESSION_REQUEST_PERMISSION_METHOD_NAME,
    #[cfg(feature = "unstable_mcp_over_acp")]
    mcp_connect: MCP_CONNECT_METHOD_NAME,
    #[cfg(feature = "unstable_mcp_over_acp")]
    mcp_message: MCP_MESSAGE_METHOD_NAME,
    #[cfg(feature = "unstable_mcp_over_acp")]
    mcp_disconnect: MCP_DISCONNECT_METHOD_NAME,
    #[cfg(feature = "unstable_elicitation")]
    elicitation_create: ELICITATION_CREATE_METHOD_NAME,
    #[cfg(feature = "unstable_elicitation")]
    elicitation_complete: ELICITATION_COMPLETE_NOTIFICATION,
};

/// Notification name for session updates.
pub(crate) const SESSION_UPDATE_NOTIFICATION: &str = "session/update";
/// Method name for requesting user permission.
pub(crate) const SESSION_REQUEST_PERMISSION_METHOD_NAME: &str = "session/request_permission";
/// Method name for elicitation.
#[cfg(feature = "unstable_elicitation")]
pub(crate) const ELICITATION_CREATE_METHOD_NAME: &str = "elicitation/create";
/// Notification name for elicitation completion.
#[cfg(feature = "unstable_elicitation")]
pub(crate) const ELICITATION_COMPLETE_NOTIFICATION: &str = "elicitation/complete";

/// All possible requests that an agent can send to a client.
///
/// This enum is used internally for routing RPC requests. You typically won't need
/// to use this directly.
///
/// This enum encompasses all method calls from agent to client.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[non_exhaustive]
pub enum AgentRequest {
    /// Requests permission from the user for a tool call operation.
    ///
    /// Called by the agent when it needs user authorization before executing
    /// a potentially sensitive operation. The client should present the options
    /// to the user and return their decision.
    ///
    /// If the client cancels the prompt turn via `session/cancel`, it MUST
    /// respond to this request with `RequestPermissionOutcome::Cancelled`.
    ///
    /// See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
    RequestPermissionRequest(RequestPermissionRequest),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Requests structured user input via a form or URL.
    #[cfg(feature = "unstable_elicitation")]
    CreateElicitationRequest(CreateElicitationRequest),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Opens an MCP-over-ACP connection.
    #[cfg(feature = "unstable_mcp_over_acp")]
    ConnectMcpRequest(ConnectMcpRequest),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Exchanges an MCP-over-ACP message.
    #[cfg(feature = "unstable_mcp_over_acp")]
    MessageMcpRequest(MessageMcpRequest),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Closes an MCP-over-ACP connection.
    #[cfg(feature = "unstable_mcp_over_acp")]
    DisconnectMcpRequest(DisconnectMcpRequest),
    /// Handles extension method requests from the agent.
    ///
    /// Allows the Agent to send an arbitrary request that is not part of the ACP spec.
    /// Extension methods provide a way to add custom functionality while maintaining
    /// protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    ExtMethodRequest(ExtRequest),
}

impl AgentRequest {
    /// Returns the corresponding method name of the request.
    #[must_use]
    pub fn method(&self) -> &str {
        match self {
            Self::RequestPermissionRequest(_) => CLIENT_METHOD_NAMES.session_request_permission,
            #[cfg(feature = "unstable_elicitation")]
            Self::CreateElicitationRequest(_) => CLIENT_METHOD_NAMES.elicitation_create,
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::ConnectMcpRequest(_) => CLIENT_METHOD_NAMES.mcp_connect,
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpRequest(_) => CLIENT_METHOD_NAMES.mcp_message,
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::DisconnectMcpRequest(_) => CLIENT_METHOD_NAMES.mcp_disconnect,
            Self::ExtMethodRequest(ext_request) => &ext_request.method,
        }
    }
}

/// All possible responses that a client can send to an agent.
///
/// This enum is used internally for routing RPC responses. You typically won't need
/// to use this directly - the responses are handled automatically by the connection.
///
/// These are responses to the corresponding `AgentRequest` variants.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[non_exhaustive]
pub enum ClientResponse {
    RequestPermissionResponse(RequestPermissionResponse),
    #[cfg(feature = "unstable_elicitation")]
    CreateElicitationResponse(CreateElicitationResponse),
    #[cfg(feature = "unstable_mcp_over_acp")]
    ConnectMcpResponse(ConnectMcpResponse),
    #[cfg(feature = "unstable_mcp_over_acp")]
    DisconnectMcpResponse(#[serde(default)] DisconnectMcpResponse),
    ExtMethodResponse(ExtResponse),
    #[cfg(feature = "unstable_mcp_over_acp")]
    MessageMcpResponse(MessageMcpResponse),
}

/// All possible notifications that an agent can send to a client.
///
/// This enum is used internally for routing RPC notifications. You typically won't need
/// to use this directly.
///
/// Notifications do not expect a response.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[expect(clippy::large_enum_variant)]
#[schemars(inline)]
#[non_exhaustive]
pub enum AgentNotification {
    /// Handles session update notifications from the agent.
    ///
    /// This is a notification endpoint (no response expected) that receives
    /// real-time updates about session progress, including message chunks,
    /// tool calls, and execution plans.
    ///
    /// Note: Clients SHOULD continue accepting tool call updates even after
    /// sending a `session/cancel` notification, as the agent may send final
    /// updates before responding with the cancelled stop reason.
    ///
    /// See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
    SessionNotification(SessionNotification),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Notification that a URL-based elicitation has completed.
    #[cfg(feature = "unstable_elicitation")]
    CompleteElicitationNotification(CompleteElicitationNotification),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Receives an MCP-over-ACP notification.
    #[cfg(feature = "unstable_mcp_over_acp")]
    MessageMcpNotification(MessageMcpNotification),
    /// Handles extension notifications from the agent.
    ///
    /// Allows the Agent to send an arbitrary notification that is not part of the ACP spec.
    /// Extension notifications provide a way to send one-way messages for custom functionality
    /// while maintaining protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    ExtNotification(ExtNotification),
}

impl AgentNotification {
    /// Returns the corresponding method name of the notification.
    #[must_use]
    pub fn method(&self) -> &str {
        match self {
            Self::SessionNotification(_) => CLIENT_METHOD_NAMES.session_update,
            #[cfg(feature = "unstable_elicitation")]
            Self::CompleteElicitationNotification(_) => CLIENT_METHOD_NAMES.elicitation_complete,
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpNotification(_) => CLIENT_METHOD_NAMES.mcp_message,
            Self::ExtNotification(ext_notification) => &ext_notification.method,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_behavior() {
        use serde_json::json;

        assert_eq!(
            serde_json::from_value::<SessionInfoUpdate>(json!({})).unwrap(),
            SessionInfoUpdate {
                title: MaybeUndefined::Undefined,
                updated_at: MaybeUndefined::Undefined,
                meta: None
            }
        );
        assert_eq!(
            serde_json::from_value::<SessionInfoUpdate>(json!({"title": null, "updatedAt": null}))
                .unwrap(),
            SessionInfoUpdate {
                title: MaybeUndefined::Null,
                updated_at: MaybeUndefined::Null,
                meta: None
            }
        );
        assert_eq!(
            serde_json::from_value::<SessionInfoUpdate>(
                json!({"title": "title", "updatedAt": "timestamp"})
            )
            .unwrap(),
            SessionInfoUpdate {
                title: MaybeUndefined::Value("title".to_string()),
                updated_at: MaybeUndefined::Value("timestamp".to_string()),
                meta: None
            }
        );

        assert_eq!(
            serde_json::to_value(SessionInfoUpdate::new()).unwrap(),
            json!({})
        );
        assert_eq!(
            serde_json::to_value(SessionInfoUpdate::new().title("title")).unwrap(),
            json!({"title": "title"})
        );
        assert_eq!(
            serde_json::to_value(SessionInfoUpdate::new().title(None)).unwrap(),
            json!({"title": null})
        );
        assert_eq!(
            serde_json::to_value(
                SessionInfoUpdate::new()
                    .title("title")
                    .title(MaybeUndefined::Undefined)
            )
            .unwrap(),
            json!({})
        );
    }

    #[test]
    fn session_update_preserves_unknown_variant() {
        use serde_json::json;

        let update: SessionUpdate = serde_json::from_value(json!({
            "sessionUpdate": "_status_badge",
            "label": "Indexing",
            "progress": 0.5
        }))
        .unwrap();

        let SessionUpdate::Other(unknown) = update else {
            panic!("expected unknown session update");
        };

        assert_eq!(unknown.session_update, "_status_badge");
        assert_eq!(unknown.fields.get("label"), Some(&json!("Indexing")));
        assert_eq!(unknown.fields.get("progress"), Some(&json!(0.5)));

        assert_eq!(
            serde_json::to_value(SessionUpdate::Other(unknown)).unwrap(),
            json!({
                "sessionUpdate": "_status_badge",
                "label": "Indexing",
                "progress": 0.5
            })
        );
    }

    #[test]
    fn test_plan_update_serialization() {
        use serde_json::json;

        let plan_update =
            SessionUpdate::PlanUpdate(PlanUpdate::new(crate::v2::PlanUpdateContent::items(
                "plan-1",
                vec![crate::v2::PlanEntry::new(
                    "Step 1",
                    crate::v2::PlanEntryPriority::High,
                    crate::v2::PlanEntryStatus::Pending,
                )],
            )));

        assert_eq!(
            serde_json::to_value(plan_update).unwrap(),
            json!({
                "sessionUpdate": "plan_update",
                "plan": {
                    "type": "items",
                    "id": "plan-1",
                    "entries": [
                        {
                            "content": "Step 1",
                            "priority": "high",
                            "status": "pending"
                        }
                    ]
                }
            })
        );
    }

    #[cfg(feature = "unstable_plan_operations")]
    #[test]
    fn test_plan_removed_serialization() {
        use serde_json::json;

        assert_eq!(
            serde_json::to_value(SessionUpdate::PlanRemoved(PlanRemoved::new("plan-1"))).unwrap(),
            json!({
                "sessionUpdate": "plan_removed",
                "id": "plan-1"
            })
        );
    }

    #[test]
    fn available_command_input_preserves_unknown_typed_variant() {
        use serde_json::json;

        let input: AvailableCommandInput = serde_json::from_value(json!({
            "type": "_choices",
            "hint": "Pick one",
            "options": ["fast", "careful"]
        }))
        .unwrap();

        let AvailableCommandInput::Other(unknown) = input else {
            panic!("expected unknown command input");
        };

        assert_eq!(unknown.type_, "_choices");
        assert_eq!(unknown.fields.get("hint"), Some(&json!("Pick one")));
        assert_eq!(
            unknown.fields.get("options"),
            Some(&json!(["fast", "careful"]))
        );
        assert_eq!(
            serde_json::to_value(AvailableCommandInput::Other(unknown)).unwrap(),
            json!({
                "type": "_choices",
                "hint": "Pick one",
                "options": ["fast", "careful"]
            })
        );
    }

    #[test]
    fn available_command_input_unknown_does_not_hide_malformed_unstructured_variant() {
        use serde_json::json;

        assert!(serde_json::from_value::<AvailableCommandInput>(json!({})).is_err());
        assert!(
            serde_json::from_value::<AvailableCommandInput>(json!({
                "type": 1,
                "hint": "Pick one"
            }))
            .is_err()
        );
    }

    #[cfg(feature = "unstable_nes")]
    #[test]
    fn test_client_capabilities_position_encodings_serialization() {
        use serde_json::json;

        let capabilities = ClientCapabilities::new().position_encodings(vec![
            PositionEncodingKind::Utf32,
            PositionEncodingKind::Utf16,
        ]);
        let json = serde_json::to_value(&capabilities).unwrap();

        assert_eq!(json["positionEncodings"], json!(["utf-32", "utf-16"]));
    }

    #[cfg(feature = "unstable_mcp_over_acp")]
    #[test]
    fn test_agent_mcp_request_method_names() {
        use serde_json::json;

        let params: serde_json::Map<String, serde_json::Value> =
            [("cursor".to_string(), json!("abc"))].into_iter().collect();

        assert_eq!(CLIENT_METHOD_NAMES.mcp_connect, "mcp/connect");
        assert_eq!(CLIENT_METHOD_NAMES.mcp_message, "mcp/message");
        assert_eq!(CLIENT_METHOD_NAMES.mcp_disconnect, "mcp/disconnect");

        assert_eq!(
            AgentRequest::ConnectMcpRequest(ConnectMcpRequest::new("server-1")).method(),
            "mcp/connect"
        );
        assert_eq!(
            AgentRequest::MessageMcpRequest(MessageMcpRequest::new("conn-1", "tools/list"))
                .method(),
            "mcp/message"
        );
        assert_eq!(
            AgentRequest::DisconnectMcpRequest(DisconnectMcpRequest::new("conn-1")).method(),
            "mcp/disconnect"
        );
        assert_eq!(
            AgentNotification::MessageMcpNotification(MessageMcpNotification::new(
                "conn-1",
                "notifications/progress"
            ))
            .method(),
            "mcp/message"
        );

        assert_eq!(
            serde_json::to_value(ConnectMcpRequest::new("server-1")).unwrap(),
            json!({ "acpId": "server-1" })
        );
        assert_eq!(
            serde_json::to_value(ConnectMcpResponse::new("conn-1")).unwrap(),
            json!({ "connectionId": "conn-1" })
        );
        assert_eq!(
            serde_json::to_value(MessageMcpRequest::new("conn-1", "tools/list").params(params))
                .unwrap(),
            json!({
                "connectionId": "conn-1",
                "method": "tools/list",
                "params": { "cursor": "abc" }
            })
        );
        assert_eq!(
            serde_json::to_value(DisconnectMcpRequest::new("conn-1")).unwrap(),
            json!({ "connectionId": "conn-1" })
        );
        assert_eq!(
            serde_json::to_value(MessageMcpNotification::new(
                "conn-1",
                "notifications/progress"
            ))
            .unwrap(),
            json!({
                "connectionId": "conn-1",
                "method": "notifications/progress"
            })
        );

        let request_with_null_params: MessageMcpRequest = serde_json::from_value(json!({
            "connectionId": "conn-1",
            "method": "tools/list",
            "params": null
        }))
        .unwrap();
        assert_eq!(request_with_null_params.params, None);
    }

    #[cfg(feature = "unstable_auth_methods")]
    #[test]
    fn test_auth_capabilities_serialize_terminal_support_as_object() {
        use serde_json::json;

        let capabilities = AuthCapabilities::new().terminal(TerminalAuthCapabilities::new());

        assert_eq!(
            serde_json::to_value(&capabilities).unwrap(),
            json!({
                "terminal": {}
            })
        );

        let deserialized: AuthCapabilities = serde_json::from_value(json!({
            "terminal": false
        }))
        .unwrap();
        assert!(deserialized.terminal.is_none());
    }
}
