use serde::{Deserialize, Serialize};

use super::{FromRef, Ref, RefError, RefType, Schema, Spec};

// TODO: update to 3.1 spec including JSON Schema conformance.

/// The Header Object mostly follows the structure of the [Parameter Object].
///
/// Deviations from Parameter Object:
/// 1. `name` MUST NOT be specified, it is given in the corresponding `headers` map.
/// 1. `in` MUST NOT be specified, it is implicitly in `header`.
/// 1. All traits that are affected by the location MUST be applicable to a location of
///    `header` (for example, [`style`]).
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#header-object>.
///
/// [Parameter Object]: https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterObject
/// [`style`]: https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterStyle
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Header {
    // FIXME: Is the third change properly implemented?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "uniqueItems")]
    pub unique_items: Option<bool>,

    /// string, number, boolean, integer, array, file ( only for formData )
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub param_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// A brief description of the parameter. This could contain examples
    /// of use.  GitHub Flavored Markdown is allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // collectionFormat: ???
    // default: ???
    // maximum ?
    // exclusiveMaximum ??
    // minimum ??
    // exclusiveMinimum ??
    // maxLength ??
    // minLength ??
    // pattern ??
    // maxItems ??
    // minItems ??
    // enum ??
    // multipleOf ??
    // allowEmptyValue ( for query / body params )
}

impl FromRef for Header {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError> {
        let refpath = path.parse::<Ref>()?;

        match refpath.kind {
            RefType::Header => spec
                .components
                .as_ref()
                .and_then(|cs| cs.headers.get(&refpath.name))
                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
                .and_then(|oor| oor.resolve(spec)),

            typ => Err(RefError::MismatchedType(typ, RefType::Example)),
        }
    }
}
