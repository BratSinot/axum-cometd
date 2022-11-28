use crate::LongPoolingServiceContext;
use ahash::AHashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const DEFAULT_TIMEOUT_MS: u64 = 20_000;
const DEFAULT_INTERVAL_MS: u64 = 0;
const DEFAULT_MAX_INTERVAL_MS: u64 = 60_000;
const DEFAULT_CHANNEL_CAPACITY: usize = 1_000_000;

/// A builder to construct `LongPoolingServiceContext`.
#[derive(Debug, Default)]
pub struct LongPoolingServiceContextBuilder {
    subscriptions_capacity: usize,
    client_ids_capacity: usize,
    consts: LongPoolingServiceContextConsts,
}

#[derive(Debug)]
pub(crate) struct LongPoolingServiceContextConsts {
    pub(crate) timeout_ms: u64,
    pub(crate) interval_ms: u64,
    pub(crate) max_interval_ms: u64,
    pub(crate) client_channel_capacity: usize,
    pub(crate) subscription_channel_capacity: usize,
}

impl Default for LongPoolingServiceContextConsts {
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

impl LongPoolingServiceContextBuilder {
    /// Construct a new `LongPoolingServiceContextBuilder`.
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a `LongPoolingServiceContextBuilder`.
    ///
    /// # Example
    /// ```rust
    /// use axum_cometd::LongPoolingServiceContextBuilder;
    ///
    /// let context = LongPoolingServiceContextBuilder::new().build();
    /// # let app = axum_cometd::RouterBuilder::new().build::<()>(&context);
    /// ```
    #[inline(always)]
    pub fn build<Msg>(self) -> Arc<LongPoolingServiceContext<Msg>> {
        let Self {
            subscriptions_capacity,
            client_ids_capacity,
            consts,
        } = self;

        Arc::new(LongPoolingServiceContext {
            consts,
            subscriptions_data: RwLock::new(AHashMap::with_capacity(subscriptions_capacity)),
            client_id_channels: Arc::new(RwLock::new(AHashMap::with_capacity(client_ids_capacity))),
        })
    }

    /// Set message wait timeout in milliseconds.
    #[inline(always)]
    pub fn timeout_ms(self, timeout_ms: u64) -> Self {
        Self {
            consts: LongPoolingServiceContextConsts {
                timeout_ms,
                ..self.consts
            },
            ..self
        }
    }

    /// Set timeout in milliseconds, that the client must wait between two connects.
    #[inline(always)]
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
    pub fn max_interval_ms(self, max_interval_ms: u64) -> Self {
        Self {
            consts: LongPoolingServiceContextConsts {
                max_interval_ms,
                ..self.consts
            },
            ..self
        }
    }

    /// Set internal-channel capacity for clientId channel.
    #[inline(always)]
    pub fn client_channel_capacity(self, client_channel_capacity: usize) -> Self {
        Self {
            consts: LongPoolingServiceContextConsts {
                client_channel_capacity,
                ..self.consts
            },
            ..self
        }
    }

    /// Set internal-channel capacity for subscription channel.
    #[inline(always)]
    pub fn subscription_channel_capacity(self, subscription_channel_capacity: usize) -> Self {
        Self {
            consts: LongPoolingServiceContextConsts {
                subscription_channel_capacity,
                ..self.consts
            },
            ..self
        }
    }
}
