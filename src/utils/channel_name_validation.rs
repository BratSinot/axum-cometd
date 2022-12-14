use regex::Regex;

const VALID_SUBSCRIBE_CHANNEL_NAME_REGEX: &str =
    r#"^(?:/[a-zA-Z0-9\-_!~()$@]+)*(?:(?:/\*\*)|(?:/\*)|/)?$"#;
const VALID_SEND_CHANNEL_NAME_REGEX: &str = r#"^(?:/[a-zA-Z0-9\-_!~()$@]+)+(?:/)?$"#;

#[derive(Debug)]
pub(crate) struct ChannelNameValidator {
    subscribe_channel_name_regex: Regex,
    send_channel_name_regex: Regex,
}

impl Default for ChannelNameValidator {
    #[inline(always)]
    fn default() -> Self {
        #[allow(clippy::expect_used)]
        Self {
            subscribe_channel_name_regex: Regex::new(VALID_SUBSCRIBE_CHANNEL_NAME_REGEX)
                .expect("wrong regex"),
            send_channel_name_regex: Regex::new(VALID_SEND_CHANNEL_NAME_REGEX)
                .expect("wrong regex"),
        }
    }
}

impl ChannelNameValidator {
    #[inline(always)]
    pub(crate) fn validate_subscribe_channel_name<E>(&self, name: &str, error: E) -> Result<(), E> {
        if name.is_empty() {
            Err(error)
        } else {
            self.subscribe_channel_name_regex
                .is_match(name)
                .then_some(())
                .ok_or(error)
        }
    }

    #[inline(always)]
    pub(crate) fn validate_send_channel_name<E>(&self, name: &str, error: E) -> Result<(), E> {
        if name.is_empty() {
            Err(error)
        } else {
            self.send_channel_name_regex
                .is_match(name)
                .then_some(())
                .ok_or(error)
        }
    }
}
