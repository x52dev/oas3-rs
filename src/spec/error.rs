use semver::{SemVerError, Version};

use crate::{spec::schema::Error as SchemaError, spec::r#ref::RefError};

/// Spec Errors
#[derive(Debug, Clone, PartialEq, Error)]
pub enum Error {
    #[error(display = "Reference error")]
    Ref(#[cause] RefError),

    #[error(display = "Schema error")]
    Schema(#[cause] SchemaError),

    #[error(display = "Semver error")]
    SemVerError(#[cause] SemVerError),

    #[error(display = "Unsupported spec file version ({})", _0)]
    UnsupportedSpecFileVersion(Version),
}

impl From<RefError> for Error {
    fn from(err: RefError) -> Self { Self::Ref(err) }
}

impl From<SemVerError> for Error {
    fn from(e: SemVerError) -> Self { Error::SemVerError(e) }
}

impl From<SchemaError> for Error {
    fn from(err: SchemaError) -> Self { Self::Schema(err) }
}
