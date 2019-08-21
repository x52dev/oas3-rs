use http::{HeaderMap, Method, StatusCode};
use reqwest::Request;

use crate::{validation::Error as ValidationError, Operation, Spec};

#[derive(Debug, Clone)]
pub enum TestAuthorization {
    Bearer(String),
    Headers(HeaderMap),
}

impl TestAuthorization {
    pub fn bearer<T: Into<String>>(token: T) -> Self { Self::Bearer(token.into()) }

    // /// Shorthand for setting cookie headers.
    // pub fn cookies<T: Into<String>>(token: T) -> Self {
    //     Self::Bearer(token.into())
    // }
}
