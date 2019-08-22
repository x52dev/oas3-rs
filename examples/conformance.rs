#![feature(todo_macro)]
#![allow(dead_code, unused_variables)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

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

    runner.immediate_test(ConformanceTestSpec::named(
        "login success",
        OperationSpec::operation_id("signin"),
        RequestSpec::from_json_example("basic"),
        ResponseSpec::from_json_schema(200),
    ));

    let login: JsonValue = runner.last_response_body().unwrap();
    let jwt = login["token"].as_str().expect("token is not a string");

    let auth_method = TestAuthorization::bearer(jwt);

    runner.add_tests(&[
        ConformanceTestSpec::named(
            "verify revoked",
            OperationSpec::operation_id("verify"),
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
    runner.immediate_test(ConformanceTestSpec::named(
        "start mobile token process",
        OperationSpec::operation_id("mtRequest"),
        RequestSpec::from_json_example("dummy").with_auth(&auth_method),
        ResponseSpec::from_json_schema(200),
    ));

    let mtr: JsonValue = runner.last_response_body().unwrap();
    let mtr_uid = mtr["uid"].as_str().unwrap();
    let scan_req = serde_json::to_vec(&mtr).unwrap();

    runner.immediate_test(ConformanceTestSpec::named(
        "scan mobile token",
        OperationSpec::operation_id("mtScan"),
        // no auth on mobile side
        // TODO: really need a way to validate dynamic requests
        RequestSpec::from_bad_raw(scan_req).override_content_type("application/json"),
        ResponseSpec::from_json_schema(200),
    ));

    runner.run_queued_tests();
    let scan: JsonValue = runner.last_response_body().unwrap();
    let status_req = serde_json::to_vec(&json!({ "uid": &mtr_uid })).unwrap();

    // confirm = mtr + confirm code
    let confirm = json!({
        "confirmation_code": scan["confirmation_code"],
        "uid": mtr["uid"],
        "requester_token": mtr["requester_token"],
        "created_at": mtr["created_at"],
    });
    let confirm_req = serde_json::to_vec(&confirm).unwrap();

    runner.add_tests(&[
        ConformanceTestSpec::named(
            "mobile token request status disallows login before confirmation",
            OperationSpec::operation_id("mtStatus"),
            // no auth on mobile side
            RequestSpec::from_bad_raw(status_req.clone()).override_content_type("application/json"),
            ResponseSpec::from_json_schema(403),
        ),
        ConformanceTestSpec::named(
            "confirm mobile token",
            OperationSpec::operation_id("mtConfirm"),
            RequestSpec::from_bad_raw(confirm_req)
                .with_auth(&auth_method)
                .override_content_type("application/json"),
            ResponseSpec::from_status(200),
        ),
        ConformanceTestSpec::named(
            "mobile token request allows login",
            OperationSpec::operation_id("mtStatus"),
            // no auth on mobile side
            RequestSpec::from_bad_raw(status_req).override_content_type("application/json"),
            ResponseSpec::from_json_schema(200),
        ),
    ]);

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
