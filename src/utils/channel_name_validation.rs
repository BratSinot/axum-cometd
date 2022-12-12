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
    pub(crate) fn validate(&self, name: &str) -> bool {
        self.0.is_match(name)
    }

    #[inline(always)]
    pub(crate) fn validate_error<E>(&self, name: &str, error: E) -> Result<(), E> {
        self.validate(name).then_some(()).ok_or(error)
    }
}
