use crate::ClientId;

pub type CometdResult<T> = Result<T, CometdError>;

#[derive(Debug, thiserror::Error)]
pub enum CometdError {
    #[error("Client with id {0} doesn't exist.")]
    ClientDoesntExist(ClientId),
}
