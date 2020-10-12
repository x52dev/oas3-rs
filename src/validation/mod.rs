//! Data validation (JSON only)

use std::fmt::Debug;

use serde_json::Value as JsonValue;

use crate::{path::Path, Spec};

#[cfg(test)]
#[macro_use]
mod test_macros;

mod error;
pub use error::*;

mod validator;
pub use validator::*;

mod allof;
mod required;
mod r#type;

pub use allof::*;
pub use r#type::*;
pub use required::*;

pub trait Validate: Debug {
    fn validate(&self, val: &JsonValue, path: Path) -> Result<(), Error>;
}

#[cfg(test)]
pub mod tests {
    use lazy_static::lazy_static;
    use serde_json::json;

    use super::*;

    pub fn s(s: &str) -> String {
        s.to_owned()
    }

    lazy_static! {
        // primitives
        pub static ref NULL: JsonValue = json!(null);
        pub static ref TRUE: JsonValue = json!(true);
        pub static ref FALSE: JsonValue = json!(false);
        pub static ref INTEGER: JsonValue = json!(1);
        pub static ref FLOAT: JsonValue = json!(1.1);
        pub static ref STRING: JsonValue = json!("im a string");

        // arrays
        pub static ref ARRAY_INTS: JsonValue = json!([1, 2]);
        pub static ref ARRAY_STRS: JsonValue = json!(["one", "two"]);
        pub static ref ARRAY_MIXED: JsonValue = json!(["one", 2]);

        // objects
        pub static ref OBJ_EMPTY: JsonValue = json!({});
        pub static ref OBJ_NUMS: JsonValue = json!({ "quantity": 83, "price": 1.5 });
        pub static ref OBJ_MIXED: JsonValue = json!({ "name": "milk", "price": 1.2 });
        pub static ref OBJ_MIXED2: JsonValue = json!({ "name": "milk", "quantity": 32, "price": 1.2 });
    }
}
