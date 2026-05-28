//! Execution plans for complex tasks that require multiple steps.
//!
//! Plans are strategies that agents share with clients through session updates,
//! providing real-time visibility into their thinking and progress.
//!
//! See: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)

#[cfg(feature = "unstable_plan_operations")]
use std::{collections::BTreeMap, sync::Arc};

#[cfg(feature = "unstable_plan_operations")]
use derive_more::{Display, From};
use schemars::JsonSchema;
#[cfg(feature = "unstable_plan_operations")]
use schemars::Schema;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, VecSkipError, serde_as, skip_serializing_none};

use super::Meta;
use crate::{IntoOption, SkipListener};

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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Unique identifier for a plan within a session.
#[cfg(feature = "unstable_plan_operations")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct PlanId(pub Arc<str>);

#[cfg(feature = "unstable_plan_operations")]
impl PlanId {
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A content update for a plan identified by ID.
#[cfg(feature = "unstable_plan_operations")]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanUpdate {
    /// The updated plan content.
    pub plan: PlanUpdateContent,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_plan_operations")]
impl PlanUpdate {
    #[must_use]
    pub fn new(plan: PlanUpdateContent) -> Self {
        Self { plan, meta: None }
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Updated content for a plan.
#[cfg(feature = "unstable_plan_operations")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(extend("discriminator" = {"propertyName": "type"}))]
#[non_exhaustive]
pub enum PlanUpdateContent {
    /// Structured plan entries.
    Items(PlanItems),
    /// A URI pointing to a file containing the plan.
    File(PlanFile),
    /// Raw markdown content for the plan.
    Markdown(PlanMarkdown),
    /// Custom or future plan update content.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    ///
    /// Receivers that do not understand this content type should preserve the
    /// raw payload when storing, replaying, proxying, or forwarding plans, and
    /// otherwise ignore it or display it generically.
    #[serde(untagged)]
    Other(OtherPlanUpdateContent),
}

/// Custom or future plan update content payload.
#[cfg(feature = "unstable_plan_operations")]
#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(inline)]
#[schemars(transform = other_plan_update_content_schema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OtherPlanUpdateContent {
    /// Custom or future plan update content type.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    #[serde(rename = "type")]
    pub type_: String,
    /// Additional fields from the unknown plan update content payload.
    #[serde(flatten)]
    pub fields: BTreeMap<String, serde_json::Value>,
}

#[cfg(feature = "unstable_plan_operations")]
impl OtherPlanUpdateContent {
    #[must_use]
    pub fn new(type_: impl Into<String>, mut fields: BTreeMap<String, serde_json::Value>) -> Self {
        fields.remove("type");
        Self {
            type_: type_.into(),
            fields,
        }
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl<'de> Deserialize<'de> for OtherPlanUpdateContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut fields = BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
        let type_ = fields
            .remove("type")
            .ok_or_else(|| serde::de::Error::missing_field("type"))?;
        let serde_json::Value::String(type_) = type_ else {
            return Err(serde::de::Error::custom("`type` must be a string"));
        };

        if is_known_plan_update_content_type(&type_) {
            return Err(serde::de::Error::custom(format!(
                "known plan update content `{type_}` did not match its schema"
            )));
        }

        Ok(Self { type_, fields })
    }
}

#[cfg(feature = "unstable_plan_operations")]
fn is_known_plan_update_content_type(type_: &str) -> bool {
    matches!(type_, "items" | "file" | "markdown")
}

#[cfg(feature = "unstable_plan_operations")]
fn other_plan_update_content_schema(schema: &mut Schema) {
    super::schema_util::reject_known_string_discriminators(
        schema,
        "type",
        &["items", "file", "markdown"],
    );
}

#[cfg(feature = "unstable_plan_operations")]
impl PlanUpdateContent {
    #[must_use]
    pub fn items(id: impl Into<PlanId>, entries: Vec<PlanEntry>) -> Self {
        Self::Items(PlanItems::new(id, entries))
    }

    #[must_use]
    pub fn file(id: impl Into<PlanId>, uri: impl Into<String>) -> Self {
        Self::File(PlanFile::new(id, uri))
    }

