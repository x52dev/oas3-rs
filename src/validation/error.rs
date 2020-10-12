use derive_more::{Display, Error, From};
use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{
    path::Path,
    spec::{Error as SchemaError, SchemaType},
};

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

    #[display(fmt = "Extraneous field: {}", _0)]
    ExtraneousField(#[error(not(source))] String),

    #[display(fmt = "Status mismatch: expected {}; got {}", _0, _1)]
    StatusMismatch(StatusCode, StatusCode),

    #[display(fmt = "Required field missing: {}", _0)]
    RequiredFieldMissing(#[error(not(source))] Path),

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
