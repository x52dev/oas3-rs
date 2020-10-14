use std::fmt;

use http::{header, HeaderMap, HeaderValue};

use crate::conformance::TestRequest;

#[derive(Clone)]
pub enum TestAuthentication {
    Bearer(String),
    Headers(HeaderMap),
    Custom(fn(TestRequest) -> TestRequest),
}

impl TestAuthentication {
    /// Use the `Authorization: Bearer ...` method to provide authentication key/token.
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer(token.into())
    }

    /// Shorthand for setting cookie header.
    pub fn cookie(cookies: Vec<impl AsRef<str>>) -> Self {
        let headers: HeaderMap = cookies
            .into_iter()
            .map(|cookie| {
                (
                    header::COOKIE,
                    HeaderValue::from_str(cookie.as_ref()).unwrap(),
                )
            })
            .collect();

        Self::Headers(headers)
    }

    /// Provide a closure that transforms a `TestRequest` into an authenticated `TestRequest`.
    pub fn custom(closure: fn(TestRequest) -> TestRequest) -> Self {
        Self::Custom(closure)
    }
}

impl fmt::Debug for TestAuthentication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(_) => write!(f, "[custom auth transformer]"),
            other => write!(f, "{:?}", other),
        }
    }
}
