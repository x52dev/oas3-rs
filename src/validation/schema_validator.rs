use std::{collections::BTreeMap, ops::Deref};

use lazy_static::lazy_static;
use log::{debug, trace, warn};
use serde_json::Value;

use crate::{Schema, Spec};

#[derive(Debug, Clone)]
pub enum SchemaType {
    Boolean,
    Integer,
    Number,
    String,
    Array(Box<SchemaValidator>),
    Object(BTreeMap<String, SchemaValidator>),
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct SchemaValidator {
    pub schema_type: SchemaType,
    pub nullable: bool,
}

impl SchemaValidator {
    pub fn require(typ: SchemaType) -> SchemaValidator {
        SchemaValidator {
            schema_type: typ,
            nullable: false,
        }
    }

    pub fn nullable(typ: SchemaType) -> SchemaValidator {
        SchemaValidator {
            schema_type: typ,
            nullable: true,
        }
    }

    pub fn from_schema(schema: &Schema, spec: &Spec) -> SchemaValidator {
        let schema_type = match &schema.schema_type.as_ref().expect("no schema type")[..] {
            "boolean" => SchemaType::Boolean,
            "integer" => SchemaType::Integer,
            "number" => SchemaType::Number,
            "string" => SchemaType::String,

            "array" => {
                let item_schema = schema
                    .items
                    .as_ref()
                    .expect("items MUST be present if the type is array");

                let item_schema = item_schema.resolve(&spec).expect("$ref unresolvable");

                SchemaType::Array(Box::new(SchemaValidator::from_schema(&item_schema, spec)))
            }

            "object" => {
                let mut prop_schemas = BTreeMap::new();

                for (key, oor) in schema.properties.iter() {
                    let schema = oor.resolve(&spec).expect("$ref unresolvable");
                    prop_schemas.insert(key.to_owned(), schema.validator(&spec));
                }

                SchemaType::Object(prop_schemas)
            }

            typ => SchemaType::Unknown(typ.to_owned()),
        };

        SchemaValidator {
            schema_type,
            nullable: schema.nullable.unwrap_or(false),
        }
    }

