use bytes::Bytes;
use http::{HeaderMap, HeaderValue, Method, StatusCode};
use lazy_static::lazy_static;
use log::{debug, error};

use crate::{
    validation::{Error as ValidationError, SchemaValidator},
    Error, Operation, Spec,
};

use super::{
    OperationSpec, RequestSource, RequestSpec, ResponseSpec, ResponseSpecSource, TestAuthorization,
    TestOperation, TestRequest, TestResponseSpec,
};

#[derive(Debug, Clone)]
pub struct ConformanceTestSpec {
    pub operation: OperationSpec,
    pub request: RequestSpec,
    pub response_spec: ResponseSpec,
}

impl ConformanceTestSpec {
    pub fn new(op: OperationSpec, req: RequestSpec, res: ResponseSpec) -> Self {
        Self {
            operation: op,
            request: req,
            response_spec: res,
        }
    }

    pub fn resolve(&self, spec: &Spec) -> Result<ResolvedConformanceTestSpec, Error> {
        Ok(ResolvedConformanceTestSpec {
            unresolved: self.clone(),
            request: self.resolve_request(&spec)?,
            response: self.resolve_response_spec(&spec)?,
        })
    }

    pub fn resolve_test_operation<'a>(&self, spec: &'a Spec) -> Result<TestOperation, Error> {
        let test_op = match &self.operation {
            OperationSpec::Parts { method, path } => {
                TestOperation::new(method.clone(), path.clone())
            }

            OperationSpec::OperationId(op_id) => spec
                .iter_operations()
                .find(|(path, method, op)| {
                    op.operation_id
                        .as_ref()
                        .map(|id| id == op_id)
                        .unwrap_or(false)
                })
                .map(|(path, method, op)| TestOperation::new(method.clone(), path.clone()))
                .ok_or_else(|| ValidationError::OperationIdNotFound(op_id.clone()))?,
        };

        Ok(test_op)
    }

    pub fn resolve_request(&self, spec: &Spec) -> Result<TestRequest, Error> {
        let test_op = self.resolve_test_operation(&spec)?;
        let op = test_op.resolve_operation(&spec)?;

        let mut req = match self.request.source {
            RequestSource::Empty => TestRequest {
                operation: test_op.clone(),
                headers: HeaderMap::new(),
                body: Bytes::new(),
            },

            RequestSource::Raw(ref data) => {
                if !self.request.bad {
                    panic!("Raw requests are expected to be malformed. Set `bad: true` on RequestSpec.")
                }

                TestRequest {
                    operation: test_op.clone(),
                    headers: HeaderMap::new(),
                    body: data.clone(),
                }
            }

            RequestSource::Example {
                ref media_type,
                ref name,
            } => {
                let req_body = op.get_request_body(&spec)?;
                let media_spec = req_body.content.get(media_type).ok_or(Error::Placeholder)?;
                let schema = media_spec.get_schema(&spec)?;
                let examples = media_spec.get_examples(&spec);
                let example = examples.get(name).ok_or(Error::Placeholder)?;

                if let Some(ref ex) = example.value {
                    // check example validity
                    let validator = schema.validator(&spec);

                    debug!("validating example: {:?}", &ex);
                    debug!("against schema: {:?}", &schema);
                    debug!("with validator: {:?}", &validator);

                    validator.validate_type(&ex).map_err(Error::Validation)?;
                }

                let mut hdrs = HeaderMap::new();
                hdrs.insert("Content-Type", media_type.clone().parse().unwrap());

                TestRequest {
                    operation: test_op.clone(),
                    headers: hdrs,
                    body: example.as_bytes().into(),
                }
            }
        };

        match self.request.auth {
            Some(TestAuthorization::Bearer(ref jwt)) => {
                let val = format!("Bearer {}", jwt);
                req.headers.append(
                    "Authorization",
                    HeaderValue::from_str(&val).expect("invalid header value"),
                );
            }
            _ => {}
        }

        Ok(req)
    }

    pub fn resolve_response_spec(&self, spec: &Spec) -> Result<TestResponseSpec, Error> {
        let test_op = self.resolve_test_operation(&spec)?;
        let op = test_op.resolve_operation(&spec)?;

        let res_spec = match &self.response_spec.source {
            ResponseSpecSource::Status(status) => TestResponseSpec {
                operation: test_op.clone(),
                status: status.clone(),
                body_validator: None,
            },

            ResponseSpecSource::Schema { status, media_type } => {
                // traverse spec
                let responses = op.get_responses(&spec);
                let status_spec = responses.get(status.as_str()).ok_or(Error::Placeholder)?;
                let media_spec = status_spec
                    .content
                    .get(media_type)
                    .ok_or(Error::Placeholder)?;
                let schema = media_spec.get_schema(&spec)?;

                // create validator
                let validator = schema.validator(&spec);

                TestResponseSpec {
                    operation: test_op.clone(),
                    status: status.clone(),
                    body_validator: Some(validator),
                }
            }

            ResponseSpecSource::Example {
                status,
                media_type,
                name,
            } => {
                // traverse spec
                let reses = op.get_responses(&spec);
                let status_spec = reses.get(status.as_str()).ok_or(Error::Placeholder)?;
                let media_spec = status_spec
                    .content
                    .get(media_type)
                    .ok_or(Error::Placeholder)?;
                let schema = media_spec.get_schema(&spec)?;
                let examples = media_spec.get_examples(&spec);
                let example = examples.get(name).ok_or(Error::Placeholder)?;

                // create validator
                let validator = schema.validator(&spec);

                if let Some(ref ex) = example.value {
                    // check example validity

                    debug!("validating example: {:?}", &ex);
                    debug!("against schema: {:?}", &schema);
                    debug!("with validator: {:?}", &validator);

                    validator.validate_type(&ex).map_err(Error::Validation)?;
                }

                let mut hdrs = HeaderMap::new();
                hdrs.insert("Content-Type", media_type.clone().parse().unwrap());

                TestResponseSpec {
                    operation: test_op.clone(),
                    status: status.clone(),
                    body_validator: Some(validator),
                }
            }

            ResponseSpecSource::Exactly(ref data) => todo!(),
        };

        Ok(res_spec)
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedConformanceTestSpec {
    pub unresolved: ConformanceTestSpec,
    pub request: TestRequest,
    pub response: TestResponseSpec,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_conformance_test_spec() {
        let test0: ConformanceTestSpec = ConformanceTestSpec {
            operation: OperationSpec::post("/token"),
            request: RequestSpec::from_example("application/json", "basic"),
            response_spec: ResponseSpec::from_schema("200", "application/json"),
        };

        let test1: ConformanceTestSpec = ConformanceTestSpec {
            operation: OperationSpec::post("/verify"),
            request: RequestSpec::from_json_example("expired"),
            response_spec: ResponseSpec::from_example("200", "application/json", "expired"),
        };

        let test2: ConformanceTestSpec = ConformanceTestSpec {
            operation: OperationSpec::get("/isloggedin"),
            request: RequestSpec::from_bad_raw("not json"),
            response_spec: ResponseSpec::from_schema("401", "application/json"),
        };
    }
}
