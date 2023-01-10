use crate::types::ClientId;
use crate::LongPollingServiceContext;
use std::{sync::Arc, time::Duration};
use tokio::{select, sync::Notify, time};

pub(super) fn spawn(
    context: Arc<LongPollingServiceContext>,
    client_id: ClientId,
    timeout: Duration,
    stop_signal: Arc<Notify>,
    start_timeout: Arc<Notify>,
    cancel_timeout: Arc<Notify>,
) {
    tokio::task::spawn(async move {
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

            if wait_until_client_disconnect(&stop_signal, &start_timeout).await {
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
