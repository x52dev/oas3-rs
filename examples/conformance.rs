#![feature(todo_macro)]
#![allow(dead_code, unused_imports, unused_variables)]

use std::{env, error::Error as StdError, string::ToString};

use colored::{ColoredString, Colorize};
use http::{Method, StatusCode};
use log::{debug, info};
use oas3::{
    conformance::{
        ConformanceTestSpec, OperationSpec, ParamPosition, RequestSource, RequestSpec,
        ResolvedConformanceTestSpec, ResponseSpec, TestAuthorization, TestRequest,
    },
    validation::Error as ValidationError,
    Error, Spec,
};
use prettytable::{cell, row, Table};

fn do_request(req: &TestRequest) -> Result<reqwest::Response, reqwest::Error> {
    let base_url = "http://localhost:9000/api/auth/v1";
    let client = reqwest::Client::new();

    // TODO: add other param types to request

    let method: reqwest::Method = req.operation.method.as_str().parse().unwrap();
    let url: String = [base_url, &req.operation.path].concat();

    let url = req
        .params
        .iter()
        .filter(|&param| param.position == ParamPosition::Path)
        .fold(url, |url, part| {
            url.replace(&["{", &part.name, "}"].concat(), &part.value)
        });

    client
        .request(method, &url)
        .headers(req.headers.clone())
        .body(req.body.to_vec())
        .send()
}

// TODO: review error type
fn do_test(spec: &Spec, test: ResolvedConformanceTestSpec) -> Result<(), ValidationError> {
    debug!("request: {:?}", &test.request);
    debug!("response spec: {:?}", &test.response);

    let mut res = do_request(&test.request).unwrap();

    // validate response status
    let validation = test.response.validate_status(&res.status())?;
    info!("validation: {:?}", &validation);

    // validate response body
    if test.response.body_validator.is_some() {
        let body = res.json().map_err(|_| ValidationError::NotJson)?;
        let status = res.status();

        debug!("response: {:?}", &res);

        let validation = test.response.validate_body(&body)?;
        info!("validation: {:?}", &validation);
    }

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

fn error_string(err: &dyn StdError) -> ColoredString {
    let mut err_str = err.to_string();
    err_str.push('\n');

    let mut cause = err.source();
    while let Some(err) = cause {
        err_str.push_str(&err.to_string());
        err_str.push('\n');
        cause = err.source();
    }

    err_str.red()
}

fn print_test_results(results: &[(ConformanceTestSpec, Option<Error>)]) {
    let mut table = Table::new();

    table.add_row(row!["TEST", "RESULT", "MESSAGE"]);

    for (test, error) in results {
        let op = &test.operation;

        let test_desc = if let Some(ref name) = test.name {
            let test_name = name.yellow();
            let op_spec = op.to_string().italic();
            format!("{}\n{}", test_name, op_spec)
        } else {
            let op_spec = op.to_string().italic();
            op_spec.to_string()
        };

        let status = if error.is_some() {
            " ERR ".red().reversed().blink()
        } else {
            " OK ".green().reversed()
        };

        let msg = error
            .as_ref()
            .map(|err| error_string(err))
            .unwrap_or_else(|| "".normal());

        table.add_row(row![test_desc, status, msg]);
    }

    table.printstd();
}

fn main() {
    let _ = dotenv::dotenv();
    pretty_env_logger::init();

    let spec = oas3::from_path(env::var("OAS_PATH").unwrap()).expect("api spec parse error");

    let auth_method = TestAuthorization::bearer(env::var("TOKEN").unwrap());

    let results = do_tests(
        &spec,
        &[
            &ConformanceTestSpec::named(
                "login success",
                OperationSpec::post("/token"),
                RequestSpec::from_json_example("basic"),
                ResponseSpec::from_json_schema(200),
            ),
            &ConformanceTestSpec::named(
                "verify revoked",
                OperationSpec::post("/verify"),
                RequestSpec::from_json_example("revoked"),
                ResponseSpec::from_example(200, "application/json", "revoked"),
            ),
            &ConformanceTestSpec::named(
                "fail login unregistered",
                OperationSpec::operation_id("signin"),
                RequestSpec::from_json_example("unregistered"),
                ResponseSpec::from_json_schema(401),
            ),
            &ConformanceTestSpec::named_get_success(
                "check logged in",
                OperationSpec::operation_id("checkLoggedIn"),
            )
            .with_auth(&auth_method),
            &ConformanceTestSpec::named_get_success(
                "fetch own tokens",
                OperationSpec::operation_id("ownTokens"),
            )
            .with_auth(&auth_method),
            &ConformanceTestSpec::named_get_success(
                "fetch own valid tokens",
                OperationSpec::operation_id("listOwnHwcreds"),
            )
            .with_auth(&auth_method),
            &ConformanceTestSpec::named(
                "start mobile token process",
                OperationSpec::operation_id("mtRequest"),
                RequestSpec::from_json_example("blank").with_auth(&auth_method),
                ResponseSpec::from_json_schema(200),
            ),
            &ConformanceTestSpec::named_get_success(
                "admin list failed logins",
                OperationSpec::operation_id("adminListFailedLogins"),
            )
            .with_auth(&auth_method),
            &ConformanceTestSpec::named(
                "admin list user tokens",
                OperationSpec::operation_id("adminListAccountTokens"),
                RequestSpec::empty()
                    .with_auth(&auth_method)
                    .add_param("acc_uid", "53ad084e-ca6e-475d-a9f3-4b184999b244"),
                ResponseSpec::from_status(200),
            ),
            &ConformanceTestSpec::named(
                "admin list user password changes",
                OperationSpec::operation_id("adminListAccountPasswordChanges"),
                RequestSpec::empty()
                    .with_auth(&auth_method)
                    .add_param("acc_id", "53ad084e-ca6e-475d-a9f3-4b184999b244"),
                ResponseSpec::from_status(200),
            ),
            &ConformanceTestSpec::named(
                "admin list user failed logins",
                OperationSpec::operation_id("adminListAccountFailedLogins"),
                RequestSpec::empty()
                    .with_auth(&auth_method)
                    .add_param("acc_id", "53ad084e-ca6e-475d-a9f3-4b184999b244"),
                ResponseSpec::from_status(200),
            ),
        ],
    );

    println!("");
    print_test_results(results.as_slice());
    println!("");
}
