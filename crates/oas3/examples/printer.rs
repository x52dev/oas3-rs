//! Demonstrates reading an OpenAPI spec file and printing back to stdout.

use std::{env, fs};

fn main() -> eyre::Result<()> {
    let Some(path) = env::args().nth(1) else {
        return Ok(());
    };

    let yaml = fs::read_to_string(path)?;
    let spec = oas3::from_yaml(yaml)?;
    println!("{}", oas3::to_yaml(&spec).unwrap());

    Ok(())
}
