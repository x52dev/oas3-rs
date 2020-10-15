use std::fmt;

use derive_more::{Display, Error};
use http::{Method, StatusCode};
use serde_json::Value as JsonValue;

use super::Path;
use crate::spec::{Error as SchemaError, SchemaType};

#[derive(Debug, Clone, PartialEq)]
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
            .map(|err| format!("  => {}", err.to_string()))
            .collect::<Vec<_>>()
            .join("\n");

        f.write_str(&errs)
    }
}

/// Validation Errors
#[derive(Clone, PartialEq, Debug, Display, Error)]
pub enum Error {
    //
    // Wrapped Errors
    //
    #[display(fmt = "Schema error")]
    Schema(SchemaError),

    //
    // Leaf Errors
    //
    #[display(fmt = "Not JSON")]
    NotJson,

    #[display(fmt = "{} is not a {:?}", _0, _1)]
    TypeMismatch(Path, SchemaType),

    #[display(fmt = "Array item type mismatch: {}", _0)]
    ArrayItemTypeMismatch(JsonValue, #[error(source)] Box<Error>),

    #[display(fmt = "Undocumented field: {}", _0)]
    UndocumentedField(#[error(not(source))] String),

    #[display(fmt = "Status mismatch: expected {}; got {}", _0, _1)]
    StatusMismatch(StatusCode, StatusCode),

    #[display(fmt = "Required field missing: {}", _0)]
    RequiredFieldMissing(#[error(not(source))] Path),

    #[display(fmt = "Type did not match any `anyOf` variant: {}\n{}", _0, _1)]
    OneOfNoMatch(Path, AggregateError),

    #[display(fmt = "Non-nullable field was null: {}", _0)]
    InvalidNull(#[error(not(source))] Path),

    #[display(fmt = "Operation not found: {} {}", _0, _1)]
    OperationNotFound(Method, String),

    #[display(fmt = "Operation ID not found: {}", _0)]
    OperationIdNotFound(#[error(not(source))] String),

    #[display(fmt = "Parameter not found: {}", _0)]
    ParameterNotFound(#[error(not(source))] String),

    #[display(fmt = "Invalid parameter location: {}", _0)]
    InvalidParameterLocation(#[error(not(source))] String),
}
