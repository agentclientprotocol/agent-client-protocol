//! Methods and notifications the client handles/receives.
//!
//! This module defines the Client trait and all associated types for implementing
//! a client that interacts with AI coding agents via the Agent Client Protocol (ACP).

use std::{path::PathBuf, sync::Arc};

use derive_more::{Display, From};
#[cfg(all(feature = "schemars", feature = "unstable_subagents"))]
use schemars::Schema;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, VecSkipError, serde_as, skip_serializing_none};
#[cfg(feature = "unstable_subagents")]
use std::collections::BTreeMap;

#[cfg(feature = "unstable_subagents")]
use super::Error;
#[cfg(all(
    feature = "unstable_subagents",
    feature = "unstable_end_turn_token_usage"
))]
use super::Usage;
use super::{
    CompleteElicitationNotification, CreateElicitationRequest, CreateElicitationResponse,
    ElicitationCapabilities,
};
use crate::{IntoMaybeUndefined, IntoOption, MaybeUndefined};

use super::{
    ContentBlock, EnvVariable, ExtNotification, ExtRequest, ExtResponse, Meta, Plan,
    SessionConfigOption, SessionId, SessionModeId, ToolCall, ToolCallUpdate,
};
#[cfg(feature = "unstable_plan_operations")]
use super::{PlanCapabilities, PlanRemoved, PlanUpdate};

#[cfg(feature = "unstable_mcp_over_acp")]
use super::mcp::{MCP_MESSAGE_METHOD_NAME, MessageMcpRequest, MessageMcpResponse};

#[cfg(feature = "unstable_nes")]
use super::{ClientNesCapabilities, PositionEncodingKind};

// Session updates

/// Notification containing a session update from the agent.
///
/// Used to stream real-time progress and results during prompt processing.
///
/// See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = SESSION_UPDATE_NOTIFICATION)))]
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
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionNotification {
    /// Builds [`SessionNotification`] with the required notification fields set; optional fields start unset or empty.
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "sessionUpdate", rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", schemars(extend("discriminator" = {"propertyName": "sessionUpdate"})))]
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
    /// The agent's execution plan for complex tasks.
    /// See protocol docs: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)
    Plan(Plan),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// A content update for a plan identified by ID.
    #[cfg(feature = "unstable_plan_operations")]
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
    /// The current mode of the session has changed
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    CurrentModeUpdate(CurrentModeUpdate),
    /// Session configuration options have been updated.
    ConfigOptionUpdate(ConfigOptionUpdate),
    /// Session metadata has been updated (title, timestamps, custom metadata)
    SessionInfoUpdate(SessionInfoUpdate),
    /// Context window and cost update for the session.
    UsageUpdate(UsageUpdate),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Advisory information for the user that is not part of session history.
    ///
    /// Agents MUST only send this update when the Client advertised
    /// [`ClientSessionCapabilities::notices`].
    #[cfg(feature = "unstable_session_notices")]
    Notice(Notice),
    /// A context compaction has been created or updated.
    ///
    /// Agents MUST only send this update when the Client advertised
    /// [`ClientSessionCapabilities::compaction`].
    CompactionUpdate(CompactionUpdate),
    /// A content block appended to a context compaction's retained summary.
    ///
    /// Agents MUST only send this update when the Client advertised
    /// [`ClientSessionCapabilities::compaction`].
    CompactionSummaryChunk(CompactionSummaryChunk),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Announces a child session created and owned by this session, or updates
    /// that ownership association's metadata.
    #[cfg(feature = "unstable_subagents")]
    SubagentUpdate(SubagentUpdate),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// A message upsert observed in this session's transcript, sent to or
    /// received from another session.
    #[cfg(feature = "unstable_subagents")]
    SessionMessage(SessionMessage),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// One content block appended to a sent or received session message.
    #[cfg(feature = "unstable_subagents")]
    SessionMessageChunk(SessionMessageChunk),
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A streamed content block of an inter-session message.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionMessageChunk {
    /// Identifier of this message within the enclosing session's transcript.
    pub message_id: MessageId,
    /// Optional sending session identity; omission or `null` retains a known value.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub sender_session_id: Option<SessionId>,
    /// Optional receiving session identity; omission or `null` retains a known value.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub recipient_session_id: Option<SessionId>,
    /// A single content block appended to the message.
    pub content: ContentBlock,
    /// Optional and nullable chunk-scoped metadata; omitted or `null` means none.
    ///
    /// Implementations MUST NOT make assumptions about values in `_meta`.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_subagents")]
impl SessionMessageChunk {
    /// Builds a single streamed content block without chunk metadata.
    #[must_use]
    pub fn new(message_id: impl Into<MessageId>, content: ContentBlock) -> Self {
        Self {
            message_id: message_id.into(),
            sender_session_id: None,
            recipient_session_id: None,
            content,
            meta: None,
        }
    }

    /// Supplies the sending session identity, when known.
    #[must_use]
    pub fn sender_session_id(mut self, sender_session_id: impl IntoOption<SessionId>) -> Self {
        self.sender_session_id = sender_session_id.into_option();
        self
    }

    /// Supplies the receiving session identity, when known.
    #[must_use]
    pub fn recipient_session_id(
        mut self,
        recipient_session_id: impl IntoOption<SessionId>,
    ) -> Self {
        self.recipient_session_id = recipient_session_id.into_option();
        self
    }

    /// Sets optional chunk-scoped metadata.
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
/// An upsert for an inter-session message.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionMessage {
    /// Identifier of this message within the enclosing session's transcript.
    pub message_id: MessageId,
    /// Optional sending session identity; omission or `null` retains a known value.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub sender_session_id: Option<SessionId>,
    /// Optional receiving session identity; omission or `null` retains a known value.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub recipient_session_id: Option<SessionId>,
    /// Omitted leaves content unchanged; `null` or `[]` clears it.
    /// A non-empty array replaces all content.
    #[serde_as(deserialize_as = "DefaultOnError<MaybeUndefined<VecSkipError<_>>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true, "x-deserialize-skip-invalid-items" = true)))]
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub content: MaybeUndefined<Vec<ContentBlock>>,
    /// Omitted leaves metadata unchanged; `null` removes it.
    ///
    /// Implementations MUST NOT make assumptions about values in `_meta`.
    #[serde_as(deserialize_as = "DefaultOnError<MaybeUndefined<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(
        default,
        rename = "_meta",
        skip_serializing_if = "MaybeUndefined::is_undefined"
    )]
    pub meta: MaybeUndefined<Meta>,
}

#[cfg(feature = "unstable_subagents")]
impl SessionMessage {
    /// Builds a message upsert with its transcript-local ID.
    #[must_use]
    pub fn new(message_id: impl Into<MessageId>) -> Self {
        Self {
            message_id: message_id.into(),
            sender_session_id: None,
            recipient_session_id: None,
            content: MaybeUndefined::Undefined,
            meta: MaybeUndefined::Undefined,
        }
    }

    /// Supplies the sending session identity, when known.
    #[must_use]
    pub fn sender_session_id(mut self, sender_session_id: impl IntoOption<SessionId>) -> Self {
        self.sender_session_id = sender_session_id.into_option();
        self
    }

    /// Supplies the receiving session identity, when known.
    #[must_use]
    pub fn recipient_session_id(
        mut self,
        recipient_session_id: impl IntoOption<SessionId>,
    ) -> Self {
        self.recipient_session_id = recipient_session_id.into_option();
        self
    }

    /// Replaces, clears, or omits the complete content patch.
    #[must_use]
    pub fn content(mut self, content: impl IntoMaybeUndefined<Vec<ContentBlock>>) -> Self {
        self.content = content.into_maybe_undefined();
        self
    }

    /// Sets, clears, or omits the metadata patch.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoMaybeUndefined<Meta>) -> Self {
        self.meta = meta.into_maybe_undefined();
        self
    }
}

#[cfg(all(test, feature = "unstable_subagents"))]
mod session_message_tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn envelopes_preserve_participants_and_multimodal_content() {
        let content = json!([
            {"type": "text", "text": "Please inspect this"},
            {"type": "image", "data": "aGVsbG8=", "mimeType": "image/png"}
        ]);
        for (transcript, sender, recipient, id) in [
            ("parent", "parent", "child", "sent-1"),
            ("child", "parent", "child", "received-9"),
            ("child", "child", "parent", "sent-2"),
        ] {
            let wire = json!({"sessionId": transcript, "update": {
                "sessionUpdate": "session_message", "messageId": id,
                "senderSessionId": sender, "recipientSessionId": recipient, "content": content
            }});
            let decoded: SessionNotification = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(&decoded).unwrap(), wire);
            assert!(matches!(decoded.update, SessionUpdate::SessionMessage(_)));
            let message = SessionMessage::new(id)
                .sender_session_id(SessionId::new(sender))
                .recipient_session_id(SessionId::new(recipient))
                .content(vec![]);
            assert_eq!(serde_json::to_value(message).unwrap()["content"], json!([]));
        }
    }

    #[test]
    fn message_upserts_validate_ids_and_patch_fields() {
        let base = json!({"sessionUpdate": "session_message", "messageId": "m1",
            "senderSessionId": "parent", "recipientSessionId": "child", "content": []});
        {
            let key = "messageId";
            let mut missing = base.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<SessionUpdate>(missing).is_err(),
                "{key}"
            );
            let mut null = base.clone();
            null[key] = Value::Null;
            assert!(
                serde_json::from_value::<SessionUpdate>(null).is_err(),
                "{key}"
            );
            let mut non_string = base.clone();
            non_string[key] = json!(42);
            assert!(
                serde_json::from_value::<SessionUpdate>(non_string).is_err(),
                "{key}"
            );
        }
        for content in [None, Some(json!({}))] {
            let mut wire = base.clone();
            wire.as_object_mut().unwrap().remove("content");
            if let Some(content) = content {
                wire["content"] = content;
            }
            let decoded: SessionUpdate = serde_json::from_value(wire).unwrap();
            let encoded = serde_json::to_value(decoded).unwrap();
            let mut unchanged = base.clone();
            unchanged.as_object_mut().unwrap().remove("content");
            assert_eq!(encoded, unchanged);
        }
        let clear = SessionMessage::new("m1")
            .sender_session_id(SessionId::new("parent"))
            .recipient_session_id(SessionId::new("child"))
            .content(None)
            .meta(None);
        let mut clear_wire = base.clone();
        clear_wire["content"] = Value::Null;
        clear_wire["_meta"] = Value::Null;
        assert_eq!(
            serde_json::to_value(&clear).unwrap()["content"],
            Value::Null
        );
        let clear_update: SessionUpdate = serde_json::from_value(clear_wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(clear_update).unwrap(), clear_wire);
        assert!(clear.content.is_null());
        assert!(clear.meta.is_null());
        for meta in [None, Some(Value::Null), Some(json!({"tag": "value"}))] {
            let mut wire = base.clone();
            if let Some(meta) = meta {
                wire["_meta"] = meta;
            }
            let decoded: SessionUpdate = serde_json::from_value(wire.clone()).unwrap();
            let encoded = serde_json::to_value(decoded).unwrap();
            assert_eq!(encoded, wire);
        }
        let metadata_only = json!({"sessionUpdate": "session_message",
            "messageId": "m1", "senderSessionId": "parent",
            "recipientSessionId": "child", "_meta": {"tag": "value"}});
        let decoded: SessionUpdate = serde_json::from_value(metadata_only.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), metadata_only);
        let unset = SessionMessage::new("m1");
        assert_eq!(
            serde_json::to_value(&unset).unwrap(),
            json!({"messageId": "m1"})
        );
        assert!(unset.content.is_undefined());
        assert!(unset.meta.is_undefined());
        let reset = SessionMessage::new("m1").content(vec![]);
        assert_eq!(serde_json::to_value(reset).unwrap()["content"], json!([]));
        let malformed: SessionMessage = serde_json::from_value(json!({
            "messageId": "m1", "senderSessionId": "parent", "recipientSessionId": "child",
            "content": false, "_meta": false
        }))
        .unwrap();
        assert!(malformed.content.is_undefined());
        assert!(malformed.meta.is_undefined());
    }

    #[test]
    fn first_chunk_and_upsert_share_transcript_local_identity() {
        for (transcript, id) in [("parent", "sent-1"), ("child", "received-9")] {
            let wire = json!({"sessionId": transcript, "update": {
                "sessionUpdate": "session_message_chunk", "messageId": id,
                "senderSessionId": "parent", "recipientSessionId": "child",
                "content": {"type": "text", "text": "first"}
            }});
            let decoded: SessionNotification = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
            let upsert = SessionMessage::new(id)
                .sender_session_id(SessionId::new("parent"))
                .recipient_session_id(SessionId::new("child"));
            let chunk = SessionMessageChunk::new(
                id,
                ContentBlock::Text(crate::v1::TextContent::new("first")),
            )
            .sender_session_id(SessionId::new("parent"))
            .recipient_session_id(SessionId::new("child"));
            assert_eq!(chunk.message_id, upsert.message_id);
            assert_eq!(chunk.sender_session_id, upsert.sender_session_id);
            assert_eq!(chunk.recipient_session_id, upsert.recipient_session_id);
            assert_eq!(
                serde_json::to_value(SessionUpdate::SessionMessageChunk(chunk)).unwrap(),
                wire["update"]
            );
        }
        let wire = json!({"sessionId": "child", "update": {
            "sessionUpdate": "session_message_chunk", "messageId": "received-9",
            "senderSessionId": "parent", "recipientSessionId": "child",
            "content": {"type": "text", "text": "first"}
        }});
        let base = wire["update"].clone();
        for key in ["messageId", "content"] {
            let mut missing = base.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<SessionUpdate>(missing).is_err(),
                "{key}"
            );
            let mut null = base.clone();
            null[key] = Value::Null;
            assert!(
                serde_json::from_value::<SessionUpdate>(null).is_err(),
                "{key}"
            );
            let mut non_string = base.clone();
            non_string[key] = json!(42);
            assert!(
                serde_json::from_value::<SessionUpdate>(non_string).is_err(),
                "{key}"
            );
        }
        for meta in [None, Some(Value::Null), Some(json!({"chunk": true}))] {
            let mut value = base.clone();
            if let Some(meta) = meta {
                value["_meta"] = meta;
            }
            let decoded: SessionUpdate = serde_json::from_value(value.clone()).unwrap();
            let encoded = serde_json::to_value(decoded).unwrap();
            if value["_meta"].is_null() {
                assert_eq!(encoded, base);
            } else {
                assert_eq!(encoded, value);
            }
        }
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn schema_requires_ids_and_chunk_content() {
        let schema = serde_json::to_value(schemars::schema_for!(SessionMessage)).unwrap();
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required, &vec![json!("messageId")]);
        let chunk = serde_json::to_value(schemars::schema_for!(SessionMessageChunk)).unwrap();
        let required = chunk["required"].as_array().unwrap();
        assert_eq!(required.len(), 2);
        assert!(required.contains(&json!("messageId")));
        assert!(required.contains(&json!("content")));
    }

    #[test]
    fn endpoints_can_arrive_late_or_be_omitted_from_later_events() {
        let block = ContentBlock::Text(crate::v1::TextContent::new("hello"));
        let minimal = SessionMessage::new("m1");
        let first = SessionMessageChunk::new("m1", block.clone());
        assert_eq!(
            serde_json::to_value(&minimal).unwrap(),
            json!({"messageId": "m1"})
        );
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            json!({"messageId": "m1", "content": {"type": "text", "text": "hello"}})
        );
        let enriched = SessionMessage::new("m1")
            .sender_session_id(SessionId::new("parent"))
            .recipient_session_id(SessionId::new("child"));
        assert_eq!(enriched.sender_session_id, Some(SessionId::new("parent")));
        assert_eq!(enriched.recipient_session_id, Some(SessionId::new("child")));
        let later =
            SessionMessageChunk::new("m1", block).sender_session_id(SessionId::new("parent"));
        assert_eq!(later.sender_session_id, Some(SessionId::new("parent")));
        assert_eq!(later.recipient_session_id, None);
        for update in [
            SessionUpdate::SessionMessage(minimal),
            SessionUpdate::SessionMessageChunk(first),
            SessionUpdate::SessionMessage(enriched),
            SessionUpdate::SessionMessageChunk(later),
        ] {
            let wire = serde_json::to_value(&update).unwrap();
            let decoded: SessionUpdate = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
        }
    }

    #[test]
    fn invalid_or_null_endpoints_are_absent_but_message_id_is_required() {
        for (kind, content) in [
            ("session_message", None),
            (
                "session_message_chunk",
                Some(json!({"type": "text", "text": "hello"})),
            ),
        ] {
            let mut base = json!({"sessionUpdate": kind, "messageId": "m1",
                "senderSessionId": null, "recipientSessionId": 42});
            if let Some(content) = content {
                base["content"] = content;
            }
            let decoded: SessionUpdate = serde_json::from_value(base.clone()).unwrap();
            let encoded = serde_json::to_value(decoded).unwrap();
            base.as_object_mut().unwrap().remove("senderSessionId");
            base.as_object_mut().unwrap().remove("recipientSessionId");
            assert_eq!(encoded, base);
            for bad in [Value::Null, json!(42)] {
                let mut invalid = base.clone();
                invalid["messageId"] = bad;
                assert!(serde_json::from_value::<SessionUpdate>(invalid).is_err());
            }
        }
    }
}

