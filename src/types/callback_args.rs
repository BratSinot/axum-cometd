use crate::{ClientId, LongPollingServiceContext};
use axum::http::HeaderMap;
use std::sync::Arc;

/// Struct used in sessionAdded / sessionRemoved callbacks.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct CallBackArguments {
    pub context: Arc<LongPollingServiceContext>,
    pub client_id: ClientId,
    pub headers: HeaderMap,
}
