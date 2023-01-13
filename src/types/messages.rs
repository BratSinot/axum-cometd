mod de;

use crate::types::{ChannelId, ClientId};
use axum::Json;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Advice {
    pub(crate) interval: Option<u64>,
    #[serde(rename = "maxInterval")]
    pub(crate) max_interval: Option<u64>,
    //#[serde(rename = "multiple-clients")]
    //pub(crate) multiple_clients: Option<bool>,
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
    pub(crate) fn handshake() -> Self {
        Self {
            reconnect: Some(Reconnect::Handshake),
            interval: Some(0),
            ..Default::default()
        }
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
    //#[serde(rename = "connectionType")]
    //pub(crate) connection_type: Option<String>,
    pub(crate) data: Option<JsonValue>,
    pub(crate) error: Option<String>,
    //pub(crate) ext: Option<JsonValue>,
    pub(crate) id: Option<String>,
    #[serde(rename = "minimumVersion")]
    pub(crate) minimum_version: Option<String>,
    //pub(crate) reestablish: Option<bool>,
    #[serde(default, deserialize_with = "de::deserialize_subscription")]
    pub(crate) subscription: Option<Vec<String>>,
    pub(crate) successful: Option<bool>,
    #[serde(rename = "supportedConnectionTypes")]
    pub(crate) supported_connection_types: Option<Vec<String>>,
    //pub(crate) timestamp: Option<String>,
    pub(crate) version: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct SubscriptionMessage {
    pub(crate) channel: ChannelId,
    pub(crate) msg: JsonValue,
}

impl Message {
    #[inline(always)]
    pub(crate) fn ok(id: Option<String>, channel: Option<String>) -> Self {
        Self {
            id,
            channel,
            successful: Some(true),
            ..Default::default()
        }
    }

    #[inline(always)]
    pub(crate) fn session_unknown(
        id: Option<String>,
        channel: Option<String>,
        advice: Option<Advice>,
    ) -> Self {
        Self {
            id,
            successful: Some(false),
            channel,
            error: Some("402::session_unknown".into()),
            advice,
            ..Default::default()
        }
    }

    #[inline(always)]
    pub(crate) fn wrong_minimum_version(
        id: Option<String>,
        minimum_version: Option<String>,
    ) -> Self {
        Self {
            id,
            successful: Some(false),
            minimum_version,
            error: Some("400::minimum_version_missing".into()),
            ..Default::default()
        }
    }

    #[inline(always)]
    pub(crate) fn subscription_missing(id: Option<String>) -> Self {
        Self {
            id,
            channel: Some("/meta/subscribe".into()),
            successful: Some(false),
            error: Some("403::subscription_missing".into()),
            ..Default::default()
        }
    }

    #[inline(always)]
    pub(crate) fn channel_missing(id: Option<String>) -> Self {
        Self {
            id,
            successful: Some(false),
            error: Some("400::channel_missing".into()),
            ..Default::default()
        }
    }
}

impl From<Message> for Json<[Message; 1]> {
    #[inline(always)]
    fn from(message: Message) -> Self {
        Json([message])
    }
}
