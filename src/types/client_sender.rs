mod client_timeout;

use crate::{
    messages::SubscriptionMessage,
    types::{ClientId, ClientReceiver, CookieId},
    LongPollingServiceContext,
};
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::sync::{
    mpsc::{error::SendError, Receiver, Sender},
    Mutex, Notify,
};

#[derive(Debug)]
pub(crate) struct ClientSender {
    cookie_id: CookieId,
    signals: Arc<Signals>,
    tx: Sender<SubscriptionMessage>,
    rx: Arc<Mutex<Receiver<SubscriptionMessage>>>,
}

#[derive(Debug, Default)]
pub(crate) struct Signals {
    pub(crate) stop_signal: Notify,
    pub(crate) start_timeout: Notify,
    pub(crate) cancel_timeout: Notify,
}

impl ClientSender {
    #[inline]
    pub(crate) fn create(
        context: Arc<LongPollingServiceContext>,
        cookie_id: CookieId,
        client_id: ClientId,
        timeout: Duration,
        tx: Sender<SubscriptionMessage>,
        rx: Receiver<SubscriptionMessage>,
    ) -> Self {
        let signals = Arc::new(Signals::default());
        let rx = Arc::new(Mutex::new(rx));

        client_timeout::spawn(context, client_id, timeout, signals.clone());

        signals.start_timeout.notify_waiters();

        Self {
            cookie_id,
            signals,
            tx,
            rx,
        }
    }

    #[inline(always)]
    pub(crate) fn cookie_id(&self) -> CookieId {
        self.cookie_id
    }

    #[inline]
    pub(crate) fn subscribe(&self) -> ClientReceiver {
        self.signals.cancel_timeout.notify_waiters();
        ClientReceiver::new(self.signals.clone(), self.rx.clone())
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
        self.signals.stop_signal.notify_one();
    }
}
