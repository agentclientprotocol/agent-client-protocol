//! Execution plans for complex tasks that require multiple steps.
//!
//! Plans are strategies that agents share with clients through session updates,
//! providing real-time visibility into their thinking and progress.
//!
//! See: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, VecSkipError, serde_as, skip_serializing_none};

use crate::{IntoOption, Meta, SkipListener};

/// An execution plan for accomplishing complex tasks.
///
/// Plans consist of multiple entries representing individual tasks or goals.
/// Agents report plans to clients to provide visibility into their execution strategy.
/// Plans can evolve during execution as the agent discovers new requirements or completes tasks.
///
/// See protocol docs: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Plan {
    /// The list of tasks to be accomplished.
    ///
    /// When updating a plan, the agent must send a complete list of all entries
    /// with their current status. The client replaces the entire plan with each update.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, SkipListener>>")]
    pub entries: Vec<PlanEntry>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Plan {
    #[must_use]
    pub fn new(entries: Vec<PlanEntry>) -> Self {
        Self {
            entries,
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

/// A single entry in the execution plan.
///
/// Represents a task or goal that the assistant intends to accomplish
/// as part of fulfilling the user's request.
/// See protocol docs: [Plan Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanEntry {
    /// Human-readable description of what this task aims to accomplish.
    pub content: String,
    /// The relative importance of this task.
    /// Used to indicate which tasks are most critical to the overall goal.
    pub priority: PlanEntryPriority,
    /// Current execution status of this task.
    pub status: PlanEntryStatus,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl PlanEntry {
    #[must_use]
    pub fn new(
        content: impl Into<String>,
        priority: PlanEntryPriority,
        status: PlanEntryStatus,
    ) -> Self {
        Self {
            content: content.into(),
            priority,
            status,
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

/// Priority levels for plan entries.
///
/// Used to indicate the relative importance or urgency of different
/// tasks in the execution plan.
/// See protocol docs: [Plan Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PlanEntryPriority {
    /// High priority task - critical to the overall goal.
    High,
    /// Medium priority task - important but not critical.
    Medium,
    /// Low priority task - nice to have but not essential.
    Low,
}

/// Status of a plan entry in the execution flow.
///
/// Tracks the lifecycle of each task from planning through completion.
/// See protocol docs: [Plan Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PlanEntryStatus {
    /// The task has not started yet.
    Pending,
    /// The task is currently being worked on.
    InProgress,
    /// The task has been successfully completed.
    Completed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry() -> PlanEntry {
        PlanEntry::new("step", PlanEntryPriority::Medium, PlanEntryStatus::Pending)
    }

    #[test]
    fn plan_entry_roundtrip_omits_optional_meta() {
        let value = entry();
        let serialized = serde_json::to_value(&value).unwrap();
        assert_eq!(
            serialized,
            json!({
                "content": "step",
                "priority": "medium",
                "status": "pending",
            })
        );
        let parsed: PlanEntry = serde_json::from_value(serialized).unwrap();
        assert_eq!(parsed, value);
    }

    #[test]
    fn plan_entry_priority_and_status_use_snake_case() {
        assert_eq!(
            serde_json::to_value(&PlanEntryPriority::High).unwrap(),
            json!("high"),
        );
        assert_eq!(
            serde_json::to_value(&PlanEntryPriority::Low).unwrap(),
            json!("low"),
        );
        assert_eq!(
            serde_json::to_value(&PlanEntryStatus::InProgress).unwrap(),
            json!("in_progress"),
        );
        assert_eq!(
            serde_json::to_value(&PlanEntryStatus::Completed).unwrap(),
            json!("completed"),
        );

        let parsed: PlanEntryPriority = serde_json::from_str("\"high\"").unwrap();
        assert_eq!(parsed, PlanEntryPriority::High);
        let parsed: PlanEntryStatus = serde_json::from_str("\"in_progress\"").unwrap();
        assert_eq!(parsed, PlanEntryStatus::InProgress);
    }

    #[test]
    fn plan_skips_malformed_entries_keeping_valid_ones() {
        // Mirrors the `DefaultOnError<VecSkipError<_, SkipListener>>` pattern
        // applied to `Plan::entries` so a single bad entry does not poison the
        // whole plan update.
        let input = json!({
            "entries": [
                {"content": "ok 1", "priority": "high", "status": "pending"},
                {"content": "missing priority", "status": "pending"},
                {"content": "wrong types", "priority": 7, "status": false},
                "not even an object",
                {"content": "ok 2", "priority": "low", "status": "completed"},
            ]
        });

        let plan: Plan = serde_json::from_value(input).unwrap();
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.entries[0].content, "ok 1");
        assert_eq!(plan.entries[0].priority, PlanEntryPriority::High);
        assert_eq!(plan.entries[1].content, "ok 2");
        assert_eq!(plan.entries[1].status, PlanEntryStatus::Completed);
    }

    #[test]
    fn plan_collapses_outer_shape_errors_to_empty_entries() {
        // `DefaultOnError` should swallow outer-shape failures so a producer
        // sending the wrong type for `entries` does not break consumers.
        // `entries` is required on the wire (no `#[serde(default)]`), so a
        // missing key still errors; that's intentional and documented here.
        let cases = [
            json!({ "entries": null }),
            json!({ "entries": "oops" }),
            json!({ "entries": {"k": 1} }),
            json!({ "entries": 42 }),
        ];
        for input in cases {
            let plan: Plan = serde_json::from_value(input.clone()).unwrap_or_else(|e| {
                panic!("expected Plan to deserialize from {input}: {e}");
            });
            assert!(
                plan.entries.is_empty(),
                "expected empty entries for {input}, got {:?}",
                plan.entries
            );
        }

        // Missing key is still an error - the field is required.
        let err = serde_json::from_value::<Plan>(json!({})).unwrap_err();
        assert!(
            err.to_string().contains("entries"),
            "expected missing-field error to mention `entries`, got: {err}"
        );
    }

    #[test]
    fn plan_preserves_meta_when_present() {
        let mut meta = Meta::new();
        meta.insert("k".to_string(), json!("v"));
        let value = Plan::new(vec![entry()]).meta(meta.clone());

        let serialized = serde_json::to_value(&value).unwrap();
        assert_eq!(serialized["_meta"], json!({"k": "v"}));
        let parsed: Plan = serde_json::from_value(serialized).unwrap();
        assert_eq!(parsed.meta, Some(meta));
    }
}
