//! JSON-RPC envelope types shared by ACP clients and agents.
//!
//! These types model the JSON-RPC 2.0 request, response, notification, and
//! batch envelopes that wrap ACP method-specific payloads.

use std::sync::Arc;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// Kept here for compatibility; these are the protocol version 1 types.
pub use crate::v1::{Request, RequestId, Response};

/// A JSON-RPC notification object.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::exhaustive_structs,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[cfg_attr(feature = "schemars", schemars(rename = "{Params}", extend("x-docs-ignore" = true)))]
#[skip_serializing_none]
pub struct Notification<Params> {
    /// The notification method name.
    pub method: Arc<str>,
    /// Method-specific notification parameters.
    pub params: Option<Params>,
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", schemars(inline))]
enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

/// A message (request, response, or notification) with `"jsonrpc": "2.0"` specified as
/// [required by JSON-RPC 2.0 Specification][1].
///
/// [1]: https://www.jsonrpc.org/specification#compatibility
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct JsonRpcMessage<M> {
    jsonrpc: JsonRpcVersion,
    #[serde(flatten)]
    message: M,
}

impl<M> JsonRpcMessage<M> {
    /// Wraps the provided message into a versioned [`JsonRpcMessage`].
    #[must_use]
    pub fn wrap(message: M) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            message,
        }
    }

    /// Returns the contained message.
    #[must_use]
    pub fn inner(&self) -> &M {
        &self.message
    }

    /// Unwraps the contained message.
    #[must_use]
    pub fn into_inner(self) -> M {
        self.message
    }
}

/// Error returned when constructing an empty JSON-RPC batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[display("JSON-RPC batch must contain at least one message")]
#[non_exhaustive]
pub struct EmptyJsonRpcBatch;

impl std::error::Error for EmptyJsonRpcBatch {}

/// A non-empty JSON-RPC 2.0 batch message.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", schemars(inline))]
#[serde(transparent)]
#[allow(
    clippy::exhaustive_structs,
    reason = "This comes from the JSON-RPC specification itself"
)]
pub struct JsonRpcBatch<M>(
    #[cfg_attr(feature = "schemars", schemars(length(min = 1)))] Vec<JsonRpcMessage<M>>,
);

impl<M> JsonRpcBatch<M> {
    /// Creates a non-empty JSON-RPC batch.
    ///
    /// Returns an error if `messages` is empty, because JSON-RPC 2.0 treats an
    /// empty batch array as an invalid request.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyJsonRpcBatch`] when `messages` is empty.
    pub fn new(messages: Vec<JsonRpcMessage<M>>) -> Result<Self, EmptyJsonRpcBatch> {
        if messages.is_empty() {
            Err(EmptyJsonRpcBatch)
        } else {
            Ok(Self(messages))
        }
    }

    /// Returns the messages in this batch.
    #[must_use]
    pub fn as_slice(&self) -> &[JsonRpcMessage<M>] {
        &self.0
    }

    /// Consumes this batch and returns its messages.
    #[must_use]
    pub fn into_vec(self) -> Vec<JsonRpcMessage<M>> {
        self.0
    }
}

impl<M> TryFrom<Vec<JsonRpcMessage<M>>> for JsonRpcBatch<M> {
    type Error = EmptyJsonRpcBatch;

    fn try_from(messages: Vec<JsonRpcMessage<M>>) -> Result<Self, Self::Error> {
        Self::new(messages)
    }
}

impl<'de, M> Deserialize<'de> for JsonRpcBatch<M>
where
    M: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let messages = Vec::<JsonRpcMessage<M>>::deserialize(deserializer)?;
        Self::new(messages).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::v1::{
        AgentNotification, CancelNotification, ClientNotification, ContentBlock, ContentChunk,
        SessionId, SessionNotification, SessionUpdate, TextContent,
    };
    use serde_json::{Value, json};

    #[test]
    fn batch_deserialization_requires_at_least_one_message() {
        let err = serde_json::from_value::<JsonRpcBatch<Notification<ClientNotification>>>(
            Value::Array(Vec::new()),
        )
        .unwrap_err();
        assert!(err.to_string().contains("at least one message"));
    }

    #[test]
    fn batch_serialization_round_trips_non_empty_messages() {
        let notification = JsonRpcMessage::wrap(Notification {
            method: "cancel".into(),
            params: Some(ClientNotification::CancelNotification(CancelNotification {
                session_id: SessionId("test-123".into()),
                meta: None,
            })),
        });

        let batch = JsonRpcBatch::new(vec![notification]).unwrap();
        let serialized = serde_json::to_value(&batch).unwrap();
        assert_eq!(
            serialized,
            json!([{
                "jsonrpc": "2.0",
                "method": "cancel",
                "params": {
                    "sessionId": "test-123"
                },
            }])
        );

        let deserialized =
            serde_json::from_value::<JsonRpcBatch<Notification<ClientNotification>>>(serialized)
                .unwrap();
        assert_eq!(deserialized.as_slice().len(), 1);
        assert_eq!(deserialized.as_slice()[0].inner().method.as_ref(), "cancel");
    }

    #[test]
    fn notification_wire_format() {
        // Test client -> agent notification wire format
        let outgoing_msg = JsonRpcMessage::wrap(Notification {
            method: "cancel".into(),
            params: Some(ClientNotification::CancelNotification(CancelNotification {
                session_id: SessionId("test-123".into()),
                meta: None,
            })),
        });

        let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
        assert_eq!(
            serialized,
            json!({
                "jsonrpc": "2.0",
                "method": "cancel",
                "params": {
                    "sessionId": "test-123"
                },
            })
        );

        // Test agent -> client notification wire format
        let outgoing_msg = JsonRpcMessage::wrap(Notification {
            method: "sessionUpdate".into(),
            params: Some(AgentNotification::SessionNotification(
                SessionNotification {
                    session_id: SessionId("test-456".into()),
                    update: SessionUpdate::AgentMessageChunk(ContentChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Hello".to_string(),
                            meta: None,
                        }),
                        message_id: None,
                        meta: None,
                    }),
                    meta: None,
                },
            )),
        });

        let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
        assert_eq!(
            serialized,
            json!({
                "jsonrpc": "2.0",
                "method": "sessionUpdate",
                "params": {
                    "sessionId": "test-456",
                    "update": {
                        "sessionUpdate": "agent_message_chunk",
                        "content": {
                            "type": "text",
                            "text": "Hello"
                        }
                    }
                }
            })
        );
    }
}
