use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::{Error, ErrorCode};

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Method {
    pub name: &'static str,
    pub request_type: &'static str,
    pub param_payload: bool,
    pub response_type: &'static str,
    pub response_payload: bool,
}

pub trait AnyRequest: Serialize + Sized + 'static {
    type Response: Serialize + 'static;
    fn from_method_and_params(method: &str, params: &RawValue) -> Result<Self, Error>;
    fn response_from_method_and_result(
        method: &str,
        params: &RawValue,
    ) -> Result<Self::Response, Error>;
}

macro_rules! acp_peer {
    (
        $handler_trait_name:ident,
        $request_trait_name:ident,
        $request_enum_name:ident,
        $response_enum_name:ident,
        $method_map_name:ident,
        $(($request_method:ident, $request_method_string:expr, $request_name:ident, $param_payload: tt, $response_name:ident, $response_payload: tt)),*
        $(,)?
    ) => {
        macro_rules! handler_trait_call_req {
            ($self: ident, $method: ident, false, $resp_name: ident, false, $params: ident) => {
                {
                    $self.$method()
                        .await
                        .map_err(|e| ErrorCode::INTERNAL_ERROR.into_error_with_details(e.to_string()))?;
                    Ok($response_enum_name::$resp_name($resp_name))
                }
            };
            ($self: ident, $method: ident, false, $resp_name: ident, true, $params: ident) => {
                {
                    let resp = $self.$method()
                        .await
                        .map_err(|e| ErrorCode::INTERNAL_ERROR.into_error_with_details(e.to_string()))?;
                    Ok($response_enum_name::$resp_name(resp))
                }
            };
            ($self: ident, $method: ident, true, $resp_name: ident, false, $params: ident) => {
                {
                    $self.$method($params)
                        .await
                        .map_err(|e| ErrorCode::INTERNAL_ERROR.into_error_with_details(e.to_string()))?;
                    Ok($response_enum_name::$resp_name($resp_name))
                }
            };
            ($self: ident, $method: ident, true, $resp_name: ident, true, $params: ident) => {
                {
                    let resp = $self.$method($params)
                        .await
                        .map_err(|e| ErrorCode::INTERNAL_ERROR.into_error_with_details(e.to_string()))?;
                    Ok($response_enum_name::$resp_name(resp))
                }
            }
        }

        macro_rules! handler_trait_req_method {
            ($method: ident, $req: ident, false, $resp: tt, false) => {
                fn $method(&self) -> impl Future<Output = anyhow::Result<()>>;
            };
            ($method: ident, $req: ident, false, $resp: tt, true) => {
                fn $method(&self) -> impl Future<Output = anyhow::Result<$resp>>;
            };
            ($method: ident, $req: ident, true, $resp: tt, false) => {
                fn $method(&self, request: $req) -> impl Future<Output = anyhow::Result<()>>;
            };
            ($method: ident, $req: ident, true, $resp: tt, true) => {
                fn $method(&self, request: $req) -> impl Future<Output = anyhow::Result<$resp>>;
            }
        }

        pub trait $handler_trait_name {
            fn call(&self, params: $request_enum_name) -> impl Future<Output = Result<$response_enum_name, Error>> {
                async move {
                    match params {
                        $(#[allow(unused_variables)]
                        $request_enum_name::$request_name(params) => {
                            handler_trait_call_req!(self, $request_method, $param_payload, $response_name, $response_payload, params)
                        }),*
                    }
                }
            }

            $(
                handler_trait_req_method!($request_method, $request_name, $param_payload, $response_name, $response_payload);
            )*
        }

        pub trait $request_trait_name {
            type Response;
            fn into_any(self) -> $request_enum_name;
            fn response_from_any(any: $response_enum_name) -> Result<Self::Response, Error>;
        }

        #[derive(Serialize, JsonSchema)]
        #[serde(untagged)]
        pub enum $request_enum_name {
            $(
                $request_name($request_name),
            )*
        }

        #[derive(Serialize, Deserialize, JsonSchema)]
        #[serde(untagged)]
        pub enum $response_enum_name {
            $(
                $response_name($response_name),
            )*
        }

        macro_rules! request_from_method_and_params {
            ($req_name: ident, false, $params: tt) => {
                Ok($request_enum_name::$req_name($req_name))
            };
            ($req_name: ident, true, $params: tt) => {
                match serde_json::from_str($params.get()) {
                    Ok(params) => Ok($request_enum_name::$req_name(params)),
                    Err(e) => Err(ErrorCode::PARSE_ERROR.into_error_with_details(e.to_string())),
                }
            };
        }

        macro_rules! response_from_method_and_result {
            ($resp_name: ident, false, $result: tt) => {
                Ok($response_enum_name::$resp_name($resp_name))
            };
            ($resp_name: ident, true, $result: tt) => {
                match serde_json::from_str($result.get()) {
                    Ok(result) => Ok($response_enum_name::$resp_name(result)),
                    Err(e) => Err(ErrorCode::PARSE_ERROR.into_error_with_details(e.to_string())),
                }
            };
        }

        impl AnyRequest for $request_enum_name {
            type Response = $response_enum_name;

            fn from_method_and_params(method: &str, params: &RawValue) -> Result<Self, Error> {
                match method {
                    $(
                        $request_method_string => {
                            request_from_method_and_params!($request_name, $param_payload, params)
                        }
                    )*
                    _ => Err(ErrorCode::METHOD_NOT_FOUND.into()),
                }
            }

            fn response_from_method_and_result(method: &str, params: &RawValue) -> Result<Self::Response, Error> {
                match method {
                    $(
                        $request_method_string => {
                            response_from_method_and_result!($response_name, $response_payload, params)
                        }
                    )*
                    _ => Err(ErrorCode::METHOD_NOT_FOUND.into()),
                }
            }
        }

        impl $request_enum_name {
            pub fn method_name(&self) -> &'static str {
                match self {
                    $(
                        $request_enum_name::$request_name(_) => $request_method_string,
                    )*
                }
            }
        }



        pub static $method_map_name: &[Method] = &[
            $(
                Method {
                    name: $request_method_string,
                    request_type: stringify!($request_name),
                    param_payload: $param_payload,
                    response_type: stringify!($response_name),
                    response_payload: $response_payload,
                },
            )*
        ];

        macro_rules! req_into_any {
            ($self: ident, $req_name: ident, false) => {
                $request_enum_name::$req_name($req_name)
            };
            ($self: ident, $req_name: ident, true) => {
                $request_enum_name::$req_name($self)
            };
        }

        macro_rules! resp_type {
            ($resp_name: ident, false) => {
                ()
            };
            ($resp_name: ident, true) => {
                $resp_name
            };
        }

        macro_rules! resp_from_any {
            ($any: ident, $resp_name: ident, false) => {
                match $any {
                    $response_enum_name::$resp_name(_) => Ok(()),
                    _ => Err(ErrorCode::INTERNAL_ERROR.into_error_with_details("Unexpected Response"))
                }
            };
            ($any: ident, $resp_name: ident, true) => {
                match $any {
                    $response_enum_name::$resp_name(this) => Ok(this),
                    _ => Err(ErrorCode::INTERNAL_ERROR.into_error_with_details("Unexpected Response"))
                }
            };
        }

        $(
            impl $request_trait_name for $request_name {
                type Response = resp_type!($response_name, $response_payload);

                fn into_any(self) -> $request_enum_name {
                    req_into_any!(self, $request_name, $param_payload)
                }

                fn response_from_any(any: $response_enum_name) -> Result<Self::Response, Error> {
                    resp_from_any!(any, $response_name, $response_payload)
                }
            }
        )*
    };
}

