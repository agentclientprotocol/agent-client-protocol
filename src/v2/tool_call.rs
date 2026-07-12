//! Tool calls represent actions that language models request agents to perform.
//!
//! When an LLM determines it needs to interact with external systems—like reading files,
//! running code, or fetching data—it generates tool calls that the agent executes on its behalf.
//!
/// See protocol docs: [Tool Calls](https://agentclientprotocol.com/protocol/tool-calls)
use std::{path::PathBuf, sync::Arc};

use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, VecSkipError, serde_as, skip_serializing_none};

use super::{ContentBlock, Error, Meta, TerminalId};
use crate::{IntoOption, SkipListener};

/// Represents a tool call that the language model has requested.
///
/// Tool calls are actions that the agent executes on behalf of the language model,
/// such as reading files, executing code, or fetching data from external sources.
///
/// See protocol docs: [Tool Calls](https://agentclientprotocol.com/protocol/tool-calls)
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolCall {
    /// Unique identifier for this tool call within the session.
    pub tool_call_id: ToolCallId,
    /// Human-readable title describing what the tool is doing.
    pub title: String,
    /// The category of tool being invoked.
    /// Helps clients choose appropriate icons and UI treatment.
    #[serde(default, skip_serializing_if = "ToolKind::is_default")]
    pub kind: ToolKind,
    /// Current execution status of the tool call.
    #[serde(default, skip_serializing_if = "ToolCallStatus::is_default")]
    pub status: ToolCallStatus,
    /// Content produced by the tool call.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, SkipListener>>")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ToolCallContent>,
    /// File locations affected by this tool call.
    /// Enables "follow-along" features in clients.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, SkipListener>>")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<ToolCallLocation>,
    /// Raw input parameters sent to the tool.
    pub raw_input: Option<serde_json::Value>,
    /// Raw output returned by the tool.
    pub raw_output: Option<serde_json::Value>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ToolCall {
    #[must_use]
    pub fn new(tool_call_id: impl Into<ToolCallId>, title: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            title: title.into(),
            kind: ToolKind::default(),
            status: ToolCallStatus::default(),
            content: Vec::default(),
            locations: Vec::default(),
            raw_input: None,
            raw_output: None,
            meta: None,
        }
    }

    /// The category of tool being invoked.
    /// Helps clients choose appropriate icons and UI treatment.
    #[must_use]
    pub fn kind(mut self, kind: ToolKind) -> Self {
        self.kind = kind;
        self
    }

    /// Current execution status of the tool call.
    #[must_use]
    pub fn status(mut self, status: ToolCallStatus) -> Self {
        self.status = status;
        self
    }

    /// Content produced by the tool call.
    #[must_use]
    pub fn content(mut self, content: Vec<ToolCallContent>) -> Self {
        self.content = content;
        self
    }

    /// File locations affected by this tool call.
    /// Enables "follow-along" features in clients.
    #[must_use]
    pub fn locations(mut self, locations: Vec<ToolCallLocation>) -> Self {
        self.locations = locations;
        self
    }

    /// Raw input parameters sent to the tool.
    #[must_use]
    pub fn raw_input(mut self, raw_input: impl IntoOption<serde_json::Value>) -> Self {
        self.raw_input = raw_input.into_option();
        self
    }

    /// Raw output returned by the tool.
    #[must_use]
    pub fn raw_output(mut self, raw_output: impl IntoOption<serde_json::Value>) -> Self {
        self.raw_output = raw_output.into_option();
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

    /// Update an existing tool call with the values in the provided update
    /// fields. Fields with collections of values are overwritten, not extended.
    pub fn update(&mut self, fields: ToolCallUpdateFields) {
        if let Some(title) = fields.title {
            self.title = title;
        }
        if let Some(kind) = fields.kind {
            self.kind = kind;
        }
        if let Some(status) = fields.status {
            self.status = status;
        }
        if let Some(content) = fields.content {
            self.content = content;
        }
        if let Some(locations) = fields.locations {
            self.locations = locations;
        }
        if let Some(raw_input) = fields.raw_input {
            self.raw_input = Some(raw_input);
        }
        if let Some(raw_output) = fields.raw_output {
            self.raw_output = Some(raw_output);
        }
    }
}

/// An update to an existing tool call.
///
/// Used to report progress and results as tools execute. All fields except
/// the tool call ID are optional - only changed fields need to be included.
///
/// See protocol docs: [Updating](https://agentclientprotocol.com/protocol/tool-calls#updating)
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolCallUpdate {
    /// The ID of the tool call being updated.
    pub tool_call_id: ToolCallId,
    /// Fields being updated.
    #[serde(flatten)]
    pub fields: ToolCallUpdateFields,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ToolCallUpdate {
    #[must_use]
    pub fn new(tool_call_id: impl Into<ToolCallId>, fields: ToolCallUpdateFields) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            fields,
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

/// Optional fields that can be updated in a tool call.
///
/// All fields are optional - only include the ones being changed.
/// Collections (content, locations) are overwritten, not extended.
///
/// See protocol docs: [Updating](https://agentclientprotocol.com/protocol/tool-calls#updating)
#[serde_as]
#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolCallUpdateFields {
    /// Update the tool kind.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub kind: Option<ToolKind>,
    /// Update the execution status.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub status: Option<ToolCallStatus>,
    /// Update the human-readable title.
    pub title: Option<String>,
    /// Replace the content collection.
    #[serde_as(deserialize_as = "DefaultOnError<Option<VecSkipError<_, SkipListener>>>")]
    #[serde(default)]
    pub content: Option<Vec<ToolCallContent>>,
    /// Replace the locations collection.
    #[serde_as(deserialize_as = "DefaultOnError<Option<VecSkipError<_, SkipListener>>>")]
    #[serde(default)]
    pub locations: Option<Vec<ToolCallLocation>>,
    /// Update the raw input.
    pub raw_input: Option<serde_json::Value>,
    /// Update the raw output.
    pub raw_output: Option<serde_json::Value>,
}

impl ToolCallUpdateFields {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the tool kind.
    #[must_use]
    pub fn kind(mut self, kind: impl IntoOption<ToolKind>) -> Self {
        self.kind = kind.into_option();
        self
    }

    /// Update the execution status.
    #[must_use]
    pub fn status(mut self, status: impl IntoOption<ToolCallStatus>) -> Self {
        self.status = status.into_option();
        self
    }

    /// Update the human-readable title.
    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
        self
    }

    /// Replace the content collection.
    #[must_use]
    pub fn content(mut self, content: impl IntoOption<Vec<ToolCallContent>>) -> Self {
        self.content = content.into_option();
        self
    }

    /// Replace the locations collection.
    #[must_use]
    pub fn locations(mut self, locations: impl IntoOption<Vec<ToolCallLocation>>) -> Self {
        self.locations = locations.into_option();
        self
    }

    /// Update the raw input.
    #[must_use]
    pub fn raw_input(mut self, raw_input: impl IntoOption<serde_json::Value>) -> Self {
        self.raw_input = raw_input.into_option();
        self
    }

    /// Update the raw output.
    #[must_use]
    pub fn raw_output(mut self, raw_output: impl IntoOption<serde_json::Value>) -> Self {
        self.raw_output = raw_output.into_option();
        self
    }
}

/// If a given tool call doesn't exist yet, allows for attempting to construct
/// one from a tool call update if possible.
impl TryFrom<ToolCallUpdate> for ToolCall {
    type Error = Error;

    fn try_from(update: ToolCallUpdate) -> Result<Self, Self::Error> {
        let ToolCallUpdate {
            tool_call_id,
            fields:
                ToolCallUpdateFields {
                    kind,
                    status,
                    title,
                    content,
                    locations,
                    raw_input,
                    raw_output,
                },
            meta,
        } = update;

        Ok(Self {
            tool_call_id,
            title: title.ok_or_else(|| {
                Error::invalid_params().data(serde_json::json!("title is required for a tool call"))
            })?,
            kind: kind.unwrap_or_default(),
            status: status.unwrap_or_default(),
            content: content.unwrap_or_default(),
            locations: locations.unwrap_or_default(),
            raw_input,
            raw_output,
            meta,
        })
    }
}

impl From<ToolCall> for ToolCallUpdate {
    fn from(value: ToolCall) -> Self {
        let ToolCall {
            tool_call_id,
            title,
            kind,
            status,
            content,
            locations,
            raw_input,
            raw_output,
            meta,
        } = value;
        Self {
            tool_call_id,
            fields: ToolCallUpdateFields {
                kind: Some(kind),
                status: Some(status),
                title: Some(title),
                content: Some(content),
                locations: Some(locations),
                raw_input,
                raw_output,
            },
            meta,
        }
    }
}

/// Unique identifier for a tool call within a session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct ToolCallId(pub Arc<str>);

impl ToolCallId {
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

impl IntoOption<ToolCallId> for &str {
    fn into_option(self) -> Option<ToolCallId> {
        Some(ToolCallId::new(self))
    }
}

/// Categories of tools that can be invoked.
///
/// Tool kinds help clients choose appropriate icons and optimize how they
/// display tool execution progress.
///
/// See protocol docs: [Creating](https://agentclientprotocol.com/protocol/tool-calls#creating)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ToolKind {
    /// Reading files or data.
    Read,
    /// Modifying files or content.
    Edit,
    /// Removing files or data.
    Delete,
    /// Moving or renaming files.
    Move,
    /// Searching for information.
    Search,
    /// Running commands or code.
    Execute,
    /// Internal reasoning or planning.
    Think,
    /// Retrieving external data.
    Fetch,
    /// Switching the current session mode.
    SwitchMode,
    /// Other tool types (default).
    #[default]
    #[serde(other)]
    Other,
}

impl ToolKind {
    #[expect(clippy::trivially_copy_pass_by_ref, reason = "Required by serde")]
    fn is_default(&self) -> bool {
        matches!(self, ToolKind::Other)
    }
}

/// Execution status of a tool call.
///
/// Tool calls progress through different statuses during their lifecycle.
///
/// See protocol docs: [Status](https://agentclientprotocol.com/protocol/tool-calls#status)
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ToolCallStatus {
    /// The tool call hasn't started running yet because the input is either
    /// streaming or we're awaiting approval.
    #[default]
    Pending,
    /// The tool call is currently running.
    InProgress,
    /// The tool call completed successfully.
    Completed,
    /// The tool call failed with an error.
    Failed,
}

impl ToolCallStatus {
    #[expect(clippy::trivially_copy_pass_by_ref, reason = "Required by serde")]
    fn is_default(&self) -> bool {
        matches!(self, ToolCallStatus::Pending)
    }
}

/// Content produced by a tool call.
///
/// Tool calls can produce different types of content including
/// standard content blocks (text, images) or file diffs.
///
/// See protocol docs: [Content](https://agentclientprotocol.com/protocol/tool-calls#content)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(extend("discriminator" = {"propertyName": "type"}))]
#[non_exhaustive]
pub enum ToolCallContent {
    /// Standard content block (text, images, resources).
    Content(Content),
    /// File modification shown as a diff.
    Diff(Diff),
    /// Embed a terminal created with `terminal/create` by its id.
    ///
    /// The terminal must be added before calling `terminal/release`.
    ///
    /// See protocol docs: [Terminal](https://agentclientprotocol.com/protocol/terminals)
    Terminal(Terminal),
}

impl<T: Into<ContentBlock>> From<T> for ToolCallContent {
    fn from(content: T) -> Self {
        ToolCallContent::Content(Content::new(content))
    }
}

impl From<Diff> for ToolCallContent {
    fn from(diff: Diff) -> Self {
        ToolCallContent::Diff(diff)
    }
}

/// Standard content block (text, images, resources).
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Content {
    /// The actual content block.
    pub content: ContentBlock,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Content {
    #[must_use]
    pub fn new(content: impl Into<ContentBlock>) -> Self {
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

/// Embed a terminal created with `terminal/create` by its id.
///
/// The terminal must be added before calling `terminal/release`.
///
/// See protocol docs: [Terminal](https://agentclientprotocol.com/protocol/terminals)
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Terminal {
    pub terminal_id: TerminalId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Terminal {
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

/// A diff representing file modifications.
///
/// Shows changes to files in a format suitable for display in the client UI.
///
/// See protocol docs: [Content](https://agentclientprotocol.com/protocol/tool-calls#content)
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Diff {
    /// The file path being modified.
    pub path: PathBuf,
    /// The original content (None for new files).
    pub old_text: Option<String>,
    /// The new content after modification.
    pub new_text: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Diff {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, new_text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            old_text: None,
            new_text: new_text.into(),
            meta: None,
        }
    }

    /// The original content (None for new files).
    #[must_use]
    pub fn old_text(mut self, old_text: impl IntoOption<String>) -> Self {
        self.old_text = old_text.into_option();
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

/// A file location being accessed or modified by a tool.
///
/// Enables clients to implement "follow-along" features that track
/// which files the agent is working with in real-time.
///
/// See protocol docs: [Following the Agent](https://agentclientprotocol.com/protocol/tool-calls#following-the-agent)
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolCallLocation {
    /// The file path being accessed or modified.
    pub path: PathBuf,
    /// Optional line number within the file.
    #[serde(default)]
    pub line: Option<u32>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ToolCallLocation {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            line: None,
            meta: None,
        }
    }

    /// Optional line number within the file.
    #[must_use]
    pub fn line(mut self, line: impl IntoOption<u32>) -> Self {
        self.line = line.into_option();
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

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::{ContentBlock, ErrorCode, TextContent};
    use serde_json::json;

    fn sample_meta() -> Meta {
        let mut map = serde_json::Map::new();
        map.insert("k".to_string(), json!("v"));
        map
    }

    fn full_tool_call() -> ToolCall {
        ToolCall::new("tc_1", "edit file")
            .kind(ToolKind::Edit)
            .status(ToolCallStatus::InProgress)
            .content(vec![ToolCallContent::Diff(
                Diff::new("/tmp/a.rs", "new").old_text("old"),
            )])
            .locations(vec![ToolCallLocation::new("/tmp/a.rs").line(7)])
            .raw_input(json!({"path": "/tmp/a.rs"}))
            .raw_output(json!({"ok": true}))
    }

    // ----- ToolCall::update behaviour -----

    #[test]
    fn update_overwrites_only_provided_fields() {
        let mut call = ToolCall::new("call-1", "original title")
            .kind(ToolKind::Read)
            .status(ToolCallStatus::Pending)
            .content(vec![ToolCallContent::from("first")])
            .locations(vec![ToolCallLocation::new("/a")])
            .raw_input(json!({"a": 1}))
            .raw_output(json!({"b": 2}))
            .meta(sample_meta());

        let original_meta = call.meta.clone();
        let original_id = call.tool_call_id.clone();

        let fields = ToolCallUpdateFields::new().status(ToolCallStatus::InProgress);
        call.update(fields);

        // Status changed; everything else left intact.
        assert_eq!(call.tool_call_id, original_id);
        assert_eq!(call.title, "original title");
        assert_eq!(call.kind, ToolKind::Read);
        assert_eq!(call.status, ToolCallStatus::InProgress);
        assert_eq!(call.content, vec![ToolCallContent::from("first")]);
        assert_eq!(call.locations, vec![ToolCallLocation::new("/a")]);
        assert_eq!(call.raw_input, Some(json!({"a": 1})));
        assert_eq!(call.raw_output, Some(json!({"b": 2})));
        assert_eq!(call.meta, original_meta);
    }

    #[test]
    fn update_replaces_collections_rather_than_extending() {
        let mut call = ToolCall::new("call-1", "title")
            .content(vec![
                ToolCallContent::from("first"),
                ToolCallContent::from("second"),
            ])
            .locations(vec![
                ToolCallLocation::new("/a"),
                ToolCallLocation::new("/b"),
            ]);

        // Empty collections in the update completely replace the existing ones.
        let fields = ToolCallUpdateFields::new()
            .content(Vec::<ToolCallContent>::new())
            .locations(Vec::<ToolCallLocation>::new());
        call.update(fields);
        assert!(call.content.is_empty());
        assert!(call.locations.is_empty());

        // Non-empty replacement also overwrites, doesn't append.
        let fields = ToolCallUpdateFields::new()
            .content(vec![ToolCallContent::from("only")])
            .locations(vec![ToolCallLocation::new("/c")]);
        call.update(fields);
        assert_eq!(call.content, vec![ToolCallContent::from("only")]);
        assert_eq!(call.locations, vec![ToolCallLocation::new("/c")]);
    }

    #[test]
    fn update_does_not_clear_raw_input_or_output() {
        // ToolCall::update sets raw_input/raw_output only when the update has
        // `Some` value — `None` does not clear an existing value. This is the
        // contract callers rely on for incremental updates.
        let mut call = ToolCall::new("call-1", "title")
            .raw_input(json!({"a": 1}))
            .raw_output(json!({"b": 2}));

        let fields = ToolCallUpdateFields::new().title("new title");
        call.update(fields);

        assert_eq!(call.title, "new title");
        assert_eq!(call.raw_input, Some(json!({"a": 1})));
        assert_eq!(call.raw_output, Some(json!({"b": 2})));
    }

    #[test]
    fn update_does_not_overwrite_meta() {
        // The update method has no branch for `meta`, so it must stay intact
        // even when other fields change.
        let mut call = ToolCall::new("call-1", "title").meta(sample_meta());
        let fields = ToolCallUpdateFields::new()
            .title("new")
            .kind(ToolKind::Edit);
        call.update(fields);
        assert_eq!(call.title, "new");
        assert_eq!(call.kind, ToolKind::Edit);
        assert_eq!(call.meta, Some(sample_meta()));
    }

    // ----- TryFrom<ToolCallUpdate> for ToolCall -----

    #[test]
    fn try_from_update_requires_title() {
        let update = ToolCallUpdate::new(
            "call-1",
            ToolCallUpdateFields::new().status(ToolCallStatus::InProgress),
        );
        let err = ToolCall::try_from(update).unwrap_err();
        // Error must be an InvalidParams JSON-RPC error to surface the missing
        // field correctly to callers.
        assert_eq!(i32::from(err.code), -32602);
        assert!(
            err.data
                .as_ref()
                .and_then(|d| d.as_str())
                .is_some_and(|s| s.contains("title"))
        );
    }

    #[test]
    fn try_from_update_applies_defaults_for_optional_fields() {
        let update = ToolCallUpdate::new("call-1", ToolCallUpdateFields::new().title("my title"));
        let call = ToolCall::try_from(update).unwrap();
        assert_eq!(call.tool_call_id, ToolCallId::new("call-1"));
        assert_eq!(call.title, "my title");
        assert_eq!(call.kind, ToolKind::default());
        assert_eq!(call.status, ToolCallStatus::default());
        assert!(call.content.is_empty());
        assert!(call.locations.is_empty());
        assert!(call.raw_input.is_none());
        assert!(call.raw_output.is_none());
        assert!(call.meta.is_none());
    }

    #[test]
    fn try_from_update_propagates_all_fields() {
        let update = ToolCallUpdate::new(
            "call-1",
            ToolCallUpdateFields::new()
                .title("t")
                .kind(ToolKind::Edit)
                .status(ToolCallStatus::Completed)
                .content(vec![ToolCallContent::from("hello")])
                .locations(vec![ToolCallLocation::new("/p")])
                .raw_input(json!({"x": 1}))
                .raw_output(json!({"y": 2})),
        )
        .meta(sample_meta());

        let call = ToolCall::try_from(update).unwrap();
        assert_eq!(call.title, "t");
        assert_eq!(call.kind, ToolKind::Edit);
        assert_eq!(call.status, ToolCallStatus::Completed);
        assert_eq!(call.content, vec![ToolCallContent::from("hello")]);
        assert_eq!(call.locations, vec![ToolCallLocation::new("/p")]);
        assert_eq!(call.raw_input, Some(json!({"x": 1})));
        assert_eq!(call.raw_output, Some(json!({"y": 2})));
        assert_eq!(call.meta, Some(sample_meta()));
    }

    // ----- From<ToolCall> for ToolCallUpdate (lossless round-trip) -----

    #[test]
    fn round_trip_tool_call_through_update_is_lossless() {
        let call = ToolCall::new("call-1", "t")
            .kind(ToolKind::Execute)
            .status(ToolCallStatus::InProgress)
            .content(vec![ToolCallContent::from("hi")])
            .locations(vec![ToolCallLocation::new("/p").line(7u32)])
            .raw_input(json!({"a": 1}))
            .raw_output(json!({"b": 2}))
            .meta(sample_meta());

        let update: ToolCallUpdate = call.clone().into();
        let reconstructed = ToolCall::try_from(update).unwrap();
        assert_eq!(reconstructed, call);
    }

    // ----- Serde defaults and unknown-variant tolerance -----

    #[test]
    fn tool_kind_is_default_only_for_other() {
        assert!(ToolKind::default().is_default());
        for kind in [
            ToolKind::Read,
            ToolKind::Edit,
            ToolKind::Delete,
            ToolKind::Move,
            ToolKind::Search,
            ToolKind::Execute,
            ToolKind::Think,
            ToolKind::Fetch,
            ToolKind::SwitchMode,
        ] {
            assert!(!kind.is_default(), "{kind:?} should not be default");
        }
    }

    #[test]
    fn tool_call_status_is_default_only_for_pending() {
        assert!(ToolCallStatus::default().is_default());
        for status in [
            ToolCallStatus::InProgress,
            ToolCallStatus::Completed,
            ToolCallStatus::Failed,
        ] {
            assert!(!status.is_default(), "{status:?} should not be default");
        }
    }

    #[test]
    fn tool_call_serialization_omits_default_kind_and_status() {
        let call = ToolCall::new("c", "t");
        let value = serde_json::to_value(&call).unwrap();
        let object = value.as_object().unwrap();
        assert!(!object.contains_key("kind"), "default kind must be skipped");
        assert!(
            !object.contains_key("status"),
            "default status must be skipped"
        );
        assert!(
            !object.contains_key("content"),
            "empty content must be skipped"
        );
        assert!(
            !object.contains_key("locations"),
            "empty locations must be skipped"
        );
    }

    #[test]
    fn tool_call_serialization_includes_non_default_kind_and_status() {
        let call = ToolCall::new("c", "t")
            .kind(ToolKind::Read)
            .status(ToolCallStatus::Completed);
        let value = serde_json::to_value(&call).unwrap();
        assert_eq!(value["kind"], json!("read"));
        assert_eq!(value["status"], json!("completed"));
    }

    #[test]
    fn tool_kind_deserializes_known_variants_in_snake_case() {
        for (text, expected) in [
            ("read", ToolKind::Read),
            ("edit", ToolKind::Edit),
            ("delete", ToolKind::Delete),
            ("move", ToolKind::Move),
            ("search", ToolKind::Search),
            ("execute", ToolKind::Execute),
            ("think", ToolKind::Think),
            ("fetch", ToolKind::Fetch),
            ("switch_mode", ToolKind::SwitchMode),
            ("other", ToolKind::Other),
        ] {
            let parsed: ToolKind = serde_json::from_value(json!(text)).unwrap();
            assert_eq!(parsed, expected, "failed to parse {text:?}");
        }
    }

    #[test]
    fn tool_kind_falls_back_to_other_for_unknown_variant() {
        // Forward compatibility: agents may send tool kinds this version does
        // not know about; they must not break deserialization.
        let parsed: ToolKind = serde_json::from_value(json!("does_not_exist")).unwrap();
        assert_eq!(parsed, ToolKind::Other);
    }

    #[test]
    fn tool_call_skips_malformed_content_entries() {
        // ToolCall uses `DefaultOnError<VecSkipError<_, SkipListener>>` so that
        // bad entries on either of these collections do not poison the whole
        // tool call. This protects clients from agent-side malformations.
        let value = json!({
            "toolCallId": "tc-1",
            "title": "t",
            "content": [
                {"type": "content", "content": {"type": "text", "text": "ok"}},
                {"type": "bogus"},
                {"type": "diff", "path": "/f", "newText": "hi"}
            ],
            "locations": [
                {"path": "/a"},
                "not-an-object",
                {"path": "/b", "line": 3}
            ]
        });
        let call: ToolCall = serde_json::from_value(value).unwrap();
        assert_eq!(call.content.len(), 2);
        assert_eq!(call.locations.len(), 2);
        assert_eq!(call.locations[0].path, PathBuf::from("/a"));
        assert_eq!(call.locations[1].path, PathBuf::from("/b"));
        assert_eq!(call.locations[1].line, Some(3));
    }

    #[test]
    fn tool_call_collapses_wrong_shaped_collection_to_default() {
        // Whole-collection shape errors collapse to empty via DefaultOnError.
        let value = json!({
            "toolCallId": "tc-1",
            "title": "t",
            "content": "not-an-array",
            "locations": null
        });
        let call: ToolCall = serde_json::from_value(value).unwrap();
        assert!(call.content.is_empty());
        assert!(call.locations.is_empty());
    }

    // ----- ToolCallContent conversions -----

    #[test]
    fn tool_call_content_from_string_yields_text_content_block() {
        let content: ToolCallContent = "hi".into();
        match content {
            ToolCallContent::Content(Content {
                content: ContentBlock::Text(t),
                ..
            }) => {
                assert_eq!(t.text, "hi");
            }
            other => panic!("expected text content, got {other:?}"),
        }
    }

    #[test]
    fn tool_call_content_from_diff_preserves_diff() {
        let diff = Diff::new("/p", "new text").old_text("old text".to_string());
        let content: ToolCallContent = diff.clone().into();
        match content {
            ToolCallContent::Diff(d) => assert_eq!(d, diff),
            other => panic!("expected diff, got {other:?}"),
        }
    }

    // ----- ToolCallId From impls -----

    #[test]
    fn tool_call_id_from_various_types() {
        let from_static: ToolCallId = "abc".into();
        let from_string: ToolCallId = String::from("abc").into();
        let from_arc: ToolCallId = Arc::<str>::from("abc").into();
        let from_new = ToolCallId::new("abc");
        assert_eq!(from_static, from_new);
        assert_eq!(from_string, from_new);
        assert_eq!(from_arc, from_new);
        // Display is the inner string.
        assert_eq!(from_new.to_string(), "abc");
    }

    // ----- Tests from origin/main -----

    #[test]
    fn tool_call_default_kind_and_status_are_omitted() {
        let value = ToolCall::new("tc", "title");
        let serialized = serde_json::to_value(&value).unwrap();
        let obj = serialized.as_object().unwrap();
        assert!(
            !obj.contains_key("kind"),
            "kind should be skipped: {serialized}"
        );
        assert!(
            !obj.contains_key("status"),
            "status should be skipped: {serialized}"
        );
        assert!(!obj.contains_key("content"));
        assert!(!obj.contains_key("locations"));
        assert!(!obj.contains_key("rawInput"));
        assert!(!obj.contains_key("rawOutput"));
        assert!(!obj.contains_key("_meta"));
    }

    #[test]
    fn tool_kind_unknown_variants_decode_to_other() {
        let kind: ToolKind = serde_json::from_str("\"future_kind\"").unwrap();
        assert_eq!(kind, ToolKind::Other);
        assert_eq!(
            serde_json::to_value(ToolKind::Other).unwrap(),
            json!("other")
        );
    }

    #[test]
    fn tool_call_status_pending_is_default() {
        assert_eq!(ToolCallStatus::default(), ToolCallStatus::Pending);
        assert_eq!(
            serde_json::to_value(ToolCallStatus::InProgress).unwrap(),
            json!("in_progress")
        );
        let parsed: ToolCallStatus = serde_json::from_str("\"completed\"").unwrap();
        assert_eq!(parsed, ToolCallStatus::Completed);
    }

    #[test]
    fn tool_call_roundtrip_preserves_all_fields() {
        let value = full_tool_call();
        let serialized = serde_json::to_value(&value).unwrap();
        let parsed: ToolCall = serde_json::from_value(serialized).unwrap();
        assert_eq!(parsed, value);
    }

    #[test]
    fn tool_call_update_only_overwrites_present_fields() {
        let mut call = full_tool_call();
        let fields = ToolCallUpdateFields::new().status(ToolCallStatus::Completed);
        call.update(fields);

        assert_eq!(call.status, ToolCallStatus::Completed);
        assert_eq!(call.title, "edit file");
        assert_eq!(call.kind, ToolKind::Edit);
        assert_eq!(call.content.len(), 1);
        assert_eq!(call.locations.len(), 1);
        assert_eq!(call.raw_input, Some(json!({"path": "/tmp/a.rs"})));
    }

    #[test]
    fn tool_call_update_replaces_collections_not_extends() {
        let mut call = full_tool_call();
        let fields = ToolCallUpdateFields::new()
            .content(vec![ToolCallContent::Content(Content::new(
                ContentBlock::Text(TextContent::new("only new")),
            ))])
            .locations(vec![ToolCallLocation::new("/other")]);
        call.update(fields);

        assert_eq!(call.content.len(), 1);
        match &call.content[0] {
            ToolCallContent::Content(c) => match &c.content {
                ContentBlock::Text(t) => assert_eq!(t.text, "only new"),
                _ => panic!("expected text content"),
            },
            _ => panic!("expected Content variant after replacement"),
        }
        assert_eq!(call.locations.len(), 1);
        assert_eq!(call.locations[0].path, std::path::PathBuf::from("/other"));
    }

    #[test]
    fn tool_call_try_from_update_requires_title() {
        let update = ToolCallUpdate::new("tc_1", ToolCallUpdateFields::new());
        let err = ToolCall::try_from(update).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidParams);
        assert_eq!(
            err.data.as_ref().and_then(|v| v.as_str()),
            Some("title is required for a tool call"),
            "title-required error should carry a stable, actionable message",
        );
    }

    #[test]
    fn tool_call_try_from_update_fills_defaults_for_missing_optionals() {
        let update = ToolCallUpdate::new(
            "tc_1",
            ToolCallUpdateFields::new()
                .title("hello")
                .raw_input(json!({"x": 1})),
        );

        let call = ToolCall::try_from(update).unwrap();
        assert_eq!(call.tool_call_id, ToolCallId::new("tc_1"));
        assert_eq!(call.title, "hello");
        assert_eq!(call.kind, ToolKind::default());
        assert_eq!(call.status, ToolCallStatus::default());
        assert!(call.content.is_empty());
        assert!(call.locations.is_empty());
        assert_eq!(call.raw_input, Some(json!({"x": 1})));
        assert_eq!(call.raw_output, None);
    }

    #[test]
    fn tool_call_to_update_preserves_every_field() {
        let call = full_tool_call();
        let update: ToolCallUpdate = call.clone().into();
        assert_eq!(update.tool_call_id, call.tool_call_id);
        assert_eq!(update.fields.title.as_deref(), Some("edit file"));
        assert_eq!(update.fields.kind, Some(ToolKind::Edit));
        assert_eq!(update.fields.status, Some(ToolCallStatus::InProgress));
        assert_eq!(update.fields.content.as_ref().map(Vec::len), Some(1));
        assert_eq!(update.fields.locations.as_ref().map(Vec::len), Some(1));
        assert_eq!(update.fields.raw_input, Some(json!({"path": "/tmp/a.rs"})));
        assert_eq!(update.fields.raw_output, Some(json!({"ok": true})));

        let rebuilt = ToolCall::try_from(update).unwrap();
        assert_eq!(rebuilt, call);
    }

    #[test]
    fn tool_call_update_flattens_fields_on_wire() {
        let update = ToolCallUpdate::new(
            "tc_1",
            ToolCallUpdateFields::new()
                .status(ToolCallStatus::Completed)
                .title("done"),
        );
        let serialized = serde_json::to_value(&update).unwrap();
        assert_eq!(serialized["toolCallId"], json!("tc_1"));
        assert_eq!(serialized["status"], json!("completed"));
        assert_eq!(serialized["title"], json!("done"));
        assert!(
            serialized.get("fields").is_none(),
            "fields should be flattened"
        );
    }

    #[test]
    fn tool_call_content_discriminator_is_type_snake_case() {
        let diff = ToolCallContent::Diff(Diff::new("/p", "new"));
        let json = serde_json::to_value(&diff).unwrap();
        assert_eq!(json["type"], json!("diff"));

        let parsed: ToolCallContent = serde_json::from_value(json!({
            "type": "content",
            "content": {"type": "text", "text": "hi"}
        }))
        .unwrap();
        match parsed {
            ToolCallContent::Content(c) => match c.content {
                ContentBlock::Text(t) => assert_eq!(t.text, "hi"),
                _ => panic!("expected text"),
            },
            _ => panic!("expected Content variant"),
        }

        let parsed: ToolCallContent = serde_json::from_value(json!({
            "type": "terminal",
            "terminalId": "term_1",
        }))
        .unwrap();
        match parsed {
            ToolCallContent::Terminal(t) => assert_eq!(t.terminal_id.0.as_ref(), "term_1"),
            _ => panic!("expected Terminal variant"),
        }
    }

    #[test]
    fn tool_call_skips_malformed_content_and_locations() {
        let input = json!({
            "toolCallId": "tc_1",
            "title": "hi",
            "content": [
                {"type": "content", "content": {"type": "text", "text": "ok"}},
                {"type": "diff", "missingPath": true},
                "not even an object",
                {"type": "totally_unknown_kind"},
                {"type": "diff", "path": "/p", "newText": "n"},
            ],
            "locations": [
                {"path": "/ok"},
                {"path": 42},
                "nope",
            ]
        });

        let call: ToolCall = serde_json::from_value(input).unwrap();
        assert_eq!(call.content.len(), 2);
        match &call.content[0] {
            ToolCallContent::Content(_) => {}
            _ => panic!("expected text content first"),
        }
        match &call.content[1] {
            ToolCallContent::Diff(d) => assert_eq!(d.new_text, "n"),
            _ => panic!("expected diff second"),
        }
        assert_eq!(call.locations.len(), 1);
        assert_eq!(call.locations[0].path, std::path::PathBuf::from("/ok"));
    }

    #[test]
    fn tool_call_treats_outer_shape_errors_as_empty_collections() {
        let input = json!({
            "toolCallId": "tc_1",
            "title": "hi",
            "content": "oops",
            "locations": {"k": 1}
        });
        let call: ToolCall = serde_json::from_value(input).unwrap();
        assert!(call.content.is_empty());
        assert!(call.locations.is_empty());
    }

    #[test]
    fn tool_call_update_fields_tolerate_unknown_kind_and_status() {
        let fields: ToolCallUpdateFields = serde_json::from_value(json!({
            "kind": "totally_new_kind",
            "status": "totally_new_status",
        }))
        .unwrap();
        assert_eq!(fields.kind, Some(ToolKind::Other));
        assert_eq!(fields.status, None);
    }

    #[test]
    fn diff_new_leaves_old_text_unset() {
        let d = Diff::new("/p", "new");
        assert_eq!(d.old_text, None);
        let serialized = serde_json::to_value(&d).unwrap();
        assert!(serialized.as_object().unwrap().get("oldText").is_none());
    }

    #[test]
    fn tool_call_content_from_content_block_wraps_in_content_variant() {
        let block: ContentBlock = ContentBlock::Text(TextContent::new("hi"));
        let tcc: ToolCallContent = block.into();
        match tcc {
            ToolCallContent::Content(_) => {}
            _ => panic!("expected Content variant from blanket From impl"),
        }

        let from_diff: ToolCallContent = Diff::new("/p", "n").into();
        match from_diff {
            ToolCallContent::Diff(_) => {}
            _ => panic!("expected Diff variant from From<Diff>"),
        }
    }
}
