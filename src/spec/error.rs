use derive_more::{Display, Error, From};
use semver::{Error as SemVerError, Version};

use crate::spec::{r#ref::RefError, schema::Error as SchemaError};

/// Spec Errors
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[display(fmt = "Reference error")]
    Ref(RefError),

    #[display(fmt = "Schema error")]
    Schema(SchemaError),

    #[display(fmt = "Semver error")]
    SemVerError(SemVerError),

    #[display(fmt = "Unsupported spec file version ({})", _0)]
    UnsupportedSpecFileVersion(#[error(not(source))] Version),
}
