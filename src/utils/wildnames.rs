use std::collections::VecDeque;

// TODO: ret.insert(0, <...>) => ret.push(<...>)
pub(crate) fn get_wild_names(name: &str) -> VecDeque<String> {
    let mut segments = name.split('/');
    let last_segment = segments.next_back();

    if !name.is_empty() && !matches!(last_segment, Some("*" | "**")) {
        let len = count_wildnames(name);

        let mut ret = VecDeque::with_capacity(len);
        let mut wild = String::with_capacity(name.len());

        for segment in segments {
            wild.push_str(segment);
            wild.push('/');
            ret.push_front(format!("{wild}**"));
        }
        ret.push_front(format!("{wild}*"));

        ret
    } else {
        VecDeque::with_capacity(0)
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
            if let Some(result) = result {
                if !name.ends_with("/*") && !name.ends_with("/**") {
                    assert_eq!(count_wildnames(name), result.len(), "{name}");
                }
            }
        }
    }
}
