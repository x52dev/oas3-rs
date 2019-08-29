use std::{collections::BTreeMap, fmt, ops::Deref};

use lazy_static::lazy_static;
use serde_json::Value as JsonValue;

use crate::{schema::Error as SchemaError, validation::Error, Schema, Spec};

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
    pub required: Vec<String>,
}

impl SchemaValidator {
    pub fn require(typ: SchemaType) -> SchemaValidator {
        SchemaValidator {
            schema_type: typ,
            nullable: false,
            required: vec![],
        }
    }

    pub fn nullable(typ: SchemaType) -> SchemaValidator {
        SchemaValidator {
            schema_type: typ,
            nullable: true,
            required: vec![],
        }
    }

    pub fn with_required_fields(self, fields: Vec<String>) -> Self {
        Self {
            required: fields,
            ..self
        }
    }

    pub fn from_schema(schema: &Schema, spec: &Spec) -> Result<SchemaValidator, SchemaError> {
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

                let item_schema = item_schema.resolve(&spec)?;

                SchemaType::Array(Box::new(SchemaValidator::from_schema(&item_schema, spec)?))
            }

            "object" => {
                let mut prop_schemas = BTreeMap::new();

                for (key, oor) in schema.properties.iter() {
                    let schema = oor.resolve(&spec)?;
                    prop_schemas.insert(key.to_owned(), schema.validator(&spec)?);
                }

                SchemaType::Object(prop_schemas)
            }

