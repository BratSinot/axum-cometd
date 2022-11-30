use crate::types::ClientId;
use crate::LongPoolingServiceContext;
use std::{sync::Arc, time::Duration};
use tokio::{select, sync::Notify, time};
use tokio_util::sync::CancellationToken;

pub(super) fn spawn<Msg>(
    context: Arc<LongPoolingServiceContext<Msg>>,
    client_id: ClientId,
    timeout: Duration,
    stop_signal: CancellationToken,
    start_timeout: Arc<Notify>,
    cancel_timeout: Arc<Notify>,
) where
    Msg: Send + Sync + 'static,
{
    tokio::task::spawn(async move {
        while wait_until_client_disconnect(&stop_signal, &start_timeout).await {
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
    });
}

#[inline]
async fn wait_until_client_disconnect(
    stop_signal: &CancellationToken,
    start_timeout: &Notify,
) -> bool {
    select! {
        _ = start_timeout.notified() => true,
        _ = stop_signal.cancelled() => false,
    }
}