    pub fn validate_type(&self, val: &Value) -> Result<(), String> {
        if self.nullable && val.is_null() {
            return Ok(());
        }

        match self.schema_type {
            SchemaType::Boolean => match val {
                Value::Bool(_) => Ok(()),
                val => Err(format!("{:?} is not bool", val)),
            },

            SchemaType::Integer => match val {
                Value::Number(num) if num.is_i64() => Ok(()),
                val => Err(format!("{:?} is not integer", val)),
            },

            SchemaType::Number => match val {
                Value::Number(_) => Ok(()),
                val => Err(format!("{:?} is not number", val)),
            },

            SchemaType::String => match val {
                Value::String(_) => Ok(()),
                val => Err(format!("{:?} is not string", val)),
            },

            SchemaType::Array(ref item_validator) => match val {
                Value::Array(items) => {
                    if items
                        .iter()
                        .all(|item| item_validator.validate_type(&item).is_ok())
                    {
                        Ok(())
                    } else {
                        Err(format!(
                            "some items of the array do not match the sub-schema"
                        ))
                    }
                }

                val => Err(format!("{:?} is not array", val)),
            },

            SchemaType::Object(ref prop_validators) => match val {
                Value::Object(props) => {
                    for (key, val) in props {
                        debug!("checking {}", &key);

                        if let Some(ref vltr) = prop_validators.get(key.deref()) {
                            let _ = vltr.validate_type(&val)?;
                        } else {
                            return Err(format!("extraneous property on object: {}", &key));
                        }
                    }

                    Ok(())
                }

                val => Err(format!("{:?} is not object", val)),
            },

            SchemaType::Unknown(ref typ) => {
                warn!(
                    "Cannot validate unknown type `{}`. Validations will assume passing.",
                    &typ
                );
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use maplit::btreemap;
    use pretty_assertions::assert_eq;
    use serde_json::{json, Number, Value};

    use super::*;

    lazy_static! {
        // primitives
        static ref NULL: Value = json!(null);
        static ref TRU: Value = json!(true);
        static ref FALS: Value = json!(false);
        static ref INTEGER: Value = json!(1);
        static ref FLOAT: Value = json!(1.1);
        static ref STRING: Value = json!("im a string");

        // arrays
        static ref ARRAY_INTS: Value = json!([1, 2]);
        static ref ARRAY_STRS: Value = json!(["one", "two"]);

        // objects
        static ref OBJ_EMPTY: Value = json!({});
        static ref OBJ_NUMS: Value = json!({ "low": 1.1, "high": 1.5 });
        static ref OBJ_MIXED: Value = json!({ "name": "milk", "price": 1.2 });
    }

    macro_rules! type_check_valid_vs_invalid {
        ($validator:expr, $valid:expr, $invalid:expr,) => {{
            let valid: &[&Value] = $valid;
            let invalid: &[&Value] = $invalid;

            for item in valid {
                trace!("should be Ok {:?}", &item);
                assert!($validator.validate_type(&item).is_ok())
            }

            for item in invalid {
                trace!("should be Err {:?}", &item);
                assert!($validator.validate_type(&item).is_err())
            }
        }};

        ($validator:expr, $valid:expr, $invalid:expr) => {{
            type_check_valid_vs_invalid!($validator, $valid, $invalid,)
        }};
    }

    #[test]
    fn type_check_bool() {
        let vltr = SchemaValidator::require(SchemaType::Boolean);

        type_check_valid_vs_invalid!(
            vltr,
            &[&TRU],
            &[&NULL, &INTEGER, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn type_check_integer() {
        let vltr = SchemaValidator::require(SchemaType::Integer);

        type_check_valid_vs_invalid!(
            vltr,
            &[&INTEGER],
            &[&FLOAT, &NULL, &TRU, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn type_check_number() {
        let vltr = SchemaValidator::require(SchemaType::Number);

        type_check_valid_vs_invalid!(
            vltr,
            &[&INTEGER, &FLOAT],
            &[&NULL, &TRU, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn type_check_string() {
        let vltr = SchemaValidator::require(SchemaType::String);

        type_check_valid_vs_invalid!(
            vltr,
            &[&STRING],
            &[&NULL, &TRU, &INTEGER, &FLOAT, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn type_check_nullable() {
        let vltr = SchemaValidator::nullable(SchemaType::Boolean);

        type_check_valid_vs_invalid!(
            vltr,
            &[&TRU, &NULL],
            &[&FLOAT, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn type_check_array() {
        let vltr_int = SchemaValidator::require(SchemaType::Integer);
        let vltr = SchemaValidator::require(SchemaType::Array(Box::new(vltr_int)));

        type_check_valid_vs_invalid!(
            vltr,
            &[&ARRAY_INTS],
            &[&ARRAY_STRS, &TRU, &NULL, &FLOAT, &STRING, &OBJ_EMPTY],
        );
    }

    #[test]
    fn type_check_object() {
        let validators = btreemap! {
            "low".to_owned() => SchemaValidator::require(SchemaType::Number),
            "high".to_owned() => SchemaValidator::require(SchemaType::Number),
        };
        let vltr = SchemaValidator::require(SchemaType::Object(validators));

        type_check_valid_vs_invalid!(
            vltr,
            &[&OBJ_NUMS, &OBJ_EMPTY],
            &[&OBJ_MIXED, &NULL, &INTEGER, &FLOAT, &STRING, &ARRAY_INTS],
        );

        let validators = btreemap! {
            "low".to_owned() => SchemaValidator::require(SchemaType::Number),
        };
        let vltr = SchemaValidator::require(SchemaType::Object(validators));

        type_check_valid_vs_invalid!(
            vltr,
            &[&OBJ_EMPTY],
            &[&OBJ_NUMS, &OBJ_MIXED, &NULL, &INTEGER, &STRING, &ARRAY_INTS],
        );

        let validators = btreemap! {
            "name".to_owned() => SchemaValidator::require(SchemaType::String),
            "price".to_owned() => SchemaValidator::require(SchemaType::Number),
        };
        let vltr = SchemaValidator::require(SchemaType::Object(validators));

        type_check_valid_vs_invalid!(
            vltr,
            &[&OBJ_MIXED, &OBJ_EMPTY],
            &[&OBJ_NUMS, &NULL, &INTEGER, &FLOAT, &STRING, &ARRAY_INTS],
        );
    }
}
