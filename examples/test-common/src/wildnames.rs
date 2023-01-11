pub const TEST_WILDNAMES: [(&str, Option<&[&str]>); 42] = [
    // 0 nest level
    (r#""#, None),
    (r#"/*"#, Some(&[])),
    (r#"/**"#, Some(&[])),
    // 1 nest level
    ("/first1", Some(&["/*", "/**"])),
    ("/first1*", None),
    ("/first1**", None),
    ("/first1/", Some(&["/first1/*", "/first1/**", "/**"])),
    ("/first1/*", Some(&[])),
    ("/first1/ *", None),
    ("/first1/**", Some(&[])),
    ("/first1/ **", None),
    // 2 nest levels
    ("/first1/second2", Some(&["/first1/*", "/first1/**", "/**"])),
    ("/first1/second2*", None),
    ("/first1/second2**", None),
    (
        "/first1/second2/",
        Some(&[
            "/first1/second2/*",
            "/first1/second2/**",
            "/first1/**",
            "/**",
        ]),
    ),
    ("/first1/second2/*", Some(&[])),
    ("/first1/second2/ *", None),
    ("/first1/second2/**", Some(&[])),
    ("/first1/second2/ **", None),
    // 3 nest levels
    (
        "/first1/second2/third3",
        Some(&[
            "/first1/second2/*",
            "/first1/second2/**",
            "/first1/**",
            "/**",
        ]),
    ),
    ("/first1/second2/third3*", None),
    ("/first1/second2/third3**", None),
    (
        "/first1/second2/third3/",
        Some(&[
            "/first1/second2/third3/*",
            "/first1/second2/third3/**",
            "/first1/second2/**",
            "/first1/**",
            "/**",
        ]),
    ),
    ("/first1/second2/third3/*", Some(&[])),
    ("/first1/second2/third3/ *", None),
    ("/first1/second2/third3/**", Some(&[])),
    ("/first1/second2/third3/ **", None),
    // 4 nest levels + special chars
    (
        "/first1/second2/third3/-_!~()$@",
        Some(&[
            "/first1/second2/third3/*",
            "/first1/second2/third3/**",
            "/first1/second2/**",
            "/first1/**",
            "/**",
        ]),
    ),
    ("/first1/second2/third3/-_!~()$@*", None),
    ("/first1/second2/third3/-_!~()$@**", None),
    (
        "/first1/second2/third3/-_!~()$@/",
        Some(&[
            "/first1/second2/third3/-_!~()$@/*",
            "/first1/second2/third3/-_!~()$@/**",
            "/first1/second2/third3/**",
            "/first1/second2/**",
            "/first1/**",
            "/**",
        ]),
    ),
    ("/first1/second2/third3/-_!~()$@/*", Some(&[])),
    ("/first1/second2/third3/-_!~()$@/ *", None),
    ("/first1/second2/third3/-_!~()$@/**", Some(&[])),
    ("/first1/second2/third3/-_!~()$@/ **", None),
    // use wild characters in the middle
    ("/first1/*/third3", None),
    ("/first1/*/third3/", None),
    ("/first1/*/third3/*", None),
    ("/first1/*/third3/**", None),
    ("/first1/second2/**/", None),
    ("/first1/second2/**/*", None),
    ("/first1/second2/**/**", None),
];
