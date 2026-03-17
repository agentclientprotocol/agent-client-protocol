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

use crate::client::{SESSION_ELICITATION_COMPLETE, SESSION_ELICITATION_METHOD_NAME};
use crate::{IntoOption, Meta, SessionId};

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

// =============================================================================
// ELICITATION SCHEMA
// =============================================================================

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
/// When `enum_values` or `one_of` is set, this represents a single-select enum
/// with `"type": "string"`.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StringPropertySchema {
    /// Optional title for the property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum string length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u32>,
    /// Maximum string length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    /// String format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<StringFormat>,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Enum values for untitled single-select enums.
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    /// Titled enum options for titled single-select enums.
    #[serde(rename = "oneOf", skip_serializing_if = "Option::is_none")]
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
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NumberPropertySchema {
    /// Optional title for the property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// Maximum value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
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
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct IntegerPropertySchema {
    /// Optional title for the property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i64>,
    /// Maximum value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
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
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BooleanPropertySchema {
    /// Optional title for the property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct UntitledMultiSelectItems {
    /// Allowed enum values.
    #[serde(rename = "enum")]
    pub values: Vec<String>,
}

/// Items definition for titled multi-select enum properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct TitledMultiSelectItems {
    /// Titled enum options.
    #[serde(rename = "anyOf", alias = "oneOf")]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MultiSelectPropertySchema {
    /// Optional title for the property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum number of items to select.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,
    /// Maximum number of items to select.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    /// The items definition describing allowed values.
    pub items: MultiSelectItems,
    /// Default selected values.
    #[serde(skip_serializing_if = "Option::is_none")]
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
            items: MultiSelectItems::Untitled(UntitledMultiSelectItems { values }),
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
/// Single-select enums use the `String` variant with `enum_values` or `one_of` set.
/// Multi-select enums use the `Array` variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(extend("discriminator" = {"propertyName": "type"}))]
#[non_exhaustive]
pub enum ElicitationPropertySchema {
    /// String property (or single-select enum when `enum`/`oneOf` is set).
    #[serde(rename = "string")]
    String(StringPropertySchema),
    /// Number (floating-point) property.
    #[serde(rename = "number")]
    Number(NumberPropertySchema),
    /// Integer property.
    #[serde(rename = "integer")]
    Integer(IntegerPropertySchema),
    /// Boolean property.
    #[serde(rename = "boolean")]
    Boolean(BooleanPropertySchema),
    /// Multi-select array property.
    #[serde(rename = "array")]
    Array(MultiSelectPropertySchema),
}

fn default_object_type() -> String {
    "object".into()
}

/// Type-safe elicitation schema for requesting structured user input.
///
/// This represents a JSON Schema object with primitive-typed properties,
/// as required by the elicitation specification.
///
/// # Example
///
/// ```
/// use agent_client_protocol_schema::*;
///
/// let schema = ElicitationSchema::new()
///     .required_string("name")
///     .required_email("email")
///     .optional_bool("newsletter");
/// ```
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationSchema {
    /// Type discriminator. Always `"object"`.
    #[serde(rename = "type", default = "default_object_type")]
    pub type_: String,
    /// Optional title for the schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Property definitions (must be primitive types).
    #[serde(default)]
    pub properties: BTreeMap<String, ElicitationPropertySchema>,
    /// List of required property names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Optional description of what this schema represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ElicitationSchema {
    /// Create a new empty elicitation schema.
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_: "object".into(),
            ..Default::default()
        }
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
    pub fn property(mut self, name: impl Into<String>, schema: ElicitationPropertySchema) -> Self {
        self.properties.insert(name.into(), schema);
        self
    }

    /// Add a required property to the schema.
    #[must_use]
    pub fn required_property(
        mut self,
        name: impl Into<String>,
        schema: ElicitationPropertySchema,
    ) -> Self {
        let name_str = name.into();
        self.required
            .get_or_insert_with(Vec::new)
            .push(name_str.clone());
        self.properties.insert(name_str, schema);
        self
    }

    /// Add a required string property.
    #[must_use]
    pub fn required_string(self, name: impl Into<String>) -> Self {
        self.required_property(
            name,
            ElicitationPropertySchema::String(StringPropertySchema::new()),
        )
    }

    /// Add an optional string property.
    #[must_use]
    pub fn optional_string(self, name: impl Into<String>) -> Self {
        self.property(
            name,
            ElicitationPropertySchema::String(StringPropertySchema::new()),
        )
    }

    /// Add a required email property.
    #[must_use]
    pub fn required_email(self, name: impl Into<String>) -> Self {
        self.required_property(
            name,
            ElicitationPropertySchema::String(StringPropertySchema::email()),
        )
    }

    /// Add an optional email property.
    #[must_use]
    pub fn optional_email(self, name: impl Into<String>) -> Self {
        self.property(
            name,
            ElicitationPropertySchema::String(StringPropertySchema::email()),
        )
    }

    /// Add a required number property with range.
    #[must_use]
    pub fn required_number(self, name: impl Into<String>, min: f64, max: f64) -> Self {
        self.required_property(
            name,
            ElicitationPropertySchema::Number(
                NumberPropertySchema::new().minimum(min).maximum(max),
            ),
        )
    }

    /// Add an optional number property with range.
    #[must_use]
    pub fn optional_number(self, name: impl Into<String>, min: f64, max: f64) -> Self {
        self.property(
            name,
            ElicitationPropertySchema::Number(
                NumberPropertySchema::new().minimum(min).maximum(max),
            ),
        )
    }

    /// Add a required integer property with range.
    #[must_use]
    pub fn required_integer(self, name: impl Into<String>, min: i64, max: i64) -> Self {
        self.required_property(
            name,
            ElicitationPropertySchema::Integer(
                IntegerPropertySchema::new().minimum(min).maximum(max),
            ),
        )
    }

    /// Add an optional integer property with range.
    #[must_use]
    pub fn optional_integer(self, name: impl Into<String>, min: i64, max: i64) -> Self {
        self.property(
            name,
            ElicitationPropertySchema::Integer(
                IntegerPropertySchema::new().minimum(min).maximum(max),
            ),
        )
    }

    /// Add a required boolean property.
    #[must_use]
    pub fn required_bool(self, name: impl Into<String>) -> Self {
        self.required_property(
            name,
            ElicitationPropertySchema::Boolean(BooleanPropertySchema::new()),
        )
    }

    /// Add an optional boolean property.
    #[must_use]
    pub fn optional_bool(self, name: impl Into<String>) -> Self {
        self.property(
            name,
            ElicitationPropertySchema::Boolean(BooleanPropertySchema::new()),
        )
    }
}

// =============================================================================
// ELICITATION CAPABILITIES
// =============================================================================

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Elicitation capabilities supported by the client.
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationCapabilities {
    /// Whether the client supports form-based elicitation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form: Option<ElicitationFormCapabilities>,
    /// Whether the client supports URL-based elicitation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<ElicitationUrlCapabilities>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
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
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationFormCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
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
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationUrlCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
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
/// Request from the agent to elicit structured user input.
///
/// The agent sends this to the client to request information from the user,
/// either via a form or by directing them to a URL.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[schemars(extend("x-side" = "client", "x-method" = SESSION_ELICITATION_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
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
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ElicitationRequest {
    #[must_use]
    pub fn new(
        session_id: impl Into<SessionId>,
        mode: ElicitationMode,
        message: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            mode,
            message: message.into(),
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Form-based elicitation mode where the client renders a form from the provided schema.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationFormMode {
    /// A JSON Schema describing the form fields to present to the user.
    pub requested_schema: ElicitationSchema,
}

impl ElicitationFormMode {
    #[must_use]
    pub fn new(requested_schema: ElicitationSchema) -> Self {
        Self { requested_schema }
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
    /// The unique identifier for this elicitation.
    pub elicitation_id: ElicitationId,
    /// The URL to direct the user to.
    pub url: String,
}

impl ElicitationUrlMode {
    #[must_use]
    pub fn new(elicitation_id: impl Into<ElicitationId>, url: impl Into<String>) -> Self {
        Self {
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[schemars(extend("x-side" = "client", "x-method" = SESSION_ELICITATION_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationResponse {
    /// The user's action in response to the elicitation.
    pub action: ElicitationAction,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ElicitationResponse {
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationAcceptAction {
    /// The user-provided content, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
}

impl ElicitationAcceptAction {
    #[must_use]
    pub fn new() -> Self {
        Self { content: None }
    }

    /// The user-provided content.
    #[must_use]
    pub fn content(mut self, content: impl IntoOption<serde_json::Value>) -> Self {
        self.content = content.into_option();
        self
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "client", "x-method" = SESSION_ELICITATION_COMPLETE))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationCompleteNotification {
    /// The ID of the elicitation that completed.
    pub elicitation_id: ElicitationId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ElicitationCompleteNotification {
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
    pub mode: String,
    /// The unique identifier for this elicitation.
    pub elicitation_id: ElicitationId,
    /// The URL the user should be directed to.
    pub url: String,
    /// A human-readable message describing what input is needed.
    pub message: String,
}

impl UrlElicitationRequiredItem {
    #[must_use]
    pub fn new(
        elicitation_id: impl Into<ElicitationId>,
        url: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            mode: "url".to_string(),
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
        let schema = ElicitationSchema::new().required_string("name");
        let req = ElicitationRequest::new(
            "sess_1",
            ElicitationMode::Form(ElicitationFormMode::new(schema)),
            "Please enter your name",
        );

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["sessionId"], "sess_1");
        assert_eq!(json["mode"], "form");
        assert_eq!(json["message"], "Please enter your name");
        assert!(json["requestedSchema"].is_object());
        assert_eq!(json["requestedSchema"]["type"], "object");
        assert_eq!(
            json["requestedSchema"]["properties"]["name"]["type"],
            "string"
        );

        let roundtripped: ElicitationRequest = serde_json::from_value(json).unwrap();
        assert_eq!(roundtripped.session_id, SessionId::new("sess_1"));
        assert_eq!(roundtripped.message, "Please enter your name");
        assert!(matches!(roundtripped.mode, ElicitationMode::Form(_)));
    }

    #[test]
    fn url_mode_request_serialization() {
        let req = ElicitationRequest::new(
            "sess_2",
            ElicitationMode::Url(ElicitationUrlMode::new(
                "elic_1",
                "https://example.com/auth",
            )),
            "Please authenticate",
        );

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["sessionId"], "sess_2");
        assert_eq!(json["mode"], "url");
        assert_eq!(json["elicitationId"], "elic_1");
        assert_eq!(json["url"], "https://example.com/auth");
        assert_eq!(json["message"], "Please authenticate");

        let roundtripped: ElicitationRequest = serde_json::from_value(json).unwrap();
        assert_eq!(roundtripped.session_id, SessionId::new("sess_2"));
        assert!(matches!(roundtripped.mode, ElicitationMode::Url(_)));
    }

    #[test]
    fn response_accept_serialization() {
        let resp = ElicitationResponse::new(ElicitationAction::Accept(
            ElicitationAcceptAction::new().content(json!({"name": "Alice"})),
        ));

        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["action"]["action"], "accept");
        assert_eq!(json["action"]["content"]["name"], "Alice");

        let roundtripped: ElicitationResponse = serde_json::from_value(json).unwrap();
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
        let resp = ElicitationResponse::new(ElicitationAction::Decline);

        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["action"]["action"], "decline");

        let roundtripped: ElicitationResponse = serde_json::from_value(json).unwrap();
        assert!(matches!(roundtripped.action, ElicitationAction::Decline));
    }

    #[test]
    fn response_cancel_serialization() {
        let resp = ElicitationResponse::new(ElicitationAction::Cancel);

        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["action"]["action"], "cancel");

        let roundtripped: ElicitationResponse = serde_json::from_value(json).unwrap();
        assert!(matches!(roundtripped.action, ElicitationAction::Cancel));
    }

    #[test]
    fn completion_notification_serialization() {
        let notif = ElicitationCompleteNotification::new("elic_1");

        let json = serde_json::to_value(&notif).unwrap();
        assert_eq!(json["elicitationId"], "elic_1");

        let roundtripped: ElicitationCompleteNotification = serde_json::from_value(json).unwrap();
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
        assert_eq!(roundtripped.elicitations[0].mode, "url");
    }

    #[test]
    fn schema_builder_serialization() {
        let schema = ElicitationSchema::new()
            .required_string("name")
            .required_email("email")
            .required_integer("age", 0, 150)
            .optional_bool("newsletter")
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
        let schema = ElicitationSchema::new().required_property(
            "color",
            ElicitationPropertySchema::String(StringPropertySchema::new().enum_values(vec![
                "red".into(),
                "green".into(),
                "blue".into(),
            ])),
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
            ElicitationPropertySchema::Array(
                MultiSelectPropertySchema::new(vec!["red".into(), "green".into(), "blue".into()])
                    .min_items(1)
                    .max_items(3),
            ),
        );

        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["properties"]["colors"]["type"], "array");
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
        let schema = ElicitationSchema::new().required_property(
            "country",
            ElicitationPropertySchema::String(StringPropertySchema::new().one_of(vec![
                EnumOption::new("us", "United States"),
                EnumOption::new("uk", "United Kingdom"),
            ])),
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
        let schema = ElicitationSchema::new().required_number("rating", 0.0, 5.0);

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
}