#[cfg(all(test, not(feature = "unstable_subagents")))]
mod disabled_session_message_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn message_updates_require_subagents_gate() {
        for (kind, content) in [
            ("session_message", json!([])),
            (
                "session_message_chunk",
                json!({"type": "text", "text": "hello"}),
            ),
        ] {
            let wire = json!({"sessionUpdate": kind, "messageId": "m1", "content": content});
            assert!(serde_json::from_value::<SessionUpdate>(wire).is_err());
        }
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Severity hint for a session notice.
#[cfg(feature = "unstable_session_notices")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum NoticeSeverity {
    /// Informational notice.
    Info,
    /// Warning notice.
    Warning,
    /// Error notice.
    Error,
    /// Custom or future notice severity.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Other unknown values are reserved for future ACP severities.
    #[serde(untagged)]
    Other(String),
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Fire-and-forget advisory information for the user.
///
/// Notices are live events rather than session history. Agents must not rely on
/// a notice being received, displayed, or seen by the user.
/// Agents MUST only send notices when the Client advertised
/// [`ClientSessionCapabilities::notices`]. Otherwise, Agents may use an agent
/// message when the information should still be surfaced to the user.
///
/// See RFD: [Session Notices](https://agentclientprotocol.com/rfds/session-notices)
#[cfg(feature = "unstable_session_notices")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Notice {
    /// Presentation severity hint.
    pub severity: NoticeSeverity,
    /// Required non-empty plain-text title that can stand alone.
    #[cfg_attr(feature = "schemars", schemars(length(min = 1)))]
    pub title: String,
    /// Optional plain-text detail or guidance.
    ///
    /// Omitted and `null` are equivalent and mean no description was supplied.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub description: Option<String>,
    /// Metadata scoped to this notice.
    ///
    /// Omitted and `null` are equivalent and mean no metadata was supplied.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_session_notices")]
impl Notice {
    /// Builds a notice with the required fields set and optional fields omitted.
    #[must_use]
    pub fn new(severity: NoticeSeverity, title: impl Into<String>) -> Self {
        Self {
            severity,
            title: title.into(),
            description: None,
            meta: None,
        }
    }

    /// Sets or clears the optional description.
    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// Sets or clears notice-scoped metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Unique identifier for a context compaction within a session.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct CompactionId(pub Arc<str>);

impl CompactionId {
    /// Wraps a protocol string as a typed [`CompactionId`].
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// Lifecycle state of a context compaction.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CompactionStatus {
    /// Compaction has started and has not finished.
    #[default]
    InProgress,
    /// Compaction finished successfully.
    Completed,
    /// Compaction finished unsuccessfully.
    Failed,
    /// Compaction was cancelled before it finished.
    Cancelled,
    /// Custom or future compaction status.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Other unknown values are reserved for future ACP statuses.
    #[serde(untagged)]
    Other(String),
}

/// A context compaction upsert. The first notification fixes the compaction's
/// timeline position. Later updates with the same ID patch that entity in place.
/// Agents MUST only send this update when the Client advertised
/// [`ClientSessionCapabilities::compaction`].
///
/// `summary`, `error`, and `_meta` have patch semantics: omission leaves the
/// stored value unchanged, `null` clears it, and a concrete value replaces it.
/// `summary: []` also clears the summary.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CompactionUpdate {
    /// The Agent-owned ID of this compaction, unique within the session.
    pub compaction_id: CompactionId,
    /// Current lifecycle status.
    pub status: CompactionStatus,
    /// Complete replacement user-displayable summary content for the compaction.
    #[serde_as(deserialize_as = "DefaultOnError<MaybeUndefined<VecSkipError<_>>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true, "x-deserialize-skip-invalid-items" = true)))]
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub summary: MaybeUndefined<Vec<ContentBlock>>,
    /// Human-readable error details for the compaction.
    #[serde_as(deserialize_as = "DefaultOnError<MaybeUndefined<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub error: MaybeUndefined<String>,
    /// Extensible metadata patch for this compaction.
    #[serde_as(deserialize_as = "DefaultOnError<MaybeUndefined<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(
        rename = "_meta",
        default,
        skip_serializing_if = "MaybeUndefined::is_undefined"
    )]
    pub meta: MaybeUndefined<Meta>,
}

impl CompactionUpdate {
    /// Builds a compaction update with optional patch fields omitted.
    #[must_use]
    pub fn new(compaction_id: impl Into<CompactionId>, status: CompactionStatus) -> Self {
        Self {
            compaction_id: compaction_id.into(),
            status,
            summary: MaybeUndefined::Undefined,
            error: MaybeUndefined::Undefined,
            meta: MaybeUndefined::Undefined,
        }
    }

    /// Sets, clears, or omits the complete retained summary patch.
    #[must_use]
    pub fn summary(mut self, summary: impl IntoMaybeUndefined<Vec<ContentBlock>>) -> Self {
        self.summary = summary.into_maybe_undefined();
        self
    }

    /// Sets, clears, or omits the error details patch.
    #[must_use]
    pub fn error(mut self, error: impl IntoMaybeUndefined<String>) -> Self {
        self.error = error.into_maybe_undefined();
        self
    }

    /// Sets, clears, or omits the metadata patch.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoMaybeUndefined<Meta>) -> Self {
        self.meta = meta.into_maybe_undefined();
        self
    }
}

/// A content block appended to a compaction's summary. A first-seen ID creates
/// an in-progress compaction. Chunks append in receive order.
/// Agents MUST only send this update when the Client advertised
/// [`ClientSessionCapabilities::compaction`].
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CompactionSummaryChunk {
    /// ID of the compaction whose summary receives this content.
    pub compaction_id: CompactionId,
    /// One content block to append.
    pub content: ContentBlock,
    /// Metadata scoped to this chunk. Omission and `null` both mean absent.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, rename = "_meta")]
    pub meta: Option<Meta>,
}

impl CompactionSummaryChunk {
    /// Builds a summary chunk without metadata.
    #[must_use]
    pub fn new(compaction_id: impl Into<CompactionId>, content: ContentBlock) -> Self {
        Self {
            compaction_id: compaction_id.into(),
            content,
            meta: None,
        }
    }

    /// Sets or clears chunk-scoped metadata.
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
/// Notification that the enclosing parent session created and owns a child session.
///
/// Later updates modify the existing association's metadata, not its ownership.
///
/// Sent on the immediate parent session. The first update for an unknown
/// [`SubagentUpdate::session_id`] announces the child and MUST be sent
/// before any live child traffic or live message naming the child as sender
/// or recipient. Parents may message and reuse an announced child across
/// multiple operations.
/// Child events are delivered automatically on the same connection; no child
/// load, resume, or subscription is needed.
///
/// Only the subagent session ID is required. Omitted patch fields keep their
/// previous values; `null` clears them. Clearing capabilities disables child
/// mutations. Clearing state leaves current activity unset/unconfirmed: it does
/// not imply idle, stop work, or create an `unknown` state snapshot. A concrete
/// state replaces the entire previous state object, not the session.
/// The title and description provide the parent's display metadata for the
/// child. They do not replace the content of individual messages or operations.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SubagentUpdate {
    /// The opaque session ID identifying the child in all ACP messages.
    pub session_id: SessionId,
    /// The parent's human-readable display title for this child. It need not be unique.
    ///
    /// Omitted means unchanged; `null` clears it. If unset, the Client chooses
    /// a fallback presentation.
    #[serde_as(deserialize_as = "DefaultOnError<MaybeUndefined<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub title: MaybeUndefined<String>,
    /// The parent's human-readable description of the child's role or purpose.
    ///
    /// Omitted means unchanged; `null` clears it. If unset, the Client chooses
    /// a fallback presentation. This is current display metadata, not the
    /// history of instructions sent to the child.
    #[serde_as(deserialize_as = "DefaultOnError<MaybeUndefined<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub description: MaybeUndefined<String>,
    /// Client-initiated session mutations permitted for this subagent session.
    ///
    /// Omitted means unchanged; `null` clears the capability set and disables
    /// child mutations. If never supplied, no session mutations are permitted.
    /// Read-only operations retain their normal protocol semantics and capability
    /// requirements. A concrete object replaces the whole capability set.
    #[serde_as(deserialize_as = "DefaultOnError<MaybeUndefined<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub capabilities: MaybeUndefined<SubagentSessionCapabilities>,
    /// Current state snapshot for the child session.
    ///
    /// Omitted means unchanged; `null` clears the current activity without
    /// asserting idle or sending an `unknown` snapshot. A concrete state
    /// replaces the previous state object wholesale. If never supplied, the
    /// current activity is unset/unconfirmed.
    #[serde_as(deserialize_as = "DefaultOnError<MaybeUndefined<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub state: MaybeUndefined<StateUpdate>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    /// Omitted means unchanged; `null` removes the metadata.
    #[serde_as(deserialize_as = "DefaultOnError<MaybeUndefined<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(
        default,
        rename = "_meta",
        skip_serializing_if = "MaybeUndefined::is_undefined"
    )]
    pub meta: MaybeUndefined<Meta>,
}

