mod http_handler_error;

use tokio::sync::mpsc as TokioMpsc;

pub(crate) use http_handler_error::*;

/// Error returned by the `LongPoolingServiceContext::send`.
#[derive(Debug, thiserror::Error)]
#[error("internal error, channel closed")]
pub struct SendError;

impl<Msg> From<TokioMpsc::error::SendError<Msg>> for SendError {
    fn from(_: TokioMpsc::error::SendError<Msg>) -> Self {
        Self
    }
}

impl<Msg> From<async_broadcast::SendError<Msg>> for SendError {
    fn from(_: async_broadcast::SendError<Msg>) -> Self {
        Self
    }
}
