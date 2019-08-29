/// Schema Errors
#[derive(Debug, Clone, PartialEq, Error)]
pub enum Error {
    #[error(display = "Missing type property")]
    NoType,

    #[error(display = "Required fields specified on a non-object schema")]
    RequiredSpecifiedOnNonObject,
}
