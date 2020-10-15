use serde_json::Value as JsonValue;

use super::{Error, Path, Validate};
use crate::spec::SchemaType;

#[derive(Debug, Clone)]
pub struct DataType {
    r#type: SchemaType,
    nullable: bool,
}

impl DataType {
    pub fn new(type_: SchemaType) -> Self {
        Self {
            r#type: type_,
            nullable: false,
        }
    }

    pub fn boolean() -> Self {
        Self::new(SchemaType::Boolean)
    }
    pub fn integer() -> Self {
        Self::new(SchemaType::Integer)
    }
    pub fn number() -> Self {
        Self::new(SchemaType::Number)
    }
    pub fn string() -> Self {
        Self::new(SchemaType::String)
    }
    pub fn array() -> Self {
        Self::new(SchemaType::Array)
    }
    pub fn object() -> Self {
        Self::new(SchemaType::Object)
    }

    pub fn set_nullable(self, nullable: bool) -> Self {
        Self { nullable, ..self }
    }

    pub fn nullable(self) -> Self {
        Self {
            nullable: true,
            ..self
        }
    }
}

impl Validate for DataType {
    /// Checks that the value provided is of expected type.
    fn validate(&self, val: &JsonValue, path: Path) -> Result<(), Error> {
        if self.nullable && val.is_null() {
            return Ok(());
        }

        let data_type = match val {
            JsonValue::Bool(_) => SchemaType::Boolean,
            JsonValue::Number(num) => {
                if num.is_i64() {
                    SchemaType::Integer
                } else {
                    SchemaType::Number
                }
            }
            JsonValue::String(_) => SchemaType::String,
            JsonValue::Array(_) => SchemaType::Array,
            JsonValue::Object(_) => SchemaType::Object,

            // already checked and returned
            JsonValue::Null => return Err(Error::InvalidNull(path)),
        };

        // check type equality
        if self.r#type != data_type {
            // integers also count as numbers
            if val.is_i64() && self.r#type == SchemaType::Number {
                return Ok(());
            }

            return Err(Error::TypeMismatch(path, self.r#type));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::*, *};

    #[test]
    fn bool_validation() {
        let v = DataType::boolean();

        valid_vs_invalid!(
            v,
            &[&TRUE],
            &[&NULL, &INTEGER, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn integer_validation() {
        let v = DataType::integer();

        valid_vs_invalid!(
            v,
            &[&INTEGER],
            &[&FLOAT, &NULL, &TRUE, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn number_validation() {
        let v = DataType::number();

        valid_vs_invalid!(
            v,
            &[&INTEGER, &FLOAT],
            &[&NULL, &TRUE, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn string_validation() {
        let v = DataType::string();

        valid_vs_invalid!(
            v,
            &[&STRING],
            &[&NULL, &TRUE, &INTEGER, &FLOAT, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn nullable_validation() {
        let v = DataType::boolean().nullable();

        valid_vs_invalid!(
            v,
            &[&TRUE, &NULL],
            &[&FLOAT, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );

        let v = DataType::string().nullable();

        valid_vs_invalid!(
            v,
            &[&STRING, &NULL],
            &[&FLOAT, &TRUE, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn array_validation() {
        let v = DataType::array();

        valid_vs_invalid!(
            v,
            &[&ARRAY_INTS, &ARRAY_STRS],
            &[&TRUE, &NULL, &FLOAT, &STRING, &OBJ_EMPTY],
        );
    }

    #[test]
    fn object_validation() {
        let v = DataType::object();

        valid_vs_invalid!(
            v,
            &[&OBJ_NUMS, &OBJ_EMPTY, &OBJ_MIXED, &OBJ_MIXED2],
            &[&NULL, &INTEGER, &FLOAT, &STRING, &ARRAY_INTS],
        );
    }
}
