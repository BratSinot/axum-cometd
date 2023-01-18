use crate::ClientId;
use axum::http::HeaderMap;

/// Struct used in sessionAdded callbacks.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct SessionAddedArgs {
    pub client_id: ClientId,
    pub headers: HeaderMap,
}

/// Struct used in subscribe callbacks.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct SubscribeArgs {
    pub client_id: ClientId,
    pub headers: HeaderMap,
    pub channels: Vec<String>,
}

/// Struct used in sessionRemoved callbacks.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct SessionRemovedArgs {
    pub client_id: ClientId,
}
