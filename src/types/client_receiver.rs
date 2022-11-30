use crate::messages::SubscriptionMessage;
use async_broadcast::{Receiver, RecvError};
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::{sync::Notify, time};

#[derive(Debug)]
pub(crate) struct ClientReceiver<Msg> {
    start_timeout: Arc<Notify>,
    rx: Receiver<SubscriptionMessage<Msg>>,
}

impl<Msg> ClientReceiver<Msg> {
    #[inline(always)]
    pub(crate) fn new(start_timeout: Arc<Notify>, rx: Receiver<SubscriptionMessage<Msg>>) -> Self {
        Self { start_timeout, rx }
    }

    #[inline]
    pub(crate) async fn recv_timeout(
        &mut self,
        duration: Duration,
    ) -> Result<Option<SubscriptionMessage<Msg>>, time::error::Elapsed>
    where
        Msg: Clone,
    {
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

impl<Msg> Drop for ClientReceiver<Msg> {
    fn drop(&mut self) {
        self.start_timeout.notify_waiters();
    }
}
