//! Methods and notifications the client handles/receives.
//!
//! This module defines the Client trait and all associated types for implementing
//! a client that interacts with AI coding agents via the Agent Client Protocol (ACP).

use std::{path::PathBuf, sync::Arc};

use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::{
    ContentBlock, ExtNotification, Plan, SessionId, SessionModeId, ToolCall, ToolCallUpdate,
    ext::ExtRequest,
};

// Session updates

/// Notification containing a session update from the agent.
///
/// Used to stream real-time progress and results during prompt processing.
///
/// See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[schemars(extend("x-side" = "client", "x-method" = SESSION_UPDATE_NOTIFICATION))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionNotification {
    /// The ID of the session this update pertains to.
    pub session_id: SessionId,
    /// The actual update content.
    pub update: SessionUpdate,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl SessionNotification {
    pub fn new(session_id: SessionId, update: SessionUpdate) -> Self {
        Self {
            session_id,
            update,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
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
    /// The agent's execution plan for complex tasks.
    /// See protocol docs: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)
    Plan(Plan),
    /// Available commands are ready or have changed
    AvailableCommandsUpdate(AvailableCommandsUpdate),
    /// The current mode of the session has changed
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    CurrentModeUpdate(CurrentModeUpdate),
}

/// The current mode of the session has changed
///
/// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CurrentModeUpdate {
    /// The ID of the current mode
    pub current_mode_id: SessionModeId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl CurrentModeUpdate {
    pub fn new(current_mode_id: SessionModeId) -> Self {
        Self {
            current_mode_id,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// A streamed item of content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ContentChunk {
    /// A single item of content
    pub content: ContentBlock,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl ContentChunk {
    pub fn new(content: ContentBlock) -> Self {
        Self {
            content,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Available commands are ready or have changed
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AvailableCommandsUpdate {
    /// Commands the agent can execute
    pub available_commands: Vec<AvailableCommand>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl AvailableCommandsUpdate {
    pub fn new(available_commands: Vec<AvailableCommand>) -> Self {
        Self {
            available_commands,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Information about a command.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AvailableCommand {
    /// Command name (e.g., `create_plan`, `research_codebase`).
    pub name: String,
    /// Human-readable description of what the command does.
    pub description: String,
    /// Input for the command if required
    pub input: Option<AvailableCommandInput>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl AvailableCommand {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input: None,
            meta: None,
        }
    }

    /// Input for the command if required
    pub fn input(mut self, input: AvailableCommandInput) -> Self {
        self.input = Some(input);
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
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
}

/// All text that was typed after the command name is provided as input.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UnstructuredCommandInput {
    /// A hint to display when the input hasn't been provided yet
    pub hint: String,
}

impl UnstructuredCommandInput {
    pub fn new(hint: impl Into<String>) -> Self {
        Self { hint: hint.into() }
    }
}

// Permission

/// Request for user permission to execute a tool call.
///
/// Sent when the agent needs authorization before performing a sensitive operation.
///
/// See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
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
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl RequestPermissionRequest {
    pub fn new(
        session_id: SessionId,
        tool_call: ToolCallUpdate,
        options: Vec<PermissionOption>,
    ) -> Self {
        Self {
            session_id,
            tool_call,
            options,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// An option presented to the user when requesting permission.
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
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl PermissionOption {
    pub fn new(
        option_id: PermissionOptionId,
        name: impl Into<String>,
        kind: PermissionOptionKind,
    ) -> Self {
        Self {
            option_id,
            name: name.into(),
            kind,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Unique identifier for a permission option.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
pub struct PermissionOptionId(pub Arc<str>);

/// The type of permission option being presented to the user.
///
/// Helps clients choose appropriate icons and UI treatment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "client", "x-method" = SESSION_REQUEST_PERMISSION_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RequestPermissionResponse {
    /// The user's decision on the permission request.
    // This extra-level is unfortunately needed because the output must be an object
    pub outcome: RequestPermissionOutcome,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl RequestPermissionResponse {
    pub fn new(outcome: RequestPermissionOutcome) -> Self {
        Self {
            outcome,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SelectedPermissionOutcome {
    /// The ID of the option the user selected.
    pub option_id: PermissionOptionId,
}

impl SelectedPermissionOutcome {
    pub fn new(option_id: PermissionOptionId) -> Self {
        Self { option_id }
    }
}

// Write text file

/// Request to write content to a text file.
///
/// Only available if the client supports the `fs.writeTextFile` capability.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "client", "x-method" = FS_WRITE_TEXT_FILE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WriteTextFileRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// Absolute path to the file to write.
    pub path: PathBuf,
    /// The text content to write to the file.
    pub content: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl WriteTextFileRequest {
    pub fn new(
        session_id: SessionId,
        path: impl Into<PathBuf>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            session_id,
            path: path.into(),
            content: content.into(),
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Response to `fs/write_text_file`
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = FS_WRITE_TEXT_FILE_METHOD_NAME))]
#[non_exhaustive]
pub struct WriteTextFileResponse {
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl WriteTextFileResponse {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

// Read text file

/// Request to read content from a text file.
///
/// Only available if the client supports the `fs.readTextFile` capability.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "client", "x-method" = FS_READ_TEXT_FILE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ReadTextFileRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// Absolute path to the file to read.
    pub path: PathBuf,
    /// Line number to start reading from (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Maximum number of lines to read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl ReadTextFileRequest {
    pub fn new(session_id: SessionId, path: impl Into<PathBuf>) -> Self {
        Self {
            session_id,
            path: path.into(),
            line: None,
            limit: None,
            meta: None,
        }
    }

    /// Line number to start reading from (1-based).
    pub fn line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Maximum number of lines to read.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Response containing the contents of a text file.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "client", "x-method" = FS_READ_TEXT_FILE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ReadTextFileResponse {
    pub content: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl ReadTextFileResponse {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

// Terminals

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
pub struct TerminalId(pub Arc<str>);

/// Request to create a new terminal and execute a command.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = TERMINAL_CREATE_METHOD_NAME))]
#[non_exhaustive]
pub struct CreateTerminalRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// The command to execute.
    pub command: String,
    /// Array of command arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Environment variables for the command.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<crate::EnvVariable>,
    /// Working directory for the command (absolute path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    /// Maximum number of output bytes to retain.
    ///
    /// When the limit is exceeded, the Client truncates from the beginning of the output
    /// to stay within the limit.
    ///
    /// The Client MUST ensure truncation happens at a character boundary to maintain valid
    /// string output, even if this means the retained output is slightly less than the
    /// specified limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_byte_limit: Option<u64>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl CreateTerminalRequest {
    pub fn new(session_id: SessionId, command: impl Into<String>) -> Self {
        Self {
            session_id,
            command: command.into(),
            args: Vec::new(),
            env: Vec::new(),
            cwd: None,
            output_byte_limit: None,
            meta: None,
        }
    }

    /// Array of command arguments.
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Environment variables for the command.
    pub fn env(mut self, env: Vec<crate::EnvVariable>) -> Self {
        self.env = env;
        self
    }

    /// Working directory for the command (absolute path).
    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
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
    pub fn output_byte_limit(mut self, output_byte_limit: u64) -> Self {
        self.output_byte_limit = Some(output_byte_limit);
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Response containing the ID of the created terminal.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = TERMINAL_CREATE_METHOD_NAME))]
#[non_exhaustive]
pub struct CreateTerminalResponse {
    /// The unique identifier for the created terminal.
    pub terminal_id: TerminalId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl CreateTerminalResponse {
    pub fn new(terminal_id: TerminalId) -> Self {
        Self {
            terminal_id,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Request to get the current output and status of a terminal.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = TERMINAL_OUTPUT_METHOD_NAME))]
#[non_exhaustive]
pub struct TerminalOutputRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// The ID of the terminal to get output from.
    pub terminal_id: TerminalId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl TerminalOutputRequest {
    pub fn new(session_id: SessionId, terminal_id: TerminalId) -> Self {
        Self {
            session_id,
            terminal_id,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Response containing the terminal output and exit status.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = TERMINAL_OUTPUT_METHOD_NAME))]
#[non_exhaustive]
pub struct TerminalOutputResponse {
    /// The terminal output captured so far.
    pub output: String,
    /// Whether the output was truncated due to byte limits.
    pub truncated: bool,
    /// Exit status if the command has completed.
    pub exit_status: Option<TerminalExitStatus>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl TerminalOutputResponse {
    pub fn new(output: impl Into<String>, truncated: bool) -> Self {
        Self {
            output: output.into(),
            truncated,
            exit_status: None,
            meta: None,
        }
    }

    /// Exit status if the command has completed.
    pub fn exit_status(mut self, exit_status: TerminalExitStatus) -> Self {
        self.exit_status = Some(exit_status);
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Request to release a terminal and free its resources.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = TERMINAL_RELEASE_METHOD_NAME))]
#[non_exhaustive]
pub struct ReleaseTerminalRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// The ID of the terminal to release.
    pub terminal_id: TerminalId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl ReleaseTerminalRequest {
    pub fn new(session_id: SessionId, terminal_id: TerminalId) -> Self {
        Self {
            session_id,
            terminal_id,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Response to terminal/release method
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = TERMINAL_RELEASE_METHOD_NAME))]
#[non_exhaustive]
pub struct ReleaseTerminalResponse {
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl ReleaseTerminalResponse {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Request to kill a terminal command without releasing the terminal.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = TERMINAL_KILL_METHOD_NAME))]
#[non_exhaustive]
pub struct KillTerminalCommandRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// The ID of the terminal to kill.
    pub terminal_id: TerminalId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl KillTerminalCommandRequest {
    pub fn new(session_id: SessionId, terminal_id: TerminalId) -> Self {
        Self {
            session_id,
            terminal_id,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Response to terminal/kill command method
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = TERMINAL_KILL_METHOD_NAME))]
#[non_exhaustive]
pub struct KillTerminalCommandResponse {
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl KillTerminalCommandResponse {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Request to wait for a terminal command to exit.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = TERMINAL_WAIT_FOR_EXIT_METHOD_NAME))]
#[non_exhaustive]
pub struct WaitForTerminalExitRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// The ID of the terminal to wait for.
    pub terminal_id: TerminalId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl WaitForTerminalExitRequest {
    pub fn new(session_id: SessionId, terminal_id: TerminalId) -> Self {
        Self {
            session_id,
            terminal_id,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Response containing the exit status of a terminal command.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = TERMINAL_WAIT_FOR_EXIT_METHOD_NAME))]
#[non_exhaustive]
pub struct WaitForTerminalExitResponse {
    /// The exit status of the terminal command.
    #[serde(flatten)]
    pub exit_status: TerminalExitStatus,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl WaitForTerminalExitResponse {
    pub fn new(exit_status: TerminalExitStatus) -> Self {
        Self {
            exit_status,
            meta: None,
        }
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Exit status of a terminal command.
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TerminalExitStatus {
    /// The process exit code (may be null if terminated by signal).
    pub exit_code: Option<u32>,
    /// The signal that terminated the process (may be null if exited normally).
    pub signal: Option<String>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl TerminalExitStatus {
    pub fn new() -> Self {
        Self::default()
    }

    /// The process exit code (may be null if terminated by signal).
    pub fn exit_code(mut self, exit_code: u32) -> Self {
        self.exit_code = Some(exit_code);
        self
    }

    /// The signal that terminated the process (may be null if exited normally).
    pub fn signal(mut self, signal: impl Into<String>) -> Self {
        self.signal = Some(signal.into());
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
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
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ClientCapabilities {
    /// File system capabilities supported by the client.
    /// Determines which file operations the agent can request.
    #[serde(default)]
    pub fs: FileSystemCapability,
    /// Whether the Client support all `terminal/*` methods.
    #[serde(default)]
    pub terminal: bool,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl ClientCapabilities {
    pub fn new() -> Self {
        Self::default()
    }

    /// File system capabilities supported by the client.
    /// Determines which file operations the agent can request.
    pub fn fs(mut self, fs: FileSystemCapability) -> Self {
        self.fs = fs;
        self
    }

    /// Whether the Client support all `terminal/*` methods.
    pub fn terminal(mut self, terminal: bool) -> Self {
        self.terminal = terminal;
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Filesystem capabilities supported by the client.
/// File system capabilities that a client may support.
///
/// See protocol docs: [FileSystem](https://agentclientprotocol.com/protocol/initialization#filesystem)
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FileSystemCapability {
    /// Whether the Client supports `fs/read_text_file` requests.
    #[serde(default)]
    pub read_text_file: bool,
    /// Whether the Client supports `fs/write_text_file` requests.
    #[serde(default)]
    pub write_text_file: bool,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl FileSystemCapability {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the Client supports `fs/read_text_file` requests.
    pub fn read_text_file(mut self, read_text_file: bool) -> Self {
        self.read_text_file = read_text_file;
        self
    }

    /// Whether the Client supports `fs/write_text_file` requests.
    pub fn write_text_file(mut self, write_text_file: bool) -> Self {
        self.write_text_file = write_text_file;
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
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

/// All possible requests that an agent can send to a client.
///
/// This enum is used internally for routing RPC requests. You typically won't need
/// to use this directly - instead, use the methods on the [`Client`] trait.
///
/// This enum encompasses all method calls from agent to client.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("x-docs-ignore" = true))]
#[non_exhaustive]
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
    /// Note: `terminal/release` when `TerminalId` is no longer needed.
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    KillTerminalCommandRequest(KillTerminalCommandRequest),
    /// Handles extension method requests from the agent.
    ///
    /// Allows the Agent to send an arbitrary request that is not part of the ACP spec.
    /// Extension methods provide a way to add custom functionality while maintaining
    /// protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    ExtMethodRequest(ExtRequest),
}

/// All possible responses that a client can send to an agent.
///
/// This enum is used internally for routing RPC responses. You typically won't need
/// to use this directly - the responses are handled automatically by the connection.
///
/// These are responses to the corresponding `AgentRequest` variants.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("x-docs-ignore" = true))]
#[non_exhaustive]
pub enum ClientResponse {
    WriteTextFileResponse(#[serde(default)] WriteTextFileResponse),
    ReadTextFileResponse(ReadTextFileResponse),
    RequestPermissionResponse(RequestPermissionResponse),
    CreateTerminalResponse(CreateTerminalResponse),
    TerminalOutputResponse(TerminalOutputResponse),
    ReleaseTerminalResponse(#[serde(default)] ReleaseTerminalResponse),
    WaitForTerminalExitResponse(WaitForTerminalExitResponse),
    KillTerminalResponse(#[serde(default)] KillTerminalCommandResponse),
    ExtMethodResponse(#[schemars(with = "serde_json::Value")] Arc<RawValue>),
}

/// All possible notifications that an agent can send to a client.
///
/// This enum is used internally for routing RPC notifications. You typically won't need
/// to use this directly - use the notification methods on the [`Client`] trait instead.
///
/// Notifications do not expect a response.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[expect(clippy::large_enum_variant)]
#[schemars(extend("x-docs-ignore" = true))]
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
    /// Handles extension notifications from the agent.
    ///
    /// Allows the Agent to send an arbitrary notification that is not part of the ACP spec.
    /// Extension notifications provide a way to send one-way messages for custom functionality
    /// while maintaining protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    ExtNotification(ExtNotification),
}
