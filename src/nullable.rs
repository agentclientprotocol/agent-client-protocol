//! A required-but-nullable field wrapper for serde.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A value that must be present on the wire but whose value may be `null`.
///
/// Unlike `Option<T>`, which serde treats as an implicitly optional field
/// (defaulting to `None` when absent), `Nullable<T>` requires the key to be
/// present during deserialization. A missing field will produce a
/// deserialization error rather than silently defaulting to `None`.
///
/// On the wire this serializes identically to `Option<T>` — either `null` or
/// the JSON representation of `T`.
///
/// # Example
///
/// ```rust
/// use agent_client_protocol_schema::Nullable;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Config {
///     // MUST be present in JSON, but its value can be null
///     value: Nullable<String>,
/// }
///
/// // ✅ Present with a value
/// let c: Config = serde_json::from_str(r#"{"value":"hello"}"#).unwrap();
/// assert_eq!(c.value, Nullable::new("hello".to_string()));
///
/// // ✅ Present as null
/// let c: Config = serde_json::from_str(r#"{"value":null}"#).unwrap();
/// assert_eq!(c.value, Nullable::null());
///
/// // ❌ Missing key — deserialization error
/// assert!(serde_json::from_str::<Config>(r#"{}"#).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema)]
#[schemars(with = "Option<T>", inline)]
#[non_exhaustive]
pub struct Nullable<T>(pub Option<T>);

impl<T> Default for Nullable<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: Serialize> Serialize for Nullable<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Nullable<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Deserialize via serde_json::Value so that `deserialize_any` is called.
        // serde's MissingFieldDeserializer errors on `deserialize_any` (good — the
        // field is required), whereas `deserialize_option` silently returns None.
        let value = serde_json::Value::deserialize(deserializer)?;
        if value.is_null() {
            Ok(Nullable(None))
        } else {
            T::deserialize(value)
                .map(Nullable::new)
                .map_err(serde::de::Error::custom)
        }
    }
}

impl<T> Nullable<T> {
    /// Creates a `Nullable` containing a value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(Some(value))
    }

    /// Creates a `Nullable` representing `null`.
    #[must_use]
    pub fn null() -> Self {
        Self(None)
    }

    /// Returns `true` if the value is `null`.
    #[must_use]
    pub fn is_null(&self) -> bool {
        self.0.is_none()
    }

    /// Returns `true` if the value is present (not null).
    #[must_use]
    pub fn is_value(&self) -> bool {
        self.0.is_some()
    }

    /// Returns a reference to the contained value, if present.
    #[must_use]
    pub fn value(&self) -> Option<&T> {
        self.0.as_ref()
    }

    /// Converts into the inner `Option<T>`.
    #[must_use]
    pub fn into_inner(self) -> Option<T> {
        self.0
    }
}

impl<T> From<Option<T>> for Nullable<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T> From<Nullable<T>> for Option<T> {
    fn from(value: Nullable<T>) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::{from_str, json, to_value};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Example {
        value: Nullable<String>,
    }

    #[test]
    fn present_with_value() {
        let example: Example = from_str(r#"{"value":"hello"}"#).unwrap();
        assert_eq!(example.value, Nullable(Some("hello".to_string())));
    }

    #[test]
    fn present_as_null() {
        let example: Example = from_str(r#"{"value":null}"#).unwrap();
        assert_eq!(example.value, Nullable(None));
    }

    #[test]
    fn missing_key_fails() {
        assert!(from_str::<Example>(r"{}").is_err());
    }

    #[test]
    fn serialize_value() {
        let example = Example {
            value: Nullable(Some("hello".to_string())),
        };
        assert_eq!(to_value(&example).unwrap(), json!({"value": "hello"}));
    }

    #[test]
    fn serialize_null() {
        let example = Example {
            value: Nullable(None),
        };
        assert_eq!(to_value(&example).unwrap(), json!({"value": null}));
    }

    #[test]
    fn from_option() {
        let nullable: Nullable<i32> = Some(42).into();
        assert_eq!(nullable, Nullable(Some(42)));

        let nullable: Nullable<i32> = None.into();
        assert_eq!(nullable, Nullable(None));
    }

    #[test]
    fn into_option() {
        let option: Option<i32> = Nullable(Some(42)).into();
        assert_eq!(option, Some(42));

        let option: Option<i32> = Nullable(None).into();
        assert_eq!(option, None);
    }

    #[test]
    fn methods() {
        let value = Nullable::new(42);
        assert!(value.is_value());
        assert!(!value.is_null());
        assert_eq!(value.value(), Some(&42));
        assert_eq!(value.into_inner(), Some(42));

        let null: Nullable<i32> = Nullable::null();
        assert!(!null.is_value());
        assert!(null.is_null());
        assert_eq!(null.value(), None);
        assert_eq!(null.into_inner(), None);
    }

    #[test]
    fn default_is_null() {
        let nullable: Nullable<i32> = Nullable::default();
        assert_eq!(nullable, Nullable(None));
    }
}
