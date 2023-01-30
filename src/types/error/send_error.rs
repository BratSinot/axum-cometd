// https://github.com/rust-lang/rust-clippy/issues/10198
#![allow(clippy::std_instead_of_core)]

use crate::ClientId;
use tokio::sync::mpsc::error::SendError as TokioSendError;

/// Error returned by the `LongPoolingServiceContext::send`.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum SendError {
    #[error("internal error, channel closed")]
    Closed,
    #[error("client ({0}) wasn't found")]
    ClientWasntFound(ClientId),
    #[error("invalid channel name")]
    InvalidChannel,
}

impl<Msg> From<TokioSendError<Msg>> for SendError {
    fn from(_: TokioSendError<Msg>) -> Self {
        Self::Closed
    }
}
