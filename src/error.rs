mod http_handler_error;

use crate::ClientId;
use tokio::sync::mpsc as TokioMpsc;

pub(crate) use http_handler_error::*;

/// Error returned by the `LongPoolingServiceContext::send`.
#[derive(Debug, thiserror::Error)]
#[error("internal error, channel closed")]
pub struct SendError;

impl<Msg> From<TokioMpsc::error::SendError<Msg>> for SendError {
    fn from(TokioMpsc::error::SendError(_): TokioMpsc::error::SendError<Msg>) -> Self {
        Self
    }
}

pub(crate) type CometdResult<T> = Result<T, CometdError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CometdError {
    #[error("Client with id {0} doesn't exist.")]
    ClientDoesntExist(ClientId),
}
