//! Elicitation types for structured user input.
//!
//! **UNSTABLE**: This module is not part of the spec yet, and may be removed or changed at any point.
//!
//! This module defines the types used for agent-initiated elicitation,
//! where the agent requests structured input from the user via forms or URLs.

use std::{collections::BTreeMap, sync::Arc};

use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, serde_as, skip_serializing_none};

use crate::client::{ELICITATION_COMPLETE_NOTIFICATION, ELICITATION_CREATE_METHOD_NAME};
use crate::tool_call::ToolCallId;
use crate::{IntoOption, Meta, RequestId, SessionId};

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Unique identifier for an elicitation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct ElicitationId(pub Arc<str>);

impl ElicitationId {
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// String format types for string properties in elicitation schemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum StringFormat {
    /// Email address format.
    Email,
    /// URI format.
    Uri,
    /// Date format (YYYY-MM-DD).
    Date,
    /// Date-time format (ISO 8601).
    DateTime,
}

/// Type discriminator for elicitation schemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ElicitationSchemaType {
    /// Object schema type.
    #[default]
    Object,
}

/// A titled enum option with a const value and human-readable title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct EnumOption {
    /// The constant value for this option.
    #[serde(rename = "const")]
    pub value: String,
    /// Human-readable title for this option.
    pub title: String,
}

impl EnumOption {
    /// Create a new enum option.
    #[must_use]
    pub fn new(value: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            title: title.into(),
        }
    }
}

/// Schema for string properties in an elicitation form.
///
/// When `enum` or `oneOf` is set, this represents a single-select enum
/// with `"type": "string"`.
#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StringPropertySchema {
    /// Optional title for the property.
    pub title: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Minimum string length.
    pub min_length: Option<u32>,
    /// Maximum string length.
    pub max_length: Option<u32>,
    /// Pattern the string must match.
    pub pattern: Option<String>,
    /// String format.
    pub format: Option<StringFormat>,
    /// Default value.
    pub default: Option<String>,
    /// Enum values for untitled single-select enums.
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    /// Titled enum options for titled single-select enums.
    #[serde(rename = "oneOf")]
    pub one_of: Option<Vec<EnumOption>>,
}

impl StringPropertySchema {
    /// Create a new string property schema.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an email string property schema.
    #[must_use]
    pub fn email() -> Self {
        Self {
            format: Some(StringFormat::Email),
            ..Default::default()
        }
    }

    /// Create a URI string property schema.
    #[must_use]
    pub fn uri() -> Self {
        Self {
            format: Some(StringFormat::Uri),
            ..Default::default()
        }
    }

    /// Create a date string property schema.
    #[must_use]
    pub fn date() -> Self {
        Self {
            format: Some(StringFormat::Date),
            ..Default::default()
        }
    }

    /// Create a date-time string property schema.
    #[must_use]
    pub fn date_time() -> Self {
        Self {
            format: Some(StringFormat::DateTime),
            ..Default::default()
        }
    }

    /// Optional title for the property.
    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
        self
    }

    /// Human-readable description.
    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// Minimum string length.
    #[must_use]
    pub fn min_length(mut self, min_length: impl IntoOption<u32>) -> Self {
        self.min_length = min_length.into_option();
        self
    }

    /// Maximum string length.
    #[must_use]
    pub fn max_length(mut self, max_length: impl IntoOption<u32>) -> Self {
        self.max_length = max_length.into_option();
        self
    }

    /// Pattern the string must match.
    #[must_use]
    pub fn pattern(mut self, pattern: impl IntoOption<String>) -> Self {
        self.pattern = pattern.into_option();
        self
    }

    /// String format.
    #[must_use]
    pub fn format(mut self, format: impl IntoOption<StringFormat>) -> Self {
        self.format = format.into_option();
        self
    }

    /// Default value.
    #[must_use]
    pub fn default_value(mut self, default: impl IntoOption<String>) -> Self {
        self.default = default.into_option();
        self
    }

    /// Enum values for untitled single-select enums.
    #[must_use]
    pub fn enum_values(mut self, enum_values: impl IntoOption<Vec<String>>) -> Self {
        self.enum_values = enum_values.into_option();
        self
    }

    /// Titled enum options for titled single-select enums.
    #[must_use]
    pub fn one_of(mut self, one_of: impl IntoOption<Vec<EnumOption>>) -> Self {
        self.one_of = one_of.into_option();
        self
    }
}

/// Schema for number (floating-point) properties in an elicitation form.
#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NumberPropertySchema {
    /// Optional title for the property.
    pub title: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Minimum value (inclusive).
    pub minimum: Option<f64>,
    /// Maximum value (inclusive).
    pub maximum: Option<f64>,
    /// Default value.
    pub default: Option<f64>,
}

impl NumberPropertySchema {
    /// Create a new number property schema.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Optional title for the property.
    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
        self
    }

    /// Human-readable description.
    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// Minimum value (inclusive).
    #[must_use]
    pub fn minimum(mut self, minimum: impl IntoOption<f64>) -> Self {
        self.minimum = minimum.into_option();
        self
    }

    /// Maximum value (inclusive).
    #[must_use]
    pub fn maximum(mut self, maximum: impl IntoOption<f64>) -> Self {
        self.maximum = maximum.into_option();
        self
    }

    /// Default value.
    #[must_use]
    pub fn default_value(mut self, default: impl IntoOption<f64>) -> Self {
        self.default = default.into_option();
        self
    }
}

