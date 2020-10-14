use derive_more::Display;
use http::Method;
use log::debug;

use crate::{spec::Operation, validation::Error as ValidationError, Spec};

#[derive(Debug, Clone, Display)]
pub enum OperationSpec {
    #[display(fmt = "{} {}", method, path)]
    Parts { method: Method, path: String },

    #[display(fmt = "OpID: {}", _0)]
    OperationId(String),
}

impl OperationSpec {
    pub fn new<P: Into<String>>(method: Method, path: P) -> Self {
        Self::Parts {
            method,
            path: path.into(),
        }
    }

    pub fn get<P: Into<String>>(path: P) -> Self {
        Self::new(Method::GET, path)
    }
    pub fn post<P: Into<String>>(path: P) -> Self {
        Self::new(Method::POST, path)
    }
    pub fn patch<P: Into<String>>(path: P) -> Self {
        Self::new(Method::PATCH, path)
    }
    pub fn put<P: Into<String>>(path: P) -> Self {
        Self::new(Method::PUT, path)
    }
    pub fn delete<P: Into<String>>(path: P) -> Self {
        Self::new(Method::DELETE, path)
    }

    pub fn operation_id<P: Into<String>>(op_id: P) -> Self {
        Self::OperationId(op_id.into())
    }
}

#[derive(Debug, Clone)]
pub struct TestOperation {
    pub method: Method,
    pub path: String,
}

impl TestOperation {
    pub fn new<P: Into<String>>(method: Method, path: P) -> Self {
        Self {
            method,
            path: path.into(),
        }
    }

    pub fn resolve_operation<'a>(&self, spec: &'a Spec) -> Result<&'a Operation, ValidationError> {
        debug!("resolving op {:?}", &self);

        spec.operation(&self.method, &self.path).ok_or_else(|| {
            ValidationError::OperationNotFound(self.method.clone(), self.path.clone())
        })
    }
}
