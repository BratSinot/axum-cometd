use async_broadcast::{SendError, Sender, TrySendError};

pub(crate) async fn ignore_inactive_broadcast<Msg>(
    tx: &Sender<Msg>,
    msg: Msg,
) -> Result<(), SendError<Msg>>
where
    Msg: Clone,
{
    match tx.try_broadcast(msg) {
        Ok(None) | Err(TrySendError::Inactive(_)) => Ok(()),
        Ok(Some(msg)) | Err(TrySendError::Full(msg)) => match tx.broadcast(msg).await {
            Ok(None) => Ok(()),
            Err(err) => Err(err),
            Ok(Some(_msg)) => unreachable!("broadcast overflow mode was enabled"),
        },
        Err(TrySendError::Closed(msg)) => Err(SendError(msg)),
    }
}
