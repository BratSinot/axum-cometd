use crate::ClientId;
use axum::http::HeaderMap;

#[non_exhaustive]
#[allow(missing_docs)]
#[derive(Debug)]
pub enum Event<AdditionalData> {
    /// Struct used in sessionAdded callbacks.
    SessionAddedArgs {
        client_id: ClientId,
        headers: HeaderMap,
        data: AdditionalData,
    },
    /// Struct used in subscribe callbacks.
    SubscribeArgs {
        client_id: ClientId,
        headers: HeaderMap,
        channels: Vec<String>,
        data: AdditionalData,
    },
    /// Struct used in sessionRemoved callbacks.
    SessionRemovedArgs { client_id: ClientId },
}
