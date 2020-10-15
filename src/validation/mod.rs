//! Data validation for JSON structures.

use std::fmt::Debug;

use serde_json::Value as JsonValue;

#[cfg(test)]
#[macro_use]
mod test_macros;

mod error;
mod path;
mod required;
mod r#type;
mod validator;

pub use error::*;
pub use path::Path;
pub use r#type::*;
pub use required::*;
pub use validator::*;

pub trait Validate: Debug {
    fn validate(&self, val: &JsonValue, path: Path) -> Result<(), Error>;
}

#[cfg(test)]
pub mod tests {
    use once_cell::sync::Lazy;
    use serde_json::json;

    use super::*;

    pub fn s(s: &str) -> String {
        s.to_owned()
    }

    // primitives
    pub static NULL: Lazy<JsonValue> = Lazy::new(|| json!(null));
    pub static TRUE: Lazy<JsonValue> = Lazy::new(|| json!(true));
    pub static FALSE: Lazy<JsonValue> = Lazy::new(|| json!(false));
    pub static INTEGER: Lazy<JsonValue> = Lazy::new(|| json!(1));
    pub static FLOAT: Lazy<JsonValue> = Lazy::new(|| json!(1.1));
    pub static STRING: Lazy<JsonValue> = Lazy::new(|| json!("im a string"));

    // arrays
    pub static ARRAY_INTS: Lazy<JsonValue> = Lazy::new(|| json!([1, 2]));
    pub static ARRAY_STRS: Lazy<JsonValue> = Lazy::new(|| json!(["one", "two"]));
    pub static ARRAY_MIXED: Lazy<JsonValue> = Lazy::new(|| json!(["one", 2]));

    // objects
    pub static OBJ_EMPTY: Lazy<JsonValue> = Lazy::new(|| json!({}));
    pub static OBJ_NUMS: Lazy<JsonValue> = Lazy::new(|| json!({ "quantity": 83, "price": 1.5 }));
    pub static OBJ_MIXED: Lazy<JsonValue> = Lazy::new(|| json!({ "name": "milk", "price": 1.2 }));
    pub static OBJ_MIXED2: Lazy<JsonValue> =
        Lazy::new(|| json!({ "name": "milk", "quantity": 32, "price": 1.2 }));
}
