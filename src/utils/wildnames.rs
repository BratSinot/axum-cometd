pub(crate) fn get_wild_names(name: &str) -> Vec<String> {
    let mut segments = name.split('/');
    let last_segment = segments.next_back();

    if !name.is_empty() && !matches!(last_segment, Some("*") | Some("**")) {
        let len = count_wildnames(name);

        let mut ret = Vec::with_capacity(len);
        let mut wild = String::with_capacity(name.len());

        for segment in segments {
            wild.push_str(segment);
            wild.push('/');
            ret.insert(0, format!("{wild}**"));
        }
        ret.insert(0, format!("{wild}*"));

        ret
    } else {
        Vec::with_capacity(0)
    }
}

#[inline(always)]
fn count_wildnames(name: &str) -> usize {
    name.split('/').count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_common::TEST_WILDNAMES;

    #[test]
    fn test_count_wildnames() {
        for (name, result) in TEST_WILDNAMES {
            if let Ok(result) = result {
                if !name.ends_with("/*") && !name.ends_with("/**") {
                    assert_eq!(count_wildnames(name), result.len(), "{name}");
                }
            }
        }
    }
}
