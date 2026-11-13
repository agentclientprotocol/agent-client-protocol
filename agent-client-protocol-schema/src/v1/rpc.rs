//! JSON-RPC request and response envelopes for protocol version 1.

use std::sync::Arc;

use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// JSON RPC Request Id
///
/// An identifier established by the Client that MUST contain a String, Number, or NULL value if included. If it is not included it is assumed to be a notification. The value SHOULD normally not be Null \[1\] and Numbers SHOULD NOT contain fractional parts \[2\]
///
/// The Server MUST reply with the same value in the Response object if included. This member is used to correlate the context between the two objects.
///
/// \[1\] The use of Null as a value for the id member in a Request object is discouraged, because this specification uses a value of Null for Responses with an unknown id. Also, because JSON-RPC 1.0 uses an id value of Null for Notifications this could cause confusion in handling.
///
/// \[2\] Fractional parts may be problematic, since many decimal fractions cannot be represented exactly as binary fractions.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(
    Debug, PartialEq, Clone, Hash, Eq, Deserialize, Serialize, PartialOrd, Ord, Display, From,
)]
#[serde(untagged)]
#[allow(
    clippy::exhaustive_enums,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[from(String, i64)]
pub enum RequestId {
    /// The JSON-RPC `null` request id.
    #[display("null")]
    Null,
    /// A numeric JSON-RPC request id.
    Number(i64),
    /// A string JSON-RPC request id.
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
pub enum Response<Result, Error = super::Error> {
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
        id: RequestId,
        /// Method-specific error data.
        error: Error,
    },
}

impl<R, E> Response<R, E> {
    /// Creates a JSON-RPC response from a Rust [`Result`].
    #[must_use]
    pub fn new(id: impl Into<RequestId>, result: std::result::Result<R, E>) -> Self {
        match result {
            Ok(result) => Self::Result {
                id: id.into(),
                result,
            },
            Err(error) => Self::Error {
                id: id.into(),
                error,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::{Number, Value};

    #[test]
    fn id_deserialization() {
        let id = serde_json::from_value::<RequestId>(Value::Null).unwrap();
        assert_eq!(id, RequestId::Null);

        let id = serde_json::from_value::<RequestId>(Value::Number(Number::from_u128(1).unwrap()))
            .unwrap();
        assert_eq!(id, RequestId::Number(1));

        let id = serde_json::from_value::<RequestId>(Value::Number(Number::from_i128(-1).unwrap()))
            .unwrap();
        assert_eq!(id, RequestId::Number(-1));

        let id = serde_json::from_value::<RequestId>(Value::String("id".to_owned())).unwrap();
        assert_eq!(id, RequestId::Str("id".to_owned()));
    }

    #[test]
    fn id_serialization() {
        let id = serde_json::to_value(RequestId::Null).unwrap();
        assert_eq!(id, Value::Null);

        let id = serde_json::to_value(RequestId::Number(1)).unwrap();
        assert_eq!(id, Value::Number(Number::from_u128(1).unwrap()));

        let id = serde_json::to_value(RequestId::Number(-1)).unwrap();
        assert_eq!(id, Value::Number(Number::from_i128(-1).unwrap()));

        let id = serde_json::to_value(RequestId::Str("id".to_owned())).unwrap();
        assert_eq!(id, Value::String("id".to_owned()));
    }

    #[test]
    fn id_display() {
        let id = RequestId::Null;
        assert_eq!(id.to_string(), "null");

        let id = RequestId::Number(1);
        assert_eq!(id.to_string(), "1");

        let id = RequestId::Number(-1);
        assert_eq!(id.to_string(), "-1");

        let id = RequestId::Str("id".to_owned());
        assert_eq!(id.to_string(), "id");
    }
}
