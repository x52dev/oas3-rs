//! Public API tests for the optional `facet` integration.

#![cfg(feature = "facet")]

use std::collections::BTreeMap;

use facet::Facet;

use jsonpath_fmt::{render_facet, Error, Template};

#[derive(Debug, Facet)]
struct Inner {
    not_field: Option<()>,
    field: &'static str,
}

#[derive(Debug, Facet)]
struct Data {
    interpolated: u8,
    inner: Inner,
}

#[derive(Debug, Facet)]
struct Item {
    name: &'static str,
}

#[derive(Debug, Facet)]
struct Meta {
    count: u8,
}

#[derive(Debug, Facet)]
struct Payload {
    flag: bool,
    items: Vec<Item>,
    meta: Meta,
    nothing: Option<u8>,
    maybe: Option<Inner>,
    counts: BTreeMap<String, u8>,
    numeric_keys: BTreeMap<u32, &'static str>,
}

#[test]
fn renders_requested_struct_example_via_facet() {
    let rendered = render_facet(
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
fn renders_lists_structs_and_nulls_via_facet() {
    let rendered = render_facet(
        "flag={flag}; item={items.1.name}; meta={meta}; none={nothing}",
        &Payload {
            flag: true,
            items: vec![Item { name: "first" }, Item { name: "second" }],
            meta: Meta { count: 2 },
            nothing: None,
            maybe: None,
            counts: BTreeMap::new(),
            numeric_keys: BTreeMap::new(),
        },
    )
    .unwrap();

    assert_eq!(
        rendered,
        r#"flag=true; item=second; meta={"count":2}; none=null"#
    );
}

#[test]
fn unwraps_some_options_during_path_resolution() {
    let rendered = render_facet(
        "{maybe.field}",
        &Payload {
            flag: false,
            items: Vec::new(),
            meta: Meta { count: 0 },
            nothing: None,
            maybe: Some(Inner {
                not_field: None,
                field: "inside",
            }),
            counts: BTreeMap::new(),
            numeric_keys: BTreeMap::new(),
        },
    )
    .unwrap();

    assert_eq!(rendered, "inside");
}

#[test]
fn resolves_string_and_numeric_map_keys() {
    let mut counts = BTreeMap::new();
    counts.insert("alpha".to_owned(), 3);

    let mut numeric_keys = BTreeMap::new();
    numeric_keys.insert(7, "seven");

    let template = Template::parse("{counts.alpha}/{numeric_keys.7}").unwrap();
    let rendered = template
        .render_facet(&Payload {
            flag: false,
            items: Vec::new(),
            meta: Meta { count: 0 },
            nothing: None,
            maybe: None,
            counts,
            numeric_keys,
        })
        .unwrap();

    assert_eq!(rendered, "3/seven");
}

#[test]
fn reports_missing_paths_via_facet() {
    let err = render_facet(
        "{inner.missing}",
        &Data {
            interpolated: 1,
            inner: Inner {
                not_field: None,
                field: "value",
            },
        },
    )
    .unwrap_err();

    assert!(matches!(
        err,
        Error::MissingPath(ref missing) if missing.path() == "inner.missing"
    ));
}
