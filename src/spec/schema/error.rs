use derive_more::{Display, Error, From};

/// Schema Errors
#[derive(Debug, Clone, PartialEq, Display, Error)]
pub enum Error {
    #[display(fmt = "Missing type property")]
    NoType,

    #[display(fmt = "Unknown type: {}", _0)]
    UnknownType(#[error(not(source))] String),

    #[display(fmt = "Required fields specified on a non-object schema")]
    RequiredSpecifiedOnNonObject,
}
