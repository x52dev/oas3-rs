use std::{collections::BTreeMap, fmt};

use super::{AggregateError, DataType, Error, Path, RequiredFields, Validate};
use crate::{
    spec::{Error as SchemaError, SchemaType},
    Schema, Spec,
};

use log::trace;
use serde_json::Value as JsonValue;

#[derive(Debug)]
pub enum ValidationBranch {
    Leaf,
    Array(Box<ValidationTree>),
    Object(BTreeMap<String, ValidationTree>),
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
        trace!(
            "creating validation tree from schema: {}",
            &schema.title.as_deref().unwrap_or("_unnamed_")
        );

        let mut valtree = ValidationTree {
            validators: vec![],
            branch: ValidationBranch::Leaf,
        };

        if let Some(type_) = schema.schema_type {
            trace!("restricting data type: {:?}", type_);

            let type_val = if let Some(nullable) = schema.nullable {
                DataType::new(type_).set_nullable(nullable)
            } else {
                DataType::new(type_)
            };

            valtree.validators.push(Box::new(type_val));
        }

        match schema.schema_type {
            Some(SchemaType::Object) => {
                trace!(
                    "adding object validators: props {}",
                    schema
                        .properties
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(",")
                );

                let vls = schema
                    .properties
                    .iter()
                    .map(|(prop, schema)| {
                        let sub_schema = schema.resolve(&spec).unwrap();
                        let valtree = ValidationTree::from_schema(&sub_schema, &spec).unwrap();
                        (prop.clone(), valtree)
                    })
                    .collect();

                valtree.branch = ValidationBranch::Object(vls);

                if !schema.required.is_empty() {
                    trace!("required fields: {:?}", &schema.required);

                    let req_fields = RequiredFields::new(schema.required.clone());
                    valtree.validators.push(Box::new(req_fields));
                }
            }

            Some(SchemaType::Array) => {
                trace!("adding array validators");

                if let Some(schema_ref) = schema.items.as_ref() {
                    let sub_schema = schema_ref.resolve(&spec).unwrap();
                    let vls = ValidationTree::from_schema(&sub_schema, &spec).unwrap();

                    valtree.branch = ValidationBranch::Array(Box::new(vls))
                }
            }

            Some(_) => {}

            None => {
                // allOf without a `type: object` declaration
                if !schema.all_of.is_empty() {
                    let vs = schema
                        .all_of
                        .iter()
                        .map(|schema_ref| schema_ref.resolve(&spec).unwrap())
                        .map(|schema| ValidationTree::from_schema(&schema, spec).unwrap())
                        .collect();

                    valtree.branch = ValidationBranch::AllOf(vs)
                }

                // anyOf without a `type: object` declaration
                if !schema.any_of.is_empty() {
                    let vs = schema
                        .any_of
                        .iter()
                        .map(|schema_ref| schema_ref.resolve(&spec).unwrap())
                        .map(|schema| ValidationTree::from_schema(&schema, spec).unwrap())
                        .collect();

                    valtree.branch = ValidationBranch::AnyOf(vs)
                }

                // oneOf without a `type: object` declaration
                if !schema.one_of.is_empty() {
                    let vs = schema
                        .one_of
                        .iter()
                        .map(|schema_ref| schema_ref.resolve(&spec).unwrap())
                        .map(|schema| ValidationTree::from_schema(&schema, spec).unwrap())
                        .collect();

                    valtree.branch = ValidationBranch::OneOf(vs)
                }
            }
        }

