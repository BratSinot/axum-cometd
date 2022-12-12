use crate::utils::ChannelNameValidator;
use test_common::TEST_SUBSCRIBE_CHANNEL_NAMES;

#[test]
fn test_subscribe_channel_name_validator() {
    let validator = ChannelNameValidator::default();

    for (channel, result) in TEST_SUBSCRIBE_CHANNEL_NAMES {
        assert_eq!(
            validator
                .validate_subscribe_channel_name(channel, ())
                .ok()
                .is_some(),
            result,
            "{channel}"
        );
    }
}
