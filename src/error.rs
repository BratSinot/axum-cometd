use crate::ClientId;
use tokio::sync::mpsc as TokioMpsc;

/// Error returned by the `LongPoolingServiceContext::send`.
#[derive(Debug, thiserror::Error)]
#[error("internal error, channel closed")]
pub struct SendError<Msg>(pub Msg);

impl<Msg> From<TokioMpsc::error::SendError<Msg>> for SendError<Msg> {
    fn from(TokioMpsc::error::SendError(msg): TokioMpsc::error::SendError<Msg>) -> Self {
        Self(msg)
    }
}

pub(crate) type CometdResult<T> = Result<T, CometdError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CometdError {
    #[error("Client with id {0} doesn't exist.")]
    ClientDoesntExist(ClientId),
}
