use crate::{messages::SubscriptionMessage, LongPoolingServiceContext};
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    select,
    sync::{broadcast, Notify},
    time,
};
use tokio_util::sync::CancellationToken;

// TODO: Replace on Arc<str>?
pub type ClientId = String;

#[derive(Debug)]
pub struct ClientIdGen(AtomicU64);

impl ClientIdGen {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    #[inline(always)]
    pub fn next(&self) -> ClientId {
        self.0.fetch_add(1, Ordering::Relaxed).to_string()
    }
}

#[derive(Debug)]
pub(crate) struct ClientSender<Msg> {
    stop_signal: CancellationToken,
    start_timeout: Arc<Notify>,
    cancel_timeout: Arc<Notify>,
    tx: broadcast::Sender<SubscriptionMessage<Msg>>,
}

impl<Msg> ClientSender<Msg> {
    #[inline]
    pub fn create(
        context: LongPoolingServiceContext<Msg>,
        client_id: ClientId,
        timeout: Duration,
        tx: broadcast::Sender<SubscriptionMessage<Msg>>,
    ) -> Self
    where
        Msg: Debug + Clone + Send + 'static,
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
                                client_id = client_id,
                                "Client `{client_id}` timeout."
                            );
                            context.unsubscribe(&client_id).await;
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
        }
    }

    #[inline]
    pub fn subscribe(&self) -> ClientReceiver<Msg> {
        self.cancel_timeout.notify_waiters();

        let start_timeout = self.start_timeout.clone();
        let rx = self.tx.subscribe();

        ClientReceiver { start_timeout, rx }
    }

    #[inline(always)]
    pub fn send(
        &self,
        msg: SubscriptionMessage<Msg>,
    ) -> Result<usize, broadcast::error::SendError<SubscriptionMessage<Msg>>> {
        self.tx.send(msg)
    }
}

impl<Msg> Drop for ClientSender<Msg> {
    fn drop(&mut self) {
        self.stop_signal.cancel();
    }
}

#[derive(Debug)]
pub(crate) struct ClientReceiver<Msg> {
    start_timeout: Arc<Notify>,
    rx: broadcast::Receiver<SubscriptionMessage<Msg>>,
}

impl<Msg> ClientReceiver<Msg> {
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
