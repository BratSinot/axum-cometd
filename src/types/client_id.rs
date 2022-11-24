use std::{
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
};

// TODO: Replace on Arc<str>?
pub(crate) type ClientId = String;

#[derive(Debug)]
pub(crate) struct ClientIdGen(AtomicU64);

impl ClientIdGen {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    #[inline(always)]
    pub(crate) fn next(&self) -> ClientId {
        self.0.fetch_add(1, Ordering::Relaxed).to_string()
    }
}
