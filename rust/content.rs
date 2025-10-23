//! Content blocks for representing various types of information in the Agent Client Protocol.
//!
//! This module defines the core content types used throughout the protocol for communication
//! between agents and clients. Content blocks provide a flexible, extensible way to represent
//! text, images, audio, and other resources in prompts, responses, and tool call results.
//!
//! The content block structure is designed to be compatible with the Model Context Protocol (MCP),
//! allowing seamless integration between ACP and MCP-based tools.
//!
//! See: [Content](https://agentclientprotocol.com/protocol/content)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Content blocks represent displayable information in the Agent Client Protocol.
///
/// They provide a structured way to handle various types of user-facing content—whether
/// it's text from language models, images for analysis, or embedded resources for context.
///
/// Content blocks appear in:
/// - User prompts sent via `session/prompt`
/// - Language model output streamed through `session/update` notifications
/// - Progress updates and results from tool calls
///
/// This structure is compatible with the Model Context Protocol (MCP), enabling
/// agents to seamlessly forward content from MCP tool outputs without transformation.
///
/// See protocol docs: [Content](https://agentclientprotocol.com/protocol/content)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content
    ///
    /// All agents MUST support text content blocks in prompts.
    Text(TextContent),
    /// Images for visual context or analysis.
    ///
    /// Requires the `image` prompt capability when included in prompts.
    Image(ImageContent),
    /// Audio data for transcription or analysis.
    ///
    /// Requires the `audio` prompt capability when included in prompts.
    Audio(AudioContent),
    /// References to resources that the agent can access.
    ///
    /// All agents MUST support resource links in prompts.
    ResourceLink(ResourceLink),
    /// Complete resource contents embedded directly in the message.
    ///
    /// Preferred for including context as it avoids extra round-trips.
    ///
    /// Requires the `embeddedContext` prompt capability when included in prompts.
    Resource(EmbeddedResource),
}

/// Text provided to or from an LLM.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[schemars(inline)]
pub struct TextContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    pub text: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl<T: Into<String>> From<T> for ContentBlock {
    fn from(value: T) -> Self {
        Self::Text(TextContent {
            annotations: None,
            text: value.into(),
            meta: None,
        })
    }
}

/// An image provided to or from an LLM.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[schemars(inline)]
pub struct ImageContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    pub data: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Audio provided to or from an LLM.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[schemars(inline)]
pub struct AudioContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    pub data: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// The contents of a resource, embedded into a prompt or tool call result.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[schemars(inline)]
pub struct EmbeddedResource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    pub resource: EmbeddedResourceResource,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Resource content that can be embedded in a message.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum EmbeddedResourceResource {
    TextResourceContents(TextResourceContents),
    BlobResourceContents(BlobResourceContents),
}

/// Text-based resource contents.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct TextResourceContents {
    #[serde(rename = "mimeType", default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub text: String,
    pub uri: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Binary resource contents.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct BlobResourceContents {
    pub blob: String,
    #[serde(rename = "mimeType", default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub uri: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// A resource that the server is capable of reading, included in a prompt or tool call result.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[schemars(inline)]
pub struct ResourceLink {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub uri: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Optional annotations for the client. The client can use annotations to inform how objects are used or displayed
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Annotations {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<Role>>,
    #[serde(
        rename = "lastModified",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_modified: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// The sender or recipient of messages and data in a conversation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
pub enum Role {
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "user")]
    User,
}
