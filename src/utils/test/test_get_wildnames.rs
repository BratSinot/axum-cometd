use crate::utils::{get_wild_names, ChannelNameValidator};
use test_common::TEST_WILDNAMES;

#[test]
fn test_get_wildnames() {
    let validator = ChannelNameValidator::default();

    for (channel, result) in TEST_WILDNAMES.iter() {
        let ret = validator
            .validate_subscribe_channel_name(channel)
            .then(|| get_wild_names(channel));

        match (ret, *result) {
            (Some(ret), Some(result)) => assert_eq!(ret, result, "{channel}"),
            (Some(ret), None) => assert_eq!(Some(ret), None, "{channel}"),
            (None, result) => assert_eq!(None, result, "{channel}"),
        }
    }
}
