//! Error types

use std::io;

use derive_more::{Display, Error, From};
use serde::{Deserialize, Serialize};

use crate::{spec::Error as SpecError, validation::Error as ValidationError};

/// Top-level Errors
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[display(fmt = "I/O error")]
    Io(io::Error),

    #[display(fmt = "Yaml error")]
    Yaml(serde_yaml::Error),

    #[display(fmt = "JSON error")]
    Serialize(serde_json::Error),

    #[display(fmt = "Reqwest error")]
    Reqwest(reqwest::Error),

    #[display(fmt = "Spec error")]
    Spec(SpecError),

    #[display(fmt = "Validation error")]
    Validation(ValidationError),
}