/// Schema for integer properties in an elicitation form.
#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct IntegerPropertySchema {
    /// Optional title for the property.
    pub title: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Minimum value (inclusive).
    pub minimum: Option<i64>,
    /// Maximum value (inclusive).
    pub maximum: Option<i64>,
    /// Default value.
    pub default: Option<i64>,
}

impl IntegerPropertySchema {
    /// Create a new integer property schema.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Optional title for the property.
    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
        self
    }

    /// Human-readable description.
    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// Minimum value (inclusive).
    #[must_use]
    pub fn minimum(mut self, minimum: impl IntoOption<i64>) -> Self {
        self.minimum = minimum.into_option();
        self
    }

    /// Maximum value (inclusive).
    #[must_use]
    pub fn maximum(mut self, maximum: impl IntoOption<i64>) -> Self {
        self.maximum = maximum.into_option();
        self
    }

    /// Default value.
    #[must_use]
    pub fn default_value(mut self, default: impl IntoOption<i64>) -> Self {
        self.default = default.into_option();
        self
    }
}

/// Schema for boolean properties in an elicitation form.
#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BooleanPropertySchema {
    /// Optional title for the property.
    pub title: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Default value.
    pub default: Option<bool>,
}

impl BooleanPropertySchema {
    /// Create a new boolean property schema.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Optional title for the property.
    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
        self
    }

    /// Human-readable description.
    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// Default value.
    #[must_use]
    pub fn default_value(mut self, default: impl IntoOption<bool>) -> Self {
        self.default = default.into_option();
        self
    }
}

/// Items definition for untitled multi-select enum properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ElicitationStringType {
    /// String schema type.
    #[default]
    String,
}

/// Items definition for untitled multi-select enum properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct UntitledMultiSelectItems {
    /// Item type discriminator. Must be `"string"`.
    #[serde(rename = "type")]
    pub type_: ElicitationStringType,
    /// Allowed enum values.
    #[serde(rename = "enum")]
    pub values: Vec<String>,
}

/// Items definition for titled multi-select enum properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct TitledMultiSelectItems {
    /// Titled enum options.
    #[serde(rename = "anyOf")]
    pub options: Vec<EnumOption>,
}

impl TitledMultiSelectItems {
    /// Create new titled multi-select items.
    #[must_use]
    pub fn new(options: Vec<EnumOption>) -> Self {
        Self { options }
    }
}

/// Items for a multi-select (array) property schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[non_exhaustive]
pub enum MultiSelectItems {
    /// Untitled multi-select items with plain string values.
    Untitled(UntitledMultiSelectItems),
    /// Titled multi-select items with human-readable labels.
    Titled(TitledMultiSelectItems),
}

/// Schema for multi-select (array) properties in an elicitation form.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MultiSelectPropertySchema {
    /// Optional title for the property.
    pub title: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Minimum number of items to select.
    pub min_items: Option<u64>,
    /// Maximum number of items to select.
    pub max_items: Option<u64>,
    /// The items definition describing allowed values.
    pub items: MultiSelectItems,
    /// Default selected values.
    pub default: Option<Vec<String>>,
}

impl MultiSelectPropertySchema {
    /// Create a new untitled multi-select property schema.
    #[must_use]
    pub fn new(values: Vec<String>) -> Self {
        Self {
            title: None,
            description: None,
            min_items: None,
            max_items: None,
            items: MultiSelectItems::Untitled(UntitledMultiSelectItems {
                type_: ElicitationStringType::String,
                values,
            }),
            default: None,
        }
    }

    /// Create a new titled multi-select property schema.
    #[must_use]
    pub fn titled(options: Vec<EnumOption>) -> Self {
        Self {
            title: None,
            description: None,
            min_items: None,
            max_items: None,
            items: MultiSelectItems::Titled(TitledMultiSelectItems { options }),
            default: None,
        }
    }

    /// Optional title for the property.
    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
        self
    }

    /// Human-readable description.
    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// Minimum number of items to select.
    #[must_use]
    pub fn min_items(mut self, min_items: impl IntoOption<u64>) -> Self {
        self.min_items = min_items.into_option();
        self
    }

    /// Maximum number of items to select.
    #[must_use]
    pub fn max_items(mut self, max_items: impl IntoOption<u64>) -> Self {
        self.max_items = max_items.into_option();
        self
    }

    /// Default selected values.
    #[must_use]
    pub fn default_value(mut self, default: impl IntoOption<Vec<String>>) -> Self {
        self.default = default.into_option();
        self
    }
}

/// Property schema for elicitation form fields.
///
/// Each variant corresponds to a JSON Schema `"type"` value.
/// Single-select enums use the `String` variant with `enum` or `oneOf` set.
/// Multi-select enums use the `Array` variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(extend("discriminator" = {"propertyName": "type"}))]
#[non_exhaustive]
pub enum ElicitationPropertySchema {
    /// String property (or single-select enum when `enum`/`oneOf` is set).
    String(StringPropertySchema),
    /// Number (floating-point) property.
    Number(NumberPropertySchema),
    /// Integer property.
    Integer(IntegerPropertySchema),
    /// Boolean property.
    Boolean(BooleanPropertySchema),
    /// Multi-select array property.
    Array(MultiSelectPropertySchema),
}

impl From<StringPropertySchema> for ElicitationPropertySchema {
    fn from(schema: StringPropertySchema) -> Self {
        Self::String(schema)
    }
}

impl From<NumberPropertySchema> for ElicitationPropertySchema {
    fn from(schema: NumberPropertySchema) -> Self {
        Self::Number(schema)
    }
}

