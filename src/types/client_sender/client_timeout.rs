use crate::{
    types::{ClientId, Signals},
    LongPollingServiceContext,
};
use core::time::Duration;
use std::sync::Arc;
use tokio::{select, sync::Notify, time};

pub(super) fn spawn<AdditionalData>(
    context: Arc<LongPollingServiceContext<AdditionalData>>,
    client_id: ClientId,
    timeout: Duration,
    signals: Arc<Signals>,
) where
    AdditionalData: Send + Sync + 'static,
{
    tokio::task::spawn(async move {
        let Signals {
            ref stop_signal,
            ref start_timeout,
            ref cancel_timeout,
        } = *signals;

        loop {
            select! {
                _ = stop_signal.notified() => break,
                _ = time::sleep(timeout) => {
                    tracing::info!(
                        client_id = %client_id,
                        "Client `{client_id}` timeout."
                    );
                    context.unsubscribe(client_id).await;
                    break;
                }
                _ = cancel_timeout.notified() => {},
            }

            if wait_until_client_disconnect(stop_signal, start_timeout).await {
                break;
            }
        }
    });
}

#[inline]
async fn wait_until_client_disconnect(stop_signal: &Notify, start_timeout: &Notify) -> bool {
    select! {
        _ = start_timeout.notified() => false,
        _ = stop_signal.notified() => true,
    }
}
