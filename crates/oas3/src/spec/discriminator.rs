//! Schema specification for [OpenAPI 3.1](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md)

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The discriminator is a specific object in a schema which is used to inform the consumer of
/// the document of an alternative schema based on the value associated with it.
///
/// See <https://spec.openapis.org/oas/v3.1.0#discriminator-object>
#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct Discriminator {
    /// The name of the property in the payload that will hold the discriminator value.
    #[serde(rename = "propertyName")]
    pub property_name: String,

    /// An object to hold mappings between payload values and schema names or references
    ///
    /// When using the discriminator, inline schemas will not be considered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping: Option<BTreeMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminator_property_name_parsed_correctly() {
        let spec = "propertyName: testName";
        let discriminator = serde_yml::from_str::<Discriminator>(spec).unwrap();
        assert_eq!("testName", discriminator.property_name);
        assert!(discriminator.mapping.is_none());
    }

    #[test]
    fn discriminator_mapping_parsed_correctly() {
        let spec = indoc::indoc! {"
            propertyName: petType
            mapping: 
              dog: '#/components/schemas/Dog'
              cat: '#/components/schemas/Cat'
              monster: 'https://gigantic-server.com/schemas/Monster/schema.json'
        "};
        let discriminator = serde_yml::from_str::<Discriminator>(spec).unwrap();

        assert_eq!("petType", discriminator.property_name);
        let mapping = discriminator.mapping.unwrap();

        assert_eq!("#/components/schemas/Dog", mapping.get("dog").unwrap());
        assert_eq!("#/components/schemas/Cat", mapping.get("cat").unwrap());
        assert_eq!(
            "https://gigantic-server.com/schemas/Monster/schema.json",
            mapping.get("monster").unwrap()
        );
    }
}
