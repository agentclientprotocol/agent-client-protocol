use std::sync::Arc;

use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// JSON RPC Request Id
///
/// An identifier established by the Client that MUST contain a String, Number, or NULL value if included. If it is not included it is assumed to be a notification. The value SHOULD normally not be Null \[1\] and Numbers SHOULD NOT contain fractional parts \[2\]
///
/// The Server MUST reply with the same value in the Response object if included. This member is used to correlate the context between the two objects.
///
/// \[1\] The use of Null as a value for the id member in a Request object is discouraged, because this specification uses a value of Null for Responses with an unknown id. Also, because JSON-RPC 1.0 uses an id value of Null for Notifications this could cause confusion in handling.
///
/// \[2\] Fractional parts may be problematic, since many decimal fractions cannot be represented exactly as binary fractions.
#[derive(
    Debug,
    PartialEq,
    Clone,
    Hash,
    Eq,
    Deserialize,
    Serialize,
    PartialOrd,
    Ord,
    Display,
    JsonSchema,
    From,
)]
#[serde(untagged)]
#[allow(
    clippy::exhaustive_enums,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[from(String, i64)]
pub enum RequestId {
    #[display("null")]
    Null,
    Number(i64),
    Str(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[allow(
    clippy::exhaustive_structs,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[schemars(rename = "{Params}", extend("x-docs-ignore" = true))]
#[skip_serializing_none]
pub struct Request<Params> {
    pub id: RequestId,
    pub method: Arc<str>,
    pub params: Option<Params>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[allow(
    clippy::exhaustive_enums,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[serde(untagged)]
#[schemars(rename = "{Result}", extend("x-docs-ignore" = true))]
pub enum Response<Result, Error> {
    Result { id: RequestId, result: Result },
    Error { id: RequestId, error: Error },
}

impl<R, E> Response<R, E> {
    #[must_use]
    pub fn new(id: impl Into<RequestId>, result: std::result::Result<R, E>) -> Self {
        match result {
            Ok(result) => Self::Result {
                id: id.into(),
                result,
            },
            Err(error) => Self::Error {
                id: id.into(),
                error,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[allow(
    clippy::exhaustive_structs,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[schemars(rename = "{Params}", extend("x-docs-ignore" = true))]
#[skip_serializing_none]
pub struct Notification<Params> {
    pub method: Arc<str>,
    pub params: Option<Params>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

/// A message (request, response, or notification) with `"jsonrpc": "2.0"` specified as
/// [required by JSON-RPC 2.0 Specification][1].
///
/// [1]: https://www.jsonrpc.org/specification#compatibility
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
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

    /// Unwraps the contained message.
    #[must_use]
    pub fn into_inner(self) -> M {
        self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        AgentNotification, CancelNotification, ClientNotification, ContentBlock, ContentChunk,
        SessionId, SessionNotification, SessionUpdate, TextContent,
    };
    use serde_json::{Number, Value, json};

    #[test]
    fn id_deserialization() {
        let id = serde_json::from_value::<RequestId>(Value::Null).unwrap();
        assert_eq!(id, RequestId::Null);

        let id = serde_json::from_value::<RequestId>(Value::Number(Number::from_u128(1).unwrap()))
            .unwrap();
        assert_eq!(id, RequestId::Number(1));

        let id = serde_json::from_value::<RequestId>(Value::Number(Number::from_i128(-1).unwrap()))
            .unwrap();
        assert_eq!(id, RequestId::Number(-1));

        let id = serde_json::from_value::<RequestId>(Value::String("id".to_owned())).unwrap();
        assert_eq!(id, RequestId::Str("id".to_owned()));
    }

    #[test]
    fn id_serialization() {
        let id = serde_json::to_value(RequestId::Null).unwrap();
        assert_eq!(id, Value::Null);

        let id = serde_json::to_value(RequestId::Number(1)).unwrap();
        assert_eq!(id, Value::Number(Number::from_u128(1).unwrap()));

        let id = serde_json::to_value(RequestId::Number(-1)).unwrap();
        assert_eq!(id, Value::Number(Number::from_i128(-1).unwrap()));

        let id = serde_json::to_value(RequestId::Str("id".to_owned())).unwrap();
        assert_eq!(id, Value::String("id".to_owned()));
    }

    #[test]
    fn id_display() {
        let id = RequestId::Null;
        assert_eq!(id.to_string(), "null");

        let id = RequestId::Number(1);
        assert_eq!(id.to_string(), "1");

        let id = RequestId::Number(-1);
        assert_eq!(id.to_string(), "-1");

        let id = RequestId::Str("id".to_owned());
        assert_eq!(id.to_string(), "id");
    }

    #[test]
    fn request_id_from_string_and_i64() {
        // The `#[from]` derived impls let callers pass raw types directly to
        // anything taking `impl Into<RequestId>` — e.g. `Response::new`.
        let from_str: RequestId = "abc".to_string().into();
        assert_eq!(from_str, RequestId::Str("abc".to_string()));

        let from_int: RequestId = 42i64.into();
        assert_eq!(from_int, RequestId::Number(42));
    }

    #[test]
    fn response_new_builds_result_for_ok() {
        let response: Response<i32, Value> = Response::new(7i64, Ok(123));
        match response {
            Response::Result { id, result } => {
                assert_eq!(id, RequestId::Number(7));
                assert_eq!(result, 123);
            }
            Response::Error { .. } => panic!("expected Result variant"),
        }
    }

    #[test]
    fn response_new_builds_error_for_err() {
        let response: Response<i32, Value> = Response::new("req-1".to_string(), Err(json!("nope")));
        match response {
            Response::Error { id, error } => {
                assert_eq!(id, RequestId::Str("req-1".to_string()));
                assert_eq!(error, json!("nope"));
            }
            Response::Result { .. } => panic!("expected Error variant"),
        }
    }

    #[test]
    fn jsonrpc_message_wrap_and_into_inner_round_trip() {
        let request = Request {
            id: RequestId::Number(1),
            method: "ping".into(),
            params: Some(json!({"x": 1})),
        };
        let wrapped = JsonRpcMessage::wrap(request.clone());
        let value = serde_json::to_value(&wrapped).unwrap();
        // Always emits the `"jsonrpc": "2.0"` envelope field.
        assert_eq!(value["jsonrpc"], "2.0");
        assert_eq!(value["id"], json!(1));
        assert_eq!(value["method"], "ping");
        assert_eq!(value["params"], json!({"x": 1}));

        let unwrapped = wrapped.into_inner();
        assert_eq!(unwrapped.id, request.id);
        assert_eq!(unwrapped.method, request.method);
        assert_eq!(unwrapped.params, request.params);
    }

    #[test]
    fn jsonrpc_message_rejects_wrong_jsonrpc_version() {
        // The version is hard-coded to "2.0"; anything else must fail to
        // deserialize so callers cannot accept JSON-RPC 1.0 or unknown
        // protocol envelopes by mistake.
        let value = json!({
            "jsonrpc": "1.0",
            "id": 1,
            "method": "ping",
            "params": null
        });
        assert!(serde_json::from_value::<JsonRpcMessage<Request<Value>>>(value).is_err());
    }

    #[test]
    fn jsonrpc_message_requires_jsonrpc_field() {
        let value = json!({
            "id": 1,
            "method": "ping"
        });
        assert!(serde_json::from_value::<JsonRpcMessage<Request<Value>>>(value).is_err());
    }

    #[test]
    fn request_round_trips_with_optional_params() {
        // Sanity-check that a request with `params: Some(...)` can be
        // serialized and deserialized without loss; this also exercises the
        // `JsonRpcVersion` envelope reading path.
        let request = Request {
            id: RequestId::Number(7),
            method: "do/thing".into(),
            params: Some(json!({"hello": "world"})),
        };
        let value = serde_json::to_value(&request).unwrap();
        let decoded: Request<Value> = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.id, request.id);
        assert_eq!(decoded.method, request.method);
        assert_eq!(decoded.params, request.params);
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
                        #[cfg(feature = "unstable_message_id")]
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
