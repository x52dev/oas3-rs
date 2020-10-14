use serde::{Deserialize, Serialize};
use url::Url;

/// License information for the exposed API.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#licenseObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct License {
    /// The license name used for the API.
    pub name: String,

    /// An SPDX license expression for the API. The identifier field is mutually exclusive of the url field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    /// A URL to the license used for the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specificationExtensions}
}
