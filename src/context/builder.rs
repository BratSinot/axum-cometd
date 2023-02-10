use crate::{consts::*, LongPollingServiceContext};
use ahash::AHashMap;
use async_broadcast::broadcast;
use core::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A builder to construct `LongPoolingServiceContext`.
#[derive(Debug)]
pub struct LongPollingServiceContextBuilder {
    events_channel_capacity: usize,
    subscriptions_storage_capacity: usize,
    client_ids_storage_capacity: usize,
    consts: LongPollingServiceContextConsts,
}

impl Default for LongPollingServiceContextBuilder {
    #[inline(always)]
    fn default() -> Self {
        Self {
            events_channel_capacity: DEFAULT_CHANNEL_CAPACITY,
            subscriptions_storage_capacity: DEFAULT_STORAGE_CAPACITY,
            client_ids_storage_capacity: DEFAULT_STORAGE_CAPACITY,
            consts: Default::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct LongPollingServiceContextConsts {
    pub(crate) timeout: Duration,
    pub(crate) interval: Duration,
    pub(crate) max_interval: Duration,
    pub(crate) client_channel_capacity: usize,
    pub(crate) subscription_channel_capacity: usize,
}

impl Default for LongPollingServiceContextConsts {
    #[inline(always)]
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            interval: DEFAULT_INTERVAL,
            max_interval: DEFAULT_MAX_INTERVAL,
            client_channel_capacity: DEFAULT_CHANNEL_CAPACITY,
            subscription_channel_capacity: DEFAULT_CHANNEL_CAPACITY,
        }
    }
}

impl LongPollingServiceContextBuilder {
    /// Construct a new `LongPoolingServiceContextBuilder`.
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a `LongPoolingServiceContextBuilder`.
    ///
    /// # Example
    /// ```rust,no_run
    /// use axum_cometd::LongPollingServiceContextBuilder;
    ///
    /// let context = LongPollingServiceContextBuilder::new().build::<(), ()>();
    /// ```
    #[inline(always)]
    pub fn build<AdditionalData, CustomData>(
        self,
    ) -> Arc<LongPollingServiceContext<AdditionalData, CustomData>> {
        let Self {
            events_channel_capacity,
            subscriptions_storage_capacity,
            client_ids_storage_capacity,
            consts,
        } = self;

        let (tx, mut rx) = broadcast(events_channel_capacity);
        rx.set_await_active(false);

        Arc::new(LongPollingServiceContext {
            tx,
            inactive_rx: rx.deactivate(),
            wildnames_cache: Default::default(),
            channel_name_validator: Default::default(),
            consts,
            channels_data: RwLock::new(AHashMap::with_capacity(subscriptions_storage_capacity)),
            client_id_senders: Arc::new(RwLock::new(AHashMap::with_capacity(
                client_ids_storage_capacity,
            ))),
        })
    }

    /// Set message wait timeout.
    #[inline(always)]
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.consts.timeout = timeout;
        self
    }

    /// Set timeout, that the client must wait between two connects.
    #[inline(always)]
    #[must_use]
    pub fn interval_ms(self, _interval_ms: u64) -> Self {
        unimplemented!()
        /*Self {
            consts: LongPoolingServiceContextConsts {
                interval_ms,
                ..self.consts
            },
            ..self
        }*/
    }

    /// Set timeout, which server wait between erase clientId.
    #[inline(always)]
    #[must_use]
    pub const fn max_interval(mut self, max_interval: Duration) -> Self {
        self.consts.max_interval = max_interval;
        self
    }

    /// Set capacity of event channel.
    #[inline(always)]
    #[must_use]
    pub const fn events_channel_capacity(mut self, capacity: usize) -> Self {
        self.events_channel_capacity = capacity;
        self
    }

    /// Set capacity of internal client channels.
    #[inline(always)]
    #[must_use]
    pub const fn client_channel_capacity(mut self, capacity: usize) -> Self {
        self.consts.client_channel_capacity = capacity;
        self
    }

    /// Set capacity of internal client channels storage.
    #[inline(always)]
    #[must_use]
    pub const fn client_storage_capacity(mut self, capacity: usize) -> Self {
        self.client_ids_storage_capacity = capacity;
        self
    }

    /// Set capacity of internal subscription channels.
    #[inline(always)]
    #[must_use]
    pub const fn subscription_channel_capacity(mut self, capacity: usize) -> Self {
        self.consts.subscription_channel_capacity = capacity;
        self
    }

    /// Set capacity of internal subscription channels storage.
    #[inline(always)]
    #[must_use]
    pub const fn subscription_storage_capacity(mut self, capacity: usize) -> Self {
        self.subscriptions_storage_capacity = capacity;
        self
    }
}
