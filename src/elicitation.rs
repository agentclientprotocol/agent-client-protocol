use serde::{Deserialize, Serialize};
use serde_json::json;
use schemars::JsonSchema;

/// An elicitation request for structured user input during a turn response.
/// Agents can request information from users through this mechanism.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[cfg_attr(feature = "unstable_elicitation", serde(rename_all = "camelCase"))]
pub struct ElicitationRequest {
    /// Unique identifier for this elicitation request within the session
    pub id: String,

    /// Type of input being requested
    #[serde(rename = "type")]
    pub input_type: ElicitationType,

    /// Human-readable title/prompt for the user
    pub title: String,

    /// Detailed description of what's being requested
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// JSON Schema constraints for the input
    pub schema: ElicitationSchema,

    /// Available options for select/multiselect types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ElicitationOption>>,

    /// For url type: authorization/callback URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// For url type: specifies how the returned value should be formatted
    /// (e.g., "token" for OAuth)
    #[serde(skip_serializing_if = "Option::is_none", rename = "returnValueFormat")]
    pub return_value_format: Option<String>,

    /// Additional metadata for extensibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl ElicitationRequest {
    pub fn new(id: impl Into<String>, input_type: ElicitationType, title: impl Into<String>, schema: ElicitationSchema) -> Self {
        Self {
            id: id.into(),
            input_type,
            title: title.into(),
            description: None,
            schema,
            options: None,
            url: None,
            return_value_format: None,
            meta: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn options(mut self, options: Vec<ElicitationOption>) -> Self {
        self.options = Some(options);
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn return_value_format(mut self, format: impl Into<String>) -> Self {
        self.return_value_format = Some(format.into());
        self
    }

    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Types of user input that can be elicited
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ElicitationType {
    /// Open-ended text input
    Text,
    /// Numeric input
    Number,
    /// Single-choice selection
    Select,
    /// Multiple-choice selection
    Multiselect,
    /// Yes/no choice
    Boolean,
    /// Masked password input
    Password,
    /// Out-of-band URL (browser-based, e.g., OAuth)
    Url,
}

impl std::fmt::Display for ElicitationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElicitationType::Text => write!(f, "text"),
            ElicitationType::Number => write!(f, "number"),
            ElicitationType::Select => write!(f, "select"),
            ElicitationType::Multiselect => write!(f, "multiselect"),
            ElicitationType::Boolean => write!(f, "boolean"),
            ElicitationType::Password => write!(f, "password"),
            ElicitationType::Url => write!(f, "url"),
        }
    }
}

/// JSON Schema constraints for elicitation input
/// Restricted subset of JSON Schema for security and simplicity
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[cfg_attr(feature = "unstable_elicitation", serde(rename_all = "camelCase"))]
pub struct ElicitationSchema {
    /// Data type of the input (must match ElicitationType)
    #[serde(rename = "type")]
    pub type_: String,

    /// Default value if user doesn't respond
    /// MUST be provided to prevent blocking scenarios
    pub default: Option<serde_json::Value>,

    /// Help text explaining the field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Allowed values (for select/multiselect)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<serde_json::Value>>,

    /// Minimum string length
    #[serde(skip_serializing_if = "Option::is_none", rename = "minLength")]
    pub min_length: Option<u64>,

    /// Maximum string length
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxLength")]
    pub max_length: Option<u64>,

    /// Minimum numeric value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i64>,

    /// Maximum numeric value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,

    /// Regex pattern for string validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

impl ElicitationSchema {
    pub fn new(type_: impl Into<String>, default: Option<serde_json::Value>) -> Self {
        Self {
            type_: type_.into(),
            default,
            description: None,
            r#enum: None,
            min_length: None,
            max_length: None,
            minimum: None,
            maximum: None,
            pattern: None,
        }
    }

    pub fn with_type(mut self, type_: impl Into<String>) -> Self {
        self.type_ = type_.into();
        self
    }

