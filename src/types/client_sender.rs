mod client_timeout;

use crate::{
    messages::SubscriptionMessage,
    types::{ClientId, ClientReceiver},
    LongPollingServiceContext,
};
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::sync::{
    mpsc::{error::SendError, Receiver, Sender},
    Mutex, Notify,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub(crate) struct ClientSender {
    stop_signal: CancellationToken,
    start_timeout: Arc<Notify>,
    cancel_timeout: Arc<Notify>,
    tx: Sender<SubscriptionMessage>,
    rx: Arc<Mutex<Receiver<SubscriptionMessage>>>,
}

impl ClientSender {
    #[inline]
    pub(crate) fn create(
        context: Arc<LongPollingServiceContext>,
        client_id: ClientId,
        timeout: Duration,
        tx: Sender<SubscriptionMessage>,
        rx: Receiver<SubscriptionMessage>,
    ) -> Self {
        let stop_signal = CancellationToken::new();
        let start_timeout = Arc::new(Notify::new());
        let cancel_timeout = Arc::new(Notify::new());
        let rx = Arc::new(Mutex::new(rx));

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
            rx,
        }
    }

    #[inline]
    pub(crate) fn subscribe(&self) -> ClientReceiver {
        self.cancel_timeout.notify_waiters();

        let start_timeout = self.start_timeout.clone();
        let rx = self.rx.clone();

        ClientReceiver::new(start_timeout, rx)
    }

    #[inline(always)]
    pub(crate) async fn send(
        &self,
        msg: SubscriptionMessage,
    ) -> Result<(), SendError<SubscriptionMessage>> {
        self.tx.send(msg).await
    }
}

impl Drop for ClientSender {
    fn drop(&mut self) {
        self.stop_signal.cancel();
    }
}