            typ => SchemaType::Unknown(typ.to_owned()),
        };

        let kind = schema.schema_type.as_ref().ok_or(SchemaError::NoType)?;
        let required = match &kind[..] {
            "object" => schema.required.clone(),

            _ => {
                if schema.required.is_empty() {
                    vec![]
                } else {
                    return Err(SchemaError::RequiredSpecifiedOnNonObject);
                }
            }
        };

        Ok(SchemaValidator {
            schema_type,
            nullable: schema.nullable.unwrap_or(false),
            required,
        })
    }

    /// Checks that the value provided is of expected type.
    /// Will validate array items and check object properties recursively.
    pub fn validate_type(&self, val: &JsonValue) -> Result<(), Error> {
        if self.nullable && val.is_null() {
            return Ok(());
        }

        match self.schema_type {
            SchemaType::Boolean => match val {
                JsonValue::Bool(_) => Ok(()),
                val => Err(Error::TypeMismatch(val.clone(), "bool")),
            },

            SchemaType::Integer => match val {
                JsonValue::Number(num) if num.is_i64() => Ok(()),
                val => Err(Error::TypeMismatch(val.clone(), "integer")),
            },

            SchemaType::Number => match val {
                JsonValue::Number(_) => Ok(()),
                val => Err(Error::TypeMismatch(val.clone(), "number")),
            },

            SchemaType::String => match val {
                JsonValue::String(_) => Ok(()),
                val => Err(Error::TypeMismatch(val.clone(), "string")),
            },

            SchemaType::Array(ref item_validator) => match val {
                JsonValue::Array(items) => {
                    // search for invalid array item
                    if let Some(item) = items
                        .iter()
                        .find(|item| item_validator.validate_type(&item).is_err())
                    {
                        // an item was invalid
                        let err = item_validator.validate_type(&item).unwrap_err();
                        Err(Error::ArrayItemTypeMismatch(item.to_owned(), Box::new(err)))
                    } else {
                        // all items ok
                        Ok(())
                    }
                }

                val => Err(Error::TypeMismatch(val.clone(), "array")),
            },

            SchemaType::Object(ref prop_validators) => match val {
                JsonValue::Object(props) => {
                    for (key, val) in props {
                        trace!("checking {}", &key);

                        if let Some(ref vltr) = prop_validators.get(key.deref()) {
                            vltr.validate_type(&val)?;
                        } else {
                            return Err(Error::ExtraneousField(key.to_owned()));
                        }
                    }

                    Ok(())
                }

                val => Err(Error::TypeMismatch(val.clone(), "object")),
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

    /// Checks that specified required fields are present on object type.
    pub fn validate_required_fields(&self, val: &JsonValue) -> Result<(), Error> {
        match self.schema_type {
            SchemaType::Object(ref prop_validators) => match val {
                JsonValue::Object(ref map) => {
                    // search for missing fields
                    match self.required.iter().find(|&req| !map.contains_key(req)) {
                        // no missing required fields
                        None => Ok(()),

                        // missing required field
                        Some(field) => Err(Error::RequiredFieldMissing(field.clone())),
                    }
                }

                val => Err(Error::TypeMismatch(val.clone(), "object")),
            },

            _ => match self.required.is_empty() {
                false => Err(Error::Schema(SchemaError::RequiredSpecifiedOnNonObject)),

                // not trying to be an object
                true => Ok(()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use maplit::btreemap;
    use pretty_assertions::assert_eq;
    use serde_json::{json, Number};

    use super::*;

    lazy_static! {
        // primitives
        static ref NULL: JsonValue = json!(null);
        static ref TRU: JsonValue = json!(true);
        static ref FALS: JsonValue = json!(false);
        static ref INTEGER: JsonValue = json!(1);
        static ref FLOAT: JsonValue = json!(1.1);
        static ref STRING: JsonValue = json!("im a string");

        // arrays
        static ref ARRAY_INTS: JsonValue = json!([1, 2]);
        static ref ARRAY_STRS: JsonValue = json!(["one", "two"]);

        // objects
        static ref OBJ_EMPTY: JsonValue = json!({});
        static ref OBJ_NUMS: JsonValue = json!({ "low": 1.1, "high": 1.5 });
        static ref OBJ_MIXED: JsonValue = json!({ "name": "milk", "price": 1.2 });
    }

    mod type_check {
        use super::*;

        macro_rules! type_check_valid_vs_invalid {
            ($validator:expr, $valid:expr, $invalid:expr,) => {{
                let valid: &[&JsonValue] = $valid;
                let invalid: &[&JsonValue] = $invalid;

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
        fn bool() {
            let vltr = SchemaValidator::require(SchemaType::Boolean);

            type_check_valid_vs_invalid!(
                vltr,
                &[&TRU],
                &[&NULL, &INTEGER, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
            );
        }

        #[test]
        fn integer() {
            let vltr = SchemaValidator::require(SchemaType::Integer);

            type_check_valid_vs_invalid!(
                vltr,
                &[&INTEGER],
                &[&FLOAT, &NULL, &TRU, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
            );
        }

        #[test]
        fn number() {
            let vltr = SchemaValidator::require(SchemaType::Number);

            type_check_valid_vs_invalid!(
                vltr,
                &[&INTEGER, &FLOAT],
                &[&NULL, &TRU, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
            );
        }

        #[test]
        fn string() {
            let vltr = SchemaValidator::require(SchemaType::String);

            type_check_valid_vs_invalid!(
                vltr,
                &[&STRING],
                &[&NULL, &TRU, &INTEGER, &FLOAT, &ARRAY_INTS, &OBJ_EMPTY],
            );
        }

        #[test]
        fn nullable() {
            let vltr = SchemaValidator::nullable(SchemaType::Boolean);

            type_check_valid_vs_invalid!(
                vltr,
                &[&TRU, &NULL],
                &[&FLOAT, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
            );
        }

        #[test]
        fn array() {
            let vltr_int = SchemaValidator::require(SchemaType::Integer);
            let vltr = SchemaValidator::require(SchemaType::Array(Box::new(vltr_int)));

            type_check_valid_vs_invalid!(
                vltr,
                &[&ARRAY_INTS],
                &[&ARRAY_STRS, &TRU, &NULL, &FLOAT, &STRING, &OBJ_EMPTY],
            );
        }

        #[test]
        fn object() {
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

    #[test]
    fn check_required_fields() {
        pretty_env_logger::init();

        let required = vec!["low".to_owned()];
        let vltr = SchemaValidator::require(SchemaType::Object(btreemap! {}))
            .with_required_fields(required);

        assert!(vltr.validate_required_fields(&OBJ_NUMS).is_ok());

        assert!(vltr.validate_required_fields(&NULL).is_err());
        assert!(vltr.validate_required_fields(&OBJ_MIXED).is_err());
    }
}
