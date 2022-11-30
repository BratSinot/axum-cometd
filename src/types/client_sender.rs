use crate::{
    messages::SubscriptionMessage,
    sugar::ignore_inactive_broadcast,
    types::{ClientId, ClientReceiver},
    LongPoolingServiceContext,
};
use async_broadcast::{InactiveReceiver, SendError, Sender};
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::{select, sync::Notify, time};
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

        tokio::task::spawn({
            let stop_signal = stop_signal.clone();
            let start_timeout = start_timeout.clone();
            let cancel_timeout = cancel_timeout.clone();
            async move {
                loop {
                    select! {
                        _ = start_timeout.notified() => {},
                        _ = stop_signal.cancelled() => break,
                    }

                    select! {
                        _ = stop_signal.cancelled() => break,
                        _ = time::sleep(timeout) => {
                            tracing::info!(
                                client_id = %client_id,
                                "Client `{client_id}` timeout."
                            );
                            context.unsubscribe(client_id).await;
                            break;
                        }
                        _ = cancel_timeout.notified() => continue,
                    }
                }
            }
        });

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

    #[inline(always)]
    pub(crate) async fn send(
        &self,
        msg: SubscriptionMessage<Msg>,
    ) -> Result<(), SendError<SubscriptionMessage<Msg>>>
    where
        Msg: Clone,
    {
        ignore_inactive_broadcast(&self.tx, msg).await
    }
}

impl<Msg> Drop for ClientSender<Msg> {
    fn drop(&mut self) {
        self.stop_signal.cancel();
    }
}
