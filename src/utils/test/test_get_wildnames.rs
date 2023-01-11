use crate::utils::{get_wild_names, ChannelNameValidator};
use test_common::TEST_WILDNAMES;

#[test]
fn test_get_wildnames() {
    let validator = ChannelNameValidator::default();

    for (channel, result) in TEST_WILDNAMES.iter() {
        assert_eq!(
            validator
                .validate_subscribe_channel_name(channel)
                .then(|| get_wild_names(channel)),
            *result,
            "{channel}"
        );
    }
}
