use std::{collections::BTreeMap, error::Error as StdError};

use serde::{Deserialize, Serialize};

use crate::spec::{
    r#ref::{FromRef, Ref, RefError, RefType},
    PathItem, Spec,
};

/// Map of possible out-of band callbacks related to the parent operation.
///
/// Each value in the map is a [Path Item Object] that describes a set of requests that may be
/// initiated by the API provider and the expected responses.
///
/// See <https://spec.openapis.org/oas/v3.1.1#callback-object>.
///
/// NB: this structure is flattened when serializing and unflattened when deserializing in order to
/// support spec extensions. I.e., `paths` is a synthetic property within the data tree that
/// comprises an OpenAPI document.
///
/// [Path Item Object]: https://spec.openapis.org/oas/v3.1.1#path-item-object
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "CallbackSerde", into = "CallbackSerde")]
pub struct Callback {
    /// Map of [Path Item Object]s that describe a set of requests that may be initiated by the API
    /// provider and the expected responses.
    ///
    /// The key value used to identify the [Path Item Object] is an expression, evaluated at
    /// runtime, that identifies a URL to use for the callback operation.
    ///
    /// [Path Item Object]: https://spec.openapis.org/oas/v3.1.1#path-item-object
    pub paths: BTreeMap<String, PathItem>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.1#specification-extensions>.
    pub extensions: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
struct CallbackSerde(serde_json::Map<String, serde_json::Value>);

impl TryFrom<CallbackSerde> for Callback {
    type Error = Box<dyn StdError>;

    fn try_from(CallbackSerde(map): CallbackSerde) -> Result<Self, Self::Error> {
        let (extensions, paths) = bisect_map(map, |key| key.starts_with("x-"));

        let paths = paths
            .into_iter()
            .map(|(key, value)| serde_json::from_value(value).map(|v| (key, v)))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            paths,
            extensions: extensions.into_iter().collect(),
        })
    }
}

fn bisect_map(
    map: serde_json::Map<String, serde_json::Value>,
    predicate: fn(&String) -> bool,
) -> (
    serde_json::Map<String, serde_json::Value>,
    serde_json::Map<String, serde_json::Value>,
) {
    let mut first = map;
    let mut second = first.clone();

    first.retain(|key, _| predicate(key));
    second.retain(|key, _| !predicate(key));

    (first, second)
}

impl From<Callback> for CallbackSerde {
    fn from(val: Callback) -> Self {
        let Callback { paths, extensions } = val;

        CallbackSerde(
            paths
                .into_iter()
                .map(|(key, val)| {
                    (
                        key,
                        serde_json::to_value(val).expect("path item serialization should not fail"),
                    )
                })
                .chain(extensions)
                .collect(),
        )
    }
}

impl FromRef for Callback {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError> {
        let refpath = path.parse::<Ref>()?;

        match refpath.kind {
            RefType::Callback => spec
                .components
                .as_ref()
                .and_then(|cs| cs.callbacks.get(&refpath.name))
                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
                .and_then(|oor| oor.resolve(spec)),

            _ => Err(RefError::MismatchedType(refpath.kind, RefType::Callback)),
        }
    }
}
