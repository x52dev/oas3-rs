use oas3::{spec::ObjectOrReference, Spec};

fn assert_ordered(text: &str, first: &str, second: &str) {
    let first_index = text
        .find(first)
        .unwrap_or_else(|| panic!("expected {first:?} in:\n{text}"));
    let second_index = text
        .find(second)
        .unwrap_or_else(|| panic!("expected {second:?} in:\n{text}"));

    assert!(
        first_index < second_index,
        "expected {first:?} before {second:?} in:\n{text}"
    );
}

fn assert_spec_preserves_map_order(spec: &Spec) {
    let paths = spec.paths.as_ref().unwrap();
    assert_eq!(
        paths.keys().map(String::as_str).collect::<Vec<_>>(),
        ["/z", "/a"]
    );

    let schemas = &spec.components.as_ref().unwrap().schemas;
    assert_eq!(
        schemas.keys().map(String::as_str).collect::<Vec<_>>(),
        ["Zebra", "Apple"]
    );

    let ObjectOrReference::Object(zebra) = &schemas["Zebra"] else {
        panic!("expected inline Zebra schema");
    };
    assert_eq!(
        zebra
            .properties
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["beta", "alpha"]
    );

    assert_eq!(
        spec.extensions
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["meta", "first", "second"]
    );

    let meta = spec.extensions["meta"].as_object().unwrap();
    assert_eq!(
        meta.keys().map(String::as_str).collect::<Vec<_>>(),
        ["zed", "alpha"]
    );
}

#[test]
fn callback() {
    let input = include_str!("../samples/pass/callback.yml");
    let spec = validate_sample(input, Format::Yaml);
    let op = spec.operation_by_id("callback").unwrap();

    let on_data = op.callbacks["onData"].resolve(&spec).unwrap();

    assert_eq!(1, on_data.paths.len());
    assert_eq!(2, on_data.extensions.len());
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
fn preserves_map_order_when_deserializing_yaml_and_json() {
    let yaml = indoc::indoc! {"
        openapi: 3.1.0
        info:
          title: order test
          version: v1
        paths:
          /z:
            get:
              responses: {}
          /a:
            get:
              responses: {}
        components:
          schemas:
            Zebra:
              type: object
              properties:
                beta:
                  type: string
                alpha:
                  type: string
            Apple:
              type: object
        x-meta:
          zed: true
          alpha: true
        x-first: true
        x-second: true
    "};

    let json = indoc::indoc! {r##"
        {
          "openapi": "3.1.0",
          "info": {
            "title": "order test",
            "version": "v1"
          },
          "paths": {
            "/z": {
              "get": {
                "responses": {}
              }
            },
            "/a": {
              "get": {
                "responses": {}
              }
            }
          },
          "components": {
            "schemas": {
              "Zebra": {
                "type": "object",
                "properties": {
                  "beta": {
                    "type": "string"
                  },
                  "alpha": {
                    "type": "string"
                  }
                }
              },
              "Apple": {
                "type": "object"
              }
            }
          },
          "x-meta": {
            "zed": true,
            "alpha": true
          },
          "x-first": true,
          "x-second": true
        }
    "##};

    let yaml_spec = oas3::from_yaml(yaml).unwrap();
    let json_spec = oas3::from_json(json).unwrap();

    assert_spec_preserves_map_order(&yaml_spec);
    assert_spec_preserves_map_order(&json_spec);

    let output = oas3::to_yaml(&yaml_spec).unwrap();
    assert_ordered(&output, "/z:", "/a:");
    assert_ordered(&output, "Zebra:", "Apple:");
    assert_ordered(&output, "beta:", "alpha:");
    assert_ordered(&output, "x-meta:", "x-first:");
    assert_ordered(&output, "x-first:", "x-second:");
    assert_ordered(&output, "zed: true", "alpha: true");

    let output = oas3::to_json(&json_spec).unwrap();
    assert_ordered(&output, "\"/z\"", "\"/a\"");
    assert_ordered(&output, "\"Zebra\"", "\"Apple\"");
    assert_ordered(&output, "\"beta\"", "\"alpha\"");
    assert_ordered(&output, "\"x-meta\"", "\"x-first\"");
    assert_ordered(&output, "\"x-first\"", "\"x-second\"");
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
            let yaml_deserializer = yaml_serde::Deserializer::from_str(input);
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
