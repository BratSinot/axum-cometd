use crate::messages::SubscriptionMessage;
use async_broadcast::{Receiver, RecvError};
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::{sync::Notify, time};

#[derive(Debug)]
pub(crate) struct ClientReceiver {
    start_timeout: Arc<Notify>,
    rx: Receiver<SubscriptionMessage>,
}

impl ClientReceiver {
    #[inline(always)]
    pub(crate) fn new(start_timeout: Arc<Notify>, rx: Receiver<SubscriptionMessage>) -> Self {
        Self { start_timeout, rx }
    }

    #[inline]
    pub(crate) async fn recv_timeout(
        &mut self,
        duration: Duration,
    ) -> Result<Option<SubscriptionMessage>, time::error::Elapsed> {
        time::timeout(duration, async {
            match self.rx.recv().await {
                Ok(data) => Some(data),
                Err(RecvError::Closed) => None,
                Err(RecvError::Overflowed(_)) => {
                    unreachable!("broadcast overflow mode was enabled")
                }
            }
        })
        .await
    }
}

impl Drop for ClientReceiver {
    fn drop(&mut self) {
        self.start_timeout.notify_waiters();
    }
}
