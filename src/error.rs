//! Error types

use std::io;

use derive_more::{Display, Error, From};

use crate::spec::Error as SpecError;
#[cfg(feature = "validation")]
use crate::validation::Error as ValidationError;

/// Top-level errors.
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[display(fmt = "I/O error")]
    Io(io::Error),

    #[display(fmt = "Yaml error")]
    Yaml(serde_yaml::Error),

    #[display(fmt = "JSON error")]
    Serialize(serde_json::Error),

    #[display(fmt = "Spec error")]
    Spec(SpecError),

    #[cfg(feature = "validation")]
    #[display(fmt = "Validation error")]
    Validation(ValidationError),

    #[cfg(feature = "conformance")]
    #[display(fmt = "Reqwest error")]
    Reqwest(reqwest::Error),
}
