use serde_json::Value as JsonValue;

#[derive(Clone, PartialEq, Debug, Error)]
pub enum Error {
    #[error(display = "Unknown error")]
    Unknown,
}
