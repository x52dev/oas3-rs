use http::{HeaderMap, StatusCode};
use serde_json::Value as JsonValue;

use super::TestOperation;
use crate::validation::{Error as ValidationError, ValidationTree};

#[derive(Debug, Clone)]
pub enum ResponseSpecSource {
    Status(StatusCode),
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
    pub fn from_status(status: u16) -> Self {
        Self {
            source: ResponseSpecSource::Status(
                StatusCode::from_u16(status).expect("invalid status code"),
            ),
        }
    }

    pub fn from_schema<M>(status: u16, media_type: M) -> Self
    where
        M: Into<String>,
    {
        Self {
            source: ResponseSpecSource::Schema {
                status: StatusCode::from_u16(status).expect("invalid status code"),
                media_type: media_type.into(),
            },
        }
    }

    pub fn from_json_schema(status: u16) -> Self {
        Self {
            source: ResponseSpecSource::Schema {
                status: StatusCode::from_u16(status).expect("invalid status code"),
                media_type: "application/json".to_owned(),
            },
        }
    }

    pub fn from_example<M, N>(status: u16, media_type: M, name: N) -> Self
    where
        M: Into<String>,
        N: Into<String>,
    {
        Self {
            source: ResponseSpecSource::Example {
                status: StatusCode::from_u16(status).expect("invalid status code"),
                media_type: media_type.into(),
                name: name.into(),
            },
        }
    }
}

#[derive(Debug)]
pub struct TestResponseSpec {
    pub operation: TestOperation,
    pub status: StatusCode,
    pub body_validator: Option<ValidationTree>,
}

impl TestResponseSpec {
    pub fn validate_status(&self, val: &StatusCode) -> Result<(), ValidationError> {
        if &self.status == val {
            Ok(())
        } else {
            Err(ValidationError::StatusMismatch(
                self.status.clone(),
                val.clone(),
            ))
        }
    }

    pub fn validate_body(&self, body: &JsonValue) -> Result<(), ValidationError> {
        if let Some(ref validator) = self.body_validator {
            validator.validate(body)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Option<JsonValue>,
}

impl TestResponse {
    pub fn body(&self) -> Option<JsonValue> {
        self.body.clone()
    }
}
