use crate::LongPollingServiceContext;
use ahash::AHashMap;
use async_broadcast::broadcast;
use std::sync::Arc;
use tokio::sync::RwLock;

const DEFAULT_TIMEOUT_MS: u64 = 20_000;
const DEFAULT_INTERVAL_MS: u64 = 0;
const DEFAULT_MAX_INTERVAL_MS: u64 = 60_000;
const DEFAULT_CHANNEL_CAPACITY: usize = 500;
const DEFAULT_STORAGE_CAPACITY: usize = 10_000;

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
    pub(crate) timeout_ms: u64,
    pub(crate) interval_ms: u64,
    pub(crate) max_interval_ms: u64,
    pub(crate) client_channel_capacity: usize,
    pub(crate) subscription_channel_capacity: usize,
}

impl Default for LongPollingServiceContextConsts {
    #[inline(always)]
    fn default() -> Self {
        Self {
            timeout_ms: DEFAULT_TIMEOUT_MS,
            interval_ms: DEFAULT_INTERVAL_MS,
            max_interval_ms: DEFAULT_MAX_INTERVAL_MS,
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
    /// ```rust
    /// use axum_cometd::LongPollingServiceContextBuilder;
    ///
    /// let context = LongPollingServiceContextBuilder::new().build::<()>();
    /// ```
    #[inline(always)]
    pub fn build<AdditionalData>(self) -> Arc<LongPollingServiceContext<AdditionalData>> {
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

    /// Set message wait timeout in milliseconds.
    #[inline(always)]
    #[must_use]
    pub const fn timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.consts.timeout_ms = timeout_ms;
        self
    }

    /// Set timeout in milliseconds, that the client must wait between two connects.
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

    /// Set timeout in milliseconds, which server wait between erase clientId.
    #[inline(always)]
    #[must_use]
    pub const fn max_interval_ms(mut self, max_interval_ms: u64) -> Self {
        self.consts.max_interval_ms = max_interval_ms;
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
