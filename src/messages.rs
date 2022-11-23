use crate::types::SubscriptionId;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_with::skip_serializing_none;
use std::fmt::Debug;

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Advice {
    pub interval: Option<u64>,
    #[serde(rename = "maxInterval")]
    pub max_interval: Option<u64>,
    #[serde(rename = "multiple-clients")]
    pub multiple_clients: Option<bool>,
    pub reconnect: Option<Reconnect>,
    pub timeout: Option<u64>,
    pub hosts: Option<Vec<String>>,
}

impl Advice {
    #[inline(always)]
    pub fn retry(timeout_ms: u64, interval_ms: u64) -> Self {
        Self {
            reconnect: Some(Reconnect::Retry),
            timeout: Some(timeout_ms),
            interval: Some(interval_ms),
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn interval(&self) -> Option<u64> {
        self.interval
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Reconnect {
    Retry,
    Handshake,
    None,
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Message {
    pub advice: Option<Advice>,
    pub channel: Option<String>,
    #[serde(rename = "clientId")]
    pub client_id: Option<String>,
    #[serde(rename = "connectionType")]
    pub connection_type: Option<String>,
    // TODO: Replace on Msg generic?
    pub data: Option<JsonValue>,
    pub error: Option<String>,
    //pub ext: Option<JsonValue>,
    pub id: Option<String>,
    #[serde(rename = "minimumVersion")]
    pub minimum_version: Option<String>,
    //pub reestablish: Option<bool>,
    pub subscription: Option<String>,
    pub successful: Option<bool>,
    #[serde(rename = "supportedConnectionTypes")]
    pub supported_connection_types: Option<Vec<String>>,
    //pub timestamp: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct SubscriptionMessage<Msg> {
    pub(crate) subscription: SubscriptionId,
    pub(crate) msg: Msg,
}

impl Message {
    #[inline]
    pub fn error<Str: Into<String>>(
        message: Str,
        channel: Option<String>,
        client_id: Option<String>,
        id: Option<String>,
    ) -> Self {
        Self {
            advice: Some(Advice {
                reconnect: Some(Reconnect::None),
                ..Default::default()
            }),
            channel,
            client_id,
            error: Some(message.into()),
            id,
            successful: Some(false),
            ..Self::default()
        }
    }
}

impl From<Message> for Json<[Message; 1]> {
    #[inline(always)]
    fn from(message: Message) -> Self {
        Json([message])
    }
}
