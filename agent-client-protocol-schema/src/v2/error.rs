//! Error handling for the Agent Client Protocol.
//!
//! This module provides error types and codes following the JSON-RPC 2.0 specification,
//! with additional protocol-specific error codes for authentication and other ACP-specific scenarios.
//!
//! All methods in the protocol follow standard JSON-RPC 2.0 error handling:
//! - Successful responses include a `result` field
//! - Errors include an `error` object with `code` and `message`
//! - Notifications never receive responses (success or error)
//!
//! See: [Error Handling](https://agentclientprotocol.com/protocol/v2/overview#error-handling)

use std::{fmt::Display, str};

#[cfg(feature = "schemars")]
use schemars::Schema;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, serde_as, skip_serializing_none};

use super::{Meta, SessionId};
use crate::{IntoOption, ProtocolVersion};

/// Convenience result type using this protocol version's error type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// JSON-RPC error object.
///
/// Represents an error that occurred during method execution, following the
/// JSON-RPC 2.0 error object specification with optional additional data.
///
/// See protocol docs: [JSON-RPC Error Object](https://www.jsonrpc.org/specification#error_object)
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct Error {
    /// A number indicating the error type that occurred.
    /// This must be an integer as defined in the JSON-RPC specification.
    pub code: ErrorCode,
    /// A string providing a short description of the error.
    /// The message should be limited to a concise single sentence.
    pub message: String,
    /// Optional primitive or structured value that contains additional information about the error.
    /// This may include debugging information or context-specific details.
    ///
    /// Omitted and `null` are equivalent and mean no additional data is provided. Some codes
    /// define a shape: [`UnsupportedProtocolVersionErrorData`] for `-32022`, and
    /// [`SessionNotFoundErrorData`] for `-32002` when a session is not found.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(with = "Option<ErrorData>", extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// Schema for [`Error::data`]: the shapes ACP defines, or any other value.
#[cfg(feature = "schemars")]
#[derive(schemars::JsonSchema)]
#[serde(untagged)]
#[expect(
    dead_code,
    reason = "Only used to describe the schema of `Error::data`"
)]
enum ErrorData {
    UnsupportedProtocolVersion(UnsupportedProtocolVersionErrorData),
    SessionNotFound(SessionNotFoundErrorData),
    Other(serde_json::Value),
}

/// Data for the `-32022` (Unsupported protocol version) error.
///
/// See protocol docs: [Version Negotiation](https://agentclientprotocol.com/protocol/v2/initialization#version-negotiation)
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UnsupportedProtocolVersionErrorData {
    /// The protocol version the Client requested.
    pub requested: ProtocolVersion,
    /// The protocol versions the Agent supports.
    pub supported: Vec<ProtocolVersion>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Omitted and `null` are equivalent and mean no metadata.
    /// Implementations MUST NOT make assumptions about values at these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/v2/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, rename = "_meta")]
    pub meta: Option<Meta>,
}

impl UnsupportedProtocolVersionErrorData {
    /// Builds the data with the requested and supported versions.
    #[must_use]
    pub fn new(requested: ProtocolVersion, supported: Vec<ProtocolVersion>) -> Self {
        Self {
            requested,
            supported,
            meta: None,
        }
    }

