//! Experimental history entries for failed foreground work.

use std::sync::Arc;

use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, serde_as, skip_serializing_none};

use super::{Meta, StopReason};
use crate::IntoOption;

/// **UNSTABLE.** An Agent-generated failure ID, unique within the session.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(forward)]
#[non_exhaustive]
pub struct FailureId(pub Arc<str>);

impl FailureId {
    /// Wraps a failure ID.
    #[must_use]
    pub fn new(id: impl Into<Self>) -> Self {
        id.into()
    }
}

/// **UNSTABLE.** A history entry explaining why foreground work failed.
///
/// A repeated `failureId` replaces the entry. It does not append a duplicate.
/// Agents send it after the work's other updates and before its idle `state_update`.
/// The idle update alone ends foreground work and still reports the same stop reason.
/// Retained entries replay in their original history position, with the same ID when possible.
/// A failure is current only while it is the latest conversation entry and the session is idle.
/// This type does not execute retries or implement history storage.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(!try_from, transform = failure_schema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", try_from = "FailureFields")]
#[non_exhaustive]
pub struct Failure {
    /// Unique ID within the enclosing session. Repeated IDs replace the existing entry.
    pub failure_id: FailureId,
    /// Why foreground work failed. Required. Unknown reasons are accepted.
    /// `end_turn` and `cancelled` must not be reported as failures.
    #[serde(flatten)]
    pub stop_reason: StopReason,
    /// Optional metadata. Omitted and `null` mean no metadata.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default, rename = "_meta")]
    pub meta: Option<Meta>,
}

#[serde_as]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FailureFields {
    failure_id: FailureId,
    #[serde(flatten)]
    stop_reason: StopReason,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default, rename = "_meta")]
    meta: Option<Meta>,
}

impl TryFrom<FailureFields> for Failure {
    type Error = &'static str;

    fn try_from(fields: FailureFields) -> Result<Self, Self::Error> {
        let mut failure = Self::new(fields.failure_id, fields.stop_reason)?;
        failure.meta = fields.meta;
        Ok(failure)
    }
}

impl Failure {
    /// Creates a failure entry. Successful completion and cancellation are not failures.
    ///
    /// # Errors
    /// Returns an error for `end_turn` and `cancelled`.
    pub fn new(
        failure_id: impl Into<FailureId>,
        stop_reason: StopReason,
    ) -> Result<Self, &'static str> {
        if matches!(stop_reason, StopReason::EndTurn | StopReason::Cancelled) {
            return Err("end_turn and cancelled are not failure stop reasons");
        }
        Ok(Self {
            failure_id: failure_id.into(),
            stop_reason,
            meta: None,
        })
    }

    /// Sets optional metadata without changing the stored failure identity.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

#[cfg(feature = "schemars")]
fn failure_schema(schema: &mut schemars::Schema) {
    super::schema_util::reject_known_string_discriminators(
        schema,
        "stopReason",
        &["end_turn", "cancelled"],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::{
        Error, ErrorStopReason, IdleStateUpdate, OtherStopReason, SessionNotification,
        SessionUpdate, StateUpdate,
    };
    use serde_json::json;

    #[test]
    fn failure_and_idle_have_the_same_error_but_distinct_roles() {
        let reason = StopReason::Error(ErrorStopReason::new().error(
            Error::internal_error().data(json!({"kind": "rate_limited", "retryable": true})),
        ));
        let failure = SessionUpdate::Failure(Failure::new("fail_1", reason.clone()).unwrap());
        let idle = SessionUpdate::StateUpdate(StateUpdate::Idle(
            IdleStateUpdate::new().stop_reason(reason),
        ));
        let wire = serde_json::to_value(&failure).unwrap();
        assert_eq!(wire["sessionUpdate"], "failure");
        assert_eq!(wire["failureId"], "fail_1");
        assert_eq!(wire["error"], serde_json::to_value(idle).unwrap()["error"]);
        assert!(wire.get("state").is_none());
        assert_eq!(
            serde_json::from_value::<SessionUpdate>(wire).unwrap(),
            failure
        );
    }

    #[test]
    fn live_and_replay_preserve_identity_and_payload() {
        for reason in [
            StopReason::MaxTokens,
            StopReason::MaxTurnRequests,
            StopReason::Refusal,
            StopReason::Error(ErrorStopReason::new()),
        ] {
            let live = SessionNotification::new(
                "sess_1",
                SessionUpdate::Failure(Failure::new("fail_1", reason).unwrap()),
            );
            let wire = serde_json::to_value(&live).unwrap();
            let replay: SessionNotification = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(replay).unwrap(), wire);
        }
    }

    #[test]
    fn custom_and_future_stop_reasons_round_trip() {
        for reason in ["_provider/output_limit", "future_failure"] {
            let wire = json!({"sessionUpdate": "failure", "failureId": "fail_1", "stopReason": reason, "detail": "kept", "_meta": {"a": 1}});
            let update: SessionUpdate = serde_json::from_value(wire.clone()).unwrap();
            assert!(matches!(
                &update,
                SessionUpdate::Failure(Failure {
                    stop_reason: StopReason::Other(_),
                    ..
                })
            ));
            assert_eq!(serde_json::to_value(update).unwrap(), wire);
        }
        let _custom = OtherStopReason::new("_custom", Default::default());
    }

    #[test]
    fn malformed_known_failure_does_not_fall_through_to_unknown_update() {
        for wire in [
            json!({"sessionUpdate": "failure", "failureId": "fail_1"}),
            json!({"sessionUpdate": "failure", "stopReason": "refusal"}),
            json!({"sessionUpdate": "failure", "failureId": null, "stopReason": "refusal"}),
            json!({"sessionUpdate": "failure", "failureId": "fail_1", "stopReason": null}),
            json!({"sessionUpdate": "failure", "failureId": "fail_1", "stopReason": "end_turn"}),
            json!({"sessionUpdate": "failure", "failureId": "fail_1", "stopReason": "cancelled"}),
        ] {
            assert!(
                serde_json::from_value::<SessionUpdate>(wire.clone()).is_err(),
                "{wire}"
            );
        }
        assert!(Failure::new("fail_1", StopReason::EndTurn).is_err());
        assert!(Failure::new("fail_1", StopReason::Cancelled).is_err());
    }
}
