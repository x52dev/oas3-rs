use crate::ref_path::RefError;

/// Schema Errors
#[derive(Debug, Clone, PartialEq, Error)]
pub enum Error {
    #[error(display = "Reference error")]
    Ref(#[cause] RefError),

    #[error(display = "Missing type property")]
    NoType,

    #[error(display = "Required fields specified on a non-object schema")]
    RequiredSpecifiedOnNonObject,
}

impl From<RefError> for Error {
    fn from(err: RefError) -> Self { Self::Ref(err) }
}
