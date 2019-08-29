//! Error types

use std::io::Error as IoError;

use crate::{spec::Error as SpecError, validation::Error as ValidationError};

/// Top-level Errors
#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "I/O error")]
    Io(#[cause] IoError),

    #[error(display = "Yaml error")]
    Yaml(#[cause] serde_yaml::Error),

    #[error(display = "JSON error")]
    Serialize(#[cause] serde_json::Error),

    #[error(display = "Reqwest error")]
    Reqwest(#[cause] reqwest::Error),

    #[error(display = "Spec error")]
    Spec(#[cause] SpecError),

    #[error(display = "Validation error")]
    Validation(#[cause] ValidationError),
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

impl From<ValidationError> for Error {
    fn from(e: ValidationError) -> Self { Error::Validation(e) }
}

impl From<SpecError> for Error {
    fn from(e: SpecError) -> Self { Error::Spec(e) }
}
