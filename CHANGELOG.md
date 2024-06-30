# Changelog

## Unreleased

- Add `from_str()` function.
- Add `spec::SecurityScheme::MutualTls` enum variant.
- Add `spec::SecurityScheme::{ApiKey, Http, OAuth2, OpenIdConnect}::description` fields.
- Add `spec::SchemaTypeSet` enum.
- Add `spec::SchemaType::Null` variant.
- The `Spec::paths` field is now optional to closer align with the spec.
- The `spec::Operation::responses` field is now optional to closer align with the spec.
- The `spec::Schema::{exclusive_maximum, exclusive_minimum}` fields are now of type `Option<serde_json:Number>` to closer align with the spec.
- Migrate YAML parsing to `serde_yml`. Exposed error type(s) have been altered.
- Add `spec::Schema::nullable` field.

## 0.5.0

- Update `http` dependency to `1`.

## 0.4.0

- The `bearer_format` field of `SecurityScheme::Http` is now optional.

## 0.3.0

- Initial re-release.

## 0.2.1

- Last version derived from <https://github.com/softprops/openapi>.
