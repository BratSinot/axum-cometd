use crate::ClientId;
use axum::http::HeaderMap;

#[allow(missing_docs)]
#[derive(Debug)]
pub enum Event<AdditionalData, CustomData> {
    /// Struct used in sessionAdded callbacks.
    SessionAdded {
        client_id: ClientId,
        headers: HeaderMap,
        data: AdditionalData,
    },
    /// Struct used in subscribe callbacks.
    Subscribe {
        client_id: ClientId,
        headers: HeaderMap,
        channels: Vec<String>,
        data: AdditionalData,
    },
    /// Struct used in sessionRemoved callbacks.
    SessionRemoved { client_id: ClientId },
    /// Some custom data to send.
    CustomData(CustomData),
}
