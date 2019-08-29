use http::{Method, StatusCode};
use serde_json::Value as JsonValue;

use crate::spec::schema::Error as SchemaError;

/// Validation Errors
#[derive(Clone, PartialEq, Debug, Error)]
pub enum Error {
    //
    // Wrapped Errors
    //
    #[error(display = "Schema error")]
    Schema(#[cause] SchemaError),

    //
    // Leaf Errors
    //
    #[error(display = "Not JSON")]
    NotJson,

    #[error(display = "{} is not a {}", _0, _1)]
    TypeMismatch(JsonValue, &'static str),

    #[error(display = "Array item type mismatch: {}", _0)]
    ArrayItemTypeMismatch(JsonValue, #[cause] Box<Error>),

    #[error(display = "Extraneous field: {}", _0)]
    ExtraneousField(String),

    #[error(display = "Status mismatch: expected {}; got {}", _0, _1)]
    StatusMismatch(StatusCode, StatusCode),

    #[error(display = "Required field missing: {}", _0)]
    RequiredFieldMissing(String),

    #[error(display = "Operation not found: {} {}", _0, _1)]
    OperationNotFound(Method, String),

    #[error(display = "Operation ID not found: {}", _0)]
    OperationIdNotFound(String),

    #[error(display = "Parameter not found: {}", _0)]
    ParameterNotFound(String),

    #[error(display = "Invalid parameter location: {}", _0)]
    InvalidParameterLocation(String),
}
