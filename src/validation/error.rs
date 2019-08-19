use http::{Method, StatusCode};
use failure::{Backtrace, Context, Fail};
use serde_json::Value as JsonValue;

// TODO: split into request and responses error enums

#[derive(Clone, PartialEq, Debug, Fail)]
pub enum Error {
    #[fail(display = "Not JSON")]
    NotJson,
    
    #[fail(display = "{} is not a {}", _0, _1)]
    TypeMismatch(JsonValue, &'static str),

    #[fail(display = "Array item type mismatch: {}", _0)]
    ArrayItemTypeMismatch(JsonValue),

    #[fail(display = "Extraneous field: {}", _0)]
    ExtraneousField(String),
    
    #[fail(display = "Status mismatch: expected {}; got {}", _0, _1)]
    StatusMismatch(StatusCode, StatusCode),

    #[fail(display = "Required field missing: {}", _0)]
    RequiredFieldMissing(String),

    #[fail(display = "Operation not found: {} {}", _0, _1)]
    OperationNotFound(Method, String),
    
    #[fail(display = "Operation ID not found: {}", _0)]
    OperationIdNotFound(String),

    #[fail(display = "schema_error")]
    // TODO: create enum variants
    SchemaError(&'static str),
}
