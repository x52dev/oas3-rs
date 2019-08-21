//! Error types

use std::io::Error as IoError;

use semver::{SemVerError, Version};

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
    Yaml(#[cause] serde_yaml::Error),

    #[error(display = "JSON error")]
    Serialize(#[cause] serde_json::Error),

    #[error(display = "Reqwest error")]
    Reqwest(#[cause] reqwest::Error),

    #[error(display = "Semver error")]
    SemVerError(#[cause] SemVerError),

    // TODO: remove and make a subset of schema errors
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

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self { Error::Yaml(e) }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self { Error::Serialize(e) }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self { Error::Reqwest(e) }
}

impl From<SemVerError> for Error {
    fn from(e: SemVerError) -> Self { Error::SemVerError(e) }
}

impl From<ValidationError> for Error {
    fn from(e: ValidationError) -> Self { Error::Validation(e) }
}
