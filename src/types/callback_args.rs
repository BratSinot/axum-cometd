use crate::{ClientId, LongPollingServiceContext};
use axum::http::HeaderMap;
use std::sync::Arc;

/// Struct used in sessionAdded callbacks.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct SessionAddedArgs {
    pub context: Arc<LongPollingServiceContext>,
    pub client_id: ClientId,
    pub headers: HeaderMap,
}

/// Struct used in sessionRemoved callbacks.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct SessionRemovedArgs {
    pub context: Arc<LongPollingServiceContext>,
    pub client_id: ClientId,
}
