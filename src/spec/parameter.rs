use serde::{Deserialize, Serialize};

use super::{FromRef, Ref, RefError, RefType, Spec};
use crate::Schema;

// FIXME: Verify against OpenAPI 3.0.1
/// Describes a single operation parameter.
/// A unique parameter is defined by a combination of a
/// [name](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterName)
/// and [location](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterIn).
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Parameter {
    /// The name of the parameter.
    pub name: String,

    // TODO: enumify
    /// The location of the parameter.
    /// Possible values are "query", "header", "path" or "cookie".
    #[serde(rename = "in")]
    pub location: String,

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
    /// Describes how the parameter value will be serialized depending on the type of the parameter
    /// value. Default values (based on value of in): for `query` - `form`; for `path` - `simple`; for
    /// `header` - `simple`; for cookie - `form`.
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<ParameterStyle>,

    #[serde(skip_serializing_if = "Option::is_none")]
    explode: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "in")]
    param_in: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum ParameterStyle {
    Form,
    Simple,
    DeepObject,
}

impl FromRef for Parameter {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError>
    where
        Self: Sized,
    {
        let refpath = path.parse::<Ref>()?;

        match refpath.kind {
            RefType::Parameter => spec
                .components
                .as_ref()
                .and_then(|cs| cs.parameters.get(&refpath.name))
                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
                .and_then(|oor| oor.resolve(spec)),

            typ => Err(RefError::MismatchedType(typ, RefType::Parameter)),
        }
    }
}
