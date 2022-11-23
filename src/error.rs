use crate::ClientId;

pub(crate) type CometdResult<T> = Result<T, CometdError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CometdError {
    #[error("Client with id {0} doesn't exist.")]
    ClientDoesntExist(ClientId),
}
