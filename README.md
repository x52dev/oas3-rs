# OAS3

> Structures and tools to parse, navigate and validate [OpenAPI v3.1 Spec][oas3-spec] files.

Originally based on v3 parts of the [`openapi`](https://crates.io/crates/openapi) crate by [softprops](https://crates.io/users/softprops).

Additional features:

- Validation constructors
- Example request/response validation
- Live API conformance testing

## Usage

```rust
fn main() {
  match oas3::from_path("path/to/openapi.yaml") {
    Ok(spec) => println!("spec: {:?}", spec),
    Err(err) => println!("error: {}", err)
  }
}
```

[oas3-spec]: https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md
