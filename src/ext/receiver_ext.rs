use async_trait::async_trait;
use std::time::Duration;
use tokio::{sync::broadcast, time};

#[async_trait]
pub trait ReceiverExt<T> {
    async fn recv_ignore_lagged(&mut self) -> Option<T>;
    async fn recv_ignore_lagged_timeout(
        &mut self,
        duration: Duration,
    ) -> Result<Option<T>, time::error::Elapsed>;
}

#[async_trait]
impl<T> ReceiverExt<T> for broadcast::Receiver<T>
where
    T: Clone + Send,
{
    #[inline]
    async fn recv_ignore_lagged(&mut self) -> Option<T> {
        loop {
            match self.recv().await {
                Ok(data) => break Some(data),
                Err(broadcast::error::RecvError::Lagged(num)) => {
                    tracing::warn!("RecvError::Laged({num})");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => break None,
            }
        }
    }

    #[inline]
    async fn recv_ignore_lagged_timeout(
        &mut self,
        duration: Duration,
    ) -> Result<Option<T>, time::error::Elapsed> {
        time::timeout(duration, self.recv_ignore_lagged()).await
    }
}