acp_peer!(
    Client,
    ClientRequest,
    AnyClientRequest,
    AnyClientResult,
    CLIENT_METHODS,
    (
        stream_assistant_message_chunk,
        "streamAssistantMessageChunk",
        StreamAssistantMessageChunkParams,
        true,
        StreamAssistantMessageChunkResponse,
        false
    ),
    (
        request_tool_call_confirmation,
        "requestToolCallConfirmation",
        RequestToolCallConfirmationParams,
        true,
        RequestToolCallConfirmationResponse,
        true
    ),
    (
        push_tool_call,
        "pushToolCall",
        PushToolCallParams,
        true,
        PushToolCallResponse,
        true
    ),
    (
        update_tool_call,
        "updateToolCall",
        UpdateToolCallParams,
        true,
        UpdateToolCallResponse,
        false
    ),
);

acp_peer!(
    Agent,
    AgentRequest,
    AnyAgentRequest,
    AnyAgentResult,
    AGENT_METHODS,
    (
        initialize,
        "initialize",
        InitializeParams,
        false,
        InitializeResponse,
        true
    ),
    (
        authenticate,
        "authenticate",
        AuthenticateParams,
        false,
        AuthenticateResponse,
        false
    ),
    (
        send_user_message,
        "sendUserMessage",
        SendUserMessageParams,
        true,
        SendUserMessageResponse,
        false
    ),
    (
        cancel_send_message,
        "cancelSendMessage",
        CancelSendMessageParams,
        false,
        CancelSendMessageResponse,
        false
    )
);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    pub is_authenticated: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateResponse;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserMessage {
    pub chunks: Vec<UserMessageChunk>,
}

