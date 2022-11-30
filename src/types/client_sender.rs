mod client_timeout;

use crate::{
    messages::SubscriptionMessage,
    types::{ClientId, ClientReceiver},
    LongPoolingServiceContext,
};
use async_broadcast::{InactiveReceiver, SendError, Sender, TrySendError};
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub(crate) struct ClientSender<Msg> {
    stop_signal: CancellationToken,
    start_timeout: Arc<Notify>,
    cancel_timeout: Arc<Notify>,
    tx: Sender<SubscriptionMessage<Msg>>,
    inactive_rx: InactiveReceiver<SubscriptionMessage<Msg>>,
}

impl<Msg> ClientSender<Msg> {
    #[inline]
    pub(crate) fn create(
        context: Arc<LongPoolingServiceContext<Msg>>,
        client_id: ClientId,
        timeout: Duration,
        tx: Sender<SubscriptionMessage<Msg>>,
        inactive_rx: InactiveReceiver<SubscriptionMessage<Msg>>,
    ) -> Self
    where
        Msg: Send + Sync + 'static,
    {
        let stop_signal = CancellationToken::new();
        let start_timeout = Arc::new(Notify::new());
        let cancel_timeout = Arc::new(Notify::new());

        client_timeout::spawn(
            context,
            client_id,
            timeout,
            stop_signal.clone(),
            start_timeout.clone(),
            cancel_timeout.clone(),
        );

        start_timeout.notify_waiters();

        Self {
            stop_signal,
            start_timeout,
            cancel_timeout,
            tx,
            inactive_rx,
        }
    }

    #[inline]
    pub(crate) fn subscribe(&self) -> ClientReceiver<Msg> {
        self.cancel_timeout.notify_waiters();

        let start_timeout = self.start_timeout.clone();
        let rx = self.inactive_rx.activate_cloned();

        ClientReceiver::new(start_timeout, rx)
    }

    #[inline]
    pub(crate) async fn send(
        &self,
        msg: SubscriptionMessage<Msg>,
    ) -> Result<(), SendError<SubscriptionMessage<Msg>>>
    where
        Msg: Clone,
    {
        match self.tx.try_broadcast(msg) {
            Ok(None) | Err(TrySendError::Inactive(_)) => Ok(()),
            Ok(Some(msg)) | Err(TrySendError::Full(msg)) => match self.tx.broadcast(msg).await {
                Ok(None) => Ok(()),
                Err(err) => Err(err),
                Ok(Some(_msg)) => unreachable!("broadcast overflow mode was enabled"),
            },
            Err(TrySendError::Closed(msg)) => Err(SendError(msg)),
        }
    }
}

impl<Msg> Drop for ClientSender<Msg> {
    fn drop(&mut self) {
        self.stop_signal.cancel();
    }
}
