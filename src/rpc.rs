use std::sync::Arc;

use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::value::RawValue;

use crate::{
    AGENT_METHOD_NAMES, AgentNotification, AgentRequest, AgentResponse, CLIENT_METHOD_NAMES,
    ClientNotification, ClientRequest, ClientResponse, Error, ExtNotification, ExtRequest, Result,
};

/// JSON RPC Request Id
///
/// An identifier established by the Client that MUST contain a String, Number, or NULL value if included. If it is not included it is assumed to be a notification. The value SHOULD normally not be Null [1] and Numbers SHOULD NOT contain fractional parts [2]
///
/// The Server MUST reply with the same value in the Response object if included. This member is used to correlate the context between the two objects.
///
/// [1] The use of Null as a value for the id member in a Request object is discouraged, because this specification uses a value of Null for Responses with an unknown id. Also, because JSON-RPC 1.0 uses an id value of Null for Notifications this could cause confusion in handling.
///
/// [2] Fractional parts may be problematic, since many decimal fractions cannot be represented exactly as binary fractions.
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
pub struct Request<Params> {
    pub id: RequestId,
    pub method: Arc<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Params>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[allow(
    clippy::exhaustive_enums,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[serde(untagged)]
#[schemars(rename = "{Result}", extend("x-docs-ignore" = true))]
pub enum Response<Result> {
    Result { id: RequestId, result: Result },
    Error { id: RequestId, error: Error },
}

impl<R> Response<R> {
    pub fn new(id: impl Into<RequestId>, result: Result<R>) -> Self {
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
pub struct Notification<Params> {
    pub method: Arc<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Params>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[allow(
    clippy::exhaustive_enums,
    reason = "This comes from the JSON-RPC specification itself"
)]
pub enum OutgoingMessage<Local: Side, Remote: Side> {
    Request(Request<Remote::InRequest>),
    Response(Response<Local::OutResponse>),
    Notification(Notification<Remote::InNotification>),
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
    /// Wraps the provided [`OutgoingMessage`] or [`IncomingMessage`] into a versioned
    /// [`JsonRpcMessage`].
    #[must_use]
    pub fn wrap(message: M) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            message,
        }
    }
}

pub trait Side: Clone {
    type InRequest: Clone + Serialize + DeserializeOwned + JsonSchema + 'static;
    type InNotification: Clone + Serialize + DeserializeOwned + JsonSchema + 'static;
    type OutResponse: Clone + Serialize + DeserializeOwned + JsonSchema + 'static;

    /// Decode a request for a given method. This will encapsulate the knowledge of mapping which
    /// serialization struct to use for each method.
    ///
    /// # Errors
    ///
    /// This function will return an error if the method is not recognized or if the parameters
    /// cannot be deserialized into the expected type.
    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<Self::InRequest>;

    /// Decode a notification for a given method. This will encapsulate the knowledge of mapping which
    /// serialization struct to use for each method.
    ///
    /// # Errors
    ///
    /// This function will return an error if the method is not recognized or if the parameters
    /// cannot be deserialized into the expected type.
    fn decode_notification(method: &str, params: Option<&RawValue>)
    -> Result<Self::InNotification>;
}

/// Marker type representing the client side of an ACP connection.
///
/// This type is used by the RPC layer to determine which messages
/// are incoming vs outgoing from the client's perspective.
///
/// See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)
#[derive(Clone, Default, Debug, JsonSchema)]
#[non_exhaustive]
pub struct ClientSide;

impl Side for ClientSide {
    type InRequest = AgentRequest;
    type InNotification = AgentNotification;
    type OutResponse = ClientResponse;

    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<AgentRequest> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            m if m == CLIENT_METHOD_NAMES.session_request_permission => {
                serde_json::from_str(params.get())
                    .map(AgentRequest::RequestPermissionRequest)
                    .map_err(Into::into)
            }
            m if m == CLIENT_METHOD_NAMES.fs_write_text_file => serde_json::from_str(params.get())
                .map(AgentRequest::WriteTextFileRequest)
                .map_err(Into::into),
            m if m == CLIENT_METHOD_NAMES.fs_read_text_file => serde_json::from_str(params.get())
                .map(AgentRequest::ReadTextFileRequest)
                .map_err(Into::into),
            m if m == CLIENT_METHOD_NAMES.terminal_create => serde_json::from_str(params.get())
                .map(AgentRequest::CreateTerminalRequest)
                .map_err(Into::into),
            m if m == CLIENT_METHOD_NAMES.terminal_output => serde_json::from_str(params.get())
                .map(AgentRequest::TerminalOutputRequest)
                .map_err(Into::into),
            m if m == CLIENT_METHOD_NAMES.terminal_kill => serde_json::from_str(params.get())
                .map(AgentRequest::KillTerminalCommandRequest)
                .map_err(Into::into),
            m if m == CLIENT_METHOD_NAMES.terminal_release => serde_json::from_str(params.get())
                .map(AgentRequest::ReleaseTerminalRequest)
                .map_err(Into::into),
            m if m == CLIENT_METHOD_NAMES.terminal_wait_for_exit => {
                serde_json::from_str(params.get())
                    .map(AgentRequest::WaitForTerminalExitRequest)
                    .map_err(Into::into)
            }
            _ => {
                if let Some(custom_method) = method.strip_prefix('_') {
                    Ok(AgentRequest::ExtMethodRequest(ExtRequest {
                        method: custom_method.into(),
                        params: params.to_owned().into(),
                    }))
                } else {
                    Err(Error::method_not_found())
                }
            }
        }
    }

    fn decode_notification(method: &str, params: Option<&RawValue>) -> Result<AgentNotification> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            m if m == CLIENT_METHOD_NAMES.session_update => serde_json::from_str(params.get())
                .map(AgentNotification::SessionNotification)
                .map_err(Into::into),
            _ => {
                if let Some(custom_method) = method.strip_prefix('_') {
                    Ok(AgentNotification::ExtNotification(ExtNotification {
                        method: custom_method.into(),
                        params: RawValue::from_string(params.get().to_string())?.into(),
                    }))
                } else {
                    Err(Error::method_not_found())
                }
            }
        }
    }
}

