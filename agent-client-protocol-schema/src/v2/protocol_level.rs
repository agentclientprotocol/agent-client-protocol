use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, serde_as, skip_serializing_none};

use super::{Meta, RequestId};
use crate::IntoOption;

/// Notification to cancel an ongoing request.
///
/// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/v2/cancellation)
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "protocol", "x-method" = CANCEL_REQUEST_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CancelRequestNotification {
    /// The ID of the request to cancel.
    pub request_id: RequestId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Omitted and `null` are equivalent and mean no metadata.
    /// Implementations MUST NOT make assumptions about values at these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/v2/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl CancelRequestNotification {
    /// Builds [`CancelRequestNotification`] with the required notification fields set; optional fields start unset or empty.
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
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/v2/extensibility)
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
    /// Method name for protocol-level request cancellation notifications.
    pub cancel_request: &'static str,
}

/// Constant containing all agent method names.
pub const PROTOCOL_LEVEL_METHOD_NAMES: GeneralMethodNames = GeneralMethodNames {
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schemars", schemars(inline))]
#[non_exhaustive]
pub enum ProtocolLevelNotification {
    /// Cancels an ongoing request.
    ///
    /// This is a notification sent by the side that sent a request to cancel that request.
    ///
    /// Upon receiving this notification, the receiver:
    ///
    /// 1. MAY cancel the corresponding request activity and all nested activities
    /// 2. MAY send any pending notifications.
    /// 3. MUST still send a response for the original request. If the request already finished
    ///    or is not cancelled, that is its normal response. If it is cancelled, it is either the
    ///    method's defined cancellation result (for example an inserted `session/prompt`, or a
    ///    `cancelled` permission outcome) or an error with code `-32800` (Request Cancelled).
    ///
    /// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/v2/cancellation)
    CancelRequestNotification(CancelRequestNotification),
}

impl ProtocolLevelNotification {
    /// Returns the corresponding method name of the notification.
    #[must_use]
    pub fn method(&self) -> &str {
        match self {
            Self::CancelRequestNotification(..) => PROTOCOL_LEVEL_METHOD_NAMES.cancel_request,
        }
    }
}
