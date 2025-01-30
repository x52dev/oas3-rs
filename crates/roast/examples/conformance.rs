#![allow(dead_code, unused_variables)]

use std::fs;

use roast::{ConformanceTestSpec, OperationSpec, RequestSpec, ResponseSpec, TestRunner};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let yaml = fs::read_to_string("./data/oas-samples/pet-store.yml")?;
    let spec = oas3::from_yaml(yaml)?;
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
            "get single pet (failure example)",
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

    println!();
    runner.run_queued_tests().await;
    runner.print_results();
    println!();

    Ok(())
}
