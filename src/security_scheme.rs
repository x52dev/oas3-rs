use crate::Flows;

/// Defines a security scheme that can be used by the operations. Supported schemes are
/// HTTP authentication, an API key (either as a header or as a query parameter),
///OAuth2's common flows (implicit, password, application and access code) as defined
/// in [RFC6749](https://tools.ietf.org/html/rfc6749), and
/// [OpenID Connect Discovery](https://tools.ietf.org/html/draft-ietf-oauth-discovery-06).
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#securitySchemeObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey {
        name: String,
        #[serde(rename = "in")]
        location: String,
    },
    #[serde(rename = "http")]
    Http {
        scheme: String,
        #[serde(rename = "bearerFormat")]
        bearer_format: String,
    },
    #[serde(rename = "oauth2")]
    OAuth2 { flows: Flows },
    #[serde(rename = "openIdConnect")]
    OpenIdConnect {
        #[serde(rename = "openIdConnectUrl")]
        open_id_connect_url: String,
    },
}
