//! Extension types and constants for protocol extensibility.
use derive_more::From;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::sync::Arc;

use crate::MaybeUndefined;

/// Value attached to a given ACP type on the `_meta` field.
///
/// The _meta property is reserved by ACP to allow clients and agents to attach
/// additional metadata to their interactions. Implementations MUST NOT make assumptions about
/// values at these keys.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
pub type Meta = serde_json::Map<String, serde_json::Value>;

/// Applies the top-level `_meta` patch of an upsert update to stored metadata.
///
/// An omitted `update` leaves `stored` unchanged, and `null` clears it. An
/// object patches `stored` one top-level key at a time: keys it omits keep
/// their stored values, a key set to `null` is removed, and any other value
/// replaces that key's stored value whole. Nested objects are not merged, and
/// a `null` nested inside a value is kept as data. Removing every key leaves an
/// empty object, which means the same as no metadata.
///
/// Use this for the top-level `_meta` of upsert updates such as
/// [`ToolCallUpdate`](super::ToolCallUpdate),
/// [`TerminalUpdate`](super::TerminalUpdate), message updates,
/// [`SessionInfoUpdate`](super::SessionInfoUpdate), and
/// [`CompactionUpdate`](super::CompactionUpdate). Other `_meta` fields are not
/// patches.
///
/// See protocol docs: [Metadata Patches](https://agentclientprotocol.com/protocol/extensibility#metadata-patches)
pub fn apply_meta_update(stored: &mut MaybeUndefined<Meta>, update: MaybeUndefined<Meta>) {
    match update {
        MaybeUndefined::Undefined => {}
        MaybeUndefined::Null => *stored = MaybeUndefined::Null,
        MaybeUndefined::Value(patch) => {
            let mut meta = match std::mem::take(stored) {
                MaybeUndefined::Value(meta) => meta,
                MaybeUndefined::Undefined | MaybeUndefined::Null => Meta::new(),
            };
            for (key, value) in patch {
                if value.is_null() {
                    meta.remove(&key);
                } else {
                    meta.insert(key, value);
                }
            }
            *stored = MaybeUndefined::Value(meta);
        }
    }
}

/// Allows for sending an arbitrary request that is not part of the ACP spec.
/// Extension methods provide a way to add custom functionality while maintaining
/// protocol compatibility.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ExtRequest {
    /// Wire method name for this extension request.
    ///
    /// Extension method names must start with `_`.
    #[serde(skip)] // this is used for routing, but when serializing we only want the params
    pub method: Arc<str>,
    /// Raw JSON parameters for this extension message.
    #[cfg_attr(feature = "schemars", schemars(with = "serde_json::Value"))]
    pub params: Arc<RawValue>,
}

impl ExtRequest {
    /// Builds [`ExtRequest`] with the required request fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(method: impl Into<Arc<str>>, params: Arc<RawValue>) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

/// Allows for sending an arbitrary response to an [`ExtRequest`] that is not part of the ACP spec.
/// Extension methods provide a way to add custom functionality while maintaining
/// protocol compatibility.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ExtResponse(
    #[cfg_attr(feature = "schemars", schemars(with = "serde_json::Value"))] pub Arc<RawValue>,
);

impl ExtResponse {
    /// Builds [`ExtResponse`] with the required response fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(params: Arc<RawValue>) -> Self {
        Self(params)
    }
}

/// Allows the Agent to send an arbitrary notification that is not part of the ACP spec.
/// Extension notifications provide a way to send one-way messages for custom functionality
/// while maintaining protocol compatibility.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ExtNotification {
    /// Wire method name for this extension notification.
    ///
    /// Extension method names must start with `_`.
    #[serde(skip)] // this is used for routing, but when serializing we only want the params
    pub method: Arc<str>,
    /// Raw JSON parameters for this extension message.
    #[cfg_attr(feature = "schemars", schemars(with = "serde_json::Value"))]
    pub params: Arc<RawValue>,
}

impl ExtNotification {
    /// Builds [`ExtNotification`] with the required notification fields set; optional fields start unset or empty.
    #[must_use]
    pub fn new(method: impl Into<Arc<str>>, params: Arc<RawValue>) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json};

    use super::*;

    fn meta(value: serde_json::Value) -> MaybeUndefined<Meta> {
        from_value(value).unwrap()
    }

    #[test]
    fn meta_update_patches_top_level_keys() {
        let mut stored = meta(json!({
            "zed.dev/kept": 1,
            "zed.dev/removed": true,
            "acme.dev/nested": {"a": 1, "b": 2},
            "acme.dev/list": [1, 2]
        }));

        apply_meta_update(
            &mut stored,
            meta(json!({
                "zed.dev/removed": null,
                "zed.dev/absent": null,
                "acme.dev/nested": {"b": 3, "c": null},
                "acme.dev/list": [3],
                "acme.dev/added": []
            })),
        );

        // Values replace the stored key whole instead of merging or appending,
        // and nested nulls are data rather than removals.
        assert_eq!(
            stored,
            meta(json!({
                "zed.dev/kept": 1,
                "acme.dev/nested": {"b": 3, "c": null},
                "acme.dev/list": [3],
                "acme.dev/added": []
            }))
        );
    }

    #[test]
    fn meta_update_omitted_empty_and_null() {
        let original = meta(json!({"zed.dev/kept": 1}));
        let mut stored = original.clone();

        apply_meta_update(&mut stored, MaybeUndefined::Undefined);
        assert_eq!(stored, original);

        // An empty patch changes no keys.
        apply_meta_update(&mut stored, meta(json!({})));
        assert_eq!(stored, original);

        apply_meta_update(&mut stored, MaybeUndefined::Null);
        assert_eq!(stored, MaybeUndefined::Null);
    }

    #[test]
    fn meta_update_starts_from_empty_after_clear_or_on_first_update() {
        for initial in [MaybeUndefined::Undefined, MaybeUndefined::Null] {
            let mut stored = initial;
            apply_meta_update(
                &mut stored,
                meta(json!({"zed.dev/added": 1, "zed.dev/removed": null})),
            );
            // A `null` key value is never stored.
            assert_eq!(stored, meta(json!({"zed.dev/added": 1})));
        }
    }
}
