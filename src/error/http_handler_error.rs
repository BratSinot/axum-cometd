use crate::messages::Message;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

pub(crate) type HandlerResult<T> = Result<T, HandlerError>;

#[derive(Debug)]
pub(crate) enum HandlerError {
    StatusCode(StatusCode),
    Message(Message),
}

impl HandlerError {
    #[cfg(feature = "test")]
    #[allow(dead_code)]
    #[inline(always)]
    pub fn into_message(self) -> Option<Message> {
        if let HandlerError::Message(message) = self {
            Some(message)
        } else {
            None
        }
    }
}

impl IntoResponse for HandlerError {
    #[inline(always)]
    fn into_response(self) -> Response {
        match self {
            HandlerError::StatusCode(code) => code.into_response(),
            HandlerError::Message(message) => Json([message]).into_response(),
        }
    }
}

impl From<StatusCode> for HandlerError {
    #[inline(always)]
    fn from(code: StatusCode) -> Self {
        Self::StatusCode(code)
    }
}

impl From<Message> for HandlerError {
    #[inline(always)]
    fn from(message: Message) -> Self {
        Self::Message(message)
    }
}
