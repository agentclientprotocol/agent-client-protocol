//! Extension types and constants for protocol extensibility.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(with = "serde_json::Value")]
#[non_exhaustive]
pub struct ExtRequest {
    #[serde(skip)] // this is used for routing, but when serializing we only want the params
    pub method: Arc<str>,
    pub params: Arc<RawValue>,
}

impl ExtRequest {
    pub fn new(method: impl Into<Arc<str>>, params: Arc<RawValue>) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

pub type ExtResponse = Arc<RawValue>;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(with = "serde_json::Value")]
#[non_exhaustive]
pub struct ExtNotification {
    #[serde(skip)] // this is used for routing, but when serializing we only want the params
    pub method: Arc<str>,
    pub params: Arc<RawValue>,
}

impl ExtNotification {
    pub fn new(method: impl Into<Arc<str>>, params: Arc<RawValue>) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}