    /// Sets optional metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Data for the `-32002` (Resource not found) error when a session-scoped request names a
/// session that does not exist or is not active.
///
/// See protocol docs: [Unknown Sessions](https://agentclientprotocol.com/protocol/v2/error#unknown-sessions)
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionNotFoundErrorData {
    /// The session ID from the request.
    pub session_id: SessionId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Omitted and `null` are equivalent and mean no metadata.
    /// Implementations MUST NOT make assumptions about values at these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/v2/extensibility)
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionNotFoundErrorData {
    /// Builds the data for the given session ID.
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>) -> Self {
        Self {
            session_id: session_id.into(),
            meta: None,
        }
    }

    /// Sets optional metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

impl Error {
    /// Creates a new error with the given code and message.
    ///
    /// The code parameter can be an `ErrorCode` constant or a tuple of (code, message).
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Error {
            code: code.into(),
            message: message.into(),
            data: None,
        }
    }

    /// Adds additional data to the error.
    ///
    /// This method is chainable and allows attaching context-specific information
    /// to help with debugging or provide more details about the error.
    #[must_use]
    pub fn data(mut self, data: impl IntoOption<serde_json::Value>) -> Self {
        self.data = data.into_option();
        self
    }

    /// Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
    #[must_use]
    pub fn parse_error() -> Self {
        ErrorCode::ParseError.into()
    }

    /// The JSON sent is not a valid Request object.
    #[must_use]
    pub fn invalid_request() -> Self {
        ErrorCode::InvalidRequest.into()
    }

    /// The method does not exist / is not available.
    #[must_use]
    pub fn method_not_found() -> Self {
        ErrorCode::MethodNotFound.into()
    }

    /// Invalid method parameter(s).
    #[must_use]
    pub fn invalid_params() -> Self {
        ErrorCode::InvalidParams.into()
    }

    /// Internal JSON-RPC error.
    #[must_use]
    pub fn internal_error() -> Self {
        ErrorCode::InternalError.into()
    }

    /// Request was cancelled.
    ///
    /// Execution of the method was aborted either due to a cancellation request from the caller
    /// or because of resource constraints or shutdown.
    #[must_use]
    pub fn request_cancelled() -> Self {
        ErrorCode::RequestCancelled.into()
    }

    /// Authentication required.
    #[must_use]
    pub fn auth_required() -> Self {
        ErrorCode::AuthRequired.into()
    }

    /// A referenced resource was not found.
    #[must_use]
    pub fn resource_not_found() -> Self {
        ErrorCode::ResourceNotFound.into()
    }

    /// The session named by a session-scoped request does not exist or is not active.
    #[must_use]
    pub fn session_not_found(session_id: impl Into<super::SessionId>) -> Self {
        Error::from(ErrorCode::ResourceNotFound)
            .data(serde_json::to_value(SessionNotFoundErrorData::new(session_id)).ok())
    }

    /// No supported protocol version is at or below the requested version.
    #[must_use]
    pub fn unsupported_protocol_version(
        requested: ProtocolVersion,
        supported: &[ProtocolVersion],
    ) -> Self {
        let error: Self = ErrorCode::UnsupportedProtocolVersion.into();
        error.data(
            serde_json::to_value(UnsupportedProtocolVersionErrorData::new(
                requested,
                supported.to_vec(),
            ))
            .ok(),
        )
    }

    /// Converts a standard error into an internal JSON-RPC error.
    ///
    /// The error's string representation is included as additional data.
    #[must_use]
    pub fn into_internal_error(err: impl std::error::Error) -> Self {
        Error::internal_error().data(err.to_string())
    }
}

/// Predefined error codes for common JSON-RPC and ACP-specific errors.
///
/// Standard errors use the JSON-RPC 2.0 codes. ACP-specific errors use the JSON-RPC
/// server-error range (-32000 to -32099), and request cancellation uses -32800 from the
/// range LSP reserves (-32899 to -32800).
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize, strum::Display)]
#[cfg_attr(test, derive(strum::EnumIter))]
#[serde(from = "i32", into = "i32")]
#[cfg_attr(feature = "schemars", schemars(!from, !into))]
#[non_exhaustive]
pub enum ErrorCode {
    // JSON-RPC errors
    /// Invalid JSON was received by the server.
    /// An error occurred on the server while parsing the JSON text.
    #[cfg_attr(feature = "schemars", schemars(transform = error_code_transform))]
    #[strum(to_string = "Parse error")]
    ParseError, // -32700
    /// The JSON sent is not a valid Request object.
    #[cfg_attr(feature = "schemars", schemars(transform = error_code_transform))]
    #[strum(to_string = "Invalid request")]
    InvalidRequest, // -32600
    /// The method does not exist or is not available.
    #[cfg_attr(feature = "schemars", schemars(transform = error_code_transform))]
    #[strum(to_string = "Method not found")]
    MethodNotFound, // -32601
    /// Invalid method parameter(s).
    #[cfg_attr(feature = "schemars", schemars(transform = error_code_transform))]
    #[strum(to_string = "Invalid params")]
    InvalidParams, // -32602
    /// Internal JSON-RPC error.
    #[cfg_attr(feature = "schemars", schemars(transform = error_code_transform))]
    #[strum(to_string = "Internal error")]
    InternalError, // -32603
    // Request cancellation (from LSP)
    /// Execution of the method was aborted either due to a cancellation request from the caller or
    /// because of resource constraints or shutdown.
    #[cfg_attr(feature = "schemars", schemars(transform = error_code_transform))]
    #[strum(to_string = "Request cancelled")]
    RequestCancelled, // -32800

