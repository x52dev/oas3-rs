//! Error types

use std::io::Error as IoError;

use failure::Fail;
use semver::{SemVerError, Version};
use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;

use crate::{RefError, validation::Error as ValidationError};

/// errors that openapi functions may return
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Io(IoError),

    #[fail(display = "{}", _0)]
    Yaml(YamlError),

    #[fail(display = "{}", _0)]
    Serialize(JsonError),

    #[fail(display = "{}", _0)]
    SemVerError(SemVerError),
    
    #[fail(display = "Unsupported spec file version ({})", _0)]
    UnsupportedSpecFileVersion(Version),
    
    #[fail(display = "Reference error: {}", _0)]
    Ref(RefError),
    
    #[fail(display = "Validation error: {}", _0)]
    Validation(ValidationError),
    
    #[fail(display = "Placeholder")]
    Placeholder,
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
