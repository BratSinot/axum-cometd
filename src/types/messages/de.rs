use serde::{Deserialize, Deserializer};

#[inline(always)]
pub(super) fn deserialize_subscription<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Data {
        Single(String),
        Array(Vec<String>),
    }

    let optional_data = Option::<Data>::deserialize(deserializer)?;

    Ok(optional_data.map(|data| match data {
        Data::Single(str) => vec![str],
        Data::Array(arr) => arr,
    }))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde::Deserialize;
    use serde_json::{from_value, json};

    #[test]
    fn test_deserialize_subscription() {
        #[derive(Debug, Deserialize)]
        struct Message {
            #[serde(rename = "name")]
            _name: String,
            #[serde(default, deserialize_with = "super::deserialize_subscription")]
            subscription: Option<Vec<String>>,
        }

        assert_eq!(
            from_value::<Message>(json!({"name": "name", "subscription": "str0"}))
                .unwrap()
                .subscription,
            Some(vec!["str0".to_owned()])
        );
        assert_eq!(
            from_value::<Message>(json!({"name": "name", "subscription": ["str0"]}))
                .unwrap()
                .subscription,
            Some(vec!["str0".to_owned()])
        );
        assert_eq!(
            from_value::<Message>(json!({"name": "name", "subscription": ["str0", "str1"]}))
                .unwrap()
                .subscription,
            Some(vec!["str0".to_owned(), "str1".to_owned()])
        );
        assert_eq!(
            from_value::<Message>(json!({"name": "name"}))
                .unwrap()
                .subscription,
            None
        );
    }
}
