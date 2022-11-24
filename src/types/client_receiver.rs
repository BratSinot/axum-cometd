use crate::messages::SubscriptionMessage;
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast, Notify},
    time,
};

#[derive(Debug)]
pub(crate) struct ClientReceiver<Msg> {
    start_timeout: Arc<Notify>,
    rx: broadcast::Receiver<SubscriptionMessage<Msg>>,
}

impl<Msg> ClientReceiver<Msg> {
    #[inline(always)]
    pub(crate) fn new(
        start_timeout: Arc<Notify>,
        rx: broadcast::Receiver<SubscriptionMessage<Msg>>,
    ) -> Self {
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
            loop {
                match self.rx.recv().await {
                    Ok(data) => break Some(data),
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break None,
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
