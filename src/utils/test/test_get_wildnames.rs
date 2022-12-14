use crate::utils::{get_wild_names, ChannelNameValidator};
use std::ops::Deref;
use test_common::TEST_WILDNAMES;

#[test]
fn test_get_wildnames() {
    let validator = ChannelNameValidator::default();

    let test_wildnames = TEST_WILDNAMES.into_iter().map(|(channel, result)| {
        let result = result.map(|result| {
            result
                .into_iter()
                .map(Deref::deref)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        });
        (channel, result)
    });
    for (channel, result) in test_wildnames {
        assert_eq!(
            validator
                .validate_subscribe_channel_name(channel, ())
                .map(|()| get_wild_names(channel)),
            result,
            "{channel}"
        );
    }
}
