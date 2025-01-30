use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};

use serde::{de, Deserializer, Serializer};

/// Deserializes fields of a map beginning with `x-`.
pub(crate) fn deserialize<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, serde_json::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ExtraFieldsVisitor;

    impl<'de> de::Visitor<'de> for ExtraFieldsVisitor {
        type Value = BTreeMap<String, serde_json::Value>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("extensions")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let mut map = HashMap::<String, serde_json::Value>::new();

            while let Some((key, value)) = access.next_entry()? {
                map.insert(key, value);
            }

            Ok(map
                .into_iter()
                .filter_map(|(key, value)| {
                    key.strip_prefix("x-").map(|key| (key.to_owned(), value))
                })
                .collect())
        }
    }

    deserializer.deserialize_map(ExtraFieldsVisitor)
}

/// Serializes fields of a map prefixed with `x-`.
pub(crate) fn serialize<S>(
    extensions: &BTreeMap<String, serde_json::Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_map(
        extensions
            .iter()
            .map(|(key, value)| (format!("x-{key}"), value)),
    )
}

#[cfg(all(test, feature = "yaml-spec"))]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::Spec;

    #[test]
    fn spec_extensions_deserialize() {
        let spec = indoc::indoc! {"
            openapi: '3.1.0'
            info:
              title: test
              version: v1
            components: {}
            x-bar: true
            qux: true
        "};

        let spec = serde_yaml::from_str::<Spec>(spec).unwrap();
        assert!(spec.components.is_some());
        assert!(!spec.extensions.contains_key("x-bar"));
        assert!(!spec.extensions.contains_key("qux"));
        assert_eq!(spec.extensions.get("bar").unwrap(), true);
    }

    #[test]
    fn spec_extensions_deserialize_with_numeric_yaml_key_nearby() {
        let spec = indoc::indoc! {"
            openapi: '3.1.0'
            info:
              title: test
              version: v1
            components: {}
            42: test numeric key doesn't break it
            x-bar: true
            44: test numeric key doesn't break it
        "};

        let spec = serde_yaml::from_str::<Spec>(spec).unwrap();
        assert!(spec.components.is_some());
        assert!(!spec.extensions.contains_key("x-bar"));
        assert_eq!(spec.extensions.get("bar").unwrap(), true);
    }

    #[test]
    fn spec_extensions_serialize() {
        let spec = indoc::indoc! {"
            openapi: 3.1.0
            info:
              title: test
              version: v1
            components: {}
            x-bar: true
        "};

        let parsed_spec = serde_yaml::from_str::<Spec>(spec).unwrap();
        let round_trip_spec = serde_yaml::to_string(&parsed_spec).unwrap();

        assert_eq!(spec, round_trip_spec);
    }
}
