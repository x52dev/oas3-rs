//! Data validation (JSON only)

use crate::Spec;

mod error;
mod solo;

pub use error::*;
pub use solo::*;

pub trait Validate {
    fn validate(&self, spec: &Spec) -> Result<(), Error>;
}
