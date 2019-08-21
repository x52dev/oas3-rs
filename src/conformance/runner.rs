use std::{env, error::Error as StdError, ops::Deref, string::ToString};

use colored::{ColoredString, Colorize};
use http::{Method, StatusCode};
use log::{debug, info};
use prettytable::{cell, row, Table};

use crate::{
    conformance::{
        ConformanceTestSpec, OperationSpec, ParamPosition, RequestSource, RequestSpec,
        ResolvedConformanceTestSpec, ResponseSpec, TestAuthorization, TestRequest,
    },
    validation::Error as ValidationError,
    Error, Spec,
};

type TestResult = (ConformanceTestSpec, Result<reqwest::Response, Error>);

#[derive(Debug)]
pub struct TestRunner {
    pub base_url: String,
    pub spec: Spec,
    pub queue: Vec<ConformanceTestSpec>,
    pub results: Vec<TestResult>,
    pub auth: Option<TestAuthorization>,
}

impl TestRunner {
    pub fn new<T: Into<String>>(base_url: T, spec: Spec) -> Self {
        Self {
            base_url: base_url.into(),
            spec,
            queue: vec![],
            results: vec![],
            auth: None,
        }
    }

    pub fn add_tests(&mut self, tests: &[ConformanceTestSpec]) {
        self.queue.append(&mut tests.to_owned())
    }

    pub fn send_request(&self, req: &TestRequest) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();

        // TODO: add other param types to request

        let method: reqwest::Method = req.operation.method.as_str().parse().unwrap();
        let url: String = [self.base_url.deref(), &req.operation.path].concat();

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
    fn run_test(
        &self,
        test: ResolvedConformanceTestSpec,
    ) -> Result<reqwest::Response, ValidationError> {
        debug!("request: {:?}", &test.request);
        debug!("response spec: {:?}", &test.response);

        let mut res = self.send_request(&test.request).unwrap();

        // validate response status
        let validation = test.response.validate_status(&res.status())?;
        info!("validation: {:?}", &validation);

        // validate response body
        if test.response.body_validator.is_some() {
            let body = res.json().map_err(|_| ValidationError::NotJson)?;
            let status = res.status();

            debug!("response body: {:?}", &res);

            let validation = test.response.validate_body(&body)?;
            info!("validation: {:?}", &validation);
        }

        Ok(res)
    }

    /// Runs tests in queue serially, removing them from the queue and appending the results and
    /// original test specs in the result list.
    pub fn run_queued_tests(&mut self) {
        while let Some(test) = self.queue.pop() {
            match test.resolve(&self.spec) {
                Ok(resolved_test) => {
                    self.results.push((
                        test.clone(),
                        self.run_test(resolved_test).map_err(Error::Validation),
                    ));
                }
                Err(err) => self.results.push((test.clone(), Err(err))),
            }
        }
    }

    pub fn results(&self) -> &[TestResult] { &self.results }

    pub fn print_results(&self) {
        let mut table = Table::new();

        table.add_row(row!["TEST", "RESULT", "MESSAGE"]);

        for (test, error) in &self.results {
            let op = &test.operation;

            let test_desc = if let Some(name) = &test.name {
                let test_name = name.yellow();
                let op_spec = op.to_string().italic();
                format!("{}\n{}", test_name, op_spec)
            } else {
                let op_spec = op.to_string().italic();
                op_spec.to_string()
            };

            let status = if error.is_err() {
                " ERR ".red().reversed().blink()
            } else {
                " OK ".green().reversed()
            };

            let msg = error
                .as_ref()
                .err()
                .map(|err| format_error(err))
                .unwrap_or_else(|| "".normal());

            table.add_row(row![test_desc, status, msg]);
        }

        table.printstd();
    }
}

pub fn format_error(err: &dyn StdError) -> ColoredString {
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
