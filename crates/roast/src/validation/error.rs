use std::fmt;

use derive_more::derive::{Display, Error};
use http::{Method, StatusCode};
use oas3::spec::{Error as SpecError, SchemaTypeSet};
use serde_json::Value as JsonValue;

use super::Path;

#[derive(Debug)]
pub struct AggregateError {
    errors: Vec<Error>,
}

impl AggregateError {
    pub fn new(errors: Vec<Error>) -> Self {
        Self { errors }
    }

    pub fn empty() -> Self {
        Self { errors: vec![] }
    }

    pub fn push(&mut self, err: Error) {
        self.errors.push(err)
    }
}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let errs = self
            .errors
            .iter()
            .map(|err| format!("  => {}", err))
            .collect::<Vec<_>>()
            .join("\n");

        f.write_str(&errs)
    }
}

/// Validation Errors
#[derive(Debug, Display, Error)]
pub enum Error {
    //
    // Wrapped Errors
    //
    #[display("Spec error")]
    Spec(SpecError),

    //
    // Leaf Errors
    //
    #[display("Not JSON")]
    NotJson,

    #[display("{} is not one of {:?}", _0, _1)]
    TypeMismatch(Path, SchemaTypeSet),

    #[display("Array item type mismatch: {}", _0)]
    ArrayItemTypeMismatch(JsonValue, #[error(source)] Box<Error>),

    #[display("Undocumented field: {}", _0)]
    UndocumentedField(#[error(not(source))] String),

    #[display("Status mismatch: expected {}; got {}", _0, _1)]
    StatusMismatch(StatusCode, StatusCode),

    #[display("Required field missing: {}", _0)]
    RequiredFieldMissing(#[error(not(source))] Path),

    #[display("Type did not match any `anyOf` variant: {}\n{}", _0, _1)]
    OneOfNoMatch(Path, AggregateError),

    #[display("Non-nullable field was null: {}", _0)]
    InvalidNull(#[error(not(source))] Path),

    #[display("Operation not found: {} {}", _0, _1)]
    OperationNotFound(Method, String),

    #[display("Operation ID not found: {}", _0)]
    OperationIdNotFound(#[error(not(source))] String),

    #[display("Parameter not found: {}", _0)]
    ParameterNotFound(#[error(not(source))] String),

    #[display("Invalid parameter location: {}", _0)]
    InvalidParameterLocation(#[error(not(source))] String),
}
