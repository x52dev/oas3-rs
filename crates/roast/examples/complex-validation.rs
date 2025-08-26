#![allow(dead_code, unused_variables, missing_docs)]

use std::fs;

use http::Method;
use serde_json::json;

fn main() -> eyre::Result<()> {
    let _ = dotenvy::dotenv();
    pretty_env_logger::init();

    let valid = json!({
        "thing": "car",
        "size": 3.1,
        "page": 1
    });

    let invalid1 = json!({
        "thing": "car",
        "size": 3.1,
    });

    let invalid2 = json!({
        "thing": "car",
        "page": 1
    });

    let invalid3 = json!({
        "thing": "car",
        "size": "3.14m",
        "page": 1
    });

    let yaml = fs::read_to_string("./data/oas-samples/pet-store.yml")?;
    let spec = oas3::from_yaml(yaml).expect("api spec parse error");
    let op = spec.operation(&Method::GET, "/").unwrap();
    let schema = &op.responses(&spec)["200"].content["application/json"].schema(&spec)?;

    // let v = ValidatorTree::from_schema(&schema, &spec).unwrap();
    // v.validate(valid).unwrap();
    // v.validate(invalid1).unwrap();
    // v.validate(invalid2).unwrap();
    // v.validate(invalid3).unwrap();

    Ok(())
}
