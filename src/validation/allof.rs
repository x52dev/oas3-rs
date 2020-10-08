use serde_json::Value as JsonValue;

use super::{Error, Validate};
use crate::{path::Path, spec::schema::Type as SchemaType, Spec};

pub struct AllOf {
    validators: Vec<Box<dyn Validate>>,
}

impl AllOf {
    pub fn new(vs: Vec<Box<dyn Validate>>) -> Self {
        Self { validators: vs }
    }
}

impl Validate for AllOf {
    fn validate(&self, val: &JsonValue, path: Path) -> Result<(), Error> {
        let obj = val
            .as_object()
            .ok_or_else(|| Error::TypeMismatch(path.clone(), SchemaType::Object))?;

        for v in &self.validators {
            v.validate(val, path.clone())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::*, *};
    use crate::validation::RequiredFields;

    #[test]
    fn allof_validatation() {
        let path = Path::default();

        let req1 = RequiredFields::new(vec!["name".to_owned()]);
        let req2 = RequiredFields::new(vec!["price".to_owned()]);

        let v = AllOf::new(vec![Box::new(req1), Box::new(req2)]);

        valid_vs_invalid!(
            v,
            &[&OBJ_MIXED, &OBJ_MIXED2],
            &[&NULL, &OBJ_EMPTY, &OBJ_NUMS]
        )
    }
}
