use failure::{Backtrace, Context, Fail};
use serde_json::Value as JsonValue;

#[derive(Clone, PartialEq, Debug, Fail)]
pub enum Error {
    #[fail(display = "Unknown error")]
    Unknown,
}