    pub fn with_default(mut self, default: serde_json::Value) -> Self {
        self.default = Some(default);
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn enum_values(mut self, values: Vec<serde_json::Value>) -> Self {
        self.r#enum = Some(values);
        self
    }

    pub fn min_length(mut self, min: u64) -> Self {
        self.min_length = Some(min);
        self
    }

    pub fn max_length(mut self, max: u64) -> Self {
        self.max_length = Some(max);
        self
    }

    pub fn minimum(mut self, min: i64) -> Self {
        self.minimum = Some(min);
        self
    }

    pub fn maximum(mut self, max: i64) -> Self {
        self.maximum = Some(max);
        self
    }

    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }
}

/// Option for select/multiselect input types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ElicitationOption {
    /// The value to send back if this option is selected
    pub value: String,

    /// Human-readable label for the option
    pub label: String,

    /// Description of what this option means
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Additional metadata for extensibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl ElicitationOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            meta: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// User's response to an elicitation request
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ElicitationResponse {
    /// ID matching the corresponding ElicitationRequest.id
    pub id: String,

    /// The user's answer/selection
    pub value: serde_json::Value,

    /// Additional metadata for extensibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl ElicitationResponse {
    pub fn new(id: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            value,
            meta: None,
        }
    }

    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Create a response with a string value
    pub fn string(id: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(id, serde_json::Value::String(value.into()))
    }

    /// Create a response with a number value
    pub fn number(id: impl Into<String>, value: i64) -> Self {
        Self::new(id, json!(value))
    }

    /// Create a response with an array value (for multiselect)
    pub fn array(id: impl Into<String>, values: Vec<String>) -> Self {
        Self::new(id, serde_json::Value::Array(
            values.into_iter().map(serde_json::Value::String).collect()
        ))
    }

