use async_trait::async_trait;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;

#[async_trait]
pub trait BufferedRecv<T> {
    async fn timeout_buffered_recv(&mut self, capacity: usize, timeout: Duration) -> Vec<T>;
}

#[async_trait]
impl<T> BufferedRecv<T> for broadcast::Receiver<T>
where
    T: Clone + Send,
{
    async fn timeout_buffered_recv(&mut self, capacity: usize, timeout: Duration) -> Vec<T> {
        if timeout == Duration::ZERO {
            return vec![];
        }

        let mut ret = Vec::with_capacity(capacity);

        let mut wait = tokio::time::interval(timeout);
        wait.reset();

        loop {
            select! {
                _ = wait.tick() => break,
                msg = self.recv() => { if let Ok(msg) = msg { ret.push(msg); } }
            }
        }

        ret
    }
}