/// Marker type representing the agent side of an ACP connection.
///
/// This type is used by the RPC layer to determine which messages
/// are incoming vs outgoing from the agent's perspective.
///
/// See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)
#[derive(Clone, Default, Debug, JsonSchema)]
#[non_exhaustive]
pub struct AgentSide;

impl Side for AgentSide {
    type InRequest = ClientRequest;
    type InNotification = ClientNotification;
    type OutResponse = AgentResponse;

    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<ClientRequest> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            m if m == AGENT_METHOD_NAMES.initialize => serde_json::from_str(params.get())
                .map(ClientRequest::InitializeRequest)
                .map_err(Into::into),
            m if m == AGENT_METHOD_NAMES.authenticate => serde_json::from_str(params.get())
                .map(ClientRequest::AuthenticateRequest)
                .map_err(Into::into),
            m if m == AGENT_METHOD_NAMES.session_new => serde_json::from_str(params.get())
                .map(ClientRequest::NewSessionRequest)
                .map_err(Into::into),
            m if m == AGENT_METHOD_NAMES.session_load => serde_json::from_str(params.get())
                .map(ClientRequest::LoadSessionRequest)
                .map_err(Into::into),
            #[cfg(feature = "unstable_session_list")]
            m if m == AGENT_METHOD_NAMES.session_list => serde_json::from_str(params.get())
                .map(ClientRequest::ListSessionsRequest)
                .map_err(Into::into),
            #[cfg(feature = "unstable_session_fork")]
            m if m == AGENT_METHOD_NAMES.session_fork => serde_json::from_str(params.get())
                .map(ClientRequest::ForkSessionRequest)
                .map_err(Into::into),
            #[cfg(feature = "unstable_session_resume")]
            m if m == AGENT_METHOD_NAMES.session_resume => serde_json::from_str(params.get())
                .map(ClientRequest::ResumeSessionRequest)
                .map_err(Into::into),
            m if m == AGENT_METHOD_NAMES.session_set_mode => serde_json::from_str(params.get())
                .map(ClientRequest::SetSessionModeRequest)
                .map_err(Into::into),
            #[cfg(feature = "unstable_session_model")]
            m if m == AGENT_METHOD_NAMES.session_set_model => serde_json::from_str(params.get())
                .map(ClientRequest::SetSessionModelRequest)
                .map_err(Into::into),
            m if m == AGENT_METHOD_NAMES.session_prompt => serde_json::from_str(params.get())
                .map(ClientRequest::PromptRequest)
                .map_err(Into::into),
            _ => {
                if let Some(custom_method) = method.strip_prefix('_') {
                    Ok(ClientRequest::ExtMethodRequest(ExtRequest {
                        method: custom_method.into(),
                        params: params.to_owned().into(),
                    }))
                } else {
                    Err(Error::method_not_found())
                }
            }
        }
    }

    fn decode_notification(method: &str, params: Option<&RawValue>) -> Result<ClientNotification> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            m if m == AGENT_METHOD_NAMES.session_cancel => serde_json::from_str(params.get())
                .map(ClientNotification::CancelNotification)
                .map_err(Into::into),
            _ => {
                if let Some(custom_method) = method.strip_prefix('_') {
                    Ok(ClientNotification::ExtNotification(ExtNotification {
                        method: custom_method.into(),
                        params: RawValue::from_string(params.get().to_string())?.into(),
                    }))
                } else {
                    Err(Error::method_not_found())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::{Number, Value};

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
}

#[test]
fn test_notification_wire_format() {
    use super::*;

    use serde_json::{Value, json};

    // Test client -> agent notification wire format
    let outgoing_msg = JsonRpcMessage::wrap(
        OutgoingMessage::<ClientSide, AgentSide>::Notification(Notification {
            method: "cancel".into(),
            params: Some(ClientNotification::CancelNotification(CancelNotification {
                session_id: SessionId("test-123".into()),
                meta: None,
            })),
        }),
    );

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
    let outgoing_msg = JsonRpcMessage::wrap(
        OutgoingMessage::<AgentSide, ClientSide>::Notification(Notification {
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
                        meta: None,
                    }),
                    meta: None,
                },
            )),
        }),
    );

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