    // Custom errors
    /// Authentication is required before this operation can be performed.
    #[cfg_attr(feature = "schemars", schemars(transform = error_code_transform))]
    #[strum(to_string = "Authentication required")]
    AuthRequired, // -32000
    /// A given resource, such as a session, was not found.
    #[cfg_attr(feature = "schemars", schemars(transform = error_code_transform))]
    #[strum(to_string = "Resource not found")]
    ResourceNotFound, // -32002
    /// No protocol version supported by the Agent is at or below the requested version.
    /// The error data is an [`UnsupportedProtocolVersionErrorData`].
    #[cfg_attr(feature = "schemars", schemars(transform = error_code_transform))]
    #[strum(to_string = "Unsupported protocol version")]
    UnsupportedProtocolVersion, // -32022
    /// Other undefined error code.
    #[cfg_attr(feature = "schemars", schemars(untagged))]
    #[strum(to_string = "Unknown error")]
    Other(i32),
}

impl From<i32> for ErrorCode {
    fn from(value: i32) -> Self {
        match value {
            -32700 => ErrorCode::ParseError,
            -32600 => ErrorCode::InvalidRequest,
            -32601 => ErrorCode::MethodNotFound,
            -32602 => ErrorCode::InvalidParams,
            -32603 => ErrorCode::InternalError,
            -32800 => ErrorCode::RequestCancelled,
            -32000 => ErrorCode::AuthRequired,
            -32002 => ErrorCode::ResourceNotFound,
            -32022 => ErrorCode::UnsupportedProtocolVersion,
            _ => ErrorCode::Other(value),
        }
    }
}

impl From<ErrorCode> for i32 {
    fn from(value: ErrorCode) -> Self {
        match value {
            ErrorCode::ParseError => -32700,
            ErrorCode::InvalidRequest => -32600,
            ErrorCode::MethodNotFound => -32601,
            ErrorCode::InvalidParams => -32602,
            ErrorCode::InternalError => -32603,
            ErrorCode::RequestCancelled => -32800,
            ErrorCode::AuthRequired => -32000,
            ErrorCode::ResourceNotFound => -32002,
            ErrorCode::UnsupportedProtocolVersion => -32022,
            ErrorCode::Other(value) => value,
        }
    }
}

impl std::fmt::Debug for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {self}", i32::from(*self))
    }
}

