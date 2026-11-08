//! Experimental classification of failures in existing JSON-RPC error data.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, serde_as, skip_serializing_none};

/// **UNSTABLE.** Optional classification fields in an object-valued error `data`.
///
/// Omitted and `null` fields mean unknown. Malformed fields are ignored individually.
/// Existing non-object error data remains valid and must not be wrapped or replaced.
/// Authentication continues to use code `-32000`, regardless of classification.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ErrorClassification {
    /// The cause of the failure. Unknown values must use generic error handling.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub kind: Option<ErrorKind>,
    /// Whether the same unchanged request may succeed. Absence means unknown, not false.
    /// This hint does not authorize automatic retries or guarantee safe side effects.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub retryable: Option<bool>,
    /// Minimum delay from receipt before retrying, supplied by the provider or runtime.
    /// Only meaningful when `retryable` is true. Agents must not invent this value.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub retry_after_ms: Option<u64>,
    /// Other existing Agent-specific data, preserved when decoding and encoding.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl ErrorClassification {
    /// Creates data with all classification fields unknown.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// **UNSTABLE.** An open classification of a failure that ends foreground work.
/// Custom values start with `_`. Other unknown values are reserved for future ACP kinds.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorKind {
    /// A temporary request or throughput limit.
    RateLimited,
    /// An account, plan, or organization usage allowance is used up.
    UsageLimit,
    /// Input exceeds the model's context window.
    ContextWindowExceeded,
    /// A configured cost or token budget was reached.
    BudgetExceeded,
    /// The service is temporarily out of capacity.
    Overloaded,
    /// The Agent cannot reach its runtime or a required service.
    Connection,
    /// The request, configuration, or model cannot be accepted.
    InvalidRequest,
    /// A policy blocks the operation. Normal refusals retain the refusal stop reason.
    Policy,
    /// The model provider failed with no more specific known cause.
    ProviderError,
    /// The Agent failed with no more specific known cause.
    Internal,
    /// Custom or future classification. Receivers must accept unknown values.
    #[serde(untagged)]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn known_and_unknown_kinds_round_trip_with_existing_data() {
        for kind in ["rate_limited", "_provider/output_limit", "future_kind"] {
            let wire = json!({"kind": kind, "retryable": true, "retryAfterMs": 15000, "requestId": "req_1"});
            let data: ErrorClassification = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(data).unwrap(), wire);
        }
    }

    #[test]
    fn malformed_fields_do_not_discard_valid_fields() {
        for delay in [json!(-1), json!(1.5), json!("soon"), json!({})] {
            let data: ErrorClassification = serde_json::from_value(json!({
                "kind": "rate_limited", "retryable": true, "retryAfterMs": delay,
            }))
            .unwrap();
            assert_eq!(data.kind, Some(ErrorKind::RateLimited));
            assert_eq!(data.retryable, Some(true));
            assert_eq!(data.retry_after_ms, None);
        }
        let data: ErrorClassification = serde_json::from_value(json!({
            "kind": 7, "retryable": "yes", "retryAfterMs": 0,
        }))
        .unwrap();
        assert_eq!(data.kind, None);
        assert_eq!(data.retryable, None);
        assert_eq!(data.retry_after_ms, Some(0));
    }

    #[test]
    fn omitted_null_and_false_remain_distinct() {
        for wire in [
            json!({}),
            json!({"kind": null, "retryable": null, "retryAfterMs": null}),
        ] {
            assert_eq!(
                serde_json::from_value::<ErrorClassification>(wire).unwrap(),
                ErrorClassification::new()
            );
        }
        let data: ErrorClassification =
            serde_json::from_value(json!({"retryable": false})).unwrap();
        assert_eq!(data.retryable, Some(false));
        assert_eq!(
            serde_json::to_value(data).unwrap(),
            json!({"retryable": false})
        );
    }

    #[test]
    fn error_accessors_preserve_raw_payloads_and_authentication_codes() {
        for value in [json!("legacy"), json!(3), json!(["legacy"]), json!(null)] {
            let error = crate::v1::Error::internal_error().data(value.clone());
            assert_eq!(error.classification(), None);
            assert_eq!(error.data, Some(value));
        }
        let error = crate::v1::Error::auth_required()
            .data(json!({"kind": "overloaded", "retryable": false}));
        assert_eq!(i32::from(error.code), -32000);
        assert_eq!(error.classification().unwrap().retryable, Some(false));
        #[cfg(feature = "unstable_protocol_v2")]
        {
            let error =
                crate::v2::Error::internal_error().data(json!({"kind": "_provider/custom"}));
            assert_eq!(
                error.classification().unwrap().kind,
                Some(ErrorKind::Other("_provider/custom".into()))
            );
        }
    }
}
