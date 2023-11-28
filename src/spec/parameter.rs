use serde::{Deserialize, Serialize};

use super::{FromRef, Ref, RefError, RefType, Spec};
use crate::Schema;
use serde::ser::SerializeStruct;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ParamLoc {
    #[default]
    Query,
    Header,
    Path,
    Cookie,
}

// FIXME: Verify against OpenAPI 3.0.1
/// Describes a single operation parameter.
/// A unique parameter is defined by a combination of a
/// [name](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterName)
/// and [location](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterIn).
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterObject>.
#[derive(Clone, Debug, Deserialize, PartialEq, Default)]
pub struct Parameter {
    pub name: String,

    #[serde(rename = "in")]
    pub location: ParamLoc,

    /// A brief description of the parameter. This could contain examples
    /// of use.  GitHub Flavored Markdown is allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    /// Specifies that a parameter is deprecated and SHOULD be transitioned out of usage.
    /// Default value is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub deprecated: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<ParameterStyle>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,

//    #[serde(skip_serializing_if = "Option::is_none")]
//    #[serde(rename = "uniqueItems")]
//    pub unique_items: Option<bool>,
//
//    /// string, number, boolean, integer, array, file ( only for formData )
//    #[serde(skip_serializing_if = "Option::is_none")]
//    #[serde(rename = "type")]
//    pub param_type: Option<String>,
//
//    #[serde(skip_serializing_if = "Option::is_none")]
//    pub format: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum ParameterStyle {
    Form,
    Simple,
    Matrix,
    Label,
    SpaceDelimited,
    PipeDelimited,
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

impl Serialize for Parameter {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Parameter", 10)?;

        s.serialize_field("name", &self.name)?;
        s.serialize_field("in", &self.location)?;
        s.serialize_field("description", &self.description)?;
        s.serialize_field("required", &self.required)?;
        s.serialize_field("deprecated", &self.deprecated)?;
        s.serialize_field("schema", &self.schema)?;
        //s.serialize_field("uniqueItems", &self.unique_items)?;
        //s.serialize_field("type", &self.param_type)?;
        //s.serialize_field("format", &self.format)?;

        if let Some(style) = &self.style {
            s.serialize_field("style", style)?;
        } else {
            match self.location {
                ParamLoc::Query | ParamLoc::Cookie => {
                    s.serialize_field("style", &ParameterStyle::Form)?;
                }
                ParamLoc::Path | ParamLoc::Header => {
                    s.serialize_field("style", &ParameterStyle::Simple)?;
                }
            }
        }

        s.end()
    }
}
