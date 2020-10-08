#![allow(dead_code, unused_variables)]

use http::Method;
use serde_json::json;

fn main() {
    let _ = dotenv::dotenv();
    pretty_env_logger::init();

    let valid = json!({
        "thing": "car",
        "size": 3.14,
        "page": 1
    });

    let invalid1 = json!({
        "thing": "car",
        "size": 3.14,
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

    let spec = oas3::from_path("./data/oas-samples/allof.yml").expect("api spec parse error");
    let op = spec.operation(&Method::GET, "/").unwrap();
    let schema = &op.responses(&spec)["200"].content["application/json"]
        .schema(&spec)
        .unwrap();

    // let v = ValidatorTree::from_schema(&schema, &spec).unwrap();
    // v.validate(valid).unwrap();
    // v.validate(invalid1).unwrap();
    // v.validate(invalid2).unwrap();
    // v.validate(invalid3).unwrap();
}
