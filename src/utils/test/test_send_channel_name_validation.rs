use crate::utils::ChannelNameValidator;
use test_common::TEST_SEND_CHANNEL_NAMES;

#[test]
fn test_send_channel_name_validator() {
    let validator = ChannelNameValidator::default();

    for (channel, result) in TEST_SEND_CHANNEL_NAMES {
        assert_eq!(
            validator
                .validate_send_channel_name(channel, ())
                .ok()
                .is_some(),
            result,
            "{channel}"
        );
    }
}