impl<T> From<T> for UserMessage
where
    T: Into<UserMessageChunk>,
{
    fn from(value: T) -> Self {
        Self {
            chunks: vec![value.into()],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UserMessageChunk {
    Text { chunk: String },
    Path { path: PathBuf },
}

impl From<&str> for UserMessageChunk {
    fn from(value: &str) -> Self {
        Self::Text {
            chunk: value.into(),
        }
    }
}

impl From<&String> for UserMessageChunk {
    fn from(value: &String) -> Self {
        Self::Text {
            chunk: value.clone(),
        }
    }
}

impl From<String> for UserMessageChunk {
    fn from(value: String) -> Self {
        Self::Text { chunk: value }
    }
}

impl From<PathBuf> for UserMessageChunk {
    fn from(value: PathBuf) -> Self {
        Self::Path { path: value }
    }
}

impl From<&Path> for UserMessageChunk {
    fn from(value: &Path) -> Self {
        Self::Path { path: value.into() }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AssistantMessageChunk {
    Text { chunk: String },
    Thought { chunk: String },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreadMetadata {
    pub title: String,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendUserMessageParams {
    pub message: UserMessage,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendUserMessageResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamAssistantMessageChunkParams {
    pub chunk: AssistantMessageChunk,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamAssistantMessageChunkResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestToolCallConfirmationParams {
    pub label: String,
    pub icon: Icon,
    pub confirmation: ToolCallConfirmation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolCallContent>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Icon {
    FileSearch,
    Folder,
    Globe,
    Hammer,
    LightBulb,
    Pencil,
    Regex,
    Terminal,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolCallConfirmation {
    #[serde(rename_all = "camelCase")]
    Edit {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Execute {
        command: String,
        root_command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Mcp {
        server_name: String,
        tool_name: String,
        tool_display_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Fetch {
        urls: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Other { description: String },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct RequestToolCallConfirmationResponse {
    pub id: ToolCallId,
    pub outcome: ToolCallConfirmationOutcome,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ToolCallConfirmationOutcome {
    Allow,
    AlwaysAllow,
    AlwaysAllowMcpServer,
    AlwaysAllowTool,
    Reject,
    Cancel,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PushToolCallParams {
    pub label: String,
    pub icon: Icon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolCallContent>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct PushToolCallResponse {
    pub id: ToolCallId,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallId(pub u64);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateToolCallParams {
    pub tool_call_id: ToolCallId,
    pub status: ToolCallStatus,
    pub content: Option<ToolCallContent>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateToolCallResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ToolCallStatus {
    Running,
    Finished,
    Error,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolCallContent {
    #[serde(rename_all = "camelCase")]
    Markdown { markdown: String },
    #[serde(rename_all = "camelCase")]
    Diff {
        #[serde(flatten)]
        diff: Diff,
    },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Diff {
    pub path: PathBuf,
    pub old_text: Option<String>,
    pub new_text: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CancelSendMessageParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CancelSendMessageResponse;
