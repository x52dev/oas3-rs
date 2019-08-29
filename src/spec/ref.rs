use std::{fs::File, io::Read, path::Path, result::Result as StdResult, str::FromStr};

use derive_more::Display;
use lazy_static::lazy_static;
use regex::Regex;

use super::Spec;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ObjectOrReference<T> {
    Ref {
        #[serde(rename = "$ref")]
        ref_path: String,
    },
    Object(T),
}

impl<T> ObjectOrReference<T>
where
    T: FromRef,
{
    pub fn resolve(&self, spec: &Spec) -> StdResult<T, RefError> {
        match self {
            Self::Object(component) => Ok(component.clone()),
            Self::Ref { ref_path } => T::from_ref(&spec, &ref_path),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Error)]
pub enum RefError {
    #[error(display = "Invalid type: {}", _0)]
    InvalidType(String),

    #[error(display = "Mismatched type: cannot reference a {} as a {}", _0, _1)]
    MismatchedType(RefType, RefType),

    // TODO: use some kind of path structure
    #[error(display = "Unresolvable path: {}", _0)]
    Unresolvable(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Display)]
pub enum RefType {
    Schema,
    Response,
    Parameter,
    Example,
    RequestBody,
    Header,
    SecurityScheme,
    Link,
    Callback,
}

impl FromStr for RefType {
    type Err = RefError;

    fn from_str(typ: &str) -> Result<Self, Self::Err> {
        Ok(match typ {
            "schemas" => Self::Schema,
            "responses" => Self::Response,
            "parameters" => Self::Parameter,
            "examples" => Self::Example,
            "requestBodies" => Self::RequestBody,
            "headers" => Self::Header,
            "securitySchemes" => Self::SecurityScheme,
            "links" => Self::Link,
            "callbacks" => Self::Callback,
            typ => return Err(RefError::InvalidType(typ.to_owned())),
        })
    }
}

pub struct RefPath {
    pub source: String,
    pub kind: RefType,
    pub name: String,
}

impl FromStr for RefPath {
    type Err = RefError;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new("^(?P<source>[^#]*)#/components/(?P<type>[^/]+)/(?P<name>.+)$").unwrap();
        }

        let parts = RE.captures(path).unwrap();

        trace!("creating RefPath: {}/{}", &parts["type"], &parts["name"]);

        Ok(Self {
            source: parts["source"].to_owned(),
            kind: parts["type"].parse()?,
            name: parts["name"].to_owned(),
        })
    }
}

pub trait FromRef: Clone {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError>;
}
