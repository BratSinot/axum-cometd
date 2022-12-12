use axum::http::StatusCode;
use regex::Regex;

const VALID_CHANNEL_NAME_REGEX: &str = r#"^(?:/[a-zA-Z0-9\-_!~()$@]+)+(?:(?:/\*\*)|(?:/\*)|/)?$"#;

#[derive(Debug)]
pub(crate) struct ChannelNameValidator(Regex);

impl Default for ChannelNameValidator {
    #[inline(always)]
    fn default() -> Self {
        #[allow(clippy::expect_used)]
        Self(Regex::new(VALID_CHANNEL_NAME_REGEX).expect("wrong regex"))
    }
}

impl ChannelNameValidator {
    #[inline(always)]
    pub(crate) fn validate(&self, name: &str) -> Result<(), StatusCode> {
        if self.0.is_match(name) {
            Ok(())
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_name_validator() {
        let validator = ChannelNameValidator::default();

        #[rustfmt::skip]
        for (channel, result) in [
            (r#"/first1"#, Ok(())),
            (r#"/first1*"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1**"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/*"#, Ok(())),
            (r#"/first1/**"#, Ok(())),
            (r#"/first1/"#, Ok(())),
            (r#"/first1/*"#, Ok(())),
            (r#"/first1/**"#, Ok(())),
            (r#"/first1/second2"#, Ok(())),
            (r#"/first1/second2*"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/second2**"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/second2/"#, Ok(())),
            (r#"/first1/second2/*"#, Ok(())),
            (r#"/first1/second2/**"#, Ok(())),
            (r#"/first1/second2/third3"#, Ok(())),
            (r#"/first1/second2/third3*"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/second2/third3**"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/second2/third3/"#, Ok(())),
            (r#"/first1/second2/third3/*"#, Ok(())),
            (r#"/first1/second2/third3/**"#, Ok(())),
            (r#"/first1/*/third3"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/*/third3/"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/*/third3/*"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/*/third3/**"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/second2/**/"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/second2/**/*"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/second2/**/**"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/second2/third3/-_!~()$@"#, Ok(())),
            (r#"/first1/second2/third3/-_!~()$@*"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/second2/third3/-_!~()$@**"#, Err(StatusCode::BAD_REQUEST)),
            (r#"/first1/second2/third3/-_!~()$@/"#, Ok(())),
            (r#"/first1/second2/third3/-_!~()$@/*"#, Ok(())),
            (r#"/first1/second2/third3/-_!~()$@/**"#, Ok(())),
        ] {
            assert_eq!(validator.validate(channel), result, "{channel}");
        }
    }
}
