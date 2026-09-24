//! Shared unstable MCP-over-ACP response carrier.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::{DefaultOnError, serde_as};

use crate::{IntoOption, MaybeUndefined};

/// **UNSTABLE**
///
/// An inner MCP error, distinct from an outer ACP binding or runtime error.
///
/// `code` and `message` are required and non-null. `data` is optional;
/// explicit `null` is preserved separately from an omitted key.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct McpError {
    /// Inner MCP error code; never an ACP error code.
    pub code: i32,
    /// Inner MCP error message.
    pub message: String,
    /// Optional error data; explicit null is retained.
    #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
    pub data: MaybeUndefined<Value>,
    /// Additional fields on the inner MCP error object.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl McpError {
    /// Construct an inner MCP error without data.
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: MaybeUndefined::Undefined,
            extra: Map::new(),
        }
    }

    /// Set data, preserving explicit JSON null.
    #[must_use]
    pub fn data(mut self, data: Value) -> Self {
        self.data = if data.is_null() {
            MaybeUndefined::Null
        } else {
            MaybeUndefined::Value(data)
        };
        self
    }
}

/// **UNSTABLE**
///
/// The successful outer ACP `mcp/message` response carries exactly one
/// inner MCP outcome: an opaque result (including JSON null), or an MCP error.
/// Outer ACP errors are reserved for binding and runtime failures.
///
/// Both branches require their carrier key. An error must be a non-null object.
/// Carrier `_meta` is optional; null is equivalent to omission.
#[serde_as]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged, deny_unknown_fields)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "client", "x-method" = "mcp/message")))]
#[non_exhaustive]
pub enum MessageMcpResponse {
    /// An opaque inner MCP result.
    Result {
        /// Required, even if JSON null.
        result: Value,
        /// Optional ACP carrier metadata.
        #[serde_as(deserialize_as = "DefaultOnError")]
        #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
        #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
        meta: Option<Map<String, Value>>,
    },
    /// A structured inner MCP error.
    Error {
        /// Required, non-null MCP error object.
        error: McpError,
        /// Optional ACP carrier metadata.
        #[serde_as(deserialize_as = "DefaultOnError")]
        #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
        #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
        meta: Option<Map<String, Value>>,
    },
}

impl MessageMcpResponse {
    /// Wrap any JSON result without interpreting its MCP result type.
    #[must_use]
    pub fn success(result: Value) -> Self {
        Self::Result { result, meta: None }
    }

    /// Wrap an inner MCP error in a successful outer ACP response.
    #[must_use]
    pub fn error(error: McpError) -> Self {
        Self::Error { error, meta: None }
    }

    /// Attach optional carrier-level ACP metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Map<String, Value>>) -> Self {
        match &mut self {
            Self::Result { meta: field, .. } | Self::Error { meta: field, .. } => {
                *field = meta.into_option();
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{McpError, MessageMcpResponse};
    use crate::MaybeUndefined;

    #[test]
    fn result_is_opaque_and_present_even_when_null() {
        for result in [
            Value::Null,
            json!(false),
            json!(42),
            json!("opaque"),
            json!([null, 1]),
            json!({"resultType": "future", "unknown": {"value": true}}),
        ] {
            let response = MessageMcpResponse::success(result.clone());
            let wire = json!({"result": result});
            assert_eq!(serde_json::to_value(&response).unwrap(), wire);
            assert_eq!(
                serde_json::from_value::<MessageMcpResponse>(wire).unwrap(),
                response
            );
        }
    }

    #[test]
    fn error_round_trips_data_and_extensions_without_acp_translation() {
        for data in [
            MaybeUndefined::Undefined,
            MaybeUndefined::Null,
            MaybeUndefined::Value(json!({"arbitrary": [1, null]})),
        ] {
            let mut error = McpError::new(-32000, "inner error");
            error.data = data.clone();
            error.extra.insert("future".into(), json!({"key": 1}));
            let response = MessageMcpResponse::error(error);
            let wire = serde_json::to_value(&response).unwrap();
            assert_eq!(wire["error"]["code"], -32000);
            assert_eq!(wire["error"].get("data").is_some(), !data.is_undefined());
            assert_eq!(wire["error"]["future"], json!({"key": 1}));
            assert_eq!(
                serde_json::from_value::<MessageMcpResponse>(wire).unwrap(),
                response
            );
        }
        assert_eq!(
            McpError::new(1, "x").data(Value::Null).data,
            MaybeUndefined::Null
        );
    }

    #[test]
    fn only_one_non_null_carrier_key_is_valid() {
        for wire in [
            Value::Null,
            json!({}),
            json!({"_meta": null}),
            json!({"result": 1, "error": {"code": 1, "message": "x"}}),
            json!({"result": 1, "error": null}),
            json!({"error": null}),
            json!({"error": 1}),
            json!({"error": {}}),
            json!({"error": {"code": null, "message": "x"}}),
            json!({"error": {"code": 1, "message": null}}),
            json!({"error": {"code": 1.5, "message": "x"}}),
            json!({"unexpected": 1, "result": 1}),
        ] {
            assert!(
                serde_json::from_value::<MessageMcpResponse>(wire.clone()).is_err(),
                "accepted {wire}"
            );
        }
    }

    #[test]
    fn carrier_metadata_is_optional_and_null_means_absent() {
        for wire in [
            json!({"result": null, "_meta": null}),
            json!({"error": {"code": 1, "message": "x"}, "_meta": null}),
        ] {
            let parsed: MessageMcpResponse = serde_json::from_value(wire).unwrap();
            assert!(serde_json::to_value(parsed).unwrap().get("_meta").is_none());
        }
        let meta = json!({"extension": [null, true]})
            .as_object()
            .unwrap()
            .clone();
        let response =
            MessageMcpResponse::success(json!({"_meta": {"inner": true}})).meta(meta.clone());
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({"result": {"_meta": {"inner": true}}, "_meta": meta})
        );
    }

    #[cfg(feature = "unstable_protocol_v2")]
    #[test]
    fn both_versions_export_the_same_types() {
        let response: crate::v1::MessageMcpResponse =
            crate::v2::MessageMcpResponse::error(crate::v2::McpError::new(-32000, "x"));
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({"error": {"code": -32000, "message": "x"}})
        );
    }
}
