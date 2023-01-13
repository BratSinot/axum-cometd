// https://github.com/rust-lang/rust-clippy/issues/10198
#![allow(clippy::std_instead_of_core)]

use crate::{messages::SubscriptionMessage, types::Signals};
use core::{fmt::Debug, time::Duration};
use std::sync::Arc;
use tokio::{
    sync::{mpsc::Receiver, Mutex, TryLockError},
    time,
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum ClientReceiverError {
    #[error("elapsed")]
    Elapsed(#[from] time::error::Elapsed),
    #[error("double lock")]
    AlreadyLocked(#[from] TryLockError),
}

// TODO: Unite Arc's.
#[derive(Debug)]
pub(crate) struct ClientReceiver {
    signals: Arc<Signals>,
    rx: Arc<Mutex<Receiver<SubscriptionMessage>>>,
}

impl ClientReceiver {
    #[inline(always)]
    pub(crate) fn new(
        signals: Arc<Signals>,
        rx: Arc<Mutex<Receiver<SubscriptionMessage>>>,
    ) -> Self {
        Self { signals, rx }
    }

    #[inline]
    pub(crate) async fn recv_timeout(
        &mut self,
        duration: Duration,
    ) -> Result<Option<SubscriptionMessage>, ClientReceiverError> {
        let mut rx = self.rx.try_lock()?;
        let msg = time::timeout(duration, rx.recv()).await?;
        Ok(msg)
    }
}

impl Drop for ClientReceiver {
    fn drop(&mut self) {
        self.signals.start_timeout.notify_waiters();
    }
}
