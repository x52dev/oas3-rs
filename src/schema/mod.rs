//! Schema specification for [OpenAPI 3.0.0](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.0.md)

use std::collections::BTreeMap;

use crate::{FromRef, ObjectOrReference, RefError, RefPath, RefType, Spec};


#[cfg(feature = "validation")]
use crate::validation::SchemaValidator;

pub mod error;
pub use error::*;

// FIXME: Verify against OpenAPI 3.0
/// The Schema Object allows the definition of input and output data types.
/// These types can be objects, but also primitives and arrays.
/// This object is an extended subset of the
/// [JSON Schema Specification Wright Draft 00](http://json-schema.org/).
/// For more information about the properties, see
/// [JSON Schema Core](https://tools.ietf.org/html/draft-wright-json-schema-00) and
/// [JSON Schema Validation](https://tools.ietf.org/html/draft-wright-json-schema-validation-00).
/// Unless stated otherwise, the property definitions follow the JSON Schema.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#schemaObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Schema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    #[serde(default)]
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ObjectOrReference<Schema>>>,

    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, ObjectOrReference<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "readOnly")]
    pub read_only: Option<bool>,

    // FIXME: Why can this be a "boolean" (as per the spec)? It doesn't make sense. Here it's not.
    /// Value can be boolean or object. Inline or referenced schema MUST be of a
    /// [Schema Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#schemaObject)
    /// and not a standard JSON Schema.
    ///
    /// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#properties>.
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "additionalProperties"
    )]
    pub additional_properties: Option<ObjectOrReference<Box<Schema>>>,

    /// A free-form property to include an example of an instance for this schema.
    /// To represent examples that cannot be naturally represented in JSON or YAML,
    /// a string value can be used to contain the example with escaping where necessary.
    /// NOTE: According to [spec], _Primitive data types in the OAS are based on the
    ///       types supported by the JSON Schema Specification Wright Draft 00._
    ///       This suggest using
    ///       [`serde_json::Value`](https://docs.serde.rs/serde_json/value/enum.Value.html). [spec][https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#data-types]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::value::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    // The following properties are taken directly from the JSON Schema definition and
    // follow the same specifications:
    // multipleOf
    // maximum
    // exclusiveMaximum
    // minimum
    // exclusiveMinimum
    // maxLength
    // minLength
    // pattern (This string SHOULD be a valid regular expression, according to the ECMA 262 regular expression dialect)
    // maxItems
    // minItems
    // uniqueItems
    // maxProperties
    // minProperties
    // required
    // enum

    // The following properties are taken from the JSON Schema definition but their
    // definitions were adjusted to the OpenAPI Specification.
    // - type - Value MUST be a string. Multiple types via an array are not supported.
    // - allOf - Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema.
    // - oneOf - Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema.
    // - anyOf - Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema.
    // - not - Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema.
    // - items - Value MUST be an object and not an array. Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema. `items` MUST be present if the `type` is `array`.
    // - properties - Property definitions MUST be a [Schema Object](#schemaObject) and not a standard JSON Schema (inline or referenced).
    // - additionalProperties - Value can be boolean or object. Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema.
    // - description - [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    // - format - See [Data Type Formats](#dataTypeFormat) for further details. While relying on JSON Schema's defined formats, the OAS offers a few additional predefined formats.
    // - default - The default value represents what would be assumed by the consumer of the input as the value of the schema if one is not provided. Unlike JSON Schema, the value MUST conform to the defined type for the Schema Object defined at the same level. For example, if `type` is `string`, then `default` can be `"foo"` but cannot be `1`.
    /// The default value represents what would be assumed by the consumer of the input as the value
    /// of the schema if one is not provided. Unlike JSON Schema, the value MUST conform to the
    /// defined type for the Schema Object defined at the same level. For example, if type is
    /// `string`, then `default` can be `"foo"` but cannot be `1`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<serde_json::Value>,

    /// Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard
    /// JSON Schema.
    #[serde(default)]
    #[serde(rename = "allOf")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<ObjectOrReference<Schema>>,
}

impl Schema {
    #[cfg(feature = "validation")]
    pub fn validator(&self, spec: &Spec) -> Result<SchemaValidator, Error> {
        SchemaValidator::from_schema(&self, spec)
    }
}

impl FromRef for Schema {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError> {
        let refpath = path.parse::<RefPath>()?;

        match refpath.kind {
            RefType::Schema => spec
                .components
                .as_ref()
                .and_then(|cs| cs.schemas.get(&refpath.name))
                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
                .and_then(|oor| oor.resolve(&spec)),

            typ => Err(RefError::MismatchedType(typ, RefType::Schema)),
        }
    }
}
