#[test]
fn validate_passing_samples() {
    oas3::from_str(include_str!("samples/pass/comp_pathitems.yaml")).unwrap();
    oas3::from_str(include_str!("samples/pass/info_summary.yaml")).unwrap();
    oas3::from_str(include_str!("samples/pass/license_identifier.yaml")).unwrap();
    oas3::from_str(include_str!("samples/pass/mega.yaml")).unwrap();
    oas3::from_str(include_str!("samples/pass/minimal_comp.yaml")).unwrap();
    oas3::from_str(include_str!("samples/pass/minimal_hooks.yaml")).unwrap();
    oas3::from_str(include_str!("samples/pass/minimal_paths.yaml")).unwrap();
    oas3::from_str(include_str!("samples/pass/path_no_response.yaml")).unwrap();
    oas3::from_str(include_str!("samples/pass/path_var_empty_pathitem.yaml")).unwrap();
    oas3::from_str(include_str!("samples/pass/schema.yaml")).unwrap();
}

#[test]
fn issue_79() {
    let spec = oas3::from_str(include_str!("samples/pass/issue-79.yaml")).unwrap();

    let op = spec
        .operations()
        .find(|(_, _, op)| {
            op.operation_id
                .as_deref()
                .map_or(false, |id| id == "listClientIdsWithSize")
        })
        .unwrap()
        .2;

    let param = op.parameter("sortBy", &spec).unwrap().unwrap();
    let schema = param.schema.unwrap().resolve(&spec).unwrap();

    assert_eq!(schema.title.as_deref(), Some("foo"));
}

#[test]
fn validate_failing_samples() {
    // TODO: implement validation for one-of: [paths, components, webhooks]
    // see https://spec.openapis.org/oas/v3.1.0#openapi-document
    // oas3::from_str(include_str!("samples/fail/no_containers.yaml")).unwrap_err();

    // TODO: implement validation for non-empty server enum
    // oas3::from_str(include_str!("samples/fail/server_enum_empty.yaml")).unwrap_err();

    // TODO: implement validation for server enum references
    // oas3::from_str(include_str!("samples/fail/server_enum_unknown.yaml")).unwrap_err();

    // TODO: reject top-level extensions? find reference for rejection
    // oas3::from_str(include_str!("samples/fail/unknown_container.yaml")).unwrap_err();
}
