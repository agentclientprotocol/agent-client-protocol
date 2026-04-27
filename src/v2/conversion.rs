//! Explicit conversion helpers for experimenting with ACP v2 while SDKs still speak v1.
//!
//! The v2 protocol types currently mirror v1. The conversions below intentionally move
//! values field-by-field and variant-by-variant instead of serializing through JSON so
//! future v2 shape changes have obvious edit points.

use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    hash::{BuildHasher, Hash},
    path::PathBuf,
    sync::Arc,
};

use serde_json::value::RawValue;

use crate::version::ProtocolVersion;

/// Result type returned by protocol conversion helpers.
pub type Result<T> = std::result::Result<T, ProtocolConversionError>;

/// Error returned when converting between v1 and v2 protocol type namespaces fails.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProtocolConversionError {
    message: String,
}

impl ProtocolConversionError {
    /// Creates a conversion error with a human-readable message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the human-readable conversion error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ProtocolConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ProtocolConversionError {}

/// Converts a value from the v2 draft type namespace into the matching v1 type.
pub trait IntoV1 {
    /// The corresponding v1 type.
    type Output;

    /// Converts this value into the corresponding v1 type.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolConversionError`] when a value cannot be represented in v1.
    fn into_v1(self) -> Result<Self::Output>;
}

/// Converts a value from the v1 type namespace into the matching v2 draft type.
pub trait IntoV2 {
    /// The corresponding v2 draft type.
    type Output;

    /// Converts this value into the corresponding v2 draft type.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolConversionError`] when a value cannot be represented in v2.
    fn into_v2(self) -> Result<Self::Output>;
}

/// Converts a v2 draft value into the corresponding v1 value type.
///
/// # Errors
///
/// Returns [`ProtocolConversionError`] when a value cannot be represented in v1.
pub fn v2_to_v1<T>(value: T) -> Result<T::Output>
where
    T: IntoV1,
{
    value.into_v1()
}

/// Converts a v1 value into the corresponding v2 draft value type.
///
/// # Errors
///
/// Returns [`ProtocolConversionError`] when a value cannot be represented in v2.
pub fn v1_to_v2<T>(value: T) -> Result<T::Output>
where
    T: IntoV2,
{
    value.into_v2()
}

macro_rules! identity_conversion {
    ($($ty:ty),* $(,)?) => {
        $(
            impl IntoV1 for $ty {
                type Output = Self;

                fn into_v1(self) -> Result<Self::Output> {
                    Ok(self)
                }
            }

            impl IntoV2 for $ty {
                type Output = Self;

                fn into_v2(self) -> Result<Self::Output> {
                    Ok(self)
                }
            }
        )*
    };
}

identity_conversion!(
    bool, f32, f64, i16, i32, i64, i8, isize, String, u16, u32, u64, u8, usize,
    &'static str, Arc<RawValue>, Arc<str>, PathBuf, ProtocolVersion,
    serde_json::Map<String, serde_json::Value>, serde_json::Value,
);

impl<T> IntoV1 for Option<T>
where
    T: IntoV1,
{
    type Output = Option<T::Output>;
    fn into_v1(self) -> Result<Self::Output> {
        self.map(IntoV1::into_v1).transpose()
    }
}

impl<T> IntoV2 for Option<T>
where
    T: IntoV2,
{
    type Output = Option<T::Output>;
    fn into_v2(self) -> Result<Self::Output> {
        self.map(IntoV2::into_v2).transpose()
    }
}

impl<T> IntoV1 for Vec<T>
where
    T: IntoV1,
{
    type Output = Vec<T::Output>;
    fn into_v1(self) -> Result<Self::Output> {
        self.into_iter().map(IntoV1::into_v1).collect()
    }
}

impl<T> IntoV2 for Vec<T>
where
    T: IntoV2,
{
    type Output = Vec<T::Output>;
    fn into_v2(self) -> Result<Self::Output> {
        self.into_iter().map(IntoV2::into_v2).collect()
    }
}

impl<K, V> IntoV1 for BTreeMap<K, V>
where
    K: IntoV1,
    K::Output: Ord,
    V: IntoV1,
{
    type Output = BTreeMap<K::Output, V::Output>;
    fn into_v1(self) -> Result<Self::Output> {
        self.into_iter()
            .map(|(key, value)| Ok((key.into_v1()?, value.into_v1()?)))
            .collect()
    }
}

impl<K, V> IntoV2 for BTreeMap<K, V>
where
    K: IntoV2,
    K::Output: Ord,
    V: IntoV2,
{
    type Output = BTreeMap<K::Output, V::Output>;
    fn into_v2(self) -> Result<Self::Output> {
        self.into_iter()
            .map(|(key, value)| Ok((key.into_v2()?, value.into_v2()?)))
            .collect()
    }
}

impl<K, V, S> IntoV1 for HashMap<K, V, S>
where
    K: IntoV1,
    K::Output: Eq + Hash,
    V: IntoV1,
    S: BuildHasher,
{
    type Output = HashMap<K::Output, V::Output>;
    fn into_v1(self) -> Result<Self::Output> {
        self.into_iter()
            .map(|(key, value)| Ok((key.into_v1()?, value.into_v1()?)))
            .collect()
    }
}

impl<K, V, S> IntoV2 for HashMap<K, V, S>
where
    K: IntoV2,
    K::Output: Eq + Hash,
    V: IntoV2,
    S: BuildHasher,
{
    type Output = HashMap<K::Output, V::Output>;
    fn into_v2(self) -> Result<Self::Output> {
        self.into_iter()
            .map(|(key, value)| Ok((key.into_v2()?, value.into_v2()?)))
            .collect()
    }
}

impl<T> IntoV1 for super::MaybeUndefined<T>
where
    T: IntoV1,
{
    type Output = crate::v1::MaybeUndefined<T::Output>;
    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Undefined => crate::v1::MaybeUndefined::Undefined,
            Self::Null => crate::v1::MaybeUndefined::Null,
            Self::Value(value) => crate::v1::MaybeUndefined::Value(value.into_v1()?),
        })
    }
}

impl<T> IntoV2 for crate::v1::MaybeUndefined<T>
where
    T: IntoV2,
{
    type Output = super::MaybeUndefined<T::Output>;
    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Undefined => super::MaybeUndefined::Undefined,
            Self::Null => super::MaybeUndefined::Null,
            Self::Value(value) => super::MaybeUndefined::Value(value.into_v2()?),
        })
    }
}

impl<Params> IntoV1 for super::Request<Params>
where
    Params: IntoV1,
{
    type Output = crate::v1::Request<Params::Output>;
    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Request {
            id: self.id.into_v1()?,
            method: self.method,
            params: self.params.into_v1()?,
        })
    }
}

impl<Params> IntoV2 for crate::v1::Request<Params>
where
    Params: IntoV2,
{
    type Output = super::Request<Params::Output>;
    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Request {
            id: self.id.into_v2()?,
            method: self.method,
            params: self.params.into_v2()?,
        })
    }
}

impl<Params> IntoV1 for super::Notification<Params>
where
    Params: IntoV1,
{
    type Output = crate::v1::Notification<Params::Output>;
    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Notification {
            method: self.method,
            params: self.params.into_v1()?,
        })
    }
}

impl<Params> IntoV2 for crate::v1::Notification<Params>
where
    Params: IntoV2,
{
    type Output = super::Notification<Params::Output>;
    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Notification {
            method: self.method,
            params: self.params.into_v2()?,
        })
    }
}

impl<Response> IntoV1 for super::Response<Response>
where
    Response: IntoV1,
{
    type Output = crate::v1::Response<Response::Output>;
    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Result { id, result } => crate::v1::Response::Result {
                id: id.into_v1()?,
                result: result.into_v1()?,
            },
            Self::Error { id, error } => crate::v1::Response::Error {
                id: id.into_v1()?,
                error: error.into_v1()?,
            },
        })
    }
}

impl<Response> IntoV2 for crate::v1::Response<Response>
where
    Response: IntoV2,
{
    type Output = super::Response<Response::Output>;
    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Result { id, result } => super::Response::Result {
                id: id.into_v2()?,
                result: result.into_v2()?,
            },
            Self::Error { id, error } => super::Response::Error {
                id: id.into_v2()?,
                error: error.into_v2()?,
            },
        })
    }
}

impl<Message> IntoV1 for super::JsonRpcMessage<Message>
where
    Message: IntoV1,
{
    type Output = crate::v1::JsonRpcMessage<Message::Output>;
    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::JsonRpcMessage::wrap(
            self.into_inner().into_v1()?,
        ))
    }
}

impl<Message> IntoV2 for crate::v1::JsonRpcMessage<Message>
where
    Message: IntoV2,
{
    type Output = super::JsonRpcMessage<Message::Output>;
    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::JsonRpcMessage::wrap(self.into_inner().into_v2()?))
    }
}

impl IntoV1 for super::SessionId {
    type Output = crate::v1::SessionId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionId(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::SessionId {
    type Output = super::SessionId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionId(self.0.into_v2()?))
    }
}

impl IntoV1 for super::Plan {
    type Output = crate::v1::Plan;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Plan {
            entries: self.entries.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Plan {
    type Output = super::Plan;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Plan {
            entries: self.entries.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PlanEntry {
    type Output = crate::v1::PlanEntry;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::PlanEntry {
            content: self.content.into_v1()?,
            priority: self.priority.into_v1()?,
            status: self.status.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::PlanEntry {
    type Output = super::PlanEntry;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::PlanEntry {
            content: self.content.into_v2()?,
            priority: self.priority.into_v2()?,
            status: self.status.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PlanEntryPriority {
    type Output = crate::v1::PlanEntryPriority;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::High => crate::v1::PlanEntryPriority::High,
            Self::Medium => crate::v1::PlanEntryPriority::Medium,
            Self::Low => crate::v1::PlanEntryPriority::Low,
        })
    }
}

impl IntoV2 for crate::v1::PlanEntryPriority {
    type Output = super::PlanEntryPriority;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::High => super::PlanEntryPriority::High,
            Self::Medium => super::PlanEntryPriority::Medium,
            Self::Low => super::PlanEntryPriority::Low,
        })
    }
}

impl IntoV1 for super::PlanEntryStatus {
    type Output = crate::v1::PlanEntryStatus;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Pending => crate::v1::PlanEntryStatus::Pending,
            Self::InProgress => crate::v1::PlanEntryStatus::InProgress,
            Self::Completed => crate::v1::PlanEntryStatus::Completed,
        })
    }
}

impl IntoV2 for crate::v1::PlanEntryStatus {
    type Output = super::PlanEntryStatus;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Pending => super::PlanEntryStatus::Pending,
            Self::InProgress => super::PlanEntryStatus::InProgress,
            Self::Completed => super::PlanEntryStatus::Completed,
        })
    }
}

