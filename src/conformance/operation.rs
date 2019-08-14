use http::{Method, StatusCode};

#[derive(Debug, Clone)]
pub struct OperationSpec {
    pub method: Method,
    pub path: String,
}

impl OperationSpec {
    pub fn new<P: Into<String>>(method: Method, path: P) -> Self {
        Self {
            method,
            path: path.into(),
        }
    }

    pub fn get<P: Into<String>>(path: P) -> Self { Self::new(Method::GET, path) }
    pub fn post<P: Into<String>>(path: P) -> Self { Self::new(Method::POST, path) }
    pub fn patch<P: Into<String>>(path: P) -> Self { Self::new(Method::PATCH, path) }
    pub fn put<P: Into<String>>(path: P) -> Self { Self::new(Method::PUT, path) }
    pub fn delete<P: Into<String>>(path: P) -> Self { Self::new(Method::DELETE, path) }
}
