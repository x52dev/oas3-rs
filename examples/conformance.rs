#![feature(todo_macro)]
#![allow(dead_code, unused_variables)]

#[macro_use]
extern crate log;

use std::env;

use oas3::conformance::{
    ConformanceTestSpec, OperationSpec, RequestSpec, ResponseSpec, TestAuthorization, TestRunner,
};
use serde_json::Value as JsonValue;

fn main() {
    let _ = dotenv::dotenv();
    pretty_env_logger::init();

    let base_url = "http://localhost:9000/api/auth/v1";
    let spec = oas3::from_path(env::var("OAS_PATH").unwrap()).expect("api spec parse error");
    let mut runner = TestRunner::new(base_url, spec.clone());
    // let auth_method = TestAuthorization::bearer(env::var("TOKEN").unwrap());

    let login_test = ConformanceTestSpec::named(
        "login success",
        OperationSpec::post("/token"),
        RequestSpec::from_json_example("basic"),
        ResponseSpec::from_json_schema(200),
    );

    runner.add_tests(&[login_test.clone()]);
    runner.run_queued_tests();

    let body: JsonValue = runner.last_response_body();
    let jwt = body
        .as_object()
        .unwrap()
        .get("token")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    info!("{:?}", &jwt);

    let auth_method = TestAuthorization::bearer(jwt);

    runner.add_tests(&[
        ConformanceTestSpec::named(
            "verify revoked",
            OperationSpec::post("/verify"),
            RequestSpec::from_json_example("revoked"),
            ResponseSpec::from_example(200, "application/json", "revoked"),
        ),
        ConformanceTestSpec::named(
            "fail login unregistered",
            OperationSpec::operation_id("signin"),
            RequestSpec::from_json_example("unregistered"),
            ResponseSpec::from_json_schema(401),
        ),
        ConformanceTestSpec::named_get_success(
            "check logged in",
            OperationSpec::operation_id("checkLoggedIn"),
        )
        .with_auth(&auth_method),
        ConformanceTestSpec::named_get_success(
            "fetch own tokens",
            OperationSpec::operation_id("ownTokens"),
        )
        .with_auth(&auth_method),
        ConformanceTestSpec::named_get_success(
            "fetch own valid tokens",
            OperationSpec::operation_id("listOwnHwcreds"),
        )
        .with_auth(&auth_method),
    ]);

    // mt tests
    ConformanceTestSpec::named(
        "start mobile token process",
        OperationSpec::operation_id("mtRequest"),
        RequestSpec::from_json_example("blank").with_auth(&auth_method),
        ResponseSpec::from_json_schema(200),
    );

    // admin tests
    runner.add_tests(&[
        ConformanceTestSpec::named_get_success(
            "admin list failed logins",
            OperationSpec::operation_id("adminListFailedLogins"),
        )
        .with_auth(&auth_method),
        ConformanceTestSpec::named(
            "admin list user tokens",
            OperationSpec::operation_id("adminListAccountTokens"),
            RequestSpec::empty()
                .with_auth(&auth_method)
                .add_param("acc_uid", "53ad084e-ca6e-475d-a9f3-4b184999b244"),
            ResponseSpec::from_status(200),
        ),
        ConformanceTestSpec::named(
            "admin list user password changes",
            OperationSpec::operation_id("adminListAccountPasswordChanges"),
            RequestSpec::empty()
                .with_auth(&auth_method)
                .add_param("acc_uid", "53ad084e-ca6e-475d-a9f3-4b184999b244"),
            ResponseSpec::from_status(200),
        ),
        ConformanceTestSpec::named(
            "admin list user failed logins",
            OperationSpec::operation_id("adminListAccountFailedLogins"),
            RequestSpec::empty()
                .with_auth(&auth_method)
                .add_param("acc_uid", "53ad084e-ca6e-475d-a9f3-4b184999b244"),
            ResponseSpec::from_status(200),
        ),
    ]);
    runner.run_queued_tests();

    println!("");
    runner.print_results();
    println!("");
}
