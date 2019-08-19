#![feature(todo_macro)]
#![allow(dead_code, unused_imports, unused_variables)]

use std::env;

use colored::Colorize;
use http::{Method, StatusCode};
use log::{debug, info};
use oas3::{
    conformance::{
        ConformanceTestSpec, OperationSpec, RequestSpec, ResolvedConformanceTestSpec, ResponseSpec,
        TestAuthorization, TestRequest,
    },
    validation::Error as ValidationError,
    Error, Spec,
};

fn do_request(req: &TestRequest) -> Result<reqwest::Response, reqwest::Error> {
    let base_url = "http://localhost:9000/api/auth/v1";
    let client = reqwest::Client::new();

    // TODO: add url params
    // TODO: add qs params

    let method: reqwest::Method = req.operation.method.as_str().parse().unwrap();
    let url: String = [base_url, &req.operation.path].concat();

    client
        .request(method, &url)
        .headers(req.headers.clone())
        .body(req.body.to_vec())
        .send()
}

fn do_test(spec: &Spec, test: ResolvedConformanceTestSpec) -> Result<(), ValidationError> {
    debug!("request: {:?}", &test.request);
    debug!("response spec: {:?}", &test.response);

    let mut res = do_request(&test.request).unwrap();
    let body = res.json().map_err(|_| ValidationError::NotJson)?;
    let status = res.status();

    debug!("response: {:?}", &res);

    let validation = test.response.validate_status(&res.status())?;
    info!("validation: {:?}", &validation);

    let validation = test.response.validate_body(&body)?;
    info!("validation: {:?}", &validation);

    Ok(())
}

fn do_tests(
    spec: &Spec,
    tests: &[&ConformanceTestSpec],
) -> Vec<(ConformanceTestSpec, Option<Error>)> {
    let mut results = vec![];

    for test in tests {
        match test.resolve(&spec) {
            Ok(resolved_test) => {
                let validation = do_test(&spec, resolved_test);
                results.push(((*test).clone(), validation.map_err(Error::Validation).err()));
            }
            Err(err) => results.push(((*test).clone(), Some(err))),
        }
    }

    results
}

fn main() {
    let _ = dotenv::dotenv();
    pretty_env_logger::init();

    let spec = oas3::from_path("../app/docs/api.yml")
        .expect("api spec parse error");

    let auth_method = TestAuthorization::bearer(env::var("TOKEN").unwrap());

    let test_pass0 = ConformanceTestSpec::new(
        OperationSpec::post("/token"),
        RequestSpec::from_example("application/json", "basic"),
        ResponseSpec::from_schema("200", "application/json"),
    );

    let test_pass1 = ConformanceTestSpec::new(
        OperationSpec::post("/verify"),
        RequestSpec::from_json_example("revoked"),
        ResponseSpec::from_example("200", "application/json", "revoked"),
    );

    let test_fail0 = ConformanceTestSpec::new(
        OperationSpec::operation_id("signin"),
        RequestSpec::from_json_example("unregistered"),
        ResponseSpec::from_example("401", "application/json", "success"),
    );

    let test_fail1 = ConformanceTestSpec::new(
        OperationSpec::operation_id("checkLoggedIn"),
        RequestSpec::empty().with_auth(&auth_method),
        ResponseSpec::from_status("200"),
    );

    let results = do_tests(&spec, &[&test_pass0, &test_pass1, &test_fail0, &test_fail1]);

    println!("");
    print_test_results(results.as_slice());
    println!("");
}

fn print_test_results(results: &[(ConformanceTestSpec, Option<Error>)]) {
    for (test, error) in results {
        let mut msg = vec![];

        let op = &test.operation;

        if error.is_some() {
            msg.push("❌ Err".red());
        } else {
            msg.push("✅ Ok ".green());
        }

        msg.push(" | ".normal());
        msg.push(format!("{}", &op).normal());

        if let Some(ref err) = error {
            msg.push(" | ".normal());
            msg.push(err.to_string().red());
        }

        for part in msg {
            print!("{}", part);
        }

        println!("");
    }
}
