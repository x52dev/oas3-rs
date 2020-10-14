use bytes::Bytes;
use http::HeaderMap;
use log::{debug, trace};

use crate::{
    spec::{Error as SpecError, RefError},
    validation::{Error as ValidationError, ValidationTree},
    Error, Spec,
};

use super::{
    OperationSpec, ParamPosition, RequestSource, RequestSpec, ResponseSpec, ResponseSpecSource,
    TestAuthentication, TestOperation, TestParam, TestRequest, TestResponseSpec,
};

#[derive(Debug, Clone)]
pub struct ConformanceTestSpec {
    pub name: Option<String>,
    pub operation: OperationSpec,
    pub request: RequestSpec,
    pub response_spec: ResponseSpec,
}

impl ConformanceTestSpec {
    pub fn new(op: OperationSpec, req: RequestSpec, res: ResponseSpec) -> Self {
        Self {
            name: None,
            operation: op,
            request: req,
            response_spec: res,
        }
    }

    pub fn named<T: Into<String>>(
        name: T,
        op: OperationSpec,
        req: RequestSpec,
        res: ResponseSpec,
    ) -> Self {
        Self {
            name: Some(name.into()),
            ..Self::new(op, req, res)
        }
    }

    pub fn named_success<T: Into<String>>(name: T, op: OperationSpec) -> Self {
        Self {
            name: Some(name.into()),
            ..Self::new(op, RequestSpec::empty(), ResponseSpec::from_status(200))
        }
    }

    pub fn with_auth(self, auth: &TestAuthentication) -> Self {
        Self {
            request: self.request.with_auth(auth),
            ..self
        }
    }

    pub fn resolve(&self, spec: &Spec) -> Result<ResolvedConformanceTestSpec, Error> {
        trace!("resolving: {:?}", &self.operation);

        let mut req = self.resolve_request(&spec)?;

        if let Some(TestAuthentication::Custom(transformer)) = self.request.auth {
            req = transformer(req);
        };

        Ok(ResolvedConformanceTestSpec {
            unresolved: self.clone(),
            request: req,
            response: self.resolve_response_spec(&spec)?,
        })
    }