    /// Create a response with a boolean value
    pub fn boolean(id: impl Into<String>, value: bool) -> Self {
        Self::new(id, serde_json::Value::Bool(value))
    }
}

/// Client capabilities for elicitation support
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ElicitationCapability {
    /// Whether the client supports elicitation at all
    pub supported: bool,

    /// List of supported input types (e.g., ["text", "select", "number"])
    #[serde(rename = "supportedTypes")]
    pub supported_types: Vec<String>,

    /// Additional metadata for extensibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl ElicitationCapability {
    pub fn new(supported: bool, supported_types: Vec<String>) -> Self {
        Self {
            supported,
            supported_types,
            meta: None,
        }
    }

    pub fn supported_all() -> Self {
        Self::new(
            true,
            vec![
                "text".to_string(),
                "number".to_string(),
                "select".to_string(),
                "multiselect".to_string(),
                "boolean".to_string(),
                "password".to_string(),
            ],
        )
    }

    pub fn unsupported() -> Self {
        Self::new(false, vec![])
    }

    pub fn meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elicitation_request_builder() {
        let schema = ElicitationSchema::new("string", Some(json!("balanced")))
            .description("Choose an approach");

        let request = ElicitationRequest::new("req1", ElicitationType::Select, "Choose Strategy", schema)
            .description("Which refactoring approach do you prefer?")
            .options(vec![
                ElicitationOption::new("conservative", "Conservative"),
                ElicitationOption::new("balanced", "Balanced"),
                ElicitationOption::new("aggressive", "Aggressive"),
            ]);

        assert_eq!(request.id, "req1");
        assert_eq!(request.input_type, ElicitationType::Select);
        assert!(request.description.is_some());
        assert!(request.options.is_some());
        assert_eq!(request.options.unwrap().len(), 3);
    }

    #[test]
    fn test_elicitation_response_builders() {
        let string_resp = ElicitationResponse::string("req1", "balanced");
        assert_eq!(string_resp.value, json!("balanced"));

        let number_resp = ElicitationResponse::number("req2", 42);
        assert_eq!(number_resp.value, json!(42));

        let bool_resp = ElicitationResponse::boolean("req3", true);
        assert_eq!(bool_resp.value, json!(true));

        let array_resp = ElicitationResponse::array("req4", vec!["opt1".to_string(), "opt2".to_string()]);
        assert!(array_resp.value.is_array());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let request = ElicitationRequest::new(
            "test-id",
            ElicitationType::Text,
            "Function Name",
            ElicitationSchema::new("string", Some(json!("processData")))
                .min_length(1)
                .max_length(64)
                .pattern("^[a-zA-Z_][a-zA-Z0-9_]*$"),
        );

        let json = serde_json::to_string(&request).expect("serialize");
        let deserialized: ElicitationRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_elicitation_capability() {
        let capability = ElicitationCapability::supported_all();
        assert!(capability.supported);
        assert_eq!(capability.supported_types.len(), 6);

        let unsupported = ElicitationCapability::unsupported();
        assert!(!unsupported.supported);
        assert!(unsupported.supported_types.is_empty());
    }

    #[test]
    fn test_elicitation_type_display() {
        assert_eq!(ElicitationType::Text.to_string(), "text");
        assert_eq!(ElicitationType::Select.to_string(), "select");
        assert_eq!(ElicitationType::Multiselect.to_string(), "multiselect");
    }

    #[test]
    fn test_schema_with_constraints() {
        let schema = ElicitationSchema::new("number", Some(json!(5)))
            .minimum(0)
            .maximum(100)
            .description("Pick a number between 0 and 100");

        assert_eq!(schema.minimum, Some(0));
        assert_eq!(schema.maximum, Some(100));
        assert!(schema.description.is_some());
    }

    #[test]
    fn test_schema_with_enum_values() {
        let schema = ElicitationSchema::new("string", Some(json!("red")))
            .enum_values(vec![json!("red"), json!("green"), json!("blue")])
            .description("Choose a color");

        assert!(schema.r#enum.is_some());
        assert_eq!(schema.r#enum.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_url_type_with_return_format() {
        let request = ElicitationRequest::new(
            "oauth-req",
            ElicitationType::Url,
            "Authenticate with GitHub",
            ElicitationSchema::new("string", None),
        )
        .url("https://github.com/login/oauth/authorize?client_id=xxx")
        .return_value_format("token");

        assert_eq!(request.input_type, ElicitationType::Url);
        assert_eq!(request.url, Some("https://github.com/login/oauth/authorize?client_id=xxx".to_string()));
        assert_eq!(request.return_value_format, Some("token".to_string()));
    }

    #[test]
    fn test_elicitation_option_with_metadata() {
        let option = ElicitationOption::new("backend", "Backend Refactoring")
            .description("Refactor backend services")
            .meta(json!({"icon": "gear", "risk": "medium"}));

        assert_eq!(option.value, "backend");
        assert_eq!(option.label, "Backend Refactoring");
        assert!(option.description.is_some());
        assert!(option.meta.is_some());
    }

    #[test]
    fn test_elicitation_response_with_metadata() {
        let response = ElicitationResponse::string("req1", "my-choice")
            .meta(json!({"timestamp": "2024-01-12T00:00:00Z"}));

        assert_eq!(response.id, "req1");
        assert!(response.meta.is_some());
    }

    #[test]
    fn test_all_elicitation_types_serialize() {
        for elicitation_type in [
            ElicitationType::Text,
            ElicitationType::Number,
            ElicitationType::Select,
            ElicitationType::Multiselect,
            ElicitationType::Boolean,
            ElicitationType::Password,
            ElicitationType::Url,
        ] {
            let json_str = serde_json::to_string(&elicitation_type).expect("serialize");
            let deserialized: ElicitationType = serde_json::from_str(&json_str).expect("deserialize");
            assert_eq!(elicitation_type, deserialized);
        }
    }

    #[test]
    fn test_multiselect_response() {
        let response = ElicitationResponse::array(
            "multi-req",
            vec!["option1".to_string(), "option2".to_string(), "option3".to_string()],
        );

        let array = response.value.as_array().expect("should be array");
        assert_eq!(array.len(), 3);
        assert_eq!(array[0], json!("option1"));
    }

    #[test]
    fn test_optional_fields_skip_serialization() {
        let request = ElicitationRequest::new(
            "minimal",
            ElicitationType::Text,
            "Enter text",
            ElicitationSchema::new("string", None),
        );

        let json = serde_json::to_string(&request).expect("serialize");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse json");

        assert!(value.get("description").is_none());
        assert!(value.get("options").is_none());
        assert!(value.get("url").is_none());
        assert!(value.get("meta").is_none());
    }

    #[test]
    fn test_capability_with_custom_types() {
        let capability = ElicitationCapability::new(
            true,
            vec!["text".to_string(), "select".to_string()],
        )
        .meta(json!({"version": "1.0"}));

        assert!(capability.supported);
        assert_eq!(capability.supported_types.len(), 2);
        assert!(capability.meta.is_some());
    }
}
