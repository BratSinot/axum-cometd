use crate::utils::ChannelNameValidator;

#[test]
fn test_channel_name_validator() {
    let validator = ChannelNameValidator::default();

    for (channel, result) in [
        // 1 nest level
        (r#"/first1"#, true),
        (r#"/first1*"#, false),
        (r#"/first1**"#, false),
        (r#"/first1/"#, true),
        (r#"/first1/*"#, true),
        (r#"/first1/ *"#, false),
        (r#"/first1/**"#, true),
        (r#"/first1/ **"#, false),
        // 2 nest levels
        (r#"/first1/second2"#, true),
        (r#"/first1/second2*"#, false),
        (r#"/first1/second2**"#, false),
        (r#"/first1/second2/"#, true),
        (r#"/first1/second2/*"#, true),
        (r#"/first1/second2/ *"#, false),
        (r#"/first1/second2/**"#, true),
        (r#"/first1/second2/ **"#, false),
        // 3 nest levels
        (r#"/first1/second2/third3"#, true),
        (r#"/first1/second2/third3*"#, false),
        (r#"/first1/second2/third3**"#, false),
        (r#"/first1/second2/third3/"#, true),
        (r#"/first1/second2/third3/*"#, true),
        (r#"/first1/second2/third3/ *"#, false),
        (r#"/first1/second2/third3/**"#, true),
        (r#"/first1/second2/third3/ **"#, false),
        // 4 nest levels + special chars
        (r#"/first1/second2/third3/-_!~()$@"#, true),
        (r#"/first1/second2/third3/-_!~()$@*"#, false),
        (r#"/first1/second2/third3/-_!~()$@**"#, false),
        (r#"/first1/second2/third3/-_!~()$@/"#, true),
        (r#"/first1/second2/third3/-_!~()$@/*"#, true),
        (r#"/first1/second2/third3/-_!~()$@/ *"#, false),
        (r#"/first1/second2/third3/-_!~()$@/**"#, true),
        (r#"/first1/second2/third3/-_!~()$@/ **"#, false),
        // use wild characters in the middle
        (r#"/first1/*/third3"#, false),
        (r#"/first1/*/third3/"#, false),
        (r#"/first1/*/third3/*"#, false),
        (r#"/first1/*/third3/**"#, false),
        (r#"/first1/second2/**/"#, false),
        (r#"/first1/second2/**/*"#, false),
        (r#"/first1/second2/**/**"#, false),
    ] {
        assert_eq!(validator.validate(channel), result, "{channel}");
    }
}
