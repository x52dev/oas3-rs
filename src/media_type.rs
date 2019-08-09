use std::collections::BTreeMap;

use crate::{Encoding, Example, ObjectOrReference, Schema, Spec};

/// Each Media Type Object provides schema and examples for the media type identified by its key.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#media-type-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct MediaType {
    /// The schema defining the type used for the request body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<ObjectOrReference<Schema>>,

    /// Example of the media type.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub examples: Option<MediaTypeExample>,

    /// A map between a property name and its encoding information. The key, being the
    /// property name, MUST exist in the schema as a property. The encoding object SHALL
    /// only apply to `requestBody` objects when the media type is `multipart`
    /// or `application/x-www-form-urlencoded`.
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub encoding: BTreeMap<String, Encoding>,
}

impl MediaType {
    pub fn get_schema(&self, spec: &Spec) -> Option<Schema> {
        self.schema.as_ref().unwrap().resolve(&spec)
    }

    pub fn get_examples(&self, spec: &Spec) -> BTreeMap<String, Example> {
        self.examples.as_ref().unwrap().resolve_all(&spec)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum MediaTypeExample {
    /// Example of the media type. The example object SHOULD be in the correct format as
    /// specified by the media type. The `example` field is mutually exclusive of the
    /// `examples` field. Furthermore, if referencing a `schema` which contains an example,
    /// the `example` value SHALL override the example provided by the schema.
    Example { example: serde_json::Value },

    /// Examples of the media type. Each example object SHOULD match the media type and
    /// specified schema if present. The `examples` field is mutually exclusive of
    /// the `example` field. Furthermore, if referencing a `schema` which contains an
    /// example, the `examples` value SHALL override the example provided by the schema.
    Examples {
        examples: BTreeMap<String, ObjectOrReference<Example>>,
    },
}

impl MediaTypeExample {
    pub fn resolve_all(&self, spec: &Spec) -> BTreeMap<String, Example> {
        match self {
            MediaTypeExample::Example { example } => {
                let example = Example {
                    description: None,
                    summary: None,
                    value: Some(example.clone()),
                };

                let mut map = BTreeMap::new();
                map.insert("default".to_owned(), example);

                map
            }

            MediaTypeExample::Examples { examples } => examples
                .iter()
                .filter_map(|(name, oor)| oor.resolve(&spec).map(|obj| (name.clone(), obj)))
                .collect(),
        }
    }
}
