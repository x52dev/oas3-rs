use bytes::Bytes;
use http::{HeaderMap, Method, StatusCode};
use lazy_static::lazy_static;
use log::{debug, error};

use crate::{validation::{SchemaValidator, Error}, Operation, Spec};

#[derive(Debug, Clone)]
pub struct OperationSpec {
    pub method: Method,
    pub path: String,
}

impl OperationSpec {
    pub fn new<P: Into<String>>(method: Method, path: P) -> Self {
        Self {
            method,
            path: path.into(),
        }
    }

    pub fn get<P: Into<String>>(path: P) -> Self { Self::new(Method::GET, path) }
    pub fn post<P: Into<String>>(path: P) -> Self { Self::new(Method::POST, path) }
    pub fn patch<P: Into<String>>(path: P) -> Self { Self::new(Method::PATCH, path) }
    pub fn put<P: Into<String>>(path: P) -> Self { Self::new(Method::PUT, path) }
    pub fn delete<P: Into<String>>(path: P) -> Self { Self::new(Method::DELETE, path) }
}

#[derive(Debug, Clone)]
pub enum RequestSource {
    Example { media_type: String, name: String },
    Raw(Bytes),
}

#[derive(Debug, Clone)]
pub struct RequestSpec {
    pub source: RequestSource,
    pub bad: bool,
}

impl RequestSpec {
    pub fn from_example<M, N>(media_type: M, name: N) -> Self
    where
        M: Into<String>,
        N: Into<String>,
    {
        Self {
            source: RequestSource::Example {
                media_type: media_type.into(),
                name: name.into(),
            },
            bad: false,
        }
    }

    pub fn from_json_example<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            source: RequestSource::Example {
                media_type: "application/json".to_owned(),
                name: name.into(),
            },
            bad: false,
        }
    }

    pub fn from_bad_raw<T>(body: T) -> Self
    where
        T: Into<Bytes>,
    {
        Self {
            source: RequestSource::Raw(body.into()),
            bad: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResponseSpecSource {
    Schema {
        status: StatusCode,
        media_type: String,
    },
    Example {
        status: StatusCode,
        media_type: String,
        name: String,
    },
    Exactly(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct ResponseSpec {
    pub source: ResponseSpecSource,
}

impl ResponseSpec {
    pub fn from_schema<M>(status: &str, media_type: M) -> Self
    where
        M: Into<String>,
    {
        Self {
            source: ResponseSpecSource::Schema {
                status: status.parse().unwrap(),
                media_type: media_type.into(),
            },
        }
    }

    pub fn from_example<M, N>(status: &str, media_type: M, name: N) -> Self
    where
        M: Into<String>,
        N: Into<String>,
    {
        Self {
            source: ResponseSpecSource::Example {
                status: status.parse().unwrap(),
                media_type: media_type.into(),
                name: name.into(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestRequest {
    pub operation: OperationSpec,
    pub headers: HeaderMap,
    // pub parameters: Vec<_>,
    pub body: Bytes,
}

#[derive(Debug, Clone)]
pub struct TestResponseSpec {
    pub operation: OperationSpec,
    pub body_validator: SchemaValidator,
}

impl TestResponseSpec {
    pub fn validate(&self, val: &serde_json::Value) -> Result<(), Error> {
        self.body_validator.validate_type(val)
    }
}

#[derive(Debug, Clone)]
pub struct ConformanceTestSpec {
    pub operation: OperationSpec,
    pub request: RequestSpec,
    pub response_spec: ResponseSpec,
}

impl ConformanceTestSpec {
    pub fn resolve(&self, spec: &Spec) -> Option<ResolvedConformanceTestSpec> {
        Some(ResolvedConformanceTestSpec {
            request: self.resolve_request(&spec)?,
            response: self.resolve_response_spec(&spec)?,
        })
    }

    pub fn resolve_operation<'a>(&self, spec: &'a Spec) -> Option<&'a Operation> {
        spec.get_operation(&self.operation.method, &self.operation.path)
    }

    pub fn resolve_request(&self, spec: &Spec) -> Option<TestRequest> {
        let op = self.resolve_operation(&spec)?;

        let req = match self.request.source {
            RequestSource::Example {
                ref media_type,
                ref name,
            } => {
                let req_body = op.get_request_body(&spec)?;
                let media_spec = req_body.content.get(media_type)?;
                let schema = media_spec.get_schema(&spec)?;
                let examples = media_spec.get_examples(&spec);
                let example = examples.get(name)?;

                if let Some(ref ex) = example.value {
                    // check example validity
                    let validator = schema.validator(&spec);

                    debug!("validating example: {:?}", &ex);
                    debug!("against schema: {:?}", &schema);
                    debug!("with validator: {:?}", &validator);

                    validator
                        .validate_type(&ex)
                        .map_err(|err| error!("{}", err))
                        .ok()?;
                }

                let mut hdrs = HeaderMap::new();
                hdrs.insert("Content-Type", media_type.clone().parse().unwrap());

                TestRequest {
                    operation: self.operation.clone(),
                    headers: hdrs,
                    body: example.as_bytes().into(),
                }
            }

            RequestSource::Raw(ref data) => {
                if !self.request.bad {
                    panic!("Raw requests are expected to be malformed. Set `bad: true` on RequestSpec.")
                }

                TestRequest {
                    operation: self.operation.clone(),
                    headers: HeaderMap::new(),
                    body: data.clone(),
                }
            }
        };

        Some(req)
    }

    pub fn resolve_response_spec(&self, spec: &Spec) -> Option<TestResponseSpec> {
        let op = self.resolve_operation(&spec)?;

        match &self.response_spec.source {
            ResponseSpecSource::Schema {
                ref status,
                ref media_type,
            } => {
                let responses = op.get_responses(&spec);
                let res = responses.get(status.as_str())?;
                let media_spec = res.content.get(media_type)?;
                let schema = media_spec.get_schema(&spec)?;

                let validator = schema.validator(&spec);

                Some(TestResponseSpec {
                    operation: self.operation.clone(),
                    body_validator: validator,
                })
            }

            ResponseSpecSource::Example { .. } => todo!(),

            ResponseSpecSource::Exactly(ref data) => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedConformanceTestSpec {
    pub request: TestRequest,
    pub response: TestResponseSpec,
}

#[cfg(test)]
mod tests {
    use super::*;

    lazy_static! {
        static ref TEST0: ConformanceTestSpec = ConformanceTestSpec {
            operation: OperationSpec::post("/token"),
            request: RequestSpec::from_example("application/json", "basic"),
            response_spec: ResponseSpec::from_schema("200", "application/json"),
        };
        static ref TEST1: ConformanceTestSpec = ConformanceTestSpec {
            operation: OperationSpec::post("/verify"),
            request: RequestSpec::from_json_example("expired"),
            response_spec: ResponseSpec::from_example("200", "application/json", "expired"),
        };
        static ref TEST2: ConformanceTestSpec = ConformanceTestSpec {
            operation: OperationSpec::get("/isloggedin"),
            request: RequestSpec::from_bad_raw("not json"),
            response_spec: ResponseSpec::from_schema("401", "application/json"),
        };
    }
}
