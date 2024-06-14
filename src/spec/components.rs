use std::collections::BTreeMap;

use crate::deserialize_extensions;
use serde::{Deserialize, Serialize};

use super::{
    schema::Schema, Callback, Example, Header, Link, ObjectOrReference, Parameter, PathItem,
    RequestBody, Response, SecurityScheme,
};

/// Holds a set of reusable objects for different aspects of the OAS.
///
/// All objects defined within the components object will have no effect on the API unless
/// they are explicitly referenced from properties outside the components object.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#componentsObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Components {
    /// An object to hold reusable [Schema Objects](Schema).
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub schemas: BTreeMap<String, ObjectOrReference<Schema>>,

    /// An object to hold reusable [Response Objects](Response).
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub responses: BTreeMap<String, ObjectOrReference<Response>>,

    /// An object to hold reusable [Parameter Objects](Parameter).
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, ObjectOrReference<Parameter>>,

    /// An object to hold reusable [Example Objects](Example).
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub examples: BTreeMap<String, ObjectOrReference<Example>>,

    /// An object to hold reusable [Request Body Objects](RequestBody).
    #[serde(default)]
    #[serde(rename = "requestBodies", skip_serializing_if = "BTreeMap::is_empty")]
    pub request_bodies: BTreeMap<String, ObjectOrReference<RequestBody>>,

    /// An object to hold reusable [Header Objects](Header).
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, ObjectOrReference<Header>>,

    /// An object to hold reusable [Path Item Objects](PathItem).
    #[serde(default)]
    #[serde(rename = "pathItems", skip_serializing_if = "BTreeMap::is_empty")]
    pub path_items: BTreeMap<String, ObjectOrReference<PathItem>>,

    /// An object to hold reusable [Security Scheme Objects](SecurityScheme).
    #[serde(default)]
    #[serde(rename = "securitySchemes", skip_serializing_if = "BTreeMap::is_empty")]
    pub security_schemes: BTreeMap<String, ObjectOrReference<SecurityScheme>>,

    /// An object to hold reusable [Link Objects](crate::spec::Link).
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub links: BTreeMap<String, ObjectOrReference<Link>>,

    /// An object to hold reusable [Callback Objects](crate::spec::Callback).
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub callbacks: BTreeMap<String, ObjectOrReference<Callback>>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_extensions")]
    pub extensions: serde_yaml::Value,
}