impl From<IntegerPropertySchema> for ElicitationPropertySchema {
    fn from(schema: IntegerPropertySchema) -> Self {
        Self::Integer(schema)
    }
}

impl From<BooleanPropertySchema> for ElicitationPropertySchema {
    fn from(schema: BooleanPropertySchema) -> Self {
        Self::Boolean(schema)
    }
}

impl From<MultiSelectPropertySchema> for ElicitationPropertySchema {
    fn from(schema: MultiSelectPropertySchema) -> Self {
        Self::Array(schema)
    }
}

fn default_object_type() -> ElicitationSchemaType {
    ElicitationSchemaType::Object
}

/// Type-safe elicitation schema for requesting structured user input.
///
/// This represents a JSON Schema object with primitive-typed properties,
/// as required by the elicitation specification.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationSchema {
    /// Type discriminator. Always `"object"`.
    #[serde(rename = "type", default = "default_object_type")]
    pub type_: ElicitationSchemaType,
    /// Optional title for the schema.
    pub title: Option<String>,
    /// Property definitions (must be primitive types).
    #[serde(default)]
    pub properties: BTreeMap<String, ElicitationPropertySchema>,
    /// List of required property names.
    pub required: Option<Vec<String>>,
    /// Optional description of what this schema represents.
    pub description: Option<String>,
}

impl Default for ElicitationSchema {
    fn default() -> Self {
        Self {
            type_: default_object_type(),
            title: None,
            properties: BTreeMap::new(),
            required: None,
            description: None,
        }
    }
}