        Ok(valtree)
    }

    #[allow(dead_code)]
    fn first_noncomposite_type_is_object(&self) -> bool {
        match &self.branch {
            ValidationBranch::Object(_) => true,
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

    /// top level validation entry-point
    pub fn validate(&self, val: &JsonValue) -> Result<(), Error> {
        let path = Path::new('.');
        self.validate_inner(val, path)
    }

    /// trigger sub-valtrees validation
    fn validate_inner(&self, val: &JsonValue, path: Path) -> Result<(), Error> {
        match &self.branch {
            ValidationBranch::AllOf(vs) => {
                // TODO: error if any self validations

                // it's arguable if this should be an error, it may be okay to have an
                // allOf that's not an object, and even so the validation error will
                // show that there is an problem with the schema
                //
                // val must be an object (TODO: should it be possible
                // to compose numeric validations ?)
                // let obj = val
                //     .as_object()
                //     .ok_or_else(|| Error::TypeMismatch(path.clone(), SchemaType::Object))?;

                for v in vs {
                    // ~each sub-valtree must be object type~
                    // if !v.first_noncomposite_type_is_object() {
                    //     // TODO: error variant
                    //     panic!("TODO: error composite type is not object-based")
                    // }

                    // match this val against each sub-valtree ignoring extraneous
                    // field errors (TODO: this enables false positive cases)

                    match v.validate_inner(val, path.clone()) {
                        // TODO: in allOf schemas extraneous fields should be evaluated as a whole
                        Ok(_) | Err(Error::UndocumentedField(_)) => continue,
                        Err(err) => return Err(err),
                    }
                }

                Ok(())
            }

            // TODO: implement subtle differences in anyOf and oneOf
            ValidationBranch::OneOf(vs) | ValidationBranch::AnyOf(vs) => {
                // TODO: error if any self validations

                // match this val against sub-valtrees
                // error if more than one match

                let mut matched = false;
                let mut errors = AggregateError::empty();

                for v in vs {
                    match v.validate_inner(val, path.clone()) {
                        Ok(_) => {
                            matched = true;
                            break;
                        }
                        Err(err) => errors.push(err),
                    }
                }

                if matched {
                    Ok(())
                } else {
                    Err(Error::OneOfNoMatch(path, errors))
                }
            }

            ValidationBranch::Array(v) => {
                // validate own valtree level and throw any errors
                for v in &self.validators {
                    v.validate(&val, path.clone())?
                }

                match val {
                    JsonValue::Array(items) => {
                        for (i, item) in items.iter().enumerate() {
                            let child_path = path.extend(format!("[{}]", i));
                            v.validate_inner(item, child_path)?;
                        }
                    }
                    _ => return Err(Error::TypeMismatch(path, SchemaType::Array)),
                }

                Ok(())
            }

            ValidationBranch::Object(validator_map) => {
                // validate own valtree level and throw any errors
                for v in &self.validators {
                    v.validate(&val, path.clone())?
                }

                match val {
                    JsonValue::Object(items) => {
                        for (prop, val) in items {
                            let child_path = path.extend(prop);

                            if let Some(validator) = validator_map.get(prop) {
                                validator.validate_inner(val, child_path)?;
                            } else {
                                return Err(Error::UndocumentedField(child_path.to_string()));
                            }
                        }
                    }
                    _ => return Err(Error::TypeMismatch(path, SchemaType::Object)),
                }

                Ok(())
            }

            ValidationBranch::Leaf => {
                // validate own valtree level and throw any errors
                for v in &self.validators {
                    v.validate(&val, path.clone())?
                }

                Ok(())
            }
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
    use crate::validation::RequiredFields;

    fn get_schema(spec: &Spec, name: &str) -> Schema {
        spec.components
            .as_ref()
            .unwrap()
            .schemas
            .get(name)
            .unwrap()
            .resolve(&spec)
            .unwrap()
    }

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
    fn valtree_check_first_noncomposite_type() {
        let vt = ValidationTree {
            validators: vec![],
            branch: ValidationBranch::Object(btreemap! {
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
            branch: ValidationBranch::Array(Box::new(ValidationTree {
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

    #[test]
    fn object_from_schema() {
        let spec_str = r#"openapi: "3"
paths: {}
info:
  title: Test API
  version: "0.1"
components:
  schemas:
    data:
      title: Data
      type: object
      properties:
        size: { title: 'Data Sizes', type: integer }
        thing: { title: 'Data Things', type: string }
      required: [size]
"#;

        let spec = crate::from_reader(spec_str.as_bytes()).unwrap();

        let schema = get_schema(&spec, "data");
        let valtree = ValidationTree::from_schema(&schema, &spec).unwrap();
        assert_eq!(valtree.validators.len(), 2);
        assert!(matches!(valtree.branch, ValidationBranch::Object(_)));

        let test = json!({ "size": 123 });
        valtree.validate(&test).unwrap();

        let test = json!({ "size": 123, "thing": "qwerty" });
        valtree.validate(&test).unwrap();

        let test = json!({ "thing": "qwerty" });
        valtree.validate(&test).unwrap_err();

        let test = json!({ "size": "qwerty" });
        valtree.validate(&test).unwrap_err();

        let test = json!({ "size": 123, "other": "what" });
        valtree.validate(&test).unwrap_err();
    }

    #[test]
    fn array_from_schema() {
        let spec_str = r#"openapi: "3"
paths: {}
info:
  title: Test API
  version: "0.1"
components:
  schemas:
    assets:
      title: Assets
      type: array
      items: { type: integer }
"#;

        let spec = crate::from_reader(spec_str.as_bytes()).unwrap();

        let schema = get_schema(&spec, "assets");
        let valtree = ValidationTree::from_schema(&schema, &spec).unwrap();
        assert_eq!(valtree.validators.len(), 1);
        assert!(matches!(valtree.branch, ValidationBranch::Array(_)));

        let test = json!([123, 456]);
        valtree.validate(&test).unwrap();
    }

    #[test]
    fn all_of_from_schema() {
        let spec_str = r#"openapi: "3"
paths: {}
info:
  title: Test API
  version: "0.1"
components:
  schemas:
    data:
      title: Full Data
      allOf:
      - { $ref: '#/components/schemas/size' }
      - { $ref: '#/components/schemas/meta' }
    meta:
      title: Metadata
      type: object
      properties:
        meta: { type: string }
      required: [meta]
    size:
      title: Data Sizes
      type: object
      properties:
        size: { type: integer }
      required: [size]
"#;

        let spec = crate::from_reader(spec_str.as_bytes()).unwrap();

        let schema = get_schema(&spec, "data");
        let valtree = ValidationTree::from_schema(&schema, &spec).unwrap();
        assert_eq!(valtree.validators.len(), 0);
        assert!(matches!(valtree.branch, ValidationBranch::AllOf(_)));

        let test = json!({ "meta": "meta", "size": 123 });
        valtree.validate(&test).unwrap();
    }

    #[test]
    fn any_of_from_schema() {
        let spec_str = r#"openapi: "3"
paths: {}
info:
  title: Test API
  version: "0.1"
components:
  schemas:
    data:
      title: Data
      anyOf: [{ type: number }, { type: string }]
    list:
      title: Data List
      type: array
      items:
        anyOf: [{ type: number }, { type: string }]
"#;

        let spec = crate::from_reader(spec_str.as_bytes()).unwrap();

        let schema = get_schema(&spec, "data");
        let valtree = ValidationTree::from_schema(&schema, &spec).unwrap();
        assert_eq!(valtree.validators.len(), 0);
        assert!(matches!(valtree.branch, ValidationBranch::AnyOf(_)));

        let test = json!("123");
        valtree.validate(&test).unwrap();

        let test = json!(123);
        valtree.validate(&test).unwrap();

        valtree.validate(&NULL).unwrap_err();

        let schema = get_schema(&spec, "list");
        let valtree = ValidationTree::from_schema(&schema, &spec).unwrap();
        assert_eq!(valtree.validators.len(), 1);
        assert!(matches!(valtree.branch, ValidationBranch::Array(_)));

        let test = json!(["123", "456"]);
        valtree.validate(&test).unwrap();

        let test = json!([123, "456", 789]);
        valtree.validate(&test).unwrap();

        let test = json!([123, null, 789]);
        valtree.validate(&test).unwrap_err();
    }
}
