use once_cell::sync::Lazy;

#[allow(clippy::type_complexity)]
pub static TEST_WILDNAMES: Lazy<[(&str, Result<Vec<String>, ()>); 42]> = Lazy::new(|| {
    [
        // 0 nest level
        (r#""#, Err(())),
        (r#"/*"#, Ok(vec![])),
        (r#"/**"#, Ok(vec![])),
        // 1 nest level
        ("/first1", Ok(vec!["/*".to_owned(), "/**".to_owned()])),
        ("/first1*", Err(())),
        ("/first1**", Err(())),
        (
            "/first1/",
            Ok(vec![
                "/first1/*".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/*", Ok(vec![])),
        ("/first1/ *", Err(())),
        ("/first1/**", Ok(vec![])),
        ("/first1/ **", Err(())),
        // 2 nest levels
        (
            "/first1/second2",
            Ok(vec![
                "/first1/*".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2*", Err(())),
        ("/first1/second2**", Err(())),
        (
            "/first1/second2/",
            Ok(vec![
                "/first1/second2/*".to_owned(),
                "/first1/second2/**".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2/*", Ok(vec![])),
        ("/first1/second2/ *", Err(())),
        ("/first1/second2/**", Ok(vec![])),
        ("/first1/second2/ **", Err(())),
        // 3 nest levels
        (
            "/first1/second2/third3",
            Ok(vec![
                "/first1/second2/*".to_owned(),
                "/first1/second2/**".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2/third3*", Err(())),
        ("/first1/second2/third3**", Err(())),
        (
            "/first1/second2/third3/",
            Ok(vec![
                "/first1/second2/third3/*".to_owned(),
                "/first1/second2/third3/**".to_owned(),
                "/first1/second2/**".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2/third3/*", Ok(vec![])),
        ("/first1/second2/third3/ *", Err(())),
        ("/first1/second2/third3/**", Ok(vec![])),
        ("/first1/second2/third3/ **", Err(())),
        // 4 nest levels + special chars
        (
            "/first1/second2/third3/-_!~()$@",
            Ok(vec![
                "/first1/second2/third3/*".to_owned(),
                "/first1/second2/third3/**".to_owned(),
                "/first1/second2/**".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2/third3/-_!~()$@*", Err(())),
        ("/first1/second2/third3/-_!~()$@**", Err(())),
        (
            "/first1/second2/third3/-_!~()$@/",
            Ok(vec![
                "/first1/second2/third3/-_!~()$@/*".to_owned(),
                "/first1/second2/third3/-_!~()$@/**".to_owned(),
                "/first1/second2/third3/**".to_owned(),
                "/first1/second2/**".to_owned(),
                "/first1/**".to_owned(),
                "/**".to_owned(),
            ]),
        ),
        ("/first1/second2/third3/-_!~()$@/*", Ok(vec![])),
        ("/first1/second2/third3/-_!~()$@/ *", Err(())),
        ("/first1/second2/third3/-_!~()$@/**", Ok(vec![])),
        ("/first1/second2/third3/-_!~()$@/ **", Err(())),
        // use wild characters in the middle
        ("/first1/*/third3", Err(())),
        ("/first1/*/third3/", Err(())),
        ("/first1/*/third3/*", Err(())),
        ("/first1/*/third3/**", Err(())),
        ("/first1/second2/**/", Err(())),
        ("/first1/second2/**/*", Err(())),
        ("/first1/second2/**/**", Err(())),
    ]
});
