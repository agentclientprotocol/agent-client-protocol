//! Proxy chain types: messages exchanged between a conductor and the proxies it manages.
//!
//! Ordinary clients and agents never send or receive these messages. See the [Proxy Chains RFD](https://agentclientprotocol.com/rfds/proxy-chains).

use std::sync::Arc;

use derive_more::From;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_with::{DefaultOnError, serde_as, skip_serializing_none};

use super::{InitializeRequest, InitializeResponse, Meta};
use crate::IntoOption;

// Initialize proxy

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for `proxy/initialize`.
///
/// A conductor sends `proxy/initialize` instead of `initialize` to every component that has a
/// successor, which tells the component to run as a proxy. The params are the same as
/// `initialize`, and the response is an ordinary `InitializeResponse`. A conductor sends
/// `initialize` to the final agent.
///
/// See the [Proxy Chains RFD](https://agentclientprotocol.com/rfds/proxy-chains).
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "proxy", "x-method" = PROXY_INITIALIZE_METHOD_NAME)))]
#[non_exhaustive]
pub struct InitializeProxyRequest {
    /// The `initialize` params, flattened into this request.
    #[serde(flatten)]
    pub initialize: InitializeRequest,
}

impl InitializeProxyRequest {
    /// Builds [`InitializeProxyRequest`] from the `initialize` params to send.
    #[must_use]
    pub fn new(initialize: InitializeRequest) -> Self {
        Self { initialize }
    }
}

impl From<InitializeRequest> for InitializeProxyRequest {
    fn from(initialize: InitializeRequest) -> Self {
        Self::new(initialize)
    }
}

