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
#[schemars(extend("discriminator" = {"propertyName": "type"}))]
#[non_exhaustive]
pub enum ContentBlock {
    /// Text content. May be plain text or formatted with Markdown.
    ///
    /// All agents MUST support text content blocks in prompts.
    /// Clients SHOULD render this text as Markdown.
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
#[non_exhaustive]
pub struct TextContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    pub text: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl TextContent {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            annotations: None,
            text: text.into(),
            meta: None,
        }
    }

    pub fn annotations(mut self, annotations: Annotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

impl<T: Into<String>> From<T> for ContentBlock {
    fn from(value: T) -> Self {
        Self::Text(TextContent::new(value))
    }
}

/// An image provided to or from an LLM.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImageContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    pub data: String,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl ImageContent {
    pub fn new(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            annotations: None,
            data: data.into(),
            mime_type: mime_type.into(),
            uri: None,
            meta: None,
        }
    }

    pub fn annotations(mut self, annotations: Annotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    pub fn uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Audio provided to or from an LLM.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AudioContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    pub data: String,
    pub mime_type: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl AudioContent {
    pub fn new(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            annotations: None,
            data: data.into(),
            mime_type: mime_type.into(),
            meta: None,
        }
    }

    pub fn annotations(mut self, annotations: Annotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// The contents of a resource, embedded into a prompt or tool call result.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[non_exhaustive]
pub struct EmbeddedResource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    pub resource: EmbeddedResourceResource,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl EmbeddedResource {
    pub fn new(resource: EmbeddedResourceResource) -> Self {
        Self {
            annotations: None,
            resource,
            meta: None,
        }
    }

    pub fn annotations(mut self, annotations: Annotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Resource content that can be embedded in a message.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
#[non_exhaustive]
pub enum EmbeddedResourceResource {
    TextResourceContents(TextResourceContents),
    BlobResourceContents(BlobResourceContents),
}

/// Text-based resource contents.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TextResourceContents {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub text: String,
    pub uri: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl TextResourceContents {
    pub fn new(text: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            mime_type: None,
            text: text.into(),
            uri: uri.into(),
            meta: None,
        }
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Binary resource contents.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BlobResourceContents {
    pub blob: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub uri: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl BlobResourceContents {
    pub fn new(blob: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            blob: blob.into(),
            mime_type: None,
            uri: uri.into(),
            meta: None,
        }
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// A resource that the server is capable of reading, included in a prompt or tool call result.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResourceLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub uri: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl ResourceLink {
    pub fn new(name: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            annotations: None,
            description: None,
            mime_type: None,
            name: name.into(),
            size: None,
            title: None,
            uri: uri.into(),
            meta: None,
        }
    }

    pub fn annotations(mut self, annotations: Annotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn size(mut self, size: i64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Optional annotations for the client. The client can use annotations to inform how objects are used or displayed
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Annotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<Role>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl Annotations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn audience(mut self, audience: Vec<Role>) -> Self {
        self.audience = Some(audience);
        self
    }

    pub fn last_modified(mut self, last_modified: impl Into<String>) -> Self {
        self.last_modified = Some(last_modified.into());
        self
    }

    pub fn priority(mut self, priority: f64) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Extension point for implementations
    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// The sender or recipient of messages and data in a conversation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Role {
    Assistant,
    User,
}