    #[must_use]
    pub fn markdown(id: impl Into<PlanId>, content: impl Into<String>) -> Self {
        Self::Markdown(PlanMarkdown::new(id, content))
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A plan represented as structured entries.
#[cfg(feature = "unstable_plan_operations")]
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanItems {
    /// The plan ID to update.
    pub id: PlanId,
    /// The list of tasks to be accomplished.
    ///
    /// When updating an item-based plan, the agent must send a complete list of all entries
    /// with their current status. The client replaces that plan with each update.
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

#[cfg(feature = "unstable_plan_operations")]
impl PlanItems {
    #[must_use]
    pub fn new(id: impl Into<PlanId>, entries: Vec<PlanEntry>) -> Self {
        Self {
            id: id.into(),
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A plan represented by a file URI.
#[cfg(feature = "unstable_plan_operations")]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanFile {
    /// The plan ID to update.
    pub id: PlanId,
    /// The URI of the file containing the plan.
    pub uri: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_plan_operations")]
impl PlanFile {
    #[must_use]
    pub fn new(id: impl Into<PlanId>, uri: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            uri: uri.into(),
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A plan represented as raw markdown content.
#[cfg(feature = "unstable_plan_operations")]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanMarkdown {
    /// The plan ID to update.
    pub id: PlanId,
    /// Markdown content for the plan.
    pub content: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_plan_operations")]
impl PlanMarkdown {
    #[must_use]
    pub fn new(id: impl Into<PlanId>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Removal notice for a plan identified by ID.
#[cfg(feature = "unstable_plan_operations")]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanRemoved {
    /// The plan ID to remove.
    pub id: PlanId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_plan_operations")]
impl PlanRemoved {
    #[must_use]
    pub fn new(id: impl Into<PlanId>) -> Self {
        Self {
            id: id.into(),
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

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Capabilities for receiving `plan_update` and `plan_removed` session updates.
#[cfg(feature = "unstable_plan_operations")]
#[skip_serializing_none]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanCapabilities {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_plan_operations")]
impl PlanCapabilities {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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
    /// Custom or future plan entry priority.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    #[serde(untagged)]
    Other(String),
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
    /// Custom or future plan entry status.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    #[serde(untagged)]
    Other(String),
}

#[cfg(all(test, feature = "unstable_plan_operations"))]
mod tests {
    use super::*;

    #[test]
    fn plan_entry_priority_preserves_unknown_variant() {
        let priority: PlanEntryPriority = serde_json::from_str("\"urgent\"").unwrap();
        assert_eq!(priority, PlanEntryPriority::Other("urgent".to_string()));
        assert_eq!(serde_json::to_value(&priority).unwrap(), "urgent");
    }

    #[test]
    fn plan_entry_status_preserves_unknown_variant() {
        let status: PlanEntryStatus = serde_json::from_str("\"blocked\"").unwrap();
        assert_eq!(status, PlanEntryStatus::Other("blocked".to_string()));
        assert_eq!(serde_json::to_value(&status).unwrap(), "blocked");
    }

    #[test]
    fn plan_update_content_preserves_unknown_variant() {
        let content: PlanUpdateContent = serde_json::from_value(serde_json::json!({
            "type": "_timeline",
            "id": "plan-1",
            "events": []
        }))
        .unwrap();

        let PlanUpdateContent::Other(unknown) = content else {
            panic!("expected unknown plan update content");
        };

        assert_eq!(unknown.type_, "_timeline");
        assert_eq!(unknown.fields.get("id"), Some(&serde_json::json!("plan-1")));
        assert_eq!(
            serde_json::to_value(PlanUpdateContent::Other(unknown)).unwrap(),
            serde_json::json!({
                "type": "_timeline",
                "id": "plan-1",
                "events": []
            })
        );
    }

    #[test]
    fn plan_update_content_does_not_hide_malformed_known_variant() {
        assert!(
            serde_json::from_value::<PlanUpdateContent>(serde_json::json!({
                "type": "items"
            }))
            .is_err()
        );
    }
}