// Successor messages

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Parameters of a `proxy/successor` request.
///
/// A proxy sends it to forward a message to its successor, and the conductor sends it to
/// deliver a message from the successor to the proxy. The envelope wraps one request: `method`
/// and `params` are the wrapped request's method and params.
///
/// The result or error of the wrapped request is returned unchanged as the result or error
/// of the `proxy/successor` request.
///
/// See the [Proxy Chains RFD](https://agentclientprotocol.com/rfds/proxy-chains).
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "proxy", "x-method" = PROXY_SUCCESSOR_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ProxySuccessorRequest {
    /// Method name of the wrapped request.
    pub method: Arc<str>,
    /// Params of the wrapped request, forwarded unchanged.
    ///
    /// Optional. Omitted and `null` are equivalent and mean the wrapped request has no params.
    /// The wrapped request's own `_meta` stays inside these params.
    #[cfg_attr(feature = "schemars", schemars(with = "Option<serde_json::Value>"))]
    #[serde(default)]
    pub params: Option<Arc<RawValue>>,
    /// Metadata for the `proxy/successor` envelope itself, separate from the wrapped
    /// message's `_meta`.
    ///
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/v1/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ProxySuccessorRequest {
    /// Builds [`ProxySuccessorRequest`] with the required fields set. Optional fields start unset or empty.
    #[must_use]
    pub fn new(method: impl Into<Arc<str>>) -> Self {
        Self {
            method: method.into(),
            params: None,
            meta: None,
        }
    }

    /// Params of the wrapped request, forwarded unchanged.
    #[must_use]
    pub fn params(mut self, params: impl IntoOption<Arc<RawValue>>) -> Self {
        self.params = params.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/v1/extensibility)
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
/// Parameters of a `proxy/successor` notification.
///
/// A proxy sends it to forward a message to its successor, and the conductor sends it to
/// deliver a message from the successor to the proxy. The envelope wraps one notification: `method`
/// and `params` are the wrapped notification's method and params.
///
/// See the [Proxy Chains RFD](https://agentclientprotocol.com/rfds/proxy-chains).
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "proxy", "x-method" = PROXY_SUCCESSOR_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ProxySuccessorNotification {
    /// Method name of the wrapped notification.
    pub method: Arc<str>,
    /// Params of the wrapped notification, forwarded unchanged.
    ///
    /// Optional. Omitted and `null` are equivalent and mean the wrapped notification has no params.
    /// The wrapped notification's own `_meta` stays inside these params.
    #[cfg_attr(feature = "schemars", schemars(with = "Option<serde_json::Value>"))]
    #[serde(default)]
    pub params: Option<Arc<RawValue>>,
    /// Metadata for the `proxy/successor` envelope itself, separate from the wrapped
    /// message's `_meta`.
    ///
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/v1/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ProxySuccessorNotification {
    /// Builds [`ProxySuccessorNotification`] with the required fields set. Optional fields start unset or empty.
    #[must_use]
    pub fn new(method: impl Into<Arc<str>>) -> Self {
        Self {
            method: method.into(),
            params: None,
            meta: None,
        }
    }

    /// Params of the wrapped notification, forwarded unchanged.
    #[must_use]
    pub fn params(mut self, params: impl IntoOption<Arc<RawValue>>) -> Self {
        self.params = params.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/v1/extensibility)
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
/// Result of a `proxy/successor` request: the wrapped request's result, forwarded unchanged.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ProxySuccessorResponse(
    #[cfg_attr(feature = "schemars", schemars(with = "serde_json::Value"))] pub Arc<RawValue>,
);

impl ProxySuccessorResponse {
    /// Wraps the successor's result.
    #[must_use]
    pub fn new(result: Arc<RawValue>) -> Self {
        Self(result)
    }
}

// Method schema

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Names of the methods exchanged between a conductor and its proxies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProxyMethodNames {
    /// Method for initializing a component as a proxy.
    pub proxy_initialize: &'static str,
    /// Method for forwarding a message to or from a proxy's successor.
    pub proxy_successor: &'static str,
}

/// Constant containing all proxy chain method names.
pub const PROXY_METHOD_NAMES: ProxyMethodNames = ProxyMethodNames {
    proxy_initialize: PROXY_INITIALIZE_METHOD_NAME,
    proxy_successor: PROXY_SUCCESSOR_METHOD_NAME,
};

/// Method name for initializing a component as a proxy.
pub(crate) const PROXY_INITIALIZE_METHOD_NAME: &str = "proxy/initialize";
/// Method name for forwarding a message to or from a proxy's successor.
pub(crate) const PROXY_SUCCESSOR_METHOD_NAME: &str = "proxy/successor";

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Requests exchanged between a conductor and its proxies.
///
/// A conductor sends `proxy/initialize` and `proxy/successor` to a proxy. A proxy sends
/// `proxy/successor` to its conductor.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(untagged)]
#[cfg_attr(feature = "schemars", schemars(inline))]
#[non_exhaustive]
pub enum ProxyRequest {
    /// Initializes a component as a proxy.
    ///
    /// Sent by a conductor to each component that has a successor, instead of `initialize`.
    /// The params and response are the same as for `initialize`.
    InitializeProxyRequest(InitializeProxyRequest),
    /// Forwards a request to or from a proxy's successor.
    ///
    /// A proxy sends it to its conductor to forward a request downstream. The conductor sends
    /// it to the proxy to deliver a request from the successor. The wrapped request's result
    /// or error is returned unchanged.
    ProxySuccessorRequest(ProxySuccessorRequest),
}

impl ProxyRequest {
    /// Returns the corresponding method name of the request.
    #[must_use]
    pub fn method(&self) -> &str {
        match self {
            Self::InitializeProxyRequest(_) => PROXY_METHOD_NAMES.proxy_initialize,
            Self::ProxySuccessorRequest(_) => PROXY_METHOD_NAMES.proxy_successor,
        }
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Responses to requests exchanged between a conductor and its proxies.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(untagged)]
#[cfg_attr(feature = "schemars", schemars(inline))]
#[non_exhaustive]
pub enum ProxyResponse {
    /// Successful result returned for a `proxy/initialize` request.
    InitializeResponse(InitializeResponse),
    /// Successful result returned for a `proxy/successor` request.
    ProxySuccessorResponse(ProxySuccessorResponse),
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Notifications exchanged between a conductor and its proxies.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schemars", schemars(inline))]
#[non_exhaustive]
pub enum ProxyNotification {
    /// Forwards a notification to or from a proxy's successor.
    ///
    /// A proxy sends it to its conductor to forward a notification downstream. The conductor
    /// sends it to the proxy to deliver a notification from the successor.
    ProxySuccessorNotification(ProxySuccessorNotification),
}

impl ProxyNotification {
    /// Returns the corresponding method name of the notification.
    #[must_use]
    pub fn method(&self) -> &str {
        match self {
            Self::ProxySuccessorNotification(_) => PROXY_METHOD_NAMES.proxy_successor,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::ProtocolVersion;

    fn raw(value: &serde_json::Value) -> Arc<RawValue> {
        RawValue::from_string(value.to_string()).unwrap().into()
    }

    #[test]
    fn test_initialize_proxy_request_flattens_initialize_params() {
        let request = InitializeProxyRequest::new(InitializeRequest::new(ProtocolVersion::V1));
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value, serde_json::to_value(&request.initialize).unwrap());
        assert_eq!(
            ProxyRequest::InitializeProxyRequest(request).method(),
            "proxy/initialize"
        );
        let parsed: InitializeProxyRequest = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.initialize.protocol_version, ProtocolVersion::V1);
    }

    #[test]
    fn test_proxy_successor_round_trip() {
        let request = ProxySuccessorRequest::new("session/prompt")
            .params(raw(
                &json!({"sessionId": "sess_abc123", "_meta": {"inner": true}}),
            ))
            .meta(json!({"outer": true}).as_object().cloned());
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            value,
            json!({
                "method": "session/prompt",
                "params": {"sessionId": "sess_abc123", "_meta": {"inner": true}},
                "_meta": {"outer": true}
            })
        );
        assert_eq!(
            ProxyRequest::ProxySuccessorRequest(request).method(),
            "proxy/successor"
        );

        for params in [
            json!({"method": "session/cancel"}),
            json!({"method": "session/cancel", "params": null}),
        ] {
            let notification: ProxySuccessorNotification = serde_json::from_value(params).unwrap();
            assert_eq!(&*notification.method, "session/cancel");
            assert!(notification.params.is_none());
        }
        assert!(
            serde_json::from_value::<ProxySuccessorNotification>(json!({"params": {}})).is_err()
        );

        let response = ProxySuccessorResponse::new(raw(&json!({"stopReason": "end_turn"})));
        assert_eq!(
            serde_json::to_value(&response).unwrap(),
            json!({"stopReason": "end_turn"})
        );
    }
}
