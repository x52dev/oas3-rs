use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::spec_extensions;

/// An object representing a Server.
///
/// See <https://spec.openapis.org/oas/v3.1.1#server-object>.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Server {
    /// A URL to the target host.
    ///
    /// This URL supports Server Variables and MAY be relative, to indicate that the host location
    /// is relative to the location where the OpenAPI document is being served. Variable
    /// substitutions will be made when a variable is named in {brackets}.
    pub url: String,

    /// An optional string describing the host designated by the URL.
    ///
    /// CommonMark syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A map between a variable name and its value.
    ///
    /// The value is used for substitution in the server's URL template.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub variables: BTreeMap<String, ServerVariable>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.1#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// An object representing a Server Variable for server URL template substitution.
///
/// See <https://spec.openapis.org/oas/v3.1.1#server-variable-object>.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ServerVariable {
    /// The default value to use for substitution, and to send, if an alternate value is not
    /// supplied.
    ///
    /// Unlike the Schema Object's default, this value MUST be provided by the consumer.
    pub default: String,

    /// An enumeration of string values to be used if the substitution options are from a limited
    /// set.
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub substitutions_enum: Vec<String>,

    /// An optional description for the server variable. [CommonMark] syntax MAY be used for rich
    /// text representation.
    ///
    /// [CommonMark]: https://spec.commonmark.org/
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.1#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Server, ServerVariable};

    #[test]
    fn server_extensions_round_trip() {
        let payload = r#"{
            "url": "https://example.com",
            "x-test": "alpha"
        }"#;

        let server: Server = serde_json::from_str(payload).expect("server parses");
        assert_eq!(server.extensions.get("test"), Some(&json!("alpha")));

        let value = serde_json::to_value(server).expect("server serializes");
        assert_eq!(value.get("x-test"), Some(&json!("alpha")));
    }

    #[test]
    fn server_variable_extensions_round_trip() {
        let payload = r#"{
            "default": "example",
            "x-meta": {"enabled": true}
        }"#;

        let variable: ServerVariable = serde_json::from_str(payload).expect("variable parses");
        assert_eq!(
            variable.extensions.get("meta"),
            Some(&json!({"enabled": true}))
        );

        let value = serde_json::to_value(variable).expect("variable serializes");
        assert_eq!(value.get("x-meta"), Some(&json!({"enabled": true})));
    }
}
