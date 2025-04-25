use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    spec_extensions, Example, FromRef, MediaType, ObjectOrReference, ObjectSchema, ParameterStyle,
    Ref, RefError, RefType, Spec,
};

/// Describes a single header for HTTP responses and for individual parts in multipart
/// representations.
///
/// This mostly follows the structure of the [Parameter Object].
///
/// Deviations from Parameter Object:
/// 1. `name` MUST NOT be specified, it is given in the corresponding `headers` map.
/// 1. `in` MUST NOT be specified, it is implicitly in `header`.
/// 1. All traits that are affected by the location MUST be applicable to a location of
///    `header` (for example, [`style`]).
///
/// See <https://spec.openapis.org/oas/v3.1.1#header-object>.
///
/// [Parameter Object]: https://spec.openapis.org/oas/v3.1.1#parameter-object
/// [`style`]: https://spec.openapis.org/oas/v3.1.1#parameterStyle
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    /// A brief description of the header.
    ///
    /// This could contain examples of use. [CommonMark] syntax MAY be used for rich text
    /// representation.
    ///
    /// [CommonMark]: https://spec.commonmark.org
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Determines whether this header is mandatory.
    ///
    /// Default value is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    /// Specifies that a header is deprecated and SHOULD be transitioned out of usage.
    ///
    /// Default value is false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    /// Describes how the header value will be serialized.
    ///
    /// If used, the value must be `simple`.
    // TODO: validation for style requirement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ParameterStyle>,

    /// True if array/object parameter values generate separate parameters for each value of the
    /// array or key-value pair of the map.
    ///
    /// For other types of parameters this property has no effect.
    ///
    /// Default value is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,

    /// The schema defining the type used for the header.
    ///
    /// A header MUST contain either a `schema` property, or a `content` property, but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<ObjectOrReference<ObjectSchema>>,

    /// Example of the header's potential value.
    ///
    /// The example SHOULD match the specified schema and encoding properties if present. The
    /// `example` field is mutually exclusive of the `examples` field. Furthermore, if referencing a
    /// `schema` that contains an example, the `example` value SHALL override the example provided
    /// by the schema. To represent examples of media types that cannot naturally be represented in
    /// JSON or YAML, a string value can contain the example with escaping where necessary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,

    /// Examples of the header's potential value.
    ///
    /// Each example SHOULD contain a value in the correct format as specified in the header
    /// encoding. The `examples` field is mutually exclusive of the `example` field. Furthermore, if
    /// referencing a `schema` that contains an example, the `examples` value SHALL override the
    /// example provided by the schema.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub examples: BTreeMap<String, ObjectOrReference<Example>>,

    /// A map containing the representations for the header.
    ///
    /// A header MUST contain either a `schema` property, or a `content` property, but not both.
    ///
    /// The key is the media type and the value describes it. The map MUST only contain one entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<BTreeMap<String, MediaType>>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.1#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
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
