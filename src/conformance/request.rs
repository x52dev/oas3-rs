use bytes::Bytes;
use http::HeaderMap;

use super::{ParamReplacement, TestAuthentication, TestOperation, TestParam};

#[derive(Debug, Clone)]
pub enum RequestSource {
    Example { media_type: String, name: String },
    Raw(Bytes),
    Empty,
}

#[derive(Debug, Clone)]
pub struct RequestSpec {
    pub source: RequestSource,
    pub bad: bool,
    pub auth: Option<TestAuthentication>,
    pub params: Vec<ParamReplacement>,
    pub content_type_override: Option<String>,
}

impl RequestSpec {
    pub fn empty() -> Self {
        Self {
            source: RequestSource::Empty,
            bad: false,
            auth: None,
            params: vec![],
            content_type_override: None,
        }
    }

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
            ..Self::empty()
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
            ..Self::empty()
        }
    }

    pub fn from_bad_raw<T>(body: T) -> Self
    where
        T: Into<Bytes>,
    {
        Self {
            source: RequestSource::Raw(body.into()),
            bad: true,
            ..Self::empty()
        }
    }

    pub fn override_content_type<T>(self, ct: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            content_type_override: Some(ct.into()),
            ..self
        }
    }

    pub fn with_auth(self, auth: &TestAuthentication) -> Self {
        Self {
            auth: Some(auth.clone()),
            ..self
        }
    }

    pub fn no_auth(self) -> Self {
        Self { auth: None, ..self }
    }

    pub fn add_param<N, V>(mut self, name: N, val: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        let param = ParamReplacement::new(name, val);
        self.params.push(param);
        self
    }
}

#[derive(Debug, Clone)]
pub struct TestRequest {
    pub operation: TestOperation,
    pub headers: HeaderMap,
    pub params: Vec<TestParam>,
    pub body: Bytes,
}
