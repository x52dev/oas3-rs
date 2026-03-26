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

    let schema = schema
        .resolve(&spec)
        .unwrap()
        .additional_properties
        .unwrap();

    assert_matches!(schema, spec::Schema::Boolean(spec::BooleanSchema(true)));

    let op = spec.operation_by_id("none").unwrap();

    let schema = op.responses(&spec)["200"].content["application/json"]
        .schema
        .clone()
        .unwrap();

    let schema = schema
        .resolve(&spec)
        .unwrap()
        .additional_properties
        .unwrap();

    assert_matches!(schema, spec::Schema::Boolean(spec::BooleanSchema(false)));
}

#[test]
fn issue_79() {
    let spec = oas3::from_yaml(include_str!("../issues/issue-79.yaml")).unwrap();

    let op = spec.operation_by_id("listClientIdsWithSize").unwrap();

    let param = op.parameter("sortBy", &spec).unwrap().unwrap();
    let schema = param.schema.unwrap().resolve(&spec).unwrap();

    assert_eq!(schema.title.as_deref(), Some("foo"));
}

#[test]
fn issue_318() {
    use assert_matches::assert_matches;
    use oas3::spec::RefError;

    let spec = oas3::from_yaml(include_str!("../issues/issue-318.yaml")).unwrap();

    let property = spec.components.as_ref().unwrap().schemas["PetDetails"]
        .resolve(&spec)
        .unwrap()
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
