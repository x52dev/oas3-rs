use serde::{Deserialize, Serialize};
use url::Url;

/// Allows referencing an external resource for extended documentation.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#externalDocumentationObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ExternalDoc {
    /// The URL for the target documentation.
    pub url: Url,

    /// A short description of the target documentation.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specificationExtensions}
}
