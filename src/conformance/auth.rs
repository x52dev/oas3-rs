use std::{fmt, ops::Deref};

use http::{HeaderMap, Method, StatusCode};
use reqwest::Request;

use crate::{
    conformance::TestRequest, spec::Operation, validation::Error as ValidationError, Spec,
};

#[derive(Clone)]
pub enum TestAuthentication {
    Bearer(String),
    Headers(HeaderMap),
    Custom(fn(TestRequest) -> TestRequest),
}

impl TestAuthentication {
    /// Use the `Authorization: Bearer ...` method to provide authentication key/token.
    pub fn bearer<T: Into<String>>(token: T) -> Self {
        Self::Bearer(token.into())
    }

    // /// Shorthand for setting cookie headers.
    // pub fn cookies<T: Into<String>>(token: T) -> Self {
    //     Self::Bearer(token.into())
    // }

    /// Provide a closure that transforms a `TestRequest` into an authenticated `TestRequest`.
    pub fn custom(closure: fn(TestRequest) -> TestRequest) -> Self {
        Self::Custom(closure)
    }
}

impl fmt::Debug for TestAuthentication {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Custom(_) => write!(f, "[custom auth transformer]"),
            debuggable => write!(f, "{:?}", debuggable),
        }
    }
}
