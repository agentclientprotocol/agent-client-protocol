//! Method-aware parsing of the aggregate protocol enums.
//!
//! These helpers select a payload type before deserializing it. They do not
//! validate JSON-RPC envelopes, track request IDs, or assign JSON-RPC error codes.

use serde::de::DeserializeOwned;
use serde_json::Value;

/// A failure to select or deserialize a method's payload.
///
/// Callers decide how to report this error. For example, a JSON-RPC runtime can
/// report an invalid request payload as Invalid params, but handle an invalid
/// response payload locally.
#[derive(Debug)]
#[non_exhaustive]
pub enum MethodDispatchError {
    /// The method is unknown or its feature is disabled.
    UnknownMethod {
        /// The method name supplied by the caller.
        method: String,
    },
    /// The payload could not be deserialized into the selected type.
    InvalidPayload {
        /// The method name supplied by the caller.
        method: String,
        /// The underlying deserialization error.
        ///
        /// Its message can contain input values. Treat it as sensitive.
        source: serde_json::Error,
    },
}

impl std::fmt::Display for MethodDispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownMethod { method } => write!(f, "unknown method `{method}`"),
            Self::InvalidPayload { method, source } => {
                write!(f, "invalid payload for method `{method}`: {source}")
            }
        }
    }
}

impl std::error::Error for MethodDispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnknownMethod { .. } => None,
            Self::InvalidPayload { source, .. } => Some(source),
        }
    }
}

pub(crate) fn deserialize<T: DeserializeOwned>(
    method: &str,
    payload: Value,
) -> Result<T, MethodDispatchError> {
    serde_json::from_value(payload).map_err(|source| MethodDispatchError::InvalidPayload {
        method: method.to_owned(),
        source,
    })
}

// Keep incoming dispatch and outgoing method names in the same table. The
// exhaustive match in method() also makes new enum variants require a mapping.
macro_rules! impl_message_dispatch {
    (
        $enum:ident {
            $( $(#[$meta:meta])* $variant:ident => $method:expr, )*
        }
        $(extension $ext_variant:ident($ext_type:ty);)?
    ) => {
        impl $enum {
            /// Returns the corresponding method name.
            #[must_use]
            pub fn method(&self) -> &str {
                match self {
                    $( $(#[$meta])* Self::$variant(_) => $method, )*
                    $(Self::$ext_variant(payload) => &payload.method,)?
                }
            }

            /// Whether this enum recognizes the method with the enabled Cargo
            /// features, including extensions when it has an extension variant.
            ///
            /// This does not check the peer's advertised capabilities.
            #[must_use]
            pub fn supports_method(method: &str) -> bool {
                match method {
                    $( $(#[$meta])* m if m == $method => true, )*
                    _ => impl_message_dispatch!(@extension method $(, $ext_variant)?),
                }
            }

            /// Parses parameters selected by the JSON-RPC method name.
            ///
            /// Use this instead of the enum's untagged `Deserialize`
            /// implementation. Untagged deserialization selects by payload
            /// shape, which can silently select an unrelated variant.
            ///
            /// The payload is passed unchanged to the selected type's
            /// deserializer. In particular, `null` is not converted to `{}`.
            /// Extension payloads retain their JSON value and method name.
            ///
            /// # Errors
            ///
            /// Returns [`crate::MethodDispatchError::UnknownMethod`] for an
            /// unsupported method, or [`crate::MethodDispatchError::InvalidPayload`]
            /// for a malformed payload.
            pub fn from_method_and_params(
                method: &str,
                params: &serde_json::Value,
            ) -> Result<Self, crate::MethodDispatchError> {
                match method {
                    $( $(#[$meta])* m if m == $method =>
                        crate::dispatch::deserialize(method, params.clone()).map(Self::$variant), )*
                    _ => {
                        $(
                            if method.starts_with('_') {
                                return crate::dispatch::deserialize(method, params.clone())
                                    .map(|mut payload: $ext_type| {
                                        payload.method = method.into();
                                        Self::$ext_variant(payload)
                                    });
                            }
                        )?
                        Err(crate::MethodDispatchError::UnknownMethod {
                            method: method.to_owned(),
                        })
                    }
                }
            }
        }
    };
    (@extension $method:ident, $variant:ident) => { $method.starts_with('_') };
    (@extension $method:ident) => { false };
}

// A request and its successful response share one method mapping.
macro_rules! impl_request_dispatch {
    (
        $request:ident => $response:ident {
            $( $(#[$meta:meta])* $req_variant:ident => $resp_variant:ident: $method:expr, )*
        }
        extension $ext_req:ident($ext_type:ty) => $ext_resp:ident;
    ) => {
        impl_message_dispatch! {
            $request {
                $( $(#[$meta])* $req_variant => $method, )*
            }
            extension $ext_req($ext_type);
        }

        impl $response {
            /// Parses a successful result using the originating request's method.
            ///
            /// Responses do not carry a method. The caller must retain it when
            /// sending the request and supply it here when the response arrives.
            /// This does not parse JSON-RPC error responses.
            ///
            /// Use this instead of the enum's untagged `Deserialize`
            /// implementation, which can silently select an unrelated variant.
            /// The result is deserialized unchanged, including `null`.
            ///
            /// # Errors
            ///
            /// Returns [`crate::MethodDispatchError::UnknownMethod`] for an
            /// unsupported method, or [`crate::MethodDispatchError::InvalidPayload`]
            /// for a malformed result.
            pub fn from_method_and_result(
                method: &str,
                result: serde_json::Value,
            ) -> Result<Self, crate::MethodDispatchError> {
                match method {
                    $( $(#[$meta])* m if m == $method =>
                        crate::dispatch::deserialize(method, result).map(Self::$resp_variant), )*
                    _ if method.starts_with('_') =>
                        crate::dispatch::deserialize(method, result).map(Self::$ext_resp),
                    _ => Err(crate::MethodDispatchError::UnknownMethod {
                        method: method.to_owned(),
                    }),
                }
            }
        }
    };
}
