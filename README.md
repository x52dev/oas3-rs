# OAS3

> Structures and tools to parse, navigate and validate [OpenAPI v3 Spec][oas3-spec] files. 

Based on v3 parts of [openapi](https://crates.io/crates/openapi) crate by [softprops](https://crates.io/users/softprops).

Additional features:
- Validation constructors
- Example request/response validation
- Live API conformance testing

## Install

add the following to your `Cargo.toml` file

```toml
[dependencies]
oas3 = "0.1"
```

## Usage

```rust
extern crate oas3;

fn main() {
  match oas3::from_path("path/to/openapi.yaml") {
    Ok(spec) => println!("spec: {:?}", spec),
    Err(err) => println!("error: {}", err)
  }
}
```

[oas3-spec]: https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md
