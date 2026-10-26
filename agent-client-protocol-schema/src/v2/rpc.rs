//! JSON-RPC request and response envelopes for protocol version 2.
//!
//! These differ from the shared JSON-RPC types only in their request IDs: v2
//! request IDs are never `null`.

use std::sync::Arc;

use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::Error;

/// A JSON-RPC request ID: a string or an integer. Unlike base JSON-RPC, it MUST NOT be `null`.
///
/// The receiver MUST reply with the same value in the response.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(
    Debug, PartialEq, Clone, Hash, Eq, Deserialize, Serialize, PartialOrd, Ord, Display, From,
)]
#[serde(untagged)]
#[allow(
    clippy::exhaustive_enums,
    reason = "Request IDs are limited to JSON-RPC's non-null ID types"
)]
#[from(String, i64)]
pub enum RequestId {
    /// A numeric request ID.
    Number(i64),
    /// A string request ID.
    Str(String),
}

/// A JSON-RPC request object.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::exhaustive_structs,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[cfg_attr(feature = "schemars", schemars(rename = "{Params}", extend("x-docs-ignore" = true)))]
#[skip_serializing_none]
pub struct Request<Params> {
    /// The request id used to correlate the matching response.
    pub id: RequestId,
    /// The method name to invoke.
    pub method: Arc<str>,
    /// Method-specific request parameters.
    pub params: Option<Params>,
}

/// A JSON-RPC response object.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::exhaustive_enums,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[serde(untagged)]
#[cfg_attr(feature = "schemars", schemars(rename = "{Result}", extend("x-docs-ignore" = true)))]
pub enum Response<Result> {
    /// A successful JSON-RPC response.
    Result {
        /// The id of the request this response answers.
        id: RequestId,
        /// Method-specific response data.
        // Require the key even when the payload's deserializer accepts null.
        #[serde(
            deserialize_with = "Deserialize::deserialize",
            bound(deserialize = "Result: Deserialize<'de>")
        )]
        result: Result,
    },
    /// A failed JSON-RPC response.
    Error {
        /// The id of the request this response answers.
        ///
        /// Required. `null` only when the request's id could not be determined, such as
        /// for a parse error or an invalid request.
        // Require the key; serde would otherwise treat a missing `Option` as `None`.
        #[serde(deserialize_with = "Option::deserialize")]
        #[cfg_attr(feature = "schemars", schemars(required))]
        id: Option<RequestId>,
        /// Method-specific error data.
        error: Error,
    },
}

impl<R> Response<R> {
    /// Creates a JSON-RPC response from a Rust [`Result`].
    #[must_use]
    pub fn new(id: impl Into<RequestId>, result: std::result::Result<R, Error>) -> Self {
        match result {
            Ok(result) => Self::Result {
                id: id.into(),
                result,
            },
            Err(error) => Self::Error {
                id: Some(id.into()),
                error,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::{Value, json};

    #[test]
    fn request_id_rejects_null() {
        assert!(serde_json::from_value::<RequestId>(Value::Null).is_err());
        assert!(
            serde_json::from_value::<Request<Value>>(json!({ "id": null, "method": "m" })).is_err()
        );
        assert_eq!(
            serde_json::from_value::<RequestId>(json!(-1)).unwrap(),
            RequestId::Number(-1)
        );
        assert_eq!(
            serde_json::from_value::<RequestId>(json!("id")).unwrap(),
            RequestId::Str("id".to_owned())
        );
    }

    #[test]
    fn error_response_id_is_required_and_nullable() {
        let error = json!({ "code": -32700, "message": "Parse error" });

        let response =
            serde_json::from_value::<Response<Value>>(json!({ "id": null, "error": error }))
                .unwrap();
        assert!(matches!(response, Response::Error { id: None, .. }));
        assert_eq!(
            serde_json::to_value(&response).unwrap(),
            json!({ "id": null, "error": error })
        );

        assert!(serde_json::from_value::<Response<Value>>(json!({ "error": error })).is_err());
        assert!(
            serde_json::from_value::<Response<Value>>(json!({ "id": null, "result": {} })).is_err()
        );
    }
}
