use bytes::Bytes;
use http::HeaderMap;

use super::OperationSpec;

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
pub struct TestRequest {
    pub operation: OperationSpec,
    pub headers: HeaderMap,
    // pub parameters: Vec<_>,
    pub body: Bytes,
}
