use crate::{
    types::Callback, ClientIdGen, LongPollingServiceContext, SessionAddedArgs, SessionRemovedArgs,
    SubscribeArgs,
};
use ahash::AHashMap;
use std::{future::Future, sync::Arc};
use tokio::sync::RwLock;

const DEFAULT_TIMEOUT_MS: u64 = 20_000;
const DEFAULT_INTERVAL_MS: u64 = 0;
const DEFAULT_MAX_INTERVAL_MS: u64 = 60_000;
const DEFAULT_CHANNEL_CAPACITY: usize = 500;
const DEFAULT_STORAGE_CAPACITY: usize = 10_000;

/// A builder to construct `LongPoolingServiceContext`.
#[derive(Debug)]
pub struct LongPollingServiceContextBuilder {
    subscriptions_storage_capacity: usize,
    client_ids_storage_capacity: usize,
    consts: LongPollingServiceContextConsts,
    session_added: Callback<SessionAddedArgs>,
    subscribe_added: Callback<SubscribeArgs>,
    session_removed: Callback<SessionRemovedArgs>,
}

impl Default for LongPollingServiceContextBuilder {
    #[inline(always)]
    fn default() -> Self {
        Self {
            subscriptions_storage_capacity: DEFAULT_STORAGE_CAPACITY,
            client_ids_storage_capacity: DEFAULT_STORAGE_CAPACITY,
            consts: Default::default(),
            session_added: Callback::Empty,
            subscribe_added: Callback::Empty,
            session_removed: Callback::Empty,
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
    /// let context = LongPollingServiceContextBuilder::new().build();
    /// ```
    #[inline(always)]
    pub fn build(self) -> Arc<LongPollingServiceContext> {
        let Self {
            subscriptions_storage_capacity,
            client_ids_storage_capacity,
            consts,
            session_added,
            subscribe_added,
            session_removed,
        } = self;

        Arc::new(LongPollingServiceContext {
            session_added,
            subscribe_added,
            session_removed,
            wildnames_cache: Default::default(),
            channel_name_validator: Default::default(),
            consts,
            channels_data: RwLock::new(AHashMap::with_capacity(subscriptions_storage_capacity)),
            client_id_senders: Arc::new(RwLock::new((
                ClientIdGen::new(),
                AHashMap::with_capacity(client_ids_storage_capacity),
            ))),
        })
    }

    /// Set message wait timeout in milliseconds.
    #[inline(always)]
    pub fn timeout_ms(self, timeout_ms: u64) -> Self {
        Self {
            consts: LongPollingServiceContextConsts {
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
            consts: LongPollingServiceContextConsts {
                max_interval_ms,
                ..self.consts
            },
            ..self
        }
    }

    /// Set capacity of internal client channels.
    #[inline(always)]
    pub fn client_channel_capacity(self, capacity: usize) -> Self {
        Self {
            consts: LongPollingServiceContextConsts {
                client_channel_capacity: capacity,
                ..self.consts
            },
            ..self
        }
    }

    /// Set capacity of internal client channels storage.
    #[inline(always)]
    pub fn client_storage_capacity(self, capacity: usize) -> Self {
        Self {
            client_ids_storage_capacity: capacity,
            ..self
        }
    }

    /// Set capacity of internal subscription channels.
    #[inline(always)]
    pub fn subscription_channel_capacity(self, capacity: usize) -> Self {
        Self {
            consts: LongPollingServiceContextConsts {
                subscription_channel_capacity: capacity,
                ..self.consts
            },
            ..self
        }
    }

    /// Set capacity of internal subscription channels storage.
    #[inline(always)]
    pub fn subscription_storage_capacity(self, capacity: usize) -> Self {
        Self {
            subscriptions_storage_capacity: capacity,
            ..self
        }
    }

    /// Set sync callback on new session creation.
    #[inline(always)]
    pub fn session_added<F>(self, callback: F) -> Self
    where
        F: Fn(SessionAddedArgs) + Send + Sync + 'static,
    {
        Self {
            session_added: Callback::new_sync(callback),
            ..self
        }
    }

    /// Set async callback on new session creation.
    #[inline(always)]
    pub fn async_session_added<F, Fut>(self, callback: F) -> Self
    where
        F: Fn(SessionAddedArgs) -> Fut + Sync + Send + 'static,
        Fut: Future<Output = ()> + Sync + Send + 'static,
    {
        Self {
            session_added: Callback::new_async(callback),
            ..self
        }
    }

    /// Set sync callback on new subscribe creation.
    #[inline(always)]
    pub fn subscribe_added<F>(self, callback: F) -> Self
    where
        F: Fn(SubscribeArgs) + Send + Sync + 'static,
    {
        Self {
            subscribe_added: Callback::new_sync(callback),
            ..self
        }
    }

    /// Set async callback on new subscribe creation.
    #[inline(always)]
    pub fn async_subscribe_added<F, Fut>(self, callback: F) -> Self
    where
        F: Fn(SubscribeArgs) -> Fut + Sync + Send + 'static,
        Fut: Future<Output = ()> + Sync + Send + 'static,
    {
        Self {
            subscribe_added: Callback::new_async(callback),
            ..self
        }
    }

    /// Set sync callback on new session creation.
    #[inline(always)]
    pub fn session_removed<F>(self, callback: F) -> Self
    where
        F: Fn(SessionRemovedArgs) + Send + Sync + 'static,
    {
        Self {
            session_removed: Callback::new_sync(callback),
            ..self
        }
    }

    /// Set async callback on new session creation.
    #[inline(always)]
    pub fn async_session_removed<F, Fut>(self, callback: F) -> Self
    where
        F: Fn(SessionRemovedArgs) -> Fut + Sync + Send + 'static,
        Fut: Future<Output = ()> + Sync + Send + 'static,
    {
        Self {
            session_removed: Callback::new_async(callback),
            ..self
        }
    }
}
