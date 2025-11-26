use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::Serialize;

/// Protocol version identifier.
///
/// This version is only bumped for breaking changes.
/// Non-breaking changes should be introduced via capabilities.
#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, From, Display)]
pub struct ProtocolVersion(u16);

impl ProtocolVersion {
    /// Version `0` of the protocol.
    ///
    /// This was a pre-release version that shouldn't be used in production.
    /// It is used as a fallback for any request whose version cannot be parsed
    /// as a valid version, and should likely be treated as unsupported.
    pub const V0: Self = Self(0);
    /// Version `1` of the protocol.
    ///
    /// <https://agentclientprotocol.com/protocol/overview>
    pub const V1: Self = Self(1);
    /// The latest supported version of the protocol.
    ///
    /// Currently, this is version `1`.
    pub const LATEST: Self = Self::V1;

    #[cfg(test)]
    #[must_use]
    pub const fn new(version: u16) -> Self {
        Self(version)
    }
}

use serde::{Deserialize, Deserializer};

impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct ProtocolVersionVisitor;

        impl Visitor<'_> for ProtocolVersionVisitor {
            type Value = ProtocolVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a protocol version number or string")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match u16::try_from(value) {
                    Ok(value) => Ok(ProtocolVersion(value)),
                    Err(_) => Err(E::custom(format!("protocol version {value} is too large"))),
                }
            }

            fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Old versions used strings, we consider all of those version 0
                Ok(ProtocolVersion::V0)
            }

            fn visit_string<E>(self, _value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Old versions used strings, we consider all of those version 0
                Ok(ProtocolVersion::V0)
            }
        }

        deserializer.deserialize_any(ProtocolVersionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_u64() {
        let json = "1";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(1));
    }

    #[test]
    fn test_deserialize_string() {
        let json = "\"1.0.0\"";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(0));
    }

    #[test]
    fn test_deserialize_large_number() {
        let json = "100000";
        let result: Result<ProtocolVersion, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_zero() {
        let json = "0";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(0));
    }

    #[test]
    fn test_deserialize_max_u16() {
        let json = "65535";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(65535));
    }
}
