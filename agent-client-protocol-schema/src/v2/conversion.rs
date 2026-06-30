//! Explicit conversion helpers for experimenting with ACP v2 while SDKs still speak v1.
//!
//! The conversions below intentionally move values field-by-field and
//! variant-by-variant instead of serializing through JSON so v2 shape changes
//! have obvious edit points.

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

fn unknown_v2_enum_variant(type_name: &str, value: &str) -> ProtocolConversionError {
    ProtocolConversionError::new(format!(
        "v2 {type_name} variant `{value}` cannot be represented in v1"
    ))
}

fn removed_v1_enum_variant(type_name: &str, value: &str) -> ProtocolConversionError {
    ProtocolConversionError::new(format!(
        "v1 {type_name} variant `{value}` cannot be represented in v2"
    ))
}

const LEGACY_V1_PLAN_ID: &str = "main";

/// Converts a [`ProtocolConversionError`] into a v1 [`Error`](crate::v1::Error)
/// so callers can use `?` to bubble conversion failures through APIs that
/// already speak the v1 error type.
///
/// The conversion is mapped onto [`Error::internal_error`](crate::v1::Error::internal_error)
/// because a failed cross-version conversion always indicates a protocol
/// mismatch on this side of the wire rather than a client mistake.
impl From<ProtocolConversionError> for crate::v1::Error {
    fn from(error: ProtocolConversionError) -> Self {
        crate::v1::Error::internal_error().data(error.message)
    }
}

/// Mirror of the [v1 `From`](#impl-From%3CProtocolConversionError%3E-for-Error)
/// impl for the v2 [`Error`](crate::v2::Error) type.
impl From<ProtocolConversionError> for crate::v2::Error {
    fn from(error: ProtocolConversionError) -> Self {
        crate::v2::Error::internal_error().data(error.message)
    }
}

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

/// Converts a value from the v2 draft type namespace into one or more v1 values.
///
/// Use this trait for protocol values where a single v2 value may need to fan
/// out into multiple v1 values. For example, a whole v2 message update contains
/// an array of content blocks, while v1 represents those blocks as separate
/// chunk updates.
///
/// This is intentionally not blanket-implemented for every [`IntoV1`] type.
/// Keeping one-to-one and one-to-many conversions separate makes future v2
/// fan-out cases explicit and avoids trait coherence conflicts when an
/// existing one-to-one shape grows a v2-only variant that needs fan-out.
pub trait IntoV1Many {
    /// The corresponding v1 item type.
    type Output;

    /// Converts this value into one or more corresponding v1 items.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolConversionError`] when a value cannot be represented in v1.
    fn into_v1_many(self) -> Result<Vec<Self::Output>>;
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

/// Converts a v2 draft value into one or more corresponding v1 values.
///
/// One-to-many conversions are stateless. When a v2 update's semantics depend
/// on previously delivered updates, this helper can only reject cases that are
/// visible in the value being converted. For example, a content-bearing whole
/// message update can be emitted as v1 chunks, but v1 cannot represent the
/// replacement semantics if the v1 side has already received content for that
/// `messageId`.
///
/// # Errors
///
/// Returns [`ProtocolConversionError`] when a value cannot be represented in v1.
pub fn v2_to_v1_many<T>(value: T) -> Result<Vec<T::Output>>
where
    T: IntoV1Many,
{
    value.into_v1_many()
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
    bool,
    f32,
    f64,
    i16,
    i32,
    i64,
    i8,
    isize,
    String,
    u16,
    u32,
    u64,
    u8,
    usize,
    &'static str,
    Arc<RawValue>,
    Arc<str>,
    PathBuf,
    ProtocolVersion,
    super::RequestId,
    serde_json::Map<String, serde_json::Value>,
    serde_json::Value,
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

fn into_v1_default_on_error<T>(value: T) -> T::Output
where
    T: IntoV1,
    T::Output: Default,
{
    value.into_v1().unwrap_or_default()
}

fn into_v2_default_on_error<T>(value: T) -> T::Output
where
    T: IntoV2,
    T::Output: Default,
{
    value.into_v2().unwrap_or_default()
}

fn into_v1_vec_skip_errors<T>(values: Vec<T>) -> Vec<T::Output>
where
    T: IntoV1,
{
    values
        .into_iter()
        .filter_map(|value| value.into_v1().ok())
        .collect()
}

fn into_v2_vec_skip_errors<T>(values: Vec<T>) -> Vec<T::Output>
where
    T: IntoV2,
{
    values
        .into_iter()
        .filter_map(|value| value.into_v2().ok())
        .collect()
}

fn option_vec_into_v1_skip_errors<T>(value: Option<Vec<T>>) -> Option<Vec<T::Output>>
where
    T: IntoV1,
{
    value.map(into_v1_vec_skip_errors)
}

fn option_vec_into_v2_skip_errors<T>(value: Option<Vec<T>>) -> Option<Vec<T::Output>>
where
    T: IntoV2,
{
    value.map(into_v2_vec_skip_errors)
}

fn option_vec_into_v2_default_skip_errors<T>(value: Option<Vec<T>>) -> Vec<T::Output>
where
    T: IntoV2,
{
    value.map(into_v2_vec_skip_errors).unwrap_or_default()
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

impl<T> IntoV1 for crate::MaybeUndefined<T>
where
    T: IntoV1,
{
    type Output = crate::MaybeUndefined<T::Output>;
    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Undefined => crate::MaybeUndefined::Undefined,
            Self::Null => crate::MaybeUndefined::Null,
            Self::Value(value) => crate::MaybeUndefined::Value(value.into_v1()?),
        })
    }
}

impl<T> IntoV2 for crate::MaybeUndefined<T>
where
    T: IntoV2,
{
    type Output = crate::MaybeUndefined<T::Output>;
    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Undefined => crate::MaybeUndefined::Undefined,
            Self::Null => crate::MaybeUndefined::Null,
            Self::Value(value) => crate::MaybeUndefined::Value(value.into_v2()?),
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

impl IntoV1Many for super::Notification<super::AgentNotification> {
    type Output = crate::v1::Notification<crate::v1::AgentNotification>;

    fn into_v1_many(self) -> Result<Vec<Self::Output>> {
        let Self { method, params } = self;
        let Some(params) = params else {
            return Ok(vec![crate::v1::Notification {
                method,
                params: None,
            }]);
        };

        params
            .into_v1_many()?
            .into_iter()
            .map(|params| {
                Ok(crate::v1::Notification {
                    method: method.clone(),
                    params: Some(params),
                })
            })
            .collect()
    }
}

impl IntoV1Many for super::JsonRpcMessage<super::Notification<super::AgentNotification>> {
    type Output = crate::v1::JsonRpcMessage<crate::v1::Notification<crate::v1::AgentNotification>>;

    fn into_v1_many(self) -> Result<Vec<Self::Output>> {
        self.into_inner()
            .into_v1_many()?
            .into_iter()
            .map(|message| Ok(crate::v1::JsonRpcMessage::wrap(message)))
            .collect()
    }
}

impl IntoV1Many for super::JsonRpcBatch<super::Notification<super::AgentNotification>> {
    type Output = crate::v1::JsonRpcMessage<crate::v1::Notification<crate::v1::AgentNotification>>;

    fn into_v1_many(self) -> Result<Vec<Self::Output>> {
        let messages = self
            .into_vec()
            .into_iter()
            .map(IntoV1Many::into_v1_many)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(messages)
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

impl IntoV1 for super::MessageId {
    type Output = crate::v1::MessageId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::MessageId(self.0.into_v1()?))
    }
}

impl IntoV2 for crate::v1::MessageId {
    type Output = super::MessageId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::MessageId(self.0.into_v2()?))
    }
}

