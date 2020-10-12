use std::{collections::BTreeMap, fmt};

use super::{Error, Validate};
use crate::{
    path::Path,
    spec::{Error as SchemaError, SchemaType},
    Schema, Spec,
};

use serde_json::Value as JsonValue;

#[derive(Debug)]
pub enum ValidationBranch {
    Leaf,
    List(Box<ValidationTree>),
    Map(BTreeMap<String, ValidationTree>),
    AllOf(Vec<ValidationTree>),
    OneOf(Vec<ValidationTree>),
    AnyOf(Vec<ValidationTree>),
}

pub struct ValidationTree {
    pub validators: Vec<Box<dyn Validate>>,
    pub branch: ValidationBranch,
}

impl ValidationTree {
    pub fn from_schema(schema: &Schema, spec: &Spec) -> Result<ValidationTree, SchemaError> {
        Ok(ValidationTree {
            validators: vec![],
            branch: ValidationBranch::Leaf,
        })
    }

    fn first_noncomposite_type_is_object(&self) -> bool {
        match &self.branch {
            ValidationBranch::Map(_) => true,
            ValidationBranch::AllOf(vs) => {
                for v in vs {
                    if !v.first_noncomposite_type_is_object() {
                        return false;
                    }
                }

                true
            }
            ValidationBranch::OneOf(_) | ValidationBranch::AnyOf(_) => {
                panic!("TODO: decide if (any|one)Of is allowed as direct composite child of allOf")
            }
            _ => false,
        }
    }

    pub fn validate(&self, val: &JsonValue) -> Result<(), Error> {
        let path = Path::default();

        // validate own valtree level and throw any errors
        for v in &self.validators {
            v.validate(&val, path.clone())?
        }

        // trigger sub-valtrees validation
        match &self.branch {
            ValidationBranch::AllOf(vs) => {
                // val must be an object (TODO: should it be possible
                // to compose numeric validations ?)
                let obj = val
                    .as_object()
                    .ok_or_else(|| Error::TypeMismatch(path.clone(), SchemaType::Object))?;

                for v in vs {
                    // each sub-valtree must be object type
                    if !v.first_noncomposite_type_is_object() {
                        // TODO: error variant
                        panic!("TODO: error composite type is not object-based")
                    }

                    // match this val against each sub-valtree ignoring extraneous
                    // field errors (TODO: this enables false positive cases)
                }

                // error if any self validations

                Ok(())
            }
            ValidationBranch::OneOf(vs) => {
                // match this val against sub-valtrees
                // error if more than one match

                // error if any self validations

                Ok(())
            }
            ValidationBranch::AnyOf(vs) => {
                // match this val against sub-valtrees
                // error if none match

                // error if any self validations

                Ok(())
            }
            ValidationBranch::List(v) => {
                // val must be list
                // check each val list item against sub-valtree

                Ok(())
            }
            ValidationBranch::Map(validator_map) => {
                //

                Ok(())
            }
            ValidationBranch::Leaf => Ok(()),
        }
    }
}

impl fmt::Debug for ValidationTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidationTree")
            .field(
                "validators",
                &format!("[validator list ({} items)]", self.validators.len()),
            )
            .field("branch", &self.branch)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use maplit::btreemap;
    use serde_json::json;

    use super::{super::tests::*, *};
    use crate::validation::{AllOf, RequiredFields};

    #[test]
    fn valtree_single_level_required() {
        let v = RequiredFields::new(vec![s("name")]);
        let vt = ValidationTree {
            validators: vec![Box::new(v)],
            branch: ValidationBranch::Leaf,
        };

        assert!(vt.validate(&OBJ_MIXED).is_ok());
        assert!(vt.validate(&OBJ_NUMS).is_err());
    }

    #[test]
    fn valtree_single_all_of_required() {
        let req1 = RequiredFields::new(vec![s("name")]);
        let req2 = RequiredFields::new(vec![s("price")]);

        let v = AllOf::new(vec![Box::new(req1), Box::new(req2)]);

        let vt = ValidationTree {
            validators: vec![Box::new(v)],
            branch: ValidationBranch::Leaf,
        };

        assert!(vt.validate(&OBJ_MIXED).is_ok());
        assert!(vt.validate(&OBJ_MIXED2).is_ok());

        assert!(vt.validate(&NULL).is_err());
        assert!(vt.validate(&OBJ_EMPTY).is_err());
        assert!(vt.validate(&OBJ_NUMS).is_err());
    }

    #[test]
    fn valtree_check_first_noncomposite_type() {
        let vt = ValidationTree {
            validators: vec![],
            branch: ValidationBranch::Map(btreemap! {
                s("product") => ValidationTree {
                    validators: vec![],
                    branch: ValidationBranch::Leaf,
                }
            }),
        };

        assert!(vt.first_noncomposite_type_is_object());

        let vt = ValidationTree {
            validators: vec![],
            branch: ValidationBranch::Leaf,
        };

        assert!(!vt.first_noncomposite_type_is_object());

        let vt = ValidationTree {
            validators: vec![],
            branch: ValidationBranch::List(Box::new(ValidationTree {
                validators: vec![],
                branch: ValidationBranch::Leaf,
            })),
        };

        assert!(!vt.first_noncomposite_type_is_object());
    }

    #[test]
    fn valtree_multi_required() {
        let multi = json!({
            "product": OBJ_MIXED.clone()
        });

        let vt = ValidationTree {
            validators: vec![Box::new(RequiredFields::new(vec![s("product")]))],
            branch: ValidationBranch::Leaf,
        };

        assert!(vt.validate(&multi).is_ok());

        assert!(vt.validate(&NULL).is_err());
        assert!(vt.validate(&OBJ_EMPTY).is_err());
        assert!(vt.validate(&OBJ_NUMS).is_err());
    }
}
