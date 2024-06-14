//! Schema specification for [OpenAPI 3.0.1](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md)

use std::collections::BTreeMap;

use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

use crate::spec::{FromRef, ObjectOrReference, Ref, RefError, RefType, Spec};

/// Schema Errors
#[derive(Debug, Clone, PartialEq, Display, Error)]
pub enum Error {
    #[display(fmt = "Missing type property")]
    NoType,

    #[display(fmt = "Unknown type: {}", _0)]
    UnknownType(#[error(not(source))] String),

    #[display(fmt = "Required fields specified on a non-object schema")]
    RequiredSpecifiedOnNonObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ObjectOrFalse<T> {
    False(bool),
    Object(T),
}

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
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#schemaObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Schema {
    //
    // display metadata
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    //
    // type
    //
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<Type>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,

    //
    // structure
    //
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ObjectOrReference<Schema>>>,

    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, ObjectOrReference<Schema>>,

    /// Value can be boolean or object. Inline or referenced schema MUST be of a
    /// [Schema Object](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#schemaObject)
    /// and not a standard JSON Schema.
    ///
    /// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#properties>.
    #[serde(rename = "additionalProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Box<ObjectOrFalse<ObjectOrReference<Schema>>>>,

    //
    // additional metadata
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,

    //
    // validation requirements
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    #[serde(default)]
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<serde_yaml::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    #[serde(rename = "multipleOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<serde_json::Number>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<serde_json::Number>,

    #[serde(rename = "exclusiveMaximum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<serde_json::Number>,

    #[serde(rename = "exclusiveMinimum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<bool>,

    #[serde(rename = "minLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u64>,

    #[serde(rename = "maxLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,

    #[serde(rename = "minItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,

    #[serde(rename = "maxItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,

    #[serde(rename = "uniqueItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,

    #[serde(rename = "maxProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<u64>,

    #[serde(rename = "minProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<u64>,

    #[serde(rename = "readOnly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    #[serde(rename = "writeOnly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    //
    // composition
    //
    #[serde(default)]
    #[serde(rename = "allOf")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<ObjectOrReference<Schema>>,

    #[serde(default)]
    #[serde(rename = "oneOf")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub one_of: Vec<ObjectOrReference<Schema>>,

    #[serde(default)]
    #[serde(rename = "anyOf")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub any_of: Vec<ObjectOrReference<Schema>>,
}

impl FromRef for Schema {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError> {
        let refpath = path.parse::<Ref>()?;

        match refpath.kind {
            RefType::Schema => spec
                .components
                .as_ref()
                .and_then(|cs| cs.schemas.get(&refpath.name))
                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
                .and_then(|oor| oor.resolve(spec)),

            typ => Err(RefError::MismatchedType(typ, RefType::Schema)),
        }
    }
}
