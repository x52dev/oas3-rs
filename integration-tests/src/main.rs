#![allow(missing_docs)]

#[cfg(test)]
mod issues;
#[cfg(test)]
mod samples;

fn main() {
    panic!(
        r#"This is an integration testing crate. You must run it using "cargo test" or another Rust test driver such as "cargo nextest"."#
    )
}
