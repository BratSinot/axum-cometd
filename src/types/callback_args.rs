use crate::ClientId;
use axum::http::HeaderMap;

/// Struct used in sessionAdded callbacks.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct SessionAddedArgs<AdditionalData> {
    pub client_id: ClientId,
    pub headers: HeaderMap,
    pub data: AdditionalData,
}

/// Struct used in subscribe callbacks.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct SubscribeArgs<AdditionalData> {
    pub client_id: ClientId,
    pub headers: HeaderMap,
    pub channels: Vec<String>,
    pub data: AdditionalData,
}

/// Struct used in sessionRemoved callbacks.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct SessionRemovedArgs {
    pub client_id: ClientId,
}
