use std::{collections::BTreeMap, fmt, ops::Deref};

use serde_json::Value as JsonValue;

use super::{Error, Validate};
use crate::{spec::schema::Error as SchemaError, Schema, Spec};

// impl Validate for Schema {
//     fn validate(&self, spec: &Spec) -> Result<(), Error> {

//     }
// }
