use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{RequestId, Result};

/// Direction of a message flowing through the RPC stream
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StreamMessageDirection {
    /// Message received from the other side
    Incoming,
    /// Message sent to the other side
    Outgoing,
}

/// Content of a message in the RPC stream
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum StreamMessageContent {
    /// A request message
    Request {
        /// Unique request identifier
        id: RequestId,
        /// The RPC method name
        method: Arc<str>,
        /// Method parameters, if any
        params: Option<serde_json::Value>,
    },
    /// A response to a request
    Response {
        /// The ID of the request being responded to
        id: RequestId,
        /// The response result (success or error)
        result: Result<Option<serde_json::Value>>,
    },
    /// A notification (no response expected)
    Notification {
        /// The RPC method name
        method: Arc<str>,
        /// Method parameters, if any
        params: Option<serde_json::Value>,
    },
}

/// A message flowing through the RPC stream
///
/// This type is useful for monitoring, logging, and debugging RPC message flow.
/// It combines the message content with its direction (incoming or outgoing).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct StreamMessage {
    /// The content of the message
    pub message: StreamMessageContent,
    /// The direction this message is flowing
    pub direction: StreamMessageDirection,
}

impl StreamMessage {
    /// Create a new incoming stream message
    pub fn incoming(message: StreamMessageContent) -> Self {
        Self {
            message,
            direction: StreamMessageDirection::Incoming,
        }
    }

    /// Create a new outgoing stream message
    pub fn outgoing(message: StreamMessageContent) -> Self {
        Self {
            message,
            direction: StreamMessageDirection::Outgoing,
        }
    }
}

/// Receiver for observing messages in the RPC stream
///
/// Used for monitoring and logging RPC traffic.
pub type StreamReceiver = async_broadcast::Receiver<StreamMessage>;

/// Sender for publishing messages in the RPC stream
///
/// Used internally by RPC implementations to broadcast messages.
pub type StreamSender = async_broadcast::Sender<StreamMessage>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_message_direction() {
        let incoming = StreamMessageDirection::Incoming;
        assert_eq!(incoming, StreamMessageDirection::Incoming);

        let outgoing = StreamMessageDirection::Outgoing;
        assert_eq!(outgoing, StreamMessageDirection::Outgoing);
    }

    #[test]
    fn test_stream_message_request() {
        let content = StreamMessageContent::Request {
            id: RequestId::Number(1),
            method: "test.method".into(),
            params: Some(serde_json::json!({"key": "value"})),
        };

        let message = StreamMessage::incoming(content.clone());
        assert_eq!(message.direction, StreamMessageDirection::Incoming);
        assert_eq!(message.message, content);
    }

    #[test]
    fn test_stream_message_response() {
        let content = StreamMessageContent::Response {
            id: RequestId::Number(1),
            result: Ok(Some(serde_json::json!({"success": true}))),
        };

        let message = StreamMessage::outgoing(content.clone());
        assert_eq!(message.direction, StreamMessageDirection::Outgoing);
        assert_eq!(message.message, content);
    }

    #[test]
    fn test_stream_message_notification() {
        let content = StreamMessageContent::Notification {
            method: "notify.event".into(),
            params: Some(serde_json::json!({"event": "test"})),
        };

        let message = StreamMessage::incoming(content.clone());
        assert_eq!(message.direction, StreamMessageDirection::Incoming);
        assert_eq!(message.message, content);
    }

    #[test]
    fn test_stream_message_serialization() {
        let message = StreamMessage::incoming(StreamMessageContent::Request {
            id: RequestId::Str("req-1".into()),
            method: "test.method".into(),
            params: None,
        });

        let json = serde_json::to_string(&message).expect("serialize");
        let deserialized: StreamMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(message, deserialized);
    }
}
