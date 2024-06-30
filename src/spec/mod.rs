//! Structures used in parsing and navigating OpenAPI specifications.
//!
//! High-level structures include [`Spec`], [`Components`] & [`Schema`].

use std::{collections::BTreeMap, iter::Iterator};

use derive_more::Error;
use http::Method;
use log::debug;
use serde::{Deserialize, Serialize};

mod components;
mod contact;
mod encoding;

mod error;
mod example;
mod external_doc;
mod flows;
mod header;
mod info;
mod license;
mod link;
mod media_type;
mod media_type_examples;
mod operation;
mod parameter;
mod path_item;
mod r#ref;
mod request_body;
mod response;
mod schema;
mod security_scheme;
mod server;
mod tag;

pub use self::components::*;
pub use self::contact::*;
pub use self::encoding::*;
pub use self::error::Error;
pub use self::example::*;
pub use self::external_doc::*;
pub use self::flows::*;
pub use self::header::*;
pub use self::info::*;
pub use self::license::*;
pub use self::link::*;
pub use self::media_type::*;
pub use self::media_type_examples::*;
pub use self::operation::*;
pub use self::parameter::*;
pub use self::path_item::*;
pub use self::r#ref::*;
pub use self::request_body::*;
pub use self::response::*;
pub use self::schema::{
    Error as SchemaError, Schema, Type as SchemaType, TypeSet as SchemaTypeSet,
};
pub use self::security_scheme::*;
pub use self::server::*;
pub use self::tag::*;

const OPENAPI_SUPPORTED_VERSION_RANGE: &str = "~3";

/// A complete OpenAPI specification.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Spec {
    /// This string MUST be the [semantic version number](https://semver.org/spec/v2.0.0.html)
    /// of the
    /// [OpenAPI Specification version](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#versions)
    /// that the OpenAPI document uses. The `openapi` field SHOULD be used by tooling
    /// specifications and clients to interpret the OpenAPI document. This is not related to
    /// the API
    /// [`info.version`](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#infoVersion)
    /// string.
    pub openapi: String,

    /// Provides metadata about the API. The metadata MAY be used by tooling as required.
    pub info: Info,

    /// An array of Server Objects, which provide connectivity information to a target server.
    /// If the `servers` property is not provided, or is an empty array, the default value would
    /// be a
    /// [Server Object](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#serverObject)
    /// with a
    /// [url](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#serverUrl)
    /// value of `/`.
    // FIXME: Provide a default value as specified in documentation instead of `None`.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,

    /// Holds the relative paths to the individual endpoints and their operations. The path is
    /// appended to the URL from the
    /// [`Server Object`](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#serverObject)
    /// in order to construct the full URL. The Paths MAY be empty, due to
    /// [ACL constraints](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#securityFiltering).
    pub paths: Option<BTreeMap<String, PathItem>>,

    /// An element to hold various schemas for the specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,

    // FIXME: Implement
    // /// A declaration of which security mechanisms can be used across the API.
    // /// The list of  values includes alternative security requirement objects that can be used.
    // /// Only one of the security requirement objects need to be satisfied to authorize a request.
    // /// Individual operations can override this definition.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub security: Option<SecurityRequirement>,
    /// A list of tags used by the specification with additional metadata.
    ///The order of the tags can be used to reflect on their order by the parsing tools.
    /// Not all tags that are used by the
    /// [Operation Object](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#operationObject)
    /// must be declared. The tags that are not declared MAY be organized randomly or
    /// based on the tools' logic. Each tag name in the list MUST be unique.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,

    /// The incoming webhooks that MAY be received as part of this API and that the API consumer MAY
    /// choose to implement. Closely related to the callbacks feature, this section describes
    /// requests initiated other than by an API call, for example by an out of band registration.
    /// The key name is a unique string to refer to each webhook, while the (optionally referenced)
    /// Path Item Object describes a request that may be initiated by the API provider and the
    /// expected responses. An example is available.
    ///
    /// See <>.
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub webhooks: BTreeMap<String, PathItem>,

    /// Additional external documentation.
    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDoc>,
}

// TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specificationExtensions}

impl Spec {
    pub fn validate_version(&self) -> Result<semver::Version, Error> {
        let spec_version = &self.openapi;
        let sem_ver = semver::Version::parse(spec_version)?;
        let required_version = semver::VersionReq::parse(OPENAPI_SUPPORTED_VERSION_RANGE).unwrap();

        if required_version.matches(&sem_ver) {
            Ok(sem_ver)
        } else {
            Err(Error::UnsupportedSpecFileVersion(sem_ver))
        }
    }

    pub fn operation(&self, method: &http::Method, path: &str) -> Option<&Operation> {
        let resource = self.paths.as_ref()?.get(path)?;

        match *method {
            Method::GET => resource.get.as_ref(),
            Method::POST => resource.post.as_ref(),
            Method::PUT => resource.put.as_ref(),
            Method::PATCH => resource.patch.as_ref(),
            Method::DELETE => resource.delete.as_ref(),
            Method::HEAD => resource.head.as_ref(),
            Method::OPTIONS => resource.options.as_ref(),
            Method::TRACE => resource.trace.as_ref(),
            _ => None,
        }
    }

    pub fn operations(&self) -> impl Iterator<Item = (String, Method, &Operation)> {
        let paths = &self.paths;

        debug!(
            "num paths: {}",
            paths.as_ref().map_or(0, |paths| paths.len())
        );

        let ops = paths
            .iter()
            .flatten()
            .flat_map(|(path, item)| {
                debug!(
                    "path: {}, methods: {}",
                    path,
                    item.methods().into_iter().count()
                );

                item.methods()
                    .into_iter()
                    .map(move |(method, op)| (path.to_owned(), method, op))
            })
            .collect::<Vec<_>>();

        debug!("num ops: {}", ops.len());

        ops.into_iter()
    }

    pub fn primary_server(&self) -> Option<&Server> {
        self.servers.first()
    }
}
