use once_cell::sync::Lazy;

#[allow(clippy::type_complexity)]
pub static TEST_WILDNAMES: Lazy<[(&str, Option<Vec<String>>); 42]> = Lazy::new(|| {
    [
        // 0 nest level
        (r#""#, None),
        (r#"/*"#, Some(vec![])),
        (r#"/**"#, Some(vec![])),
        // 1 nest level
        ("/first1", Some(vec!["/*".to_owned(), "/**".to_owned()])),
        ("/first1*", None),
        ("/first1**", None),
        (
            "/first1/",
            Some(vec![
                "/first1/*".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/*", Some(vec![])),
        ("/first1/ *", None),
        ("/first1/**", Some(vec![])),
        ("/first1/ **", None),
        // 2 nest levels
        (
            "/first1/second2",
            Some(vec![
                "/first1/*".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2*", None),
        ("/first1/second2**", None),
        (
            "/first1/second2/",
            Some(vec![
                "/first1/second2/*".to_owned(),
                "/first1/second2/**".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2/*", Some(vec![])),
        ("/first1/second2/ *", None),
        ("/first1/second2/**", Some(vec![])),
        ("/first1/second2/ **", None),
        // 3 nest levels
        (
            "/first1/second2/third3",
            Some(vec![
                "/first1/second2/*".to_owned(),
                "/first1/second2/**".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2/third3*", None),
        ("/first1/second2/third3**", None),
        (
            "/first1/second2/third3/",
            Some(vec![
                "/first1/second2/third3/*".to_owned(),
                "/first1/second2/third3/**".to_owned(),
                "/first1/second2/**".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2/third3/*", Some(vec![])),
        ("/first1/second2/third3/ *", None),
        ("/first1/second2/third3/**", Some(vec![])),
        ("/first1/second2/third3/ **", None),
        // 4 nest levels + special chars
        (
            "/first1/second2/third3/-_!~()$@",
            Some(vec![
                "/first1/second2/third3/*".to_owned(),
                "/first1/second2/third3/**".to_owned(),
                "/first1/second2/**".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2/third3/-_!~()$@*", None),
        ("/first1/second2/third3/-_!~()$@**", None),
        (
            "/first1/second2/third3/-_!~()$@/",
            Some(vec![
                "/first1/second2/third3/-_!~()$@/*".to_owned(),
                "/first1/second2/third3/-_!~()$@/**".to_owned(),
                "/first1/second2/third3/**".to_owned(),
                "/first1/second2/**".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2/third3/-_!~()$@/*", Some(vec![])),
        ("/first1/second2/third3/-_!~()$@/ *", None),
        ("/first1/second2/third3/-_!~()$@/**", Some(vec![])),
        ("/first1/second2/third3/-_!~()$@/ **", None),
        // use wild characters in the middle
        ("/first1/*/third3", None),
        ("/first1/*/third3/", None),
        ("/first1/*/third3/*", None),
        ("/first1/*/third3/**", None),
        ("/first1/second2/**/", None),
        ("/first1/second2/**/*", None),
        ("/first1/second2/**/**", None),
    ]
});