impl ElicitationSchema {
    /// Create a new empty elicitation schema.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Optional title for the schema.
    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
        self
    }

    /// Optional description of what this schema represents.
    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    /// Add a property to the schema.
    #[must_use]
    pub fn property<S>(mut self, name: impl Into<String>, schema: S, required: bool) -> Self
    where
        S: Into<ElicitationPropertySchema>,
    {
        let name = name.into();
        self.properties.insert(name.clone(), schema.into());

        if required {
            let required_fields = self.required.get_or_insert_with(Vec::new);
            if !required_fields.contains(&name) {
                required_fields.push(name);
            }
        } else if let Some(required_fields) = &mut self.required {
            required_fields.retain(|field| field != &name);

            if required_fields.is_empty() {
                self.required = None;
            }
        }

        self
    }

    /// Add a string property.
    #[must_use]
    pub fn string(self, name: impl Into<String>, required: bool) -> Self {
        self.property(name, StringPropertySchema::new(), required)
    }

    /// Add an email property.
    #[must_use]
    pub fn email(self, name: impl Into<String>, required: bool) -> Self {
        self.property(name, StringPropertySchema::email(), required)
    }

    /// Add a URI property.
    #[must_use]
    pub fn uri(self, name: impl Into<String>, required: bool) -> Self {
        self.property(name, StringPropertySchema::uri(), required)
    }

    /// Add a date property.
    #[must_use]
    pub fn date(self, name: impl Into<String>, required: bool) -> Self {
        self.property(name, StringPropertySchema::date(), required)
    }

    /// Add a date-time property.
    #[must_use]
    pub fn date_time(self, name: impl Into<String>, required: bool) -> Self {
        self.property(name, StringPropertySchema::date_time(), required)
    }

    /// Add a number property with range.
    #[must_use]
    pub fn number(self, name: impl Into<String>, min: f64, max: f64, required: bool) -> Self {
        self.property(
            name,
            NumberPropertySchema::new().minimum(min).maximum(max),
            required,
        )
    }

    /// Add an integer property with range.
    #[must_use]
    pub fn integer(self, name: impl Into<String>, min: i64, max: i64, required: bool) -> Self {
        self.property(
            name,
            IntegerPropertySchema::new().minimum(min).maximum(max),
            required,
        )
    }

    /// Add a boolean property.
    #[must_use]
    pub fn boolean(self, name: impl Into<String>, required: bool) -> Self {
        self.property(name, BooleanPropertySchema::new(), required)
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Elicitation capabilities supported by the client.
#[serde_as]
#[skip_serializing_none]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationCapabilities {
    /// Whether the client supports form-based elicitation.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub form: Option<ElicitationFormCapabilities>,
    /// Whether the client supports URL-based elicitation.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub url: Option<ElicitationUrlCapabilities>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ElicitationCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the client supports form-based elicitation.
    #[must_use]
    pub fn form(mut self, form: impl IntoOption<ElicitationFormCapabilities>) -> Self {
        self.form = form.into_option();
        self
    }

    /// Whether the client supports URL-based elicitation.
    #[must_use]
    pub fn url(mut self, url: impl IntoOption<ElicitationUrlCapabilities>) -> Self {
        self.url = url.into_option();
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Form-based elicitation capabilities.
#[skip_serializing_none]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationFormCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ElicitationFormCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// URL-based elicitation capabilities.
#[skip_serializing_none]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationUrlCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ElicitationUrlCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// The scope of an elicitation request, determining what context it's tied to.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ElicitationScope {
    /// Tied to a session, optionally to a specific tool call within that session.
    Session(ElicitationSessionScope),
    /// Tied to a specific JSON-RPC request outside of a session
    /// (e.g., during auth/configuration phases before any session is started).
    Request(ElicitationRequestScope),
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Session-scoped elicitation, optionally tied to a specific tool call.
///
/// When `tool_call_id` is set, the elicitation is tied to a specific tool call.
/// This is useful when an agent receives an elicitation from an MCP server
/// during a tool call and needs to redirect it to the user.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationSessionScope {
    /// The session this elicitation is tied to.
    pub session_id: SessionId,
    /// Optional tool call within the session.
    pub tool_call_id: Option<ToolCallId>,
}

impl ElicitationSessionScope {
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>) -> Self {
        Self {
            session_id: session_id.into(),
            tool_call_id: None,
        }
    }

    #[must_use]
    pub fn tool_call_id(mut self, tool_call_id: impl IntoOption<ToolCallId>) -> Self {
        self.tool_call_id = tool_call_id.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request-scoped elicitation, tied to a specific JSON-RPC request outside of a session
/// (e.g., during auth/configuration phases before any session is started).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationRequestScope {
    /// The request this elicitation is tied to.
    pub request_id: RequestId,
}

impl ElicitationRequestScope {
    #[must_use]
    pub fn new(request_id: impl Into<RequestId>) -> Self {
        Self {
            request_id: request_id.into(),
        }
    }
}

impl From<ElicitationSessionScope> for ElicitationScope {
    fn from(scope: ElicitationSessionScope) -> Self {
        Self::Session(scope)
    }
}

impl From<ElicitationRequestScope> for ElicitationScope {
    fn from(scope: ElicitationRequestScope) -> Self {
        Self::Request(scope)
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request from the agent to elicit structured user input.
///
/// The agent sends this to the client to request information from the user,
/// either via a form or by directing them to a URL.
/// Elicitations are tied to a session (optionally a tool call) or a request.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[schemars(extend("x-side" = "client", "x-method" = ELICITATION_CREATE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreateElicitationRequest {
    /// The elicitation mode and its mode-specific fields.
    #[serde(flatten)]
    pub mode: ElicitationMode,
    /// A human-readable message describing what input is needed.
    pub message: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl CreateElicitationRequest {
    #[must_use]
    pub fn new(mode: impl Into<ElicitationMode>, message: impl Into<String>) -> Self {
        Self {
            mode: mode.into(),
            message: message.into(),
            meta: None,
        }
    }

    /// Returns the scope this elicitation is tied to.
    #[must_use]
    pub fn scope(&self) -> &ElicitationScope {
        self.mode.scope()
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// The mode of elicitation, determining how user input is collected.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "mode", rename_all = "snake_case")]
#[schemars(extend("discriminator" = {"propertyName": "mode"}))]
#[non_exhaustive]
pub enum ElicitationMode {
    /// Form-based elicitation where the client renders a form from the provided schema.
    Form(ElicitationFormMode),
    /// URL-based elicitation where the client directs the user to a URL.
    Url(ElicitationUrlMode),
}

impl From<ElicitationFormMode> for ElicitationMode {
    fn from(mode: ElicitationFormMode) -> Self {
        Self::Form(mode)
    }
}

impl From<ElicitationUrlMode> for ElicitationMode {
    fn from(mode: ElicitationUrlMode) -> Self {
        Self::Url(mode)
    }
}

impl ElicitationMode {
    /// Returns the scope this elicitation mode is tied to.
    #[must_use]
    pub fn scope(&self) -> &ElicitationScope {
        match self {
            Self::Form(f) => &f.scope,
            Self::Url(u) => &u.scope,
        }
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Form-based elicitation mode where the client renders a form from the provided schema.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationFormMode {
    /// The scope this elicitation is tied to.
    #[serde(flatten)]
    pub scope: ElicitationScope,
    /// A JSON Schema describing the form fields to present to the user.
    pub requested_schema: ElicitationSchema,
}

impl ElicitationFormMode {
    #[must_use]
    pub fn new(scope: impl Into<ElicitationScope>, requested_schema: ElicitationSchema) -> Self {
        Self {
            scope: scope.into(),
            requested_schema,
        }
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// URL-based elicitation mode where the client directs the user to a URL.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationUrlMode {
    /// The scope this elicitation is tied to.
    #[serde(flatten)]
    pub scope: ElicitationScope,
    /// The unique identifier for this elicitation.
    pub elicitation_id: ElicitationId,
    /// The URL to direct the user to.
    #[schemars(extend("format" = "uri"))]
    pub url: String,
}

impl ElicitationUrlMode {
    #[must_use]
    pub fn new(
        scope: impl Into<ElicitationScope>,
        elicitation_id: impl Into<ElicitationId>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            scope: scope.into(),
            elicitation_id: elicitation_id.into(),
            url: url.into(),
        }
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response from the client to an elicitation request.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[schemars(extend("x-side" = "client", "x-method" = ELICITATION_CREATE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreateElicitationResponse {
    /// The user's action in response to the elicitation.
    #[serde(flatten)]
    pub action: ElicitationAction,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl CreateElicitationResponse {
    #[must_use]
    pub fn new(action: ElicitationAction) -> Self {
        Self { action, meta: None }
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// The user's action in response to an elicitation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
#[schemars(extend("discriminator" = {"propertyName": "action"}))]
#[non_exhaustive]
pub enum ElicitationAction {
    /// The user accepted and provided content.
    Accept(ElicitationAcceptAction),
    /// The user declined the elicitation.
    Decline,
    /// The elicitation was cancelled.
    Cancel,
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// The user accepted the elicitation and provided content.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationAcceptAction {
    /// The user-provided content, if any, as an object matching the requested schema.
    #[serde(default)]
    pub content: Option<BTreeMap<String, ElicitationContentValue>>,
}

impl ElicitationAcceptAction {
    #[must_use]
    pub fn new() -> Self {
        Self { content: None }
    }

    /// The user-provided content as an object matching the requested schema.
    #[must_use]
    pub fn content(
        mut self,
        content: impl IntoOption<BTreeMap<String, ElicitationContentValue>>,
    ) -> Self {
        self.content = content.into_option();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ElicitationContentValue {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    StringArray(Vec<String>),
}

impl From<String> for ElicitationContentValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for ElicitationContentValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<i64> for ElicitationContentValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<i32> for ElicitationContentValue {
    fn from(value: i32) -> Self {
        Self::Integer(i64::from(value))
    }
}

impl From<f64> for ElicitationContentValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<bool> for ElicitationContentValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<Vec<String>> for ElicitationContentValue {
    fn from(value: Vec<String>) -> Self {
        Self::StringArray(value)
    }
}

impl From<Vec<&str>> for ElicitationContentValue {
    fn from(value: Vec<&str>) -> Self {
        Self::StringArray(value.into_iter().map(str::to_string).collect())
    }
}

impl Default for ElicitationAcceptAction {
    fn default() -> Self {
        Self::new()
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Notification sent by the agent when a URL-based elicitation is complete.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "client", "x-method" = ELICITATION_COMPLETE_NOTIFICATION))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CompleteElicitationNotification {
    /// The ID of the elicitation that completed.
    pub elicitation_id: ElicitationId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl CompleteElicitationNotification {
    #[must_use]
    pub fn new(elicitation_id: impl Into<ElicitationId>) -> Self {
        Self {
            elicitation_id: elicitation_id.into(),
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Data payload for the `UrlElicitationRequired` error, describing the URL elicitations
/// the user must complete.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UrlElicitationRequiredData {
    /// The URL elicitations the user must complete.
    pub elicitations: Vec<UrlElicitationRequiredItem>,
}

impl UrlElicitationRequiredData {
    #[must_use]
    pub fn new(elicitations: Vec<UrlElicitationRequiredItem>) -> Self {
        Self { elicitations }
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A single URL elicitation item within the `UrlElicitationRequired` error data.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UrlElicitationRequiredItem {
    /// The elicitation mode (always `"url"` for this item type).
    pub mode: ElicitationUrlOnlyMode,
    /// The unique identifier for this elicitation.
    pub elicitation_id: ElicitationId,
    /// The URL the user should be directed to.
    #[schemars(extend("format" = "uri"))]
    pub url: String,
    /// A human-readable message describing what input is needed.
    pub message: String,
}

/// Type discriminator for URL-only elicitation error items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ElicitationUrlOnlyMode {
    /// URL elicitation mode.
    #[default]
    Url,
}

impl UrlElicitationRequiredItem {
    #[must_use]
    pub fn new(
        elicitation_id: impl Into<ElicitationId>,
        url: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            mode: ElicitationUrlOnlyMode::Url,
            elicitation_id: elicitation_id.into(),
            url: url.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn form_mode_request_serialization() {
        let schema = ElicitationSchema::new().string("name", true);
        let req = CreateElicitationRequest::new(
            ElicitationFormMode::new(ElicitationSessionScope::new("sess_1"), schema),
            "Please enter your name",
        );

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["sessionId"], "sess_1");
        assert!(json.get("toolCallId").is_none());
        assert_eq!(json["mode"], "form");
        assert_eq!(json["message"], "Please enter your name");
        assert!(json["requestedSchema"].is_object());
        assert_eq!(json["requestedSchema"]["type"], "object");
        assert_eq!(
            json["requestedSchema"]["properties"]["name"]["type"],
            "string"
        );

        let roundtripped: CreateElicitationRequest = serde_json::from_value(json).unwrap();
        assert_eq!(
            *roundtripped.scope(),
            ElicitationSessionScope::new("sess_1").into()
        );
        assert_eq!(roundtripped.message, "Please enter your name");
        assert!(matches!(roundtripped.mode, ElicitationMode::Form(_)));
    }

    #[test]
    fn url_mode_request_serialization() {
        let req = CreateElicitationRequest::new(
            ElicitationUrlMode::new(
                ElicitationSessionScope::new("sess_2").tool_call_id("tc_1"),
                "elic_1",
                "https://example.com/auth",
            ),
            "Please authenticate",
        );

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["sessionId"], "sess_2");
        assert_eq!(json["toolCallId"], "tc_1");
        assert_eq!(json["mode"], "url");
        assert_eq!(json["elicitationId"], "elic_1");
        assert_eq!(json["url"], "https://example.com/auth");
        assert_eq!(json["message"], "Please authenticate");

        let roundtripped: CreateElicitationRequest = serde_json::from_value(json).unwrap();
        assert_eq!(
            *roundtripped.scope(),
            ElicitationSessionScope::new("sess_2")
                .tool_call_id("tc_1")
                .into()
        );
        assert!(matches!(roundtripped.mode, ElicitationMode::Url(_)));
    }

    #[test]
    fn response_accept_serialization() {
        let resp = CreateElicitationResponse::new(ElicitationAction::Accept(
            ElicitationAcceptAction::new().content(BTreeMap::from([(
                "name".to_string(),
                ElicitationContentValue::from("Alice"),
            )])),
        ));

        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["action"], "accept");
        assert_eq!(json["content"]["name"], "Alice");

        let roundtripped: CreateElicitationResponse = serde_json::from_value(json).unwrap();
        assert!(matches!(
            roundtripped.action,
            ElicitationAction::Accept(ElicitationAcceptAction {
                content: Some(_),
                ..
            })
        ));
    }

    #[test]
    fn response_decline_serialization() {
        let resp = CreateElicitationResponse::new(ElicitationAction::Decline);

        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["action"], "decline");

        let roundtripped: CreateElicitationResponse = serde_json::from_value(json).unwrap();
        assert!(matches!(roundtripped.action, ElicitationAction::Decline));
    }

    #[test]
    fn response_cancel_serialization() {
        let resp = CreateElicitationResponse::new(ElicitationAction::Cancel);

        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["action"], "cancel");

        let roundtripped: CreateElicitationResponse = serde_json::from_value(json).unwrap();
        assert!(matches!(roundtripped.action, ElicitationAction::Cancel));
    }

    #[test]
    fn url_mode_request_scope_serialization() {
        let req = CreateElicitationRequest::new(
            ElicitationUrlMode::new(
                ElicitationRequestScope::new(RequestId::Number(42)),
                "elic_2",
                "https://example.com/setup",
            ),
            "Please complete setup",
        );

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["requestId"], 42);
        assert!(json.get("sessionId").is_none());
        assert_eq!(json["mode"], "url");
        assert_eq!(json["elicitationId"], "elic_2");
        assert_eq!(json["url"], "https://example.com/setup");
        assert_eq!(json["message"], "Please complete setup");

        let roundtripped: CreateElicitationRequest = serde_json::from_value(json).unwrap();
        assert_eq!(
            *roundtripped.scope(),
            ElicitationRequestScope::new(RequestId::Number(42)).into()
        );
        assert!(matches!(roundtripped.mode, ElicitationMode::Url(_)));
    }

    #[test]
    fn request_scope_request_serialization() {
        let req = CreateElicitationRequest::new(
            ElicitationFormMode::new(
                ElicitationRequestScope::new(RequestId::Number(99)),
                ElicitationSchema::new().string("workspace", true),
            ),
            "Enter workspace name",
        );

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["requestId"], 99);
        assert!(json.get("sessionId").is_none());

        let roundtripped: CreateElicitationRequest = serde_json::from_value(json).unwrap();
        assert_eq!(
            *roundtripped.scope(),
            ElicitationRequestScope::new(RequestId::Number(99)).into()
        );
    }

    /// `ClientResponse` is `#[serde(untagged)]` with `WriteTextFileResponse` (which has
    /// `#[serde(default)]`) listed first, so standalone deserialization is ambiguous.
    /// In practice, the RPC layer selects the correct variant based on the originating
    /// request method. These tests verify that serialization through `ClientResponse`
    /// produces the correct flattened wire format and round-trips back via the
    /// concrete `CreateElicitationResponse` type.
    #[test]
    fn client_response_serialization_accept() {
        use crate::ClientResponse;

        let resp = ClientResponse::CreateElicitationResponse(CreateElicitationResponse::new(
            ElicitationAction::Accept(ElicitationAcceptAction::new().content(BTreeMap::from([(
                "name".to_string(),
                ElicitationContentValue::from("Alice"),
            )]))),
        ));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["action"], "accept");
        assert_eq!(json["content"]["name"], "Alice");

        // Round-trip back through the concrete type
        let roundtripped: CreateElicitationResponse = serde_json::from_value(json).unwrap();
        assert!(matches!(roundtripped.action, ElicitationAction::Accept(_)));
    }

    #[test]
    fn client_response_serialization_decline() {
        use crate::ClientResponse;

        let resp = ClientResponse::CreateElicitationResponse(CreateElicitationResponse::new(
            ElicitationAction::Decline,
        ));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["action"], "decline");

        let roundtripped: CreateElicitationResponse = serde_json::from_value(json).unwrap();
        assert!(matches!(roundtripped.action, ElicitationAction::Decline));
    }

    #[test]
    fn client_response_serialization_cancel() {
        use crate::ClientResponse;

        let resp = ClientResponse::CreateElicitationResponse(CreateElicitationResponse::new(
            ElicitationAction::Cancel,
        ));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["action"], "cancel");

        let roundtripped: CreateElicitationResponse = serde_json::from_value(json).unwrap();
        assert!(matches!(roundtripped.action, ElicitationAction::Cancel));
    }

    /// Guard against serde regressions with the `flatten` + internally-tagged combination.
    /// Extra fields in the JSON must not cause deserialization failures.
    #[test]
    fn request_tolerates_extra_fields() {
        let json = json!({
            "sessionId": "sess_1",
            "mode": "form",
            "message": "Enter your name",
            "requestedSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "title": "Name" }
                },
                "required": ["name"]
            },
            "unknownStringField": "hello",
            "unknownNumberField": 42
        });

        let req: CreateElicitationRequest = serde_json::from_value(json).unwrap();
        assert_eq!(*req.scope(), ElicitationSessionScope::new("sess_1").into());
        assert_eq!(req.message, "Enter your name");
        assert!(matches!(req.mode, ElicitationMode::Form(_)));
    }

    #[test]
    fn completion_notification_serialization() {
        let notif = CompleteElicitationNotification::new("elic_1");

        let json = serde_json::to_value(&notif).unwrap();
        assert_eq!(json["elicitationId"], "elic_1");

        let roundtripped: CompleteElicitationNotification = serde_json::from_value(json).unwrap();
        assert_eq!(roundtripped.elicitation_id, ElicitationId::new("elic_1"));
    }

    #[test]
    fn capabilities_form_only() {
        let caps = ElicitationCapabilities::new().form(ElicitationFormCapabilities::new());

        let json = serde_json::to_value(&caps).unwrap();
        assert!(json["form"].is_object());
        assert!(json.get("url").is_none());

        let roundtripped: ElicitationCapabilities = serde_json::from_value(json).unwrap();
        assert!(roundtripped.form.is_some());
        assert!(roundtripped.url.is_none());
    }

    #[test]
    fn capabilities_url_only() {
        let caps = ElicitationCapabilities::new().url(ElicitationUrlCapabilities::new());

        let json = serde_json::to_value(&caps).unwrap();
        assert!(json.get("form").is_none());
        assert!(json["url"].is_object());

        let roundtripped: ElicitationCapabilities = serde_json::from_value(json).unwrap();
        assert!(roundtripped.form.is_none());
        assert!(roundtripped.url.is_some());
    }

    #[test]
    fn capabilities_both() {
        let caps = ElicitationCapabilities::new()
            .form(ElicitationFormCapabilities::new())
            .url(ElicitationUrlCapabilities::new());

        let json = serde_json::to_value(&caps).unwrap();
        assert!(json["form"].is_object());
        assert!(json["url"].is_object());

        let roundtripped: ElicitationCapabilities = serde_json::from_value(json).unwrap();
        assert!(roundtripped.form.is_some());
        assert!(roundtripped.url.is_some());
    }

    #[test]
    fn url_elicitation_required_data_serialization() {
        let data = UrlElicitationRequiredData::new(vec![UrlElicitationRequiredItem::new(
            "elic_1",
            "https://example.com/auth",
            "Please authenticate",
        )]);

        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["elicitations"][0]["mode"], "url");
        assert_eq!(json["elicitations"][0]["elicitationId"], "elic_1");
        assert_eq!(json["elicitations"][0]["url"], "https://example.com/auth");

        let roundtripped: UrlElicitationRequiredData = serde_json::from_value(json).unwrap();
        assert_eq!(roundtripped.elicitations.len(), 1);
        assert_eq!(
            roundtripped.elicitations[0].mode,
            ElicitationUrlOnlyMode::Url
        );
    }

    #[test]
    fn schema_default_sets_object_type() {
        let schema = ElicitationSchema::default();

        assert_eq!(schema.type_, ElicitationSchemaType::Object);
        assert!(schema.properties.is_empty());

        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["type"], "object");
    }

    #[test]
    fn schema_builder_serialization() {
        let schema = ElicitationSchema::new()
            .string("name", true)
            .email("email", true)
            .integer("age", 0, 150, true)
            .boolean("newsletter", false)
            .description("User registration");

        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["type"], "object");
        assert_eq!(json["description"], "User registration");
        assert_eq!(json["properties"]["name"]["type"], "string");
        assert_eq!(json["properties"]["email"]["type"], "string");
        assert_eq!(json["properties"]["email"]["format"], "email");
        assert_eq!(json["properties"]["age"]["type"], "integer");
        assert_eq!(json["properties"]["age"]["minimum"], 0);
        assert_eq!(json["properties"]["age"]["maximum"], 150);
        assert_eq!(json["properties"]["newsletter"]["type"], "boolean");

        let required = json["required"].as_array().unwrap();
        assert!(required.contains(&json!("name")));
        assert!(required.contains(&json!("email")));
        assert!(required.contains(&json!("age")));
        assert!(!required.contains(&json!("newsletter")));

        let roundtripped: ElicitationSchema = serde_json::from_value(json).unwrap();
        assert_eq!(roundtripped.properties.len(), 4);
        assert!(roundtripped.required.unwrap().contains(&"name".to_string()));
    }

    #[test]
    fn schema_string_enum_serialization() {
        let schema = ElicitationSchema::new().property(
            "color",
            StringPropertySchema::new().enum_values(vec![
                "red".into(),
                "green".into(),
                "blue".into(),
            ]),
            true,
        );

        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["properties"]["color"]["type"], "string");
        let enum_vals = json["properties"]["color"]["enum"].as_array().unwrap();
        assert_eq!(enum_vals.len(), 3);

        let roundtripped: ElicitationSchema = serde_json::from_value(json).unwrap();
        if let ElicitationPropertySchema::String(s) = roundtripped.properties.get("color").unwrap()
        {
            assert_eq!(s.enum_values.as_ref().unwrap().len(), 3);
        } else {
            panic!("expected String variant");
        }
    }

    #[test]
    fn schema_multi_select_serialization() {
        let schema = ElicitationSchema::new().property(
            "colors",
            MultiSelectPropertySchema::new(vec!["red".into(), "green".into(), "blue".into()])
                .min_items(1)
                .max_items(3),
            false,
        );

        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["properties"]["colors"]["type"], "array");
        assert_eq!(json["properties"]["colors"]["items"]["type"], "string");
        assert_eq!(json["properties"]["colors"]["minItems"], 1);
        assert_eq!(json["properties"]["colors"]["maxItems"], 3);

        let roundtripped: ElicitationSchema = serde_json::from_value(json).unwrap();
        assert!(matches!(
            roundtripped.properties.get("colors").unwrap(),
            ElicitationPropertySchema::Array(_)
        ));
    }

    #[test]
    fn schema_titled_enum_serialization() {
        let schema = ElicitationSchema::new().property(
            "country",
            StringPropertySchema::new().one_of(vec![
                EnumOption::new("us", "United States"),
                EnumOption::new("uk", "United Kingdom"),
            ]),
            true,
        );

        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["properties"]["country"]["type"], "string");
        let one_of = json["properties"]["country"]["oneOf"].as_array().unwrap();
        assert_eq!(one_of.len(), 2);
        assert_eq!(one_of[0]["const"], "us");
        assert_eq!(one_of[0]["title"], "United States");

        let roundtripped: ElicitationSchema = serde_json::from_value(json).unwrap();
        if let ElicitationPropertySchema::String(s) =
            roundtripped.properties.get("country").unwrap()
        {
            assert_eq!(s.one_of.as_ref().unwrap().len(), 2);
        } else {
            panic!("expected String variant");
        }
    }

    #[test]
    fn schema_number_property_serialization() {
        let schema = ElicitationSchema::new().number("rating", 0.0, 5.0, true);

        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["properties"]["rating"]["type"], "number");
        assert_eq!(json["properties"]["rating"]["minimum"], 0.0);
        assert_eq!(json["properties"]["rating"]["maximum"], 5.0);

        let roundtripped: ElicitationSchema = serde_json::from_value(json).unwrap();
        if let ElicitationPropertySchema::Number(n) = roundtripped.properties.get("rating").unwrap()
        {
            assert_eq!(n.minimum, Some(0.0));
            assert_eq!(n.maximum, Some(5.0));
        } else {
            panic!("expected Number variant");
        }
    }

    #[test]
    fn schema_string_format_serialization() {
        let schema = ElicitationSchema::new()
            .uri("website", true)
            .date("birthday", true)
            .date_time("updated_at", false);

        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["properties"]["website"]["type"], "string");
        assert_eq!(json["properties"]["website"]["format"], "uri");
        assert_eq!(json["properties"]["birthday"]["type"], "string");
        assert_eq!(json["properties"]["birthday"]["format"], "date");
        assert_eq!(json["properties"]["updated_at"]["type"], "string");
        assert_eq!(json["properties"]["updated_at"]["format"], "date-time");

        let required = json["required"].as_array().unwrap();
        assert!(required.contains(&json!("website")));
        assert!(required.contains(&json!("birthday")));
        assert!(!required.contains(&json!("updated_at")));
    }

    #[test]
    fn schema_string_pattern_serialization() {
        let schema = ElicitationSchema::new().property(
            "name",
            StringPropertySchema::new()
                .min_length(1)
                .max_length(64)
                .pattern("^[a-zA-Z_][a-zA-Z0-9_]*$"),
            true,
        );

        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["properties"]["name"]["type"], "string");
        assert_eq!(
            json["properties"]["name"]["pattern"],
            "^[a-zA-Z_][a-zA-Z0-9_]*$"
        );

        let roundtripped: ElicitationSchema = serde_json::from_value(json).unwrap();
        if let ElicitationPropertySchema::String(s) = roundtripped.properties.get("name").unwrap() {
            assert_eq!(s.pattern.as_deref(), Some("^[a-zA-Z_][a-zA-Z0-9_]*$"));
        } else {
            panic!("expected String variant");
        }
    }

    #[test]
    fn schema_property_updates_required_state() {
        let schema = ElicitationSchema::new()
            .string("name", true)
            .email("name", false);

        let json = serde_json::to_value(&schema).unwrap();
        assert!(json.get("required").is_none());
        assert_eq!(json["properties"]["name"]["format"], "email");
    }

    #[test]
    fn schema_rejects_invalid_object_type() {
        let err = serde_json::from_value::<ElicitationSchema>(json!({
            "type": "array",
            "properties": {
                "name": {
                    "type": "string"
                }
            }
        }))
        .unwrap_err();

        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn titled_multi_select_items_reject_one_of() {
        let err = serde_json::from_value::<TitledMultiSelectItems>(json!({
            "oneOf": [
                {
                    "const": "red",
                    "title": "Red"
                }
            ]
        }))
        .unwrap_err();

        assert!(err.to_string().contains("missing field `anyOf`"));
    }

    #[test]
    fn response_accept_rejects_non_object_content() {
        let err = serde_json::from_value::<CreateElicitationResponse>(json!({
            "action": "accept",
            "content": "Alice"
        }))
        .unwrap_err();

        assert!(err.to_string().contains("invalid type"));
    }

    #[test]
    fn response_accept_rejects_nested_object_content() {
        let err = serde_json::from_value::<CreateElicitationResponse>(json!({
            "action": "accept",
            "content": {
                "profile": {
                    "name": "Alice"
                }
            }
        }))
        .unwrap_err();

        assert!(err.to_string().contains("data did not match any variant"));
    }

    #[test]
    fn response_accept_allows_primitive_and_string_array_content() {
        let response = CreateElicitationResponse::new(ElicitationAction::Accept(
            ElicitationAcceptAction::new().content(BTreeMap::from([
                ("name".to_string(), ElicitationContentValue::from("Alice")),
                ("age".to_string(), ElicitationContentValue::from(30_i32)),
                ("score".to_string(), ElicitationContentValue::from(9.5_f64)),
                (
                    "subscribed".to_string(),
                    ElicitationContentValue::from(true),
                ),
                (
                    "tags".to_string(),
                    ElicitationContentValue::from(vec!["rust", "acp"]),
                ),
            ])),
        ));

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["action"], "accept");
        assert_eq!(json["content"]["name"], "Alice");
        assert_eq!(json["content"]["age"], 30);
        assert_eq!(json["content"]["score"], 9.5);
        assert_eq!(json["content"]["subscribed"], true);
        assert_eq!(json["content"]["tags"][0], "rust");
        assert_eq!(json["content"]["tags"][1], "acp");
    }
}