#[cfg(feature = "unstable_subagents")]
impl SubagentUpdate {
    /// Builds a subagent upsert with only its required session ID set.
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>) -> Self {
        Self {
            session_id: session_id.into(),
            title: MaybeUndefined::Undefined,
            description: MaybeUndefined::Undefined,
            capabilities: MaybeUndefined::Undefined,
            state: MaybeUndefined::Undefined,
            meta: MaybeUndefined::Undefined,
        }
    }

    /// Sets, clears, or omits the parent's display title patch.
    #[must_use]
    pub fn title(mut self, title: impl IntoMaybeUndefined<String>) -> Self {
        self.title = title.into_maybe_undefined();
        self
    }

    /// Sets, clears, or omits the parent's description patch.
    #[must_use]
    pub fn description(mut self, description: impl IntoMaybeUndefined<String>) -> Self {
        self.description = description.into_maybe_undefined();
        self
    }

    /// Replaces, clears, or omits the permitted client-initiated mutations patch.
    #[must_use]
    pub fn capabilities(
        mut self,
        capabilities: impl IntoMaybeUndefined<SubagentSessionCapabilities>,
    ) -> Self {
        self.capabilities = capabilities.into_maybe_undefined();
        self
    }

    /// Replaces, clears, or omits the current activity patch.
    #[must_use]
    pub fn state(mut self, state: impl IntoMaybeUndefined<StateUpdate>) -> Self {
        self.state = state.into_maybe_undefined();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys. Sets, clears, or omits this metadata patch.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoMaybeUndefined<Meta>) -> Self {
        self.meta = meta.into_maybe_undefined();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Client-initiated session mutations permitted for a specific subagent session.
///
/// A mutation requires an explicit per-child capability; support for the method
/// on ordinary sessions does not grant support on a child.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SubagentSessionCapabilities {
    /// Permits the client to cancel this child's current work without ending
    /// the session. Omitted or `null` means unsupported; an object (including
    /// `{}`) means supported.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub cancel: Option<SessionCancelCapabilities>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_subagents")]
impl SubagentSessionCapabilities {
    /// Builds an empty capability set; cancellation is disabled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets or removes permission to cancel this child's current work.
    #[must_use]
    pub fn cancel(mut self, cancel: impl IntoOption<SessionCancelCapabilities>) -> Self {
        self.cancel = cancel.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
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
/// Capability to cancel work in a subagent session without ending that session.
///
/// Supplying `{}` advertises support; an omitted or `null` `cancel` does not.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct SessionCancelCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_subagents")]
impl SessionCancelCapabilities {
    /// Builds an empty capability object advertising cancellation support.
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
/// Current foreground-work state of a reusable child session.
///
/// Each update is a whole-object snapshot. Idle does not terminate the child;
/// the parent can message it again, transitioning it back to running.
/// Background activity may still emit other session updates while idle.
#[cfg(feature = "unstable_subagents")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
#[non_exhaustive]
pub enum StateUpdate {
    /// Foreground work is in progress.
    Running(RunningStateUpdate),
    /// The child is ready to process another prompt.
    Idle(IdleStateUpdate),
    /// Foreground work is blocked on user action.
    RequiresAction(RequiresActionStateUpdate),
    /// The Agent cannot currently determine foreground activity.
    ///
    /// This replaces previously confirmed activity without ending the work or session.
    Unknown(UnknownStateUpdate),
    /// Custom or future state.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Other unknown values are reserved for future ACP variants.
    #[serde(untagged)]
    Other(OtherStateUpdate),
}

/// Foreground work is in progress.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RunningStateUpdate {
    /// The _meta property is reserved by ACP for additional metadata.
    /// Implementations MUST NOT make assumptions about values at these keys.
    /// Optional; omitted and `null` mean no metadata for this state snapshot.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_subagents")]
impl RunningStateUpdate {
    /// Builds an empty running state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reserved metadata for extensions.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// The child is ready to process another prompt.
///
/// An omitted, `null`, or malformed `stopReason` means not reported.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(transform = idle_state_update_schema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct IdleStateUpdate {
    /// Reason foreground work stopped, with any fields of that stop reason.
    #[serde(flatten)]
    pub stop_reason: Option<IdleStopReason>,
    /// **UNSTABLE** Token usage for completed foreground work.
    ///
    /// Optional; omitted or `null` means not reported.
    #[cfg(feature = "unstable_end_turn_token_usage")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub usage: Option<Box<Usage>>,
    /// The _meta property is reserved by ACP for additional metadata.
    /// Implementations MUST NOT make assumptions about values at these keys.
    /// Optional; omitted and `null` mean no metadata for this state snapshot.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_subagents")]
impl IdleStateUpdate {
    /// Builds an empty idle state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reason foreground work stopped.
    #[must_use]
    pub fn stop_reason(mut self, stop_reason: impl IntoOption<IdleStopReason>) -> Self {
        self.stop_reason = stop_reason.into_option();
        self
    }

    /// Token usage for completed foreground work.
    #[cfg(feature = "unstable_end_turn_token_usage")]
    #[must_use]
    pub fn usage(mut self, usage: impl IntoOption<Usage>) -> Self {
        self.usage = usage.into_option().map(Box::new);
        self
    }

    /// Reserved metadata for extensions.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Makes the schema check stop reasons and keeps their default-on-error hint.
///
/// Flattening `Option<IdleStopReason>` adds an empty `{}` branch to the `anyOf`. It
/// matches any value, so the stop reason branches would check nothing; this
/// replaces it with a branch for an omitted or `null` `stopReason`. Flattening
/// also removes the `stopReason` property that carried
/// `x-deserialize-default-on-error`, which SDKs generated from the schema rely on
/// to tolerate a malformed value, so this adds it back as a shared property.
#[cfg(all(feature = "unstable_subagents", feature = "schemars"))]
fn idle_state_update_schema(schema: &mut Schema) {
    if let Some(properties) = schema
        .get_mut("properties")
        .and_then(serde_json::Value::as_object_mut)
    {
        properties.insert(
            "stopReason".into(),
            serde_json::json!({
                "description": "Why foreground work stopped. The value selects one of this type's variants, which may add fields of their own.\n\nOptional. Omitted or `null` both mean the agent is not reporting a stop reason; a malformed value is treated the same way.\n\nSee protocol docs: [Current work state](https://agentclientprotocol.com/rfds/subagents#current-work-state)",
                "type": ["string", "null"],
                "x-deserialize-default-on-error": true
            }),
        );
    }
    let Some(variants) = schema
        .get_mut("anyOf")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return;
    };
    for variant in variants {
        if variant.as_object().is_some_and(serde_json::Map::is_empty) {
            *variant = serde_json::json!({
                "title": "none",
                "description": "No stop reason: `stopReason` is omitted or `null`.",
                "type": "object",
                "properties": {
                    "stopReason": { "type": "null" }
                }
            });
        }
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Reason a child's foreground work stopped, reported on its idle state snapshot.
///
/// Tagged by `stopReason` and flattened into [`IdleStateUpdate`], so the snapshot
/// matches v2's idle `state_update`, including its handling of unknown values.
/// The known values are those of [`StopReason`](super::StopReason) plus `error`.
/// A child has no prompt response of its own, so this snapshot is the only place
/// to report that its work failed. The prompt response keeps
/// [`StopReason`](super::StopReason) and reports failures as JSON-RPC errors.
#[cfg(feature = "unstable_subagents")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "stopReason", rename_all = "snake_case")]
#[non_exhaustive]
pub enum IdleStopReason {
    /// The work ended successfully.
    EndTurn,
    /// The work ended because the agent reached the maximum number of tokens.
    MaxTokens,
    /// The work ended because the agent reached the maximum number of allowed
    /// agent requests.
    MaxTurnRequests,
    /// The work ended because the agent refused to continue.
    Refusal,
    /// The work was cancelled.
    Cancelled,
    /// The work ended because something failed.
    Error(ErrorStopReason),
    /// Custom or future stop reason.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Other unknown values are reserved for future ACP variants.
    #[serde(untagged)]
    Other(OtherStopReason),
}

#[cfg(feature = "unstable_subagents")]
impl From<ErrorStopReason> for IdleStopReason {
    fn from(value: ErrorStopReason) -> Self {
        Self::Error(value)
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Details of a failure that ended a child's foreground work.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ErrorStopReason {
    /// The failure, as a JSON-RPC error object.
    ///
    /// Optional. Omitted or `null` both mean the agent is not reporting failure details.
    /// Agents SHOULD include it.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub error: Option<Box<Error>>,
}

#[cfg(feature = "unstable_subagents")]
impl ErrorStopReason {
    /// Builds [`ErrorStopReason`] with the required fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The failure, as a JSON-RPC error object.
    #[must_use]
    pub fn error(mut self, error: impl IntoOption<Error>) -> Self {
        self.error = error.into_option().map(Box::new);
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Custom or future stop reason payload, preserving its discriminator and fields.
#[cfg(feature = "unstable_subagents")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", schemars(inline))]
#[cfg_attr(feature = "schemars", schemars(transform = other_stop_reason_schema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OtherStopReason {
    /// Unrecognized stop reason.
    pub stop_reason: String,
    /// Remaining fields of the unknown stop reason.
    #[serde(flatten)]
    pub fields: BTreeMap<String, serde_json::Value>,
}

#[cfg(feature = "unstable_subagents")]
impl OtherStopReason {
    /// Builds a custom stop reason, preserving its extension fields.
    #[must_use]
    pub fn new(
        stop_reason: impl Into<String>,
        mut fields: BTreeMap<String, serde_json::Value>,
    ) -> Self {
        remove_idle_state_update_fields(&mut fields);
        Self {
            stop_reason: stop_reason.into(),
            fields,
        }
    }
}

#[cfg(feature = "unstable_subagents")]
impl<'de> Deserialize<'de> for OtherStopReason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut fields = BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
        let stop_reason = fields
            .remove("stopReason")
            .ok_or_else(|| serde::de::Error::missing_field("stopReason"))?;
        let serde_json::Value::String(stop_reason) = stop_reason else {
            return Err(serde::de::Error::custom("`stopReason` must be a string"));
        };
        if KNOWN_STOP_REASONS.contains(&stop_reason.as_str()) {
            return Err(serde::de::Error::custom(format!(
                "known stop reason `{stop_reason}` did not match its schema"
            )));
        }
        remove_idle_state_update_fields(&mut fields);
        Ok(Self {
            stop_reason,
            fields,
        })
    }
}

#[cfg(feature = "unstable_subagents")]
const KNOWN_STOP_REASONS: &[&str] = &[
    "end_turn",
    "max_tokens",
    "max_turn_requests",
    "refusal",
    "cancelled",
    "error",
];

/// Keeps the idle snapshot's own fields out of an unknown stop reason's payload.
#[cfg(feature = "unstable_subagents")]
fn remove_idle_state_update_fields(fields: &mut BTreeMap<String, serde_json::Value>) {
    fields.remove("stopReason");
    fields.remove("usage");
    fields.remove("_meta");
}

#[cfg(all(feature = "unstable_subagents", feature = "schemars"))]
fn other_stop_reason_schema(schema: &mut Schema) {
    let known = KNOWN_STOP_REASONS
        .iter()
        .map(|stop_reason| {
            serde_json::json!({
                "properties": { "stopReason": { "const": stop_reason, "type": "string" } },
                "required": ["stopReason"],
                "type": "object"
            })
        })
        .collect::<Vec<_>>();
    schema.insert("not".into(), serde_json::json!({ "anyOf": known }));
}

/// Foreground work is blocked on user action.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RequiresActionStateUpdate {
    /// The _meta property is reserved by ACP for additional metadata.
    /// Implementations MUST NOT make assumptions about values at these keys.
    /// Optional; omitted and `null` mean no metadata for this state snapshot.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_subagents")]
impl RequiresActionStateUpdate {
    /// Builds an empty requires-action state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reserved metadata for extensions.
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
/// The Agent cannot currently determine foreground activity.
///
/// Report this when activity becomes unobservable, not merely because the child
/// has been quiet. The Client MUST stop presenting the previous state as confirmed
/// current activity, but may retain it as last known. A later state replaces this
/// snapshot normally.
///
/// This is not a task outcome or session closure. It does not cancel work, resolve
/// pending requests, or revoke capabilities; capabilities are updated separately.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UnknownStateUpdate {
    /// The _meta property is reserved by ACP for additional metadata.
    /// Implementations MUST NOT make assumptions about values at these keys.
    /// Optional; omitted and `null` mean no metadata for this state snapshot.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_subagents")]
impl UnknownStateUpdate {
    /// Builds an unknown-activity state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets optional metadata for this state snapshot.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Custom or future state payload, preserving its discriminator and fields.
#[cfg(feature = "unstable_subagents")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "schemars", schemars(inline))]
#[cfg_attr(feature = "schemars", schemars(transform = other_state_update_schema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OtherStateUpdate {
    /// Unrecognized state discriminator.
    pub state: String,
    /// Remaining fields in the state object.
    #[serde(flatten)]
    pub fields: BTreeMap<String, serde_json::Value>,
}

#[cfg(feature = "unstable_subagents")]
impl OtherStateUpdate {
    /// Builds a custom state, preserving its extension fields.
    #[must_use]
    pub fn new(state: impl Into<String>, mut fields: BTreeMap<String, serde_json::Value>) -> Self {
        fields.remove("state");
        Self {
            state: state.into(),
            fields,
        }
    }
}

#[cfg(feature = "unstable_subagents")]
impl<'de> Deserialize<'de> for OtherStateUpdate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut fields = BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
        let state = fields
            .remove("state")
            .ok_or_else(|| serde::de::Error::missing_field("state"))?;
        let serde_json::Value::String(state) = state else {
            return Err(serde::de::Error::custom("`state` must be a string"));
        };
        if is_known_state_update(&state) {
            return Err(serde::de::Error::custom(format!(
                "known state update `{state}` did not match its schema"
            )));
        }
        Ok(Self { state, fields })
    }
}

#[cfg(feature = "unstable_subagents")]
const KNOWN_STATE_UPDATE_STATES: &[&str] = &["running", "idle", "requires_action", "unknown"];

#[cfg(feature = "unstable_subagents")]
fn is_known_state_update(state: &str) -> bool {
    KNOWN_STATE_UPDATE_STATES.contains(&state)
}

#[cfg(all(feature = "unstable_subagents", feature = "schemars"))]
fn other_state_update_schema(schema: &mut Schema) {
    let known = KNOWN_STATE_UPDATE_STATES
        .iter()
        .map(|state| {
            serde_json::json!({
                "properties": { "state": { "const": state, "type": "string" } },
                "required": ["state"],
                "type": "object"
            })
        })
        .collect::<Vec<_>>();
    schema.insert("not".into(), serde_json::json!({ "anyOf": known }));
}

/// The current mode of the session has changed
///
/// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CurrentModeUpdate {
    /// The ID of the current mode
    pub current_mode_id: SessionModeId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl CurrentModeUpdate {
    /// Builds [`CurrentModeUpdate`] with the required fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(current_mode_id: impl Into<SessionModeId>) -> Self {
        Self {
            current_mode_id: current_mode_id.into(),
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

/// Session configuration options have been updated.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ConfigOptionUpdate {
    /// The full set of configuration options and their current values.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true, "x-deserialize-skip-invalid-items" = true)))]
    pub config_options: Vec<SessionConfigOption>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ConfigOptionUpdate {
    /// Builds [`ConfigOptionUpdate`] with the required fields set; optional fields start unset or empty.
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
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionInfoUpdate {
    /// Human-readable title for the session. Set to null to clear.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub title: MaybeUndefined<String>,
    /// ISO 8601 timestamp of last activity. Set to null to clear.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub updated_at: MaybeUndefined<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionInfoUpdate {
    /// Builds [`SessionInfoUpdate`] with the required fields set; optional fields start unset or empty.
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

/// Context window and cost update for a session.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UsageUpdate {
    /// Tokens currently in context.
    pub used: u64,
    /// Total context window size in tokens.
    pub size: u64,
    /// Cumulative session cost (optional).
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub cost: Option<Cost>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl UsageUpdate {
    /// Builds [`UsageUpdate`] with the required fields set; optional fields start unset or empty.
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

/// Cost information for a session.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Cost {
    /// Total cumulative cost for session.
    pub amount: f64,
    /// ISO 4217 currency code (e.g., "USD", "EUR").
    pub currency: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Cost {
    /// Builds [`Cost`] with the required fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(amount: f64, currency: impl Into<String>) -> Self {
        Self {
            amount,
            currency: currency.into(),
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

/// A streamed item of content
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ContentChunk {
    /// A single item of content
    pub content: ContentBlock,
    /// A unique identifier for the message this chunk belongs to.
    ///
    /// All chunks belonging to the same message share the same `messageId`.
    /// A change in `messageId` indicates a new message has started.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub message_id: Option<MessageId>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ContentChunk {
    /// Builds [`ContentChunk`] with the required fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(content: ContentBlock) -> Self {
        Self {
            content,
            message_id: None,
            meta: None,
        }
    }

    /// A unique identifier for the message this chunk belongs to.
    ///
    /// All chunks belonging to the same message share the same `messageId`.
    /// A change in `messageId` indicates a new message has started.
    #[must_use]
    pub fn message_id(mut self, message_id: impl IntoOption<MessageId>) -> Self {
        self.message_id = message_id.into_option();
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct MessageId(pub Arc<str>);

impl MessageId {
    /// Wraps a protocol string as a typed [`MessageId`].
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

impl IntoOption<MessageId> for &str {
    fn into_option(self) -> Option<MessageId> {
        Some(MessageId::new(self))
    }
}

/// Available commands are ready or have changed
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AvailableCommandsUpdate {
    /// Commands the agent can execute
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true, "x-deserialize-skip-invalid-items" = true)))]
    pub available_commands: Vec<AvailableCommand>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl AvailableCommandsUpdate {
    /// Builds [`AvailableCommandsUpdate`] with the required fields set; optional fields start unset or empty.
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AvailableCommand {
    /// Command name (e.g., `create_plan`, `research_codebase`).
    pub name: String,
    /// Human-readable description of what the command does.
    pub description: String,
    /// Input for the command if required
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub input: Option<AvailableCommandInput>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl AvailableCommand {
    /// Builds [`AvailableCommand`] with the required fields set; optional fields start unset or empty.
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged, rename_all = "camelCase")]
#[non_exhaustive]
pub enum AvailableCommandInput {
    /// All text that was typed after the command name is provided as input.
    Unstructured(UnstructuredCommandInput),
}

/// All text that was typed after the command name is provided as input.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl UnstructuredCommandInput {
    /// Builds [`UnstructuredCommandInput`] with the required fields set; optional fields start unset or empty.
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

// Permission

/// Request for user permission to execute a tool call.
///
/// Sent when the agent needs authorization before performing a sensitive operation.
///
/// See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = SESSION_REQUEST_PERMISSION_METHOD_NAME)))]
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
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl RequestPermissionRequest {
    /// Builds [`RequestPermissionRequest`] with the required request fields set; optional fields start unset or empty.
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
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl PermissionOption {
    /// Builds [`PermissionOption`] with the required fields set; optional fields start unset or empty.
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct PermissionOptionId(pub Arc<str>);

impl PermissionOptionId {
    /// Wraps a protocol string as a typed [`PermissionOptionId`].
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// The type of permission option being presented to the user.
///
/// Helps clients choose appropriate icons and UI treatment.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
}

/// Response to a permission request.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = SESSION_REQUEST_PERMISSION_METHOD_NAME)))]
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
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl RequestPermissionResponse {
    /// Builds [`RequestPermissionResponse`] with the required response fields set; optional fields start unset or empty.
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", schemars(extend("discriminator" = {"propertyName": "outcome"})))]
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
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SelectedPermissionOutcome {
    /// Builds [`SelectedPermissionOutcome`] with the required fields set; optional fields start unset or empty.
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

// Write text file

/// Request to write content to a text file.
///
/// Only available if the client supports the `fs.writeTextFile` capability.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = FS_WRITE_TEXT_FILE_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WriteTextFileRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// Absolute path to the file to write.
    pub path: PathBuf,
    /// The text content to write to the file.
    pub content: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl WriteTextFileRequest {
    /// Builds [`WriteTextFileRequest`] with the required request fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(
        session_id: impl Into<SessionId>,
        path: impl Into<PathBuf>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            path: path.into(),
            content: content.into(),
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

crate::serde_util::default_on_null! {
    /// Response to `fs/write_text_file`
    #[serde_as]
    #[skip_serializing_none]
    #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
    #[derive(Default, Debug, Clone, Serialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = FS_WRITE_TEXT_FILE_METHOD_NAME)))]
    #[non_exhaustive]
    pub struct WriteTextFileResponse {
        /// The _meta property is reserved by ACP to allow clients and agents to attach additional
        /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
        /// these keys.
        ///
        /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
        #[serde_as(deserialize_as = "DefaultOnError")]
        #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
        #[serde(default)]
        #[serde(rename = "_meta")]
        pub meta: Option<Meta>,
    }
}

impl WriteTextFileResponse {
    /// Builds [`WriteTextFileResponse`] with the required response fields set; optional fields start unset or empty.
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

// Read text file

/// Request to read content from a text file.
///
/// Only available if the client supports the `fs.readTextFile` capability.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = FS_READ_TEXT_FILE_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ReadTextFileRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// Absolute path to the file to read.
    pub path: PathBuf,
    /// Line number to start reading from (1-based).
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub line: Option<u32>,
    /// Maximum number of lines to read.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub limit: Option<u32>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ReadTextFileRequest {
    /// Builds [`ReadTextFileRequest`] with the required request fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, path: impl Into<PathBuf>) -> Self {
        Self {
            session_id: session_id.into(),
            path: path.into(),
            line: None,
            limit: None,
            meta: None,
        }
    }

    /// Line number to start reading from (1-based).
    #[must_use]
    pub fn line(mut self, line: impl IntoOption<u32>) -> Self {
        self.line = line.into_option();
        self
    }

    /// Maximum number of lines to read.
    #[must_use]
    pub fn limit(mut self, limit: impl IntoOption<u32>) -> Self {
        self.limit = limit.into_option();
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

/// Response containing the contents of a text file.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = FS_READ_TEXT_FILE_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ReadTextFileResponse {
    /// Content payload returned by this response.
    pub content: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ReadTextFileResponse {
    /// Builds [`ReadTextFileResponse`] with the required response fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
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

// Terminals

/// Typed identifier used for terminal values on the wire.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct TerminalId(pub Arc<str>);

impl TerminalId {
    /// Wraps a protocol string as a typed [`TerminalId`].
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// Request to create a new terminal and execute a command.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = TERMINAL_CREATE_METHOD_NAME)))]
#[non_exhaustive]
pub struct CreateTerminalRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// The command to execute.
    pub command: String,
    /// Array of command arguments.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true, "x-deserialize-skip-invalid-items" = true)))]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Environment variables for the command.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true, "x-deserialize-skip-invalid-items" = true)))]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<EnvVariable>,
    /// Working directory for the command. Must be an absolute path.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub cwd: Option<PathBuf>,
    /// Maximum number of output bytes to retain.
    ///
    /// When the limit is exceeded, the Client truncates from the beginning of the output
    /// to stay within the limit.
    ///
    /// The Client MUST ensure truncation happens at a character boundary to maintain valid
    /// string output, even if this means the retained output is slightly less than the
    /// specified limit.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub output_byte_limit: Option<u64>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl CreateTerminalRequest {
    /// Builds [`CreateTerminalRequest`] with the required request fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, command: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            command: command.into(),
            args: Vec::new(),
            env: Vec::new(),
            cwd: None,
            output_byte_limit: None,
            meta: None,
        }
    }

    /// Array of command arguments.
    #[must_use]
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Environment variables for the command.
    #[must_use]
    pub fn env(mut self, env: Vec<EnvVariable>) -> Self {
        self.env = env;
        self
    }

    /// Working directory for the command. Must be an absolute path.
    #[must_use]
    pub fn cwd(mut self, cwd: impl IntoOption<PathBuf>) -> Self {
        self.cwd = cwd.into_option();
        self
    }

    /// Maximum number of output bytes to retain.
    ///
    /// When the limit is exceeded, the Client truncates from the beginning of the output
    /// to stay within the limit.
    ///
    /// The Client MUST ensure truncation happens at a character boundary to maintain valid
    /// string output, even if this means the retained output is slightly less than the
    /// specified limit.
    #[must_use]
    pub fn output_byte_limit(mut self, output_byte_limit: impl IntoOption<u64>) -> Self {
        self.output_byte_limit = output_byte_limit.into_option();
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

/// Response containing the ID of the created terminal.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = TERMINAL_CREATE_METHOD_NAME)))]
#[non_exhaustive]
pub struct CreateTerminalResponse {
    /// The unique identifier for the created terminal.
    pub terminal_id: TerminalId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl CreateTerminalResponse {
    /// Builds [`CreateTerminalResponse`] with the required response fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(terminal_id: impl Into<TerminalId>) -> Self {
        Self {
            terminal_id: terminal_id.into(),
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

/// Request to get the current output and status of a terminal.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = TERMINAL_OUTPUT_METHOD_NAME)))]
#[non_exhaustive]
pub struct TerminalOutputRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// The ID of the terminal to get output from.
    pub terminal_id: TerminalId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl TerminalOutputRequest {
    /// Builds [`TerminalOutputRequest`] with the required request fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, terminal_id: impl Into<TerminalId>) -> Self {
        Self {
            session_id: session_id.into(),
            terminal_id: terminal_id.into(),
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

/// Response containing the terminal output and exit status.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = TERMINAL_OUTPUT_METHOD_NAME)))]
#[non_exhaustive]
pub struct TerminalOutputResponse {
    /// The terminal output captured so far.
    pub output: String,
    /// Whether the output was truncated due to byte limits.
    pub truncated: bool,
    /// Exit status if the command has completed.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub exit_status: Option<TerminalExitStatus>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl TerminalOutputResponse {
    /// Builds [`TerminalOutputResponse`] with the required response fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(output: impl Into<String>, truncated: bool) -> Self {
        Self {
            output: output.into(),
            truncated,
            exit_status: None,
            meta: None,
        }
    }

    /// Exit status if the command has completed.
    #[must_use]
    pub fn exit_status(mut self, exit_status: impl IntoOption<TerminalExitStatus>) -> Self {
        self.exit_status = exit_status.into_option();
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

/// Request to release a terminal and free its resources.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = TERMINAL_RELEASE_METHOD_NAME)))]
#[non_exhaustive]
pub struct ReleaseTerminalRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// The ID of the terminal to release.
    pub terminal_id: TerminalId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ReleaseTerminalRequest {
    /// Builds [`ReleaseTerminalRequest`] with the required request fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, terminal_id: impl Into<TerminalId>) -> Self {
        Self {
            session_id: session_id.into(),
            terminal_id: terminal_id.into(),
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

crate::serde_util::default_on_null! {
    /// Response to terminal/release method
    #[serde_as]
    #[skip_serializing_none]
    #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
    #[derive(Default, Debug, Clone, Serialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = TERMINAL_RELEASE_METHOD_NAME)))]
    #[non_exhaustive]
    pub struct ReleaseTerminalResponse {
        /// The _meta property is reserved by ACP to allow clients and agents to attach additional
        /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
        /// these keys.
        ///
        /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
        #[serde_as(deserialize_as = "DefaultOnError")]
        #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
        #[serde(default)]
        #[serde(rename = "_meta")]
        pub meta: Option<Meta>,
    }
}

impl ReleaseTerminalResponse {
    /// Builds [`ReleaseTerminalResponse`] with the required response fields set; optional fields start unset or empty.
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

/// Request to kill a terminal without releasing it.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = TERMINAL_KILL_METHOD_NAME)))]
#[non_exhaustive]
pub struct KillTerminalRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// The ID of the terminal to kill.
    pub terminal_id: TerminalId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl KillTerminalRequest {
    /// Builds [`KillTerminalRequest`] with the required request fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, terminal_id: impl Into<TerminalId>) -> Self {
        Self {
            session_id: session_id.into(),
            terminal_id: terminal_id.into(),
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

crate::serde_util::default_on_null! {
    /// Response to `terminal/kill` method
    #[serde_as]
    #[skip_serializing_none]
    #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
    #[derive(Default, Debug, Clone, Serialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = TERMINAL_KILL_METHOD_NAME)))]
    #[non_exhaustive]
    pub struct KillTerminalResponse {
        /// The _meta property is reserved by ACP to allow clients and agents to attach additional
        /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
        /// these keys.
        ///
        /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
        #[serde_as(deserialize_as = "DefaultOnError")]
        #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
        #[serde(default)]
        #[serde(rename = "_meta")]
        pub meta: Option<Meta>,
    }
}

impl KillTerminalResponse {
    /// Builds [`KillTerminalResponse`] with the required response fields set; optional fields start unset or empty.
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

/// Request to wait for a terminal command to exit.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = TERMINAL_WAIT_FOR_EXIT_METHOD_NAME)))]
#[non_exhaustive]
pub struct WaitForTerminalExitRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// The ID of the terminal to wait for.
    pub terminal_id: TerminalId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl WaitForTerminalExitRequest {
    /// Builds [`WaitForTerminalExitRequest`] with the required request fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, terminal_id: impl Into<TerminalId>) -> Self {
        Self {
            session_id: session_id.into(),
            terminal_id: terminal_id.into(),
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

crate::serde_util::default_on_null! {
    /// Response containing the exit status of a terminal command.
    #[serde_as]
    #[skip_serializing_none]
    #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
    #[derive(Default, Debug, Clone, Serialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = TERMINAL_WAIT_FOR_EXIT_METHOD_NAME)))]
    #[non_exhaustive]
    pub struct WaitForTerminalExitResponse {
        /// The exit status of the terminal command.
        #[serde(flatten)]
        pub exit_status: TerminalExitStatus,
        /// The _meta property is reserved by ACP to allow clients and agents to attach additional
        /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
        /// these keys.
        ///
        /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
        #[serde_as(deserialize_as = "DefaultOnError")]
        #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
        #[serde(default)]
        #[serde(rename = "_meta")]
        pub meta: Option<Meta>,
    }
}

impl WaitForTerminalExitResponse {
    /// Builds [`WaitForTerminalExitResponse`] with the required response fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(exit_status: TerminalExitStatus) -> Self {
        Self {
            exit_status,
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

/// Exit status of a terminal command.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TerminalExitStatus {
    /// The process exit code (may be null if terminated by signal).
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub exit_code: Option<u32>,
    /// The signal that terminated the process (may be null if exited normally).
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub signal: Option<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl TerminalExitStatus {
    /// Builds [`TerminalExitStatus`] with the required fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The process exit code (may be null if terminated by signal).
    #[must_use]
    pub fn exit_code(mut self, exit_code: impl IntoOption<u32>) -> Self {
        self.exit_code = exit_code.into_option();
        self
    }

    /// The signal that terminated the process (may be null if exited normally).
    #[must_use]
    pub fn signal(mut self, signal: impl IntoOption<String>) -> Self {
        self.signal = signal.into_option();
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

// Capabilities

/// Capabilities supported by the client.
///
/// Advertised during initialization to inform the agent about
/// available features and methods.
///
/// See protocol docs: [Client Capabilities](https://agentclientprotocol.com/protocol/initialization#client-capabilities)
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ClientCapabilities {
    /// File system capabilities supported by the client.
    /// Determines which file operations the agent can request.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub fs: FileSystemCapabilities,
    /// Whether the Client support all `terminal/*` methods.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub terminal: bool,
    /// Session-related capabilities supported by the client.
    ///
    /// Optional. Omitted or `null` both mean the client does not advertise any
    /// session-related extensions.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub session: Option<ClientSessionCapabilities>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Whether the client understands exposed subagent sessions.
    ///
    /// Optional and nullable. Omitted or `null` both mean the client does not
    /// advertise support.
    /// Supplying `{}` means the client understands child associations, work-state
    /// snapshots, session-directed messages, and restricted-session semantics.
    #[cfg(feature = "unstable_subagents")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub subagents: Option<SubagentCapabilities>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Whether the client supports `plan_update` and `plan_removed` session updates.
    ///
    /// Optional. Omitted or `null` both mean the client does not advertise support.
    /// Supplying `{}` means the client can receive both update types.
    #[cfg(feature = "unstable_plan_operations")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub plan: Option<PlanCapabilities>,
    /// Authentication capabilities supported by the client.
    /// Determines which authentication method types the agent may include
    /// in its `InitializeResponse`.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub auth: AuthCapabilities,
    /// Elicitation capabilities supported by the client.
    /// Determines which elicitation modes the agent may use.
    ///
    /// Optional. Omitted or `null` both mean the client does not advertise
    /// elicitation support.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub elicitation: Option<ElicitationCapabilities>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// NES (Next Edit Suggestions) capabilities supported by the client.
    ///
    /// Optional. Omitted or `null` both mean the client does not advertise any
    /// NES suggestion-kind extensions.
    #[cfg(feature = "unstable_nes")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub nes: Option<ClientNesCapabilities>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// The position encodings supported by the client, in order of preference.
    #[cfg(feature = "unstable_nes")]
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_>>")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true, "x-deserialize-skip-invalid-items" = true)))]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub position_encodings: Vec<PositionEncodingKind>,

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ClientCapabilities {
    /// Builds an empty [`ClientCapabilities`]; use builder methods to advertise supported sub-capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// File system capabilities supported by the client.
    /// Determines which file operations the agent can request.
    #[must_use]
    pub fn fs(mut self, fs: FileSystemCapabilities) -> Self {
        self.fs = fs;
        self
    }

    /// Whether the Client support all `terminal/*` methods.
    #[must_use]
    pub fn terminal(mut self, terminal: bool) -> Self {
        self.terminal = terminal;
        self
    }

    /// Session-related capabilities supported by the client.
    #[must_use]
    pub fn session(mut self, session: impl IntoOption<ClientSessionCapabilities>) -> Self {
        self.session = session.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Whether the client understands exposed subagent sessions.
    #[cfg(feature = "unstable_subagents")]
    #[must_use]
    pub fn subagents(mut self, subagents: impl IntoOption<SubagentCapabilities>) -> Self {
        self.subagents = subagents.into_option();
        self
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Whether the client supports `plan_update` and `plan_removed` session updates.
    ///
    /// Omitted or `null` both mean the client does not advertise support.
    /// Supplying `{}` means the client can receive both update types.
    #[cfg(feature = "unstable_plan_operations")]
    #[must_use]
    pub fn plan(mut self, plan: impl IntoOption<PlanCapabilities>) -> Self {
        self.plan = plan.into_option();
        self
    }

    /// Authentication capabilities supported by the client.
    /// Determines which authentication method types the agent may include
    /// in its `InitializeResponse`.
    #[must_use]
    pub fn auth(mut self, auth: AuthCapabilities) -> Self {
        self.auth = auth;
        self
    }

    /// Elicitation capabilities supported by the client.
    /// Determines which elicitation modes the agent may use.
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
/// Capability marker for exposing reusable child sessions as restricted ACP sessions.
///
/// Supplying `{}` advertises support for child association and state updates,
/// session-directed messages, and restricted-session semantics. The client
/// must advertise this capability before the agent sends subagent updates or
/// session-directed messages.
#[cfg(feature = "unstable_subagents")]
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct SubagentCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_subagents")]
impl SubagentCapabilities {
    /// Builds an empty capability marker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Session-related capabilities supported by the client.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ClientSessionCapabilities {
    /// Support for ID-addressed context compaction updates. Omitted or `null`
    /// means unsupported; `{}` advertises the complete compaction contract.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub compaction: Option<CompactionCapabilities>,
    /// Config option capabilities supported by the client.
    ///
    /// Omitted or `null` both mean the client does not advertise support for any
    /// config option extensions.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub config_options: Option<SessionConfigOptionsCapabilities>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Support for live advisory `notice` session updates.
    ///
    /// Optional. Omitted or `null` both mean the client does not advertise support.
    /// Supplying `{}` means the client can present notices to the user.
    #[cfg(feature = "unstable_session_notices")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub notices: Option<NoticeCapabilities>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ClientSessionCapabilities {
    /// Builds an empty [`ClientSessionCapabilities`]; use builder methods to advertise supported sub-capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Advertises support for ID-addressed context compaction updates.
    #[must_use]
    pub fn compaction(mut self, compaction: impl IntoOption<CompactionCapabilities>) -> Self {
        self.compaction = compaction.into_option();
        self
    }

    /// Config option capabilities supported by the client.
    ///
    /// Omitted or `null` both mean the client does not advertise support for any
    /// config option extensions.
    #[must_use]
    pub fn config_options(
        mut self,
        config_options: impl IntoOption<SessionConfigOptionsCapabilities>,
    ) -> Self {
        self.config_options = config_options.into_option();
        self
    }

    /// Advertises support for presenting live advisory notices to the user.
    #[cfg(feature = "unstable_session_notices")]
    #[must_use]
    pub fn notices(mut self, notices: impl IntoOption<NoticeCapabilities>) -> Self {
        self.notices = notices.into_option();
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

/// Client support for ID-addressed context compaction updates.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CompactionCapabilities {}

impl CompactionCapabilities {
    /// Advertises the complete compaction update contract.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Client support for presenting live advisory notices to the user.
#[cfg(feature = "unstable_session_notices")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NoticeCapabilities {}

#[cfg(feature = "unstable_session_notices")]
impl NoticeCapabilities {
    /// Advertises support for presenting live advisory notices to the user.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

/// Session configuration option capabilities supported by the client.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionConfigOptionsCapabilities {
    /// Whether the client supports boolean session configuration options.
    ///
    /// Optional. Omitted or `null` both mean the client does not advertise support.
    /// Supplying `{}` means agents may include `type: "boolean"` entries in
    /// `configOptions`, and the client may send `session/set_config_option`
    /// requests with `type: "boolean"` and a boolean `value`.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub boolean: Option<BooleanConfigOptionCapabilities>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionConfigOptionsCapabilities {
    /// Builds an empty [`SessionConfigOptionsCapabilities`]; use builder methods to advertise supported sub-capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the client supports boolean session configuration options.
    ///
    /// Omitted or `null` both mean the client does not advertise support.
    /// Supplying `{}` means agents may include `type: "boolean"` entries in
    /// `configOptions`, and the client may send `session/set_config_option`
    /// requests with `type: "boolean"` and a boolean `value`.
    #[must_use]
    pub fn boolean(mut self, boolean: impl IntoOption<BooleanConfigOptionCapabilities>) -> Self {
        self.boolean = boolean.into_option();
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

/// Capabilities for boolean session configuration options.
///
/// Supplying `{}` means the client supports boolean session configuration options.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct BooleanConfigOptionCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl BooleanConfigOptionCapabilities {
    /// Builds an empty [`BooleanConfigOptionCapabilities`]; use builder methods to advertise supported sub-capabilities.
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

/// Authentication capabilities supported by the client.
///
/// Advertised during initialization to inform the agent which authentication
/// method types the client can handle. This governs opt-in types that require
/// additional client-side support.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AuthCapabilities {
    /// Whether the client supports `terminal` authentication methods.
    ///
    /// The client should set this to `true` only when it can reproduce the
    /// configured agent invocation in an interactive terminal. When `true`, the
    /// agent may include `terminal` entries in its authentication methods.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub terminal: bool,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl AuthCapabilities {
    /// Builds an empty [`AuthCapabilities`]; use builder methods to advertise supported sub-capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the client supports `terminal` authentication methods.
    ///
    /// The client should set this to `true` only when it can reproduce the
    /// configured agent invocation in an interactive terminal. When `true`, the
    /// agent may include `AuthMethod::Terminal` entries in its authentication
    /// methods.
    #[must_use]
    pub fn terminal(mut self, terminal: bool) -> Self {
        self.terminal = terminal;
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

/// File system capabilities that a client may support.
///
/// See protocol docs: [FileSystem](https://agentclientprotocol.com/protocol/initialization#filesystem)
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FileSystemCapabilities {
    /// Whether the Client supports `fs/read_text_file` requests.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub read_text_file: bool,
    /// Whether the Client supports `fs/write_text_file` requests.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub write_text_file: bool,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl FileSystemCapabilities {
    /// Builds an empty [`FileSystemCapabilities`]; use builder methods to advertise supported sub-capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the Client supports `fs/read_text_file` requests.
    #[must_use]
    pub fn read_text_file(mut self, read_text_file: bool) -> Self {
        self.read_text_file = read_text_file;
        self
    }

    /// Whether the Client supports `fs/write_text_file` requests.
    #[must_use]
    pub fn write_text_file(mut self, write_text_file: bool) -> Self {
        self.write_text_file = write_text_file;
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
    /// Method for writing text files.
    pub fs_write_text_file: &'static str,
    /// Method for reading text files.
    pub fs_read_text_file: &'static str,
    /// Method for creating new terminals.
    pub terminal_create: &'static str,
    /// Method for getting terminals output.
    pub terminal_output: &'static str,
    /// Method for releasing a terminal.
    pub terminal_release: &'static str,
    /// Method for waiting for a terminal to finish.
    pub terminal_wait_for_exit: &'static str,
    /// Method for killing a terminal.
    pub terminal_kill: &'static str,
    /// Method for exchanging MCP-over-ACP messages.
    #[cfg(feature = "unstable_mcp_over_acp")]
    pub mcp_message: &'static str,
    /// Method for elicitation.
    pub elicitation_create: &'static str,
    /// Notification for elicitation completion.
    pub elicitation_complete: &'static str,
}

/// Constant containing all client method names.
pub const CLIENT_METHOD_NAMES: ClientMethodNames = ClientMethodNames {
    session_update: SESSION_UPDATE_NOTIFICATION,
    session_request_permission: SESSION_REQUEST_PERMISSION_METHOD_NAME,
    fs_write_text_file: FS_WRITE_TEXT_FILE_METHOD_NAME,
    fs_read_text_file: FS_READ_TEXT_FILE_METHOD_NAME,
    terminal_create: TERMINAL_CREATE_METHOD_NAME,
    terminal_output: TERMINAL_OUTPUT_METHOD_NAME,
    terminal_release: TERMINAL_RELEASE_METHOD_NAME,
    terminal_wait_for_exit: TERMINAL_WAIT_FOR_EXIT_METHOD_NAME,
    terminal_kill: TERMINAL_KILL_METHOD_NAME,
    #[cfg(feature = "unstable_mcp_over_acp")]
    mcp_message: MCP_MESSAGE_METHOD_NAME,
    elicitation_create: ELICITATION_CREATE_METHOD_NAME,
    elicitation_complete: ELICITATION_COMPLETE_NOTIFICATION,
};

/// Notification name for session updates.
pub(crate) const SESSION_UPDATE_NOTIFICATION: &str = "session/update";
/// Method name for requesting user permission.
pub(crate) const SESSION_REQUEST_PERMISSION_METHOD_NAME: &str = "session/request_permission";
/// Method name for writing text files.
pub(crate) const FS_WRITE_TEXT_FILE_METHOD_NAME: &str = "fs/write_text_file";
/// Method name for reading text files.
pub(crate) const FS_READ_TEXT_FILE_METHOD_NAME: &str = "fs/read_text_file";
/// Method name for creating a new terminal.
pub(crate) const TERMINAL_CREATE_METHOD_NAME: &str = "terminal/create";
/// Method for getting terminals output.
pub(crate) const TERMINAL_OUTPUT_METHOD_NAME: &str = "terminal/output";
/// Method for releasing a terminal.
pub(crate) const TERMINAL_RELEASE_METHOD_NAME: &str = "terminal/release";
/// Method for waiting for a terminal to finish.
pub(crate) const TERMINAL_WAIT_FOR_EXIT_METHOD_NAME: &str = "terminal/wait_for_exit";
/// Method for killing a terminal.
pub(crate) const TERMINAL_KILL_METHOD_NAME: &str = "terminal/kill";
/// Method name for elicitation.
pub(crate) const ELICITATION_CREATE_METHOD_NAME: &str = "elicitation/create";
/// Notification name for elicitation completion.
pub(crate) const ELICITATION_COMPLETE_NOTIFICATION: &str = "elicitation/complete";

/// All possible requests that an agent can send to a client.
///
/// This enum is used internally for routing RPC requests. You typically won't need
/// to use this directly.
///
/// This enum encompasses all method calls from agent to client.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schemars", schemars(inline))]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum AgentRequest {
    /// Writes content to a text file in the client's file system.
    ///
    /// Only available if the client advertises the `fs.writeTextFile` capability.
    /// Allows the agent to create or modify files within the client's environment.
    ///
    /// See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
    WriteTextFileRequest(WriteTextFileRequest),
    /// Reads content from a text file in the client's file system.
    ///
    /// Only available if the client advertises the `fs.readTextFile` capability.
    /// Allows the agent to access file contents within the client's environment.
    ///
    /// See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
    ReadTextFileRequest(ReadTextFileRequest),
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
    /// Executes a command in a new terminal
    ///
    /// Only available if the `terminal` Client capability is set to `true`.
    ///
    /// Returns a `TerminalId` that can be used with other terminal methods
    /// to get the current output, wait for exit, and kill the command.
    ///
    /// The `TerminalId` can also be used to embed the terminal in a tool call
    /// by using the `ToolCallContent::Terminal` variant.
    ///
    /// The Agent is responsible for releasing the terminal by using the `terminal/release`
    /// method.
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    CreateTerminalRequest(CreateTerminalRequest),
    /// Gets the terminal output and exit status
    ///
    /// Returns the current content in the terminal without waiting for the command to exit.
    /// If the command has already exited, the exit status is included.
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    TerminalOutputRequest(TerminalOutputRequest),
    /// Releases a terminal
    ///
    /// The command is killed if it hasn't exited yet. Use `terminal/wait_for_exit`
    /// to wait for the command to exit before releasing the terminal.
    ///
    /// After release, the `TerminalId` can no longer be used with other `terminal/*` methods,
    /// but tool calls that already contain it, continue to display its output.
    ///
    /// The `terminal/kill` method can be used to terminate the command without releasing
    /// the terminal, allowing the Agent to call `terminal/output` and other methods.
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    ReleaseTerminalRequest(ReleaseTerminalRequest),
    /// Waits for the terminal command to exit and return its exit status
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    WaitForTerminalExitRequest(WaitForTerminalExitRequest),
    /// Kills the terminal command without releasing the terminal
    ///
    /// While `terminal/release` will also kill the command, this method will keep
    /// the `TerminalId` valid so it can be used with other methods.
    ///
    /// This method can be helpful when implementing command timeouts which terminate
    /// the command as soon as elapsed, and then get the final output so it can be sent
    /// to the model.
    ///
    /// Note: Call `terminal/release` when `TerminalId` is no longer needed.
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    KillTerminalRequest(KillTerminalRequest),
    /// Requests structured user input via a form or URL.
    ///
    /// See protocol docs: [Elicitation](https://agentclientprotocol.com/protocol/elicitation)
    CreateElicitationRequest(CreateElicitationRequest),
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Exchanges an MCP-over-ACP message.
    #[cfg(feature = "unstable_mcp_over_acp")]
    MessageMcpRequest(MessageMcpRequest),
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
            Self::WriteTextFileRequest(_) => CLIENT_METHOD_NAMES.fs_write_text_file,
            Self::ReadTextFileRequest(_) => CLIENT_METHOD_NAMES.fs_read_text_file,
            Self::RequestPermissionRequest(_) => CLIENT_METHOD_NAMES.session_request_permission,
            Self::CreateTerminalRequest(_) => CLIENT_METHOD_NAMES.terminal_create,
            Self::TerminalOutputRequest(_) => CLIENT_METHOD_NAMES.terminal_output,
            Self::ReleaseTerminalRequest(_) => CLIENT_METHOD_NAMES.terminal_release,
            Self::WaitForTerminalExitRequest(_) => CLIENT_METHOD_NAMES.terminal_wait_for_exit,
            Self::KillTerminalRequest(_) => CLIENT_METHOD_NAMES.terminal_kill,
            Self::CreateElicitationRequest(_) => CLIENT_METHOD_NAMES.elicitation_create,
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpRequest(_) => CLIENT_METHOD_NAMES.mcp_message,
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schemars", schemars(inline))]
#[non_exhaustive]
pub enum ClientResponse {
    /// Successful result returned for a `fs/write_text_file` request.
    WriteTextFileResponse(#[serde(default)] WriteTextFileResponse),
    /// Successful result returned for a `fs/read_text_file` request.
    ReadTextFileResponse(ReadTextFileResponse),
    /// Successful result returned for a `session/request_permission` request.
    RequestPermissionResponse(RequestPermissionResponse),
    /// Successful result returned for a `terminal/create` request.
    CreateTerminalResponse(CreateTerminalResponse),
    /// Successful result returned for a `terminal/output` request.
    TerminalOutputResponse(TerminalOutputResponse),
    /// Successful result returned for a `terminal/release` request.
    ReleaseTerminalResponse(#[serde(default)] ReleaseTerminalResponse),
    /// Successful result returned for a `terminal/wait_for_exit` request.
    WaitForTerminalExitResponse(WaitForTerminalExitResponse),
    /// Successful result returned for a `terminal/kill` request.
    KillTerminalResponse(#[serde(default)] KillTerminalResponse),
    /// Successful result returned for a `elicitation/create` request.
    CreateElicitationResponse(CreateElicitationResponse),
    /// Successful result returned by an MCP-over-ACP `mcp/message` request.
    #[cfg(feature = "unstable_mcp_over_acp")]
    MessageMcpResponse(MessageMcpResponse),
    /// Successful result returned by an extension method outside the core ACP method set.
    ExtMethodResponse(ExtResponse),
}

/// All possible notifications that an agent can send to a client.
///
/// This enum is used internally for routing RPC notifications. You typically won't need
/// to use this directly.
///
/// Notifications do not expect a response.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[expect(clippy::large_enum_variant)]
#[cfg_attr(feature = "schemars", schemars(inline))]
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
    /// Notification that a URL-based elicitation has completed.
    ///
    /// See protocol docs: [Elicitation](https://agentclientprotocol.com/protocol/elicitation#url-completion)
    CompleteElicitationNotification(CompleteElicitationNotification),
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
            Self::CompleteElicitationNotification(_) => CLIENT_METHOD_NAMES.elicitation_complete,
            Self::ExtNotification(ext_notification) => &ext_notification.method,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "unstable_session_notices")]
    #[test]
    fn notice_preserves_wire_shape_nullable_fields_and_open_severity() {
        use serde_json::json;

        let mut meta = Meta::new();
        meta.insert("source".into(), json!("fallback"));
        assert_eq!(
            serde_json::to_value(SessionUpdate::Notice(
                Notice::new(NoticeSeverity::Warning, "MCP server unavailable")
                    .description("Continuing without it.")
                    .meta(meta),
            ))
            .unwrap(),
            json!({
                "sessionUpdate": "notice",
                "severity": "warning",
                "title": "MCP server unavailable",
                "description": "Continuing without it.",
                "_meta": { "source": "fallback" }
            })
        );

        assert_eq!(
            serde_json::to_value(SessionUpdate::Notice(Notice::new(
                NoticeSeverity::Info,
                "Indexing workspace",
            )))
            .unwrap(),
            json!({
                "sessionUpdate": "notice",
                "severity": "info",
                "title": "Indexing workspace"
            })
        );

        let SessionUpdate::Notice(notice) = serde_json::from_value(json!({
            "sessionUpdate": "notice",
            "severity": "critical",
            "title": "Provider degraded",
            "description": null,
            "_meta": null
        }))
        .unwrap() else {
            panic!("expected notice");
        };

        assert_eq!(
            notice.severity,
            NoticeSeverity::Other("critical".to_string())
        );
        assert_eq!(notice.description, None);
        assert_eq!(notice.meta, None);
        assert_eq!(
            serde_json::to_value(SessionUpdate::Notice(notice)).unwrap(),
            json!({
                "sessionUpdate": "notice",
                "severity": "critical",
                "title": "Provider degraded"
            })
        );
    }

    #[cfg(feature = "unstable_session_notices")]
    #[test]
    fn notice_requires_non_null_severity_and_title() {
        use serde_json::json;

        for malformed in [
            json!({
                "sessionUpdate": "notice",
                "severity": "warning"
            }),
            json!({
                "sessionUpdate": "notice",
                "severity": "warning",
                "title": null
            }),
            json!({
                "sessionUpdate": "notice",
                "title": "MCP server unavailable"
            }),
            json!({
                "sessionUpdate": "notice",
                "severity": null,
                "title": "MCP server unavailable"
            }),
        ] {
            assert!(serde_json::from_value::<SessionUpdate>(malformed).is_err());
        }
    }

    #[cfg(feature = "unstable_session_notices")]
    #[test]
    fn notice_capability_advertises_support_only_when_present() {
        use serde_json::json;

        let capabilities = ClientCapabilities::new()
            .session(ClientSessionCapabilities::new().notices(NoticeCapabilities::new()));
        let value = serde_json::to_value(&capabilities).unwrap();
        assert_eq!(value["session"], json!({ "notices": {} }));
        assert_eq!(
            serde_json::from_value::<ClientCapabilities>(value).unwrap(),
            capabilities
        );

        for unsupported in [
            json!({}),
            json!({ "session": null }),
            json!({ "session": {} }),
            json!({ "session": { "notices": null } }),
            json!({ "session": { "notices": false } }),
            json!({ "session": { "notices": true } }),
            json!({ "session": { "notices": "supported" } }),
        ] {
            let capabilities: ClientCapabilities = serde_json::from_value(unsupported).unwrap();
            assert!(
                capabilities
                    .session
                    .and_then(|session| session.notices)
                    .is_none()
            );
        }

        assert_eq!(
            serde_json::to_value(
                ClientSessionCapabilities::new()
                    .notices(NoticeCapabilities::new())
                    .notices(None)
            )
            .unwrap(),
            json!({})
        );
    }

    #[cfg(not(feature = "unstable_session_notices"))]
    #[test]
    fn unsupported_notice_capability_is_ignored() {
        use serde_json::json;

        let capabilities: ClientSessionCapabilities =
            serde_json::from_value(json!({ "notices": {} })).unwrap();
        assert_eq!(serde_json::to_value(capabilities).unwrap(), json!({}));
    }

    #[cfg(not(feature = "unstable_session_notices"))]
    #[test]
    fn unsupported_notice_is_rejected_by_closed_update_union() {
        use serde_json::json;

        assert!(
            serde_json::from_value::<SessionUpdate>(json!({
                "sessionUpdate": "notice",
                "severity": "warning",
                "title": "MCP server unavailable"
            }))
            .is_err()
        );
    }

    #[test]
    fn compaction_updates_preserve_patch_and_open_status_semantics() {
        use serde_json::json;

        assert_eq!(
            serde_json::to_value(SessionUpdate::CompactionUpdate(CompactionUpdate::new(
                "cmp_001",
                CompactionStatus::InProgress,
            )))
            .unwrap(),
            json!({
                "sessionUpdate": "compaction_update",
                "compactionId": "cmp_001",
                "status": "in_progress"
            })
        );

        let SessionUpdate::CompactionUpdate(update) = serde_json::from_value(json!({
            "sessionUpdate": "compaction_update",
            "compactionId": "cmp_001",
            "status": "paused",
            "summary": null,
            "error": null
        }))
        .unwrap() else {
            panic!("expected compaction update");
        };
        assert_eq!(update.status, CompactionStatus::Other("paused".into()));
        assert!(update.summary.is_null());
        assert!(update.error.is_null());
        assert!(update.meta.is_undefined());
    }

    #[test]
    fn compaction_chunk_and_v1_capability_serialize() {
        use serde_json::json;

        assert_eq!(
            serde_json::to_value(SessionUpdate::CompactionSummaryChunk(
                CompactionSummaryChunk::new(
                    "cmp_001",
                    ContentBlock::Text(crate::v1::TextContent::new("retained")),
                ),
            ))
            .unwrap(),
            json!({
                "sessionUpdate": "compaction_summary_chunk",
                "compactionId": "cmp_001",
                "content": { "type": "text", "text": "retained" }
            })
        );
        assert_eq!(
            serde_json::to_value(
                ClientSessionCapabilities::new().compaction(CompactionCapabilities::new())
            )
            .unwrap(),
            json!({ "compaction": {} })
        );
        let absent: ClientSessionCapabilities = serde_json::from_value(json!({})).unwrap();
        let null: ClientSessionCapabilities =
            serde_json::from_value(json!({ "compaction": null })).unwrap();
        assert!(absent.compaction.is_none());
        assert!(null.compaction.is_none());
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn test_subagent_updates_serialization() {
        use serde_json::json;

        let announced = SessionUpdate::SubagentUpdate(SubagentUpdate::new("sess_child_1"));
        assert_eq!(
            serde_json::to_value(&announced).unwrap(),
            json!({
                "sessionUpdate": "subagent_update",
                "sessionId": "sess_child_1"
            })
        );
        assert_eq!(
            serde_json::from_value::<SessionUpdate>(serde_json::to_value(&announced).unwrap())
                .unwrap(),
            announced
        );
        let capable = SubagentUpdate::new("sess_child_1").capabilities(
            SubagentSessionCapabilities::new().cancel(SessionCancelCapabilities::new()),
        );
        assert_eq!(
            serde_json::to_value(capable).unwrap(),
            json!({
                "sessionId": "sess_child_1",
                "capabilities": {"cancel": {}}
            })
        );
        let minimal: SubagentUpdate =
            serde_json::from_value(json!({ "sessionId": "sess_child_3" })).unwrap();
        assert!(minimal.capabilities.is_undefined());
        assert!(minimal.state.is_undefined());

        let nulls: SubagentUpdate = serde_json::from_value(json!({
            "sessionId": "sess_child_3",
            "capabilities": null,
            "state": null
        }))
        .unwrap();
        assert!(nulls.capabilities.is_null());
        assert!(nulls.state.is_null());
        assert_eq!(
            serde_json::to_value(nulls).unwrap(),
            json!({"sessionId": "sess_child_3", "capabilities": null, "state": null})
        );
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn test_subagent_display_metadata() {
        use serde_json::json;

        let update = SubagentUpdate::new("child")
            .title("Test investigator".to_string())
            .description("Investigates platform-specific test failures.".to_string());
        let wire = json!({
            "sessionId": "child",
            "title": "Test investigator",
            "description": "Investigates platform-specific test failures."
        });
        assert_eq!(serde_json::to_value(&update).unwrap(), wire);
        assert_eq!(
            serde_json::from_value::<SubagentUpdate>(wire).unwrap(),
            update
        );

        let minimal = SubagentUpdate::new("child");
        assert_eq!(
            serde_json::to_value(&minimal).unwrap(),
            json!({"sessionId": "child"})
        );
        let cleared: SubagentUpdate = serde_json::from_value(json!({
            "sessionId": "child", "title": null, "description": null
        }))
        .unwrap();
        assert!(cleared.title.is_null());
        assert!(cleared.description.is_null());
        assert_eq!(
            serde_json::to_value(cleared).unwrap(),
            json!({"sessionId": "child", "title": null, "description": null})
        );
        let invalid: SubagentUpdate = serde_json::from_value(json!({
            "sessionId": "child", "title": false, "description": false
        }))
        .unwrap();
        assert_eq!(invalid, minimal);

        let title_only: SubagentUpdate = serde_json::from_value(json!({
            "sessionId": "child", "title": "Updated title"
        }))
        .unwrap();
        assert_eq!(
            title_only.title.value().map(String::as_str),
            Some("Updated title")
        );
        assert!(title_only.description.is_undefined());
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn subagent_patch_fields_preserve_omitted_null_and_concrete() {
        use serde_json::json;

        let omitted = SubagentUpdate::new("child");
        for field in ["title", "description", "capabilities", "state", "_meta"] {
            let wire = json!({"sessionId": "child", field: null});
            let decoded: SubagentUpdate = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), wire, "{field}");
            let malformed = json!({"sessionId": "child", field: false});
            let decoded: SubagentUpdate = serde_json::from_value(malformed).unwrap();
            assert_eq!(decoded, omitted, "{field}");
        }
        let concrete = json!({
            "sessionId": "child", "title": "Investigator",
            "description": "Tests", "capabilities": {"cancel": {}},
            "state": {"state": "running"}, "_meta": {"source": "parent"}
        });
        let decoded: SubagentUpdate = serde_json::from_value(concrete.clone()).unwrap();
        assert!(decoded.title.value().is_some());
        assert!(decoded.description.value().is_some());
        assert!(decoded.capabilities.value().is_some());
        assert!(decoded.state.value().is_some());
        assert!(decoded.meta.value().is_some());
        assert_eq!(serde_json::to_value(decoded).unwrap(), concrete);
        let cleared = SubagentUpdate::new("child")
            .title(None)
            .description(None)
            .capabilities(None)
            .state(None)
            .meta(None);
        assert_eq!(
            serde_json::to_value(cleared).unwrap(),
            json!({
                "sessionId": "child", "title": null, "description": null,
                "capabilities": null, "state": null, "_meta": null
            })
        );
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn test_subagent_notification_keeps_parent_and_child_ids_nested() {
        use serde_json::json;

        let notification = SessionNotification::new(
            "parent",
            SessionUpdate::SubagentUpdate(SubagentUpdate::new("child")),
        );
        let wire = json!({
            "sessionId": "parent",
            "update": {
                "sessionUpdate": "subagent_update",
                "sessionId": "child"
            }
        });
        assert_eq!(serde_json::to_value(&notification).unwrap(), wire);
        assert_eq!(
            serde_json::from_value::<SessionNotification>(wire).unwrap(),
            notification
        );
        // Neither ID can stand in for the other: both nesting levels require one.
        for malformed in [
            json!({"sessionId": "parent", "update": {"sessionUpdate": "subagent_update"}}),
            json!({"update": {"sessionUpdate": "subagent_update", "sessionId": "child"}}),
        ] {
            assert!(serde_json::from_value::<SessionNotification>(malformed).is_err());
        }
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn test_subagent_states_round_trip() {
        use serde_json::json;

        for (wire_state, expected) in [
            (
                json!({"state": "running"}),
                StateUpdate::Running(RunningStateUpdate::new()),
            ),
            (
                json!({"state": "idle", "stopReason": "end_turn"}),
                StateUpdate::Idle(IdleStateUpdate::new().stop_reason(IdleStopReason::EndTurn)),
            ),
            (
                json!({"state": "requires_action"}),
                StateUpdate::RequiresAction(RequiresActionStateUpdate::new()),
            ),
            (
                json!({"state": "unknown"}),
                StateUpdate::Unknown(UnknownStateUpdate::new()),
            ),
            (
                json!({"state": "_waiting", "detail": {"ticket": 7}}),
                StateUpdate::Other(OtherStateUpdate::new(
                    "_waiting",
                    [("detail".into(), json!({"ticket": 7}))].into(),
                )),
            ),
            (
                json!({"state": "paused", "retryAt": 42}),
                StateUpdate::Other(OtherStateUpdate::new(
                    "paused",
                    [("retryAt".into(), json!(42))].into(),
                )),
            ),
        ] {
            let wire = json!({
                "sessionUpdate": "subagent_update",
                "sessionId": "sess_child",
                "state": wire_state
            });
            let parsed: SessionUpdate = serde_json::from_value(wire.clone()).unwrap();
            let SessionUpdate::SubagentUpdate(update) = &parsed else {
                panic!("expected subagent update");
            };
            assert_eq!(update.state.value(), Some(&expected));
            assert_eq!(serde_json::to_value(parsed).unwrap(), wire);
        }

        for reason in [
            "end_turn",
            "max_tokens",
            "max_turn_requests",
            "refusal",
            "cancelled",
        ] {
            let state: StateUpdate = serde_json::from_value(json!({
                "state": "idle", "stopReason": reason
            }))
            .unwrap();
            assert_eq!(serde_json::to_value(state).unwrap()["stopReason"], reason);
        }

        // Unknown activity is a replaceable snapshot, not a session outcome.
        // Later snapshots still address the same child without stale stop reasons.
        for state in [
            StateUpdate::Running(RunningStateUpdate::new()),
            StateUpdate::RequiresAction(RequiresActionStateUpdate::new()),
            StateUpdate::Unknown(UnknownStateUpdate::new()),
            StateUpdate::Running(RunningStateUpdate::new()),
            StateUpdate::Idle(IdleStateUpdate::new().stop_reason(IdleStopReason::EndTurn)),
            StateUpdate::Unknown(UnknownStateUpdate::new()),
            StateUpdate::Running(RunningStateUpdate::new()),
        ] {
            let update = SubagentUpdate::new("sess_child").state(state.clone());
            let wire = serde_json::to_value(&update).unwrap();
            assert_eq!(wire["sessionId"], "sess_child");
            assert_eq!(
                serde_json::from_value::<SubagentUpdate>(wire)
                    .unwrap()
                    .state,
                MaybeUndefined::Value(state)
            );
        }
        assert_eq!(
            serde_json::to_value(
                SubagentUpdate::new("sess_child").state(StateUpdate::Idle(IdleStateUpdate::new()))
            )
            .unwrap()["state"],
            json!({"state": "idle"})
        );
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn subagent_idle_snapshot_reports_child_failure() {
        use serde_json::json;

        let update = SubagentUpdate::new("sess_child").state(StateUpdate::Idle(
            IdleStateUpdate::new().stop_reason(IdleStopReason::from(
                ErrorStopReason::new().error(Error::auth_required()),
            )),
        ));
        let wire = json!({
            "sessionId": "sess_child",
            "state": {
                "state": "idle",
                "stopReason": "error",
                "error": { "code": -32000, "message": "Authentication required" }
            }
        });
        assert_eq!(serde_json::to_value(&update).unwrap(), wire);
        assert_eq!(
            serde_json::from_value::<SubagentUpdate>(wire).unwrap(),
            update
        );

        // Missing, null, and malformed details still end the work as a failure.
        let without_details = StateUpdate::Idle(
            IdleStateUpdate::new().stop_reason(IdleStopReason::from(ErrorStopReason::new())),
        );
        for error in [None, Some(json!(null)), Some(json!("failed"))] {
            let mut wire = json!({ "state": "idle", "stopReason": "error" });
            if let Some(error) = error {
                wire["error"] = error;
            }
            assert_eq!(
                serde_json::from_value::<StateUpdate>(wire).unwrap(),
                without_details
            );
        }

        // `error` belongs to the `error` stop reason; beside another one it is
        // an unknown field.
        let parsed: StateUpdate = serde_json::from_value(json!({
            "state": "idle",
            "stopReason": "end_turn",
            "error": { "code": -32603, "message": "Internal error" }
        }))
        .unwrap();
        assert_eq!(
            parsed,
            StateUpdate::Idle(IdleStateUpdate::new().stop_reason(IdleStopReason::EndTurn))
        );

        // Omitted, null, and malformed stop reasons mean none was reported,
        // and the rest of the snapshot survives.
        let meta: Meta = [("source".into(), json!("worker"))].into_iter().collect();
        for stop_reason in [None, Some(json!(null)), Some(json!(true))] {
            let mut wire = json!({ "state": "idle", "_meta": { "source": "worker" } });
            if let Some(stop_reason) = stop_reason {
                wire["stopReason"] = stop_reason;
            }
            assert_eq!(
                serde_json::from_value::<StateUpdate>(wire).unwrap(),
                StateUpdate::Idle(IdleStateUpdate::new().meta(meta.clone()))
            );
        }

        // Unknown stop reasons keep their own fields, as in v2, while the
        // snapshot's fields stay on the snapshot.
        for stop_reason in ["_paused", "not_a_reason"] {
            let wire = json!({
                "state": "idle",
                "stopReason": stop_reason,
                "resumeAfter": 30,
                "_meta": { "source": "worker" }
            });
            let parsed: StateUpdate = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(
                parsed,
                StateUpdate::Idle(
                    IdleStateUpdate::new()
                        .stop_reason(IdleStopReason::Other(OtherStopReason::new(
                            stop_reason,
                            [("resumeAfter".into(), json!(30))].into_iter().collect(),
                        )))
                        .meta(meta.clone())
                )
            );
            assert_eq!(serde_json::to_value(&parsed).unwrap(), wire);
        }
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn subagent_known_stop_reasons_are_never_read_as_other() {
        use serde_json::json;

        let known = [
            IdleStopReason::EndTurn,
            IdleStopReason::MaxTokens,
            IdleStopReason::MaxTurnRequests,
            IdleStopReason::Refusal,
            IdleStopReason::Cancelled,
            IdleStopReason::Error(ErrorStopReason::new()),
        ];
        // Adding a variant breaks this match, so `known` stays complete.
        for stop_reason in &known {
            match stop_reason {
                IdleStopReason::EndTurn
                | IdleStopReason::MaxTokens
                | IdleStopReason::MaxTurnRequests
                | IdleStopReason::Refusal
                | IdleStopReason::Cancelled
                | IdleStopReason::Error(_)
                | IdleStopReason::Other(_) => {}
            }
        }

        for stop_reason in known {
            let state = StateUpdate::Idle(IdleStateUpdate::new().stop_reason(stop_reason));
            let wire = serde_json::to_value(&state).unwrap();
            assert_eq!(
                serde_json::from_value::<StateUpdate>(wire.clone()).unwrap(),
                state
            );
            assert!(
                serde_json::from_value::<OtherStopReason>(
                    json!({ "stopReason": wire["stopReason"] })
                )
                .is_err(),
                "{} must not be accepted as a custom stop reason",
                wire["stopReason"]
            );
        }
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn subagent_unknown_stop_reason_leaves_usage_on_the_snapshot() {
        use serde_json::json;

        let usage = json!({ "totalTokens": 10, "inputTokens": 6, "outputTokens": 4 });
        let wire = json!({ "state": "idle", "stopReason": "_paused", "usage": usage });
        let StateUpdate::Idle(idle) = serde_json::from_value(wire).unwrap() else {
            panic!("expected idle state snapshot");
        };
        let Some(IdleStopReason::Other(other)) = &idle.stop_reason else {
            panic!("expected a custom stop reason");
        };
        assert!(other.fields.is_empty());

        #[cfg(feature = "unstable_end_turn_token_usage")]
        {
            assert!(idle.usage.is_some());
            assert_eq!(serde_json::to_value(&idle).unwrap()["usage"], usage);
        }
        // Without the feature, `usage` is not a field of the snapshot, and it
        // must not be smuggled through as a field of the stop reason either.
        #[cfg(not(feature = "unstable_end_turn_token_usage"))]
        assert!(serde_json::to_value(&idle).unwrap().get("usage").is_none());
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn test_subagent_unknown_activity_metadata_and_capabilities() {
        use serde_json::json;

        let update = SubagentUpdate::new("child").state(StateUpdate::Unknown(
            UnknownStateUpdate::new().meta(
                [("source".into(), json!("worker"))]
                    .into_iter()
                    .collect::<Meta>(),
            ),
        ));
        let wire = json!({
            "sessionId": "child",
            "state": {
                "state": "unknown",
                "_meta": { "source": "worker" }
            }
        });
        assert_eq!(serde_json::to_value(&update).unwrap(), wire);
        assert_eq!(
            serde_json::from_value::<SubagentUpdate>(wire).unwrap(),
            update
        );
        // Reporting unknown activity does not send a capability revocation.
        assert!(update.capabilities.is_undefined());

        for meta in [json!(null), json!(false)] {
            let state: StateUpdate = serde_json::from_value(json!({
                "state": "unknown",
                "_meta": meta
            }))
            .unwrap();
            assert_eq!(state, StateUpdate::Unknown(UnknownStateUpdate::new()));
            assert_eq!(
                serde_json::to_value(state).unwrap(),
                json!({"state": "unknown"})
            );
        }
        // The standard unknown-activity report is recognized, not an extension fallback.
        assert!(serde_json::from_value::<OtherStateUpdate>(json!({"state": "unknown"})).is_err());
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn test_subagent_malformed_optional_state() {
        use serde_json::json;

        for malformed in [json!("running"), json!({}), json!({"state": 4})] {
            assert!(serde_json::from_value::<StateUpdate>(malformed.clone()).is_err());
            let update: SubagentUpdate = serde_json::from_value(json!({
                "sessionId": "child", "state": malformed
            }))
            .unwrap();
            assert!(update.state.is_undefined());
        }
        let bad_reason: StateUpdate =
            serde_json::from_value(json!({"state": "idle", "stopReason": false})).unwrap();
        assert_eq!(bad_reason, StateUpdate::Idle(IdleStateUpdate::new()));
        let null_reason: StateUpdate =
            serde_json::from_value(json!({"state": "idle", "stopReason": null})).unwrap();
        assert_eq!(null_reason, StateUpdate::Idle(IdleStateUpdate::new()));
    }

    #[cfg(all(feature = "unstable_subagents", feature = "unstable_protocol_v2"))]
    #[test]
    fn test_subagent_state_v2_wire_parity() {
        use serde_json::json;

        for wire in [
            json!({"state": "running"}),
            json!({"state": "idle", "stopReason": "cancelled"}),
            json!({"state": "requires_action"}),
            json!({"state": "unknown"}),
            json!({"state": "unknown", "_meta": {"source": "worker"}}),
            json!({"state": "_waiting", "detail": {"ticket": 7}}),
        ] {
            let v1: StateUpdate = serde_json::from_value(wire.clone()).unwrap();
            let v2: crate::v2::StateUpdate = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(v1).unwrap(), wire);
            assert_eq!(serde_json::to_value(v2).unwrap(), wire);
        }
    }

    #[cfg(feature = "unstable_subagents")]
    #[test]
    fn test_subagent_capability_semantics() {
        use serde_json::json;

        let capabilities =
            serde_json::to_value(ClientCapabilities::new().subagents(SubagentCapabilities::new()))
                .unwrap();
        assert_eq!(capabilities["subagents"], json!({}));

        let omitted: ClientCapabilities = serde_json::from_value(json!({})).unwrap();
        assert!(omitted.subagents.is_none());

        let null: ClientCapabilities =
            serde_json::from_value(json!({ "subagents": null })).unwrap();
        assert!(null.subagents.is_none());
        assert_eq!(null, omitted);
        let serialized = serde_json::to_value(null).unwrap();
        assert_eq!(serialized, serde_json::to_value(omitted).unwrap());
        assert!(serialized.get("subagents").is_none());

        for wire in [
            json!({}),
            json!({"cancel": null}),
            json!({"cancel": true}),
            json!({"cancel": false}),
        ] {
            let child: SubagentSessionCapabilities = serde_json::from_value(wire).unwrap();
            assert!(child.cancel.is_none());
            assert_eq!(serde_json::to_value(child).unwrap(), json!({}));
        }
        let enabled = SubagentSessionCapabilities::new().cancel(SessionCancelCapabilities::new());
        assert_eq!(
            serde_json::to_value(&enabled).unwrap(),
            json!({"cancel": {}})
        );
        assert_eq!(
            serde_json::from_value::<SubagentSessionCapabilities>(json!({"cancel": {}})).unwrap(),
            enabled
        );
        let meta: Meta = [("source".into(), json!("worker"))].into_iter().collect();
        let with_meta =
            SubagentSessionCapabilities::new().cancel(SessionCancelCapabilities::new().meta(meta));
        let wire = json!({"cancel": {"_meta": {"source": "worker"}}});
        assert_eq!(serde_json::to_value(&with_meta).unwrap(), wire);
        assert_eq!(
            serde_json::from_value::<SubagentSessionCapabilities>(wire).unwrap(),
            with_meta
        );
        // Each supplied child capability set replaces the previous set.
        let removed = SubagentSessionCapabilities::new();
        assert_eq!(serde_json::to_value(&removed).unwrap(), json!({}));
        assert!(removed.cancel.is_none());
        assert!(enabled.cancel.is_some());
        assert!(enabled.cancel(None).cancel.is_none());
        let replacement = SubagentUpdate::new("child").capabilities(removed);
        assert_eq!(
            serde_json::to_value(replacement).unwrap(),
            json!({"sessionId": "child", "capabilities": {}})
        );
    }

    #[cfg(all(feature = "unstable_subagents", feature = "schemars"))]
    #[test]
    fn test_subagent_capability_schema_is_optional_and_nullable() {
        use serde_json::json;

        let schema = serde_json::to_value(schemars::schema_for!(ClientCapabilities)).unwrap();
        let variants = schema["properties"]["subagents"]["anyOf"]
            .as_array()
            .expect("subagents must allow the capability object or null");
        assert!(variants.contains(&json!({"type": "null"})));
        assert!(
            !schema["required"]
                .as_array()
                .is_some_and(|required| required.contains(&json!("subagents")))
        );
        let child =
            serde_json::to_value(schemars::schema_for!(SubagentSessionCapabilities)).unwrap();
        let variants = child["properties"]["cancel"]["anyOf"]
            .as_array()
            .expect("cancel must allow the capability object or null");
        assert!(variants.contains(&json!({"type": "null"})));
        assert!(
            variants
                .iter()
                .any(|variant| variant["type"] == "object" || variant.get("$ref").is_some())
        );
        assert!(!variants.iter().any(|variant| variant["type"] == "boolean"));
        let cancel =
            serde_json::to_value(schemars::schema_for!(SessionCancelCapabilities)).unwrap();
        assert_eq!(cancel["type"], "object");
        assert!(
            !child["required"]
                .as_array()
                .is_some_and(|required| required.contains(&json!("cancel")))
        );
    }

    #[test]
    fn test_elicitation_capability_semantics() {
        use serde_json::json;

        let unsupported: ClientCapabilities = serde_json::from_value(json!({})).unwrap();
        assert!(unsupported.elicitation.is_none());

        let null: ClientCapabilities =
            serde_json::from_value(json!({ "elicitation": null })).unwrap();
        assert!(null.elicitation.is_none());

        let malformed: ClientCapabilities =
            serde_json::from_value(json!({ "elicitation": false })).unwrap();
        assert!(malformed.elicitation.is_none());

        let empty: ClientCapabilities =
            serde_json::from_value(json!({ "elicitation": {} })).unwrap();
        let empty = empty.elicitation.expect("present capability");
        assert!(!empty.supports_form());
        assert!(!empty.supports_url());

        let form_only: ClientCapabilities = serde_json::from_value(json!({
            "elicitation": { "form": {} }
        }))
        .unwrap();
        let form_only = form_only.elicitation.expect("advertised capability");
        assert!(form_only.supports_form());
        assert!(!form_only.supports_url());

        let url_only: ClientCapabilities = serde_json::from_value(json!({
            "elicitation": { "url": {} }
        }))
        .unwrap();
        let url_only = url_only.elicitation.expect("advertised capability");
        assert!(!url_only.supports_form());
        assert!(url_only.supports_url());

        let both: ClientCapabilities = serde_json::from_value(json!({
            "elicitation": { "form": {}, "url": {} }
        }))
        .unwrap();
        let both = both.elicitation.expect("advertised capability");
        assert!(both.supports_form());
        assert!(both.supports_url());
    }

    #[test]
    fn test_elicitation_method_routing_and_envelopes() {
        use serde_json::json;

        assert_eq!(CLIENT_METHOD_NAMES.elicitation_create, "elicitation/create");
        assert_eq!(
            CLIENT_METHOD_NAMES.elicitation_complete,
            "elicitation/complete"
        );

        let request = AgentRequest::CreateElicitationRequest(CreateElicitationRequest::new(
            crate::v1::ElicitationFormMode::new(
                crate::v1::ElicitationSessionScope::new("sess_1"),
                crate::v1::ElicitationSchema::new(),
            ),
            "Choose a value",
        ));
        assert_eq!(request.method(), "elicitation/create");
        let method = Arc::from(request.method());
        let request = crate::v1::JsonRpcMessage::wrap(crate::v1::Request {
            id: crate::v1::RequestId::Number(7),
            method,
            params: Some(request),
        });
        assert_eq!(
            serde_json::to_value(request).unwrap(),
            json!({
                "jsonrpc": "2.0",
                "id": 7,
                "method": "elicitation/create",
                "params": {
                    "mode": "form",
                    "sessionId": "sess_1",
                    "message": "Choose a value",
                    "requestedSchema": { "type": "object", "properties": {} }
                }
            })
        );

        let notification = AgentNotification::CompleteElicitationNotification(
            CompleteElicitationNotification::new("elic_1"),
        );
        assert_eq!(notification.method(), "elicitation/complete");
        let method = Arc::from(notification.method());
        let notification = crate::v1::JsonRpcMessage::wrap(crate::v1::Notification {
            method,
            params: Some(notification),
        });
        assert_eq!(
            serde_json::to_value(notification).unwrap(),
            json!({
                "jsonrpc": "2.0",
                "method": "elicitation/complete",
                "params": { "elicitationId": "elic_1" }
            })
        );
    }

    #[test]
    fn test_client_capabilities_default_on_malformed_values() {
        use serde_json::json;

        let capabilities: ClientCapabilities = serde_json::from_value(json!({
            "fs": {
                "readTextFile": "yes",
                "writeTextFile": true
            },
            "terminal": {}
        }))
        .unwrap();

        assert!(!capabilities.fs.read_text_file);
        assert!(capabilities.fs.write_text_file);
        assert!(!capabilities.terminal);

        let capabilities: ClientCapabilities = serde_json::from_value(json!({
            "fs": false
        }))
        .unwrap();
        assert_eq!(capabilities.fs, FileSystemCapabilities::default());

        {
            let capabilities: ClientCapabilities = serde_json::from_value(json!({
                "auth": false
            }))
            .unwrap();
            assert_eq!(capabilities.auth, AuthCapabilities::default());

            let capabilities: AuthCapabilities = serde_json::from_value(json!({
                "terminal": {}
            }))
            .unwrap();
            assert!(!capabilities.terminal);
        }
    }

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
    fn test_content_chunk_message_id_serialization() {
        use serde_json::json;

        assert_eq!(
            serde_json::to_value(SessionUpdate::AgentMessageChunk(ContentChunk::new(
                ContentBlock::Text(crate::v1::TextContent::new("Hello"))
            )))
            .unwrap(),
            json!({
                "sessionUpdate": "agent_message_chunk",
                "content": {
                    "type": "text",
                    "text": "Hello"
                }
            })
        );

        assert_eq!(
            serde_json::to_value(SessionUpdate::AgentMessageChunk(
                ContentChunk::new(ContentBlock::Text(crate::v1::TextContent::new("Hello")))
                    .message_id("msg_agent_c42b9")
            ))
            .unwrap(),
            json!({
                "sessionUpdate": "agent_message_chunk",
                "messageId": "msg_agent_c42b9",
                "content": {
                    "type": "text",
                    "text": "Hello"
                }
            })
        );

        let SessionUpdate::AgentMessageChunk(chunk) = serde_json::from_value(json!({
            "sessionUpdate": "agent_message_chunk",
            "messageId": null,
            "content": {
                "type": "text",
                "text": "Hello"
            }
        }))
        .unwrap() else {
            panic!("expected agent message chunk");
        };

        assert_eq!(chunk.message_id, None);
    }

    #[test]
    fn test_usage_update_serialization() {
        use serde_json::json;

        assert_eq!(
            serde_json::to_value(SessionUpdate::UsageUpdate(UsageUpdate::new(
                53_000, 200_000
            )))
            .unwrap(),
            json!({
                "sessionUpdate": "usage_update",
                "used": 53000,
                "size": 200_000
            })
        );

        assert_eq!(
            serde_json::to_value(SessionUpdate::UsageUpdate(
                UsageUpdate::new(53_000, 200_000).cost(Cost::new(0.045, "USD"))
            ))
            .unwrap(),
            json!({
                "sessionUpdate": "usage_update",
                "used": 53000,
                "size": 200_000,
                "cost": {
                    "amount": 0.045,
                    "currency": "USD"
                }
            })
        );

        let SessionUpdate::UsageUpdate(update) = serde_json::from_value(json!({
            "sessionUpdate": "usage_update",
            "used": 53000,
            "size": 200_000,
            "cost": null
        }))
        .unwrap() else {
            panic!("expected usage update");
        };

        assert_eq!(update.cost, None);
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

    #[test]
    fn test_client_capabilities_boolean_config_options_serialization() {
        use serde_json::json;

        let capabilities = ClientCapabilities::new().session(
            ClientSessionCapabilities::new().config_options(
                SessionConfigOptionsCapabilities::new()
                    .boolean(BooleanConfigOptionCapabilities::new()),
            ),
        );
        let json = serde_json::to_value(&capabilities).unwrap();

        assert_eq!(json["session"]["configOptions"]["boolean"], json!({}));

        let omitted: ClientCapabilities = serde_json::from_value(json!({})).unwrap();
        assert!(omitted.session.is_none());

        let null_session: ClientCapabilities = serde_json::from_value(json!({
            "session": null
        }))
        .unwrap();
        assert!(null_session.session.is_none());

        let null_config_options: ClientCapabilities = serde_json::from_value(json!({
            "session": {
                "configOptions": null
            }
        }))
        .unwrap();
        assert!(
            null_config_options
                .session
                .and_then(|session| session.config_options)
                .is_none()
        );

        let null_boolean: ClientCapabilities = serde_json::from_value(json!({
            "session": {
                "configOptions": {
                    "boolean": null
                }
            }
        }))
        .unwrap();
        assert!(
            null_boolean
                .session
                .and_then(|session| session.config_options)
                .and_then(|config_options| config_options.boolean)
                .is_none()
        );
    }

    #[cfg(feature = "unstable_plan_operations")]
    #[test]
    fn test_plan_operations_serialization() {
        use serde_json::json;

        use crate::v1::{PlanEntry, PlanEntryPriority, PlanEntryStatus, PlanUpdateContent};

        let plan_update = SessionUpdate::PlanUpdate(PlanUpdate::new(PlanUpdateContent::items(
            "plan-1",
            vec![PlanEntry::new(
                "Step 1",
                PlanEntryPriority::High,
                PlanEntryStatus::Pending,
            )],
        )));

        assert_eq!(
            serde_json::to_value(plan_update).unwrap(),
            json!({
                "sessionUpdate": "plan_update",
                "plan": {
                    "type": "items",
                    "planId": "plan-1",
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

        assert_eq!(
            serde_json::to_value(SessionUpdate::PlanRemoved(PlanRemoved::new("plan-1"))).unwrap(),
            json!({
                "sessionUpdate": "plan_removed",
                "planId": "plan-1"
            })
        );

        let capabilities = ClientCapabilities::new().plan(PlanCapabilities::new());
        let json = serde_json::to_value(&capabilities).unwrap();
        assert_eq!(json["plan"], json!({}));

        assert_eq!(
            serde_json::from_value::<ClientCapabilities>(json!({ "plan": null }))
                .unwrap()
                .plan,
            None
        );
    }

    #[cfg(feature = "unstable_mcp_over_acp")]
    #[test]
    fn test_agent_mcp_request_method_names() {
        use serde_json::json;

        let params: serde_json::Map<String, serde_json::Value> =
            [("cursor".to_string(), json!("abc"))].into_iter().collect();

        assert_eq!(CLIENT_METHOD_NAMES.mcp_message, "mcp/message");
        assert_eq!(
            AgentRequest::MessageMcpRequest(MessageMcpRequest::new(
                "server-1",
                "req-1",
                "tools/list"
            ))
            .method(),
            "mcp/message"
        );
        assert_eq!(
            serde_json::to_value(
                MessageMcpRequest::new("server-1", "req-1", "tools/list").params(params)
            )
            .unwrap(),
            json!({
                "serverId": "server-1",
                "requestId": "req-1",
                "method": "tools/list",
                "params": { "cursor": "abc" }
            })
        );

        let request_with_null_params: MessageMcpRequest = serde_json::from_value(json!({
            "serverId": "server-1",
            "requestId": "req-1",
            "method": "tools/list",
            "params": null,
            "_meta": null
        }))
        .unwrap();
        assert_eq!(request_with_null_params.params, None);
        assert_eq!(request_with_null_params.meta, None);
        for key in ["serverId", "requestId", "method"] {
            let mut value =
                json!({"serverId":"server-1", "requestId":"req-1", "method":"tools/list"});
            value.as_object_mut().unwrap().remove(key);
            assert!(serde_json::from_value::<MessageMcpRequest>(value).is_err());
        }
        for key in ["serverId", "requestId", "method"] {
            let mut value =
                json!({"serverId":"server-1", "requestId":"req-1", "method":"tools/list"});
            value[key] = serde_json::Value::Null;
            assert!(serde_json::from_value::<MessageMcpRequest>(value).is_err());
        }
    }

    #[test]
    fn request_permission_request_rejects_malformed_options() {
        use serde_json::json;

        assert!(
            serde_json::from_value::<RequestPermissionRequest>(json!({
                "sessionId": "sess-1",
                "toolCall": {"toolCallId": "tc-1"},
                "options": "not-an-array"
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<RequestPermissionRequest>(json!({
                "sessionId": "sess-1",
                "toolCall": {"toolCallId": "tc-1"},
                "options": [{"optionId": "allow"}]
            }))
            .is_err()
        );
    }
}