#[cfg(feature = "schemars")]
fn error_code_transform(schema: &mut Schema) {
    let name = schema
        .get("const")
        .expect("Unexpected schema for ErrorCode")
        .as_str()
        .expect("unexpected type for schema");
    let code = match name {
        "ParseError" => ErrorCode::ParseError,
        "InvalidRequest" => ErrorCode::InvalidRequest,
        "MethodNotFound" => ErrorCode::MethodNotFound,
        "InvalidParams" => ErrorCode::InvalidParams,
        "InternalError" => ErrorCode::InternalError,
        "RequestCancelled" => ErrorCode::RequestCancelled,
        "AuthRequired" => ErrorCode::AuthRequired,
        "ResourceNotFound" => ErrorCode::ResourceNotFound,
        "UnsupportedProtocolVersion" => ErrorCode::UnsupportedProtocolVersion,
        _ => panic!("Unexpected error code name {name}"),
    };
    let mut description = schema
        .get("description")
        .expect("Missing description")
        .as_str()
        .expect("Unexpected type for description")
        .to_owned();
    schema.insert("title".into(), code.to_string().into());
    description.insert_str(0, &format!("**{code}**: "));
    schema.insert("description".into(), description.into());
    schema.insert("const".into(), i32::from(code).into());
    schema.insert("type".into(), "integer".into());
    schema.insert("format".into(), "int32".into());
}

impl From<ErrorCode> for Error {
    fn from(error_code: ErrorCode) -> Self {
        Error::new(error_code.into(), error_code.to_string())
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.message.is_empty() {
            write!(f, "{}", i32::from(self.code))?;
        } else {
            write!(f, "{}", self.message)?;
        }

        if let Some(data) = &self.data {
            let pretty = serde_json::to_string_pretty(data).unwrap_or_else(|_| data.to_string());
            write!(f, ": {pretty}")?;
        }

        Ok(())
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        match error.downcast::<Self>() {
            Ok(error) => error,
            Err(error) => Error::into_internal_error(&*error),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::invalid_params().data(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    #[test]
    fn helper_data_parses_as_typed_error_data() {
        let error = Error::session_not_found("sess_1");
        let data: SessionNotFoundErrorData = serde_json::from_value(error.data.unwrap()).unwrap();
        assert_eq!(data, SessionNotFoundErrorData::new("sess_1"));

        let error =
            Error::unsupported_protocol_version(ProtocolVersion::V2, &[ProtocolVersion::V1]);
        let data: UnsupportedProtocolVersionErrorData =
            serde_json::from_value(error.data.unwrap()).unwrap();
        assert_eq!(
            data,
            UnsupportedProtocolVersionErrorData::new(
                ProtocolVersion::V2,
                vec![ProtocolVersion::V1]
            )
        );
    }

    use super::*;

    #[test]
    fn unsupported_protocol_version_includes_requested_and_supported() {
        let error = Error::unsupported_protocol_version(
            ProtocolVersion::V1,
            &[ProtocolVersion::from(2_u16)],
        );
        assert_eq!(
            serde_json::to_value(error).unwrap(),
            serde_json::json!({
                "code": -32022,
                "message": "Unsupported protocol version",
                "data": { "requested": 1, "supported": [2] },
            })
        );
        assert_eq!(
            Error::unsupported_protocol_version(ProtocolVersion::V1, &[]).data,
            Some(serde_json::json!({ "requested": 1, "supported": [] }))
        );
    }

    #[test]
    fn serialize_error_code() {
        assert_eq!(
            serde_json::from_value::<ErrorCode>(serde_json::json!(-32700)).unwrap(),
            ErrorCode::ParseError
        );
        assert_eq!(
            serde_json::to_value(ErrorCode::ParseError).unwrap(),
            serde_json::json!(-32700)
        );

        assert_eq!(
            serde_json::from_value::<ErrorCode>(serde_json::json!(1)).unwrap(),
            ErrorCode::Other(1)
        );
        assert_eq!(
            serde_json::to_value(ErrorCode::Other(1)).unwrap(),
            serde_json::json!(1)
        );
    }

    #[test]
    fn serialize_error_code_equality() {
        // Make sure schema generation doesn't panic when enabled.
        #[cfg(feature = "schemars")]
        let _schema = schemars::schema_for!(ErrorCode);
        for error in ErrorCode::iter() {
            assert_eq!(
                error,
                serde_json::from_value(serde_json::to_value(error).unwrap()).unwrap()
            );
        }
    }
}
