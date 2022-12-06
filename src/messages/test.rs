use crate::messages::{Advice, Message, Reconnect};

impl PartialEq for Advice {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            interval,
            max_interval,
            multiple_clients,
            reconnect,
            timeout,
            hosts,
        } = self;

        (
            interval,
            max_interval,
            multiple_clients,
            reconnect,
            timeout,
            hosts,
        ) == (
            &other.interval,
            &other.max_interval,
            &other.multiple_clients,
            &other.reconnect,
            &other.timeout,
            &other.hosts,
        )
    }
}
impl Eq for Advice {}

impl PartialEq for Reconnect {
    fn eq(&self, other: &Self) -> bool {
        use Reconnect::*;
        matches!(
            (self, other),
            (Retry, Retry) | (Handshake, Handshake) | (None, None)
        )
    }
}
impl Eq for Reconnect {}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            advice,
            channel,
            client_id,
            data,
            error,
            id,
            minimum_version,
            subscription,
            successful,
            supported_connection_types,
            version,
        } = self;

        (
            advice,
            channel,
            client_id,
            data,
            error,
            id,
            minimum_version,
            subscription,
            successful,
            supported_connection_types,
            version,
        ) == (
            &other.advice,
            &other.channel,
            &other.client_id,
            &other.data,
            &other.error,
            &other.id,
            &other.minimum_version,
            &other.subscription,
            &other.successful,
            &other.supported_connection_types,
            &other.version,
        )
    }
}
impl Eq for Message {}
