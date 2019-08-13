use failure::{Backtrace, Context, Fail};
use serde_json::Value as JsonValue;

#[derive(Clone, PartialEq, Debug, Fail)]
pub enum Error {
    #[fail(display = "{} is not a {}", _0, _1)]
    TypeMismatch(JsonValue, &'static str),

    #[fail(display = "Array item type mismatch: {}", _0)]
    ArrayItemTypeMismatch(JsonValue),

    #[fail(display = "Extraneous field: {}", _0)]
    ExtraneousField(String),

    #[fail(display = "Required field missing: {}", _0)]
    RequiredFieldMissing(String),

    #[fail(display = "schema_error")]
    // TODO: create enum variants
    SchemaError(&'static str),
}
