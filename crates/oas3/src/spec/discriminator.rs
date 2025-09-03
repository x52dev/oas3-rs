//! Schema specification for [OpenAPI 3.1](https://spec.openapis.org/oas/v3.1.1)

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A discriminator object can be used to aid in serialization, deserialization, and validation when
/// payloads may be one of a number of different schemas.
///
/// The discriminator is a specific object in a schema which is used to inform the consumer of the
/// document of an alternative schema based on the value associated with it.
///
/// See <https://spec.openapis.org/oas/v3.1.1#discriminator-object>.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Discriminator {
    /// Name of the property in the payload that will hold the discriminator value.
    pub property_name: String,

    /// Object to hold mappings between payload values and schema names or references.
    ///
    /// When using the discriminator, inline schemas will not be considered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping: Option<BTreeMap<String, String>>,
}

impl Discriminator {
    /// Creates a new discriminator with the given property name.
    pub fn new(property_name: impl Into<String>) -> Self {
        Self {
            property_name: property_name.into(),
            mapping: None,
        }
    }

    /// Creates a new discriminator with property name and mappings.
    pub fn with_mapping(
        property_name: impl Into<String>,
        mapping: BTreeMap<String, String>,
    ) -> Self {
        Self {
            property_name: property_name.into(),
            mapping: Some(mapping),
        }
    }

    /// Adds a mapping from a discriminator value to a schema reference.
    pub fn add_mapping(&mut self, value: impl Into<String>, schema_ref: impl Into<String>) {
        let mappings = self.mapping.get_or_insert_with(BTreeMap::new);
        mappings.insert(value.into(), schema_ref.into());
    }

    /// Returns the schema reference for a given discriminator value.
    ///
    /// If a mapping exists for the value, it returns the mapped reference.
    /// Otherwise, it returns None.
    pub fn get_schema_ref(&self, value: &str) -> Option<&str> {
        self.mapping
            .as_ref()
            .and_then(|m| m.get(value))
            .map(|s| s.as_str())
    }

    /// Returns true if this discriminator has any mappings defined.
    pub fn has_mappings(&self) -> bool {
        self.mapping.as_ref().is_some_and(|m| !m.is_empty())
    }

    /// Validates that the discriminator property name is not empty.
    pub fn validate(&self) -> Result<(), DiscriminatorError> {
        if self.property_name.is_empty() {
            return Err(DiscriminatorError::EmptyPropertyName);
        }
        Ok(())
    }
}

/// Errors that can occur when working with discriminators.
#[derive(Debug, Clone, PartialEq)]
pub enum DiscriminatorError {
    /// The property name is empty.
    EmptyPropertyName,
}

#[cfg(all(test, feature = "yaml-spec"))]
mod tests {
    use super::*;

    #[test]
    fn discriminator_property_name_parsed_correctly() {
        let spec = "propertyName: testName";
        let discriminator = serde_yaml::from_str::<Discriminator>(spec).unwrap();
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
        let discriminator = serde_yaml::from_str::<Discriminator>(spec).unwrap();

        assert_eq!("petType", discriminator.property_name);
        let mapping = discriminator.mapping.unwrap();

        assert_eq!("#/components/schemas/Dog", mapping.get("dog").unwrap());
        assert_eq!("#/components/schemas/Cat", mapping.get("cat").unwrap());
        assert_eq!(
            "https://gigantic-server.com/schemas/Monster/schema.json",
            mapping.get("monster").unwrap()
        );
    }

    #[test]
    fn discriminator_new() {
        let disc = Discriminator::new("type");
        assert_eq!(disc.property_name, "type");
        assert!(disc.mapping.is_none());
    }

    #[test]
    fn discriminator_with_mapping() {
        let mut mapping = BTreeMap::new();
        mapping.insert(
            "physical".into(),
            "#/components/schemas/PhysicalProduct".into(),
        );
        mapping.insert(
            "digital".into(),
            "#/components/schemas/DigitalProduct".into(),
        );

        let disc = Discriminator::with_mapping("productType", mapping);
        assert_eq!(disc.property_name, "productType");
        assert!(disc.mapping.is_some());
        assert_eq!(disc.mapping.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn discriminator_add_mapping() {
        let mut disc = Discriminator::new("type");
        disc.add_mapping("cat", "#/components/schemas/Cat");
        disc.add_mapping("dog", "#/components/schemas/Dog");

        assert!(disc.has_mappings());
        assert_eq!(disc.get_schema_ref("cat"), Some("#/components/schemas/Cat"));
        assert_eq!(disc.get_schema_ref("dog"), Some("#/components/schemas/Dog"));
        assert_eq!(disc.get_schema_ref("bird"), None);
    }

    #[test]
    fn discriminator_get_schema_ref() {
        let mut mapping = BTreeMap::new();
        mapping.insert(
            "service".into(),
            "#/components/schemas/ServiceProduct".into(),
        );

        let disc = Discriminator::with_mapping("type", mapping);
        assert_eq!(
            disc.get_schema_ref("service"),
            Some("#/components/schemas/ServiceProduct")
        );
        assert_eq!(disc.get_schema_ref("unknown"), None);
    }

    #[test]
    fn discriminator_has_mappings() {
        let disc1 = Discriminator::new("type");
        assert!(!disc1.has_mappings());

        let mut disc2 = Discriminator::new("type");
        disc2.add_mapping("key", "value");
        assert!(disc2.has_mappings());
    }

    #[test]
    fn discriminator_validation() {
        let valid_disc = Discriminator::new("type");
        assert!(valid_disc.validate().is_ok());

        let invalid_disc = Discriminator::new("");
        assert_eq!(
            invalid_disc.validate(),
            Err(DiscriminatorError::EmptyPropertyName)
        );
    }

    #[test]
    fn product_discriminator_example() {
        let spec = indoc::indoc! {"
            propertyName: productType
            mapping:
              physical: '#/components/schemas/PhysicalProduct'
              digital: '#/components/schemas/DigitalProduct'
              service: '#/components/schemas/ServiceProduct'
        "};
        let discriminator = serde_yaml::from_str::<Discriminator>(spec).unwrap();

        assert_eq!("productType", discriminator.property_name);
        assert!(discriminator.validate().is_ok());
        assert!(discriminator.has_mappings());

        assert_eq!(
            discriminator.get_schema_ref("physical"),
            Some("#/components/schemas/PhysicalProduct")
        );
        assert_eq!(
            discriminator.get_schema_ref("digital"),
            Some("#/components/schemas/DigitalProduct")
        );
        assert_eq!(
            discriminator.get_schema_ref("service"),
            Some("#/components/schemas/ServiceProduct")
        );
    }
}
