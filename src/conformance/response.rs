use bytes::Bytes;
use http::StatusCode;

use super::OperationSpec;
use crate::validation::{Error as ValidationError, SchemaValidator};

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
pub struct TestResponseSpec {
    pub operation: OperationSpec,
    pub body_validator: SchemaValidator,
}

impl TestResponseSpec {
    pub fn validate(&self, val: &serde_json::Value) -> Result<(), ValidationError> {
        self.body_validator
            .validate_type(val)
            .and(self.body_validator.validate_required_fields(val))
    }
}
