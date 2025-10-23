use std::sync::Arc;

use derive_more::Display;
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
    Debug, PartialEq, Clone, Hash, Eq, Deserialize, Serialize, PartialOrd, Ord, Display, JsonSchema,
)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
#[schemars(inline)]
pub enum RequestId {
    #[display("null")]
    #[schemars(title = "null")]
    Null,
    #[schemars(title = "number")]
    Number(i64),
    #[schemars(title = "string")]
    Str(String),
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
pub enum OutgoingMessage<Local: Side, Remote: Side> {
    Request {
        id: RequestId,
        method: Arc<str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Remote::InRequest>,
    },
    Response {
        id: RequestId,
        #[serde(flatten)]
        result: ResponseResult<Local::OutResponse>,
    },
    Notification {
        method: Arc<str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Remote::InNotification>,
    },
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

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseResult<Res> {
    Result(Res),
    Error(Error),
}

impl<T> From<Result<T>> for ResponseResult<T> {
    fn from(result: Result<T>) -> Self {
        match result {
            Ok(value) => ResponseResult::Result(value),
            Err(error) => ResponseResult::Error(error),
        }
    }
}

pub trait Side: Clone {
    type InRequest: Clone + Serialize + DeserializeOwned + JsonSchema + 'static;
    type InNotification: Clone + Serialize + DeserializeOwned + JsonSchema + 'static;
    type OutResponse: Clone + Serialize + DeserializeOwned + JsonSchema + 'static;

    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<Self::InRequest>;

    fn decode_notification(method: &str, params: Option<&RawValue>)
    -> Result<Self::InNotification>;
}

/// Marker type representing the client side of an ACP connection.
///
/// This type is used by the RPC layer to determine which messages
/// are incoming vs outgoing from the client's perspective.
///
/// See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)
#[derive(Clone, JsonSchema)]
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
#[derive(Clone, JsonSchema)]
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
            m if m == AGENT_METHOD_NAMES.session_set_mode => serde_json::from_str(params.get())
                .map(ClientRequest::SetSessionModeRequest)
                .map_err(Into::into),
            #[cfg(feature = "unstable")]
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
