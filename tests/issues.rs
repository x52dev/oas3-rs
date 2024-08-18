use assert_matches::assert_matches;
use oas3::spec::{self, BooleanSchema};

#[test]
fn issue_52() {
    let spec = oas3::from_str(include_str!("issues/issue-52.yaml")).unwrap();

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

    assert_matches!(schema, spec::Schema::Boolean(BooleanSchema(true)));

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

    assert_matches!(schema, spec::Schema::Boolean(BooleanSchema(false)));
}

#[test]
fn issue_79() {
    let spec = oas3::from_str(include_str!("issues/issue-79.yaml")).unwrap();

    let op = spec.operation_by_id("listClientIdsWithSize").unwrap();

    let param = op.parameter("sortBy", &spec).unwrap().unwrap();
    let schema = param.schema.unwrap().resolve(&spec).unwrap();

    assert_eq!(schema.title.as_deref(), Some("foo"));
}
