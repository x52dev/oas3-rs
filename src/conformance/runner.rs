use std::{
    collections::VecDeque, error::Error as StdError, future::Future, ops::Deref, string::ToString,
    sync::atomic::AtomicUsize, sync::atomic::Ordering, sync::Arc,
};

use colored::{ColoredString, Colorize};
use futures_util::{stream, FutureExt as _, StreamExt as _};
use log::{debug, trace};
use prettytable::{cell, row, Table};
use serde_json::Value as JsonValue;
use url::Url;

use crate::{
    conformance::{
        ConformanceTestSpec, ParamPosition, ResolvedConformanceTestSpec, TestAuthentication,
        TestRequest, TestResponse,
    },
    validation::Error as ValidationError,
    Error, Spec,
};

type TestResult = (ConformanceTestSpec, Result<TestResponse, Error>);

#[derive(Debug)]
pub struct TestRunner {
    pub base_url: String,
    pub spec: Spec,
    pub queue: VecDeque<ConformanceTestSpec>,
    pub results: Vec<TestResult>,
    pub auth: Option<TestAuthentication>,
}

impl TestRunner {
    pub fn new<T: Into<String>>(base_url: T, spec: Spec) -> Self {
        Self {
            base_url: base_url.into(),
            spec,
            queue: VecDeque::new(),
            results: vec![],
            auth: None,
        }
    }

    pub fn add_tests(&mut self, tests: &[ConformanceTestSpec]) {
        self.queue.append(&mut tests.to_owned().into())
    }

    pub fn add_test(&mut self, test: ConformanceTestSpec) {
        self.queue.push_back(test);
    }

    pub fn immediate_test(&mut self, test: ConformanceTestSpec) -> impl Future<Output = ()> + '_ {
        self.add_test(test);
        self.run_queued_tests()
    }

    pub async fn send_request(&self, req: &TestRequest) -> Result<TestResponse, Error> {
        let client = reqwest::Client::new();

        let method: reqwest::Method = req.operation.method.as_str().parse().unwrap();
        let url: String = [self.base_url.deref(), &req.operation.path].concat();

        // path params
        let url = req
            .params
            .iter()
            .filter(|&param| param.position == ParamPosition::Path)
            .fold(url, |url, part| {
                url.replace(&["{", &part.name, "}"].concat(), &part.value)
            });

        let mut url: Url = url.parse().unwrap();

        {
            // query params
            let mut qs = url.query_pairs_mut();
            qs.clear();
            req.params
                .iter()
                .filter(|&param| param.position == ParamPosition::Query)
                .for_each(|param| {
                    qs.append_pair(&param.name, &param.value);
                });
        }

        // TODO: add other param types to request

        let res = client
            .request(method, &url.to_string())
            .headers(req.headers.clone())
            .body(req.body.to_vec())
            .send()
            .await?;

        let status = res.status();
        let headers = res.headers().clone();

        let body_is_empty = res.content_length().map(|x| x == 0).unwrap_or(false);
        let body = if body_is_empty {
            None
        } else {
            Some(res.json().await.map_err(|_| ValidationError::NotJson)?)
        };

        Ok(TestResponse {
            status,
            headers,
            body,
        })
    }

    async fn run_test(&self, test: ResolvedConformanceTestSpec) -> Result<TestResponse, Error> {
        debug!("request: {:?}", &test.request);
        debug!("response spec: {:?}", &test.response);

        let res = self.send_request(&test.request).await?;

        // validate response status
        test.response.validate_status(&res.status)?;

        // validate response body
        if test.response.body_validator.is_some() {
            if res.body().is_none() {
                return Err(ValidationError::NotJson.into());
            }

            test.response.validate_body(&res.body().unwrap())?;
        }

        Ok(res)
    }

    /// Runs tests in queue serially, removing them from the queue and appending the results and
    /// original test specs in the result list.
    pub async fn run_queued_tests(&mut self) {
        trace!("run queued tests");

        let spec = self.spec.clone();
        let num = Arc::new(AtomicUsize::new(self.queue.len()));

        let resolved_tests = self
            .queue
            .drain(..)
            .map(|test_spec| (test_spec.clone(), test_spec.resolve(&spec)))
            .collect::<Vec<_>>();

        let mut ok_tests = vec![];

        for (test_spec, test) in resolved_tests {
            match test {
                Ok(test) => ok_tests.push((test_spec, test)),
                Err(err) => self.results.push((test_spec, Err(err))),
            }
        }

        let mut test_results = stream::iter(ok_tests.drain(..))
            .map(|(test_spec, test)| {
                let num = Arc::clone(&num).fetch_sub(1, Ordering::SeqCst);

                println!("running test: {}", num);
                trace!("run test: {:?}", &test);

                self.run_test(test).map(|result| (test_spec, result))
            })
            .buffered(8)
            .collect::<Vec<_>>()
            .await;

        self.results.append(&mut test_results);
    }

    pub fn results(&self) -> &[TestResult] {
        &self.results
    }

    pub fn last_response_body(&self) -> Option<JsonValue> {
        let (_, res) = self.results.last().unwrap();
        let res = res.as_ref().unwrap();
        res.body()
    }

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

    pub fn clear_results(&mut self) {
        self.results.clear();
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
