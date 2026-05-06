use oas3::{
    spec::{ObjectOrReference, ObjectSchema, Schema},
    Spec,
};

fn resolved_object_schema(schema: Schema, spec: &Spec) -> ObjectSchema {
    let schema = schema.resolve(spec).unwrap();

    let Schema::Object(schema) = schema else {
        panic!("expected schema to resolve to an object schema");
    };

    let ObjectOrReference::Object(schema) = *schema else {
        unreachable!("Schema::resolve() should resolve outer schema references");
    };

    schema
}

#[test]
fn issue_52() {
    use assert_matches::assert_matches;
    use oas3::spec;

    let spec = oas3::from_yaml(include_str!("../issues/issue-52.yaml")).unwrap();

    let op = spec.operation_by_id("any").unwrap();

    let schema = op.responses(&spec)["200"].content["application/json"]
        .schema
        .clone()
        .unwrap();

    let schema = resolved_object_schema(schema, &spec)
        .additional_properties
        .unwrap();

    assert_matches!(schema, spec::Schema::Boolean(spec::BooleanSchema(true)));

    let op = spec.operation_by_id("none").unwrap();

    let schema = op.responses(&spec)["200"].content["application/json"]
        .schema
        .clone()
        .unwrap();

    let schema = resolved_object_schema(schema, &spec)
        .additional_properties
        .unwrap();

    assert_matches!(schema, spec::Schema::Boolean(spec::BooleanSchema(false)));
}

#[test]
fn issue_79() {
    let spec = oas3::from_yaml(include_str!("../issues/issue-79.yaml")).unwrap();

    let op = spec.operation_by_id("listClientIdsWithSize").unwrap();

    let param = op.parameter("sortBy", &spec).unwrap().unwrap();
    let schema = resolved_object_schema(param.schema.unwrap(), &spec);

    assert_eq!(schema.title.as_deref(), Some("foo"));
}

#[test]
fn issue_318() {
    use assert_matches::assert_matches;
    use oas3::spec::RefError;

    let spec = oas3::from_yaml(include_str!("../issues/issue-318.yaml")).unwrap();

    let property = resolved_object_schema(
        spec.components.as_ref().unwrap().schemas["PetDetails"].clone(),
        &spec,
    )
    .properties["petDetailsId"]
        .clone();

    let err = property
        .resolve(&spec)
        .expect_err("malformed $ref should return an error");

    assert_matches!(
        err,
        RefError::Unresolvable(path)
            if path == "/components/schemas/petdetails#pet_details_id"
    );
}

#[test]
fn boolean_schemas_in_schema_bearing_fields() {
    use assert_matches::assert_matches;
    use indoc::indoc;
    use oas3::spec::{BooleanSchema, Schema};

    let spec = oas3::from_yaml(indoc! {r#"
        openapi: 3.1.0
        info:
          title: Test API
          version: "0.1"
        paths:
          /test:
            get:
              operationId: getTest
              parameters:
                - name: allow
                  in: query
                  schema: true
              responses:
                "200":
                  description: ok
                  headers:
                    X-Never:
                      schema: false
                  content:
                    application/json:
                      schema: false
        components:
          schemas:
            Anything: true
            Nothing: false
    "#})
    .unwrap();

    let schemas = &spec.components.as_ref().unwrap().schemas;
    assert_matches!(
        schemas["Anything"].resolve(&spec),
        Ok(Schema::Boolean(BooleanSchema(true)))
    );
    assert_matches!(
        schemas["Nothing"].resolve(&spec),
        Ok(Schema::Boolean(BooleanSchema(false)))
    );

    let op = spec.operation_by_id("getTest").unwrap();
    let param = op.parameter("allow", &spec).unwrap().unwrap();
    assert_matches!(
        param.schema.unwrap().resolve(&spec),
        Ok(Schema::Boolean(BooleanSchema(true)))
    );

    let response = &op.responses(&spec)["200"];
    let header = response.headers["X-Never"].resolve(&spec).unwrap();
    assert_matches!(
        header.schema.unwrap().resolve(&spec),
        Ok(Schema::Boolean(BooleanSchema(false)))
    );

    let media_type = &response.content["application/json"];
    assert_matches!(
        media_type.schema(&spec),
        Ok(Some(Schema::Boolean(BooleanSchema(false))))
    );
}
