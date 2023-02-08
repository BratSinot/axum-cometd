use crate::Event;
use async_broadcast::Sender;
use std::sync::Arc;

/// Sender channel to send custom data.
#[derive(Debug)]
pub struct CometdCustomDataSender<AdditionalData, CustomData>(
    pub(crate) Sender<Arc<Event<AdditionalData, CustomData>>>,
);

impl<AdditionalData, CustomData> Clone for CometdCustomDataSender<AdditionalData, CustomData> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<AdditionalData, CustomData> CometdCustomDataSender<AdditionalData, CustomData> {
    /// Send custom data to event channel.
    #[inline(always)]
    pub async fn send(&self, data: CustomData) {
        let _ = self.0.broadcast(Arc::new(Event::CustomData(data))).await;
    }
}
