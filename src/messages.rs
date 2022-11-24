use crate::types::{ClientId, SubscriptionId};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_with::skip_serializing_none;
use std::fmt::Debug;

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Advice {
    pub(crate) interval: Option<u64>,
    #[serde(rename = "maxInterval")]
    pub(crate) max_interval: Option<u64>,
    #[serde(rename = "multiple-clients")]
    pub(crate) multiple_clients: Option<bool>,
    pub(crate) reconnect: Option<Reconnect>,
    pub(crate) timeout: Option<u64>,
    pub(crate) hosts: Option<Vec<String>>,
}

impl Advice {
    #[inline(always)]
    pub(crate) fn retry(timeout_ms: u64, interval_ms: u64) -> Self {
        Self {
            reconnect: Some(Reconnect::Retry),
            timeout: Some(timeout_ms),
            interval: Some(interval_ms),
            ..Default::default()
        }
    }

    #[inline(always)]
    pub(crate) fn interval(&self) -> Option<u64> {
        self.interval
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Reconnect {
    Retry,
    Handshake,
    None,
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Message {
    pub(crate) advice: Option<Advice>,
    pub(crate) channel: Option<String>,
    #[serde(rename = "clientId")]
    pub(crate) client_id: Option<ClientId>,
    #[serde(rename = "connectionType")]
    pub(crate) connection_type: Option<String>,
    // TODO: Replace on Msg generic?
    pub(crate) data: Option<JsonValue>,
    pub(crate) error: Option<String>,
    //pub(crate) ext: Option<JsonValue>,
    pub(crate) id: Option<String>,
    #[serde(rename = "minimumVersion")]
    pub(crate) minimum_version: Option<String>,
    //pub(crate) reestablish: Option<bool>,
    pub(crate) subscription: Option<String>,
    pub(crate) successful: Option<bool>,
    #[serde(rename = "supportedConnectionTypes")]
    pub(crate) supported_connection_types: Option<Vec<String>>,
    //pub(crate) timestamp: Option<String>,
    pub(crate) version: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct SubscriptionMessage<Msg> {
    pub(crate) subscription: SubscriptionId,
    pub(crate) msg: Msg,
}

impl Message {
    #[inline]
    pub(crate) fn error<Str: Into<String>>(
        message: Str,
        channel: Option<String>,
        client_id: Option<ClientId>,
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
