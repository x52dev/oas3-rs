//! Error types

use std::io;

use derive_more::derive::{Display, Error, From};

use crate::spec::Error as SpecError;
#[cfg(feature = "validation")]
use crate::validation::Error as ValidationError;

/// Top-level errors.
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[display("I/O error")]
    Io(io::Error),

    #[display("YAML error")]
    Yaml(serde_yml::Error),

    #[display("JSON error")]
    Serialize(serde_json::Error),

    #[display("Spec error")]
    Spec(SpecError),

    #[cfg(feature = "validation")]
    #[display("Validation error")]
    Validation(ValidationError),

    #[cfg(feature = "conformance")]
    #[display("Reqwest error")]
    Reqwest(reqwest::Error),
}
