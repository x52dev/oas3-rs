//! ROAST: Rust OpenAPI Specification Testing

mod conformance;
mod validation;

// use std::io;

use derive_more::derive::{Display, Error, From};

pub use self::{conformance::*, validation::*};

/// Top-level errors.
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[display("Spec error")]
    Spec(oas3::spec::Error),

    #[display("Validation error")]
    Validation(crate::validation::Error),

    #[display("Reqwest error")]
    Reqwest(reqwest::Error),
}
