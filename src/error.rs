//! Error types

use std::io::Error as IoError;

use semver::{SemVerError, Version};
use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;

use crate::{validation::Error as ValidationError, RefError};

/// Top-level Errors
#[derive(Debug, Error)]
pub enum Error {
    //
    // Wrapped Errors
    //
    #[error(display = "I/O error")]
    Io(#[cause] IoError),

    #[error(display = "Yaml error")]
    Yaml(#[cause] YamlError),

    #[error(display = "JSON error")]
    Serialize(#[cause] JsonError),

    #[error(display = "Semver error")]
    SemVerError(#[cause] SemVerError),

    // TODO: subset of schema errors
    #[error(display = "Reference error")]
    Ref(#[cause] RefError),

    #[error(display = "Validation error")]
    Validation(#[cause] ValidationError),

    //
    // Leaf Errors
    //
    #[error(display = "Unsupported spec file version ({})", _0)]
    UnsupportedSpecFileVersion(Version),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self { Error::Io(e) }
}

impl From<YamlError> for Error {
    fn from(e: YamlError) -> Self { Error::Yaml(e) }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self { Error::Serialize(e) }
}

impl From<SemVerError> for Error {
    fn from(e: SemVerError) -> Self { Error::SemVerError(e) }
}

impl From<ValidationError> for Error {
    fn from(e: ValidationError) -> Self { Error::Validation(e) }
}
