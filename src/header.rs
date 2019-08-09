
use crate::Schema;

/// The Header Object follows the structure of the
/// [Parameter Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterObject)
/// with the following changes:
/// 1. `name` MUST NOT be specified, it is given in the corresponding `headers` map.
/// 1. `in` MUST NOT be specified, it is implicitly in `header`.
/// 1. All traits that are affected by the location MUST be applicable to a location of
///    `header` (for example, [`style`](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterStyle)).
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#headerObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Header {
    // FIXME: Is the third change properly implemented?
    // FIXME: Merge `ObjectOrReference<Header>::Reference` and `ParameterOrRef::Reference`
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
