#![allow(dead_code, unused_variables)]

#[cfg(feature = "conformance")]
use oas3::conformance::{
    ConformanceTestSpec, OperationSpec, RequestSpec, ResponseSpec, TestRunner,
};

#[cfg(feature = "conformance")]
fn main() {
    let _ = dotenv::dotenv();
    pretty_env_logger::init();

    let spec = oas3::from_path("./data/oas-samples/pet-store.yaml").expect("api spec parse error");
    let base_url: &str = &spec.primary_server().expect("no primary server").url;
    let mut runner = TestRunner::new(base_url, spec.clone());

    runner.add_tests(&[
        ConformanceTestSpec::named(
            "list pets",
            OperationSpec::operation_id("findPetsByStatus"),
            RequestSpec::empty().add_param("status", "available"),
            ResponseSpec::from_json_schema(200),
        ),
        ConformanceTestSpec::named(
            "get single pet",
            OperationSpec::operation_id("getPetById"),
            RequestSpec::empty().add_param("petId", "9199424981609313390"),
            ResponseSpec::from_status(200),
        ),
        ConformanceTestSpec::named(
            "get non-existent pet",
            OperationSpec::operation_id("getPetById"),
            RequestSpec::empty().add_param("petId", "0"),
            ResponseSpec::from_status(404),
        ),
    ]);

    println!("");
    runner.run_queued_tests();
    runner.print_results();
    println!("");
}

#[cfg(feature = "validation")]
fn main() {
    println!("run this example with the `conformance` feature enabled");
}
