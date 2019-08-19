use http::{Method, StatusCode};

use crate::{validation::Error as ValidationError, Operation, Spec};


#[derive(Debug, Clone)]
pub enum TestAuthorization {
    Bearer(String)
}

impl TestAuthorization {
    pub fn bearer<T: Into<String>>(token: T) -> Self {
        Self::Bearer(token.into())
    }
}
