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

pub use components::*;
pub use contact::*;
pub use encoding::*;
pub use example::*;
pub use external_doc::*;
pub use flows::*;
pub use header::*;
pub use info::*;
pub use license::*;
pub use link::*;
pub use media_type::*;
pub use media_type_examples::*;
pub use operation::*;
pub use parameter::*;
pub use path_item::*;
pub use r#ref::*;
pub use request_body::*;
pub use response::*;
pub use security_scheme::*;
pub use server::*;
pub use tag::*;

pub use error::Error;
pub use schema::{Error as SchemaError, Schema, Type as SchemaType};

const OPENAPI_SUPPORTED_VERSION_RANGE: &str = "~3";

/// top level document
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Spec {
    /// This string MUST be the [semantic version number](https://semver.org/spec/v2.0.0.html)
    /// of the
    /// [OpenAPI Specification version](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#versions)
    /// that the OpenAPI document uses. The `openapi` field SHOULD be used by tooling
    /// specifications and clients to interpret the OpenAPI document. This is not related to
    /// the API
    /// [`info.version`](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#infoVersion)
    /// string.
    pub openapi: String,

    /// Provides metadata about the API. The metadata MAY be used by tooling as required.
    pub info: Info,

    /// An array of Server Objects, which provide connectivity information to a target server.
    /// If the `servers` property is not provided, or is an empty array, the default value would
    /// be a
    /// [Server Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#serverObject)
    /// with a
    /// [url](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#serverUrl)
    /// value of `/`.
    // FIXME: Provide a default value as specified in documentation instead of `None`.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,

    /// Holds the relative paths to the individual endpoints and their operations. The path is
    /// appended to the URL from the
    /// [`Server Object`](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#serverObject)
    /// in order to construct the full URL. The Paths MAY be empty, due to
    /// [ACL constraints](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#securityFiltering).
    pub paths: BTreeMap<String, PathItem>,

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
    /// [Operation Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#operationObject)
    /// must be declared. The tags that are not declared MAY be organized randomly or
    /// based on the tools' logic. Each tag name in the list MUST be unique.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,

    /// Additional external documentation.
    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDoc>,
    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions}
}

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
        let resource = self.paths.get(path)?;

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

        debug!("num paths: {}", paths.len());

        let ops = paths
            .iter()
            .flat_map(|(path, item)| {
                debug!(
                    "path: {}, methods: {}",
                    path,
                    item.methods().into_iter().collect::<Vec<_>>().len()
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
