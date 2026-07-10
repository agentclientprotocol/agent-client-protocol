//! Extension types and constants for protocol extensibility.
use derive_more::From;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::sync::Arc;

/// Value attached to a given ACP type on the `_meta` field.
///
/// The _meta property is reserved by ACP to allow clients and agents to attach
/// additional metadata to their interactions. Implementations MUST NOT make assumptions about
/// values at these keys.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
pub type Meta = serde_json::Map<String, serde_json::Value>;

/// Allows for sending an arbitrary request that is not part of the ACP spec.
/// Extension methods provide a way to add custom functionality while maintaining
/// protocol compatibility.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ExtRequest {
    /// Wire method name for this extension request.
    ///
    /// Extension method names must start with `_`.
    #[serde(skip)] // this is used for routing, but when serializing we only want the params
    pub method: Arc<str>,
    #[schemars(with = "serde_json::Value")]
    pub params: Arc<RawValue>,
}

impl ExtRequest {
    #[must_use]
    pub fn new(method: impl Into<Arc<str>>, params: Arc<RawValue>) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

/// Allows for sending an arbitrary response to an [`ExtRequest`] that is not part of the ACP spec.
/// Extension methods provide a way to add custom functionality while maintaining
/// protocol compatibility.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, From)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ExtResponse(#[schemars(with = "serde_json::Value")] pub Arc<RawValue>);

impl ExtResponse {
    #[must_use]
    pub fn new(params: Arc<RawValue>) -> Self {
        Self(params)
    }
}

/// Allows the Agent to send an arbitrary notification that is not part of the ACP spec.
/// Extension notifications provide a way to send one-way messages for custom functionality
/// while maintaining protocol compatibility.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ExtNotification {
    /// Wire method name for this extension notification.
    ///
    /// Extension method names must start with `_`.
    #[serde(skip)] // this is used for routing, but when serializing we only want the params
    pub method: Arc<str>,
    #[schemars(with = "serde_json::Value")]
    pub params: Arc<RawValue>,
}

impl ExtNotification {
    #[must_use]
    pub fn new(method: impl Into<Arc<str>>, params: Arc<RawValue>) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::value::RawValue;

    fn raw(s: &str) -> Arc<RawValue> {
        RawValue::from_string(s.to_string()).unwrap().into()
    }

    #[test]
    fn ext_request_new_preserves_method_verbatim() {
        let req = ExtRequest::new("_vendor/custom_action", raw(r#"{"x":1}"#));
        assert_eq!(req.method.as_ref(), "_vendor/custom_action");

        let req = ExtRequest::new(String::from("_other"), raw("null"));
        assert_eq!(req.method.as_ref(), "_other");

        let arc: Arc<str> = Arc::from("_keep");
        let req = ExtRequest::new(arc.clone(), raw("[]"));
        assert!(Arc::ptr_eq(&req.method, &arc), "Arc<str> should be reused");
    }

    #[test]
    fn ext_request_serializes_only_params_not_method() {
        let req = ExtRequest::new("_vendor/x", raw(r#"{"answer":42}"#));
        let serialized = serde_json::to_value(&req).unwrap();
        assert_eq!(serialized, serde_json::json!({"answer": 42}));
        assert!(
            !serialized.to_string().contains("_vendor/x"),
            "method name must not leak into the serialized params"
        );
    }

    #[test]
    fn ext_notification_serializes_only_params_not_method() {
        let note = ExtNotification::new("_telemetry/event", raw(r#"{"k":"v"}"#));
        let serialized = serde_json::to_value(&note).unwrap();
        assert_eq!(serialized, serde_json::json!({"k": "v"}));
    }

    #[test]
    fn ext_response_serializes_transparently() {
        let resp = ExtResponse::new(raw(r#"{"ok":true}"#));
        let serialized = serde_json::to_value(&resp).unwrap();
        assert_eq!(serialized, serde_json::json!({"ok": true}));

        let from_impl: ExtResponse = raw("42").into();
        let serialized = serde_json::to_value(&from_impl).unwrap();
        assert_eq!(serialized, serde_json::json!(42));
    }
}
