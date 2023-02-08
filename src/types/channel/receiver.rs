use crate::Event;
use async_broadcast::{Receiver, RecvError};
use std::sync::Arc;

/// Event channel receiver.
#[derive(Debug)]
pub struct CometdEventReceiver<AdditionalData, CustomData>(
    pub(crate) Receiver<Arc<Event<AdditionalData, CustomData>>>,
);

impl<AdditionalData, CustomData> CometdEventReceiver<AdditionalData, CustomData> {
    /// Receive event from event channel.
    /// Return `None` if channel was closed.
    #[inline(always)]
    pub async fn recv(&mut self) -> Option<Arc<Event<AdditionalData, CustomData>>> {
        match self.0.recv().await {
            Ok(event) => Some(event),
            Err(RecvError::Closed) => None,
            Err(RecvError::Overflowed(_)) => unreachable!(),
        }
    }
}
