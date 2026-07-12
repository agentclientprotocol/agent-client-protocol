use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{IntoOption, Meta, RequestId};

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Notification to cancel an ongoing request.
///
/// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/cancellation)
#[cfg(feature = "unstable_cancel_request")]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "protocol", "x-method" = CANCEL_REQUEST_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CancelRequestNotification {
    /// The ID of the request to cancel.
    pub request_id: RequestId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_cancel_request")]
impl CancelRequestNotification {
    #[must_use]
    pub fn new(request_id: impl Into<RequestId>) -> Self {
        Self {
            request_id: request_id.into(),
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

// Method schema

/// Names of all methods that agents handle.
///
/// Provides a centralized definition of method names used in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct GeneralMethodNames {
    #[cfg(feature = "unstable_cancel_request")]
    pub cancel_request: &'static str,
}

/// Constant containing all agent method names.
pub const PROTOCOL_LEVEL_METHOD_NAMES: GeneralMethodNames = GeneralMethodNames {
    #[cfg(feature = "unstable_cancel_request")]
    cancel_request: CANCEL_REQUEST_METHOD_NAME,
};

/// Method name for general cancel notification
pub(crate) const CANCEL_REQUEST_METHOD_NAME: &str = "$/cancel_request";

/// General protocol-level notifications that all sides are expected to
/// implement.
///
/// Notifications whose methods start with '$/' are messages which
/// are protocol implementation dependent and might not be implementable in all
/// clients or agents. For example if the implementation uses a single threaded
/// synchronous programming language then there is little it can do to react to
/// a `$/cancel_request` notification. If an agent or client receives
/// notifications starting with '$/' it is free to ignore the notification.
///
/// Notifications do not expect a response.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[non_exhaustive]
pub enum ProtocolLevelNotification {
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or
    /// changed at any point.
    ///
    /// Cancels an ongoing request.
    ///
    /// This is a notification sent by the side that sent a request to cancel that request.
    ///
    /// Upon receiving this notification, the receiver:
    ///
    /// 1. MUST cancel the corresponding request activity and all nested activities
    /// 2. MAY send any pending notifications.
    /// 3. MUST send one of these responses for the original request:
    ///   - Valid response with appropriate data (partial results or cancellation marker)
    ///   - Error response with code `-32800` (Cancelled)
    ///
    /// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/cancellation)
    #[cfg(feature = "unstable_cancel_request")]
    CancelRequestNotification(CancelRequestNotification),
}

impl ProtocolLevelNotification {
    /// Returns the corresponding method name of the notification.
    #[must_use]
    pub fn method(&self) -> &str {
        match self {
            #[cfg(feature = "unstable_cancel_request")]
            Self::CancelRequestNotification(..) => PROTOCOL_LEVEL_METHOD_NAMES.cancel_request,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::RequestId;
    use serde_json::json;

    #[test]
    fn cancel_request_method_name_is_dollar_namespace() {
        // `$/cancel_request` is the protocol-level method name that all sides
        // are expected to recognize (or safely ignore). Renaming this is a
        // wire-format break.
        assert_eq!(CANCEL_REQUEST_METHOD_NAME, "$/cancel_request");
        assert_eq!(
            PROTOCOL_LEVEL_METHOD_NAMES.cancel_request,
            CANCEL_REQUEST_METHOD_NAME
        );
    }

    #[test]
    fn protocol_level_notification_method_returns_cancel_request_name() {
        let notif = ProtocolLevelNotification::CancelRequestNotification(
            CancelRequestNotification::new(RequestId::Number(7)),
        );
        assert_eq!(notif.method(), CANCEL_REQUEST_METHOD_NAME);
    }

    #[test]
    fn cancel_request_notification_roundtrip_omits_meta_when_unset() {
        let notif = CancelRequestNotification::new(RequestId::Str("req_1".to_string()));
        let serialized = serde_json::to_value(&notif).unwrap();
        assert_eq!(serialized, json!({"requestId": "req_1"}));

        let parsed: CancelRequestNotification = serde_json::from_value(serialized).unwrap();
        assert_eq!(parsed, notif);
    }

    #[test]
    fn cancel_request_notification_preserves_meta_when_set() {
        let mut meta = Meta::new();
        meta.insert("trace".to_string(), json!("abc123"));
        let notif = CancelRequestNotification::new(RequestId::Number(42)).meta(meta.clone());

        let serialized = serde_json::to_value(&notif).unwrap();
        assert_eq!(
            serialized,
            json!({"requestId": 42, "_meta": {"trace": "abc123"}})
        );

        let parsed: CancelRequestNotification = serde_json::from_value(serialized).unwrap();
        assert_eq!(parsed.meta, Some(meta));
    }

    #[test]
    fn cancel_request_accepts_all_request_id_shapes() {
        // RequestId is untagged: null / number / string. The cancellation
        // notification must round-trip any of them so cancellation works
        // regardless of how the original request was issued.
        let cases = [
            (RequestId::Null, json!(null)),
            (RequestId::Number(0), json!(0)),
            (RequestId::Number(-1), json!(-1)),
            (RequestId::Str("hex-id".to_string()), json!("hex-id")),
        ];
        for (id, expected_id_json) in cases {
            let notif = CancelRequestNotification::new(id.clone());
            let serialized = serde_json::to_value(&notif).unwrap();
            assert_eq!(serialized["requestId"], expected_id_json, "id: {id:?}");
            let parsed: CancelRequestNotification = serde_json::from_value(serialized).unwrap();
            assert_eq!(parsed.request_id, id);
        }
    }
}
