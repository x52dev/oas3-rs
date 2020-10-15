use serde_json::Value as JsonValue;

use super::{Error, Validate, Path};
use crate::{spec::SchemaType};

#[derive(Debug, Clone)]
pub struct RequiredFields {
    fields: Vec<String>,
}

impl RequiredFields {
    pub fn new(vs: Vec<String>) -> Self {
        Self { fields: vs }
    }
}

impl Validate for RequiredFields {
    fn validate(&self, val: &JsonValue, path: Path) -> Result<(), Error> {
        let obj = val
            .as_object()
            .ok_or_else(|| Error::TypeMismatch(path.clone(), SchemaType::Object))?;

        for field in &self.fields {
            let path = path.extend(field);

            if obj.get(&field[..]).is_none() {
                return Err(Error::RequiredFieldMissing(path));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::*, *};

    #[test]
    fn required_fields_validation() {
        let v = RequiredFields::new(vec!["name".to_owned(), "price".to_owned()]);

        valid_vs_invalid!(
            v,
            &[&OBJ_MIXED, &OBJ_MIXED2],
            &[&NULL, &OBJ_EMPTY, &OBJ_NUMS],
        );
    }
}
