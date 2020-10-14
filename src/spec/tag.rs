use serde::{Deserialize, Serialize};

/// Adds metadata to a single tag that is used by the
/// [Operation Object](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#operationObject).
/// It is not mandatory to have a Tag Object per tag defined in the Operation Object instances.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#tagObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Tag {
    /// The name of the tag.
    pub name: String,

    /// A short description for the tag.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // /// Additional external documentation for this tag.
    // #[serde(default)]
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub external_docs: Vec<ExternalDoc>,

    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specificationExtensions}
}