#[cfg(not(feature = "unstable_plan_operations"))]
impl IntoV1 for super::PlanUpdate {
    type Output = crate::v1::Plan;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { plan, meta } = self;
        Ok(match plan {
            super::PlanUpdateContent::Items(items) => {
                let super::PlanItems {
                    id: _,
                    entries,
                    meta: items_meta,
                } = items;
                crate::v1::Plan {
                    entries: into_v1_vec_skip_errors(entries),
                    meta: meta.or(items_meta).into_v1()?,
                }
            }
            super::PlanUpdateContent::Other(value) => {
                return Err(unknown_v2_enum_variant("PlanUpdateContent", &value.type_));
            }
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV1 for super::PlanId {
    type Output = crate::v1::PlanId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::PlanId(self.0.into_v1()?))
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV2 for crate::v1::PlanId {
    type Output = super::PlanId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::PlanId(self.0.into_v2()?))
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV1 for super::PlanUpdate {
    type Output = crate::v1::PlanUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { plan, meta } = self;
        Ok(crate::v1::PlanUpdate {
            plan: plan.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV2 for crate::v1::PlanUpdate {
    type Output = super::PlanUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { plan, meta } = self;
        Ok(super::PlanUpdate {
            plan: plan.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV1 for super::PlanUpdateContent {
    type Output = crate::v1::PlanUpdateContent;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Items(value) => crate::v1::PlanUpdateContent::Items(value.into_v1()?),
            Self::File(value) => crate::v1::PlanUpdateContent::File(value.into_v1()?),
            Self::Markdown(value) => crate::v1::PlanUpdateContent::Markdown(value.into_v1()?),
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant("PlanUpdateContent", &value.type_));
            }
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV2 for crate::v1::PlanUpdateContent {
    type Output = super::PlanUpdateContent;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Items(value) => super::PlanUpdateContent::Items(value.into_v2()?),
            Self::File(value) => super::PlanUpdateContent::File(value.into_v2()?),
            Self::Markdown(value) => super::PlanUpdateContent::Markdown(value.into_v2()?),
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV1 for super::PlanItems {
    type Output = crate::v1::PlanItems;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { id, entries, meta } = self;
        Ok(crate::v1::PlanItems {
            id: id.into_v1()?,
            entries: into_v1_vec_skip_errors(entries),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV2 for crate::v1::PlanItems {
    type Output = super::PlanItems;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { id, entries, meta } = self;
        Ok(super::PlanItems {
            id: id.into_v2()?,
            entries: into_v2_vec_skip_errors(entries),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV1 for super::PlanFile {
    type Output = crate::v1::PlanFile;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { id, uri, meta } = self;
        Ok(crate::v1::PlanFile {
            id: id.into_v1()?,
            uri: uri.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV2 for crate::v1::PlanFile {
    type Output = super::PlanFile;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { id, uri, meta } = self;
        Ok(super::PlanFile {
            id: id.into_v2()?,
            uri: uri.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV1 for super::PlanMarkdown {
    type Output = crate::v1::PlanMarkdown;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { id, content, meta } = self;
        Ok(crate::v1::PlanMarkdown {
            id: id.into_v1()?,
            content: content.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV2 for crate::v1::PlanMarkdown {
    type Output = super::PlanMarkdown;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { id, content, meta } = self;
        Ok(super::PlanMarkdown {
            id: id.into_v2()?,
            content: content.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV1 for super::PlanRemoved {
    type Output = crate::v1::PlanRemoved;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { id, meta } = self;
        Ok(crate::v1::PlanRemoved {
            id: id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_plan_operations")]
impl IntoV2 for crate::v1::PlanRemoved {
    type Output = super::PlanRemoved;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { id, meta } = self;
        Ok(super::PlanRemoved {
            id: id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PlanEntry {
    type Output = crate::v1::PlanEntry;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            content,
            priority,
            status,
            meta,
        } = self;
        Ok(crate::v1::PlanEntry {
            content: content.into_v1()?,
            priority: priority.into_v1()?,
            status: status.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::PlanEntry {
    type Output = super::PlanEntry;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            content,
            priority,
            status,
            meta,
        } = self;
        Ok(super::PlanEntry {
            content: content.into_v2()?,
            priority: priority.into_v2()?,
            status: status.into_v2()?,
            meta: meta.into_v2()?,
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
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant("PlanEntryPriority", &value));
            }
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
            Self::Other(value) => return Err(unknown_v2_enum_variant("PlanEntryStatus", &value)),
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

impl IntoV1 for super::CancelRequestNotification {
    type Output = crate::v1::CancelRequestNotification;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { request_id, meta } = self;
        Ok(crate::v1::CancelRequestNotification {
            request_id: request_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::CancelRequestNotification {
    type Output = super::CancelRequestNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { request_id, meta } = self;
        Ok(super::CancelRequestNotification {
            request_id: request_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

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

impl IntoV1Many for super::UpdateSessionNotification {
    type Output = crate::v1::SessionNotification;

    fn into_v1_many(self) -> Result<Vec<Self::Output>> {
        let Self {
            session_id,
            update,
            meta,
        } = self;
        let session_id = session_id.into_v1()?;
        let meta = meta.into_v1()?;
        update
            .into_v1_many()?
            .into_iter()
            .map(|update| {
                Ok(crate::v1::SessionNotification {
                    session_id: session_id.clone(),
                    update,
                    meta: meta.clone(),
                })
            })
            .collect()
    }
}

impl IntoV2 for crate::v1::SessionNotification {
    type Output = super::UpdateSessionNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            update,
            meta,
        } = self;
        Ok(super::UpdateSessionNotification {
            session_id: session_id.into_v2()?,
            update: update.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1Many for super::SessionUpdate {
    type Output = crate::v1::SessionUpdate;

    fn into_v1_many(self) -> Result<Vec<Self::Output>> {
        Ok(match self {
            Self::UserMessageChunk(value) => {
                vec![crate::v1::SessionUpdate::UserMessageChunk(value.into_v1()?)]
            }
            Self::UserMessage(value) => v2_message_update_into_v1_chunks(
                "user_message",
                value.message_id,
                value.content,
                value.meta,
                crate::v1::SessionUpdate::UserMessageChunk,
            )?,
            Self::AgentMessageChunk(value) => {
                vec![crate::v1::SessionUpdate::AgentMessageChunk(
                    value.into_v1()?,
                )]
            }
            Self::AgentMessage(value) => v2_message_update_into_v1_chunks(
                "agent_message",
                value.message_id,
                value.content,
                value.meta,
                crate::v1::SessionUpdate::AgentMessageChunk,
            )?,
            Self::AgentThoughtChunk(value) => {
                vec![crate::v1::SessionUpdate::AgentThoughtChunk(
                    value.into_v1()?,
                )]
            }
            Self::AgentThought(value) => v2_message_update_into_v1_chunks(
                "agent_thought",
                value.message_id,
                value.content,
                value.meta,
                crate::v1::SessionUpdate::AgentThoughtChunk,
            )?,
            Self::StateUpdate(_) => {
                return Err(ProtocolConversionError::new(
                    "v2 SessionUpdate variant `state_update` cannot be represented in v1 because v1 reports completion in the session/prompt response",
                ));
            }
            Self::ToolCallContentChunk(_) => {
                return Err(ProtocolConversionError::new(
                    "v2 SessionUpdate variant `tool_call_content_chunk` cannot be represented in v1 because v1 tool-call content updates replace content instead of appending",
                ));
            }
            Self::ToolCallUpdate(value) => {
                vec![crate::v1::SessionUpdate::ToolCallUpdate(value.into_v1()?)]
            }
            #[cfg(feature = "unstable_plan_operations")]
            Self::PlanUpdate(value) => vec![crate::v1::SessionUpdate::PlanUpdate(value.into_v1()?)],
            #[cfg(not(feature = "unstable_plan_operations"))]
            Self::PlanUpdate(value) => vec![crate::v1::SessionUpdate::Plan(value.into_v1()?)],
            #[cfg(feature = "unstable_plan_operations")]
            Self::PlanRemoved(value) => {
                vec![crate::v1::SessionUpdate::PlanRemoved(value.into_v1()?)]
            }
            Self::AvailableCommandsUpdate(value) => {
                vec![crate::v1::SessionUpdate::AvailableCommandsUpdate(
                    value.into_v1()?,
                )]
            }
            Self::ConfigOptionUpdate(value) => {
                vec![crate::v1::SessionUpdate::ConfigOptionUpdate(
                    value.into_v1()?,
                )]
            }
            Self::SessionInfoUpdate(value) => {
                vec![crate::v1::SessionUpdate::SessionInfoUpdate(
                    value.into_v1()?,
                )]
            }
            Self::UsageUpdate(value) => {
                vec![crate::v1::SessionUpdate::UsageUpdate(value.into_v1()?)]
            }
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant(
                    "SessionUpdate",
                    &value.session_update,
                ));
            }
        })
    }
}

fn v2_message_update_into_v1_chunks(
    variant: &str,
    message_id: super::MessageId,
    content: crate::MaybeUndefined<Vec<super::ContentBlock>>,
    meta: crate::MaybeUndefined<super::Meta>,
    wrap: impl Fn(crate::v1::ContentChunk) -> crate::v1::SessionUpdate,
) -> Result<Vec<crate::v1::SessionUpdate>> {
    let content = match content {
        crate::MaybeUndefined::Value(content) if !content.is_empty() => content,
        crate::MaybeUndefined::Value(_) => {
            return Err(ProtocolConversionError::new(format!(
                "v2 SessionUpdate variant `{variant}` with empty content cannot be represented in v1 chunks"
            )));
        }
        crate::MaybeUndefined::Null => {
            return Err(ProtocolConversionError::new(format!(
                "v2 SessionUpdate variant `{variant}` with null content cannot be represented in v1 chunks"
            )));
        }
        crate::MaybeUndefined::Undefined => {
            return Err(ProtocolConversionError::new(format!(
                "v2 SessionUpdate variant `{variant}` without content cannot be represented in v1 chunks"
            )));
        }
    };
    let message_id = message_id.into_v1()?;
    let meta = match meta {
        crate::MaybeUndefined::Value(meta) => Some(meta.into_v1()?),
        crate::MaybeUndefined::Null => {
            return Err(ProtocolConversionError::new(format!(
                "v2 SessionUpdate variant `{variant}` with null _meta cannot be represented in v1 chunks"
            )));
        }
        crate::MaybeUndefined::Undefined => None,
    };

    content
        .into_iter()
        .map(|content| {
            Ok(wrap(crate::v1::ContentChunk {
                content: content.into_v1()?,
                message_id: Some(message_id.clone()),
                meta: meta.clone(),
            }))
        })
        .collect()
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
            Self::ToolCall(value) => super::SessionUpdate::ToolCallUpdate(value.into_v2()?),
            Self::ToolCallUpdate(value) => super::SessionUpdate::ToolCallUpdate(value.into_v2()?),
            Self::Plan(value) => {
                let crate::v1::Plan { entries, meta } = value;
                super::SessionUpdate::PlanUpdate(super::PlanUpdate {
                    plan: super::PlanUpdateContent::items(LEGACY_V1_PLAN_ID, entries.into_v2()?),
                    meta: meta.into_v2()?,
                })
            }
            #[cfg(feature = "unstable_plan_operations")]
            Self::PlanUpdate(value) => super::SessionUpdate::PlanUpdate(value.into_v2()?),
            #[cfg(feature = "unstable_plan_operations")]
            Self::PlanRemoved(value) => super::SessionUpdate::PlanRemoved(value.into_v2()?),
            Self::AvailableCommandsUpdate(value) => {
                super::SessionUpdate::AvailableCommandsUpdate(value.into_v2()?)
            }
            Self::CurrentModeUpdate(_) => {
                return Err(removed_v1_enum_variant(
                    "SessionUpdate",
                    "current_mode_update",
                ));
            }
            Self::ConfigOptionUpdate(value) => {
                super::SessionUpdate::ConfigOptionUpdate(value.into_v2()?)
            }
            Self::SessionInfoUpdate(value) => {
                super::SessionUpdate::SessionInfoUpdate(value.into_v2()?)
            }
            Self::UsageUpdate(value) => super::SessionUpdate::UsageUpdate(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::ConfigOptionUpdate {
    type Output = crate::v1::ConfigOptionUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            config_options,
            meta,
        } = self;
        Ok(crate::v1::ConfigOptionUpdate {
            config_options: into_v1_vec_skip_errors(config_options),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ConfigOptionUpdate {
    type Output = super::ConfigOptionUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            config_options,
            meta,
        } = self;
        Ok(super::ConfigOptionUpdate {
            config_options: into_v2_vec_skip_errors(config_options),
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionInfoUpdate {
    type Output = crate::v1::SessionInfoUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            title,
            updated_at,
            meta,
        } = self;
        Ok(crate::v1::SessionInfoUpdate {
            title: title.into_v1()?,
            updated_at: updated_at.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionInfoUpdate {
    type Output = super::SessionInfoUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            title,
            updated_at,
            meta,
        } = self;
        Ok(super::SessionInfoUpdate {
            title: title.into_v2()?,
            updated_at: updated_at.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::UsageUpdate {
    type Output = crate::v1::UsageUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            used,
            size,
            cost,
            meta,
        } = self;
        Ok(crate::v1::UsageUpdate {
            used: used.into_v1()?,
            size: size.into_v1()?,
            cost: into_v1_default_on_error(cost),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::UsageUpdate {
    type Output = super::UsageUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            used,
            size,
            cost,
            meta,
        } = self;
        Ok(super::UsageUpdate {
            used: used.into_v2()?,
            size: size.into_v2()?,
            cost: into_v2_default_on_error(cost),
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::Cost {
    type Output = crate::v1::Cost;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            amount,
            currency,
            meta,
        } = self;
        Ok(crate::v1::Cost {
            amount: amount.into_v1()?,
            currency: currency.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Cost {
    type Output = super::Cost;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            amount,
            currency,
            meta,
        } = self;
        Ok(super::Cost {
            amount: amount.into_v2()?,
            currency: currency.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ContentChunk {
    type Output = crate::v1::ContentChunk;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            content,
            message_id,
            meta,
        } = self;
        Ok(crate::v1::ContentChunk {
            content: content.into_v1()?,
            message_id: Some(message_id.into_v1()?),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ContentChunk {
    type Output = super::ContentChunk;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            content,
            message_id,
            meta,
        } = self;
        Ok(super::ContentChunk {
            content: content.into_v2()?,
            message_id: message_id
                .ok_or_else(|| {
                    ProtocolConversionError::new(
                        "v1 ContentChunk without messageId cannot be represented in v2",
                    )
                })?
                .into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AvailableCommandsUpdate {
    type Output = crate::v1::AvailableCommandsUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            available_commands,
            meta,
        } = self;
        Ok(crate::v1::AvailableCommandsUpdate {
            available_commands: into_v1_vec_skip_errors(available_commands),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AvailableCommandsUpdate {
    type Output = super::AvailableCommandsUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            available_commands,
            meta,
        } = self;
        Ok(super::AvailableCommandsUpdate {
            available_commands: into_v2_vec_skip_errors(available_commands),
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AvailableCommand {
    type Output = crate::v1::AvailableCommand;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            name,
            description,
            input,
            meta,
        } = self;
        Ok(crate::v1::AvailableCommand {
            name: name.into_v1()?,
            description: description.into_v1()?,
            input: into_v1_default_on_error(input),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AvailableCommand {
    type Output = super::AvailableCommand;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            name,
            description,
            input,
            meta,
        } = self;
        Ok(super::AvailableCommand {
            name: name.into_v2()?,
            description: description.into_v2()?,
            input: into_v2_default_on_error(input),
            meta: meta.into_v2()?,
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
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant(
                    "AvailableCommandInput",
                    &value.type_,
                ));
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
        let Self { hint, meta } = self;
        Ok(crate::v1::UnstructuredCommandInput {
            hint: hint.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::UnstructuredCommandInput {
    type Output = super::UnstructuredCommandInput;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { hint, meta } = self;
        Ok(super::UnstructuredCommandInput {
            hint: hint.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::RequestPermissionRequest {
    type Output = crate::v1::RequestPermissionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            tool_call,
            options,
            meta,
        } = self;
        Ok(crate::v1::RequestPermissionRequest {
            session_id: session_id.into_v1()?,
            tool_call: tool_call.into_v1()?,
            options: options.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::RequestPermissionRequest {
    type Output = super::RequestPermissionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            tool_call,
            options,
            meta,
        } = self;
        Ok(super::RequestPermissionRequest {
            session_id: session_id.into_v2()?,
            tool_call: tool_call.into_v2()?,
            options: options.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PermissionOption {
    type Output = crate::v1::PermissionOption;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            option_id,
            name,
            kind,
            meta,
        } = self;
        Ok(crate::v1::PermissionOption {
            option_id: option_id.into_v1()?,
            name: name.into_v1()?,
            kind: kind.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::PermissionOption {
    type Output = super::PermissionOption;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            option_id,
            name,
            kind,
            meta,
        } = self;
        Ok(super::PermissionOption {
            option_id: option_id.into_v2()?,
            name: name.into_v2()?,
            kind: kind.into_v2()?,
            meta: meta.into_v2()?,
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
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant("PermissionOptionKind", &value));
            }
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
        let Self { outcome, meta } = self;
        Ok(crate::v1::RequestPermissionResponse {
            outcome: outcome.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::RequestPermissionResponse {
    type Output = super::RequestPermissionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { outcome, meta } = self;
        Ok(super::RequestPermissionResponse {
            outcome: outcome.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self { option_id, meta } = self;
        Ok(crate::v1::SelectedPermissionOutcome {
            option_id: option_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SelectedPermissionOutcome {
    type Output = super::SelectedPermissionOutcome;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { option_id, meta } = self;
        Ok(super::SelectedPermissionOutcome {
            option_id: option_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV1 for super::ConnectMcpRequest {
    type Output = crate::v1::ConnectMcpRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { acp_id, meta } = self;
        Ok(crate::v1::ConnectMcpRequest {
            acp_id: acp_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV2 for crate::v1::ConnectMcpRequest {
    type Output = super::ConnectMcpRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { acp_id, meta } = self;
        Ok(super::ConnectMcpRequest {
            acp_id: acp_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV1 for super::ConnectMcpResponse {
    type Output = crate::v1::ConnectMcpResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            connection_id,
            meta,
        } = self;
        Ok(crate::v1::ConnectMcpResponse {
            connection_id: connection_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV2 for crate::v1::ConnectMcpResponse {
    type Output = super::ConnectMcpResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            connection_id,
            meta,
        } = self;
        Ok(super::ConnectMcpResponse {
            connection_id: connection_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV1 for super::MessageMcpRequest {
    type Output = crate::v1::MessageMcpRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            connection_id,
            method,
            params,
            meta,
        } = self;
        Ok(crate::v1::MessageMcpRequest {
            connection_id: connection_id.into_v1()?,
            method: method.into_v1()?,
            params: params.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV2 for crate::v1::MessageMcpRequest {
    type Output = super::MessageMcpRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            connection_id,
            method,
            params,
            meta,
        } = self;
        Ok(super::MessageMcpRequest {
            connection_id: connection_id.into_v2()?,
            method: method.into_v2()?,
            params: params.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV1 for super::MessageMcpNotification {
    type Output = crate::v1::MessageMcpNotification;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            connection_id,
            method,
            params,
            meta,
        } = self;
        Ok(crate::v1::MessageMcpNotification {
            connection_id: connection_id.into_v1()?,
            method: method.into_v1()?,
            params: params.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV2 for crate::v1::MessageMcpNotification {
    type Output = super::MessageMcpNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            connection_id,
            method,
            params,
            meta,
        } = self;
        Ok(super::MessageMcpNotification {
            connection_id: connection_id.into_v2()?,
            method: method.into_v2()?,
            params: params.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV1 for super::MessageMcpResponse {
    type Output = crate::v1::MessageMcpResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self(result) = self;
        Ok(crate::v1::MessageMcpResponse::new(result.into_v1()?))
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV2 for crate::v1::MessageMcpResponse {
    type Output = super::MessageMcpResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self(result) = self;
        Ok(super::MessageMcpResponse::new(result.into_v2()?))
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV1 for super::DisconnectMcpRequest {
    type Output = crate::v1::DisconnectMcpRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            connection_id,
            meta,
        } = self;
        Ok(crate::v1::DisconnectMcpRequest {
            connection_id: connection_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV2 for crate::v1::DisconnectMcpRequest {
    type Output = super::DisconnectMcpRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            connection_id,
            meta,
        } = self;
        Ok(super::DisconnectMcpRequest {
            connection_id: connection_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV1 for super::DisconnectMcpResponse {
    type Output = crate::v1::DisconnectMcpResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::DisconnectMcpResponse {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV2 for crate::v1::DisconnectMcpResponse {
    type Output = super::DisconnectMcpResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::DisconnectMcpResponse {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ClientCapabilities {
    type Output = crate::v1::ClientCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            #[cfg(feature = "unstable_auth_methods")]
            auth,
            #[cfg(feature = "unstable_elicitation")]
            elicitation,
            #[cfg(feature = "unstable_nes")]
            nes,
            #[cfg(feature = "unstable_nes")]
            position_encodings,
            meta,
        } = self;
        Ok(crate::v1::ClientCapabilities {
            fs: crate::v1::FileSystemCapabilities::default(),
            terminal: false,
            #[cfg(feature = "unstable_boolean_config")]
            session: Some(
                crate::v1::ClientSessionCapabilities::new().config_options(
                    crate::v1::SessionConfigOptionsCapabilities::new()
                        .boolean(crate::v1::BooleanConfigOptionCapabilities::new()),
                ),
            ),
            #[cfg(feature = "unstable_plan_operations")]
            plan: None,
            #[cfg(feature = "unstable_auth_methods")]
            auth: auth.map(IntoV1::into_v1).transpose()?.unwrap_or_default(),
            #[cfg(feature = "unstable_elicitation")]
            elicitation: into_v1_default_on_error(elicitation),
            #[cfg(feature = "unstable_nes")]
            nes: into_v1_default_on_error(nes),
            #[cfg(feature = "unstable_nes")]
            position_encodings: into_v1_vec_skip_errors(position_encodings),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ClientCapabilities {
    type Output = super::ClientCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            fs: _,
            terminal: _,
            #[cfg(feature = "unstable_boolean_config")]
                session: _,
            #[cfg(feature = "unstable_plan_operations")]
                plan: _,
            #[cfg(feature = "unstable_auth_methods")]
            auth,
            #[cfg(feature = "unstable_elicitation")]
            elicitation,
            #[cfg(feature = "unstable_nes")]
            nes,
            #[cfg(feature = "unstable_nes")]
            position_encodings,
            meta,
        } = self;
        Ok(super::ClientCapabilities {
            #[cfg(feature = "unstable_auth_methods")]
            auth: Some(auth.into_v2()?),
            #[cfg(feature = "unstable_elicitation")]
            elicitation: into_v2_default_on_error(elicitation),
            #[cfg(feature = "unstable_nes")]
            nes: into_v2_default_on_error(nes),
            #[cfg(feature = "unstable_nes")]
            position_encodings: into_v2_vec_skip_errors(position_encodings),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV1 for super::AuthCapabilities {
    type Output = crate::v1::AuthCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { terminal, meta } = self;
        Ok(crate::v1::AuthCapabilities {
            terminal: terminal.is_some(),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV2 for crate::v1::AuthCapabilities {
    type Output = super::AuthCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { terminal, meta } = self;
        Ok(super::AuthCapabilities {
            terminal: terminal.then(super::TerminalAuthCapabilities::new),
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AgentRequest {
    type Output = crate::v1::AgentRequest;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::RequestPermissionRequest(value) => {
                crate::v1::AgentRequest::RequestPermissionRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CreateElicitationRequest(value) => {
                crate::v1::AgentRequest::CreateElicitationRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::ConnectMcpRequest(value) => {
                crate::v1::AgentRequest::ConnectMcpRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpRequest(value) => {
                crate::v1::AgentRequest::MessageMcpRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::DisconnectMcpRequest(value) => {
                crate::v1::AgentRequest::DisconnectMcpRequest(value.into_v1()?)
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
            Self::WriteTextFileRequest(_) => {
                return Err(removed_v1_enum_variant(
                    "AgentRequest",
                    "fs/write_text_file",
                ));
            }
            Self::ReadTextFileRequest(_) => {
                return Err(removed_v1_enum_variant("AgentRequest", "fs/read_text_file"));
            }
            Self::RequestPermissionRequest(value) => {
                super::AgentRequest::RequestPermissionRequest(Box::new(value.into_v2()?))
            }
            Self::CreateTerminalRequest(_) => {
                return Err(removed_v1_enum_variant("AgentRequest", "terminal/create"));
            }
            Self::TerminalOutputRequest(_) => {
                return Err(removed_v1_enum_variant("AgentRequest", "terminal/output"));
            }
            Self::ReleaseTerminalRequest(_) => {
                return Err(removed_v1_enum_variant("AgentRequest", "terminal/release"));
            }
            Self::WaitForTerminalExitRequest(_) => {
                return Err(removed_v1_enum_variant(
                    "AgentRequest",
                    "terminal/wait_for_exit",
                ));
            }
            Self::KillTerminalRequest(_) => {
                return Err(removed_v1_enum_variant("AgentRequest", "terminal/kill"));
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CreateElicitationRequest(value) => {
                super::AgentRequest::CreateElicitationRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::ConnectMcpRequest(value) => {
                super::AgentRequest::ConnectMcpRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpRequest(value) => {
                super::AgentRequest::MessageMcpRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::DisconnectMcpRequest(value) => {
                super::AgentRequest::DisconnectMcpRequest(Box::new(value.into_v2()?))
            }
            Self::ExtMethodRequest(value) => {
                super::AgentRequest::ExtMethodRequest(Box::new(value.into_v2()?))
            }
        })
    }
}

impl IntoV1 for super::ClientResponse {
    type Output = crate::v1::ClientResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::RequestPermissionResponse(value) => {
                crate::v1::ClientResponse::RequestPermissionResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CreateElicitationResponse(value) => {
                crate::v1::ClientResponse::CreateElicitationResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::ConnectMcpResponse(value) => {
                crate::v1::ClientResponse::ConnectMcpResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpResponse(value) => {
                crate::v1::ClientResponse::MessageMcpResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::DisconnectMcpResponse(value) => {
                crate::v1::ClientResponse::DisconnectMcpResponse(value.into_v1()?)
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
            Self::WriteTextFileResponse(_) => {
                return Err(removed_v1_enum_variant(
                    "ClientResponse",
                    "fs/write_text_file",
                ));
            }
            Self::ReadTextFileResponse(_) => {
                return Err(removed_v1_enum_variant(
                    "ClientResponse",
                    "fs/read_text_file",
                ));
            }
            Self::RequestPermissionResponse(value) => {
                super::ClientResponse::RequestPermissionResponse(Box::new(value.into_v2()?))
            }
            Self::CreateTerminalResponse(_) => {
                return Err(removed_v1_enum_variant("ClientResponse", "terminal/create"));
            }
            Self::TerminalOutputResponse(_) => {
                return Err(removed_v1_enum_variant("ClientResponse", "terminal/output"));
            }
            Self::ReleaseTerminalResponse(_) => {
                return Err(removed_v1_enum_variant(
                    "ClientResponse",
                    "terminal/release",
                ));
            }
            Self::WaitForTerminalExitResponse(_) => {
                return Err(removed_v1_enum_variant(
                    "ClientResponse",
                    "terminal/wait_for_exit",
                ));
            }
            Self::KillTerminalResponse(_) => {
                return Err(removed_v1_enum_variant("ClientResponse", "terminal/kill"));
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CreateElicitationResponse(value) => {
                super::ClientResponse::CreateElicitationResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::ConnectMcpResponse(value) => {
                super::ClientResponse::ConnectMcpResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpResponse(value) => {
                super::ClientResponse::MessageMcpResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::DisconnectMcpResponse(value) => {
                super::ClientResponse::DisconnectMcpResponse(Box::new(value.into_v2()?))
            }
            Self::ExtMethodResponse(value) => {
                super::ClientResponse::ExtMethodResponse(Box::new(value.into_v2()?))
            }
        })
    }
}

impl IntoV1Many for super::AgentNotification {
    type Output = crate::v1::AgentNotification;

    fn into_v1_many(self) -> Result<Vec<Self::Output>> {
        Ok(match self {
            Self::UpdateSessionNotification(value) => {
                return value
                    .into_v1_many()?
                    .into_iter()
                    .map(|value| Ok(crate::v1::AgentNotification::SessionNotification(value)))
                    .collect();
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CompleteElicitationNotification(value) => {
                vec![
                    crate::v1::AgentNotification::CompleteElicitationNotification(value.into_v1()?),
                ]
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpNotification(value) => {
                vec![crate::v1::AgentNotification::MessageMcpNotification(
                    value.into_v1()?,
                )]
            }
            Self::ExtNotification(value) => {
                vec![crate::v1::AgentNotification::ExtNotification(
                    value.into_v1()?,
                )]
            }
        })
    }
}

impl IntoV2 for crate::v1::AgentNotification {
    type Output = super::AgentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::SessionNotification(value) => {
                super::AgentNotification::UpdateSessionNotification(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_elicitation")]
            Self::CompleteElicitationNotification(value) => {
                super::AgentNotification::CompleteElicitationNotification(Box::new(
                    value.into_v2()?,
                ))
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpNotification(value) => {
                super::AgentNotification::MessageMcpNotification(Box::new(value.into_v2()?))
            }
            Self::ExtNotification(value) => {
                super::AgentNotification::ExtNotification(Box::new(value.into_v2()?))
            }
        })
    }
}

impl IntoV1 for super::Error {
    type Output = crate::v1::Error;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            code,
            message,
            data,
        } = self;
        Ok(crate::v1::Error {
            code: code.into_v1()?,
            message: message.into_v1()?,
            data: data.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Error {
    type Output = super::Error;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            code,
            message,
            data,
        } = self;
        Ok(super::Error {
            code: code.into_v2()?,
            message: message.into_v2()?,
            data: data.into_v2()?,
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
        let Self { method, params } = self;
        Ok(crate::v1::ExtRequest {
            method: method.into_v1()?,
            params: params.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ExtRequest {
    type Output = super::ExtRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { method, params } = self;
        Ok(super::ExtRequest {
            method: method.into_v2()?,
            params: params.into_v2()?,
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
        let Self { method, params } = self;
        Ok(crate::v1::ExtNotification {
            method: method.into_v1()?,
            params: params.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ExtNotification {
    type Output = super::ExtNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { method, params } = self;
        Ok(super::ExtNotification {
            method: method.into_v2()?,
            params: params.into_v2()?,
        })
    }
}

fn maybe_undefined_value_into_v1_option<T>(value: crate::MaybeUndefined<T>) -> Option<T::Output>
where
    T: IntoV1,
{
    match value {
        crate::MaybeUndefined::Value(value) => value.into_v1().ok(),
        crate::MaybeUndefined::Null | crate::MaybeUndefined::Undefined => None,
    }
}

fn maybe_undefined_vec_into_v1_option<T>(
    value: crate::MaybeUndefined<Vec<T>>,
) -> Option<Vec<T::Output>>
where
    T: IntoV1,
{
    match value {
        crate::MaybeUndefined::Value(value) => Some(into_v1_vec_skip_errors(value)),
        crate::MaybeUndefined::Null => Some(Vec::new()),
        crate::MaybeUndefined::Undefined => None,
    }
}

fn option_into_v2_maybe_undefined<T>(value: Option<T>) -> Result<crate::MaybeUndefined<T::Output>>
where
    T: IntoV2,
{
    match value {
        Some(value) => Ok(crate::MaybeUndefined::Value(value.into_v2()?)),
        None => Ok(crate::MaybeUndefined::Undefined),
    }
}

fn option_vec_into_v2_maybe_undefined_skip_errors<T>(
    value: Option<Vec<T>>,
) -> crate::MaybeUndefined<Vec<T::Output>>
where
    T: IntoV2,
{
    match value {
        Some(value) => crate::MaybeUndefined::Value(into_v2_vec_skip_errors(value)),
        None => crate::MaybeUndefined::Undefined,
    }
}

fn vec_into_v2_maybe_undefined_skip_errors<T>(
    value: Vec<T>,
) -> crate::MaybeUndefined<Vec<T::Output>>
where
    T: IntoV2,
{
    if value.is_empty() {
        crate::MaybeUndefined::Undefined
    } else {
        crate::MaybeUndefined::Value(into_v2_vec_skip_errors(value))
    }
}

impl IntoV1 for super::ToolCallUpdate {
    type Output = crate::v1::ToolCallUpdate;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            tool_call_id,
            title,
            kind,
            status,
            content,
            locations,
            raw_input,
            raw_output,
            meta,
        } = self;
        Ok(crate::v1::ToolCallUpdate {
            tool_call_id: tool_call_id.into_v1()?,
            fields: crate::v1::ToolCallUpdateFields {
                kind: maybe_undefined_value_into_v1_option(kind),
                status: maybe_undefined_value_into_v1_option(status),
                title: maybe_undefined_value_into_v1_option(title),
                content: maybe_undefined_vec_into_v1_option(content),
                locations: maybe_undefined_vec_into_v1_option(locations),
                raw_input: maybe_undefined_value_into_v1_option(raw_input),
                raw_output: maybe_undefined_value_into_v1_option(raw_output),
            },
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ToolCall {
    type Output = super::ToolCallUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            tool_call_id,
            title,
            kind,
            status,
            content,
            locations,
            raw_input,
            raw_output,
            meta,
        } = self;
        Ok(super::ToolCallUpdate {
            tool_call_id: tool_call_id.into_v2()?,
            title: crate::MaybeUndefined::Value(title.into_v2()?),
            kind: if matches!(kind, crate::v1::ToolKind::Other) {
                crate::MaybeUndefined::Undefined
            } else {
                crate::MaybeUndefined::Value(kind.into_v2()?)
            },
            status: if matches!(status, crate::v1::ToolCallStatus::Pending) {
                crate::MaybeUndefined::Undefined
            } else {
                crate::MaybeUndefined::Value(status.into_v2()?)
            },
            content: vec_into_v2_maybe_undefined_skip_errors(content),
            locations: vec_into_v2_maybe_undefined_skip_errors(locations),
            raw_input: option_into_v2_maybe_undefined(raw_input)?,
            raw_output: option_into_v2_maybe_undefined(raw_output)?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV2 for crate::v1::ToolCallUpdate {
    type Output = super::ToolCallUpdate;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            tool_call_id,
            fields,
            meta,
        } = self;
        let crate::v1::ToolCallUpdateFields {
            kind,
            status,
            title,
            content,
            locations,
            raw_input,
            raw_output,
        } = fields;
        Ok(super::ToolCallUpdate {
            tool_call_id: tool_call_id.into_v2()?,
            kind: option_into_v2_maybe_undefined(kind)?,
            status: option_into_v2_maybe_undefined(status)?,
            title: option_into_v2_maybe_undefined(title)?,
            content: option_vec_into_v2_maybe_undefined_skip_errors(content),
            locations: option_vec_into_v2_maybe_undefined_skip_errors(locations),
            raw_input: option_into_v2_maybe_undefined(raw_input)?,
            raw_output: option_into_v2_maybe_undefined(raw_output)?,
            meta: meta.into_v2()?,
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
            Self::Unknown(value) => return Err(unknown_v2_enum_variant("ToolKind", &value)),
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
            Self::Other(value) => return Err(unknown_v2_enum_variant("ToolCallStatus", &value)),
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
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant("ToolCallContent", &value.type_));
            }
        })
    }
}

impl IntoV2 for crate::v1::ToolCallContent {
    type Output = super::ToolCallContent;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Content(value) => super::ToolCallContent::Content(Box::new(value.into_v2()?)),
            Self::Diff(value) => super::ToolCallContent::Diff(value.into_v2()?),
            Self::Terminal(_) => {
                return Err(removed_v1_enum_variant("ToolCallContent", "terminal"));
            }
        })
    }
}

impl IntoV1 for super::Content {
    type Output = crate::v1::Content;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { content, meta } = self;
        Ok(crate::v1::Content {
            content: content.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Content {
    type Output = super::Content;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { content, meta } = self;
        Ok(super::Content {
            content: content.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::Diff {
    type Output = crate::v1::Diff;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            path,
            old_text,
            new_text,
            meta,
        } = self;
        Ok(crate::v1::Diff {
            path: path.into_v1()?,
            old_text: old_text.into_v1()?,
            new_text: new_text.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Diff {
    type Output = super::Diff;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            path,
            old_text,
            new_text,
            meta,
        } = self;
        Ok(super::Diff {
            path: path.into_v2()?,
            old_text: old_text.into_v2()?,
            new_text: new_text.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ToolCallLocation {
    type Output = crate::v1::ToolCallLocation;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { path, line, meta } = self;
        Ok(crate::v1::ToolCallLocation {
            path: path.into_v1()?,
            line: line.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ToolCallLocation {
    type Output = super::ToolCallLocation;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { path, line, meta } = self;
        Ok(super::ToolCallLocation {
            path: path.into_v2()?,
            line: line.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::InitializeRequest {
    type Output = crate::v1::InitializeRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            protocol_version,
            capabilities,
            info,
            meta,
        } = self;
        Ok(crate::v1::InitializeRequest {
            protocol_version: protocol_version.into_v1()?,
            client_capabilities: capabilities.into_v1()?,
            client_info: Some(info.into_v1()?),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::InitializeRequest {
    type Output = super::InitializeRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            protocol_version,
            client_capabilities,
            client_info,
            meta,
        } = self;
        let info = match client_info {
            Some(client_info) => client_info.into_v2()?,
            None => {
                return Err(ProtocolConversionError::new(
                    "v1 InitializeRequest without `clientInfo` cannot be represented in v2",
                ));
            }
        };
        Ok(super::InitializeRequest {
            protocol_version: protocol_version.into_v2()?,
            capabilities: client_capabilities.into_v2()?,
            info,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::InitializeResponse {
    type Output = crate::v1::InitializeResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            protocol_version,
            capabilities: agent_capabilities,
            auth_methods,
            info,
            meta,
        } = self;
        Ok(crate::v1::InitializeResponse {
            protocol_version: protocol_version.into_v1()?,
            agent_capabilities: agent_capabilities.into_v1()?,
            auth_methods: into_v1_vec_skip_errors(auth_methods),
            agent_info: Some(info.into_v1()?),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::InitializeResponse {
    type Output = super::InitializeResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            protocol_version,
            agent_capabilities,
            auth_methods,
            agent_info,
            meta,
        } = self;
        let info = match agent_info {
            Some(agent_info) => agent_info.into_v2()?,
            None => {
                return Err(ProtocolConversionError::new(
                    "v1 InitializeResponse without `agentInfo` cannot be represented in v2",
                ));
            }
        };
        Ok(super::InitializeResponse {
            protocol_version: protocol_version.into_v2()?,
            capabilities: agent_capabilities.into_v2()?,
            auth_methods: into_v2_vec_skip_errors(auth_methods),
            info,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::Implementation {
    type Output = crate::v1::Implementation;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            name,
            title,
            version,
            meta,
        } = self;
        Ok(crate::v1::Implementation {
            name: name.into_v1()?,
            title: title.into_v1()?,
            version: version.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Implementation {
    type Output = super::Implementation;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            name,
            title,
            version,
            meta,
        } = self;
        Ok(super::Implementation {
            name: name.into_v2()?,
            title: title.into_v2()?,
            version: version.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::LoginAuthRequest {
    type Output = crate::v1::AuthenticateRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { method_id, meta } = self;
        Ok(crate::v1::AuthenticateRequest {
            method_id: method_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AuthenticateRequest {
    type Output = super::LoginAuthRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { method_id, meta } = self;
        Ok(super::LoginAuthRequest {
            method_id: method_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::LoginAuthResponse {
    type Output = crate::v1::AuthenticateResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::AuthenticateResponse {
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AuthenticateResponse {
    type Output = super::LoginAuthResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::LoginAuthResponse {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::LogoutAuthRequest {
    type Output = crate::v1::LogoutRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::LogoutRequest {
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::LogoutRequest {
    type Output = super::LogoutAuthRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::LogoutAuthRequest {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::LogoutAuthResponse {
    type Output = crate::v1::LogoutResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::LogoutResponse {
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::LogoutResponse {
    type Output = super::LogoutAuthResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::LogoutAuthResponse {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AgentAuthCapabilities {
    type Output = crate::v1::AgentAuthCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::AgentAuthCapabilities {
            logout: Some(crate::v1::LogoutCapabilities::new()),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AgentAuthCapabilities {
    type Output = super::AgentAuthCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { logout: _, meta } = self;
        Ok(super::AgentAuthCapabilities {
            meta: meta.into_v2()?,
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
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant("AuthMethod", &value.type_));
            }
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
        let Self {
            id,
            name,
            description,
            meta,
        } = self;
        Ok(crate::v1::AuthMethodAgent {
            id: id.into_v1()?,
            name: name.into_v1()?,
            description: description.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AuthMethodAgent {
    type Output = super::AuthMethodAgent;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            id,
            name,
            description,
            meta,
        } = self;
        Ok(super::AuthMethodAgent {
            id: id.into_v2()?,
            name: name.into_v2()?,
            description: description.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV1 for super::AuthMethodEnvVar {
    type Output = crate::v1::AuthMethodEnvVar;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            id,
            name,
            description,
            vars,
            link,
            meta,
        } = self;
        Ok(crate::v1::AuthMethodEnvVar {
            id: id.into_v1()?,
            name: name.into_v1()?,
            description: description.into_v1()?,
            vars: vars.into_v1()?,
            link: link.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV2 for crate::v1::AuthMethodEnvVar {
    type Output = super::AuthMethodEnvVar;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            id,
            name,
            description,
            vars,
            link,
            meta,
        } = self;
        Ok(super::AuthMethodEnvVar {
            id: id.into_v2()?,
            name: name.into_v2()?,
            description: description.into_v2()?,
            vars: vars.into_v2()?,
            link: link.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV1 for super::AuthEnvVar {
    type Output = crate::v1::AuthEnvVar;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            name,
            label,
            secret,
            optional,
            meta,
        } = self;
        Ok(crate::v1::AuthEnvVar {
            name: name.into_v1()?,
            label: label.into_v1()?,
            secret: secret.into_v1()?,
            optional: optional.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV2 for crate::v1::AuthEnvVar {
    type Output = super::AuthEnvVar;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            name,
            label,
            secret,
            optional,
            meta,
        } = self;
        Ok(super::AuthEnvVar {
            name: name.into_v2()?,
            label: label.into_v2()?,
            secret: secret.into_v2()?,
            optional: optional.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV1 for super::AuthMethodTerminal {
    type Output = crate::v1::AuthMethodTerminal;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            id,
            name,
            description,
            args,
            env,
            meta,
        } = self;
        let env = env
            .into_iter()
            .map(|env_var| {
                let super::EnvVariable { name, value, meta } = env_var;
                if meta.is_some() {
                    return Err(ProtocolConversionError::new(
                        "v2 AuthMethodTerminal env variable `_meta` cannot be represented in v1",
                    ));
                }
                Ok((name.into_v1()?, value.into_v1()?))
            })
            .try_fold(HashMap::new(), |mut env, item| {
                let (name, value) = item?;
                if env.insert(name.clone(), value).is_some() {
                    return Err(ProtocolConversionError::new(format!(
                        "v2 AuthMethodTerminal env variable `{name}` is duplicated and cannot be represented in v1",
                    )));
                }
                Ok(env)
            })?;
        Ok(crate::v1::AuthMethodTerminal {
            id: id.into_v1()?,
            name: name.into_v1()?,
            description: description.into_v1()?,
            args: args.into_v1()?,
            env,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl IntoV2 for crate::v1::AuthMethodTerminal {
    type Output = super::AuthMethodTerminal;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            id,
            name,
            description,
            args,
            env,
            meta,
        } = self;
        let mut env = env
            .into_iter()
            .map(|(name, value)| Ok(super::EnvVariable::new(name.into_v2()?, value.into_v2()?)))
            .collect::<Result<Vec<_>>>()?;
        env.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(super::AuthMethodTerminal {
            id: id.into_v2()?,
            name: name.into_v2()?,
            description: description.into_v2()?,
            args: args.into_v2()?,
            env,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::NewSessionRequest {
    type Output = crate::v1::NewSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            cwd,
            additional_directories,
            mcp_servers,
            meta,
        } = self;
        Ok(crate::v1::NewSessionRequest {
            cwd: cwd.into_v1()?,
            additional_directories: additional_directories.into_v1()?,
            mcp_servers: mcp_servers.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::NewSessionRequest {
    type Output = super::NewSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            cwd,
            additional_directories,
            mcp_servers,
            meta,
        } = self;
        Ok(super::NewSessionRequest {
            cwd: cwd.into_v2()?,
            additional_directories: additional_directories.into_v2()?,
            mcp_servers: mcp_servers.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::NewSessionResponse {
    type Output = crate::v1::NewSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            config_options,
            meta,
        } = self;
        Ok(crate::v1::NewSessionResponse {
            session_id: session_id.into_v1()?,
            modes: None,
            config_options: Some(into_v1_vec_skip_errors(config_options)),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::NewSessionResponse {
    type Output = super::NewSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            modes: _,
            config_options,
            meta,
        } = self;
        Ok(super::NewSessionResponse {
            session_id: session_id.into_v2()?,
            config_options: option_vec_into_v2_default_skip_errors(config_options),
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::LoadSessionRequest {
    type Output = crate::v1::LoadSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            mcp_servers,
            cwd,
            additional_directories,
            session_id,
            meta,
        } = self;
        Ok(crate::v1::LoadSessionRequest {
            mcp_servers: mcp_servers.into_v1()?,
            cwd: cwd.into_v1()?,
            additional_directories: additional_directories.into_v1()?,
            session_id: session_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::LoadSessionRequest {
    type Output = super::LoadSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            mcp_servers,
            cwd,
            additional_directories,
            session_id,
            meta,
        } = self;
        Ok(super::LoadSessionRequest {
            mcp_servers: mcp_servers.into_v2()?,
            cwd: cwd.into_v2()?,
            additional_directories: additional_directories.into_v2()?,
            session_id: session_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::LoadSessionResponse {
    type Output = crate::v1::LoadSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            config_options,
            meta,
        } = self;
        Ok(crate::v1::LoadSessionResponse {
            modes: None,
            config_options: Some(into_v1_vec_skip_errors(config_options)),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::LoadSessionResponse {
    type Output = super::LoadSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            modes: _,
            config_options,
            meta,
        } = self;
        Ok(super::LoadSessionResponse {
            config_options: option_vec_into_v2_default_skip_errors(config_options),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV1 for super::ForkSessionRequest {
    type Output = crate::v1::ForkSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            cwd,
            additional_directories,
            mcp_servers,
            meta,
        } = self;
        Ok(crate::v1::ForkSessionRequest {
            session_id: session_id.into_v1()?,
            cwd: cwd.into_v1()?,
            additional_directories: additional_directories.into_v1()?,
            mcp_servers: mcp_servers.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV2 for crate::v1::ForkSessionRequest {
    type Output = super::ForkSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            cwd,
            additional_directories,
            mcp_servers,
            meta,
        } = self;
        Ok(super::ForkSessionRequest {
            session_id: session_id.into_v2()?,
            cwd: cwd.into_v2()?,
            additional_directories: additional_directories.into_v2()?,
            mcp_servers: mcp_servers.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV1 for super::ForkSessionResponse {
    type Output = crate::v1::ForkSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            config_options,
            meta,
        } = self;
        Ok(crate::v1::ForkSessionResponse {
            session_id: session_id.into_v1()?,
            modes: None,
            config_options: Some(into_v1_vec_skip_errors(config_options)),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV2 for crate::v1::ForkSessionResponse {
    type Output = super::ForkSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            modes: _,
            config_options,
            meta,
        } = self;
        Ok(super::ForkSessionResponse {
            session_id: session_id.into_v2()?,
            config_options: option_vec_into_v2_default_skip_errors(config_options),
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ResumeSessionRequest {
    type Output = crate::v1::ResumeSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            cwd,
            additional_directories,
            mcp_servers,
            meta,
        } = self;
        Ok(crate::v1::ResumeSessionRequest {
            session_id: session_id.into_v1()?,
            cwd: cwd.into_v1()?,
            additional_directories: additional_directories.into_v1()?,
            mcp_servers: mcp_servers.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ResumeSessionRequest {
    type Output = super::ResumeSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            cwd,
            additional_directories,
            mcp_servers,
            meta,
        } = self;
        Ok(super::ResumeSessionRequest {
            session_id: session_id.into_v2()?,
            cwd: cwd.into_v2()?,
            additional_directories: additional_directories.into_v2()?,
            mcp_servers: mcp_servers.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ResumeSessionResponse {
    type Output = crate::v1::ResumeSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            config_options,
            meta,
        } = self;
        Ok(crate::v1::ResumeSessionResponse {
            modes: None,
            config_options: Some(into_v1_vec_skip_errors(config_options)),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ResumeSessionResponse {
    type Output = super::ResumeSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            modes: _,
            config_options,
            meta,
        } = self;
        Ok(super::ResumeSessionResponse {
            config_options: option_vec_into_v2_default_skip_errors(config_options),
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::CloseSessionRequest {
    type Output = crate::v1::CloseSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { session_id, meta } = self;
        Ok(crate::v1::CloseSessionRequest {
            session_id: session_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::CloseSessionRequest {
    type Output = super::CloseSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { session_id, meta } = self;
        Ok(super::CloseSessionRequest {
            session_id: session_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::CloseSessionResponse {
    type Output = crate::v1::CloseSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::CloseSessionResponse {
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::CloseSessionResponse {
    type Output = super::CloseSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::CloseSessionResponse {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::DeleteSessionRequest {
    type Output = crate::v1::DeleteSessionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { session_id, meta } = self;
        Ok(crate::v1::DeleteSessionRequest {
            session_id: session_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::DeleteSessionRequest {
    type Output = super::DeleteSessionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { session_id, meta } = self;
        Ok(super::DeleteSessionRequest {
            session_id: session_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::DeleteSessionResponse {
    type Output = crate::v1::DeleteSessionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::DeleteSessionResponse {
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::DeleteSessionResponse {
    type Output = super::DeleteSessionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::DeleteSessionResponse {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ListSessionsRequest {
    type Output = crate::v1::ListSessionsRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { cwd, cursor, meta } = self;
        Ok(crate::v1::ListSessionsRequest {
            cwd: cwd.into_v1()?,
            cursor: cursor.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ListSessionsRequest {
    type Output = super::ListSessionsRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { cwd, cursor, meta } = self;
        Ok(super::ListSessionsRequest {
            cwd: cwd.into_v2()?,
            cursor: cursor.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ListSessionsResponse {
    type Output = crate::v1::ListSessionsResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            sessions,
            next_cursor,
            meta,
        } = self;
        Ok(crate::v1::ListSessionsResponse {
            sessions: into_v1_vec_skip_errors(sessions),
            next_cursor: next_cursor.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ListSessionsResponse {
    type Output = super::ListSessionsResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            sessions,
            next_cursor,
            meta,
        } = self;
        Ok(super::ListSessionsResponse {
            sessions: into_v2_vec_skip_errors(sessions),
            next_cursor: next_cursor.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionInfo {
    type Output = crate::v1::SessionInfo;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            cwd,
            additional_directories,
            title,
            updated_at,
            meta,
        } = self;
        Ok(crate::v1::SessionInfo {
            session_id: session_id.into_v1()?,
            cwd: cwd.into_v1()?,
            additional_directories: additional_directories.into_v1()?,
            title: into_v1_default_on_error(title),
            updated_at: into_v1_default_on_error(updated_at),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionInfo {
    type Output = super::SessionInfo;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            cwd,
            additional_directories,
            title,
            updated_at,
            meta,
        } = self;
        Ok(super::SessionInfo {
            session_id: session_id.into_v2()?,
            cwd: cwd.into_v2()?,
            additional_directories: additional_directories.into_v2()?,
            title: into_v2_default_on_error(title),
            updated_at: into_v2_default_on_error(updated_at),
            meta: meta.into_v2()?,
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
        let Self {
            value,
            name,
            description,
            meta,
        } = self;
        Ok(crate::v1::SessionConfigSelectOption {
            value: value.into_v1()?,
            name: name.into_v1()?,
            description: description.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigSelectOption {
    type Output = super::SessionConfigSelectOption;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            value,
            name,
            description,
            meta,
        } = self;
        Ok(super::SessionConfigSelectOption {
            value: value.into_v2()?,
            name: name.into_v2()?,
            description: description.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionConfigSelectGroup {
    type Output = crate::v1::SessionConfigSelectGroup;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            group,
            name,
            options,
            meta,
        } = self;
        Ok(crate::v1::SessionConfigSelectGroup {
            group: group.into_v1()?,
            name: name.into_v1()?,
            options: options.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigSelectGroup {
    type Output = super::SessionConfigSelectGroup;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            group,
            name,
            options,
            meta,
        } = self;
        Ok(super::SessionConfigSelectGroup {
            group: group.into_v2()?,
            name: name.into_v2()?,
            options: options.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self {
            current_value,
            options,
        } = self;
        Ok(crate::v1::SessionConfigSelect {
            current_value: current_value.into_v1()?,
            options: options.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigSelect {
    type Output = super::SessionConfigSelect;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            current_value,
            options,
        } = self;
        Ok(super::SessionConfigSelect {
            current_value: current_value.into_v2()?,
            options: options.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_boolean_config")]
impl IntoV1 for super::SessionConfigBoolean {
    type Output = crate::v1::SessionConfigBoolean;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { current_value } = self;
        Ok(crate::v1::SessionConfigBoolean {
            current_value: current_value.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_boolean_config")]
impl IntoV2 for crate::v1::SessionConfigBoolean {
    type Output = super::SessionConfigBoolean;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { current_value } = self;
        Ok(super::SessionConfigBoolean {
            current_value: current_value.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionConfigOptionCategory {
    type Output = crate::v1::SessionConfigOptionCategory;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Mode => crate::v1::SessionConfigOptionCategory::Mode,
            Self::Model => crate::v1::SessionConfigOptionCategory::Model,
            Self::ModelConfig => crate::v1::SessionConfigOptionCategory::ModelConfig,
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
            Self::ModelConfig => super::SessionConfigOptionCategory::ModelConfig,
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
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant("SessionConfigKind", &value.type_));
            }
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
        let Self {
            id,
            name,
            description,
            category,
            kind,
            meta,
        } = self;
        Ok(crate::v1::SessionConfigOption {
            id: id.into_v1()?,
            name: name.into_v1()?,
            description: description.into_v1()?,
            category: into_v1_default_on_error(category),
            kind: kind.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionConfigOption {
    type Output = super::SessionConfigOption;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            id,
            name,
            description,
            category,
            kind,
            meta,
        } = self;
        Ok(super::SessionConfigOption {
            id: id.into_v2()?,
            name: name.into_v2()?,
            description: description.into_v2()?,
            category: into_v2_default_on_error(category),
            kind: kind.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_boolean_config")]
impl IntoV1 for super::SessionConfigOptionValue {
    type Output = crate::v1::SessionConfigOptionValue;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Id { value } => crate::v1::SessionConfigOptionValue::ValueId {
                value: value.into_v1()?,
            },
            Self::Boolean { value } => crate::v1::SessionConfigOptionValue::Boolean {
                value: value.into_v1()?,
            },
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant(
                    "SessionConfigOptionValue",
                    &value.type_,
                ));
            }
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
            Self::ValueId { value } => super::SessionConfigOptionValue::Id {
                value: value.into_v2()?,
            },
        })
    }
}

impl IntoV1 for super::SetSessionConfigOptionRequest {
    type Output = crate::v1::SetSessionConfigOptionRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            config_id,
            value,
            meta,
        } = self;
        Ok(crate::v1::SetSessionConfigOptionRequest {
            session_id: session_id.into_v1()?,
            config_id: config_id.into_v1()?,
            value: value.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SetSessionConfigOptionRequest {
    type Output = super::SetSessionConfigOptionRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            config_id,
            value,
            meta,
        } = self;
        Ok(super::SetSessionConfigOptionRequest {
            session_id: session_id.into_v2()?,
            config_id: config_id.into_v2()?,
            value: value.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SetSessionConfigOptionResponse {
    type Output = crate::v1::SetSessionConfigOptionResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            config_options,
            meta,
        } = self;
        Ok(crate::v1::SetSessionConfigOptionResponse {
            config_options: into_v1_vec_skip_errors(config_options),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SetSessionConfigOptionResponse {
    type Output = super::SetSessionConfigOptionResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            config_options,
            meta,
        } = self;
        Ok(super::SetSessionConfigOptionResponse {
            config_options: into_v2_vec_skip_errors(config_options),
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::McpServer {
    type Output = crate::v1::McpServer;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Http(value) => crate::v1::McpServer::Http(value.into_v1()?),
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::Acp(value) => crate::v1::McpServer::Acp(value.into_v1()?),
            Self::Stdio(value) => crate::v1::McpServer::Stdio(value.into_v1()?),
            Self::Other(value) => return Err(unknown_v2_enum_variant("McpServer", &value.type_)),
        })
    }
}

impl IntoV2 for crate::v1::McpServer {
    type Output = super::McpServer;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Http(value) => super::McpServer::Http(value.into_v2()?),
            Self::Sse(_) => return Err(removed_v1_enum_variant("McpServer", "sse")),
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::Acp(value) => super::McpServer::Acp(value.into_v2()?),
            Self::Stdio(value) => super::McpServer::Stdio(value.into_v2()?),
        })
    }
}

impl IntoV1 for super::McpServerHttp {
    type Output = crate::v1::McpServerHttp;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            name,
            url,
            headers,
            meta,
        } = self;
        Ok(crate::v1::McpServerHttp {
            name: name.into_v1()?,
            url: url.into_v1()?,
            headers: headers.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::McpServerHttp {
    type Output = super::McpServerHttp;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            name,
            url,
            headers,
            meta,
        } = self;
        Ok(super::McpServerHttp {
            name: name.into_v2()?,
            url: url.into_v2()?,
            headers: headers.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV1 for super::McpServerAcpId {
    type Output = crate::v1::McpServerAcpId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::McpServerAcpId(self.0.into_v1()?))
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV2 for crate::v1::McpServerAcpId {
    type Output = super::McpServerAcpId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::McpServerAcpId(self.0.into_v2()?))
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV1 for super::McpConnectionId {
    type Output = crate::v1::McpConnectionId;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(crate::v1::McpConnectionId(self.0.into_v1()?))
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV2 for crate::v1::McpConnectionId {
    type Output = super::McpConnectionId;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(super::McpConnectionId(self.0.into_v2()?))
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV1 for super::McpServerAcp {
    type Output = crate::v1::McpServerAcp;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { name, id, meta } = self;
        Ok(crate::v1::McpServerAcp {
            name: name.into_v1()?,
            id: id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_mcp_over_acp")]
impl IntoV2 for crate::v1::McpServerAcp {
    type Output = super::McpServerAcp;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { name, id, meta } = self;
        Ok(super::McpServerAcp {
            name: name.into_v2()?,
            id: id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::McpServerStdio {
    type Output = crate::v1::McpServerStdio;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            name,
            command,
            args,
            env,
            meta,
        } = self;
        Ok(crate::v1::McpServerStdio {
            name: name.into_v1()?,
            command: command.into_v1()?,
            args: args.into_v1()?,
            env: env.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::McpServerStdio {
    type Output = super::McpServerStdio;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            name,
            command,
            args,
            env,
            meta,
        } = self;
        Ok(super::McpServerStdio {
            name: name.into_v2()?,
            command: command.into_v2()?,
            args: args.into_v2()?,
            env: env.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::EnvVariable {
    type Output = crate::v1::EnvVariable;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { name, value, meta } = self;
        Ok(crate::v1::EnvVariable {
            name: name.into_v1()?,
            value: value.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::EnvVariable {
    type Output = super::EnvVariable;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { name, value, meta } = self;
        Ok(super::EnvVariable {
            name: name.into_v2()?,
            value: value.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::HttpHeader {
    type Output = crate::v1::HttpHeader;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { name, value, meta } = self;
        Ok(crate::v1::HttpHeader {
            name: name.into_v1()?,
            value: value.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::HttpHeader {
    type Output = super::HttpHeader;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { name, value, meta } = self;
        Ok(super::HttpHeader {
            name: name.into_v2()?,
            value: value.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PromptRequest {
    type Output = crate::v1::PromptRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            prompt,
            meta,
        } = self;
        Ok(crate::v1::PromptRequest {
            session_id: session_id.into_v1()?,
            prompt: prompt.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::PromptRequest {
    type Output = super::PromptRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            prompt,
            meta,
        } = self;
        Ok(super::PromptRequest {
            session_id: session_id.into_v2()?,
            prompt: prompt.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PromptResponse {
    type Output = crate::v1::PromptResponse;

    fn into_v1(self) -> Result<Self::Output> {
        Err(ProtocolConversionError::new(
            "v2 PromptResponse cannot be represented in v1 because v2 reports completion with state_update session updates",
        ))
    }
}

impl IntoV2 for crate::v1::PromptResponse {
    type Output = super::PromptResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Err(ProtocolConversionError::new(
            "v1 PromptResponse cannot be represented in v2 by itself because v2 reports completion with state_update session updates",
        ))
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
            Self::Other(value) => return Err(unknown_v2_enum_variant("StopReason", &value)),
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

#[cfg(feature = "unstable_end_turn_token_usage")]
impl IntoV1 for super::Usage {
    type Output = crate::v1::Usage;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            total_tokens,
            input_tokens,
            output_tokens,
            thought_tokens,
            cached_read_tokens,
            cached_write_tokens,
            meta,
        } = self;
        Ok(crate::v1::Usage {
            total_tokens: total_tokens.into_v1()?,
            input_tokens: input_tokens.into_v1()?,
            output_tokens: output_tokens.into_v1()?,
            thought_tokens: thought_tokens.into_v1()?,
            cached_read_tokens: cached_read_tokens.into_v1()?,
            cached_write_tokens: cached_write_tokens.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_end_turn_token_usage")]
impl IntoV2 for crate::v1::Usage {
    type Output = super::Usage;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            total_tokens,
            input_tokens,
            output_tokens,
            thought_tokens,
            cached_read_tokens,
            cached_write_tokens,
            meta,
        } = self;
        Ok(super::Usage {
            total_tokens: total_tokens.into_v2()?,
            input_tokens: input_tokens.into_v2()?,
            output_tokens: output_tokens.into_v2()?,
            thought_tokens: thought_tokens.into_v2()?,
            cached_read_tokens: cached_read_tokens.into_v2()?,
            cached_write_tokens: cached_write_tokens.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self {
            api_type,
            base_url,
            meta,
        } = self;
        Ok(crate::v1::ProviderCurrentConfig {
            api_type: api_type.into_v1()?,
            base_url: base_url.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::ProviderCurrentConfig {
    type Output = super::ProviderCurrentConfig;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            api_type,
            base_url,
            meta,
        } = self;
        Ok(super::ProviderCurrentConfig {
            api_type: api_type.into_v2()?,
            base_url: base_url.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::ProviderInfo {
    type Output = crate::v1::ProviderInfo;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            id,
            supported,
            required,
            current,
            meta,
        } = self;
        Ok(crate::v1::ProviderInfo {
            id: id.into_v1()?,
            supported: into_v1_vec_skip_errors(supported),
            required: required.into_v1()?,
            current: current.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::ProviderInfo {
    type Output = super::ProviderInfo;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            id,
            supported,
            required,
            current,
            meta,
        } = self;
        Ok(super::ProviderInfo {
            id: id.into_v2()?,
            supported: into_v2_vec_skip_errors(supported),
            required: required.into_v2()?,
            current: current.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::ListProvidersRequest {
    type Output = crate::v1::ListProvidersRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::ListProvidersRequest {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::ListProvidersRequest {
    type Output = super::ListProvidersRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::ListProvidersRequest {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::ListProvidersResponse {
    type Output = crate::v1::ListProvidersResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { providers, meta } = self;
        Ok(crate::v1::ListProvidersResponse {
            providers: into_v1_vec_skip_errors(providers),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::ListProvidersResponse {
    type Output = super::ListProvidersResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { providers, meta } = self;
        Ok(super::ListProvidersResponse {
            providers: into_v2_vec_skip_errors(providers),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::SetProviderRequest {
    type Output = crate::v1::SetProviderRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            id,
            api_type,
            base_url,
            headers,
            meta,
        } = self;
        Ok(crate::v1::SetProviderRequest {
            id: id.into_v1()?,
            api_type: api_type.into_v1()?,
            base_url: base_url.into_v1()?,
            headers: headers.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::SetProviderRequest {
    type Output = super::SetProviderRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            id,
            api_type,
            base_url,
            headers,
            meta,
        } = self;
        Ok(super::SetProviderRequest {
            id: id.into_v2()?,
            api_type: api_type.into_v2()?,
            base_url: base_url.into_v2()?,
            headers: headers.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::SetProviderResponse {
    type Output = crate::v1::SetProviderResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::SetProviderResponse {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::SetProviderResponse {
    type Output = super::SetProviderResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::SetProviderResponse {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::DisableProviderRequest {
    type Output = crate::v1::DisableProviderRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { id, meta } = self;
        Ok(crate::v1::DisableProviderRequest {
            id: id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::DisableProviderRequest {
    type Output = super::DisableProviderRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { id, meta } = self;
        Ok(super::DisableProviderRequest {
            id: id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::DisableProviderResponse {
    type Output = crate::v1::DisableProviderResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::DisableProviderResponse {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::DisableProviderResponse {
    type Output = super::DisableProviderResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::DisableProviderResponse {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AgentCapabilities {
    type Output = crate::v1::AgentCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session,
            auth,
            #[cfg(feature = "unstable_llm_providers")]
            providers,
            #[cfg(feature = "unstable_nes")]
            nes,
            #[cfg(feature = "unstable_nes")]
            position_encoding,
            meta,
        } = self;
        let Some(session) = session else {
            return Err(ProtocolConversionError::new(
                "v2 AgentCapabilities without `session` cannot be represented in v1",
            ));
        };
        let V1SessionCapabilityParts {
            session_capabilities,
            prompt_capabilities,
            load_session,
            mcp_capabilities,
        } = session.into_v1()?;

        Ok(crate::v1::AgentCapabilities {
            load_session: load_session.into_v1()?,
            prompt_capabilities,
            mcp_capabilities,
            session_capabilities,
            auth: auth.map(IntoV1::into_v1).transpose()?.unwrap_or_default(),
            #[cfg(feature = "unstable_llm_providers")]
            providers: into_v1_default_on_error(providers),
            #[cfg(feature = "unstable_nes")]
            nes: into_v1_default_on_error(nes),
            #[cfg(feature = "unstable_nes")]
            position_encoding: into_v1_default_on_error(position_encoding),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AgentCapabilities {
    type Output = super::AgentCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            load_session,
            prompt_capabilities,
            mcp_capabilities,
            session_capabilities,
            auth,
            #[cfg(feature = "unstable_llm_providers")]
            providers,
            #[cfg(feature = "unstable_nes")]
            nes,
            #[cfg(feature = "unstable_nes")]
            position_encoding,
            meta,
        } = self;
        let session = super::SessionCapabilities::from_v1(
            session_capabilities,
            prompt_capabilities,
            load_session,
            mcp_capabilities,
        )?;

        Ok(super::AgentCapabilities {
            session: Some(session),
            auth: Some(auth.into_v2()?),
            #[cfg(feature = "unstable_llm_providers")]
            providers: into_v2_default_on_error(providers),
            #[cfg(feature = "unstable_nes")]
            nes: into_v2_default_on_error(nes),
            #[cfg(feature = "unstable_nes")]
            position_encoding: into_v2_default_on_error(position_encoding),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV1 for super::ProvidersCapabilities {
    type Output = crate::v1::ProvidersCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::ProvidersCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_llm_providers")]
impl IntoV2 for crate::v1::ProvidersCapabilities {
    type Output = super::ProvidersCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::ProvidersCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

/// The v1 capability fields represented by v2 `SessionCapabilities`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct V1SessionCapabilityParts {
    /// Session-specific v1 capabilities.
    pub session_capabilities: crate::v1::SessionCapabilities,
    /// Prompt capabilities for v1 `session/prompt` requests.
    pub prompt_capabilities: crate::v1::PromptCapabilities,
    /// Whether v1 `session/load` is supported.
    pub load_session: bool,
    /// MCP capabilities for v1 session lifecycle requests.
    pub mcp_capabilities: crate::v1::McpCapabilities,
}

impl super::SessionCapabilities {
    /// Converts these v2 draft session capabilities into the v1 capability
    /// fields they represent.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolConversionError`] when any contained capability field
    /// cannot be represented in v1.
    pub fn into_v1(self) -> Result<V1SessionCapabilityParts> {
        let Self {
            prompt,
            mcp,
            load,
            list,
            delete,
            additional_directories,
            #[cfg(feature = "unstable_session_fork")]
            fork,
            resume,
            close,
            meta,
        } = self;

        Ok(V1SessionCapabilityParts {
            session_capabilities: crate::v1::SessionCapabilities {
                list: into_v1_default_on_error(list),
                delete: into_v1_default_on_error(delete),
                additional_directories: into_v1_default_on_error(additional_directories),
                #[cfg(feature = "unstable_session_fork")]
                fork: into_v1_default_on_error(fork),
                resume: into_v1_default_on_error(resume),
                close: into_v1_default_on_error(close),
                meta: meta.into_v1()?,
            },
            prompt_capabilities: prompt.unwrap_or_default().into_v1()?,
            load_session: load.is_some(),
            mcp_capabilities: mcp.unwrap_or_default().into_v1()?,
        })
    }

    /// Builds v2 draft session capabilities from the v1 agent capability fields
    /// that now live under `session` in v2.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolConversionError`] when any of the supplied v1
    /// capability fields cannot be represented in v2.
    pub fn from_v1(
        session_capabilities: crate::v1::SessionCapabilities,
        prompt_capabilities: crate::v1::PromptCapabilities,
        load_session: bool,
        mcp_capabilities: crate::v1::McpCapabilities,
    ) -> Result<Self> {
        let crate::v1::SessionCapabilities {
            list,
            delete,
            additional_directories,
            #[cfg(feature = "unstable_session_fork")]
            fork,
            resume,
            close,
            meta,
        } = session_capabilities;

        Ok(super::SessionCapabilities {
            prompt: Some(prompt_capabilities.into_v2()?),
            mcp: Some(mcp_capabilities.into_v2()?),
            load: load_session.then(super::SessionLoadCapabilities::new),
            list: into_v2_default_on_error(list),
            delete: into_v2_default_on_error(delete),
            additional_directories: into_v2_default_on_error(additional_directories),
            #[cfg(feature = "unstable_session_fork")]
            fork: into_v2_default_on_error(fork),
            resume: into_v2_default_on_error(resume),
            close: into_v2_default_on_error(close),
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionListCapabilities {
    type Output = crate::v1::SessionListCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::SessionListCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionListCapabilities {
    type Output = super::SessionListCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::SessionListCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionDeleteCapabilities {
    type Output = crate::v1::SessionDeleteCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::SessionDeleteCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionDeleteCapabilities {
    type Output = super::SessionDeleteCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::SessionDeleteCapabilities {
            meta: meta.into_v2()?,
        })
    }
}
impl IntoV1 for super::SessionAdditionalDirectoriesCapabilities {
    type Output = crate::v1::SessionAdditionalDirectoriesCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::SessionAdditionalDirectoriesCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionAdditionalDirectoriesCapabilities {
    type Output = super::SessionAdditionalDirectoriesCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::SessionAdditionalDirectoriesCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV1 for super::SessionForkCapabilities {
    type Output = crate::v1::SessionForkCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::SessionForkCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_session_fork")]
impl IntoV2 for crate::v1::SessionForkCapabilities {
    type Output = super::SessionForkCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::SessionForkCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionResumeCapabilities {
    type Output = crate::v1::SessionResumeCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::SessionResumeCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionResumeCapabilities {
    type Output = super::SessionResumeCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::SessionResumeCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::SessionCloseCapabilities {
    type Output = crate::v1::SessionCloseCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::SessionCloseCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::SessionCloseCapabilities {
    type Output = super::SessionCloseCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::SessionCloseCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::PromptCapabilities {
    type Output = crate::v1::PromptCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            image,
            audio,
            embedded_context,
            meta,
        } = self;
        Ok(crate::v1::PromptCapabilities {
            image: image.is_some(),
            audio: audio.is_some(),
            embedded_context: embedded_context.is_some(),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::PromptCapabilities {
    type Output = super::PromptCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            image,
            audio,
            embedded_context,
            meta,
        } = self;
        Ok(super::PromptCapabilities {
            image: image.then(super::PromptImageCapabilities::new),
            audio: audio.then(super::PromptAudioCapabilities::new),
            embedded_context: embedded_context.then(super::PromptEmbeddedContextCapabilities::new),
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::McpCapabilities {
    type Output = crate::v1::McpCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            stdio: _,
            http,
            #[cfg(feature = "unstable_mcp_over_acp")]
            acp,
            meta,
        } = self;
        Ok(crate::v1::McpCapabilities {
            http: http.is_some(),
            sse: false,
            #[cfg(feature = "unstable_mcp_over_acp")]
            acp: acp.is_some(),
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::McpCapabilities {
    type Output = super::McpCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            http,
            sse: _,
            #[cfg(feature = "unstable_mcp_over_acp")]
            acp,
            meta,
        } = self;
        Ok(super::McpCapabilities {
            stdio: Some(super::McpStdioCapabilities::new()),
            http: http.then(super::McpHttpCapabilities::new),
            #[cfg(feature = "unstable_mcp_over_acp")]
            acp: acp.then(super::McpAcpCapabilities::new),
            meta: meta.into_v2()?,
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
            Self::LoginAuthRequest(value) => {
                crate::v1::ClientRequest::AuthenticateRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::ListProvidersRequest(value) => {
                crate::v1::ClientRequest::ListProvidersRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::SetProviderRequest(value) => {
                crate::v1::ClientRequest::SetProviderRequest(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::DisableProviderRequest(value) => {
                crate::v1::ClientRequest::DisableProviderRequest(value.into_v1()?)
            }
            Self::LogoutAuthRequest(value) => {
                crate::v1::ClientRequest::LogoutRequest(value.into_v1()?)
            }
            Self::NewSessionRequest(value) => {
                crate::v1::ClientRequest::NewSessionRequest(value.into_v1()?)
            }
            Self::LoadSessionRequest(value) => {
                crate::v1::ClientRequest::LoadSessionRequest(value.into_v1()?)
            }
            Self::ListSessionsRequest(value) => {
                crate::v1::ClientRequest::ListSessionsRequest(value.into_v1()?)
            }
            Self::DeleteSessionRequest(value) => {
                crate::v1::ClientRequest::DeleteSessionRequest(value.into_v1()?)
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
            Self::SetSessionConfigOptionRequest(value) => {
                crate::v1::ClientRequest::SetSessionConfigOptionRequest(value.into_v1()?)
            }
            Self::PromptRequest(value) => crate::v1::ClientRequest::PromptRequest(value.into_v1()?),
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
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpRequest(value) => {
                crate::v1::ClientRequest::MessageMcpRequest(value.into_v1()?)
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
                super::ClientRequest::InitializeRequest(Box::new(value.into_v2()?))
            }
            Self::AuthenticateRequest(value) => {
                super::ClientRequest::LoginAuthRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::ListProvidersRequest(value) => {
                super::ClientRequest::ListProvidersRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::SetProviderRequest(value) => {
                super::ClientRequest::SetProviderRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::DisableProviderRequest(value) => {
                super::ClientRequest::DisableProviderRequest(Box::new(value.into_v2()?))
            }
            Self::LogoutRequest(value) => {
                super::ClientRequest::LogoutAuthRequest(Box::new(value.into_v2()?))
            }
            Self::NewSessionRequest(value) => {
                super::ClientRequest::NewSessionRequest(Box::new(value.into_v2()?))
            }
            Self::LoadSessionRequest(value) => {
                super::ClientRequest::LoadSessionRequest(Box::new(value.into_v2()?))
            }
            Self::ListSessionsRequest(value) => {
                super::ClientRequest::ListSessionsRequest(Box::new(value.into_v2()?))
            }
            Self::DeleteSessionRequest(value) => {
                super::ClientRequest::DeleteSessionRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_session_fork")]
            Self::ForkSessionRequest(value) => {
                super::ClientRequest::ForkSessionRequest(Box::new(value.into_v2()?))
            }
            Self::ResumeSessionRequest(value) => {
                super::ClientRequest::ResumeSessionRequest(Box::new(value.into_v2()?))
            }
            Self::CloseSessionRequest(value) => {
                super::ClientRequest::CloseSessionRequest(Box::new(value.into_v2()?))
            }
            Self::SetSessionModeRequest(_) => {
                return Err(removed_v1_enum_variant("ClientRequest", "session/set_mode"));
            }
            Self::SetSessionConfigOptionRequest(value) => {
                super::ClientRequest::SetSessionConfigOptionRequest(Box::new(value.into_v2()?))
            }
            Self::PromptRequest(value) => {
                super::ClientRequest::PromptRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::StartNesRequest(value) => {
                super::ClientRequest::StartNesRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::SuggestNesRequest(value) => {
                super::ClientRequest::SuggestNesRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::CloseNesRequest(value) => {
                super::ClientRequest::CloseNesRequest(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpRequest(value) => {
                super::ClientRequest::MessageMcpRequest(Box::new(value.into_v2()?))
            }
            Self::ExtMethodRequest(value) => {
                super::ClientRequest::ExtMethodRequest(Box::new(value.into_v2()?))
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
            Self::LoginAuthResponse(value) => {
                crate::v1::AgentResponse::AuthenticateResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::ListProvidersResponse(value) => {
                crate::v1::AgentResponse::ListProvidersResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::SetProviderResponse(value) => {
                crate::v1::AgentResponse::SetProviderResponse(value.into_v1()?)
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::DisableProviderResponse(value) => {
                crate::v1::AgentResponse::DisableProviderResponse(value.into_v1()?)
            }
            Self::LogoutAuthResponse(value) => {
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
            Self::DeleteSessionResponse(value) => {
                crate::v1::AgentResponse::DeleteSessionResponse(value.into_v1()?)
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
            Self::SetSessionConfigOptionResponse(value) => {
                crate::v1::AgentResponse::SetSessionConfigOptionResponse(value.into_v1()?)
            }
            Self::PromptResponse(value) => {
                crate::v1::AgentResponse::PromptResponse(value.into_v1()?)
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
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpResponse(value) => {
                crate::v1::AgentResponse::MessageMcpResponse(value.into_v1()?)
            }
        })
    }
}

impl IntoV2 for crate::v1::AgentResponse {
    type Output = super::AgentResponse;

    fn into_v2(self) -> Result<Self::Output> {
        Ok(match self {
            Self::InitializeResponse(value) => {
                super::AgentResponse::InitializeResponse(Box::new(value.into_v2()?))
            }
            Self::AuthenticateResponse(value) => {
                super::AgentResponse::LoginAuthResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::ListProvidersResponse(value) => {
                super::AgentResponse::ListProvidersResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::SetProviderResponse(value) => {
                super::AgentResponse::SetProviderResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_llm_providers")]
            Self::DisableProviderResponse(value) => {
                super::AgentResponse::DisableProviderResponse(Box::new(value.into_v2()?))
            }
            Self::LogoutResponse(value) => {
                super::AgentResponse::LogoutAuthResponse(Box::new(value.into_v2()?))
            }
            Self::NewSessionResponse(value) => {
                super::AgentResponse::NewSessionResponse(Box::new(value.into_v2()?))
            }
            Self::LoadSessionResponse(value) => {
                super::AgentResponse::LoadSessionResponse(Box::new(value.into_v2()?))
            }
            Self::ListSessionsResponse(value) => {
                super::AgentResponse::ListSessionsResponse(Box::new(value.into_v2()?))
            }
            Self::DeleteSessionResponse(value) => {
                super::AgentResponse::DeleteSessionResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_session_fork")]
            Self::ForkSessionResponse(value) => {
                super::AgentResponse::ForkSessionResponse(Box::new(value.into_v2()?))
            }
            Self::ResumeSessionResponse(value) => {
                super::AgentResponse::ResumeSessionResponse(Box::new(value.into_v2()?))
            }
            Self::CloseSessionResponse(value) => {
                super::AgentResponse::CloseSessionResponse(Box::new(value.into_v2()?))
            }
            Self::SetSessionModeResponse(_) => {
                return Err(removed_v1_enum_variant("AgentResponse", "session/set_mode"));
            }
            Self::SetSessionConfigOptionResponse(value) => {
                super::AgentResponse::SetSessionConfigOptionResponse(Box::new(value.into_v2()?))
            }
            Self::PromptResponse(value) => {
                super::AgentResponse::PromptResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::StartNesResponse(value) => {
                super::AgentResponse::StartNesResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::SuggestNesResponse(value) => {
                super::AgentResponse::SuggestNesResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::CloseNesResponse(value) => {
                super::AgentResponse::CloseNesResponse(Box::new(value.into_v2()?))
            }
            Self::ExtMethodResponse(value) => {
                super::AgentResponse::ExtMethodResponse(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpResponse(value) => {
                super::AgentResponse::MessageMcpResponse(Box::new(value.into_v2()?))
            }
        })
    }
}

impl IntoV1 for super::ClientNotification {
    type Output = crate::v1::ClientNotification;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::CancelSessionNotification(value) => {
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
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpNotification(value) => {
                crate::v1::ClientNotification::MessageMcpNotification(value.into_v1()?)
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
                super::ClientNotification::CancelSessionNotification(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidOpenDocumentNotification(value) => {
                super::ClientNotification::DidOpenDocumentNotification(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidChangeDocumentNotification(value) => {
                super::ClientNotification::DidChangeDocumentNotification(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidCloseDocumentNotification(value) => {
                super::ClientNotification::DidCloseDocumentNotification(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidSaveDocumentNotification(value) => {
                super::ClientNotification::DidSaveDocumentNotification(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::DidFocusDocumentNotification(value) => {
                super::ClientNotification::DidFocusDocumentNotification(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::AcceptNesNotification(value) => {
                super::ClientNotification::AcceptNesNotification(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_nes")]
            Self::RejectNesNotification(value) => {
                super::ClientNotification::RejectNesNotification(Box::new(value.into_v2()?))
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            Self::MessageMcpNotification(value) => {
                super::ClientNotification::MessageMcpNotification(Box::new(value.into_v2()?))
            }
            Self::ExtNotification(value) => {
                super::ClientNotification::ExtNotification(Box::new(value.into_v2()?))
            }
        })
    }
}

impl IntoV1 for super::CancelSessionNotification {
    type Output = crate::v1::CancelNotification;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { session_id, meta } = self;
        Ok(crate::v1::CancelNotification {
            session_id: session_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::CancelNotification {
    type Output = super::CancelSessionNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { session_id, meta } = self;
        Ok(super::CancelSessionNotification {
            session_id: session_id.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self {
            line,
            character,
            meta,
        } = self;
        Ok(crate::v1::Position {
            line: line.into_v1()?,
            character: character.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::Position {
    type Output = super::Position;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            line,
            character,
            meta,
        } = self;
        Ok(super::Position {
            line: line.into_v2()?,
            character: character.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::Range {
    type Output = crate::v1::Range;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { start, end, meta } = self;
        Ok(crate::v1::Range {
            start: start.into_v1()?,
            end: end.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::Range {
    type Output = super::Range;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { start, end, meta } = self;
        Ok(super::Range {
            start: start.into_v2()?,
            end: end.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesCapabilities {
    type Output = crate::v1::NesCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            events,
            context,
            meta,
        } = self;
        Ok(crate::v1::NesCapabilities {
            events: into_v1_default_on_error(events),
            context: into_v1_default_on_error(context),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesCapabilities {
    type Output = super::NesCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            events,
            context,
            meta,
        } = self;
        Ok(super::NesCapabilities {
            events: into_v2_default_on_error(events),
            context: into_v2_default_on_error(context),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesEventCapabilities {
    type Output = crate::v1::NesEventCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { document, meta } = self;
        Ok(crate::v1::NesEventCapabilities {
            document: into_v1_default_on_error(document),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesEventCapabilities {
    type Output = super::NesEventCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { document, meta } = self;
        Ok(super::NesEventCapabilities {
            document: into_v2_default_on_error(document),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentEventCapabilities {
    type Output = crate::v1::NesDocumentEventCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            did_open,
            did_change,
            did_close,
            did_save,
            did_focus,
            meta,
        } = self;
        Ok(crate::v1::NesDocumentEventCapabilities {
            did_open: into_v1_default_on_error(did_open),
            did_change: into_v1_default_on_error(did_change),
            did_close: into_v1_default_on_error(did_close),
            did_save: into_v1_default_on_error(did_save),
            did_focus: into_v1_default_on_error(did_focus),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentEventCapabilities {
    type Output = super::NesDocumentEventCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            did_open,
            did_change,
            did_close,
            did_save,
            did_focus,
            meta,
        } = self;
        Ok(super::NesDocumentEventCapabilities {
            did_open: into_v2_default_on_error(did_open),
            did_change: into_v2_default_on_error(did_change),
            did_close: into_v2_default_on_error(did_close),
            did_save: into_v2_default_on_error(did_save),
            did_focus: into_v2_default_on_error(did_focus),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentDidOpenCapabilities {
    type Output = crate::v1::NesDocumentDidOpenCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::NesDocumentDidOpenCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentDidOpenCapabilities {
    type Output = super::NesDocumentDidOpenCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::NesDocumentDidOpenCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentDidChangeCapabilities {
    type Output = crate::v1::NesDocumentDidChangeCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { sync_kind, meta } = self;
        Ok(crate::v1::NesDocumentDidChangeCapabilities {
            sync_kind: sync_kind.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentDidChangeCapabilities {
    type Output = super::NesDocumentDidChangeCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { sync_kind, meta } = self;
        Ok(super::NesDocumentDidChangeCapabilities {
            sync_kind: sync_kind.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self { meta } = self;
        Ok(crate::v1::NesDocumentDidCloseCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentDidCloseCapabilities {
    type Output = super::NesDocumentDidCloseCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::NesDocumentDidCloseCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentDidSaveCapabilities {
    type Output = crate::v1::NesDocumentDidSaveCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::NesDocumentDidSaveCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentDidSaveCapabilities {
    type Output = super::NesDocumentDidSaveCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::NesDocumentDidSaveCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDocumentDidFocusCapabilities {
    type Output = crate::v1::NesDocumentDidFocusCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::NesDocumentDidFocusCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDocumentDidFocusCapabilities {
    type Output = super::NesDocumentDidFocusCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::NesDocumentDidFocusCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesContextCapabilities {
    type Output = crate::v1::NesContextCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            recent_files,
            related_snippets,
            edit_history,
            user_actions,
            open_files,
            diagnostics,
            meta,
        } = self;
        Ok(crate::v1::NesContextCapabilities {
            recent_files: into_v1_default_on_error(recent_files),
            related_snippets: into_v1_default_on_error(related_snippets),
            edit_history: into_v1_default_on_error(edit_history),
            user_actions: into_v1_default_on_error(user_actions),
            open_files: into_v1_default_on_error(open_files),
            diagnostics: into_v1_default_on_error(diagnostics),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesContextCapabilities {
    type Output = super::NesContextCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            recent_files,
            related_snippets,
            edit_history,
            user_actions,
            open_files,
            diagnostics,
            meta,
        } = self;
        Ok(super::NesContextCapabilities {
            recent_files: into_v2_default_on_error(recent_files),
            related_snippets: into_v2_default_on_error(related_snippets),
            edit_history: into_v2_default_on_error(edit_history),
            user_actions: into_v2_default_on_error(user_actions),
            open_files: into_v2_default_on_error(open_files),
            diagnostics: into_v2_default_on_error(diagnostics),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRecentFilesCapabilities {
    type Output = crate::v1::NesRecentFilesCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { max_count, meta } = self;
        Ok(crate::v1::NesRecentFilesCapabilities {
            max_count: max_count.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRecentFilesCapabilities {
    type Output = super::NesRecentFilesCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { max_count, meta } = self;
        Ok(super::NesRecentFilesCapabilities {
            max_count: max_count.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRelatedSnippetsCapabilities {
    type Output = crate::v1::NesRelatedSnippetsCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::NesRelatedSnippetsCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRelatedSnippetsCapabilities {
    type Output = super::NesRelatedSnippetsCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::NesRelatedSnippetsCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesEditHistoryCapabilities {
    type Output = crate::v1::NesEditHistoryCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { max_count, meta } = self;
        Ok(crate::v1::NesEditHistoryCapabilities {
            max_count: max_count.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesEditHistoryCapabilities {
    type Output = super::NesEditHistoryCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { max_count, meta } = self;
        Ok(super::NesEditHistoryCapabilities {
            max_count: max_count.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesUserActionsCapabilities {
    type Output = crate::v1::NesUserActionsCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { max_count, meta } = self;
        Ok(crate::v1::NesUserActionsCapabilities {
            max_count: max_count.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesUserActionsCapabilities {
    type Output = super::NesUserActionsCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { max_count, meta } = self;
        Ok(super::NesUserActionsCapabilities {
            max_count: max_count.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesOpenFilesCapabilities {
    type Output = crate::v1::NesOpenFilesCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::NesOpenFilesCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesOpenFilesCapabilities {
    type Output = super::NesOpenFilesCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::NesOpenFilesCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDiagnosticsCapabilities {
    type Output = crate::v1::NesDiagnosticsCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::NesDiagnosticsCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDiagnosticsCapabilities {
    type Output = super::NesDiagnosticsCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::NesDiagnosticsCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::ClientNesCapabilities {
    type Output = crate::v1::ClientNesCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            jump,
            rename,
            search_and_replace,
            meta,
        } = self;
        Ok(crate::v1::ClientNesCapabilities {
            jump: into_v1_default_on_error(jump),
            rename: into_v1_default_on_error(rename),
            search_and_replace: into_v1_default_on_error(search_and_replace),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::ClientNesCapabilities {
    type Output = super::ClientNesCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            jump,
            rename,
            search_and_replace,
            meta,
        } = self;
        Ok(super::ClientNesCapabilities {
            jump: into_v2_default_on_error(jump),
            rename: into_v2_default_on_error(rename),
            search_and_replace: into_v2_default_on_error(search_and_replace),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesJumpCapabilities {
    type Output = crate::v1::NesJumpCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::NesJumpCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesJumpCapabilities {
    type Output = super::NesJumpCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::NesJumpCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRenameCapabilities {
    type Output = crate::v1::NesRenameCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::NesRenameCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRenameCapabilities {
    type Output = super::NesRenameCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::NesRenameCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesSearchAndReplaceCapabilities {
    type Output = crate::v1::NesSearchAndReplaceCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::NesSearchAndReplaceCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesSearchAndReplaceCapabilities {
    type Output = super::NesSearchAndReplaceCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::NesSearchAndReplaceCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::DidOpenDocumentNotification {
    type Output = crate::v1::DidOpenDocumentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            language_id,
            version,
            text,
            meta,
        } = self;
        Ok(crate::v1::DidOpenDocumentNotification {
            session_id: session_id.into_v1()?,
            uri: uri.into_v1()?,
            language_id: language_id.into_v1()?,
            version: version.into_v1()?,
            text: text.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::DidOpenDocumentNotification {
    type Output = super::DidOpenDocumentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            language_id,
            version,
            text,
            meta,
        } = self;
        Ok(super::DidOpenDocumentNotification {
            session_id: session_id.into_v2()?,
            uri: uri.into_v2()?,
            language_id: language_id.into_v2()?,
            version: version.into_v2()?,
            text: text.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::DidChangeDocumentNotification {
    type Output = crate::v1::DidChangeDocumentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            version,
            content_changes,
            meta,
        } = self;
        Ok(crate::v1::DidChangeDocumentNotification {
            session_id: session_id.into_v1()?,
            uri: uri.into_v1()?,
            version: version.into_v1()?,
            content_changes: content_changes.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::DidChangeDocumentNotification {
    type Output = super::DidChangeDocumentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            version,
            content_changes,
            meta,
        } = self;
        Ok(super::DidChangeDocumentNotification {
            session_id: session_id.into_v2()?,
            uri: uri.into_v2()?,
            version: version.into_v2()?,
            content_changes: content_changes.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::TextDocumentContentChangeEvent {
    type Output = crate::v1::TextDocumentContentChangeEvent;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { range, text, meta } = self;
        Ok(crate::v1::TextDocumentContentChangeEvent {
            range: range.into_v1()?,
            text: text.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::TextDocumentContentChangeEvent {
    type Output = super::TextDocumentContentChangeEvent;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { range, text, meta } = self;
        Ok(super::TextDocumentContentChangeEvent {
            range: range.into_v2()?,
            text: text.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::DidCloseDocumentNotification {
    type Output = crate::v1::DidCloseDocumentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            meta,
        } = self;
        Ok(crate::v1::DidCloseDocumentNotification {
            session_id: session_id.into_v1()?,
            uri: uri.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::DidCloseDocumentNotification {
    type Output = super::DidCloseDocumentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            meta,
        } = self;
        Ok(super::DidCloseDocumentNotification {
            session_id: session_id.into_v2()?,
            uri: uri.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::DidSaveDocumentNotification {
    type Output = crate::v1::DidSaveDocumentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            meta,
        } = self;
        Ok(crate::v1::DidSaveDocumentNotification {
            session_id: session_id.into_v1()?,
            uri: uri.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::DidSaveDocumentNotification {
    type Output = super::DidSaveDocumentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            meta,
        } = self;
        Ok(super::DidSaveDocumentNotification {
            session_id: session_id.into_v2()?,
            uri: uri.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::DidFocusDocumentNotification {
    type Output = crate::v1::DidFocusDocumentNotification;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            version,
            position,
            visible_range,
            meta,
        } = self;
        Ok(crate::v1::DidFocusDocumentNotification {
            session_id: session_id.into_v1()?,
            uri: uri.into_v1()?,
            version: version.into_v1()?,
            position: position.into_v1()?,
            visible_range: visible_range.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::DidFocusDocumentNotification {
    type Output = super::DidFocusDocumentNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            version,
            position,
            visible_range,
            meta,
        } = self;
        Ok(super::DidFocusDocumentNotification {
            session_id: session_id.into_v2()?,
            uri: uri.into_v2()?,
            version: version.into_v2()?,
            position: position.into_v2()?,
            visible_range: visible_range.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::StartNesRequest {
    type Output = crate::v1::StartNesRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            workspace_uri,
            workspace_folders,
            repository,
            meta,
        } = self;
        Ok(crate::v1::StartNesRequest {
            workspace_uri: workspace_uri.into_v1()?,
            workspace_folders: option_vec_into_v1_skip_errors(workspace_folders),
            repository: into_v1_default_on_error(repository),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::StartNesRequest {
    type Output = super::StartNesRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            workspace_uri,
            workspace_folders,
            repository,
            meta,
        } = self;
        Ok(super::StartNesRequest {
            workspace_uri: workspace_uri.into_v2()?,
            workspace_folders: option_vec_into_v2_skip_errors(workspace_folders),
            repository: into_v2_default_on_error(repository),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::WorkspaceFolder {
    type Output = crate::v1::WorkspaceFolder;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { uri, name, meta } = self;
        Ok(crate::v1::WorkspaceFolder {
            uri: uri.into_v1()?,
            name: name.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::WorkspaceFolder {
    type Output = super::WorkspaceFolder;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { uri, name, meta } = self;
        Ok(super::WorkspaceFolder {
            uri: uri.into_v2()?,
            name: name.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRepository {
    type Output = crate::v1::NesRepository;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            name,
            owner,
            remote_url,
            meta,
        } = self;
        Ok(crate::v1::NesRepository {
            name: name.into_v1()?,
            owner: owner.into_v1()?,
            remote_url: remote_url.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRepository {
    type Output = super::NesRepository;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            name,
            owner,
            remote_url,
            meta,
        } = self;
        Ok(super::NesRepository {
            name: name.into_v2()?,
            owner: owner.into_v2()?,
            remote_url: remote_url.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::StartNesResponse {
    type Output = crate::v1::StartNesResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { session_id, meta } = self;
        Ok(crate::v1::StartNesResponse {
            session_id: session_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::StartNesResponse {
    type Output = super::StartNesResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { session_id, meta } = self;
        Ok(super::StartNesResponse {
            session_id: session_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::CloseNesRequest {
    type Output = crate::v1::CloseNesRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { session_id, meta } = self;
        Ok(crate::v1::CloseNesRequest {
            session_id: session_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::CloseNesRequest {
    type Output = super::CloseNesRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { session_id, meta } = self;
        Ok(super::CloseNesRequest {
            session_id: session_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::CloseNesResponse {
    type Output = crate::v1::CloseNesResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::CloseNesResponse {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::CloseNesResponse {
    type Output = super::CloseNesResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::CloseNesResponse {
            meta: meta.into_v2()?,
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
            Self::Other(value) => return Err(unknown_v2_enum_variant("NesTriggerKind", &value)),
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
        let Self {
            session_id,
            uri,
            version,
            position,
            selection,
            trigger_kind,
            context,
            meta,
        } = self;
        Ok(crate::v1::SuggestNesRequest {
            session_id: session_id.into_v1()?,
            uri: uri.into_v1()?,
            version: version.into_v1()?,
            position: position.into_v1()?,
            selection: into_v1_default_on_error(selection),
            trigger_kind: trigger_kind.into_v1()?,
            context: into_v1_default_on_error(context),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::SuggestNesRequest {
    type Output = super::SuggestNesRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            uri,
            version,
            position,
            selection,
            trigger_kind,
            context,
            meta,
        } = self;
        Ok(super::SuggestNesRequest {
            session_id: session_id.into_v2()?,
            uri: uri.into_v2()?,
            version: version.into_v2()?,
            position: position.into_v2()?,
            selection: into_v2_default_on_error(selection),
            trigger_kind: trigger_kind.into_v2()?,
            context: into_v2_default_on_error(context),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesSuggestContext {
    type Output = crate::v1::NesSuggestContext;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            recent_files,
            related_snippets,
            edit_history,
            user_actions,
            open_files,
            diagnostics,
            meta,
        } = self;
        Ok(crate::v1::NesSuggestContext {
            recent_files: option_vec_into_v1_skip_errors(recent_files),
            related_snippets: option_vec_into_v1_skip_errors(related_snippets),
            edit_history: option_vec_into_v1_skip_errors(edit_history),
            user_actions: option_vec_into_v1_skip_errors(user_actions),
            open_files: option_vec_into_v1_skip_errors(open_files),
            diagnostics: option_vec_into_v1_skip_errors(diagnostics),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesSuggestContext {
    type Output = super::NesSuggestContext;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            recent_files,
            related_snippets,
            edit_history,
            user_actions,
            open_files,
            diagnostics,
            meta,
        } = self;
        Ok(super::NesSuggestContext {
            recent_files: option_vec_into_v2_skip_errors(recent_files),
            related_snippets: option_vec_into_v2_skip_errors(related_snippets),
            edit_history: option_vec_into_v2_skip_errors(edit_history),
            user_actions: option_vec_into_v2_skip_errors(user_actions),
            open_files: option_vec_into_v2_skip_errors(open_files),
            diagnostics: option_vec_into_v2_skip_errors(diagnostics),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRecentFile {
    type Output = crate::v1::NesRecentFile;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            uri,
            language_id,
            text,
            meta,
        } = self;
        Ok(crate::v1::NesRecentFile {
            uri: uri.into_v1()?,
            language_id: language_id.into_v1()?,
            text: text.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRecentFile {
    type Output = super::NesRecentFile;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            uri,
            language_id,
            text,
            meta,
        } = self;
        Ok(super::NesRecentFile {
            uri: uri.into_v2()?,
            language_id: language_id.into_v2()?,
            text: text.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRelatedSnippet {
    type Output = crate::v1::NesRelatedSnippet;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            uri,
            excerpts,
            meta,
        } = self;
        Ok(crate::v1::NesRelatedSnippet {
            uri: uri.into_v1()?,
            excerpts: excerpts.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRelatedSnippet {
    type Output = super::NesRelatedSnippet;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            uri,
            excerpts,
            meta,
        } = self;
        Ok(super::NesRelatedSnippet {
            uri: uri.into_v2()?,
            excerpts: excerpts.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesExcerpt {
    type Output = crate::v1::NesExcerpt;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            start_line,
            end_line,
            text,
            meta,
        } = self;
        Ok(crate::v1::NesExcerpt {
            start_line: start_line.into_v1()?,
            end_line: end_line.into_v1()?,
            text: text.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesExcerpt {
    type Output = super::NesExcerpt;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            start_line,
            end_line,
            text,
            meta,
        } = self;
        Ok(super::NesExcerpt {
            start_line: start_line.into_v2()?,
            end_line: end_line.into_v2()?,
            text: text.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesEditHistoryEntry {
    type Output = crate::v1::NesEditHistoryEntry;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { uri, diff, meta } = self;
        Ok(crate::v1::NesEditHistoryEntry {
            uri: uri.into_v1()?,
            diff: diff.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesEditHistoryEntry {
    type Output = super::NesEditHistoryEntry;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { uri, diff, meta } = self;
        Ok(super::NesEditHistoryEntry {
            uri: uri.into_v2()?,
            diff: diff.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesUserAction {
    type Output = crate::v1::NesUserAction;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            action,
            uri,
            position,
            timestamp_ms,
            meta,
        } = self;
        Ok(crate::v1::NesUserAction {
            action: action.into_v1()?,
            uri: uri.into_v1()?,
            position: position.into_v1()?,
            timestamp_ms: timestamp_ms.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesUserAction {
    type Output = super::NesUserAction;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            action,
            uri,
            position,
            timestamp_ms,
            meta,
        } = self;
        Ok(super::NesUserAction {
            action: action.into_v2()?,
            uri: uri.into_v2()?,
            position: position.into_v2()?,
            timestamp_ms: timestamp_ms.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesOpenFile {
    type Output = crate::v1::NesOpenFile;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            uri,
            language_id,
            visible_range,
            last_focused_ms,
            meta,
        } = self;
        Ok(crate::v1::NesOpenFile {
            uri: uri.into_v1()?,
            language_id: language_id.into_v1()?,
            visible_range: into_v1_default_on_error(visible_range),
            last_focused_ms: into_v1_default_on_error(last_focused_ms),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesOpenFile {
    type Output = super::NesOpenFile;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            uri,
            language_id,
            visible_range,
            last_focused_ms,
            meta,
        } = self;
        Ok(super::NesOpenFile {
            uri: uri.into_v2()?,
            language_id: language_id.into_v2()?,
            visible_range: into_v2_default_on_error(visible_range),
            last_focused_ms: into_v2_default_on_error(last_focused_ms),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesDiagnostic {
    type Output = crate::v1::NesDiagnostic;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            uri,
            range,
            severity,
            message,
            meta,
        } = self;
        Ok(crate::v1::NesDiagnostic {
            uri: uri.into_v1()?,
            range: range.into_v1()?,
            severity: severity.into_v1()?,
            message: message.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesDiagnostic {
    type Output = super::NesDiagnostic;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            uri,
            range,
            severity,
            message,
            meta,
        } = self;
        Ok(super::NesDiagnostic {
            uri: uri.into_v2()?,
            range: range.into_v2()?,
            severity: severity.into_v2()?,
            message: message.into_v2()?,
            meta: meta.into_v2()?,
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
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant("NesDiagnosticSeverity", &value));
            }
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
        let Self { suggestions, meta } = self;
        Ok(crate::v1::SuggestNesResponse {
            suggestions: into_v1_vec_skip_errors(suggestions),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::SuggestNesResponse {
    type Output = super::SuggestNesResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { suggestions, meta } = self;
        Ok(super::SuggestNesResponse {
            suggestions: into_v2_vec_skip_errors(suggestions),
            meta: meta.into_v2()?,
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
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant("NesSuggestion", &value.kind));
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
        let Self {
            id,
            uri,
            edits,
            cursor_position,
            meta,
        } = self;
        Ok(crate::v1::NesEditSuggestion {
            id: id.into_v1()?,
            uri: uri.into_v1()?,
            edits: edits.into_v1()?,
            cursor_position: into_v1_default_on_error(cursor_position),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesEditSuggestion {
    type Output = super::NesEditSuggestion;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            id,
            uri,
            edits,
            cursor_position,
            meta,
        } = self;
        Ok(super::NesEditSuggestion {
            id: id.into_v2()?,
            uri: uri.into_v2()?,
            edits: edits.into_v2()?,
            cursor_position: into_v2_default_on_error(cursor_position),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesTextEdit {
    type Output = crate::v1::NesTextEdit;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            range,
            new_text,
            meta,
        } = self;
        Ok(crate::v1::NesTextEdit {
            range: range.into_v1()?,
            new_text: new_text.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesTextEdit {
    type Output = super::NesTextEdit;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            range,
            new_text,
            meta,
        } = self;
        Ok(super::NesTextEdit {
            range: range.into_v2()?,
            new_text: new_text.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesJumpSuggestion {
    type Output = crate::v1::NesJumpSuggestion;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            id,
            uri,
            position,
            meta,
        } = self;
        Ok(crate::v1::NesJumpSuggestion {
            id: id.into_v1()?,
            uri: uri.into_v1()?,
            position: position.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesJumpSuggestion {
    type Output = super::NesJumpSuggestion;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            id,
            uri,
            position,
            meta,
        } = self;
        Ok(super::NesJumpSuggestion {
            id: id.into_v2()?,
            uri: uri.into_v2()?,
            position: position.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesRenameSuggestion {
    type Output = crate::v1::NesRenameSuggestion;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            id,
            uri,
            position,
            new_name,
            meta,
        } = self;
        Ok(crate::v1::NesRenameSuggestion {
            id: id.into_v1()?,
            uri: uri.into_v1()?,
            position: position.into_v1()?,
            new_name: new_name.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesRenameSuggestion {
    type Output = super::NesRenameSuggestion;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            id,
            uri,
            position,
            new_name,
            meta,
        } = self;
        Ok(super::NesRenameSuggestion {
            id: id.into_v2()?,
            uri: uri.into_v2()?,
            position: position.into_v2()?,
            new_name: new_name.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::NesSearchAndReplaceSuggestion {
    type Output = crate::v1::NesSearchAndReplaceSuggestion;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            id,
            uri,
            search,
            replace,
            is_regex,
            meta,
        } = self;
        Ok(crate::v1::NesSearchAndReplaceSuggestion {
            id: id.into_v1()?,
            uri: uri.into_v1()?,
            search: search.into_v1()?,
            replace: replace.into_v1()?,
            is_regex: is_regex.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::NesSearchAndReplaceSuggestion {
    type Output = super::NesSearchAndReplaceSuggestion;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            id,
            uri,
            search,
            replace,
            is_regex,
            meta,
        } = self;
        Ok(super::NesSearchAndReplaceSuggestion {
            id: id.into_v2()?,
            uri: uri.into_v2()?,
            search: search.into_v2()?,
            replace: replace.into_v2()?,
            is_regex: is_regex.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::AcceptNesNotification {
    type Output = crate::v1::AcceptNesNotification;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            id,
            meta,
        } = self;
        Ok(crate::v1::AcceptNesNotification {
            session_id: session_id.into_v1()?,
            id: id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::AcceptNesNotification {
    type Output = super::AcceptNesNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            id,
            meta,
        } = self;
        Ok(super::AcceptNesNotification {
            session_id: session_id.into_v2()?,
            id: id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV1 for super::RejectNesNotification {
    type Output = crate::v1::RejectNesNotification;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            session_id,
            id,
            reason,
            meta,
        } = self;
        Ok(crate::v1::RejectNesNotification {
            session_id: session_id.into_v1()?,
            id: id.into_v1()?,
            reason: into_v1_default_on_error(reason),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_nes")]
impl IntoV2 for crate::v1::RejectNesNotification {
    type Output = super::RejectNesNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            id,
            reason,
            meta,
        } = self;
        Ok(super::RejectNesNotification {
            session_id: session_id.into_v2()?,
            id: id.into_v2()?,
            reason: into_v2_default_on_error(reason),
            meta: meta.into_v2()?,
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
            Self::Other(value) => return Err(unknown_v2_enum_variant("NesRejectReason", &value)),
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
            Self::Other(value) => return Err(unknown_v2_enum_variant("StringFormat", &value)),
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
        let Self { value, title, meta } = self;
        Ok(crate::v1::EnumOption {
            value: value.into_v1()?,
            title: title.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::EnumOption {
    type Output = super::EnumOption;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { value, title, meta } = self;
        Ok(super::EnumOption {
            value: value.into_v2()?,
            title: title.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::StringPropertySchema {
    type Output = crate::v1::StringPropertySchema;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            title,
            description,
            min_length,
            max_length,
            pattern,
            format,
            default,
            enum_values,
            one_of,
            meta,
        } = self;
        Ok(crate::v1::StringPropertySchema {
            title: title.into_v1()?,
            description: description.into_v1()?,
            min_length: min_length.into_v1()?,
            max_length: max_length.into_v1()?,
            pattern: pattern.into_v1()?,
            format: format.into_v1()?,
            default: default.into_v1()?,
            enum_values: enum_values.into_v1()?,
            one_of: one_of.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::StringPropertySchema {
    type Output = super::StringPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            title,
            description,
            min_length,
            max_length,
            pattern,
            format,
            default,
            enum_values,
            one_of,
            meta,
        } = self;
        Ok(super::StringPropertySchema {
            title: title.into_v2()?,
            description: description.into_v2()?,
            min_length: min_length.into_v2()?,
            max_length: max_length.into_v2()?,
            pattern: pattern.into_v2()?,
            format: format.into_v2()?,
            default: default.into_v2()?,
            enum_values: enum_values.into_v2()?,
            one_of: one_of.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::NumberPropertySchema {
    type Output = crate::v1::NumberPropertySchema;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            title,
            description,
            minimum,
            maximum,
            default,
            meta,
        } = self;
        Ok(crate::v1::NumberPropertySchema {
            title: title.into_v1()?,
            description: description.into_v1()?,
            minimum: minimum.into_v1()?,
            maximum: maximum.into_v1()?,
            default: default.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::NumberPropertySchema {
    type Output = super::NumberPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            title,
            description,
            minimum,
            maximum,
            default,
            meta,
        } = self;
        Ok(super::NumberPropertySchema {
            title: title.into_v2()?,
            description: description.into_v2()?,
            minimum: minimum.into_v2()?,
            maximum: maximum.into_v2()?,
            default: default.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::IntegerPropertySchema {
    type Output = crate::v1::IntegerPropertySchema;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            title,
            description,
            minimum,
            maximum,
            default,
            meta,
        } = self;
        Ok(crate::v1::IntegerPropertySchema {
            title: title.into_v1()?,
            description: description.into_v1()?,
            minimum: minimum.into_v1()?,
            maximum: maximum.into_v1()?,
            default: default.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::IntegerPropertySchema {
    type Output = super::IntegerPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            title,
            description,
            minimum,
            maximum,
            default,
            meta,
        } = self;
        Ok(super::IntegerPropertySchema {
            title: title.into_v2()?,
            description: description.into_v2()?,
            minimum: minimum.into_v2()?,
            maximum: maximum.into_v2()?,
            default: default.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::BooleanPropertySchema {
    type Output = crate::v1::BooleanPropertySchema;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            title,
            description,
            default,
            meta,
        } = self;
        Ok(crate::v1::BooleanPropertySchema {
            title: title.into_v1()?,
            description: description.into_v1()?,
            default: default.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::BooleanPropertySchema {
    type Output = super::BooleanPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            title,
            description,
            default,
            meta,
        } = self;
        Ok(super::BooleanPropertySchema {
            title: title.into_v2()?,
            description: description.into_v2()?,
            default: default.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self {
            type_,
            values,
            meta,
        } = self;
        Ok(crate::v1::UntitledMultiSelectItems {
            type_: type_.into_v1()?,
            values: values.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::UntitledMultiSelectItems {
    type Output = super::UntitledMultiSelectItems;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            type_,
            values,
            meta,
        } = self;
        Ok(super::UntitledMultiSelectItems {
            type_: type_.into_v2()?,
            values: values.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::TitledMultiSelectItems {
    type Output = crate::v1::TitledMultiSelectItems;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { options, meta } = self;
        Ok(crate::v1::TitledMultiSelectItems {
            options: options.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::TitledMultiSelectItems {
    type Output = super::TitledMultiSelectItems;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { options, meta } = self;
        Ok(super::TitledMultiSelectItems {
            options: options.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self {
            title,
            description,
            min_items,
            max_items,
            items,
            default,
            meta,
        } = self;
        Ok(crate::v1::MultiSelectPropertySchema {
            title: title.into_v1()?,
            description: description.into_v1()?,
            min_items: min_items.into_v1()?,
            max_items: max_items.into_v1()?,
            items: items.into_v1()?,
            default: default.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::MultiSelectPropertySchema {
    type Output = super::MultiSelectPropertySchema;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            title,
            description,
            min_items,
            max_items,
            items,
            default,
            meta,
        } = self;
        Ok(super::MultiSelectPropertySchema {
            title: title.into_v2()?,
            description: description.into_v2()?,
            min_items: min_items.into_v2()?,
            max_items: max_items.into_v2()?,
            items: items.into_v2()?,
            default: default.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self {
            type_,
            title,
            properties,
            required,
            description,
            meta,
        } = self;
        Ok(crate::v1::ElicitationSchema {
            type_: type_.into_v1()?,
            title: title.into_v1()?,
            properties: properties.into_v1()?,
            required: required.into_v1()?,
            description: description.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationSchema {
    type Output = super::ElicitationSchema;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            type_,
            title,
            properties,
            required,
            description,
            meta,
        } = self;
        Ok(super::ElicitationSchema {
            type_: type_.into_v2()?,
            title: title.into_v2()?,
            properties: properties.into_v2()?,
            required: required.into_v2()?,
            description: description.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationCapabilities {
    type Output = crate::v1::ElicitationCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { form, url, meta } = self;
        Ok(crate::v1::ElicitationCapabilities {
            form: into_v1_default_on_error(form),
            url: into_v1_default_on_error(url),
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationCapabilities {
    type Output = super::ElicitationCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { form, url, meta } = self;
        Ok(super::ElicitationCapabilities {
            form: into_v2_default_on_error(form),
            url: into_v2_default_on_error(url),
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationFormCapabilities {
    type Output = crate::v1::ElicitationFormCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::ElicitationFormCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationFormCapabilities {
    type Output = super::ElicitationFormCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::ElicitationFormCapabilities {
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationUrlCapabilities {
    type Output = crate::v1::ElicitationUrlCapabilities;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(crate::v1::ElicitationUrlCapabilities {
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationUrlCapabilities {
    type Output = super::ElicitationUrlCapabilities;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { meta } = self;
        Ok(super::ElicitationUrlCapabilities {
            meta: meta.into_v2()?,
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
        let Self {
            session_id,
            tool_call_id,
        } = self;
        Ok(crate::v1::ElicitationSessionScope {
            session_id: session_id.into_v1()?,
            tool_call_id: tool_call_id.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationSessionScope {
    type Output = super::ElicitationSessionScope;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            session_id,
            tool_call_id,
        } = self;
        Ok(super::ElicitationSessionScope {
            session_id: session_id.into_v2()?,
            tool_call_id: tool_call_id.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationRequestScope {
    type Output = crate::v1::ElicitationRequestScope;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { request_id } = self;
        Ok(crate::v1::ElicitationRequestScope {
            request_id: request_id.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationRequestScope {
    type Output = super::ElicitationRequestScope;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { request_id } = self;
        Ok(super::ElicitationRequestScope {
            request_id: request_id.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::CreateElicitationRequest {
    type Output = crate::v1::CreateElicitationRequest;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            mode,
            message,
            meta,
        } = self;
        Ok(crate::v1::CreateElicitationRequest {
            mode: mode.into_v1()?,
            message: message.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::CreateElicitationRequest {
    type Output = super::CreateElicitationRequest;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            mode,
            message,
            meta,
        } = self;
        Ok(super::CreateElicitationRequest {
            mode: mode.into_v2()?,
            message: message.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self {
            scope,
            requested_schema,
        } = self;
        Ok(crate::v1::ElicitationFormMode {
            scope: scope.into_v1()?,
            requested_schema: requested_schema.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationFormMode {
    type Output = super::ElicitationFormMode;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            scope,
            requested_schema,
        } = self;
        Ok(super::ElicitationFormMode {
            scope: scope.into_v2()?,
            requested_schema: requested_schema.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::ElicitationUrlMode {
    type Output = crate::v1::ElicitationUrlMode;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            scope,
            elicitation_id,
            url,
        } = self;
        Ok(crate::v1::ElicitationUrlMode {
            scope: scope.into_v1()?,
            elicitation_id: elicitation_id.into_v1()?,
            url: url.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationUrlMode {
    type Output = super::ElicitationUrlMode;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            scope,
            elicitation_id,
            url,
        } = self;
        Ok(super::ElicitationUrlMode {
            scope: scope.into_v2()?,
            elicitation_id: elicitation_id.into_v2()?,
            url: url.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::CreateElicitationResponse {
    type Output = crate::v1::CreateElicitationResponse;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { action, meta } = self;
        Ok(crate::v1::CreateElicitationResponse {
            action: action.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::CreateElicitationResponse {
    type Output = super::CreateElicitationResponse;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { action, meta } = self;
        Ok(super::CreateElicitationResponse {
            action: action.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self { content } = self;
        Ok(crate::v1::ElicitationAcceptAction {
            content: content.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::ElicitationAcceptAction {
    type Output = super::ElicitationAcceptAction;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { content } = self;
        Ok(super::ElicitationAcceptAction {
            content: content.into_v2()?,
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
        let Self {
            elicitation_id,
            meta,
        } = self;
        Ok(crate::v1::CompleteElicitationNotification {
            elicitation_id: elicitation_id.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::CompleteElicitationNotification {
    type Output = super::CompleteElicitationNotification;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            elicitation_id,
            meta,
        } = self;
        Ok(super::CompleteElicitationNotification {
            elicitation_id: elicitation_id.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::UrlElicitationRequiredData {
    type Output = crate::v1::UrlElicitationRequiredData;

    fn into_v1(self) -> Result<Self::Output> {
        let Self { elicitations } = self;
        Ok(crate::v1::UrlElicitationRequiredData {
            elicitations: elicitations.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::UrlElicitationRequiredData {
    type Output = super::UrlElicitationRequiredData;

    fn into_v2(self) -> Result<Self::Output> {
        let Self { elicitations } = self;
        Ok(super::UrlElicitationRequiredData {
            elicitations: elicitations.into_v2()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV1 for super::UrlElicitationRequiredItem {
    type Output = crate::v1::UrlElicitationRequiredItem;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            mode,
            elicitation_id,
            url,
            message,
        } = self;
        Ok(crate::v1::UrlElicitationRequiredItem {
            mode: mode.into_v1()?,
            elicitation_id: elicitation_id.into_v1()?,
            url: url.into_v1()?,
            message: message.into_v1()?,
        })
    }
}

#[cfg(feature = "unstable_elicitation")]
impl IntoV2 for crate::v1::UrlElicitationRequiredItem {
    type Output = super::UrlElicitationRequiredItem;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            mode,
            elicitation_id,
            url,
            message,
        } = self;
        Ok(super::UrlElicitationRequiredItem {
            mode: mode.into_v2()?,
            elicitation_id: elicitation_id.into_v2()?,
            url: url.into_v2()?,
            message: message.into_v2()?,
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
            Self::Other(value) => {
                return Err(unknown_v2_enum_variant("ContentBlock", &value.type_));
            }
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
        let Self {
            annotations,
            text,
            meta,
        } = self;
        Ok(crate::v1::TextContent {
            annotations: into_v1_default_on_error(annotations),
            text: text.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::TextContent {
    type Output = super::TextContent;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            annotations,
            text,
            meta,
        } = self;
        Ok(super::TextContent {
            annotations: into_v2_default_on_error(annotations),
            text: text.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ImageContent {
    type Output = crate::v1::ImageContent;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            annotations,
            data,
            mime_type,
            uri,
            meta,
        } = self;
        Ok(crate::v1::ImageContent {
            annotations: into_v1_default_on_error(annotations),
            data: data.into_v1()?,
            mime_type: mime_type.into_v1()?,
            uri: uri.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ImageContent {
    type Output = super::ImageContent;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            annotations,
            data,
            mime_type,
            uri,
            meta,
        } = self;
        Ok(super::ImageContent {
            annotations: into_v2_default_on_error(annotations),
            data: data.into_v2()?,
            mime_type: mime_type.into_v2()?,
            uri: uri.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::AudioContent {
    type Output = crate::v1::AudioContent;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            annotations,
            data,
            mime_type,
            meta,
        } = self;
        Ok(crate::v1::AudioContent {
            annotations: into_v1_default_on_error(annotations),
            data: data.into_v1()?,
            mime_type: mime_type.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::AudioContent {
    type Output = super::AudioContent;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            annotations,
            data,
            mime_type,
            meta,
        } = self;
        Ok(super::AudioContent {
            annotations: into_v2_default_on_error(annotations),
            data: data.into_v2()?,
            mime_type: mime_type.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::EmbeddedResource {
    type Output = crate::v1::EmbeddedResource;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            annotations,
            resource,
            meta,
        } = self;
        Ok(crate::v1::EmbeddedResource {
            annotations: into_v1_default_on_error(annotations),
            resource: resource.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::EmbeddedResource {
    type Output = super::EmbeddedResource;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            annotations,
            resource,
            meta,
        } = self;
        Ok(super::EmbeddedResource {
            annotations: into_v2_default_on_error(annotations),
            resource: resource.into_v2()?,
            meta: meta.into_v2()?,
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
        let Self {
            mime_type,
            text,
            uri,
            meta,
        } = self;
        Ok(crate::v1::TextResourceContents {
            mime_type: mime_type.into_v1()?,
            text: text.into_v1()?,
            uri: uri.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::TextResourceContents {
    type Output = super::TextResourceContents;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            mime_type,
            text,
            uri,
            meta,
        } = self;
        Ok(super::TextResourceContents {
            mime_type: mime_type.into_v2()?,
            text: text.into_v2()?,
            uri: uri.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::BlobResourceContents {
    type Output = crate::v1::BlobResourceContents;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            blob,
            mime_type,
            uri,
            meta,
        } = self;
        Ok(crate::v1::BlobResourceContents {
            blob: blob.into_v1()?,
            mime_type: mime_type.into_v1()?,
            uri: uri.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::BlobResourceContents {
    type Output = super::BlobResourceContents;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            blob,
            mime_type,
            uri,
            meta,
        } = self;
        Ok(super::BlobResourceContents {
            blob: blob.into_v2()?,
            mime_type: mime_type.into_v2()?,
            uri: uri.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::ResourceLink {
    type Output = crate::v1::ResourceLink;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            annotations,
            description,
            mime_type,
            name,
            size,
            title,
            uri,
            meta,
        } = self;
        Ok(crate::v1::ResourceLink {
            annotations: into_v1_default_on_error(annotations),
            description: description.into_v1()?,
            mime_type: mime_type.into_v1()?,
            name: name.into_v1()?,
            size: size.into_v1()?,
            title: title.into_v1()?,
            uri: uri.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::ResourceLink {
    type Output = super::ResourceLink;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            annotations,
            description,
            mime_type,
            name,
            size,
            title,
            uri,
            meta,
        } = self;
        Ok(super::ResourceLink {
            annotations: into_v2_default_on_error(annotations),
            description: description.into_v2()?,
            mime_type: mime_type.into_v2()?,
            name: name.into_v2()?,
            size: size.into_v2()?,
            title: title.into_v2()?,
            uri: uri.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::Annotations {
    type Output = crate::v1::Annotations;

    fn into_v1(self) -> Result<Self::Output> {
        let Self {
            audience,
            last_modified,
            priority,
            meta,
        } = self;
        Ok(crate::v1::Annotations {
            audience: option_vec_into_v1_skip_errors(audience),
            last_modified: last_modified.into_v1()?,
            priority: priority.into_v1()?,
            meta: meta.into_v1()?,
        })
    }
}

impl IntoV2 for crate::v1::Annotations {
    type Output = super::Annotations;

    fn into_v2(self) -> Result<Self::Output> {
        let Self {
            audience,
            last_modified,
            priority,
            meta,
        } = self;
        Ok(super::Annotations {
            audience: option_vec_into_v2_skip_errors(audience),
            last_modified: last_modified.into_v2()?,
            priority: priority.into_v2()?,
            meta: meta.into_v2()?,
        })
    }
}

impl IntoV1 for super::Role {
    type Output = crate::v1::Role;

    fn into_v1(self) -> Result<Self::Output> {
        Ok(match self {
            Self::Assistant => crate::v1::Role::Assistant,
            Self::User => crate::v1::Role::User,
            Self::Other(value) => return Err(unknown_v2_enum_variant("Role", &value)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{v1, v2};

    /// Round-trip a v1 value through v2 and back, asserting equality.
    ///
    /// This catches dropped fields, renamed fields, and missing enum
    /// variants in either direction of the conversion as soon as v1 and v2
    /// diverge.
    fn assert_v1_round_trip<T1, T2>(value: T1)
    where
        T1: IntoV2<Output = T2> + Clone + std::fmt::Debug + PartialEq,
        T2: IntoV1<Output = T1>,
    {
        let original = value.clone();
        let as_v2 = v1_to_v2(value).expect("v1 -> v2 conversion failed");
        let back = v2_to_v1(as_v2).expect("v2 -> v1 conversion failed");
        assert_eq!(
            original, back,
            "value did not survive v1 -> v2 -> v1 round trip"
        );
    }

    /// Round-trip a v2 value through v1 and back, asserting equality.
    fn assert_v2_round_trip<T2, T1>(value: T2)
    where
        T2: IntoV1<Output = T1> + Clone + std::fmt::Debug + PartialEq,
        T1: IntoV2<Output = T2>,
    {
        let original = value.clone();
        let as_v1 = v2_to_v1(value).expect("v2 -> v1 conversion failed");
        let back = v1_to_v2(as_v1).expect("v1 -> v2 conversion failed");
        assert_eq!(
            original, back,
            "value did not survive v2 -> v1 -> v2 round trip"
        );
    }

    /// While v1 and v2 are structurally identical, JSON produced by either
    /// module must be byte-equal after a conversion. This is a cheap insurance
    /// against accidental field renames or shape drift in conversions.
    ///
    /// Starts from a v1 value, converts forward to v2 and asserts JSON
    /// equality, then converts back to v1 and asserts JSON equality again.
    /// Catches shape drift in both directions of the conversion.
    fn assert_json_eq_after_v1_to_v2<T1, T2>(value: T1)
    where
        T1: IntoV2<Output = T2> + serde::Serialize + Clone,
        T2: IntoV1<Output = T1> + serde::Serialize,
    {
        let v1_json = serde_json::to_value(&value).expect("v1 serialize");
        let as_v2 = v1_to_v2(value).expect("v1 -> v2 conversion");
        let v2_json = serde_json::to_value(&as_v2).expect("v2 serialize");
        assert_eq!(
            v1_json, v2_json,
            "JSON shape diverged after v1 -> v2 conversion"
        );

        let back_to_v1 = v2_to_v1(as_v2).expect("v2 -> v1 conversion");
        let v1_json_after =
            serde_json::to_value(&back_to_v1).expect("v1 serialize after round trip");
        assert_eq!(
            v2_json, v1_json_after,
            "JSON shape diverged after v2 -> v1 conversion"
        );
    }

    /// Mirror of [`assert_json_eq_after_v1_to_v2`] that starts from a v2 value.
    /// Useful when the natural starting point is a v2 type (for example when
    /// the type only exists today in v2's draft API surface).
    fn assert_json_eq_after_v2_to_v1<T2, T1>(value: T2)
    where
        T2: IntoV1<Output = T1> + serde::Serialize + Clone,
        T1: IntoV2<Output = T2> + serde::Serialize,
    {
        let v2_json = serde_json::to_value(&value).expect("v2 serialize");
        let as_v1 = v2_to_v1(value).expect("v2 -> v1 conversion");
        let v1_json = serde_json::to_value(&as_v1).expect("v1 serialize");
        assert_eq!(
            v2_json, v1_json,
            "JSON shape diverged after v2 -> v1 conversion"
        );

        let back_to_v2 = v1_to_v2(as_v1).expect("v1 -> v2 conversion");
        let v2_json_after =
            serde_json::to_value(&back_to_v2).expect("v2 serialize after round trip");
        assert_eq!(
            v1_json, v2_json_after,
            "JSON shape diverged after v1 -> v2 conversion"
        );
    }

    fn assert_v2_to_v1_error<T>(value: T, expected: &str)
    where
        T: IntoV1,
        T::Output: std::fmt::Debug,
    {
        let error = v2_to_v1(value).unwrap_err();
        assert_eq!(error.message(), expected);
    }

    fn assert_v2_to_v1_many_error<T>(value: T, expected: &str)
    where
        T: IntoV1Many,
        T::Output: std::fmt::Debug,
    {
        let error = v2_to_v1_many(value).unwrap_err();
        assert_eq!(error.message(), expected);
    }

    fn assert_v1_to_v2_error<T>(value: T, expected: &str)
    where
        T: IntoV2,
        T::Output: std::fmt::Debug,
    {
        let error = v1_to_v2(value).unwrap_err();
        assert_eq!(error.message(), expected);
    }

    #[test]
    fn round_trips_session_config_option_categories() {
        for category in [
            v1::SessionConfigOptionCategory::Mode,
            v1::SessionConfigOptionCategory::Model,
            v1::SessionConfigOptionCategory::ModelConfig,
            v1::SessionConfigOptionCategory::ThoughtLevel,
            v1::SessionConfigOptionCategory::Other("_custom_category".to_string()),
        ] {
            assert_v1_round_trip::<v1::SessionConfigOptionCategory, v2::SessionConfigOptionCategory>(
                category,
            );
        }

        for category in [
            v2::SessionConfigOptionCategory::Mode,
            v2::SessionConfigOptionCategory::Model,
            v2::SessionConfigOptionCategory::ModelConfig,
            v2::SessionConfigOptionCategory::ThoughtLevel,
            v2::SessionConfigOptionCategory::Other("_custom_category".to_string()),
        ] {
            assert_v2_round_trip::<v2::SessionConfigOptionCategory, v1::SessionConfigOptionCategory>(
                category,
            );
        }
    }

    #[test]
    fn converts_v2_initialize_request_to_v1_without_serde() {
        let request = v2::InitializeRequest::new(
            ProtocolVersion::V2,
            v2::Implementation::new("test-client", "1.0.0"),
        );

        let converted: v1::InitializeRequest = v2_to_v1(request).unwrap();

        assert_eq!(converted.protocol_version, ProtocolVersion::V2);
        assert_eq!(
            converted
                .client_info
                .as_ref()
                .map(|info| info.name.as_str()),
            Some("test-client")
        );
    }

    #[test]
    fn v1_initialize_request_without_client_info_does_not_convert_to_v2() {
        let request = v1::InitializeRequest::new(ProtocolVersion::V1);

        assert_v1_to_v2_error(
            request,
            "v1 InitializeRequest without `clientInfo` cannot be represented in v2",
        );
    }

    #[test]
    fn v1_initialize_response_without_agent_info_does_not_convert_to_v2() {
        let response = v1::InitializeResponse::new(ProtocolVersion::V1);

        assert_v1_to_v2_error(
            response,
            "v1 InitializeResponse without `agentInfo` cannot be represented in v2",
        );
    }

    #[test]
    fn round_trips_initialize_request() {
        let mut client_capabilities = v1::ClientCapabilities::new();
        #[cfg(feature = "unstable_boolean_config")]
        {
            client_capabilities = client_capabilities.session(
                v1::ClientSessionCapabilities::new().config_options(
                    v1::SessionConfigOptionsCapabilities::new()
                        .boolean(v1::BooleanConfigOptionCapabilities::new()),
                ),
            );
        }

        let request = v1::InitializeRequest::new(ProtocolVersion::V1)
            .client_capabilities(client_capabilities)
            .client_info(v1::Implementation::new("test-client", "1.0.0").title("Test Client"));

        assert_v1_round_trip::<v1::InitializeRequest, v2::InitializeRequest>(request.clone());
        let converted: v2::InitializeRequest =
            v1_to_v2(request).expect("v1 -> v2 conversion failed");
        let converted_capabilities =
            serde_json::to_value(&converted.capabilities).expect("v2 serialize");
        assert_eq!(converted_capabilities.get("fs"), None);
        assert_eq!(converted_capabilities.get("terminal"), None);
        let converted_json = serde_json::to_value(&converted).expect("v2 serialize");
        assert_eq!(converted_json.get("clientInfo"), None);
        assert_eq!(converted_json.get("implementation"), None);
        assert!(converted_json.get("info").is_some());
    }

    #[test]
    fn round_trips_initialize_response() {
        let response = v1::InitializeResponse::new(ProtocolVersion::V1)
            .agent_capabilities(
                v1::AgentCapabilities::new()
                    .auth(v1::AgentAuthCapabilities::new().logout(v1::LogoutCapabilities::new())),
            )
            .agent_info(v1::Implementation::new("test-agent", "2.0.0").title("Test Agent"));
        assert_v1_round_trip::<v1::InitializeResponse, v2::InitializeResponse>(response.clone());
        let converted: v2::InitializeResponse =
            v1_to_v2(response).expect("v1 -> v2 conversion failed");
        let converted_json = serde_json::to_value(&converted).expect("v2 serialize");
        assert_eq!(converted_json.get("agentCapabilities"), None);
        assert!(converted_json.get("capabilities").is_some());
        assert_eq!(converted_json.get("agentInfo"), None);
        assert_eq!(converted_json.get("implementation"), None);
        assert!(converted_json.get("info").is_some());
        assert_eq!(converted_json.pointer("/capabilities/loadSession"), None);
    }

    #[test]
    fn agent_load_session_capability_moves_between_v1_and_v2() {
        let v1_capabilities = v1::AgentCapabilities::new().load_session(true);

        let v2_capabilities: v2::AgentCapabilities =
            v1_to_v2(v1_capabilities).expect("v1 -> v2 conversion");
        let session = v2_capabilities
            .session
            .as_ref()
            .expect("v1 capabilities imply v2 session support");
        assert!(session.load.is_some());
        let v2_json = serde_json::to_value(&v2_capabilities).expect("v2 serialize");
        assert_eq!(v2_json.get("loadSession"), None);
        assert_eq!(
            v2_json.pointer("/session/load"),
            Some(&serde_json::json!({}))
        );

        let v1_after: v1::AgentCapabilities =
            v2_to_v1(v2_capabilities).expect("v2 -> v1 conversion");
        assert!(v1_after.load_session);
    }

    #[test]
    fn v2_agent_capabilities_without_session_do_not_convert_to_v1() {
        let error = v2::AgentCapabilities::new().into_v1().unwrap_err();
        assert_eq!(
            error.message(),
            "v2 AgentCapabilities without `session` cannot be represented in v1"
        );
    }

    #[test]
    fn v2_auth_logout_is_baseline_not_capability_marker() {
        let v2_auth = v2::AgentAuthCapabilities::new();
        let v2_json = serde_json::to_value(&v2_auth).expect("v2 serialize");
        assert_eq!(v2_json.get("logout"), None);

        let v1_auth: v1::AgentAuthCapabilities = v2_to_v1(v2_auth).expect("v2 -> v1 conversion");
        assert!(v1_auth.logout.is_some());

        let v1_auth_without_logout = v1::AgentAuthCapabilities::new();
        let v2_auth: v2::AgentAuthCapabilities =
            v1_to_v2(v1_auth_without_logout).expect("v1 -> v2 conversion");
        let v2_json = serde_json::to_value(&v2_auth).expect("v2 serialize");
        assert_eq!(v2_json.get("logout"), None);
    }

    #[test]
    fn v2_session_capabilities_convert_to_v1_agent_capability_parts() {
        let parts = v2::SessionCapabilities::new()
            .load(v2::SessionLoadCapabilities::new())
            .prompt(v2::PromptCapabilities::new().image(v2::PromptImageCapabilities::new()))
            .mcp(v2::McpCapabilities::new().http(v2::McpHttpCapabilities::new()))
            .list(v2::SessionListCapabilities::new())
            .into_v1()
            .expect("v2 session capabilities -> v1 parts");

        assert!(parts.session_capabilities.list.is_some());
        assert!(parts.prompt_capabilities.image);
        assert!(parts.load_session);
        assert!(parts.mcp_capabilities.http);
    }

    #[test]
    fn v1_prompt_capability_bools_convert_to_v2_objects() {
        let v1_capabilities = v1::PromptCapabilities::new()
            .image(true)
            .audio(true)
            .embedded_context(true);

        let v2_capabilities: v2::PromptCapabilities =
            v1_to_v2(v1_capabilities).expect("v1 -> v2 conversion");
        let v2_json = serde_json::to_value(&v2_capabilities).expect("v2 serialize");
        assert_eq!(v2_json.pointer("/image"), Some(&serde_json::json!({})));
        assert_eq!(v2_json.pointer("/audio"), Some(&serde_json::json!({})));
        assert_eq!(
            v2_json.pointer("/embeddedContext"),
            Some(&serde_json::json!({}))
        );

        let v1_after: v1::PromptCapabilities =
            v2_to_v1(v2_capabilities).expect("v2 -> v1 conversion");
        assert!(v1_after.image);
        assert!(v1_after.audio);
        assert!(v1_after.embedded_context);
    }

    #[test]
    fn v1_mcp_capabilities_convert_to_v2_transport_objects() {
        let v1_capabilities = v1::McpCapabilities::new().http(true).sse(true);

        let v2_capabilities: v2::McpCapabilities =
            v1_to_v2(v1_capabilities).expect("v1 -> v2 conversion");
        let v2_json = serde_json::to_value(&v2_capabilities).expect("v2 serialize");
        assert_eq!(v2_json.pointer("/stdio"), Some(&serde_json::json!({})));
        assert_eq!(v2_json.pointer("/http"), Some(&serde_json::json!({})));
        assert_eq!(v2_json.pointer("/sse"), None);

        let v1_after: v1::McpCapabilities = v2_to_v1(v2_capabilities).expect("v2 -> v1 conversion");
        assert!(v1_after.http);
        assert!(!v1_after.sse);
    }

    #[cfg(feature = "unstable_mcp_over_acp")]
    #[test]
    fn v1_mcp_acp_capability_bool_converts_to_v2_object() {
        let v1_capabilities = v1::McpCapabilities::new().acp(true);

        let v2_capabilities: v2::McpCapabilities =
            v1_to_v2(v1_capabilities).expect("v1 -> v2 conversion");
        let v2_json = serde_json::to_value(&v2_capabilities).expect("v2 serialize");
        assert_eq!(v2_json.pointer("/acp"), Some(&serde_json::json!({})));

        let v1_after: v1::McpCapabilities = v2_to_v1(v2_capabilities).expect("v2 -> v1 conversion");
        assert!(v1_after.acp);
    }

    #[cfg(feature = "unstable_auth_methods")]
    #[test]
    fn v1_auth_terminal_capability_bool_converts_to_v2_object() {
        let v1_capabilities = v1::AuthCapabilities::new().terminal(true);

        let v2_capabilities: v2::AuthCapabilities =
            v1_to_v2(v1_capabilities).expect("v1 -> v2 conversion");
        let v2_json = serde_json::to_value(&v2_capabilities).expect("v2 serialize");
        assert_eq!(v2_json.pointer("/terminal"), Some(&serde_json::json!({})));

        let v1_after: v1::AuthCapabilities =
            v2_to_v1(v2_capabilities).expect("v2 -> v1 conversion");
        assert!(v1_after.terminal);
    }

    #[cfg(feature = "unstable_auth_methods")]
    #[test]
    fn auth_method_terminal_env_converts_between_map_and_variable_array() {
        let mut env = HashMap::new();
        env.insert("TERM".to_string(), "xterm-256color".to_string());
        env.insert("API_KEY".to_string(), "secret".to_string());

        let v1_method = v1::AuthMethodTerminal::new("tui-auth", "Terminal Auth").env(env);
        let v2_method: v2::AuthMethodTerminal = v1_to_v2(v1_method).expect("v1 -> v2 conversion");
        let v2_json = serde_json::to_value(&v2_method).expect("v2 serialize");
        assert_eq!(
            v2_json.pointer("/env"),
            Some(&serde_json::json!([
                {
                    "name": "API_KEY",
                    "value": "secret"
                },
                {
                    "name": "TERM",
                    "value": "xterm-256color"
                }
            ]))
        );

        let v1_after: v1::AuthMethodTerminal = v2_to_v1(v2_method).expect("v2 -> v1 conversion");
        assert_eq!(
            v1_after.env.get("TERM").map(String::as_str),
            Some("xterm-256color")
        );
        assert_eq!(
            v1_after.env.get("API_KEY").map(String::as_str),
            Some("secret")
        );
    }

    #[cfg(feature = "unstable_auth_methods")]
    #[test]
    fn auth_method_terminal_duplicate_env_names_do_not_convert_to_v1() {
        let v2_method = v2::AuthMethodTerminal::new("tui-auth", "Terminal Auth").env(vec![
            v2::EnvVariable::new("TERM", "xterm"),
            v2::EnvVariable::new("TERM", "xterm-256color"),
        ]);

        assert_v2_to_v1_error(
            v2_method,
            "v2 AuthMethodTerminal env variable `TERM` is duplicated and cannot be represented in v1",
        );
    }

    #[test]
    fn v1_client_fs_and_terminal_capabilities_are_removed_in_v2() {
        let v1_capabilities =
            v1::ClientCapabilities::new()
                .terminal(true)
                .fs(v1::FileSystemCapabilities::new()
                    .read_text_file(true)
                    .write_text_file(true));

        let v2_capabilities: v2::ClientCapabilities =
            v1_to_v2(v1_capabilities).expect("v1 -> v2 conversion");
        let v2_json = serde_json::to_value(&v2_capabilities).expect("v2 serialize");
        assert_eq!(
            v2_json.get("fs"),
            None,
            "v2 ClientCapabilities must not include filesystem capabilities"
        );
        assert_eq!(
            v2_json.get("terminal"),
            None,
            "v2 ClientCapabilities must not include terminal capabilities"
        );

        let v1_after: v1::ClientCapabilities =
            v2_to_v1(v2_capabilities).expect("v2 -> v1 conversion");
        assert!(!v1_after.fs.read_text_file);
        assert!(!v1_after.fs.write_text_file);
        assert!(!v1_after.terminal);
    }

    #[cfg(feature = "unstable_boolean_config")]
    #[test]
    fn v2_client_capabilities_default_to_v1_boolean_config_option_support() {
        let v2_capabilities = v2::ClientCapabilities::new();

        let v1_capabilities: v1::ClientCapabilities =
            v2_to_v1(v2_capabilities).expect("v2 -> v1 conversion");

        assert!(
            v1_capabilities
                .session
                .and_then(|session| session.config_options)
                .and_then(|config_options| config_options.boolean)
                .is_some()
        );
    }

    #[test]
    fn v1_client_fs_and_terminal_methods_do_not_convert_to_v2() {
        assert_v1_to_v2_error(
            v1::AgentRequest::WriteTextFileRequest(v1::WriteTextFileRequest::new(
                "sess",
                "/workspace/file.txt",
                "contents",
            )),
            "v1 AgentRequest variant `fs/write_text_file` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::AgentRequest::ReadTextFileRequest(v1::ReadTextFileRequest::new(
                "sess",
                "/workspace/file.txt",
            )),
            "v1 AgentRequest variant `fs/read_text_file` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::AgentRequest::CreateTerminalRequest(v1::CreateTerminalRequest::new("sess", "echo")),
            "v1 AgentRequest variant `terminal/create` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::AgentRequest::TerminalOutputRequest(v1::TerminalOutputRequest::new("sess", "term")),
            "v1 AgentRequest variant `terminal/output` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::AgentRequest::ReleaseTerminalRequest(v1::ReleaseTerminalRequest::new(
                "sess", "term",
            )),
            "v1 AgentRequest variant `terminal/release` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::AgentRequest::WaitForTerminalExitRequest(v1::WaitForTerminalExitRequest::new(
                "sess", "term",
            )),
            "v1 AgentRequest variant `terminal/wait_for_exit` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::AgentRequest::KillTerminalRequest(v1::KillTerminalRequest::new("sess", "term")),
            "v1 AgentRequest variant `terminal/kill` cannot be represented in v2",
        );

        assert_v1_to_v2_error(
            v1::ClientResponse::WriteTextFileResponse(v1::WriteTextFileResponse::new()),
            "v1 ClientResponse variant `fs/write_text_file` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::ClientResponse::ReadTextFileResponse(v1::ReadTextFileResponse::new("contents")),
            "v1 ClientResponse variant `fs/read_text_file` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::ClientResponse::CreateTerminalResponse(v1::CreateTerminalResponse::new("term")),
            "v1 ClientResponse variant `terminal/create` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::ClientResponse::TerminalOutputResponse(v1::TerminalOutputResponse::new("", false)),
            "v1 ClientResponse variant `terminal/output` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::ClientResponse::ReleaseTerminalResponse(v1::ReleaseTerminalResponse::new()),
            "v1 ClientResponse variant `terminal/release` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::ClientResponse::WaitForTerminalExitResponse(v1::WaitForTerminalExitResponse::new(
                v1::TerminalExitStatus::new(),
            )),
            "v1 ClientResponse variant `terminal/wait_for_exit` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::ClientResponse::KillTerminalResponse(v1::KillTerminalResponse::new()),
            "v1 ClientResponse variant `terminal/kill` cannot be represented in v2",
        );
    }

    #[test]
    fn v1_terminal_tool_call_content_does_not_convert_to_v2() {
        assert_v1_to_v2_error(
            v1::ToolCallContent::Terminal(v1::Terminal::new("term")),
            "v1 ToolCallContent variant `terminal` cannot be represented in v2",
        );
    }

    #[test]
    fn v1_mcp_sse_transport_does_not_convert_to_v2() {
        assert_v1_to_v2_error(
            v1::McpServer::Sse(v1::McpServerSse::new("events", "https://example.com/sse")),
            "v1 McpServer variant `sse` cannot be represented in v2",
        );
    }

    #[test]
    fn v2_unknown_mcp_transport_does_not_convert_to_v1() {
        assert_v2_to_v1_error(
            v2::McpServer::Other(v2::OtherMcpServer::new("websocket", BTreeMap::default())),
            "v2 McpServer variant `websocket` cannot be represented in v1",
        );
    }

    #[test]
    fn round_trips_new_session_request_with_mcp_variants() {
        let request = v1::NewSessionRequest::new("/workspace").mcp_servers(vec![
            v1::McpServer::Stdio(v1::McpServerStdio::new("local", "/usr/bin/mcp")),
            v1::McpServer::Http(v1::McpServerHttp::new("remote", "https://example.com")),
        ]);

        assert_v1_round_trip::<v1::NewSessionRequest, v2::NewSessionRequest>(request.clone());

        let v2_request: v2::NewSessionRequest = v1_to_v2(request).expect("v1 -> v2 conversion");
        assert_eq!(
            serde_json::to_value(&v2_request).expect("v2 serialize"),
            serde_json::json!({
                "cwd": "/workspace",
                "mcpServers": [
                    {
                        "type": "stdio",
                        "name": "local",
                        "command": "/usr/bin/mcp",
                        "args": [],
                        "env": []
                    },
                    {
                        "type": "http",
                        "name": "remote",
                        "url": "https://example.com",
                        "headers": []
                    }
                ]
            })
        );
    }

    #[test]
    fn round_trips_prompt_request_with_content_variants() {
        let prompt = vec![
            v1::ContentBlock::Text(v1::TextContent::new("hello")),
            v1::ContentBlock::Image(v1::ImageContent::new("data", "image/png")),
            v1::ContentBlock::ResourceLink(v1::ResourceLink::new("file.txt", "file:///file.txt")),
        ];
        let request = v1::PromptRequest::new("sess_1", prompt);
        assert_v1_round_trip::<v1::PromptRequest, v2::PromptRequest>(request.clone());
        assert_json_eq_after_v1_to_v2::<v1::PromptRequest, v2::PromptRequest>(request);
    }

    #[test]
    fn prompt_responses_do_not_convert_across_v1_v2_lifecycle_boundary() {
        assert_v2_to_v1_error(
            v2::PromptResponse::new(),
            "v2 PromptResponse cannot be represented in v1 because v2 reports completion with state_update session updates",
        );
        assert_v1_to_v2_error(
            v1::PromptResponse::new(v1::StopReason::EndTurn),
            "v1 PromptResponse cannot be represented in v2 by itself because v2 reports completion with state_update session updates",
        );
    }

    #[test]
    fn v1_tool_call_converts_to_v2_upsert_with_diff_and_locations() {
        let tool_call = v1::ToolCall::new("tc_1", "editing files")
            .kind(v1::ToolKind::Edit)
            .status(v1::ToolCallStatus::InProgress)
            .content(vec![v1::ToolCallContent::Diff(
                v1::Diff::new("/path", "new contents").old_text("old contents"),
            )])
            .locations(vec![v1::ToolCallLocation::new("/path").line(42)])
            .raw_input(serde_json::json!({"foo": "bar"}))
            .raw_output(serde_json::json!({"ok": true}));

        let converted: v2::ToolCallUpdate = v1_to_v2(tool_call).expect("v1 -> v2 conversion");
        assert_eq!(
            serde_json::to_value(&converted).expect("v2 serialize"),
            serde_json::json!({
                "toolCallId": "tc_1",
                "title": "editing files",
                "kind": "edit",
                "status": "in_progress",
                "content": [
                    {
                        "type": "diff",
                        "path": "/path",
                        "oldText": "old contents",
                        "newText": "new contents"
                    }
                ],
                "locations": [
                    {
                        "path": "/path",
                        "line": 42
                    }
                ],
                "rawInput": {
                    "foo": "bar"
                },
                "rawOutput": {
                    "ok": true
                }
            })
        );

        let back: v1::ToolCallUpdate = v2_to_v1(converted).expect("v2 -> v1 conversion");
        assert_eq!(back.tool_call_id.0.as_ref(), "tc_1");
        assert_eq!(back.fields.title.as_deref(), Some("editing files"));
        assert_eq!(back.fields.kind, Some(v1::ToolKind::Edit));
        assert_eq!(back.fields.status, Some(v1::ToolCallStatus::InProgress));
        assert_eq!(back.fields.content.as_ref().map(Vec::len), Some(1));
        assert_eq!(back.fields.locations.as_ref().map(Vec::len), Some(1));
        assert_eq!(
            back.fields.raw_input,
            Some(serde_json::json!({"foo": "bar"}))
        );
        assert_eq!(
            back.fields.raw_output,
            Some(serde_json::json!({"ok": true}))
        );
    }

    #[test]
    fn v1_tool_call_update_round_trips_through_v2_tool_call_update_upsert() {
        let update = v1::ToolCallUpdate::new(
            "tc",
            v1::ToolCallUpdateFields::new()
                .status(v1::ToolCallStatus::Completed)
                .content(Vec::new()),
        );

        assert_v1_round_trip::<v1::ToolCallUpdate, v2::ToolCallUpdate>(update.clone());
        assert_json_eq_after_v1_to_v2::<v1::ToolCallUpdate, v2::ToolCallUpdate>(update);
    }

    #[test]
    fn round_trips_session_notification_for_unchanged_update_kinds() {
        fn content_chunk(text: &str, message_id: &str) -> v1::ContentChunk {
            let chunk = v1::ContentChunk::new(v1::ContentBlock::Text(v1::TextContent::new(text)));
            chunk.message_id(message_id)
        }

        let cases: Vec<v1::SessionUpdate> = vec![
            v1::SessionUpdate::UserMessageChunk(content_chunk("u", "msg_user")),
            v1::SessionUpdate::AgentMessageChunk(content_chunk("a", "msg_agent")),
            v1::SessionUpdate::AgentThoughtChunk(content_chunk("t", "msg_thought")),
            #[cfg(feature = "unstable_plan_operations")]
            v1::SessionUpdate::PlanUpdate(v1::PlanUpdate::new(v1::PlanUpdateContent::markdown(
                "plan-1",
                "## Steps\n- [ ] Test conversion",
            ))),
            #[cfg(feature = "unstable_plan_operations")]
            v1::SessionUpdate::PlanRemoved(v1::PlanRemoved::new("plan-1")),
            v1::SessionUpdate::SessionInfoUpdate(v1::SessionInfoUpdate::new().title("hi")),
            v1::SessionUpdate::UsageUpdate(
                v1::UsageUpdate::new(53_000, 200_000).cost(v1::Cost::new(0.045, "USD")),
            ),
        ];
        for update in cases {
            let notification = v1::SessionNotification::new("sess", update);
            let original_json = serde_json::to_value(&notification).expect("v1 serialize");
            let as_v2: v2::UpdateSessionNotification =
                v1_to_v2(notification.clone()).expect("v1 -> v2 conversion");
            let v2_json = serde_json::to_value(&as_v2).expect("v2 serialize");
            assert_eq!(
                original_json, v2_json,
                "JSON shape diverged after v1 -> v2 conversion"
            );

            let back = v2_to_v1_many(as_v2).expect("v2 -> v1 conversion");
            assert_eq!(back, vec![notification]);
            let back_json = serde_json::to_value(&back[0]).expect("v1 serialize after round trip");
            assert_eq!(
                original_json, back_json,
                "JSON shape diverged after v2 -> v1 conversion"
            );
        }
    }

    #[test]
    fn v1_tool_call_session_updates_convert_to_unified_v2_tool_call_update() {
        let create = v1::SessionNotification::new(
            "sess",
            v1::SessionUpdate::ToolCall(v1::ToolCall::new("tc", "title")),
        );
        let create_v2: v2::UpdateSessionNotification =
            v1_to_v2(create).expect("v1 -> v2 conversion");
        assert!(matches!(
            create_v2.update,
            v2::SessionUpdate::ToolCallUpdate(_)
        ));
        assert_eq!(
            serde_json::to_value(&create_v2).expect("v2 serialize"),
            serde_json::json!({
                "sessionId": "sess",
                "update": {
                    "sessionUpdate": "tool_call_update",
                    "toolCallId": "tc",
                    "title": "title"
                }
            })
        );

        let update = v1::SessionNotification::new(
            "sess",
            v1::SessionUpdate::ToolCallUpdate(v1::ToolCallUpdate::new(
                "tc",
                v1::ToolCallUpdateFields::new().status(v1::ToolCallStatus::Completed),
            )),
        );
        let update_v2: v2::UpdateSessionNotification =
            v1_to_v2(update).expect("v1 -> v2 conversion");
        assert!(matches!(
            update_v2.update,
            v2::SessionUpdate::ToolCallUpdate(_)
        ));
        assert_eq!(
            serde_json::to_value(&update_v2).expect("v2 serialize"),
            serde_json::json!({
                "sessionId": "sess",
                "update": {
                    "sessionUpdate": "tool_call_update",
                    "toolCallId": "tc",
                    "status": "completed"
                }
            })
        );
    }

    #[test]
    fn v2_full_messages_convert_to_v1_message_chunks() {
        let mut meta = v2::Meta::new();
        meta.insert("source".to_string(), serde_json::json!("full"));

        let chunks = v2_to_v1_many(v2::SessionUpdate::UserMessage(
            v2::UserMessage::new("msg_user")
                .content(vec![
                    v2::ContentBlock::Text(v2::TextContent::new("hello")),
                    v2::ContentBlock::Text(v2::TextContent::new("world")),
                ])
                .meta(meta.clone()),
        ))
        .expect("v2 -> v1 conversion");
        assert_eq!(
            chunks,
            vec![
                v1::SessionUpdate::UserMessageChunk(
                    v1::ContentChunk::new(v1::ContentBlock::Text(v1::TextContent::new("hello")))
                        .message_id("msg_user")
                        .meta(meta.clone())
                ),
                v1::SessionUpdate::UserMessageChunk(
                    v1::ContentChunk::new(v1::ContentBlock::Text(v1::TextContent::new("world")))
                        .message_id("msg_user")
                        .meta(meta)
                ),
            ]
        );

        assert_eq!(
            v2_to_v1_many(v2::SessionUpdate::AgentMessage(
                v2::AgentMessage::new("msg_agent")
                    .content(vec![v2::ContentBlock::Text(v2::TextContent::new("hello"))])
            ))
            .expect("v2 -> v1 conversion"),
            vec![v1::SessionUpdate::AgentMessageChunk(
                v1::ContentChunk::new(v1::ContentBlock::Text(v1::TextContent::new("hello")))
                    .message_id("msg_agent")
            )]
        );

        assert_eq!(
            v2_to_v1_many(v2::SessionUpdate::AgentThought(
                v2::AgentThought::new("msg_thought").content(vec![v2::ContentBlock::Text(
                    v2::TextContent::new("thinking")
                )])
            ))
            .expect("v2 -> v1 conversion"),
            vec![v1::SessionUpdate::AgentThoughtChunk(
                v1::ContentChunk::new(v1::ContentBlock::Text(v1::TextContent::new("thinking")))
                    .message_id("msg_thought")
            )]
        );
    }

    #[test]
    fn v2_full_message_session_notification_fans_out_to_v1_chunk_notifications() {
        let notification = v2::UpdateSessionNotification::new(
            "sess",
            v2::SessionUpdate::AgentMessage(v2::AgentMessage::new("msg_agent").content(vec![
                v2::ContentBlock::Text(v2::TextContent::new("hello")),
                v2::ContentBlock::Text(v2::TextContent::new("world")),
            ])),
        );

        let notifications = v2_to_v1_many(notification).expect("v2 -> v1 conversion");
        assert_eq!(
            notifications,
            vec![
                v1::SessionNotification::new(
                    "sess",
                    v1::SessionUpdate::AgentMessageChunk(
                        v1::ContentChunk::new(v1::ContentBlock::Text(v1::TextContent::new(
                            "hello"
                        )))
                        .message_id("msg_agent")
                    )
                ),
                v1::SessionNotification::new(
                    "sess",
                    v1::SessionUpdate::AgentMessageChunk(
                        v1::ContentChunk::new(v1::ContentBlock::Text(v1::TextContent::new(
                            "world"
                        )))
                        .message_id("msg_agent")
                    )
                ),
            ]
        );
    }

    #[test]
    fn v2_json_rpc_agent_notification_fans_out_to_v1_chunk_notifications() {
        let message = v2::JsonRpcMessage::wrap(v2::Notification {
            method: "session/update".into(),
            params: Some(v2::AgentNotification::UpdateSessionNotification(Box::new(
                v2::UpdateSessionNotification::new(
                    "sess",
                    v2::SessionUpdate::AgentMessage(v2::AgentMessage::new("msg_agent").content(
                        vec![
                            v2::ContentBlock::Text(v2::TextContent::new("hello")),
                            v2::ContentBlock::Text(v2::TextContent::new("world")),
                        ],
                    )),
                ),
            ))),
        });

        let messages = v2_to_v1_many(message).expect("v2 -> v1 conversion");
        let json = messages
            .into_iter()
            .map(|message| serde_json::to_value(message).expect("serialize v1 message"))
            .collect::<Vec<_>>();
        assert_eq!(
            json,
            vec![
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "session/update",
                    "params": {
                        "sessionId": "sess",
                        "update": {
                            "sessionUpdate": "agent_message_chunk",
                            "content": {
                                "type": "text",
                                "text": "hello"
                            },
                            "messageId": "msg_agent"
                        }
                    }
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "session/update",
                    "params": {
                        "sessionId": "sess",
                        "update": {
                            "sessionUpdate": "agent_message_chunk",
                            "content": {
                                "type": "text",
                                "text": "world"
                            },
                            "messageId": "msg_agent"
                        }
                    }
                }),
            ]
        );
    }

    #[test]
    fn v2_message_patches_and_clears_do_not_convert_to_v1_chunks() {
        assert_v2_to_v1_many_error(
            v2::SessionUpdate::AgentMessage(v2::AgentMessage::new("msg_agent")),
            "v2 SessionUpdate variant `agent_message` without content cannot be represented in v1 chunks",
        );

        assert_v2_to_v1_many_error(
            v2::SessionUpdate::AgentMessage(
                v2::AgentMessage::new("msg_agent").content(None::<Vec<v2::ContentBlock>>),
            ),
            "v2 SessionUpdate variant `agent_message` with null content cannot be represented in v1 chunks",
        );

        assert_v2_to_v1_many_error(
            v2::SessionUpdate::AgentMessage(
                v2::AgentMessage::new("msg_agent").content(Vec::<v2::ContentBlock>::new()),
            ),
            "v2 SessionUpdate variant `agent_message` with empty content cannot be represented in v1 chunks",
        );

        assert_v2_to_v1_many_error(
            v2::SessionUpdate::AgentMessage(
                v2::AgentMessage::new("msg_agent")
                    .content(vec![v2::ContentBlock::Text(v2::TextContent::new("hello"))])
                    .meta(None::<v2::Meta>),
            ),
            "v2 SessionUpdate variant `agent_message` with null _meta cannot be represented in v1 chunks",
        );
    }

    #[test]
    fn v2_tool_call_content_chunk_does_not_convert_to_v1_replacement_update() {
        assert_v2_to_v1_many_error(
            v2::SessionUpdate::ToolCallContentChunk(v2::ToolCallContentChunk::new(
                "tc_1",
                v2::ContentBlock::Text(v2::TextContent::new("partial output")),
            )),
            "v2 SessionUpdate variant `tool_call_content_chunk` cannot be represented in v1 because v1 tool-call content updates replace content instead of appending",
        );
    }

    #[test]
    fn v1_content_chunk_without_message_id_does_not_convert_to_v2() {
        assert_v1_to_v2_error(
            v1::ContentChunk::new(v1::ContentBlock::Text(v1::TextContent::new("missing"))),
            "v1 ContentChunk without messageId cannot be represented in v2",
        );
    }

    #[test]
    fn v1_plan_session_update_converts_to_v2_item_plan_update() {
        let update = v1::SessionUpdate::Plan(v1::Plan::new(vec![v1::PlanEntry::new(
            "step",
            v1::PlanEntryPriority::High,
            v1::PlanEntryStatus::InProgress,
        )]));

        let as_v2: v2::SessionUpdate = v1_to_v2(update.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&as_v2).unwrap(),
            serde_json::json!({
                "sessionUpdate": "plan_update",
                "plan": {
                    "type": "items",
                    "id": LEGACY_V1_PLAN_ID,
                    "entries": [
                        {
                            "content": "step",
                            "priority": "high",
                            "status": "in_progress"
                        }
                    ]
                }
            })
        );

        let back = v2_to_v1_many(as_v2).unwrap();
        #[cfg(not(feature = "unstable_plan_operations"))]
        assert_eq!(back, vec![update]);
        #[cfg(feature = "unstable_plan_operations")]
        assert!(matches!(
            back.as_slice(),
            [v1::SessionUpdate::PlanUpdate(_)]
        ));
    }

    #[test]
    fn unknown_v2_session_update_does_not_convert_to_v1() {
        let update = v2::SessionUpdate::Other(v2::OtherSessionUpdate::new(
            "_status_badge",
            std::collections::BTreeMap::new(),
        ));

        assert_v2_to_v1_many_error(
            update,
            "v2 SessionUpdate variant `_status_badge` cannot be represented in v1",
        );
    }

    #[test]
    fn v2_state_update_does_not_convert_to_v1() {
        assert_v2_to_v1_many_error(
            v2::SessionUpdate::StateUpdate(v2::StateUpdate::Idle(
                v2::IdleStateUpdate::new().stop_reason(v2::StopReason::EndTurn),
            )),
            "v2 SessionUpdate variant `state_update` cannot be represented in v1 because v1 reports completion in the session/prompt response",
        );
    }

    #[test]
    fn v1_current_mode_update_does_not_convert_to_v2() {
        assert_v1_to_v2_error(
            v1::SessionUpdate::CurrentModeUpdate(v1::CurrentModeUpdate::new("ask")),
            "v1 SessionUpdate variant `current_mode_update` cannot be represented in v2",
        );
    }

    #[test]
    fn v1_session_mode_methods_do_not_convert_to_v2() {
        assert_v1_to_v2_error(
            v1::ClientRequest::SetSessionModeRequest(v1::SetSessionModeRequest::new("sess", "ask")),
            "v1 ClientRequest variant `session/set_mode` cannot be represented in v2",
        );
        assert_v1_to_v2_error(
            v1::AgentResponse::SetSessionModeResponse(v1::SetSessionModeResponse::new()),
            "v1 AgentResponse variant `session/set_mode` cannot be represented in v2",
        );
    }

    #[test]
    fn v1_session_response_modes_fall_back_to_none_in_v2() {
        let response = v1::NewSessionResponse::new("sess").modes(v1::SessionModeState::new(
            "ask",
            vec![v1::SessionMode::new("ask", "Ask")],
        ));

        let as_v2: v2::NewSessionResponse = v1_to_v2(response).unwrap();
        let back_to_v1: v1::NewSessionResponse = v2_to_v1(as_v2).unwrap();

        assert!(back_to_v1.modes.is_none());
    }

    #[test]
    fn v1_session_response_missing_config_options_becomes_empty_v2_vec() {
        let new_response: v2::NewSessionResponse =
            v1_to_v2(v1::NewSessionResponse::new("sess")).unwrap();
        assert!(new_response.config_options.is_empty());

        let load_response: v2::LoadSessionResponse =
            v1_to_v2(v1::LoadSessionResponse::new()).unwrap();
        assert!(load_response.config_options.is_empty());

        let resume_response: v2::ResumeSessionResponse =
            v1_to_v2(v1::ResumeSessionResponse::new()).unwrap();
        assert!(resume_response.config_options.is_empty());

        #[cfg(feature = "unstable_session_fork")]
        {
            let fork_response: v2::ForkSessionResponse =
                v1_to_v2(v1::ForkSessionResponse::new("fork")).unwrap();
            assert!(fork_response.config_options.is_empty());
        }
    }

    #[test]
    fn v2_session_response_converts_to_v1_without_mode_state() {
        let response: v1::NewSessionResponse =
            v2_to_v1(v2::NewSessionResponse::new("sess")).unwrap();

        assert!(response.modes.is_none());
        assert!(matches!(
            response.config_options,
            Some(config_options) if config_options.is_empty()
        ));
    }

    #[test]
    fn v2_tool_call_update_conversion_matches_v1_default_on_error_fields() {
        let update = v2::ToolCallUpdate::new("tc")
            .kind(v2::ToolKind::Unknown("_future_kind".to_string()))
            .status(v2::ToolCallStatus::Other("_paused".to_string()))
            .content(vec![
                v2::ToolCallContent::Other(v2::OtherToolCallContent::new(
                    "_chart",
                    BTreeMap::default(),
                )),
                v2::ToolCallContent::Diff(v2::Diff::new("/tmp/file.txt", "new")),
            ]);

        let converted: v1::ToolCallUpdate = v2_to_v1(update).unwrap();

        assert_eq!(converted.fields.kind, None);
        assert_eq!(converted.fields.status, None);
        assert_eq!(
            converted.fields.content,
            Some(vec![v1::ToolCallContent::Diff(v1::Diff::new(
                "/tmp/file.txt",
                "new"
            ))])
        );
    }

    #[test]
    fn v2_collection_conversion_skips_items_like_v1_vec_skip_error() {
        let response = v2::InitializeResponse::new(
            ProtocolVersion::V2,
            v2::Implementation::new("test-agent", "2.0.0"),
        )
        .capabilities(v2::AgentCapabilities::new().session(v2::SessionCapabilities::new()))
        .auth_methods(vec![
            v2::AuthMethod::Other(v2::OtherAuthMethod::new(
                "_oauth",
                "oauth",
                "OAuth",
                BTreeMap::default(),
            )),
            v2::AuthMethod::Agent(v2::AuthMethodAgent::new("agent", "Agent")),
        ]);
        let converted: v1::InitializeResponse = v2_to_v1(response).unwrap();
        assert_eq!(converted.auth_methods.len(), 1);
        assert!(matches!(
            converted.auth_methods[0],
            v1::AuthMethod::Agent(_)
        ));

        let config_update = v2::ConfigOptionUpdate::new(vec![
            v2::SessionConfigOption::select(
                "mode",
                "Mode",
                "ask",
                vec![v2::SessionConfigSelectOption::new("ask", "Ask")],
            ),
            v2::SessionConfigOption::new(
                "future",
                "Future",
                v2::SessionConfigKind::Other(v2::OtherSessionConfigKind::new(
                    "_slider",
                    BTreeMap::default(),
                )),
            ),
        ]);
        let converted: v1::ConfigOptionUpdate = v2_to_v1(config_update).unwrap();
        assert_eq!(converted.config_options.len(), 1);
        assert_eq!(converted.config_options[0].id.0.as_ref(), "mode");
    }

    #[test]
    fn v2_default_on_error_fields_drop_unrepresentable_nested_values() {
        let command = v2::AvailableCommand::new("review", "Review changes").input(
            v2::AvailableCommandInput::Other(v2::OtherAvailableCommandInput::new(
                "_choices",
                BTreeMap::default(),
            )),
        );
        let converted: v1::AvailableCommandsUpdate =
            v2_to_v1(v2::AvailableCommandsUpdate::new(vec![command])).unwrap();

        assert_eq!(converted.available_commands.len(), 1);
        assert_eq!(converted.available_commands[0].input, None);

        let content = v2::TextContent::new("hello").annotations(
            v2::Annotations::new()
                .audience(vec![v2::Role::Other("_critic".to_string()), v2::Role::User]),
        );
        let converted: v1::TextContent = v2_to_v1(content).unwrap();

        assert_eq!(
            converted
                .annotations
                .and_then(|annotations| annotations.audience),
            Some(vec![v1::Role::User])
        );
    }

    #[test]
    fn v2_plan_entries_skip_unrepresentable_items_inside_tolerant_vectors() {
        let update = v2::PlanUpdate::new(v2::PlanUpdateContent::items(
            "main",
            vec![
                v2::PlanEntry::new(
                    "keep",
                    v2::PlanEntryPriority::High,
                    v2::PlanEntryStatus::Pending,
                ),
                v2::PlanEntry::new(
                    "drop",
                    v2::PlanEntryPriority::Other("_critical".to_string()),
                    v2::PlanEntryStatus::Pending,
                ),
            ],
        ));

        #[cfg(not(feature = "unstable_plan_operations"))]
        {
            let converted: v1::Plan = v2_to_v1(update).unwrap();
            assert_eq!(converted.entries.len(), 1);
            assert_eq!(converted.entries[0].content, "keep");
        }

        #[cfg(feature = "unstable_plan_operations")]
        {
            let converted: v1::PlanUpdate = v2_to_v1(update).unwrap();
            let v1::PlanUpdateContent::Items(items) = converted.plan else {
                panic!("expected item plan update");
            };
            assert_eq!(items.entries.len(), 1);
            assert_eq!(items.entries[0].content, "keep");
        }
    }

    #[test]
    fn v1_tool_call_update_conversion_skips_items_for_v2_vec_skip_error_fields() {
        let update = v1::ToolCallUpdate::new(
            "tc",
            v1::ToolCallUpdateFields::new().content(vec![
                v1::ToolCallContent::Terminal(v1::Terminal::new("term")),
                v1::ToolCallContent::Diff(v1::Diff::new("/tmp/file.txt", "new")),
            ]),
        );

        let converted: v2::ToolCallUpdate = v1_to_v2(update).unwrap();
        assert_eq!(
            converted.content,
            crate::MaybeUndefined::Value(vec![v2::ToolCallContent::Diff(v2::Diff::new(
                "/tmp/file.txt",
                "new"
            ))])
        );
    }

    #[test]
    fn unknown_v2_raw_fallbacks_do_not_convert_to_v1() {
        assert_v2_to_v1_error(
            v2::ContentBlock::Other(v2::OtherContentBlock::new(
                "_widget",
                std::collections::BTreeMap::new(),
            )),
            "v2 ContentBlock variant `_widget` cannot be represented in v1",
        );
        assert_v2_to_v1_error(
            v2::ToolCallContent::Other(v2::OtherToolCallContent::new(
                "_chart",
                std::collections::BTreeMap::new(),
            )),
            "v2 ToolCallContent variant `_chart` cannot be represented in v1",
        );
        assert_v2_to_v1_error(
            v2::AvailableCommandInput::Other(v2::OtherAvailableCommandInput::new(
                "_choices",
                std::collections::BTreeMap::new(),
            )),
            "v2 AvailableCommandInput variant `_choices` cannot be represented in v1",
        );
        assert_v2_to_v1_error(
            v2::SessionConfigKind::Other(v2::OtherSessionConfigKind::new(
                "_slider",
                std::collections::BTreeMap::new(),
            )),
            "v2 SessionConfigKind variant `_slider` cannot be represented in v1",
        );
        assert_v2_to_v1_error(
            v2::AuthMethod::Other(v2::OtherAuthMethod::new(
                "_oauth",
                "oauth",
                "OAuth",
                std::collections::BTreeMap::new(),
            )),
            "v2 AuthMethod variant `_oauth` cannot be represented in v1",
        );
        assert_v2_to_v1_error(
            v2::PlanUpdate::new(v2::PlanUpdateContent::Other(
                v2::OtherPlanUpdateContent::new(
                    "_timeline",
                    "plan-1",
                    std::collections::BTreeMap::new(),
                ),
            )),
            "v2 PlanUpdateContent variant `_timeline` cannot be represented in v1",
        );
        #[cfg(feature = "unstable_nes")]
        assert_v2_to_v1_error(
            v2::NesSuggestion::Other(v2::OtherNesSuggestion::new(
                "_preview",
                std::collections::BTreeMap::new(),
            )),
            "v2 NesSuggestion variant `_preview` cannot be represented in v1",
        );
    }

    #[test]
    fn round_trips_request_permission_outcomes() {
        let cancelled = v1::RequestPermissionResponse::new(v1::RequestPermissionOutcome::Cancelled);
        assert_v1_round_trip::<v1::RequestPermissionResponse, v2::RequestPermissionResponse>(
            cancelled,
        );

        let selected = v1::RequestPermissionResponse::new(v1::RequestPermissionOutcome::Selected(
            v1::SelectedPermissionOutcome::new("opt_1"),
        ));
        assert_v1_round_trip::<v1::RequestPermissionResponse, v2::RequestPermissionResponse>(
            selected,
        );
    }

    #[test]
    fn round_trips_error_with_data_payload() {
        let err = v1::Error::invalid_params().data(serde_json::json!({
            "reason": "missing field",
            "field": "sessionId",
        }));
        assert_v1_round_trip::<v1::Error, v2::Error>(err);
    }

    #[test]
    fn round_trips_v2_value_back_through_v1() {
        // Same coverage but starting from v2, to exercise IntoV1 explicitly.
        let request = v2::PromptRequest::new(
            "sess_2",
            vec![v2::ContentBlock::Text(v2::TextContent::new("hi"))],
        );
        assert_v2_round_trip::<v2::PromptRequest, v1::PromptRequest>(request.clone());
        assert_json_eq_after_v2_to_v1::<v2::PromptRequest, v1::PromptRequest>(request);
    }

    #[test]
    fn protocol_version_constants_remain_explicit() {
        assert_eq!(ProtocolVersion::V1.as_u16(), 1);
        assert_eq!(ProtocolVersion::V2.as_u16(), 2);
    }

    /// `?` bubbles a [`ProtocolConversionError`] into a [`v1::Error`] without
    /// loss of message, mapped onto the internal-error code.
    #[test]
    fn protocol_conversion_error_maps_into_v1_error() {
        fn run() -> std::result::Result<(), v1::Error> {
            // Synthesize a conversion error so we don't have to wait for v2
            // to actually diverge before exercising the `?` path.
            Err(ProtocolConversionError::new("missing required field"))?;
            unreachable!();
        }

        let err = run().unwrap_err();
        assert_eq!(err.code, v1::ErrorCode::InternalError);
        assert_eq!(
            err.data,
            Some(serde_json::Value::String(
                "missing required field".to_string()
            ))
        );
    }

    /// Mirror of the v1 test for the v2 [`Error`] type.
    #[test]
    fn protocol_conversion_error_maps_into_v2_error() {
        fn run() -> std::result::Result<(), v2::Error> {
            Err(ProtocolConversionError::new("missing required field"))?;
            unreachable!();
        }

        let err = run().unwrap_err();
        assert_eq!(err.code, v2::ErrorCode::InternalError);
        assert_eq!(
            err.data,
            Some(serde_json::Value::String(
                "missing required field".to_string()
            ))
        );
    }
}
