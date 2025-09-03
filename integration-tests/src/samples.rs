use oas3::Spec;

#[test]
fn callback() {
    let input = include_str!("../samples/pass/callback.yml");
    let spec = validate_sample(input, Format::Yaml);
    let op = spec.operation_by_id("callback").unwrap();
    assert_eq!(1, op.callbacks["onData"].paths.len());
    assert_eq!(2, op.callbacks["onData"].extensions.len());
}

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

#[test]
fn test_security_complex_yaml() {
    let input = include_str!("../samples/pass/security_complex.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_security_disabled_yaml() {
    let input = include_str!("../samples/pass/security_disabled.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_security_multiple_schemes_yaml() {
    let input = include_str!("../samples/pass/security_multiple_schemes.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_security_multiple_scopes_yaml() {
    let input = include_str!("../samples/pass/security_multiple_scopes.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_security_optional_or_yaml() {
    let input = include_str!("../samples/pass/security_optional_or.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_security_optional_yaml() {
    let input = include_str!("../samples/pass/security_optional.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_security_schemes_choice_yaml() {
    let input = include_str!("../samples/pass/security_schemes_choice.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_schema_prefix_items_yaml() {
    let input = include_str!("../samples/pass/schema_prefix_items.yaml");
    validate_sample(input, Format::Yaml);
}

#[test]
fn test_discriminator_products_yaml() {
    let input = include_str!("../samples/pass/discriminator-products.yaml");
    let spec = validate_sample(input, Format::Yaml);
    
    // Verify the Product schema has a discriminator
    let product_schema = spec.components.as_ref()
        .and_then(|c| c.schemas.get("Product"))
        .and_then(|s| s.resolve(&spec).ok())
        .expect("Product schema should exist");
    
    assert!(product_schema.discriminator.is_some(), "Product schema should have discriminator");
    let discriminator = product_schema.discriminator.as_ref().unwrap();
    assert_eq!(discriminator.property_name, "productType");
    assert_eq!(discriminator.mapping.as_ref().unwrap().len(), 3);
}

/// Describes the format of the text input.
enum Format {
    Json,
    Yaml,
}

/// Validate that a given `sample` is being parsed by `oas3` without immediate errors. Panics, if
/// an error is encountered and gives error context in the panic message.
fn validate_sample(input: &str, format: Format) -> Spec {
    match format {
        Format::Json => {
            let mut json_deserializer = serde_json::Deserializer::from_str(input);
            let result: Result<oas3::OpenApiV3Spec, _> =
                serde_path_to_error::deserialize(&mut json_deserializer);

            match result {
                Ok(spec) => spec,
                Err(err) => {
                    panic!("{}: {}", err, err.path());
                }
            }
        }
        Format::Yaml => {
            let yaml_deserializer = serde_yaml::Deserializer::from_str(input);
            let result: Result<oas3::OpenApiV3Spec, _> =
                serde_path_to_error::deserialize(yaml_deserializer);

            match result {
                Ok(spec) => spec,
                Err(err) => {
                    panic!("{}: {}", err, err.path());
                }
            }
        }
    }
}

#[test]
fn validate_failing_samples() {
    // TODO: implement validation for one-of: [paths, components, webhooks]
    // see https://spec.openapis.org/oas/v3.1.1#openapi-document
    // oas3::from_str(include_str!("samples/fail/no_containers.yaml")).unwrap_err();

    // TODO: implement validation for non-empty server enum
    // oas3::from_str(include_str!("samples/fail/server_enum_empty.yaml")).unwrap_err();

    // TODO: implement validation for server enum references
    // oas3::from_str(include_str!("samples/fail/server_enum_unknown.yaml")).unwrap_err();

    // TODO: reject top-level extensions? find reference for rejection
    // oas3::from_str(include_str!("samples/fail/unknown_container.yaml")).unwrap_err();
}
