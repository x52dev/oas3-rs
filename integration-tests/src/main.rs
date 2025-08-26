#![expect(missing_docs)]

mod issues;
mod samples;

fn main() {
    panic!(
        r#"This is an integration testing crate. You must run it using "cargo test" or another Rust test driver such as "cargo nextest"."#
    )
}