#[cfg(feature = "unstable_cancel_request")]
impl IntoV1 for super::CancelRequestNotification {
    type Output = crate::v1::CancelRequestNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CancelRequestNotification {
            request_id: self.request_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_cancel_request")]
impl IntoV2 for crate::v1::CancelRequestNotification {
    type Output = super::CancelRequestNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CancelRequestNotification {
            request_id: self.request_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_cancel_request")]
impl IntoV1 for super::GeneralMethodNames {
    type Output = crate::v1::GeneralMethodNames;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::GeneralMethodNames {
            cancel_request: self.cancel_request.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_cancel_request")]
impl IntoV2 for crate::v1::GeneralMethodNames {
    type Output = super::GeneralMethodNames;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::GeneralMethodNames {
            cancel_request: self.cancel_request.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_cancel_request")]
impl IntoV1 for super::ProtocolLevelNotification {
    type Output = crate::v1::ProtocolLevelNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::CancelRequestNotification(value) => {
                crate::v1::ProtocolLevelNotification::CancelRequestNotification(value.into_v1()?)
            }
        })
    }
}

#[cfg(feature = "unstable_cancel_request")]
impl IntoV2 for crate::v1::ProtocolLevelNotification {
    type Output = super::ProtocolLevelNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::CancelRequestNotification(value) => {
                super::ProtocolLevelNotification::CancelRequestNotification(value.into_v2()?)
            }
        })
    }
}

impl IntoV1 for super::SessionNotification {
    type Output = crate::v1::SessionNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionNotification {
            session_id: self.session_id.into_v1()?,
            update: self.update.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionNotification {
    type Output = super::SessionNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionNotification {
            session_id: self.session_id.into_v2()?,
            update: self.update.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionUpdate {
    type Output = crate::v1::SessionUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::UserMessageChunk(value) => {
                crate::v1::SessionUpdate::UserMessageChunk(value.into_v1()?)
            }
            Self::AgentMessageChunk(value) => {
                crate::v1::SessionUpdate::AgentMessageChunk(value.into_v1()?)
            }
            Self::AgentThoughtChunk(value) => {
                crate::v1::SessionUpdate::AgentThoughtChunk(value.into_v1()?)
            }
            Self::ToolCall(value) => crate::v1::SessionUpdate::ToolCall(value.into_v1()?),
            Self::ToolCallUpdate(value) => {
                crate::v1::SessionUpdate::ToolCallUpdate(value.into_v1()?)
            }
            Self::Plan(value) => crate::v1::SessionUpdate::Plan(value.into_v1()?),
            Self::AvailableCommandsUpdate(value) => {
                crate::v1::SessionUpdate::AvailableCommandsUpdate(value.into_v1()?)
            }
            Self::CurrentModeUpdate(value) => {
                crate::v1::SessionUpdate::CurrentModeUpdate(value.into_v1()?)
            }
            Self::ConfigOptionUpdate(value) => {
                crate::v1::SessionUpdate::ConfigOptionUpdate(value.into_v1()?)
            }
            Self::SessionInfoUpdate(value) => {
                crate::v1::SessionUpdate::SessionInfoUpdate(value.into_v1()?)
            }
            #[cfg(feature = "unstable_session_usage")]
            Self::UsageUpdate(value) => crate::v1::SessionUpdate::UsageUpdate(value.into_v1()?),
        })
    }
}

impl IntoV2 for crate::v1::SessionUpdate {
    type Output = super::SessionUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::UserMessageChunk(value) => {
                super::SessionUpdate::UserMessageChunk(value.into_v2()?)
            }
            Self::AgentMessageChunk(value) => {
                super::SessionUpdate::AgentMessageChunk(value.into_v2()?)
            }
            Self::AgentThoughtChunk(value) => {
                super::SessionUpdate::AgentThoughtChunk(value.into_v2()?)
            }
            Self::ToolCall(value) => super::SessionUpdate::ToolCall(value.into_v2()?),
            Self::ToolCallUpdate(value) => super::SessionUpdate::ToolCallUpdate(value.into_v2()?),
            Self::Plan(value) => super::SessionUpdate::Plan(value.into_v2()?),
            Self::AvailableCommandsUpdate(value) => {
                super::SessionUpdate::AvailableCommandsUpdate(value.into_v2()?)
            }
            Self::CurrentModeUpdate(value) => {
                super::SessionUpdate::CurrentModeUpdate(value.into_v2()?)
            }
            Self::ConfigOptionUpdate(value) => {
                super::SessionUpdate::ConfigOptionUpdate(value.into_v2()?)
            }
            Self::SessionInfoUpdate(value) => {
                super::SessionUpdate::SessionInfoUpdate(value.into_v2()?)
            }
            #[cfg(feature = "unstable_session_usage")]
            Self::UsageUpdate(value) => super::SessionUpdate::UsageUpdate(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::CurrentModeUpdate {
    type Output = crate::v1::CurrentModeUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CurrentModeUpdate {
            current_mode_id: self.current_mode_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::CurrentModeUpdate {
    type Output = super::CurrentModeUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CurrentModeUpdate {
            current_mode_id: self.current_mode_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ConfigOptionUpdate {
    type Output = crate::v1::ConfigOptionUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ConfigOptionUpdate {
            config_options: self.config_options.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ConfigOptionUpdate {
    type Output = super::ConfigOptionUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ConfigOptionUpdate {
            config_options: self.config_options.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionInfoUpdate {
    type Output = crate::v1::SessionInfoUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionInfoUpdate {
            title: self.title.into_v1()?,
            updated_at: self.updated_at.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionInfoUpdate {
    type Output = super::SessionInfoUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionInfoUpdate {
            title: self.title.into_v2()?,
            updated_at: self.updated_at.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_usage")]
impl IntoV1 for super::UsageUpdate {
    type Output = crate::v1::UsageUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::UsageUpdate {
            used: self.used.into_v1()?,
            size: self.size.into_v1()?,
            cost: self.cost.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_usage")]
impl IntoV2 for crate::v1::UsageUpdate {
    type Output = super::UsageUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::UsageUpdate {
            used: self.used.into_v2()?,
            size: self.size.into_v2()?,
            cost: self.cost.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_usage")]
impl IntoV1 for super::Cost {
    type Output = crate::v1::Cost;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Cost {
            amount: self.amount.into_v1()?,
            currency: self.currency.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_usage")]
impl IntoV2 for crate::v1::Cost {
    type Output = super::Cost;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Cost {
            amount: self.amount.into_v2()?,
            currency: self.currency.into_v2()?,
        })
    }
}

impl IntoV1 for super::ContentChunk {
    type Output = crate::v1::ContentChunk;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ContentChunk {
            content: self.content.into_v1()?,
            #[cfg(feature = "unstable_message_id")]
            message_id: self.message_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ContentChunk {
    type Output = super::ContentChunk;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ContentChunk {
            content: self.content.into_v2()?,
            #[cfg(feature = "unstable_message_id")]
            message_id: self.message_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AvailableCommandsUpdate {
    type Output = crate::v1::AvailableCommandsUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AvailableCommandsUpdate {
            available_commands: self.available_commands.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AvailableCommandsUpdate {
    type Output = super::AvailableCommandsUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AvailableCommandsUpdate {
            available_commands: self.available_commands.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AvailableCommand {
    type Output = crate::v1::AvailableCommand;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AvailableCommand {
            name: self.name.into_v1()?,
            description: self.description.into_v1()?,
            input: self.input.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AvailableCommand {
    type Output = super::AvailableCommand;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AvailableCommand {
            name: self.name.into_v2()?,
            description: self.description.into_v2()?,
            input: self.input.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AvailableCommandInput {
    type Output = crate::v1::AvailableCommandInput;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Unstructured(value) => {
                crate::v1::AvailableCommandInput::Unstructured(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::AvailableCommandInput {
    type Output = super::AvailableCommandInput;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Unstructured(value) => {
                super::AvailableCommandInput::Unstructured(value.into_v2()?)
            }
        })
    }
}

impl IntoV1 for super::UnstructuredCommandInput {
    type Output = crate::v1::UnstructuredCommandInput;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::UnstructuredCommandInput {
            hint: self.hint.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::UnstructuredCommandInput {
    type Output = super::UnstructuredCommandInput;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::UnstructuredCommandInput {
            hint: self.hint.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::RequestPermissionRequest {
    type Output = crate::v1::RequestPermissionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::RequestPermissionRequest {
            session_id: self.session_id.into_v1()?,
            tool_call: self.tool_call.into_v1()?,
            options: self.options.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::RequestPermissionRequest {
    type Output = super::RequestPermissionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::RequestPermissionRequest {
            session_id: self.session_id.into_v2()?,
            tool_call: self.tool_call.into_v2()?,
            options: self.options.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PermissionOption {
    type Output = crate::v1::PermissionOption;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::PermissionOption {
            option_id: self.option_id.into_v1()?,
            name: self.name.into_v1()?,
            kind: self.kind.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::PermissionOption {
    type Output = super::PermissionOption;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::PermissionOption {
            option_id: self.option_id.into_v2()?,
            name: self.name.into_v2()?,
            kind: self.kind.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PermissionOptionId {
    type Output = crate::v1::PermissionOptionId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::PermissionOptionId(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::PermissionOptionId {
    type Output = super::PermissionOptionId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::PermissionOptionId(self.0.into_v2()?))
    }
}

impl IntoV1 for super::PermissionOptionKind {
    type Output = crate::v1::PermissionOptionKind;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::AllowOnce => crate::v1::PermissionOptionKind::AllowOnce,
            Self::AllowAlways => crate::v1::PermissionOptionKind::AllowAlways,
            Self::RejectOnce => crate::v1::PermissionOptionKind::RejectOnce,
            Self::RejectAlways => crate::v1::PermissionOptionKind::RejectAlways,
        })
    }
}

impl IntoV2 for crate::v1::PermissionOptionKind {
    type Output = super::PermissionOptionKind;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::AllowOnce => super::PermissionOptionKind::AllowOnce,
            Self::AllowAlways => super::PermissionOptionKind::AllowAlways,
            Self::RejectOnce => super::PermissionOptionKind::RejectOnce,
            Self::RejectAlways => super::PermissionOptionKind::RejectAlways,
        })
    }
}

impl IntoV1 for super::RequestPermissionResponse {
    type Output = crate::v1::RequestPermissionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::RequestPermissionResponse {
            outcome: self.outcome.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::RequestPermissionResponse {
    type Output = super::RequestPermissionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::RequestPermissionResponse {
            outcome: self.outcome.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::RequestPermissionOutcome {
    type Output = crate::v1::RequestPermissionOutcome;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Cancelled => crate::v1::RequestPermissionOutcome::Cancelled,
            Self::Selected(value) => {
                crate::v1::RequestPermissionOutcome::Selected(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::RequestPermissionOutcome {
    type Output = super::RequestPermissionOutcome;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Cancelled => super::RequestPermissionOutcome::Cancelled,
            Self::Selected(value) => super::RequestPermissionOutcome::Selected(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::SelectedPermissionOutcome {
    type Output = crate::v1::SelectedPermissionOutcome;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SelectedPermissionOutcome {
            option_id: self.option_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SelectedPermissionOutcome {
    type Output = super::SelectedPermissionOutcome;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SelectedPermissionOutcome {
            option_id: self.option_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::WriteTextFileRequest {
    type Output = crate::v1::WriteTextFileRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::WriteTextFileRequest {
            session_id: self.session_id.into_v1()?,
            path: self.path.into_v1()?,
            content: self.content.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::WriteTextFileRequest {
    type Output = super::WriteTextFileRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::WriteTextFileRequest {
            session_id: self.session_id.into_v2()?,
            path: self.path.into_v2()?,
            content: self.content.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::WriteTextFileResponse {
    type Output = crate::v1::WriteTextFileResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::WriteTextFileResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::WriteTextFileResponse {
    type Output = super::WriteTextFileResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::WriteTextFileResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ReadTextFileRequest {
    type Output = crate::v1::ReadTextFileRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ReadTextFileRequest {
            session_id: self.session_id.into_v1()?,
            path: self.path.into_v1()?,
            line: self.line.into_v1()?,
            limit: self.limit.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ReadTextFileRequest {
    type Output = super::ReadTextFileRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ReadTextFileRequest {
            session_id: self.session_id.into_v2()?,
            path: self.path.into_v2()?,
            line: self.line.into_v2()?,
            limit: self.limit.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ReadTextFileResponse {
    type Output = crate::v1::ReadTextFileResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ReadTextFileResponse {
            content: self.content.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ReadTextFileResponse {
    type Output = super::ReadTextFileResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ReadTextFileResponse {
            content: self.content.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::TerminalId {
    type Output = crate::v1::TerminalId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::TerminalId(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::TerminalId {
    type Output = super::TerminalId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::TerminalId(self.0.into_v2()?))
    }
}

impl IntoV1 for super::CreateTerminalRequest {
    type Output = crate::v1::CreateTerminalRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CreateTerminalRequest {
            session_id: self.session_id.into_v1()?,
            command: self.command.into_v1()?,
            args: self.args.into_v1()?,
            env: self.env.into_v1()?,
            cwd: self.cwd.into_v1()?,
            output_byte_limit: self.output_byte_limit.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::CreateTerminalRequest {
    type Output = super::CreateTerminalRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CreateTerminalRequest {
            session_id: self.session_id.into_v2()?,
            command: self.command.into_v2()?,
            args: self.args.into_v2()?,
            env: self.env.into_v2()?,
            cwd: self.cwd.into_v2()?,
            output_byte_limit: self.output_byte_limit.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::CreateTerminalResponse {
    type Output = crate::v1::CreateTerminalResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CreateTerminalResponse {
            terminal_id: self.terminal_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::CreateTerminalResponse {
    type Output = super::CreateTerminalResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CreateTerminalResponse {
            terminal_id: self.terminal_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::TerminalOutputRequest {
    type Output = crate::v1::TerminalOutputRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::TerminalOutputRequest {
            session_id: self.session_id.into_v1()?,
            terminal_id: self.terminal_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::TerminalOutputRequest {
    type Output = super::TerminalOutputRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::TerminalOutputRequest {
            session_id: self.session_id.into_v2()?,
            terminal_id: self.terminal_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::TerminalOutputResponse {
    type Output = crate::v1::TerminalOutputResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::TerminalOutputResponse {
            output: self.output.into_v1()?,
            truncated: self.truncated.into_v1()?,
            exit_status: self.exit_status.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::TerminalOutputResponse {
    type Output = super::TerminalOutputResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::TerminalOutputResponse {
            output: self.output.into_v2()?,
            truncated: self.truncated.into_v2()?,
            exit_status: self.exit_status.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ReleaseTerminalRequest {
    type Output = crate::v1::ReleaseTerminalRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ReleaseTerminalRequest {
            session_id: self.session_id.into_v1()?,
            terminal_id: self.terminal_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ReleaseTerminalRequest {
    type Output = super::ReleaseTerminalRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ReleaseTerminalRequest {
            session_id: self.session_id.into_v2()?,
            terminal_id: self.terminal_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ReleaseTerminalResponse {
    type Output = crate::v1::ReleaseTerminalResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ReleaseTerminalResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ReleaseTerminalResponse {
    type Output = super::ReleaseTerminalResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ReleaseTerminalResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::KillTerminalRequest {
    type Output = crate::v1::KillTerminalRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::KillTerminalRequest {
            session_id: self.session_id.into_v1()?,
            terminal_id: self.terminal_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::KillTerminalRequest {
    type Output = super::KillTerminalRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::KillTerminalRequest {
            session_id: self.session_id.into_v2()?,
            terminal_id: self.terminal_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::KillTerminalResponse {
    type Output = crate::v1::KillTerminalResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::KillTerminalResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::KillTerminalResponse {
    type Output = super::KillTerminalResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::KillTerminalResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::WaitForTerminalExitRequest {
    type Output = crate::v1::WaitForTerminalExitRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::WaitForTerminalExitRequest {
            session_id: self.session_id.into_v1()?,
            terminal_id: self.terminal_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::WaitForTerminalExitRequest {
    type Output = super::WaitForTerminalExitRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::WaitForTerminalExitRequest {
            session_id: self.session_id.into_v2()?,
            terminal_id: self.terminal_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::WaitForTerminalExitResponse {
    type Output = crate::v1::WaitForTerminalExitResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::WaitForTerminalExitResponse {
            exit_status: self.exit_status.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::WaitForTerminalExitResponse {
    type Output = super::WaitForTerminalExitResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::WaitForTerminalExitResponse {
            exit_status: self.exit_status.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::TerminalExitStatus {
    type Output = crate::v1::TerminalExitStatus;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::TerminalExitStatus {
            exit_code: self.exit_code.into_v1()?,
            signal: self.signal.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::TerminalExitStatus {
    type Output = super::TerminalExitStatus;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::TerminalExitStatus {
            exit_code: self.exit_code.into_v2()?,
            signal: self.signal.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ClientCapabilities {
    type Output = crate::v1::ClientCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ClientCapabilities {
            fs: self.fs.into_v1()?,
            terminal: self.terminal.into_v1()?,
            #[cfg(feature = "unstable_auth_methods")]
            auth: self.auth.into_v1()?,
            #[cfg(feature = "unstable_elicitation")]
            elicitation: self.elicitation.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            nes: self.nes.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            position_encodings: self.position_encodings.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ClientCapabilities {
    type Output = super::ClientCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ClientCapabilities {
            fs: self.fs.into_v2()?,
            terminal: self.terminal.into_v2()?,
            #[cfg(feature = "unstable_auth_methods")]
            auth: self.auth.into_v2()?,
            #[cfg(feature = "unstable_elicitation")]
            elicitation: self.elicitation.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            nes: self.nes.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            position_encodings: self.position_encodings.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV1 for super::AuthCapabilities {
    type Output = crate::v1::AuthCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AuthCapabilities {
            terminal: self.terminal.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV2 for crate::v1::AuthCapabilities {
    type Output = super::AuthCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AuthCapabilities {
            terminal: self.terminal.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::FileSystemCapabilities {
    type Output = crate::v1::FileSystemCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::FileSystemCapabilities {
            read_text_file: self.read_text_file.into_v1()?,
            write_text_file: self.write_text_file.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::FileSystemCapabilities {
    type Output = super::FileSystemCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::FileSystemCapabilities {
            read_text_file: self.read_text_file.into_v2()?,
            write_text_file: self.write_text_file.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ClientMethodNames {
    type Output = crate::v1::ClientMethodNames;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ClientMethodNames {
            session_request_permission: self.session_request_permission.into_v1()?,
            session_update: self.session_update.into_v1()?,
            fs_write_text_file: self.fs_write_text_file.into_v1()?,
            fs_read_text_file: self.fs_read_text_file.into_v1()?,
            terminal_create: self.terminal_create.into_v1()?,
            terminal_output: self.terminal_output.into_v1()?,
            terminal_release: self.terminal_release.into_v1()?,
            terminal_wait_for_exit: self.terminal_wait_for_exit.into_v1()?,
            terminal_kill: self.terminal_kill.into_v1()?,
            #[cfg(feature = "unstable_elicitation")]
            elicitation_create: self.elicitation_create.into_v1()?,
            #[cfg(feature = "unstable_elicitation")]
            elicitation_complete: self.elicitation_complete.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ClientMethodNames {
    type Output = super::ClientMethodNames;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ClientMethodNames {
            session_request_permission: self.session_request_permission.into_v2()?,
            session_update: self.session_update.into_v2()?,
            fs_write_text_file: self.fs_write_text_file.into_v2()?,
            fs_read_text_file: self.fs_read_text_file.into_v2()?,
            terminal_create: self.terminal_create.into_v2()?,
            terminal_output: self.terminal_output.into_v2()?,
            terminal_release: self.terminal_release.into_v2()?,
            terminal_wait_for_exit: self.terminal_wait_for_exit.into_v2()?,
            terminal_kill: self.terminal_kill.into_v2()?,
            #[cfg(feature = "unstable_elicitation")]
            elicitation_create: self.elicitation_create.into_v2()?,
            #[cfg(feature = "unstable_elicitation")]
            elicitation_complete: self.elicitation_complete.into_v2()?,
        })
    }
}

impl IntoV1 for super::AgentRequest {
    type Output = crate::v1::AgentRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::WriteTextFileRequest(value) => {
                crate::v1::AgentRequest::WriteTextFileRequest(value.into_v1()?)
            }
            Self::ReadTextFileRequest(value) => {
                crate::v1::AgentRequest::ReadTextFileRequest(value.into_v1()?)
            }
            Self::RequestPermissionRequest(value) => {
                crate::v1::AgentRequest::RequestPermissionRequest(value.into_v1()?)
            }
            Self::CreateTerminalRequest(value) => {
                crate::v1::AgentRequest::CreateTerminalRequest(value.into_v1()?)
            }
            Self::TerminalOutputRequest(value) => {
                crate::v1::AgentRequest::TerminalOutputRequest(value.into_v1()?)
            }
            Self::ReleaseTerminalRequest(value) => {
                crate::v1::AgentRequest::ReleaseTerminalRequest(value.into_v1()?)
            }
            Self::WaitForTerminalExitRequest(value) => {
                crate::v1::AgentRequest::WaitForTerminalExitRequest(value.into_v1()?)
            }
            Self::KillTerminalRequest(value) => {
                crate::v1::AgentRequest::KillTerminalRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CreateElicitationRequest(value) => {
                crate::v1::AgentRequest::CreateElicitationRequest(value.into_v1()?)
            }
            Self::ExtMethodRequest(value) => {
                crate::v1::AgentRequest::ExtMethodRequest(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::AgentRequest {
    type Output = super::AgentRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::WriteTextFileRequest(value) => {
                super::AgentRequest::WriteTextFileRequest(value.into_v2()?)
            }
            Self::ReadTextFileRequest(value) => {
                super::AgentRequest::ReadTextFileRequest(value.into_v2()?)
            }
            Self::RequestPermissionRequest(value) => {
                super::AgentRequest::RequestPermissionRequest(value.into_v2()?)
            }
            Self::CreateTerminalRequest(value) => {
                super::AgentRequest::CreateTerminalRequest(value.into_v2()?)
            }
            Self::TerminalOutputRequest(value) => {
                super::AgentRequest::TerminalOutputRequest(value.into_v2()?)
            }
            Self::ReleaseTerminalRequest(value) => {
                super::AgentRequest::ReleaseTerminalRequest(value.into_v2()?)
            }
            Self::WaitForTerminalExitRequest(value) => {
                super::AgentRequest::WaitForTerminalExitRequest(value.into_v2()?)
            }
            Self::KillTerminalRequest(value) => {
                super::AgentRequest::KillTerminalRequest(value.into_v2()?)
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CreateElicitationRequest(value) => {
                super::AgentRequest::CreateElicitationRequest(value.into_v2()?)
            }
            Self::ExtMethodRequest(value) => {
                super::AgentRequest::ExtMethodRequest(value.into_v2()?)
            }
        })
    }
}

impl IntoV1 for super::ClientResponse {
    type Output = crate::v1::ClientResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::WriteTextFileResponse(value) => {
                crate::v1::ClientResponse::WriteTextFileResponse(value.into_v1()?)
            }
            Self::ReadTextFileResponse(value) => {
                crate::v1::ClientResponse::ReadTextFileResponse(value.into_v1()?)
            }
            Self::RequestPermissionResponse(value) => {
                crate::v1::ClientResponse::RequestPermissionResponse(value.into_v1()?)
            }
            Self::CreateTerminalResponse(value) => {
                crate::v1::ClientResponse::CreateTerminalResponse(value.into_v1()?)
            }
            Self::TerminalOutputResponse(value) => {
                crate::v1::ClientResponse::TerminalOutputResponse(value.into_v1()?)
            }
            Self::ReleaseTerminalResponse(value) => {
                crate::v1::ClientResponse::ReleaseTerminalResponse(value.into_v1()?)
            }
            Self::WaitForTerminalExitResponse(value) => {
                crate::v1::ClientResponse::WaitForTerminalExitResponse(value.into_v1()?)
            }
            Self::KillTerminalResponse(value) => {
                crate::v1::ClientResponse::KillTerminalResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CreateElicitationResponse(value) => {
                crate::v1::ClientResponse::CreateElicitationResponse(value.into_v1()?)
            }
            Self::ExtMethodResponse(value) => {
                crate::v1::ClientResponse::ExtMethodResponse(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::ClientResponse {
    type Output = super::ClientResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::WriteTextFileResponse(value) => {
                super::ClientResponse::WriteTextFileResponse(value.into_v2()?)
            }
            Self::ReadTextFileResponse(value) => {
                super::ClientResponse::ReadTextFileResponse(value.into_v2()?)
            }
            Self::RequestPermissionResponse(value) => {
                super::ClientResponse::RequestPermissionResponse(value.into_v2()?)
            }
            Self::CreateTerminalResponse(value) => {
                super::ClientResponse::CreateTerminalResponse(value.into_v2()?)
            }
            Self::TerminalOutputResponse(value) => {
                super::ClientResponse::TerminalOutputResponse(value.into_v2()?)
            }
            Self::ReleaseTerminalResponse(value) => {
                super::ClientResponse::ReleaseTerminalResponse(value.into_v2()?)
            }
            Self::WaitForTerminalExitResponse(value) => {
                super::ClientResponse::WaitForTerminalExitResponse(value.into_v2()?)
            }
            Self::KillTerminalResponse(value) => {
                super::ClientResponse::KillTerminalResponse(value.into_v2()?)
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CreateElicitationResponse(value) => {
                super::ClientResponse::CreateElicitationResponse(value.into_v2()?)
            }
            Self::ExtMethodResponse(value) => {
                super::ClientResponse::ExtMethodResponse(value.into_v2()?)
            }
        })
    }
}

impl IntoV1 for super::AgentNotification {
    type Output = crate::v1::AgentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::SessionNotification(value) => {
                crate::v1::AgentNotification::SessionNotification(value.into_v1()?)
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CompleteElicitationNotification(value) => {
                crate::v1::AgentNotification::CompleteElicitationNotification(value.into_v1()?)
            }
            Self::ExtNotification(value) => {
                crate::v1::AgentNotification::ExtNotification(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::AgentNotification {
    type Output = super::AgentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::SessionNotification(value) => {
                super::AgentNotification::SessionNotification(value.into_v2()?)
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CompleteElicitationNotification(value) => {
                super::AgentNotification::CompleteElicitationNotification(value.into_v2()?)
            }
            Self::ExtNotification(value) => {
                super::AgentNotification::ExtNotification(value.into_v2()?)
            }
        })
    }
}

impl IntoV1 for super::Error {
    type Output = crate::v1::Error;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Error {
            code: self.code.into_v1()?,
            message: self.message.into_v1()?,
            data: self.data.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Error {
    type Output = super::Error;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Error {
            code: self.code.into_v2()?,
            message: self.message.into_v2()?,
            data: self.data.into_v2()?,
        })
    }
}

impl IntoV1 for super::ErrorCode {
    type Output = crate::v1::ErrorCode;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(i32::from(self).into())
    }
}

impl IntoV2 for crate::v1::ErrorCode {
    type Output = super::ErrorCode;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(i32::from(self).into())
    }
}

impl IntoV1 for super::ExtRequest {
    type Output = crate::v1::ExtRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ExtRequest {
            method: self.method.into_v1()?,
            params: self.params.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ExtRequest {
    type Output = super::ExtRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ExtRequest {
            method: self.method.into_v2()?,
            params: self.params.into_v2()?,
        })
    }
}

impl IntoV1 for super::ExtResponse {
    type Output = crate::v1::ExtResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ExtResponse(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::ExtResponse {
    type Output = super::ExtResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ExtResponse(self.0.into_v2()?))
    }
}

impl IntoV1 for super::ExtNotification {
    type Output = crate::v1::ExtNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ExtNotification {
            method: self.method.into_v1()?,
            params: self.params.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ExtNotification {
    type Output = super::ExtNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ExtNotification {
            method: self.method.into_v2()?,
            params: self.params.into_v2()?,
        })
    }
}

impl IntoV1 for super::ToolCall {
    type Output = crate::v1::ToolCall;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ToolCall {
            tool_call_id: self.tool_call_id.into_v1()?,
            title: self.title.into_v1()?,
            kind: self.kind.into_v1()?,
            status: self.status.into_v1()?,
            content: self.content.into_v1()?,
            locations: self.locations.into_v1()?,
            raw_input: self.raw_input.into_v1()?,
            raw_output: self.raw_output.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ToolCall {
    type Output = super::ToolCall;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ToolCall {
            tool_call_id: self.tool_call_id.into_v2()?,
            title: self.title.into_v2()?,
            kind: self.kind.into_v2()?,
            status: self.status.into_v2()?,
            content: self.content.into_v2()?,
            locations: self.locations.into_v2()?,
            raw_input: self.raw_input.into_v2()?,
            raw_output: self.raw_output.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ToolCallUpdate {
    type Output = crate::v1::ToolCallUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ToolCallUpdate {
            tool_call_id: self.tool_call_id.into_v1()?,
            fields: self.fields.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ToolCallUpdate {
    type Output = super::ToolCallUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ToolCallUpdate {
            tool_call_id: self.tool_call_id.into_v2()?,
            fields: self.fields.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ToolCallUpdateFields {
    type Output = crate::v1::ToolCallUpdateFields;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ToolCallUpdateFields {
            kind: self.kind.into_v1()?,
            status: self.status.into_v1()?,
            title: self.title.into_v1()?,
            content: self.content.into_v1()?,
            locations: self.locations.into_v1()?,
            raw_input: self.raw_input.into_v1()?,
            raw_output: self.raw_output.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ToolCallUpdateFields {
    type Output = super::ToolCallUpdateFields;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ToolCallUpdateFields {
            kind: self.kind.into_v2()?,
            status: self.status.into_v2()?,
            title: self.title.into_v2()?,
            content: self.content.into_v2()?,
            locations: self.locations.into_v2()?,
            raw_input: self.raw_input.into_v2()?,
            raw_output: self.raw_output.into_v2()?,
        })
    }
}

impl IntoV1 for super::ToolCallId {
    type Output = crate::v1::ToolCallId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ToolCallId(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::ToolCallId {
    type Output = super::ToolCallId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ToolCallId(self.0.into_v2()?))
    }
}

impl IntoV1 for super::ToolKind {
    type Output = crate::v1::ToolKind;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Read => crate::v1::ToolKind::Read,
            Self::Edit => crate::v1::ToolKind::Edit,
            Self::Delete => crate::v1::ToolKind::Delete,
            Self::Move => crate::v1::ToolKind::Move,
            Self::Search => crate::v1::ToolKind::Search,
            Self::Execute => crate::v1::ToolKind::Execute,
            Self::Think => crate::v1::ToolKind::Think,
            Self::Fetch => crate::v1::ToolKind::Fetch,
            Self::SwitchMode => crate::v1::ToolKind::SwitchMode,
            Self::Other => crate::v1::ToolKind::Other,
        })
    }
}

impl IntoV2 for crate::v1::ToolKind {
    type Output = super::ToolKind;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Read => super::ToolKind::Read,
            Self::Edit => super::ToolKind::Edit,
            Self::Delete => super::ToolKind::Delete,
            Self::Move => super::ToolKind::Move,
            Self::Search => super::ToolKind::Search,
            Self::Execute => super::ToolKind::Execute,
            Self::Think => super::ToolKind::Think,
            Self::Fetch => super::ToolKind::Fetch,
            Self::SwitchMode => super::ToolKind::SwitchMode,
            Self::Other => super::ToolKind::Other,
        })
    }
}

impl IntoV1 for super::ToolCallStatus {
    type Output = crate::v1::ToolCallStatus;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Pending => crate::v1::ToolCallStatus::Pending,
            Self::InProgress => crate::v1::ToolCallStatus::InProgress,
            Self::Completed => crate::v1::ToolCallStatus::Completed,
            Self::Failed => crate::v1::ToolCallStatus::Failed,
        })
    }
}

impl IntoV2 for crate::v1::ToolCallStatus {
    type Output = super::ToolCallStatus;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Pending => super::ToolCallStatus::Pending,
            Self::InProgress => super::ToolCallStatus::InProgress,
            Self::Completed => super::ToolCallStatus::Completed,
            Self::Failed => super::ToolCallStatus::Failed,
        })
    }
}

impl IntoV1 for super::ToolCallContent {
    type Output = crate::v1::ToolCallContent;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Content(value) => crate::v1::ToolCallContent::Content(value.into_v1()?),
            Self::Diff(value) => crate::v1::ToolCallContent::Diff(value.into_v1()?),
            Self::Terminal(value) => crate::v1::ToolCallContent::Terminal(value.into_v1()?),
        })
    }
}

impl IntoV2 for crate::v1::ToolCallContent {
    type Output = super::ToolCallContent;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Content(value) => super::ToolCallContent::Content(value.into_v2()?),
            Self::Diff(value) => super::ToolCallContent::Diff(value.into_v2()?),
            Self::Terminal(value) => super::ToolCallContent::Terminal(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::Content {
    type Output = crate::v1::Content;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Content {
            content: self.content.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Content {
    type Output = super::Content;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Content {
            content: self.content.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::Terminal {
    type Output = crate::v1::Terminal;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Terminal {
            terminal_id: self.terminal_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Terminal {
    type Output = super::Terminal;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Terminal {
            terminal_id: self.terminal_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::Diff {
    type Output = crate::v1::Diff;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Diff {
            path: self.path.into_v1()?,
            old_text: self.old_text.into_v1()?,
            new_text: self.new_text.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Diff {
    type Output = super::Diff;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Diff {
            path: self.path.into_v2()?,
            old_text: self.old_text.into_v2()?,
            new_text: self.new_text.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ToolCallLocation {
    type Output = crate::v1::ToolCallLocation;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ToolCallLocation {
            path: self.path.into_v1()?,
            line: self.line.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ToolCallLocation {
    type Output = super::ToolCallLocation;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ToolCallLocation {
            path: self.path.into_v2()?,
            line: self.line.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::InitializeRequest {
    type Output = crate::v1::InitializeRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::InitializeRequest {
            protocol_version: self.protocol_version.into_v1()?,
            client_capabilities: self.client_capabilities.into_v1()?,
            client_info: self.client_info.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::InitializeRequest {
    type Output = super::InitializeRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::InitializeRequest {
            protocol_version: self.protocol_version.into_v2()?,
            client_capabilities: self.client_capabilities.into_v2()?,
            client_info: self.client_info.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::InitializeResponse {
    type Output = crate::v1::InitializeResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::InitializeResponse {
            protocol_version: self.protocol_version.into_v1()?,
            agent_capabilities: self.agent_capabilities.into_v1()?,
            auth_methods: self.auth_methods.into_v1()?,
            agent_info: self.agent_info.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::InitializeResponse {
    type Output = super::InitializeResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::InitializeResponse {
            protocol_version: self.protocol_version.into_v2()?,
            agent_capabilities: self.agent_capabilities.into_v2()?,
            auth_methods: self.auth_methods.into_v2()?,
            agent_info: self.agent_info.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::Implementation {
    type Output = crate::v1::Implementation;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Implementation {
            name: self.name.into_v1()?,
            title: self.title.into_v1()?,
            version: self.version.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Implementation {
    type Output = super::Implementation;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Implementation {
            name: self.name.into_v2()?,
            title: self.title.into_v2()?,
            version: self.version.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AuthenticateRequest {
    type Output = crate::v1::AuthenticateRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AuthenticateRequest {
            method_id: self.method_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AuthenticateRequest {
    type Output = super::AuthenticateRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AuthenticateRequest {
            method_id: self.method_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AuthenticateResponse {
    type Output = crate::v1::AuthenticateResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AuthenticateResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AuthenticateResponse {
    type Output = super::AuthenticateResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AuthenticateResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_logout")]
impl IntoV1 for super::LogoutRequest {
    type Output = crate::v1::LogoutRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::LogoutRequest {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_logout")]
impl IntoV2 for crate::v1::LogoutRequest {
    type Output = super::LogoutRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::LogoutRequest {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_logout")]
impl IntoV1 for super::LogoutResponse {
    type Output = crate::v1::LogoutResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::LogoutResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_logout")]
impl IntoV2 for crate::v1::LogoutResponse {
    type Output = super::LogoutResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::LogoutResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_logout")]
impl IntoV1 for super::AgentAuthCapabilities {
    type Output = crate::v1::AgentAuthCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AgentAuthCapabilities {
            logout: self.logout.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_logout")]
impl IntoV2 for crate::v1::AgentAuthCapabilities {
    type Output = super::AgentAuthCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AgentAuthCapabilities {
            logout: self.logout.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_logout")]
impl IntoV1 for super::LogoutCapabilities {
    type Output = crate::v1::LogoutCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::LogoutCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_logout")]
impl IntoV2 for crate::v1::LogoutCapabilities {
    type Output = super::LogoutCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::LogoutCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AuthMethodId {
    type Output = crate::v1::AuthMethodId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AuthMethodId(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::AuthMethodId {
    type Output = super::AuthMethodId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AuthMethodId(self.0.into_v2()?))
    }
}

impl IntoV1 for super::AuthMethod {
    type Output = crate::v1::AuthMethod;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            #[cfg(feature = "unstable_auth_methods")]
            Self::EnvVar(value) => crate::v1::AuthMethod::EnvVar(value.into_v1()?),
            #[cfg(feature = "unstable_auth_methods")]
            Self::Terminal(value) => crate::v1::AuthMethod::Terminal(value.into_v1()?),
            Self::Agent(value) => crate::v1::AuthMethod::Agent(value.into_v1()?),
        })
    }
}

impl IntoV2 for crate::v1::AuthMethod {
    type Output = super::AuthMethod;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            #[cfg(feature = "unstable_auth_methods")]
            Self::EnvVar(value) => super::AuthMethod::EnvVar(value.into_v2()?),
            #[cfg(feature = "unstable_auth_methods")]
            Self::Terminal(value) => super::AuthMethod::Terminal(value.into_v2()?),
            Self::Agent(value) => super::AuthMethod::Agent(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::AuthMethodAgent {
    type Output = crate::v1::AuthMethodAgent;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AuthMethodAgent {
            id: self.id.into_v1()?,
            name: self.name.into_v1()?,
            description: self.description.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AuthMethodAgent {
    type Output = super::AuthMethodAgent;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AuthMethodAgent {
            id: self.id.into_v2()?,
            name: self.name.into_v2()?,
            description: self.description.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV1 for super::AuthMethodEnvVar {
    type Output = crate::v1::AuthMethodEnvVar;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AuthMethodEnvVar {
            id: self.id.into_v1()?,
            name: self.name.into_v1()?,
            description: self.description.into_v1()?,
            vars: self.vars.into_v1()?,
            link: self.link.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV2 for crate::v1::AuthMethodEnvVar {
    type Output = super::AuthMethodEnvVar;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AuthMethodEnvVar {
            id: self.id.into_v2()?,
            name: self.name.into_v2()?,
            description: self.description.into_v2()?,
            vars: self.vars.into_v2()?,
            link: self.link.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV1 for super::AuthEnvVar {
    type Output = crate::v1::AuthEnvVar;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AuthEnvVar {
            name: self.name.into_v1()?,
            label: self.label.into_v1()?,
            secret: self.secret.into_v1()?,
            optional: self.optional.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV2 for crate::v1::AuthEnvVar {
    type Output = super::AuthEnvVar;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AuthEnvVar {
            name: self.name.into_v2()?,
            label: self.label.into_v2()?,
            secret: self.secret.into_v2()?,
            optional: self.optional.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV1 for super::AuthMethodTerminal {
    type Output = crate::v1::AuthMethodTerminal;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AuthMethodTerminal {
            id: self.id.into_v1()?,
            name: self.name.into_v1()?,
            description: self.description.into_v1()?,
            args: self.args.into_v1()?,
            env: self.env.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV2 for crate::v1::AuthMethodTerminal {
    type Output = super::AuthMethodTerminal;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AuthMethodTerminal {
            id: self.id.into_v2()?,
            name: self.name.into_v2()?,
            description: self.description.into_v2()?,
            args: self.args.into_v2()?,
            env: self.env.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::NewSessionRequest {
    type Output = crate::v1::NewSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NewSessionRequest {
            cwd: self.cwd.into_v1()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v1()?,
            mcp_servers: self.mcp_servers.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::NewSessionRequest {
    type Output = super::NewSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NewSessionRequest {
            cwd: self.cwd.into_v2()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v2()?,
            mcp_servers: self.mcp_servers.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::NewSessionResponse {
    type Output = crate::v1::NewSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NewSessionResponse {
            session_id: self.session_id.into_v1()?,
            modes: self.modes.into_v1()?,
            #[cfg(feature = "unstable_session_model")]
            models: self.models.into_v1()?,
            config_options: self.config_options.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::NewSessionResponse {
    type Output = super::NewSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NewSessionResponse {
            session_id: self.session_id.into_v2()?,
            modes: self.modes.into_v2()?,
            #[cfg(feature = "unstable_session_model")]
            models: self.models.into_v2()?,
            config_options: self.config_options.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::LoadSessionRequest {
    type Output = crate::v1::LoadSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::LoadSessionRequest {
            mcp_servers: self.mcp_servers.into_v1()?,
            cwd: self.cwd.into_v1()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v1()?,
            session_id: self.session_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::LoadSessionRequest {
    type Output = super::LoadSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::LoadSessionRequest {
            mcp_servers: self.mcp_servers.into_v2()?,
            cwd: self.cwd.into_v2()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v2()?,
            session_id: self.session_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::LoadSessionResponse {
    type Output = crate::v1::LoadSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::LoadSessionResponse {
            modes: self.modes.into_v1()?,
            #[cfg(feature = "unstable_session_model")]
            models: self.models.into_v1()?,
            config_options: self.config_options.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::LoadSessionResponse {
    type Output = super::LoadSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::LoadSessionResponse {
            modes: self.modes.into_v2()?,
            #[cfg(feature = "unstable_session_model")]
            models: self.models.into_v2()?,
            config_options: self.config_options.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV1 for super::ForkSessionRequest {
    type Output = crate::v1::ForkSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ForkSessionRequest {
            session_id: self.session_id.into_v1()?,
            cwd: self.cwd.into_v1()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v1()?,
            mcp_servers: self.mcp_servers.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV2 for crate::v1::ForkSessionRequest {
    type Output = super::ForkSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ForkSessionRequest {
            session_id: self.session_id.into_v2()?,
            cwd: self.cwd.into_v2()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v2()?,
            mcp_servers: self.mcp_servers.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV1 for super::ForkSessionResponse {
    type Output = crate::v1::ForkSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ForkSessionResponse {
            session_id: self.session_id.into_v1()?,
            modes: self.modes.into_v1()?,
            #[cfg(feature = "unstable_session_model")]
            models: self.models.into_v1()?,
            config_options: self.config_options.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV2 for crate::v1::ForkSessionResponse {
    type Output = super::ForkSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ForkSessionResponse {
            session_id: self.session_id.into_v2()?,
            modes: self.modes.into_v2()?,
            #[cfg(feature = "unstable_session_model")]
            models: self.models.into_v2()?,
            config_options: self.config_options.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ResumeSessionRequest {
    type Output = crate::v1::ResumeSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ResumeSessionRequest {
            session_id: self.session_id.into_v1()?,
            cwd: self.cwd.into_v1()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v1()?,
            mcp_servers: self.mcp_servers.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ResumeSessionRequest {
    type Output = super::ResumeSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ResumeSessionRequest {
            session_id: self.session_id.into_v2()?,
            cwd: self.cwd.into_v2()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v2()?,
            mcp_servers: self.mcp_servers.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ResumeSessionResponse {
    type Output = crate::v1::ResumeSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ResumeSessionResponse {
            modes: self.modes.into_v1()?,
            #[cfg(feature = "unstable_session_model")]
            models: self.models.into_v1()?,
            config_options: self.config_options.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ResumeSessionResponse {
    type Output = super::ResumeSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ResumeSessionResponse {
            modes: self.modes.into_v2()?,
            #[cfg(feature = "unstable_session_model")]
            models: self.models.into_v2()?,
            config_options: self.config_options.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::CloseSessionRequest {
    type Output = crate::v1::CloseSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CloseSessionRequest {
            session_id: self.session_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::CloseSessionRequest {
    type Output = super::CloseSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CloseSessionRequest {
            session_id: self.session_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::CloseSessionResponse {
    type Output = crate::v1::CloseSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CloseSessionResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::CloseSessionResponse {
    type Output = super::CloseSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CloseSessionResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ListSessionsRequest {
    type Output = crate::v1::ListSessionsRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ListSessionsRequest {
            cwd: self.cwd.into_v1()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v1()?,
            cursor: self.cursor.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ListSessionsRequest {
    type Output = super::ListSessionsRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ListSessionsRequest {
            cwd: self.cwd.into_v2()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v2()?,
            cursor: self.cursor.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ListSessionsResponse {
    type Output = crate::v1::ListSessionsResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ListSessionsResponse {
            sessions: self.sessions.into_v1()?,
            next_cursor: self.next_cursor.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ListSessionsResponse {
    type Output = super::ListSessionsResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ListSessionsResponse {
            sessions: self.sessions.into_v2()?,
            next_cursor: self.next_cursor.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionInfo {
    type Output = crate::v1::SessionInfo;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionInfo {
            session_id: self.session_id.into_v1()?,
            cwd: self.cwd.into_v1()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v1()?,
            title: self.title.into_v1()?,
            updated_at: self.updated_at.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionInfo {
    type Output = super::SessionInfo;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionInfo {
            session_id: self.session_id.into_v2()?,
            cwd: self.cwd.into_v2()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v2()?,
            title: self.title.into_v2()?,
            updated_at: self.updated_at.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionModeState {
    type Output = crate::v1::SessionModeState;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionModeState {
            current_mode_id: self.current_mode_id.into_v1()?,
            available_modes: self.available_modes.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionModeState {
    type Output = super::SessionModeState;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionModeState {
            current_mode_id: self.current_mode_id.into_v2()?,
            available_modes: self.available_modes.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionMode {
    type Output = crate::v1::SessionMode;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionMode {
            id: self.id.into_v1()?,
            name: self.name.into_v1()?,
            description: self.description.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionMode {
    type Output = super::SessionMode;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionMode {
            id: self.id.into_v2()?,
            name: self.name.into_v2()?,
            description: self.description.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionModeId {
    type Output = crate::v1::SessionModeId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionModeId(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::SessionModeId {
    type Output = super::SessionModeId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionModeId(self.0.into_v2()?))
    }
}

impl IntoV1 for super::SetSessionModeRequest {
    type Output = crate::v1::SetSessionModeRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SetSessionModeRequest {
            session_id: self.session_id.into_v1()?,
            mode_id: self.mode_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SetSessionModeRequest {
    type Output = super::SetSessionModeRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SetSessionModeRequest {
            session_id: self.session_id.into_v2()?,
            mode_id: self.mode_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SetSessionModeResponse {
    type Output = crate::v1::SetSessionModeResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SetSessionModeResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SetSessionModeResponse {
    type Output = super::SetSessionModeResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SetSessionModeResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionConfigId {
    type Output = crate::v1::SessionConfigId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionConfigId(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::SessionConfigId {
    type Output = super::SessionConfigId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionConfigId(self.0.into_v2()?))
    }
}

impl IntoV1 for super::SessionConfigValueId {
    type Output = crate::v1::SessionConfigValueId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionConfigValueId(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::SessionConfigValueId {
    type Output = super::SessionConfigValueId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionConfigValueId(self.0.into_v2()?))
    }
}

impl IntoV1 for super::SessionConfigGroupId {
    type Output = crate::v1::SessionConfigGroupId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionConfigGroupId(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::SessionConfigGroupId {
    type Output = super::SessionConfigGroupId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionConfigGroupId(self.0.into_v2()?))
    }
}

impl IntoV1 for super::SessionConfigSelectOption {
    type Output = crate::v1::SessionConfigSelectOption;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionConfigSelectOption {
            value: self.value.into_v1()?,
            name: self.name.into_v1()?,
            description: self.description.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigSelectOption {
    type Output = super::SessionConfigSelectOption;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionConfigSelectOption {
            value: self.value.into_v2()?,
            name: self.name.into_v2()?,
            description: self.description.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionConfigSelectGroup {
    type Output = crate::v1::SessionConfigSelectGroup;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionConfigSelectGroup {
            group: self.group.into_v1()?,
            name: self.name.into_v1()?,
            options: self.options.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigSelectGroup {
    type Output = super::SessionConfigSelectGroup;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionConfigSelectGroup {
            group: self.group.into_v2()?,
            name: self.name.into_v2()?,
            options: self.options.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionConfigSelectOptions {
    type Output = crate::v1::SessionConfigSelectOptions;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Ungrouped(value) => {
                crate::v1::SessionConfigSelectOptions::Ungrouped(value.into_v1()?)
            }
            Self::Grouped(value) => {
                crate::v1::SessionConfigSelectOptions::Grouped(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigSelectOptions {
    type Output = super::SessionConfigSelectOptions;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Ungrouped(value) => {
                super::SessionConfigSelectOptions::Ungrouped(value.into_v2()?)
            }
            Self::Grouped(value) => super::SessionConfigSelectOptions::Grouped(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::SessionConfigSelect {
    type Output = crate::v1::SessionConfigSelect;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionConfigSelect {
            current_value: self.current_value.into_v1()?,
            options: self.options.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigSelect {
    type Output = super::SessionConfigSelect;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionConfigSelect {
            current_value: self.current_value.into_v2()?,
            options: self.options.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_boolean_config")]
impl IntoV1 for super::SessionConfigBoolean {
    type Output = crate::v1::SessionConfigBoolean;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionConfigBoolean {
            current_value: self.current_value.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_boolean_config")]
impl IntoV2 for crate::v1::SessionConfigBoolean {
    type Output = super::SessionConfigBoolean;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionConfigBoolean {
            current_value: self.current_value.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionConfigOptionCategory {
    type Output = crate::v1::SessionConfigOptionCategory;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Mode => crate::v1::SessionConfigOptionCategory::Mode,
            Self::Model => crate::v1::SessionConfigOptionCategory::Model,
            Self::ThoughtLevel => crate::v1::SessionConfigOptionCategory::ThoughtLevel,
            Self::Other(value) => crate::v1::SessionConfigOptionCategory::Other(value.into_v1()?),
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigOptionCategory {
    type Output = super::SessionConfigOptionCategory;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Mode => super::SessionConfigOptionCategory::Mode,
            Self::Model => super::SessionConfigOptionCategory::Model,
            Self::ThoughtLevel => super::SessionConfigOptionCategory::ThoughtLevel,
            Self::Other(value) => super::SessionConfigOptionCategory::Other(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::SessionConfigKind {
    type Output = crate::v1::SessionConfigKind;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Select(value) => crate::v1::SessionConfigKind::Select(value.into_v1()?),
            #[cfg(feature = "unstable_boolean_config")]
            Self::Boolean(value) => crate::v1::SessionConfigKind::Boolean(value.into_v1()?),
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigKind {
    type Output = super::SessionConfigKind;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Select(value) => super::SessionConfigKind::Select(value.into_v2()?),
            #[cfg(feature = "unstable_boolean_config")]
            Self::Boolean(value) => super::SessionConfigKind::Boolean(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::SessionConfigOption {
    type Output = crate::v1::SessionConfigOption;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionConfigOption {
            id: self.id.into_v1()?,
            name: self.name.into_v1()?,
            description: self.description.into_v1()?,
            category: self.category.into_v1()?,
            kind: self.kind.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigOption {
    type Output = super::SessionConfigOption;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionConfigOption {
            id: self.id.into_v2()?,
            name: self.name.into_v2()?,
            description: self.description.into_v2()?,
            category: self.category.into_v2()?,
            kind: self.kind.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_boolean_config")]
impl IntoV1 for super::SessionConfigOptionValue {
    type Output = crate::v1::SessionConfigOptionValue;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Boolean { value } => crate::v1::SessionConfigOptionValue::Boolean {
                value: value.into_v1()?,
            },
            Self::ValueId { value } => crate::v1::SessionConfigOptionValue::ValueId {
                value: value.into_v1()?,
            },
        })
    }
}

#[cfg(feature = "unstable_boolean_config")]
impl IntoV2 for crate::v1::SessionConfigOptionValue {
    type Output = super::SessionConfigOptionValue;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Boolean { value } => super::SessionConfigOptionValue::Boolean {
                value: value.into_v2()?,
            },
            Self::ValueId { value } => super::SessionConfigOptionValue::ValueId {
                value: value.into_v2()?,
            },
        })
    }
}

impl IntoV1 for super::SetSessionConfigOptionRequest {
    type Output = crate::v1::SetSessionConfigOptionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SetSessionConfigOptionRequest {
            session_id: self.session_id.into_v1()?,
            config_id: self.config_id.into_v1()?,
            #[cfg(feature = "unstable_boolean_config")]
            value: self.value.into_v1()?,
            #[cfg(not(feature = "unstable_boolean_config"))]
            value: self.value.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SetSessionConfigOptionRequest {
    type Output = super::SetSessionConfigOptionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SetSessionConfigOptionRequest {
            session_id: self.session_id.into_v2()?,
            config_id: self.config_id.into_v2()?,
            #[cfg(feature = "unstable_boolean_config")]
            value: self.value.into_v2()?,
            #[cfg(not(feature = "unstable_boolean_config"))]
            value: self.value.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SetSessionConfigOptionResponse {
    type Output = crate::v1::SetSessionConfigOptionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SetSessionConfigOptionResponse {
            config_options: self.config_options.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SetSessionConfigOptionResponse {
    type Output = super::SetSessionConfigOptionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SetSessionConfigOptionResponse {
            config_options: self.config_options.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::McpServer {
    type Output = crate::v1::McpServer;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Http(value) => crate::v1::McpServer::Http(value.into_v1()?),
            Self::Sse(value) => crate::v1::McpServer::Sse(value.into_v1()?),
            Self::Stdio(value) => crate::v1::McpServer::Stdio(value.into_v1()?),
        })
    }
}

impl IntoV2 for crate::v1::McpServer {
    type Output = super::McpServer;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Http(value) => super::McpServer::Http(value.into_v2()?),
            Self::Sse(value) => super::McpServer::Sse(value.into_v2()?),
            Self::Stdio(value) => super::McpServer::Stdio(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::McpServerHttp {
    type Output = crate::v1::McpServerHttp;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::McpServerHttp {
            name: self.name.into_v1()?,
            url: self.url.into_v1()?,
            headers: self.headers.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::McpServerHttp {
    type Output = super::McpServerHttp;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::McpServerHttp {
            name: self.name.into_v2()?,
            url: self.url.into_v2()?,
            headers: self.headers.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::McpServerSse {
    type Output = crate::v1::McpServerSse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::McpServerSse {
            name: self.name.into_v1()?,
            url: self.url.into_v1()?,
            headers: self.headers.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::McpServerSse {
    type Output = super::McpServerSse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::McpServerSse {
            name: self.name.into_v2()?,
            url: self.url.into_v2()?,
            headers: self.headers.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::McpServerStdio {
    type Output = crate::v1::McpServerStdio;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::McpServerStdio {
            name: self.name.into_v1()?,
            command: self.command.into_v1()?,
            args: self.args.into_v1()?,
            env: self.env.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::McpServerStdio {
    type Output = super::McpServerStdio;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::McpServerStdio {
            name: self.name.into_v2()?,
            command: self.command.into_v2()?,
            args: self.args.into_v2()?,
            env: self.env.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::EnvVariable {
    type Output = crate::v1::EnvVariable;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::EnvVariable {
            name: self.name.into_v1()?,
            value: self.value.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::EnvVariable {
    type Output = super::EnvVariable;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::EnvVariable {
            name: self.name.into_v2()?,
            value: self.value.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::HttpHeader {
    type Output = crate::v1::HttpHeader;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::HttpHeader {
            name: self.name.into_v1()?,
            value: self.value.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::HttpHeader {
    type Output = super::HttpHeader;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::HttpHeader {
            name: self.name.into_v2()?,
            value: self.value.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PromptRequest {
    type Output = crate::v1::PromptRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::PromptRequest {
            session_id: self.session_id.into_v1()?,
            #[cfg(feature = "unstable_message_id")]
            message_id: self.message_id.into_v1()?,
            prompt: self.prompt.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::PromptRequest {
    type Output = super::PromptRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::PromptRequest {
            session_id: self.session_id.into_v2()?,
            #[cfg(feature = "unstable_message_id")]
            message_id: self.message_id.into_v2()?,
            prompt: self.prompt.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PromptResponse {
    type Output = crate::v1::PromptResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::PromptResponse {
            #[cfg(feature = "unstable_message_id")]
            user_message_id: self.user_message_id.into_v1()?,
            stop_reason: self.stop_reason.into_v1()?,
            #[cfg(feature = "unstable_session_usage")]
            usage: self.usage.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::PromptResponse {
    type Output = super::PromptResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::PromptResponse {
            #[cfg(feature = "unstable_message_id")]
            user_message_id: self.user_message_id.into_v2()?,
            stop_reason: self.stop_reason.into_v2()?,
            #[cfg(feature = "unstable_session_usage")]
            usage: self.usage.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::StopReason {
    type Output = crate::v1::StopReason;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::EndTurn => crate::v1::StopReason::EndTurn,
            Self::MaxTokens => crate::v1::StopReason::MaxTokens,
            Self::MaxTurnRequests => crate::v1::StopReason::MaxTurnRequests,
            Self::Refusal => crate::v1::StopReason::Refusal,
            Self::Cancelled => crate::v1::StopReason::Cancelled,
        })
    }
}

impl IntoV2 for crate::v1::StopReason {
    type Output = super::StopReason;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::EndTurn => super::StopReason::EndTurn,
            Self::MaxTokens => super::StopReason::MaxTokens,
            Self::MaxTurnRequests => super::StopReason::MaxTurnRequests,
            Self::Refusal => super::StopReason::Refusal,
            Self::Cancelled => super::StopReason::Cancelled,
        })
    }
}

#[cfg(feature = "unstable_session_usage")]
impl IntoV1 for super::Usage {
    type Output = crate::v1::Usage;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Usage {
            total_tokens: self.total_tokens.into_v1()?,
            input_tokens: self.input_tokens.into_v1()?,
            output_tokens: self.output_tokens.into_v1()?,
            thought_tokens: self.thought_tokens.into_v1()?,
            cached_read_tokens: self.cached_read_tokens.into_v1()?,
            cached_write_tokens: self.cached_write_tokens.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_usage")]
impl IntoV2 for crate::v1::Usage {
    type Output = super::Usage;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Usage {
            total_tokens: self.total_tokens.into_v2()?,
            input_tokens: self.input_tokens.into_v2()?,
            output_tokens: self.output_tokens.into_v2()?,
            thought_tokens: self.thought_tokens.into_v2()?,
            cached_read_tokens: self.cached_read_tokens.into_v2()?,
            cached_write_tokens: self.cached_write_tokens.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_model")]
impl IntoV1 for super::SessionModelState {
    type Output = crate::v1::SessionModelState;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionModelState {
            current_model_id: self.current_model_id.into_v1()?,
            available_models: self.available_models.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_model")]
impl IntoV2 for crate::v1::SessionModelState {
    type Output = super::SessionModelState;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionModelState {
            current_model_id: self.current_model_id.into_v2()?,
            available_models: self.available_models.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_model")]
impl IntoV1 for super::ModelId {
    type Output = crate::v1::ModelId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ModelId(self.0.into_v1()?))
    }
}

#[cfg(feature = "unstable_session_model")]
impl IntoV2 for crate::v1::ModelId {
    type Output = super::ModelId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ModelId(self.0.into_v2()?))
    }
}

#[cfg(feature = "unstable_session_model")]
impl IntoV1 for super::ModelInfo {
    type Output = crate::v1::ModelInfo;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ModelInfo {
            model_id: self.model_id.into_v1()?,
            name: self.name.into_v1()?,
            description: self.description.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_model")]
impl IntoV2 for crate::v1::ModelInfo {
    type Output = super::ModelInfo;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ModelInfo {
            model_id: self.model_id.into_v2()?,
            name: self.name.into_v2()?,
            description: self.description.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_model")]
impl IntoV1 for super::SetSessionModelRequest {
    type Output = crate::v1::SetSessionModelRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SetSessionModelRequest {
            session_id: self.session_id.into_v1()?,
            model_id: self.model_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_model")]
impl IntoV2 for crate::v1::SetSessionModelRequest {
    type Output = super::SetSessionModelRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SetSessionModelRequest {
            session_id: self.session_id.into_v2()?,
            model_id: self.model_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_model")]
impl IntoV1 for super::SetSessionModelResponse {
    type Output = crate::v1::SetSessionModelResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SetSessionModelResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_model")]
impl IntoV2 for crate::v1::SetSessionModelResponse {
    type Output = super::SetSessionModelResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SetSessionModelResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::LlmProtocol {
    type Output = crate::v1::LlmProtocol;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Anthropic => crate::v1::LlmProtocol::Anthropic,
            Self::OpenAi => crate::v1::LlmProtocol::OpenAi,
            Self::Azure => crate::v1::LlmProtocol::Azure,
            Self::Vertex => crate::v1::LlmProtocol::Vertex,
            Self::Bedrock => crate::v1::LlmProtocol::Bedrock,
            Self::Other(value) => crate::v1::LlmProtocol::Other(value.into_v1()?),
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::LlmProtocol {
    type Output = super::LlmProtocol;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Anthropic => super::LlmProtocol::Anthropic,
            Self::OpenAi => super::LlmProtocol::OpenAi,
            Self::Azure => super::LlmProtocol::Azure,
            Self::Vertex => super::LlmProtocol::Vertex,
            Self::Bedrock => super::LlmProtocol::Bedrock,
            Self::Other(value) => super::LlmProtocol::Other(value.into_v2()?),
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::ProviderCurrentConfig {
    type Output = crate::v1::ProviderCurrentConfig;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ProviderCurrentConfig {
            api_type: self.api_type.into_v1()?,
            base_url: self.base_url.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::ProviderCurrentConfig {
    type Output = super::ProviderCurrentConfig;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ProviderCurrentConfig {
            api_type: self.api_type.into_v2()?,
            base_url: self.base_url.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::ProviderInfo {
    type Output = crate::v1::ProviderInfo;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ProviderInfo {
            id: self.id.into_v1()?,
            supported: self.supported.into_v1()?,
            required: self.required.into_v1()?,
            current: self.current.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::ProviderInfo {
    type Output = super::ProviderInfo;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ProviderInfo {
            id: self.id.into_v2()?,
            supported: self.supported.into_v2()?,
            required: self.required.into_v2()?,
            current: self.current.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::ListProvidersRequest {
    type Output = crate::v1::ListProvidersRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ListProvidersRequest {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::ListProvidersRequest {
    type Output = super::ListProvidersRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ListProvidersRequest {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::ListProvidersResponse {
    type Output = crate::v1::ListProvidersResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ListProvidersResponse {
            providers: self.providers.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::ListProvidersResponse {
    type Output = super::ListProvidersResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ListProvidersResponse {
            providers: self.providers.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::SetProvidersRequest {
    type Output = crate::v1::SetProvidersRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SetProvidersRequest {
            id: self.id.into_v1()?,
            api_type: self.api_type.into_v1()?,
            base_url: self.base_url.into_v1()?,
            headers: self.headers.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::SetProvidersRequest {
    type Output = super::SetProvidersRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SetProvidersRequest {
            id: self.id.into_v2()?,
            api_type: self.api_type.into_v2()?,
            base_url: self.base_url.into_v2()?,
            headers: self.headers.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::SetProvidersResponse {
    type Output = crate::v1::SetProvidersResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SetProvidersResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::SetProvidersResponse {
    type Output = super::SetProvidersResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SetProvidersResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::DisableProvidersRequest {
    type Output = crate::v1::DisableProvidersRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::DisableProvidersRequest {
            id: self.id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::DisableProvidersRequest {
    type Output = super::DisableProvidersRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::DisableProvidersRequest {
            id: self.id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::DisableProvidersResponse {
    type Output = crate::v1::DisableProvidersResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::DisableProvidersResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::DisableProvidersResponse {
    type Output = super::DisableProvidersResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::DisableProvidersResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AgentCapabilities {
    type Output = crate::v1::AgentCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AgentCapabilities {
            load_session: self.load_session.into_v1()?,
            prompt_capabilities: self.prompt_capabilities.into_v1()?,
            mcp_capabilities: self.mcp_capabilities.into_v1()?,
            session_capabilities: self.session_capabilities.into_v1()?,
            #[cfg(feature = "unstable_logout")]
            auth: self.auth.into_v1()?,
            #[cfg(feature = "unstable_llm_providers")]
            providers: self.providers.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            nes: self.nes.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            position_encoding: self.position_encoding.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AgentCapabilities {
    type Output = super::AgentCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AgentCapabilities {
            load_session: self.load_session.into_v2()?,
            prompt_capabilities: self.prompt_capabilities.into_v2()?,
            mcp_capabilities: self.mcp_capabilities.into_v2()?,
            session_capabilities: self.session_capabilities.into_v2()?,
            #[cfg(feature = "unstable_logout")]
            auth: self.auth.into_v2()?,
            #[cfg(feature = "unstable_llm_providers")]
            providers: self.providers.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            nes: self.nes.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            position_encoding: self.position_encoding.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::ProvidersCapabilities {
    type Output = crate::v1::ProvidersCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ProvidersCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::ProvidersCapabilities {
    type Output = super::ProvidersCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ProvidersCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionCapabilities {
    type Output = crate::v1::SessionCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionCapabilities {
            list: self.list.into_v1()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v1()?,
            #[cfg(feature = "unstable_session_fork")]
            fork: self.fork.into_v1()?,
            resume: self.resume.into_v1()?,
            close: self.close.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionCapabilities {
    type Output = super::SessionCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionCapabilities {
            list: self.list.into_v2()?,
            #[cfg(feature = "unstable_session_additional_directories")]
            additional_directories: self.additional_directories.into_v2()?,
            #[cfg(feature = "unstable_session_fork")]
            fork: self.fork.into_v2()?,
            resume: self.resume.into_v2()?,
            close: self.close.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionListCapabilities {
    type Output = crate::v1::SessionListCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionListCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionListCapabilities {
    type Output = super::SessionListCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionListCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_additional_directories")]
impl IntoV1 for super::SessionAdditionalDirectoriesCapabilities {
    type Output = crate::v1::SessionAdditionalDirectoriesCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionAdditionalDirectoriesCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_additional_directories")]
impl IntoV2 for crate::v1::SessionAdditionalDirectoriesCapabilities {
    type Output = super::SessionAdditionalDirectoriesCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionAdditionalDirectoriesCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV1 for super::SessionForkCapabilities {
    type Output = crate::v1::SessionForkCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionForkCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV2 for crate::v1::SessionForkCapabilities {
    type Output = super::SessionForkCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionForkCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionResumeCapabilities {
    type Output = crate::v1::SessionResumeCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionResumeCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionResumeCapabilities {
    type Output = super::SessionResumeCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionResumeCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionCloseCapabilities {
    type Output = crate::v1::SessionCloseCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SessionCloseCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionCloseCapabilities {
    type Output = super::SessionCloseCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SessionCloseCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PromptCapabilities {
    type Output = crate::v1::PromptCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::PromptCapabilities {
            image: self.image.into_v1()?,
            audio: self.audio.into_v1()?,
            embedded_context: self.embedded_context.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::PromptCapabilities {
    type Output = super::PromptCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::PromptCapabilities {
            image: self.image.into_v2()?,
            audio: self.audio.into_v2()?,
            embedded_context: self.embedded_context.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::McpCapabilities {
    type Output = crate::v1::McpCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::McpCapabilities {
            http: self.http.into_v1()?,
            sse: self.sse.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::McpCapabilities {
    type Output = super::McpCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::McpCapabilities {
            http: self.http.into_v2()?,
            sse: self.sse.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AgentMethodNames {
    type Output = crate::v1::AgentMethodNames;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AgentMethodNames {
            initialize: self.initialize.into_v1()?,
            authenticate: self.authenticate.into_v1()?,
            #[cfg(feature = "unstable_llm_providers")]
            providers_list: self.providers_list.into_v1()?,
            #[cfg(feature = "unstable_llm_providers")]
            providers_set: self.providers_set.into_v1()?,
            #[cfg(feature = "unstable_llm_providers")]
            providers_disable: self.providers_disable.into_v1()?,
            session_new: self.session_new.into_v1()?,
            session_load: self.session_load.into_v1()?,
            session_set_mode: self.session_set_mode.into_v1()?,
            session_set_config_option: self.session_set_config_option.into_v1()?,
            session_prompt: self.session_prompt.into_v1()?,
            session_cancel: self.session_cancel.into_v1()?,
            #[cfg(feature = "unstable_session_model")]
            session_set_model: self.session_set_model.into_v1()?,
            session_list: self.session_list.into_v1()?,
            #[cfg(feature = "unstable_session_fork")]
            session_fork: self.session_fork.into_v1()?,
            session_resume: self.session_resume.into_v1()?,
            session_close: self.session_close.into_v1()?,
            #[cfg(feature = "unstable_logout")]
            logout: self.logout.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            nes_start: self.nes_start.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            nes_suggest: self.nes_suggest.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            nes_accept: self.nes_accept.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            nes_reject: self.nes_reject.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            nes_close: self.nes_close.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            document_did_open: self.document_did_open.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            document_did_change: self.document_did_change.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            document_did_close: self.document_did_close.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            document_did_save: self.document_did_save.into_v1()?,
            #[cfg(feature = "unstable_nes")]
            document_did_focus: self.document_did_focus.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AgentMethodNames {
    type Output = super::AgentMethodNames;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AgentMethodNames {
            initialize: self.initialize.into_v2()?,
            authenticate: self.authenticate.into_v2()?,
            #[cfg(feature = "unstable_llm_providers")]
            providers_list: self.providers_list.into_v2()?,
            #[cfg(feature = "unstable_llm_providers")]
            providers_set: self.providers_set.into_v2()?,
            #[cfg(feature = "unstable_llm_providers")]
            providers_disable: self.providers_disable.into_v2()?,
            session_new: self.session_new.into_v2()?,
            session_load: self.session_load.into_v2()?,
            session_set_mode: self.session_set_mode.into_v2()?,
            session_set_config_option: self.session_set_config_option.into_v2()?,
            session_prompt: self.session_prompt.into_v2()?,
            session_cancel: self.session_cancel.into_v2()?,
            #[cfg(feature = "unstable_session_model")]
            session_set_model: self.session_set_model.into_v2()?,
            session_list: self.session_list.into_v2()?,
            #[cfg(feature = "unstable_session_fork")]
            session_fork: self.session_fork.into_v2()?,
            session_resume: self.session_resume.into_v2()?,
            session_close: self.session_close.into_v2()?,
            #[cfg(feature = "unstable_logout")]
            logout: self.logout.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            nes_start: self.nes_start.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            nes_suggest: self.nes_suggest.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            nes_accept: self.nes_accept.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            nes_reject: self.nes_reject.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            nes_close: self.nes_close.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            document_did_open: self.document_did_open.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            document_did_change: self.document_did_change.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            document_did_close: self.document_did_close.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            document_did_save: self.document_did_save.into_v2()?,
            #[cfg(feature = "unstable_nes")]
            document_did_focus: self.document_did_focus.into_v2()?,
        })
    }
}

impl IntoV1 for super::ClientRequest {
    type Output = crate::v1::ClientRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::InitializeRequest(value) => {
                crate::v1::ClientRequest::InitializeRequest(value.into_v1()?)
            }
            Self::AuthenticateRequest(value) => {
                crate::v1::ClientRequest::AuthenticateRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::ListProvidersRequest(value) => {
                crate::v1::ClientRequest::ListProvidersRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::SetProvidersRequest(value) => {
                crate::v1::ClientRequest::SetProvidersRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::DisableProvidersRequest(value) => {
                crate::v1::ClientRequest::DisableProvidersRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_logout")]
            Self::LogoutRequest(value) => crate::v1::ClientRequest::LogoutRequest(value.into_v1()?),
            Self::NewSessionRequest(value) => {
                crate::v1::ClientRequest::NewSessionRequest(value.into_v1()?)
            }
            Self::LoadSessionRequest(value) => {
                crate::v1::ClientRequest::LoadSessionRequest(value.into_v1()?)
            }
            Self::ListSessionsRequest(value) => {
                crate::v1::ClientRequest::ListSessionsRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_session_fork")]
            Self::ForkSessionRequest(value) => {
                crate::v1::ClientRequest::ForkSessionRequest(value.into_v1()?)
            }
            Self::ResumeSessionRequest(value) => {
                crate::v1::ClientRequest::ResumeSessionRequest(value.into_v1()?)
            }
            Self::CloseSessionRequest(value) => {
                crate::v1::ClientRequest::CloseSessionRequest(value.into_v1()?)
            }
            Self::SetSessionModeRequest(value) => {
                crate::v1::ClientRequest::SetSessionModeRequest(value.into_v1()?)
            }
            Self::SetSessionConfigOptionRequest(value) => {
                crate::v1::ClientRequest::SetSessionConfigOptionRequest(value.into_v1()?)
            }
            Self::PromptRequest(value) => crate::v1::ClientRequest::PromptRequest(value.into_v1()?),
            #[cfg(feature = "unstable_session_model")]
            Self::SetSessionModelRequest(value) => {
                crate::v1::ClientRequest::SetSessionModelRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::StartNesRequest(value) => {
                crate::v1::ClientRequest::StartNesRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::SuggestNesRequest(value) => {
                crate::v1::ClientRequest::SuggestNesRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::CloseNesRequest(value) => {
                crate::v1::ClientRequest::CloseNesRequest(value.into_v1()?)
            }
            Self::ExtMethodRequest(value) => {
                crate::v1::ClientRequest::ExtMethodRequest(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::ClientRequest {
    type Output = super::ClientRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::InitializeRequest(value) => {
                super::ClientRequest::InitializeRequest(value.into_v2()?)
            }
            Self::AuthenticateRequest(value) => {
                super::ClientRequest::AuthenticateRequest(value.into_v2()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::ListProvidersRequest(value) => {
                super::ClientRequest::ListProvidersRequest(value.into_v2()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::SetProvidersRequest(value) => {
                super::ClientRequest::SetProvidersRequest(value.into_v2()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::DisableProvidersRequest(value) => {
                super::ClientRequest::DisableProvidersRequest(value.into_v2()?)
            }
            #[cfg(feature = "unstable_logout")]
            Self::LogoutRequest(value) => super::ClientRequest::LogoutRequest(value.into_v2()?),
            Self::NewSessionRequest(value) => {
                super::ClientRequest::NewSessionRequest(value.into_v2()?)
            }
            Self::LoadSessionRequest(value) => {
                super::ClientRequest::LoadSessionRequest(value.into_v2()?)
            }
            Self::ListSessionsRequest(value) => {
                super::ClientRequest::ListSessionsRequest(value.into_v2()?)
            }
            #[cfg(feature = "unstable_session_fork")]
            Self::ForkSessionRequest(value) => {
                super::ClientRequest::ForkSessionRequest(value.into_v2()?)
            }
            Self::ResumeSessionRequest(value) => {
                super::ClientRequest::ResumeSessionRequest(value.into_v2()?)
            }
            Self::CloseSessionRequest(value) => {
                super::ClientRequest::CloseSessionRequest(value.into_v2()?)
            }
            Self::SetSessionModeRequest(value) => {
                super::ClientRequest::SetSessionModeRequest(value.into_v2()?)
            }
            Self::SetSessionConfigOptionRequest(value) => {
                super::ClientRequest::SetSessionConfigOptionRequest(value.into_v2()?)
            }
            Self::PromptRequest(value) => super::ClientRequest::PromptRequest(value.into_v2()?),
            #[cfg(feature = "unstable_session_model")]
            Self::SetSessionModelRequest(value) => {
                super::ClientRequest::SetSessionModelRequest(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::StartNesRequest(value) => super::ClientRequest::StartNesRequest(value.into_v2()?),
            #[cfg(feature = "unstable_nes")]
            Self::SuggestNesRequest(value) => {
                super::ClientRequest::SuggestNesRequest(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::CloseNesRequest(value) => super::ClientRequest::CloseNesRequest(value.into_v2()?),
            Self::ExtMethodRequest(value) => {
                super::ClientRequest::ExtMethodRequest(value.into_v2()?)
            }
        })
    }
}

impl IntoV1 for super::AgentResponse {
    type Output = crate::v1::AgentResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::InitializeResponse(value) => {
                crate::v1::AgentResponse::InitializeResponse(value.into_v1()?)
            }
            Self::AuthenticateResponse(value) => {
                crate::v1::AgentResponse::AuthenticateResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::ListProvidersResponse(value) => {
                crate::v1::AgentResponse::ListProvidersResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::SetProvidersResponse(value) => {
                crate::v1::AgentResponse::SetProvidersResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::DisableProvidersResponse(value) => {
                crate::v1::AgentResponse::DisableProvidersResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_logout")]
            Self::LogoutResponse(value) => {
                crate::v1::AgentResponse::LogoutResponse(value.into_v1()?)
            }
            Self::NewSessionResponse(value) => {
                crate::v1::AgentResponse::NewSessionResponse(value.into_v1()?)
            }
            Self::LoadSessionResponse(value) => {
                crate::v1::AgentResponse::LoadSessionResponse(value.into_v1()?)
            }
            Self::ListSessionsResponse(value) => {
                crate::v1::AgentResponse::ListSessionsResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_session_fork")]
            Self::ForkSessionResponse(value) => {
                crate::v1::AgentResponse::ForkSessionResponse(value.into_v1()?)
            }
            Self::ResumeSessionResponse(value) => {
                crate::v1::AgentResponse::ResumeSessionResponse(value.into_v1()?)
            }
            Self::CloseSessionResponse(value) => {
                crate::v1::AgentResponse::CloseSessionResponse(value.into_v1()?)
            }
            Self::SetSessionModeResponse(value) => {
                crate::v1::AgentResponse::SetSessionModeResponse(value.into_v1()?)
            }
            Self::SetSessionConfigOptionResponse(value) => {
                crate::v1::AgentResponse::SetSessionConfigOptionResponse(value.into_v1()?)
            }
            Self::PromptResponse(value) => {
                crate::v1::AgentResponse::PromptResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_session_model")]
            Self::SetSessionModelResponse(value) => {
                crate::v1::AgentResponse::SetSessionModelResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::StartNesResponse(value) => {
                crate::v1::AgentResponse::StartNesResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::SuggestNesResponse(value) => {
                crate::v1::AgentResponse::SuggestNesResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::CloseNesResponse(value) => {
                crate::v1::AgentResponse::CloseNesResponse(value.into_v1()?)
            }
            Self::ExtMethodResponse(value) => {
                crate::v1::AgentResponse::ExtMethodResponse(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::AgentResponse {
    type Output = super::AgentResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::InitializeResponse(value) => {
                super::AgentResponse::InitializeResponse(value.into_v2()?)
            }
            Self::AuthenticateResponse(value) => {
                super::AgentResponse::AuthenticateResponse(value.into_v2()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::ListProvidersResponse(value) => {
                super::AgentResponse::ListProvidersResponse(value.into_v2()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::SetProvidersResponse(value) => {
                super::AgentResponse::SetProvidersResponse(value.into_v2()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::DisableProvidersResponse(value) => {
                super::AgentResponse::DisableProvidersResponse(value.into_v2()?)
            }
            #[cfg(feature = "unstable_logout")]
            Self::LogoutResponse(value) => super::AgentResponse::LogoutResponse(value.into_v2()?),
            Self::NewSessionResponse(value) => {
                super::AgentResponse::NewSessionResponse(value.into_v2()?)
            }
            Self::LoadSessionResponse(value) => {
                super::AgentResponse::LoadSessionResponse(value.into_v2()?)
            }
            Self::ListSessionsResponse(value) => {
                super::AgentResponse::ListSessionsResponse(value.into_v2()?)
            }
            #[cfg(feature = "unstable_session_fork")]
            Self::ForkSessionResponse(value) => {
                super::AgentResponse::ForkSessionResponse(value.into_v2()?)
            }
            Self::ResumeSessionResponse(value) => {
                super::AgentResponse::ResumeSessionResponse(value.into_v2()?)
            }
            Self::CloseSessionResponse(value) => {
                super::AgentResponse::CloseSessionResponse(value.into_v2()?)
            }
            Self::SetSessionModeResponse(value) => {
                super::AgentResponse::SetSessionModeResponse(value.into_v2()?)
            }
            Self::SetSessionConfigOptionResponse(value) => {
                super::AgentResponse::SetSessionConfigOptionResponse(value.into_v2()?)
            }
            Self::PromptResponse(value) => super::AgentResponse::PromptResponse(value.into_v2()?),
            #[cfg(feature = "unstable_session_model")]
            Self::SetSessionModelResponse(value) => {
                super::AgentResponse::SetSessionModelResponse(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::StartNesResponse(value) => {
                super::AgentResponse::StartNesResponse(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::SuggestNesResponse(value) => {
                super::AgentResponse::SuggestNesResponse(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::CloseNesResponse(value) => {
                super::AgentResponse::CloseNesResponse(value.into_v2()?)
            }
            Self::ExtMethodResponse(value) => {
                super::AgentResponse::ExtMethodResponse(value.into_v2()?)
            }
        })
    }
}

impl IntoV1 for super::ClientNotification {
    type Output = crate::v1::ClientNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::CancelNotification(value) => {
                crate::v1::ClientNotification::CancelNotification(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidOpenDocumentNotification(value) => {
                crate::v1::ClientNotification::DidOpenDocumentNotification(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidChangeDocumentNotification(value) => {
                crate::v1::ClientNotification::DidChangeDocumentNotification(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidCloseDocumentNotification(value) => {
                crate::v1::ClientNotification::DidCloseDocumentNotification(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidSaveDocumentNotification(value) => {
                crate::v1::ClientNotification::DidSaveDocumentNotification(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidFocusDocumentNotification(value) => {
                crate::v1::ClientNotification::DidFocusDocumentNotification(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::AcceptNesNotification(value) => {
                crate::v1::ClientNotification::AcceptNesNotification(value.into_v1()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::RejectNesNotification(value) => {
                crate::v1::ClientNotification::RejectNesNotification(value.into_v1()?)
            }
            Self::ExtNotification(value) => {
                crate::v1::ClientNotification::ExtNotification(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::ClientNotification {
    type Output = super::ClientNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::CancelNotification(value) => {
                super::ClientNotification::CancelNotification(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidOpenDocumentNotification(value) => {
                super::ClientNotification::DidOpenDocumentNotification(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidChangeDocumentNotification(value) => {
                super::ClientNotification::DidChangeDocumentNotification(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidCloseDocumentNotification(value) => {
                super::ClientNotification::DidCloseDocumentNotification(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidSaveDocumentNotification(value) => {
                super::ClientNotification::DidSaveDocumentNotification(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidFocusDocumentNotification(value) => {
                super::ClientNotification::DidFocusDocumentNotification(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::AcceptNesNotification(value) => {
                super::ClientNotification::AcceptNesNotification(value.into_v2()?)
            }
            #[cfg(feature = "unstable_nes")]
            Self::RejectNesNotification(value) => {
                super::ClientNotification::RejectNesNotification(value.into_v2()?)
            }
            Self::ExtNotification(value) => {
                super::ClientNotification::ExtNotification(value.into_v2()?)
            }
        })
    }
}

impl IntoV1 for super::CancelNotification {
    type Output = crate::v1::CancelNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CancelNotification {
            session_id: self.session_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::CancelNotification {
    type Output = super::CancelNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CancelNotification {
            session_id: self.session_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::PositionEncodingKind {
    type Output = crate::v1::PositionEncodingKind;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Utf16 => crate::v1::PositionEncodingKind::Utf16,
            Self::Utf32 => crate::v1::PositionEncodingKind::Utf32,
            Self::Utf8 => crate::v1::PositionEncodingKind::Utf8,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::PositionEncodingKind {
    type Output = super::PositionEncodingKind;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Utf16 => super::PositionEncodingKind::Utf16,
            Self::Utf32 => super::PositionEncodingKind::Utf32,
            Self::Utf8 => super::PositionEncodingKind::Utf8,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::Position {
    type Output = crate::v1::Position;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Position {
            line: self.line.into_v1()?,
            character: self.character.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::Position {
    type Output = super::Position;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Position {
            line: self.line.into_v2()?,
            character: self.character.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::Range {
    type Output = crate::v1::Range;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Range {
            start: self.start.into_v1()?,
            end: self.end.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::Range {
    type Output = super::Range;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Range {
            start: self.start.into_v2()?,
            end: self.end.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesCapabilities {
    type Output = crate::v1::NesCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesCapabilities {
            events: self.events.into_v1()?,
            context: self.context.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesCapabilities {
    type Output = super::NesCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesCapabilities {
            events: self.events.into_v2()?,
            context: self.context.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesEventCapabilities {
    type Output = crate::v1::NesEventCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesEventCapabilities {
            document: self.document.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesEventCapabilities {
    type Output = super::NesEventCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesEventCapabilities {
            document: self.document.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentEventCapabilities {
    type Output = crate::v1::NesDocumentEventCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesDocumentEventCapabilities {
            did_open: self.did_open.into_v1()?,
            did_change: self.did_change.into_v1()?,
            did_close: self.did_close.into_v1()?,
            did_save: self.did_save.into_v1()?,
            did_focus: self.did_focus.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentEventCapabilities {
    type Output = super::NesDocumentEventCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesDocumentEventCapabilities {
            did_open: self.did_open.into_v2()?,
            did_change: self.did_change.into_v2()?,
            did_close: self.did_close.into_v2()?,
            did_save: self.did_save.into_v2()?,
            did_focus: self.did_focus.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentDidOpenCapabilities {
    type Output = crate::v1::NesDocumentDidOpenCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesDocumentDidOpenCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentDidOpenCapabilities {
    type Output = super::NesDocumentDidOpenCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesDocumentDidOpenCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentDidChangeCapabilities {
    type Output = crate::v1::NesDocumentDidChangeCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesDocumentDidChangeCapabilities {
            sync_kind: self.sync_kind.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentDidChangeCapabilities {
    type Output = super::NesDocumentDidChangeCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesDocumentDidChangeCapabilities {
            sync_kind: self.sync_kind.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::TextDocumentSyncKind {
    type Output = crate::v1::TextDocumentSyncKind;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Full => crate::v1::TextDocumentSyncKind::Full,
            Self::Incremental => crate::v1::TextDocumentSyncKind::Incremental,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::TextDocumentSyncKind {
    type Output = super::TextDocumentSyncKind;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Full => super::TextDocumentSyncKind::Full,
            Self::Incremental => super::TextDocumentSyncKind::Incremental,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentDidCloseCapabilities {
    type Output = crate::v1::NesDocumentDidCloseCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesDocumentDidCloseCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentDidCloseCapabilities {
    type Output = super::NesDocumentDidCloseCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesDocumentDidCloseCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentDidSaveCapabilities {
    type Output = crate::v1::NesDocumentDidSaveCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesDocumentDidSaveCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentDidSaveCapabilities {
    type Output = super::NesDocumentDidSaveCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesDocumentDidSaveCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentDidFocusCapabilities {
    type Output = crate::v1::NesDocumentDidFocusCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesDocumentDidFocusCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentDidFocusCapabilities {
    type Output = super::NesDocumentDidFocusCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesDocumentDidFocusCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesContextCapabilities {
    type Output = crate::v1::NesContextCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesContextCapabilities {
            recent_files: self.recent_files.into_v1()?,
            related_snippets: self.related_snippets.into_v1()?,
            edit_history: self.edit_history.into_v1()?,
            user_actions: self.user_actions.into_v1()?,
            open_files: self.open_files.into_v1()?,
            diagnostics: self.diagnostics.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesContextCapabilities {
    type Output = super::NesContextCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesContextCapabilities {
            recent_files: self.recent_files.into_v2()?,
            related_snippets: self.related_snippets.into_v2()?,
            edit_history: self.edit_history.into_v2()?,
            user_actions: self.user_actions.into_v2()?,
            open_files: self.open_files.into_v2()?,
            diagnostics: self.diagnostics.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRecentFilesCapabilities {
    type Output = crate::v1::NesRecentFilesCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesRecentFilesCapabilities {
            max_count: self.max_count.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRecentFilesCapabilities {
    type Output = super::NesRecentFilesCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesRecentFilesCapabilities {
            max_count: self.max_count.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRelatedSnippetsCapabilities {
    type Output = crate::v1::NesRelatedSnippetsCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesRelatedSnippetsCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRelatedSnippetsCapabilities {
    type Output = super::NesRelatedSnippetsCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesRelatedSnippetsCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesEditHistoryCapabilities {
    type Output = crate::v1::NesEditHistoryCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesEditHistoryCapabilities {
            max_count: self.max_count.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesEditHistoryCapabilities {
    type Output = super::NesEditHistoryCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesEditHistoryCapabilities {
            max_count: self.max_count.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesUserActionsCapabilities {
    type Output = crate::v1::NesUserActionsCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesUserActionsCapabilities {
            max_count: self.max_count.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesUserActionsCapabilities {
    type Output = super::NesUserActionsCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesUserActionsCapabilities {
            max_count: self.max_count.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesOpenFilesCapabilities {
    type Output = crate::v1::NesOpenFilesCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesOpenFilesCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesOpenFilesCapabilities {
    type Output = super::NesOpenFilesCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesOpenFilesCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDiagnosticsCapabilities {
    type Output = crate::v1::NesDiagnosticsCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesDiagnosticsCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDiagnosticsCapabilities {
    type Output = super::NesDiagnosticsCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesDiagnosticsCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::ClientNesCapabilities {
    type Output = crate::v1::ClientNesCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ClientNesCapabilities {
            jump: self.jump.into_v1()?,
            rename: self.rename.into_v1()?,
            search_and_replace: self.search_and_replace.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::ClientNesCapabilities {
    type Output = super::ClientNesCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ClientNesCapabilities {
            jump: self.jump.into_v2()?,
            rename: self.rename.into_v2()?,
            search_and_replace: self.search_and_replace.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesJumpCapabilities {
    type Output = crate::v1::NesJumpCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesJumpCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesJumpCapabilities {
    type Output = super::NesJumpCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesJumpCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRenameCapabilities {
    type Output = crate::v1::NesRenameCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesRenameCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRenameCapabilities {
    type Output = super::NesRenameCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesRenameCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesSearchAndReplaceCapabilities {
    type Output = crate::v1::NesSearchAndReplaceCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesSearchAndReplaceCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesSearchAndReplaceCapabilities {
    type Output = super::NesSearchAndReplaceCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesSearchAndReplaceCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::DidOpenDocumentNotification {
    type Output = crate::v1::DidOpenDocumentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::DidOpenDocumentNotification {
            session_id: self.session_id.into_v1()?,
            uri: self.uri.into_v1()?,
            language_id: self.language_id.into_v1()?,
            version: self.version.into_v1()?,
            text: self.text.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::DidOpenDocumentNotification {
    type Output = super::DidOpenDocumentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::DidOpenDocumentNotification {
            session_id: self.session_id.into_v2()?,
            uri: self.uri.into_v2()?,
            language_id: self.language_id.into_v2()?,
            version: self.version.into_v2()?,
            text: self.text.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::DidChangeDocumentNotification {
    type Output = crate::v1::DidChangeDocumentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::DidChangeDocumentNotification {
            session_id: self.session_id.into_v1()?,
            uri: self.uri.into_v1()?,
            version: self.version.into_v1()?,
            content_changes: self.content_changes.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::DidChangeDocumentNotification {
    type Output = super::DidChangeDocumentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::DidChangeDocumentNotification {
            session_id: self.session_id.into_v2()?,
            uri: self.uri.into_v2()?,
            version: self.version.into_v2()?,
            content_changes: self.content_changes.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::TextDocumentContentChangeEvent {
    type Output = crate::v1::TextDocumentContentChangeEvent;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::TextDocumentContentChangeEvent {
            range: self.range.into_v1()?,
            text: self.text.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::TextDocumentContentChangeEvent {
    type Output = super::TextDocumentContentChangeEvent;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::TextDocumentContentChangeEvent {
            range: self.range.into_v2()?,
            text: self.text.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::DidCloseDocumentNotification {
    type Output = crate::v1::DidCloseDocumentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::DidCloseDocumentNotification {
            session_id: self.session_id.into_v1()?,
            uri: self.uri.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::DidCloseDocumentNotification {
    type Output = super::DidCloseDocumentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::DidCloseDocumentNotification {
            session_id: self.session_id.into_v2()?,
            uri: self.uri.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::DidSaveDocumentNotification {
    type Output = crate::v1::DidSaveDocumentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::DidSaveDocumentNotification {
            session_id: self.session_id.into_v1()?,
            uri: self.uri.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::DidSaveDocumentNotification {
    type Output = super::DidSaveDocumentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::DidSaveDocumentNotification {
            session_id: self.session_id.into_v2()?,
            uri: self.uri.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::DidFocusDocumentNotification {
    type Output = crate::v1::DidFocusDocumentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::DidFocusDocumentNotification {
            session_id: self.session_id.into_v1()?,
            uri: self.uri.into_v1()?,
            version: self.version.into_v1()?,
            position: self.position.into_v1()?,
            visible_range: self.visible_range.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::DidFocusDocumentNotification {
    type Output = super::DidFocusDocumentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::DidFocusDocumentNotification {
            session_id: self.session_id.into_v2()?,
            uri: self.uri.into_v2()?,
            version: self.version.into_v2()?,
            position: self.position.into_v2()?,
            visible_range: self.visible_range.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::StartNesRequest {
    type Output = crate::v1::StartNesRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::StartNesRequest {
            workspace_uri: self.workspace_uri.into_v1()?,
            workspace_folders: self.workspace_folders.into_v1()?,
            repository: self.repository.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::StartNesRequest {
    type Output = super::StartNesRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::StartNesRequest {
            workspace_uri: self.workspace_uri.into_v2()?,
            workspace_folders: self.workspace_folders.into_v2()?,
            repository: self.repository.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::WorkspaceFolder {
    type Output = crate::v1::WorkspaceFolder;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::WorkspaceFolder {
            uri: self.uri.into_v1()?,
            name: self.name.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::WorkspaceFolder {
    type Output = super::WorkspaceFolder;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::WorkspaceFolder {
            uri: self.uri.into_v2()?,
            name: self.name.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRepository {
    type Output = crate::v1::NesRepository;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesRepository {
            name: self.name.into_v1()?,
            owner: self.owner.into_v1()?,
            remote_url: self.remote_url.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRepository {
    type Output = super::NesRepository;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesRepository {
            name: self.name.into_v2()?,
            owner: self.owner.into_v2()?,
            remote_url: self.remote_url.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::StartNesResponse {
    type Output = crate::v1::StartNesResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::StartNesResponse {
            session_id: self.session_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::StartNesResponse {
    type Output = super::StartNesResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::StartNesResponse {
            session_id: self.session_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::CloseNesRequest {
    type Output = crate::v1::CloseNesRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CloseNesRequest {
            session_id: self.session_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::CloseNesRequest {
    type Output = super::CloseNesRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CloseNesRequest {
            session_id: self.session_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::CloseNesResponse {
    type Output = crate::v1::CloseNesResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CloseNesResponse {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::CloseNesResponse {
    type Output = super::CloseNesResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CloseNesResponse {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesTriggerKind {
    type Output = crate::v1::NesTriggerKind;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Automatic => crate::v1::NesTriggerKind::Automatic,
            Self::Diagnostic => crate::v1::NesTriggerKind::Diagnostic,
            Self::Manual => crate::v1::NesTriggerKind::Manual,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesTriggerKind {
    type Output = super::NesTriggerKind;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Automatic => super::NesTriggerKind::Automatic,
            Self::Diagnostic => super::NesTriggerKind::Diagnostic,
            Self::Manual => super::NesTriggerKind::Manual,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::SuggestNesRequest {
    type Output = crate::v1::SuggestNesRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SuggestNesRequest {
            session_id: self.session_id.into_v1()?,
            uri: self.uri.into_v1()?,
            version: self.version.into_v1()?,
            position: self.position.into_v1()?,
            selection: self.selection.into_v1()?,
            trigger_kind: self.trigger_kind.into_v1()?,
            context: self.context.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::SuggestNesRequest {
    type Output = super::SuggestNesRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SuggestNesRequest {
            session_id: self.session_id.into_v2()?,
            uri: self.uri.into_v2()?,
            version: self.version.into_v2()?,
            position: self.position.into_v2()?,
            selection: self.selection.into_v2()?,
            trigger_kind: self.trigger_kind.into_v2()?,
            context: self.context.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesSuggestContext {
    type Output = crate::v1::NesSuggestContext;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesSuggestContext {
            recent_files: self.recent_files.into_v1()?,
            related_snippets: self.related_snippets.into_v1()?,
            edit_history: self.edit_history.into_v1()?,
            user_actions: self.user_actions.into_v1()?,
            open_files: self.open_files.into_v1()?,
            diagnostics: self.diagnostics.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesSuggestContext {
    type Output = super::NesSuggestContext;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesSuggestContext {
            recent_files: self.recent_files.into_v2()?,
            related_snippets: self.related_snippets.into_v2()?,
            edit_history: self.edit_history.into_v2()?,
            user_actions: self.user_actions.into_v2()?,
            open_files: self.open_files.into_v2()?,
            diagnostics: self.diagnostics.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRecentFile {
    type Output = crate::v1::NesRecentFile;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesRecentFile {
            uri: self.uri.into_v1()?,
            language_id: self.language_id.into_v1()?,
            text: self.text.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRecentFile {
    type Output = super::NesRecentFile;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesRecentFile {
            uri: self.uri.into_v2()?,
            language_id: self.language_id.into_v2()?,
            text: self.text.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRelatedSnippet {
    type Output = crate::v1::NesRelatedSnippet;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesRelatedSnippet {
            uri: self.uri.into_v1()?,
            excerpts: self.excerpts.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRelatedSnippet {
    type Output = super::NesRelatedSnippet;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesRelatedSnippet {
            uri: self.uri.into_v2()?,
            excerpts: self.excerpts.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesExcerpt {
    type Output = crate::v1::NesExcerpt;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesExcerpt {
            start_line: self.start_line.into_v1()?,
            end_line: self.end_line.into_v1()?,
            text: self.text.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesExcerpt {
    type Output = super::NesExcerpt;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesExcerpt {
            start_line: self.start_line.into_v2()?,
            end_line: self.end_line.into_v2()?,
            text: self.text.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesEditHistoryEntry {
    type Output = crate::v1::NesEditHistoryEntry;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesEditHistoryEntry {
            uri: self.uri.into_v1()?,
            diff: self.diff.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesEditHistoryEntry {
    type Output = super::NesEditHistoryEntry;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesEditHistoryEntry {
            uri: self.uri.into_v2()?,
            diff: self.diff.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesUserAction {
    type Output = crate::v1::NesUserAction;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesUserAction {
            action: self.action.into_v1()?,
            uri: self.uri.into_v1()?,
            position: self.position.into_v1()?,
            timestamp_ms: self.timestamp_ms.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesUserAction {
    type Output = super::NesUserAction;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesUserAction {
            action: self.action.into_v2()?,
            uri: self.uri.into_v2()?,
            position: self.position.into_v2()?,
            timestamp_ms: self.timestamp_ms.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesOpenFile {
    type Output = crate::v1::NesOpenFile;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesOpenFile {
            uri: self.uri.into_v1()?,
            language_id: self.language_id.into_v1()?,
            visible_range: self.visible_range.into_v1()?,
            last_focused_ms: self.last_focused_ms.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesOpenFile {
    type Output = super::NesOpenFile;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesOpenFile {
            uri: self.uri.into_v2()?,
            language_id: self.language_id.into_v2()?,
            visible_range: self.visible_range.into_v2()?,
            last_focused_ms: self.last_focused_ms.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDiagnostic {
    type Output = crate::v1::NesDiagnostic;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesDiagnostic {
            uri: self.uri.into_v1()?,
            range: self.range.into_v1()?,
            severity: self.severity.into_v1()?,
            message: self.message.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDiagnostic {
    type Output = super::NesDiagnostic;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesDiagnostic {
            uri: self.uri.into_v2()?,
            range: self.range.into_v2()?,
            severity: self.severity.into_v2()?,
            message: self.message.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDiagnosticSeverity {
    type Output = crate::v1::NesDiagnosticSeverity;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Error => crate::v1::NesDiagnosticSeverity::Error,
            Self::Warning => crate::v1::NesDiagnosticSeverity::Warning,
            Self::Information => crate::v1::NesDiagnosticSeverity::Information,
            Self::Hint => crate::v1::NesDiagnosticSeverity::Hint,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDiagnosticSeverity {
    type Output = super::NesDiagnosticSeverity;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Error => super::NesDiagnosticSeverity::Error,
            Self::Warning => super::NesDiagnosticSeverity::Warning,
            Self::Information => super::NesDiagnosticSeverity::Information,
            Self::Hint => super::NesDiagnosticSeverity::Hint,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::SuggestNesResponse {
    type Output = crate::v1::SuggestNesResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::SuggestNesResponse {
            suggestions: self.suggestions.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::SuggestNesResponse {
    type Output = super::SuggestNesResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::SuggestNesResponse {
            suggestions: self.suggestions.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesSuggestion {
    type Output = crate::v1::NesSuggestion;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Edit(value) => crate::v1::NesSuggestion::Edit(value.into_v1()?),
            Self::Jump(value) => crate::v1::NesSuggestion::Jump(value.into_v1()?),
            Self::Rename(value) => crate::v1::NesSuggestion::Rename(value.into_v1()?),
            Self::SearchAndReplace(value) => {
                crate::v1::NesSuggestion::SearchAndReplace(value.into_v1()?)
            }
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesSuggestion {
    type Output = super::NesSuggestion;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Edit(value) => super::NesSuggestion::Edit(value.into_v2()?),
            Self::Jump(value) => super::NesSuggestion::Jump(value.into_v2()?),
            Self::Rename(value) => super::NesSuggestion::Rename(value.into_v2()?),
            Self::SearchAndReplace(value) => {
                super::NesSuggestion::SearchAndReplace(value.into_v2()?)
            }
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesEditSuggestion {
    type Output = crate::v1::NesEditSuggestion;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesEditSuggestion {
            id: self.id.into_v1()?,
            uri: self.uri.into_v1()?,
            edits: self.edits.into_v1()?,
            cursor_position: self.cursor_position.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesEditSuggestion {
    type Output = super::NesEditSuggestion;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesEditSuggestion {
            id: self.id.into_v2()?,
            uri: self.uri.into_v2()?,
            edits: self.edits.into_v2()?,
            cursor_position: self.cursor_position.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesTextEdit {
    type Output = crate::v1::NesTextEdit;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesTextEdit {
            range: self.range.into_v1()?,
            new_text: self.new_text.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesTextEdit {
    type Output = super::NesTextEdit;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesTextEdit {
            range: self.range.into_v2()?,
            new_text: self.new_text.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesJumpSuggestion {
    type Output = crate::v1::NesJumpSuggestion;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesJumpSuggestion {
            id: self.id.into_v1()?,
            uri: self.uri.into_v1()?,
            position: self.position.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesJumpSuggestion {
    type Output = super::NesJumpSuggestion;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesJumpSuggestion {
            id: self.id.into_v2()?,
            uri: self.uri.into_v2()?,
            position: self.position.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRenameSuggestion {
    type Output = crate::v1::NesRenameSuggestion;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesRenameSuggestion {
            id: self.id.into_v1()?,
            uri: self.uri.into_v1()?,
            position: self.position.into_v1()?,
            new_name: self.new_name.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRenameSuggestion {
    type Output = super::NesRenameSuggestion;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesRenameSuggestion {
            id: self.id.into_v2()?,
            uri: self.uri.into_v2()?,
            position: self.position.into_v2()?,
            new_name: self.new_name.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesSearchAndReplaceSuggestion {
    type Output = crate::v1::NesSearchAndReplaceSuggestion;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NesSearchAndReplaceSuggestion {
            id: self.id.into_v1()?,
            uri: self.uri.into_v1()?,
            search: self.search.into_v1()?,
            replace: self.replace.into_v1()?,
            is_regex: self.is_regex.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesSearchAndReplaceSuggestion {
    type Output = super::NesSearchAndReplaceSuggestion;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NesSearchAndReplaceSuggestion {
            id: self.id.into_v2()?,
            uri: self.uri.into_v2()?,
            search: self.search.into_v2()?,
            replace: self.replace.into_v2()?,
            is_regex: self.is_regex.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::AcceptNesNotification {
    type Output = crate::v1::AcceptNesNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AcceptNesNotification {
            session_id: self.session_id.into_v1()?,
            id: self.id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::AcceptNesNotification {
    type Output = super::AcceptNesNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AcceptNesNotification {
            session_id: self.session_id.into_v2()?,
            id: self.id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::RejectNesNotification {
    type Output = crate::v1::RejectNesNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::RejectNesNotification {
            session_id: self.session_id.into_v1()?,
            id: self.id.into_v1()?,
            reason: self.reason.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::RejectNesNotification {
    type Output = super::RejectNesNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::RejectNesNotification {
            session_id: self.session_id.into_v2()?,
            id: self.id.into_v2()?,
            reason: self.reason.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRejectReason {
    type Output = crate::v1::NesRejectReason;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Rejected => crate::v1::NesRejectReason::Rejected,
            Self::Ignored => crate::v1::NesRejectReason::Ignored,
            Self::Replaced => crate::v1::NesRejectReason::Replaced,
            Self::Cancelled => crate::v1::NesRejectReason::Cancelled,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRejectReason {
    type Output = super::NesRejectReason;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Rejected => super::NesRejectReason::Rejected,
            Self::Ignored => super::NesRejectReason::Ignored,
            Self::Replaced => super::NesRejectReason::Replaced,
            Self::Cancelled => super::NesRejectReason::Cancelled,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationId {
    type Output = crate::v1::ElicitationId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ElicitationId(self.0.into_v1()?))
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationId {
    type Output = super::ElicitationId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ElicitationId(self.0.into_v2()?))
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::StringFormat {
    type Output = crate::v1::StringFormat;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Email => crate::v1::StringFormat::Email,
            Self::Uri => crate::v1::StringFormat::Uri,
            Self::Date => crate::v1::StringFormat::Date,
            Self::DateTime => crate::v1::StringFormat::DateTime,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::StringFormat {
    type Output = super::StringFormat;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Email => super::StringFormat::Email,
            Self::Uri => super::StringFormat::Uri,
            Self::Date => super::StringFormat::Date,
            Self::DateTime => super::StringFormat::DateTime,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationSchemaType {
    type Output = crate::v1::ElicitationSchemaType;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Object => crate::v1::ElicitationSchemaType::Object,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationSchemaType {
    type Output = super::ElicitationSchemaType;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Object => super::ElicitationSchemaType::Object,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::EnumOption {
    type Output = crate::v1::EnumOption;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::EnumOption {
            value: self.value.into_v1()?,
            title: self.title.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::EnumOption {
    type Output = super::EnumOption;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::EnumOption {
            value: self.value.into_v2()?,
            title: self.title.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::StringPropertySchema {
    type Output = crate::v1::StringPropertySchema;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::StringPropertySchema {
            title: self.title.into_v1()?,
            description: self.description.into_v1()?,
            min_length: self.min_length.into_v1()?,
            max_length: self.max_length.into_v1()?,
            pattern: self.pattern.into_v1()?,
            format: self.format.into_v1()?,
            default: self.default.into_v1()?,
            enum_values: self.enum_values.into_v1()?,
            one_of: self.one_of.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::StringPropertySchema {
    type Output = super::StringPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::StringPropertySchema {
            title: self.title.into_v2()?,
            description: self.description.into_v2()?,
            min_length: self.min_length.into_v2()?,
            max_length: self.max_length.into_v2()?,
            pattern: self.pattern.into_v2()?,
            format: self.format.into_v2()?,
            default: self.default.into_v2()?,
            enum_values: self.enum_values.into_v2()?,
            one_of: self.one_of.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::NumberPropertySchema {
    type Output = crate::v1::NumberPropertySchema;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::NumberPropertySchema {
            title: self.title.into_v1()?,
            description: self.description.into_v1()?,
            minimum: self.minimum.into_v1()?,
            maximum: self.maximum.into_v1()?,
            default: self.default.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::NumberPropertySchema {
    type Output = super::NumberPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::NumberPropertySchema {
            title: self.title.into_v2()?,
            description: self.description.into_v2()?,
            minimum: self.minimum.into_v2()?,
            maximum: self.maximum.into_v2()?,
            default: self.default.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::IntegerPropertySchema {
    type Output = crate::v1::IntegerPropertySchema;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::IntegerPropertySchema {
            title: self.title.into_v1()?,
            description: self.description.into_v1()?,
            minimum: self.minimum.into_v1()?,
            maximum: self.maximum.into_v1()?,
            default: self.default.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::IntegerPropertySchema {
    type Output = super::IntegerPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::IntegerPropertySchema {
            title: self.title.into_v2()?,
            description: self.description.into_v2()?,
            minimum: self.minimum.into_v2()?,
            maximum: self.maximum.into_v2()?,
            default: self.default.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::BooleanPropertySchema {
    type Output = crate::v1::BooleanPropertySchema;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::BooleanPropertySchema {
            title: self.title.into_v1()?,
            description: self.description.into_v1()?,
            default: self.default.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::BooleanPropertySchema {
    type Output = super::BooleanPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::BooleanPropertySchema {
            title: self.title.into_v2()?,
            description: self.description.into_v2()?,
            default: self.default.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationStringType {
    type Output = crate::v1::ElicitationStringType;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::String => crate::v1::ElicitationStringType::String,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationStringType {
    type Output = super::ElicitationStringType;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::String => super::ElicitationStringType::String,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::UntitledMultiSelectItems {
    type Output = crate::v1::UntitledMultiSelectItems;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::UntitledMultiSelectItems {
            type_: self.type_.into_v1()?,
            values: self.values.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::UntitledMultiSelectItems {
    type Output = super::UntitledMultiSelectItems;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::UntitledMultiSelectItems {
            type_: self.type_.into_v2()?,
            values: self.values.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::TitledMultiSelectItems {
    type Output = crate::v1::TitledMultiSelectItems;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::TitledMultiSelectItems {
            options: self.options.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::TitledMultiSelectItems {
    type Output = super::TitledMultiSelectItems;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::TitledMultiSelectItems {
            options: self.options.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::MultiSelectItems {
    type Output = crate::v1::MultiSelectItems;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Untitled(value) => crate::v1::MultiSelectItems::Untitled(value.into_v1()?),
            Self::Titled(value) => crate::v1::MultiSelectItems::Titled(value.into_v1()?),
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::MultiSelectItems {
    type Output = super::MultiSelectItems;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Untitled(value) => super::MultiSelectItems::Untitled(value.into_v2()?),
            Self::Titled(value) => super::MultiSelectItems::Titled(value.into_v2()?),
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::MultiSelectPropertySchema {
    type Output = crate::v1::MultiSelectPropertySchema;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::MultiSelectPropertySchema {
            title: self.title.into_v1()?,
            description: self.description.into_v1()?,
            min_items: self.min_items.into_v1()?,
            max_items: self.max_items.into_v1()?,
            items: self.items.into_v1()?,
            default: self.default.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::MultiSelectPropertySchema {
    type Output = super::MultiSelectPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::MultiSelectPropertySchema {
            title: self.title.into_v2()?,
            description: self.description.into_v2()?,
            min_items: self.min_items.into_v2()?,
            max_items: self.max_items.into_v2()?,
            items: self.items.into_v2()?,
            default: self.default.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationPropertySchema {
    type Output = crate::v1::ElicitationPropertySchema;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::String(value) => crate::v1::ElicitationPropertySchema::String(value.into_v1()?),
            Self::Number(value) => crate::v1::ElicitationPropertySchema::Number(value.into_v1()?),
            Self::Integer(value) => crate::v1::ElicitationPropertySchema::Integer(value.into_v1()?),
            Self::Boolean(value) => crate::v1::ElicitationPropertySchema::Boolean(value.into_v1()?),
            Self::Array(value) => crate::v1::ElicitationPropertySchema::Array(value.into_v1()?),
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationPropertySchema {
    type Output = super::ElicitationPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::String(value) => super::ElicitationPropertySchema::String(value.into_v2()?),
            Self::Number(value) => super::ElicitationPropertySchema::Number(value.into_v2()?),
            Self::Integer(value) => super::ElicitationPropertySchema::Integer(value.into_v2()?),
            Self::Boolean(value) => super::ElicitationPropertySchema::Boolean(value.into_v2()?),
            Self::Array(value) => super::ElicitationPropertySchema::Array(value.into_v2()?),
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationSchema {
    type Output = crate::v1::ElicitationSchema;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ElicitationSchema {
            type_: self.type_.into_v1()?,
            title: self.title.into_v1()?,
            properties: self.properties.into_v1()?,
            required: self.required.into_v1()?,
            description: self.description.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationSchema {
    type Output = super::ElicitationSchema;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ElicitationSchema {
            type_: self.type_.into_v2()?,
            title: self.title.into_v2()?,
            properties: self.properties.into_v2()?,
            required: self.required.into_v2()?,
            description: self.description.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationCapabilities {
    type Output = crate::v1::ElicitationCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ElicitationCapabilities {
            form: self.form.into_v1()?,
            url: self.url.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationCapabilities {
    type Output = super::ElicitationCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ElicitationCapabilities {
            form: self.form.into_v2()?,
            url: self.url.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationFormCapabilities {
    type Output = crate::v1::ElicitationFormCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ElicitationFormCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationFormCapabilities {
    type Output = super::ElicitationFormCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ElicitationFormCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationUrlCapabilities {
    type Output = crate::v1::ElicitationUrlCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ElicitationUrlCapabilities {
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationUrlCapabilities {
    type Output = super::ElicitationUrlCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ElicitationUrlCapabilities {
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationScope {
    type Output = crate::v1::ElicitationScope;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Session(value) => crate::v1::ElicitationScope::Session(value.into_v1()?),
            Self::Request(value) => crate::v1::ElicitationScope::Request(value.into_v1()?),
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationScope {
    type Output = super::ElicitationScope;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Session(value) => super::ElicitationScope::Session(value.into_v2()?),
            Self::Request(value) => super::ElicitationScope::Request(value.into_v2()?),
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationSessionScope {
    type Output = crate::v1::ElicitationSessionScope;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ElicitationSessionScope {
            session_id: self.session_id.into_v1()?,
            tool_call_id: self.tool_call_id.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationSessionScope {
    type Output = super::ElicitationSessionScope;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ElicitationSessionScope {
            session_id: self.session_id.into_v2()?,
            tool_call_id: self.tool_call_id.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationRequestScope {
    type Output = crate::v1::ElicitationRequestScope;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ElicitationRequestScope {
            request_id: self.request_id.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationRequestScope {
    type Output = super::ElicitationRequestScope;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ElicitationRequestScope {
            request_id: self.request_id.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::CreateElicitationRequest {
    type Output = crate::v1::CreateElicitationRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CreateElicitationRequest {
            mode: self.mode.into_v1()?,
            message: self.message.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::CreateElicitationRequest {
    type Output = super::CreateElicitationRequest;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CreateElicitationRequest {
            mode: self.mode.into_v2()?,
            message: self.message.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationMode {
    type Output = crate::v1::ElicitationMode;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Form(value) => crate::v1::ElicitationMode::Form(value.into_v1()?),
            Self::Url(value) => crate::v1::ElicitationMode::Url(value.into_v1()?),
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationMode {
    type Output = super::ElicitationMode;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Form(value) => super::ElicitationMode::Form(value.into_v2()?),
            Self::Url(value) => super::ElicitationMode::Url(value.into_v2()?),
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationFormMode {
    type Output = crate::v1::ElicitationFormMode;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ElicitationFormMode {
            scope: self.scope.into_v1()?,
            requested_schema: self.requested_schema.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationFormMode {
    type Output = super::ElicitationFormMode;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ElicitationFormMode {
            scope: self.scope.into_v2()?,
            requested_schema: self.requested_schema.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationUrlMode {
    type Output = crate::v1::ElicitationUrlMode;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ElicitationUrlMode {
            scope: self.scope.into_v1()?,
            elicitation_id: self.elicitation_id.into_v1()?,
            url: self.url.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationUrlMode {
    type Output = super::ElicitationUrlMode;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ElicitationUrlMode {
            scope: self.scope.into_v2()?,
            elicitation_id: self.elicitation_id.into_v2()?,
            url: self.url.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::CreateElicitationResponse {
    type Output = crate::v1::CreateElicitationResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CreateElicitationResponse {
            action: self.action.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::CreateElicitationResponse {
    type Output = super::CreateElicitationResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CreateElicitationResponse {
            action: self.action.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationAction {
    type Output = crate::v1::ElicitationAction;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Accept(value) => crate::v1::ElicitationAction::Accept(value.into_v1()?),
            Self::Decline => crate::v1::ElicitationAction::Decline,
            Self::Cancel => crate::v1::ElicitationAction::Cancel,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationAction {
    type Output = super::ElicitationAction;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Accept(value) => super::ElicitationAction::Accept(value.into_v2()?),
            Self::Decline => super::ElicitationAction::Decline,
            Self::Cancel => super::ElicitationAction::Cancel,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationAcceptAction {
    type Output = crate::v1::ElicitationAcceptAction;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ElicitationAcceptAction {
            content: self.content.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationAcceptAction {
    type Output = super::ElicitationAcceptAction;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ElicitationAcceptAction {
            content: self.content.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationContentValue {
    type Output = crate::v1::ElicitationContentValue;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::String(value) => crate::v1::ElicitationContentValue::String(value.into_v1()?),
            Self::Integer(value) => crate::v1::ElicitationContentValue::Integer(value.into_v1()?),
            Self::Number(value) => crate::v1::ElicitationContentValue::Number(value.into_v1()?),
            Self::Boolean(value) => crate::v1::ElicitationContentValue::Boolean(value.into_v1()?),
            Self::StringArray(value) => {
                crate::v1::ElicitationContentValue::StringArray(value.into_v1()?)
            }
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationContentValue {
    type Output = super::ElicitationContentValue;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::String(value) => super::ElicitationContentValue::String(value.into_v2()?),
            Self::Integer(value) => super::ElicitationContentValue::Integer(value.into_v2()?),
            Self::Number(value) => super::ElicitationContentValue::Number(value.into_v2()?),
            Self::Boolean(value) => super::ElicitationContentValue::Boolean(value.into_v2()?),
            Self::StringArray(value) => {
                super::ElicitationContentValue::StringArray(value.into_v2()?)
            }
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::CompleteElicitationNotification {
    type Output = crate::v1::CompleteElicitationNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::CompleteElicitationNotification {
            elicitation_id: self.elicitation_id.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::CompleteElicitationNotification {
    type Output = super::CompleteElicitationNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::CompleteElicitationNotification {
            elicitation_id: self.elicitation_id.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::UrlElicitationRequiredData {
    type Output = crate::v1::UrlElicitationRequiredData;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::UrlElicitationRequiredData {
            elicitations: self.elicitations.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::UrlElicitationRequiredData {
    type Output = super::UrlElicitationRequiredData;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::UrlElicitationRequiredData {
            elicitations: self.elicitations.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::UrlElicitationRequiredItem {
    type Output = crate::v1::UrlElicitationRequiredItem;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::UrlElicitationRequiredItem {
            mode: self.mode.into_v1()?,
            elicitation_id: self.elicitation_id.into_v1()?,
            url: self.url.into_v1()?,
            message: self.message.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::UrlElicitationRequiredItem {
    type Output = super::UrlElicitationRequiredItem;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::UrlElicitationRequiredItem {
            mode: self.mode.into_v2()?,
            elicitation_id: self.elicitation_id.into_v2()?,
            url: self.url.into_v2()?,
            message: self.message.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationUrlOnlyMode {
    type Output = crate::v1::ElicitationUrlOnlyMode;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Url => crate::v1::ElicitationUrlOnlyMode::Url,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationUrlOnlyMode {
    type Output = super::ElicitationUrlOnlyMode;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Url => super::ElicitationUrlOnlyMode::Url,
        })
    }
}

impl IntoV1 for super::ContentBlock {
    type Output = crate::v1::ContentBlock;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Text(value) => crate::v1::ContentBlock::Text(value.into_v1()?),
            Self::Image(value) => crate::v1::ContentBlock::Image(value.into_v1()?),
            Self::Audio(value) => crate::v1::ContentBlock::Audio(value.into_v1()?),
            Self::ResourceLink(value) => crate::v1::ContentBlock::ResourceLink(value.into_v1()?),
            Self::Resource(value) => crate::v1::ContentBlock::Resource(value.into_v1()?),
        })
    }
}

impl IntoV2 for crate::v1::ContentBlock {
    type Output = super::ContentBlock;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Text(value) => super::ContentBlock::Text(value.into_v2()?),
            Self::Image(value) => super::ContentBlock::Image(value.into_v2()?),
            Self::Audio(value) => super::ContentBlock::Audio(value.into_v2()?),
            Self::ResourceLink(value) => super::ContentBlock::ResourceLink(value.into_v2()?),
            Self::Resource(value) => super::ContentBlock::Resource(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::TextContent {
    type Output = crate::v1::TextContent;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::TextContent {
            annotations: self.annotations.into_v1()?,
            text: self.text.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::TextContent {
    type Output = super::TextContent;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::TextContent {
            annotations: self.annotations.into_v2()?,
            text: self.text.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ImageContent {
    type Output = crate::v1::ImageContent;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ImageContent {
            annotations: self.annotations.into_v1()?,
            data: self.data.into_v1()?,
            mime_type: self.mime_type.into_v1()?,
            uri: self.uri.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ImageContent {
    type Output = super::ImageContent;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ImageContent {
            annotations: self.annotations.into_v2()?,
            data: self.data.into_v2()?,
            mime_type: self.mime_type.into_v2()?,
            uri: self.uri.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AudioContent {
    type Output = crate::v1::AudioContent;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::AudioContent {
            annotations: self.annotations.into_v1()?,
            data: self.data.into_v1()?,
            mime_type: self.mime_type.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AudioContent {
    type Output = super::AudioContent;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::AudioContent {
            annotations: self.annotations.into_v2()?,
            data: self.data.into_v2()?,
            mime_type: self.mime_type.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::EmbeddedResource {
    type Output = crate::v1::EmbeddedResource;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::EmbeddedResource {
            annotations: self.annotations.into_v1()?,
            resource: self.resource.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::EmbeddedResource {
    type Output = super::EmbeddedResource;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::EmbeddedResource {
            annotations: self.annotations.into_v2()?,
            resource: self.resource.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::EmbeddedResourceResource {
    type Output = crate::v1::EmbeddedResourceResource;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::TextResourceContents(value) => {
                crate::v1::EmbeddedResourceResource::TextResourceContents(value.into_v1()?)
            }
            Self::BlobResourceContents(value) => {
                crate::v1::EmbeddedResourceResource::BlobResourceContents(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::EmbeddedResourceResource {
    type Output = super::EmbeddedResourceResource;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::TextResourceContents(value) => {
                super::EmbeddedResourceResource::TextResourceContents(value.into_v2()?)
            }
            Self::BlobResourceContents(value) => {
                super::EmbeddedResourceResource::BlobResourceContents(value.into_v2()?)
            }
        })
    }
}

impl IntoV1 for super::TextResourceContents {
    type Output = crate::v1::TextResourceContents;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::TextResourceContents {
            mime_type: self.mime_type.into_v1()?,
            text: self.text.into_v1()?,
            uri: self.uri.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::TextResourceContents {
    type Output = super::TextResourceContents;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::TextResourceContents {
            mime_type: self.mime_type.into_v2()?,
            text: self.text.into_v2()?,
            uri: self.uri.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::BlobResourceContents {
    type Output = crate::v1::BlobResourceContents;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::BlobResourceContents {
            blob: self.blob.into_v1()?,
            mime_type: self.mime_type.into_v1()?,
            uri: self.uri.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::BlobResourceContents {
    type Output = super::BlobResourceContents;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::BlobResourceContents {
            blob: self.blob.into_v2()?,
            mime_type: self.mime_type.into_v2()?,
            uri: self.uri.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ResourceLink {
    type Output = crate::v1::ResourceLink;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::ResourceLink {
            annotations: self.annotations.into_v1()?,
            description: self.description.into_v1()?,
            mime_type: self.mime_type.into_v1()?,
            name: self.name.into_v1()?,
            size: self.size.into_v1()?,
            title: self.title.into_v1()?,
            uri: self.uri.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ResourceLink {
    type Output = super::ResourceLink;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::ResourceLink {
            annotations: self.annotations.into_v2()?,
            description: self.description.into_v2()?,
            mime_type: self.mime_type.into_v2()?,
            name: self.name.into_v2()?,
            size: self.size.into_v2()?,
            title: self.title.into_v2()?,
            uri: self.uri.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::Annotations {
    type Output = crate::v1::Annotations;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::Annotations {
            audience: self.audience.into_v1()?,
            last_modified: self.last_modified.into_v1()?,
            priority: self.priority.into_v1()?,
            meta: self.meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Annotations {
    type Output = super::Annotations;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::Annotations {
            audience: self.audience.into_v2()?,
            last_modified: self.last_modified.into_v2()?,
            priority: self.priority.into_v2()?,
            meta: self.meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::Role {
    type Output = crate::v1::Role;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Assistant => crate::v1::Role::Assistant,
            Self::User => crate::v1::Role::User,
        })
    }
}

impl IntoV2 for crate::v1::Role {
    type Output = super::Role;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Assistant => super::Role::Assistant,
            Self::User => super::Role::User,
        })
    }
}

impl IntoV1 for super::RequestId {
    type Output = crate::v1::RequestId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Null => crate::v1::RequestId::Null,
            Self::Number(value) => crate::v1::RequestId::Number(value.into_v1()?),
            Self::Str(value) => crate::v1::RequestId::Str(value.into_v1()?),
        })
    }
}

impl IntoV2 for crate::v1::RequestId {
    type Output = super::RequestId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Null => super::RequestId::Null,
            Self::Number(value) => super::RequestId::Number(value.into_v2()?),
            Self::Str(value) => super::RequestId::Str(value.into_v2()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{v1, v2};

    #[test]
    fn converts_v2_initialize_request_to_v1_without_serde() {
        let request = v2::InitializeRequest::new(ProtocolVersion::V2);

        let converted: v1::InitializeRequest = v2_to_v1(request).unwrap();

        assert_eq!(converted.protocol_version, ProtocolVersion::V2);
    }

    #[test]
    fn converts_v1_initialize_request_to_v2_without_serde() {
        let request = v1::InitializeRequest::new(ProtocolVersion::V1);

        let converted: v2::InitializeRequest = v1_to_v2(request).unwrap();

        assert_eq!(converted.protocol_version, ProtocolVersion::V1);
    }
}
