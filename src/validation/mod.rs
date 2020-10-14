//! Data validation (JSON only)

use std::fmt::Debug;

use serde_json::Value as JsonValue;

use crate::{path::Path};

#[cfg(test)]
#[macro_use]
mod test_macros;

mod error;
pub use error::*;

mod validator;
pub use validator::*;

mod required;
mod r#type;

pub use r#type::*;
pub use required::*;

pub trait Validate: Debug {
    fn validate(&self, val: &JsonValue, path: Path) -> Result<(), Error>;
}

#[cfg(test)]
pub mod tests {
    use serde_json::json;

    use super::*;

    pub fn s(s: &str) -> String {
        s.to_owned()
    }

    // primitives
    pub const NULL: JsonValue = json!(null);
    pub const TRUE: JsonValue = json!(true);
    pub const FALSE: JsonValue = json!(false);
    pub const INTEGER: JsonValue = json!(1);
    pub const FLOAT: JsonValue = json!(1.1);
    pub const STRING: JsonValue = json!("im a string");

    // arrays
    pub const ARRAY_INTS: JsonValue = json!([1, 2]);
    pub const ARRAY_STRS: JsonValue = json!(["one", "two"]);
    pub const ARRAY_MIXED: JsonValue = json!(["one", 2]);

    // objects
    pub const OBJ_EMPTY: JsonValue = json!({});
    pub const OBJ_NUMS: JsonValue = json!({ "quantity": 83, "price": 1.5 });
    pub const OBJ_MIXED: JsonValue = json!({ "name": "milk", "price": 1.2 });
    pub const OBJ_MIXED2: JsonValue = json!({ "name": "milk", "quantity": 32, "price": 1.2 });
}
