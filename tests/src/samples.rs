#[test]
fn test_comp_pathitems_yaml() {
    let input = include_str!("../samples/pass/comp_pathitems.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_info_summary_yaml() {
    let input = include_str!("../samples/pass/info_summary.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_license_identifier_yaml() {
    let input = include_str!("../samples/pass/license_identifier.yaml");
    validate_sample(input, Format::Yaml);
}

// ... similar individual tests for other YAML files ...

#[test]
fn test_matrix01_yaml() {
    let input = include_str!("../samples/pass/matrix-01.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_matrix02_json() {
    let input = include_str!("../samples/pass/matrix-02.json");
    validate_sample(input, Format::Json);
}

/// Validate that a given `sample` is being parsed by `oas3` without immediate errors. Panics, if
/// an error is encountered and gives error context in the panic message.
#[cfg(test)]
fn validate_sample(input: &str, format: Format) {
    match format {
        Format::Json => {
            let mut json_deserializer = serde_json::Deserializer::from_str(input);
            let result: Result<oas3::OpenApiV3Spec, _> =
                serde_path_to_error::deserialize(&mut json_deserializer);
            match result {
                Ok(_) => (),
                Err(err) => panic!("{}: {}", err, err.path()),
            };
        }
        Format::Yaml => {
            let yaml_deserializer = serde_yml::Deserializer::from_str(input);
            let result: Result<oas3::OpenApiV3Spec, _> =
                serde_path_to_error::deserialize(yaml_deserializer);
            match result {
                Ok(_) => (),
                Err(err) => panic!("{}: {}", err, err.path()),
            };
        }
    }
}

enum Format {
    Json,
    Yaml,
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
