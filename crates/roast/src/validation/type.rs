use serde_json::Value as JsonValue;

use super::{Error, Path, Validate};
use oas3::spec::{SchemaType, SchemaTypeSet};

#[derive(Debug, Clone)]
pub struct DataType {
    type_set: SchemaTypeSet,
    nullable: bool,
}

impl DataType {
    pub fn new(type_set: SchemaTypeSet) -> Self {
        Self {
            type_set,
            nullable: false,
        }
    }

    pub fn boolean() -> Self {
        Self::new(SchemaTypeSet::Single(SchemaType::Boolean))
    }
    pub fn integer() -> Self {
        Self::new(SchemaTypeSet::Single(SchemaType::Integer))
    }
    pub fn number() -> Self {
        Self::new(SchemaTypeSet::Single(SchemaType::Number))
    }
    pub fn string() -> Self {
        Self::new(SchemaTypeSet::Single(SchemaType::String))
    }
    pub fn array() -> Self {
        Self::new(SchemaTypeSet::Single(SchemaType::Array))
    }
    pub fn object() -> Self {
        Self::new(SchemaTypeSet::Single(SchemaType::Object))
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
        if !self.type_set.contains(data_type) {
            // integers also count as numbers
            if val.is_i64() && self.type_set.contains(SchemaType::Number) {
                return Ok(());
            }

            return Err(Error::TypeMismatch(path, self.type_set.clone()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::*, *};

    #[test]
    fn bool_validation() {
        let val = DataType::boolean();

        valid_vs_invalid!(
            val,
            &[&TRUE, &FALSE],
            &[&NULL, &INTEGER, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn integer_validation() {
        let val = DataType::integer();

        valid_vs_invalid!(
            val,
            &[&INTEGER],
            &[&FLOAT, &NULL, &TRUE, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn number_validation() {
        let val = DataType::number();

        valid_vs_invalid!(
            val,
            &[&INTEGER, &FLOAT],
            &[&NULL, &TRUE, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn string_validation() {
        let val = DataType::string();

        valid_vs_invalid!(
            val,
            &[&STRING],
            &[&NULL, &TRUE, &INTEGER, &FLOAT, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn nullable_validation() {
        let val = DataType::boolean().nullable();

        valid_vs_invalid!(
            val,
            &[&TRUE, &NULL],
            &[&FLOAT, &STRING, &ARRAY_INTS, &OBJ_EMPTY],
        );

        let val = DataType::string().nullable();

        valid_vs_invalid!(
            val,
            &[&STRING, &NULL],
            &[&FLOAT, &TRUE, &ARRAY_INTS, &OBJ_EMPTY],
        );
    }

    #[test]
    fn array_validation() {
        let val = DataType::array();

        valid_vs_invalid!(
            val,
            &[&ARRAY_INTS, &ARRAY_STRS],
            &[&TRUE, &NULL, &FLOAT, &STRING, &OBJ_EMPTY],
        );
    }

    #[test]
    fn object_validation() {
        let val = DataType::object();

        valid_vs_invalid!(
            val,
            &[&OBJ_NUMS, &OBJ_EMPTY, &OBJ_MIXED, &OBJ_MIXED2],
            &[&NULL, &INTEGER, &FLOAT, &STRING, &ARRAY_INTS],
        );
    }
}
