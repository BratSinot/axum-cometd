use serde::de;
use serde::de::Unexpected;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError<'a> {
    #[error("invalid length: {0}")]
    InvalidLength(usize),
    #[error("invalid value: `{0}`")]
    InvalidValue(&'a str),
}

impl<'a> ParseError<'a> {
    #[inline(always)]
    pub(crate) fn into_de_error<T: de::Error>(self) -> T {
        match self {
            ParseError::InvalidLength(len) => T::invalid_length(len, &"40"),
            ParseError::InvalidValue(s) => {
                T::invalid_value(Unexpected::Str(s), &"valid hex string")
            }
        }
    }
}
