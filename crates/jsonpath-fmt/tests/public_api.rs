//! Public API tests for `jsonpath-fmt`.

use jsonpath_fmt::{render, render_value, Error, ParseError, Template};
use serde::{
    ser::{Error as _, Serializer},
    Serialize,
};
use serde_json::json;

#[derive(Debug, Serialize)]
struct Inner {
    not_field: Option<()>,
    field: &'static str,
}

#[derive(Debug, Serialize)]
struct Data {
    interpolated: u8,
    inner: Inner,
}

struct FailingSerialize;

impl Serialize for FailingSerialize {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(S::Error::custom("cannot serialize"))
    }
}

#[test]
fn renders_requested_struct_example() {
    let rendered = render(
        "my-{interpolated}-string-{inner.field}",
        &Data {
            interpolated: 67,
            inner: Inner {
                not_field: None,
                field: "woahg",
            },
        },
    )
    .unwrap();

    assert_eq!(rendered, "my-67-string-woahg");
}

#[test]
fn renders_from_json_values() {
    let rendered = render_value(
        "flag={flag}; item={items.1.name}; meta={meta}",
        &json!({
            "flag": true,
            "items": [
                { "name": "first" },
                { "name": "second" }
            ],
            "meta": { "count": 2 }
        }),
    )
    .unwrap();

    assert_eq!(rendered, r#"flag=true; item=second; meta={"count":2}"#);
}

#[test]
fn supports_whitespace_around_paths() {
    let rendered = render_value(
        "value={ inner . field }",
        &json!({
            "inner": { "field": "trimmed" }
        }),
    )
    .unwrap();

    assert_eq!(rendered, "value=trimmed");
}

#[test]
fn supports_repeated_placeholders() {
    let rendered = render_value(
        "{name}-{name}-{nested.value}",
        &json!({
            "name": "echo",
            "nested": { "value": 3 }
        }),
    )
    .unwrap();

    assert_eq!(rendered, "echo-echo-3");
}

#[test]
fn supports_literal_only_templates() {
    let template = Template::parse("no placeholders here").unwrap();

    assert_eq!(
        template.render_value(&json!({ "ignored": true })).unwrap(),
        "no placeholders here"
    );
}

#[test]
fn supports_empty_templates() {
    let template = Template::parse("").unwrap();

    assert_eq!(template.render_value(&json!({})).unwrap(), "");
}

#[test]
fn supports_escaped_braces_around_placeholders() {
    let rendered = render("{{{value}}}", &json!({ "value": 42 })).unwrap();

    assert_eq!(rendered, "{42}");
}

#[test]
fn supports_escaped_braces_without_placeholders() {
    let rendered = render_value("{{hello}}", &json!({})).unwrap();

    assert_eq!(rendered, "{hello}");
}

#[test]
fn renders_strings_without_extra_quotes() {
    let rendered = render_value("{value}", &json!({ "value": "plain" })).unwrap();

    assert_eq!(rendered, "plain");
}

#[test]
fn renders_null_scalars_arrays_and_objects_as_json() {
    let rendered = render_value(
        "null={nothing}; bool={flag}; num={count}; arr={list}; obj={meta}",
        &json!({
            "nothing": null,
            "flag": false,
            "count": 12.5,
            "list": [1, 2, 3],
            "meta": { "ok": true }
        }),
    )
    .unwrap();

    assert_eq!(
        rendered,
        r#"null=null; bool=false; num=12.5; arr=[1,2,3]; obj={"ok":true}"#
    );
}

#[test]
fn resolves_array_indexes_from_root_arrays() {
    let rendered = render_value(
        "{0.name}/{1.name}",
        &json!([
            { "name": "first" },
            { "name": "second" }
        ]),
    )
    .unwrap();

    assert_eq!(rendered, "first/second");
}

#[test]
fn treats_numeric_object_keys_as_object_fields() {
    let rendered = render_value("{items.0}", &json!({ "items": { "0": "zero" } })).unwrap();

    assert_eq!(rendered, "zero");
}

#[test]
fn template_can_be_reused() {
    let template = Template::parse("{name}-{items.0}").unwrap();

    assert_eq!(
        template
            .render_value(&json!({ "name": "first", "items": ["a"] }))
            .unwrap(),
        "first-a"
    );
    assert_eq!(
        template
            .render_value(&json!({ "name": "second", "items": ["b"] }))
            .unwrap(),
        "second-b"
    );
}

#[test]
fn reports_missing_object_fields() {
    let err = render("hello-{missing}", &json!({ "present": true })).unwrap_err();

    assert!(matches!(
        err,
        Error::MissingPath(ref missing) if missing.path() == "missing"
    ));
}

#[test]
fn reports_missing_array_indexes() {
    let err = render_value("{items.9}", &json!({ "items": ["a"] })).unwrap_err();

    assert!(matches!(
        err,
        Error::MissingPath(ref missing) if missing.path() == "items.9"
    ));
}

#[test]
fn reports_missing_nested_fields_after_non_container_values() {
    let err = render_value("{name.first}", &json!({ "name": "flat" })).unwrap_err();

    assert!(matches!(
        err,
        Error::MissingPath(ref missing) if missing.path() == "name.first"
    ));
}

#[test]
fn rejects_empty_placeholders() {
    let err = Template::parse("{}").unwrap_err();

    assert!(matches!(
        err,
        Error::Parse(ParseError::EmptyPlaceholder { position: 0 })
    ));
}

#[test]
fn rejects_whitespace_only_placeholders() {
    let err = Template::parse("{   }").unwrap_err();

    assert!(matches!(
        err,
        Error::Parse(ParseError::EmptyPlaceholder { position: 0 })
    ));
}

#[test]
fn rejects_invalid_paths_with_empty_segments() {
    assert!(matches!(
        Template::parse("{inner..field}").unwrap_err(),
        Error::Parse(ParseError::InvalidPath { position: 0, path })
            if path == "inner..field"
    ));
    assert!(matches!(
        Template::parse("{.field}").unwrap_err(),
        Error::Parse(ParseError::InvalidPath { position: 0, path })
            if path == ".field"
    ));
    assert!(matches!(
        Template::parse("{field.}").unwrap_err(),
        Error::Parse(ParseError::InvalidPath { position: 0, path })
            if path == "field."
    ));
}

#[test]
fn rejects_unmatched_braces_with_positions() {
    assert!(matches!(
        Template::parse("{value").unwrap_err(),
        Error::Parse(ParseError::UnmatchedOpeningBrace { position: 0 })
    ));
    assert!(matches!(
        Template::parse("value}").unwrap_err(),
        Error::Parse(ParseError::UnmatchedClosingBrace { position: 5 })
    ));
    assert!(matches!(
        Template::parse("x{value").unwrap_err(),
        Error::Parse(ParseError::UnmatchedOpeningBrace { position: 1 })
    ));
}

#[test]
fn rejects_nested_opening_braces() {
    let err = Template::parse("{outer{inner}}").unwrap_err();

    assert!(matches!(
        err,
        Error::Parse(ParseError::NestedOpeningBrace { position: 0 })
    ));
}

#[test]
fn returns_serialize_errors() {
    let err = render("{anything}", &FailingSerialize).unwrap_err();

    assert!(matches!(err, Error::Serialize(_)));
}
