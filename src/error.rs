mod http_handler_error;

use async_broadcast::SendError as AsyncSendError;
use tokio::sync::mpsc::error::SendError as TokioSendError;

use crate::ClientId;
pub(crate) use http_handler_error::*;

/// Error returned by the `LongPoolingServiceContext::send`.
#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("internal error, channel closed")]
    Closed,
    #[error("client ({0}) wasn't found")]
    ClientWasntFound(ClientId),
}

impl<Msg> From<TokioSendError<Msg>> for SendError {
    fn from(_: TokioSendError<Msg>) -> Self {
        Self::Closed
    }
}

impl<Msg> From<AsyncSendError<Msg>> for SendError {
    fn from(_: AsyncSendError<Msg>) -> Self {
        Self::Closed
    }
}
