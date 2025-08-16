//! Methods and notifications the client handles/receives

use std::{fmt, path::PathBuf, sync::Arc};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ContentBlock, Error, Plan, SessionId, ToolCall, ToolCallId, ToolCallUpdate};

pub trait Client {
    fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> impl Future<Output = Result<RequestPermissionResponse, Error>>;

    fn write_text_file(
        &self,
        args: WriteTextFileRequest,
    ) -> impl Future<Output = Result<(), Error>>;

    fn read_text_file(
        &self,
        args: ReadTextFileRequest,
    ) -> impl Future<Output = Result<ReadTextFileResponse, Error>>;

    fn session_notification(
        &self,
        args: SessionNotification,
    ) -> impl Future<Output = Result<(), Error>>;
}

// Session updates

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionNotification {
    pub session_id: SessionId,
    pub update: SessionUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "sessionUpdate", rename_all = "snake_case")]
pub enum SessionUpdate {
    UserMessageChunk { content: ContentBlock },
    AgentMessageChunk { content: ContentBlock },
    AgentThoughtChunk { content: ContentBlock },
    ToolCall(ToolCall),
    ToolCallUpdate(ToolCallUpdate),
    Plan(Plan),
}

// Permission

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestPermissionRequest {
    pub session_id: SessionId,
    pub tool_call: ToolCallRef,
    pub options: Vec<PermissionOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ToolCallRef {
    Id(ToolCallId),
    Inline(ToolCall),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PermissionOption {
    #[serde(rename = "optionId")]
    pub id: PermissionOptionId,
    pub name: String,
    pub kind: PermissionOptionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PermissionOptionId(pub Arc<str>);

impl fmt::Display for PermissionOptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionOptionKind {
    AllowOnce,
    AllowAlways,
    RejectOnce,
    RejectAlways,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestPermissionResponse {
    // This extra-level is unfortunately needed because the output must be an object
    pub outcome: RequestPermissionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum RequestPermissionOutcome {
    Cancelled,
    #[serde(rename_all = "camelCase")]
    Selected {
        option_id: PermissionOptionId,
    },
}

// Write text file

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WriteTextFileRequest {
    pub session_id: SessionId,
    pub path: PathBuf,
    pub content: String,
}

// Read text file

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileRequest {
    pub session_id: SessionId,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileResponse {
    pub content: String,
}

// Capabilities

/// Capabilities supported by the client
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// FileSystem capabilities supported by the client.
    #[serde(default)]
    pub fs: FileSystemCapability,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemCapability {
    /// Client supports `fs/read_text_file`
    #[serde(default)]
    pub read_text_file: bool,
    /// Client supports `fs/write_text_file`
    #[serde(default)]
    pub write_text_file: bool,
}

// Method schema

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMethodNames {
    pub session_request_permission: &'static str,
    pub session_update: &'static str,
    pub fs_write_text_file: &'static str,
    pub fs_read_text_file: &'static str,
}

pub const CLIENT_METHOD_NAMES: ClientMethodNames = ClientMethodNames {
    session_update: SESSION_UPDATE_NOTIFICATION,
    session_request_permission: SESSION_REQUEST_PERMISSION_METHOD_NAME,
    fs_write_text_file: FS_WRITE_TEXT_FILE_METHOD_NAME,
    fs_read_text_file: FS_READ_TEXT_FILE_METHOD_NAME,
};

pub const SESSION_UPDATE_NOTIFICATION: &str = "session/update";
pub const SESSION_REQUEST_PERMISSION_METHOD_NAME: &str = "session/request_permission";
pub const FS_WRITE_TEXT_FILE_METHOD_NAME: &str = "fs/write_text_file";
pub const FS_READ_TEXT_FILE_METHOD_NAME: &str = "fs/read_text_file";

/// Requests the agent sends to the client
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AgentRequest {
    WriteTextFileRequest(WriteTextFileRequest),
    ReadTextFileRequest(ReadTextFileRequest),
    RequestPermissionRequest(RequestPermissionRequest),
}

/// Responses the client sends to the agent
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ClientResponse {
    WriteTextFileResponse,
    ReadTextFileResponse(ReadTextFileResponse),
    RequestPermissionResponse(RequestPermissionResponse),
}

/// Notifications the agent sends to the client
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AgentNotification {
    SessionNotification(SessionNotification),
}