    pub fn resolve_test_operation<'a>(&self, spec: &'a Spec) -> Result<TestOperation, Error> {
        trace!("resolve_test_operation: {:?}", &self.operation);

        let test_op = match &self.operation {
            OperationSpec::Parts { method, path } => {
                TestOperation::new(method.clone(), path.clone())
            }

            OperationSpec::OperationId(op_id) => spec
                .operations()
                .find(|(path, method, op)| {
                    trace!("checking {} {} ({:?})", &method, &path, &op);

                    op.operation_id
                        .as_ref()
                        .map(|id| id == op_id)
                        .unwrap_or(false)
                })
                .map(|(path, method, _op)| TestOperation::new(method.clone(), path.clone()))
                .ok_or_else(|| ValidationError::OperationIdNotFound(op_id.clone()))?,
        };

        Ok(test_op)
    }

    pub fn resolve_params(&self, spec: &Spec) -> Result<Vec<TestParam>, Error> {
        let test_op = self.resolve_test_operation(&spec)?;
        let op = test_op.resolve_operation(&spec)?;

        let mut test_params = vec![];

        // iterate params
        for param in &self.request.params {
            // resolve in spec
            let parameter = op
                .parameter(&param.name, &spec)?
                .ok_or(ValidationError::ParameterNotFound(param.name.clone()))?;

            // validate position
            let pos = match parameter.location.as_ref() {
                "query" => ParamPosition::Query,
                "header" => ParamPosition::Header,
                "path" => ParamPosition::Path,
                "cookie" => ParamPosition::Cookie,
                pos_str => Err(ValidationError::InvalidParameterLocation(
                    pos_str.to_owned(),
                ))?,
            };

            // TODO: validate type
            // TODO: validate other spec options

            // insert into test params
            let test_param = TestParam::new(&param.name, &param.value, pos);
            test_params.push(test_param);

            // mark param as used for redundancy checks later
            param.used.replace(true);
        }

        // TODO: check params for unused and report
        // TODO: check spec params for unreplaced and report if not required

        Ok(test_params)
    }

    pub fn resolve_request(&self, spec: &Spec) -> Result<TestRequest, Error> {
        trace!("resolving request: {:?}", &self.operation);

        let test_op = self.resolve_test_operation(&spec)?;
        let op = test_op.resolve_operation(&spec)?;

        let mut req = match self.request.source {
            RequestSource::Empty => TestRequest {
                operation: test_op.clone(),
                headers: HeaderMap::new(),
                params: self.resolve_params(&spec)?,
                body: Bytes::new(),
            },

            RequestSource::Raw(ref data) => {
                if !self.request.bad {
                    panic!("Raw requests are expected to be malformed. Set `bad: true` on RequestSpec.")
                }

                TestRequest {
                    operation: test_op.clone(),
                    headers: HeaderMap::new(),
                    params: self.resolve_params(&spec)?,
                    body: data.clone(),
                }
            }

            RequestSource::Example {
                ref media_type,
                ref name,
            } => {
                let req_body = op.request_body(&spec)?;
                let media_spec = req_body.content.get(media_type).ok_or(SpecError::Ref(
                    RefError::Unresolvable(format!("mediaType/{}", &name)),
                ))?;
                let schema = media_spec.schema(&spec)?;
                let examples = media_spec.examples(&spec);
                let example = examples
                    .get(name)
                    .ok_or(SpecError::Ref(RefError::Unresolvable(format!(
                        "example/{}",
                        &name
                    ))))?;

                if let Some(ref ex) = example.value {
                    // check example validity
                    let validator = ValidationTree::from_schema(&schema, spec)?;

                    debug!("validating example: {:?}", &ex);
                    debug!("against schema: {:?}", &schema);
                    debug!("with validator: {:?}", &validator);

                    // TODO: restore
                    // validator.validate(&ex)?;
                }

                let mut hdrs = HeaderMap::new();
                hdrs.insert("Content-Type", media_type.clone().parse().unwrap());

                TestRequest {
                    operation: test_op.clone(),
                    headers: hdrs,
                    params: self.resolve_params(&spec)?,
                    body: example.as_bytes().into(),
                }
            }
        };

        if let Some(TestAuthentication::Bearer(ref jwt)) = self.request.auth {
            let val = format!("Bearer {}", jwt);
            req.headers
                .insert("Authorization", val.parse().expect("invalid auth token"));
        }

        if let Some(ct) = self.request.content_type_override.as_ref() {
            req.headers
                .insert("Content-Type", ct.parse().expect("invalid content type"));
        }

        Ok(req)
    }

    pub fn resolve_response_spec(&self, spec: &Spec) -> Result<TestResponseSpec, Error> {
        let test_op = self.resolve_test_operation(&spec)?;
        let op = test_op.resolve_operation(&spec)?;

        let res_spec =
            match &self.response_spec.source {
                ResponseSpecSource::Status(status) => TestResponseSpec {
                    operation: test_op.clone(),
                    status: status.clone(),
                    body_validator: None,
                },

                ResponseSpecSource::Schema { status, media_type } => {
                    // traverse spec
                    let responses = op.responses(&spec);
                    let status_spec = responses.get(status.as_str()).ok_or(SpecError::Ref(
                        RefError::Unresolvable(format!("status/{}", &status.as_u16())),
                    ))?;
                    let media_spec = status_spec.content.get(media_type).ok_or(SpecError::Ref(
                        RefError::Unresolvable(format!("mediaType/{}", &media_type)),
                    ))?;
                    let schema = media_spec.schema(&spec)?;

                    // create validator
                    let validator = ValidationTree::from_schema(&schema, spec)?;

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
                    let reses = op.responses(&spec);
                    let status_spec = reses.get(status.as_str()).ok_or(SpecError::Ref(
                        RefError::Unresolvable(format!("status/{}", &status.as_u16())),
                    ))?;
                    let media_spec = status_spec.content.get(media_type).ok_or(SpecError::Ref(
                        RefError::Unresolvable(format!("mediaType/{}", &media_type)),
                    ))?;
                    let schema = media_spec.schema(&spec)?;
                    let examples = media_spec.examples(&spec);
                    let example =
                        examples
                            .get(name)
                            .ok_or(SpecError::Ref(RefError::Unresolvable(format!(
                                "example/{}",
                                &name
                            ))))?;

                    // create validator
                    let validator = ValidationTree::from_schema(&schema, spec)?;

                    if let Some(ref ex) = example.value {
                        // check example validity

                        debug!("validating example: {:?}", &ex);
                        debug!("against schema: {:?}", &schema);
                        debug!("with validator: {:?}", &validator);

                        validator.validate(&ex).map_err(Error::Validation)?;
                    }

                    let mut hdrs = HeaderMap::new();
                    hdrs.insert("Content-Type", media_type.clone().parse().unwrap());

                    TestResponseSpec {
                        operation: test_op.clone(),
                        status: status.clone(),
                        body_validator: Some(validator),
                    }
                }

                ResponseSpecSource::Exactly(ref _data) => todo!(),
            };

        Ok(res_spec)
    }
}

#[derive(Debug)]
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
        ConformanceTestSpec::named(
            "basic login",
            OperationSpec::post("/token"),
            RequestSpec::from_example("application/json", "basic"),
            ResponseSpec::from_schema(200, "application/json"),
        );

        ConformanceTestSpec::named(
            "verify expired",
            OperationSpec::post("/verify"),
            RequestSpec::from_json_example("expired"),
            ResponseSpec::from_example(200, "application/json", "expired"),
        );

        ConformanceTestSpec::named(
            "is not logged in",
            OperationSpec::get("/isloggedin"),
            RequestSpec::from_bad_raw("not json"),
            ResponseSpec::from_schema(401, "application/json"),
        );
    }
}
