//! Elicitation types for structured user input.
//!
//! **UNSTABLE**: This module is not part of the spec yet, and may be removed or changed at any point.
//!
//! This module defines the types used for agent-initiated elicitation,
//! where the agent requests structured input from the user via forms or URLs.

use std::sync::Arc;

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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ElicitationFormMode {
    /// A JSON Schema describing the form fields to present to the user.
    pub requested_schema: serde_json::Value,
}

impl ElicitationFormMode {
    #[must_use]
    pub fn new(requested_schema: serde_json::Value) -> Self {
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
        let req = ElicitationRequest::new(
            "sess_1",
            ElicitationMode::Form(ElicitationFormMode::new(
                json!({"type": "object", "properties": {"name": {"type": "string"}}}),
            )),
            "Please enter your name",
        );

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["sessionId"], "sess_1");
        assert_eq!(json["mode"], "form");
        assert_eq!(json["message"], "Please enter your name");
        assert!(json["requestedSchema"].is_object());

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
}
