# OAS3

> Structures and tools to parse and navigate [OpenAPI v3 Spec][oas3-spec] files.

Doug Tangren (softprops) 2017

## install

add the following to your `Cargo.toml` file

```toml
[dependencies]
oas3 = "*"
```

## usage

```rust
extern crate oas3;

fn main() {
  match oas3::from_path("path/to/openapi.yaml") {
    Ok(spec) => println!("spec: {:?}", spec),
    Err(err) => println!("error: {}", err)
  }
}
```

[oas3-spec]: https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md
