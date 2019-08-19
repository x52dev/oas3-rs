use std::fmt;

use http::{Method, StatusCode};

use crate::{validation::Error as ValidationError, Operation, Spec};

#[derive(Debug, Clone)]
pub enum OperationSpec {
    Parts { method: Method, path: String },
    OperationId(String),
}

impl OperationSpec {
    pub fn new<P: Into<String>>(method: Method, path: P) -> Self {
        Self::Parts {
            method,
            path: path.into(),
        }
    }

    pub fn get<P: Into<String>>(path: P) -> Self { Self::new(Method::GET, path) }
    pub fn post<P: Into<String>>(path: P) -> Self { Self::new(Method::POST, path) }
    pub fn patch<P: Into<String>>(path: P) -> Self { Self::new(Method::PATCH, path) }
    pub fn put<P: Into<String>>(path: P) -> Self { Self::new(Method::PUT, path) }
    pub fn delete<P: Into<String>>(path: P) -> Self { Self::new(Method::DELETE, path) }

    pub fn operation_id<P: Into<String>>(op_id: P) -> Self { Self::OperationId(op_id.into()) }
}

impl fmt::Display for OperationSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parts { method, path } => write!(f, "{} {}", method, path),
            Self::OperationId(op_id) => write!(f, "Operation ID: {}", op_id)
        }
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
        spec.get_operation(&self.method, &self.path).ok_or_else(|| {
            ValidationError::OperationNotFound(self.method.clone(), self.path.clone())
        })
    }
}
