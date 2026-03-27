//! Interpolate strings from structured data using dotted JSON-style paths.
//!
//! # Example
//!
//! ```
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Inner {
//!     field: &'static str,
//! }
//!
//! #[derive(Serialize)]
//! struct Data {
//!     interpolated: u8,
//!     inner: Inner,
//! }
//!
//! let rendered = jsonpath_fmt::render(
//!     "my-{interpolated}-string-{inner.field}",
//!     &Data {
//!         interpolated: 67,
//!         inner: Inner { field: "woahg" },
//!     },
//! )?;
//!
//! assert_eq!(rendered, "my-67-string-woahg");
//! # Ok::<(), jsonpath_fmt::Error>(())
//! ```

use derive_more::{Display, Error, From};
#[cfg(feature = "facet")]
use facet::{Facet as FacetIntrospect, Peek as FacetPeek, ScalarType};
use serde::Serialize;
use serde_json::Value;
use winnow::{prelude::*, token::take_till, ModalResult};

/// Render a template with `{path}` placeholders against any serializable value.
///
/// # Example
///
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Data {
///     greeting: &'static str,
///     user: User,
/// }
///
/// #[derive(Serialize)]
/// struct User {
///     name: &'static str,
/// }
///
/// let rendered = jsonpath_fmt::render(
///     "{greeting}, {user.name}!",
///     &Data {
///         greeting: "hello",
///         user: User { name: "Ada" },
///     },
/// )?;
///
/// assert_eq!(rendered, "hello, Ada!");
/// # Ok::<(), jsonpath_fmt::Error>(())
/// ```
pub fn render<T>(template: impl AsRef<str>, data: &T) -> Result<String, Error>
where
    T: Serialize,
{
    Template::parse(template)?.render(data)
}

/// Render a template with `{path}` placeholders against a [`serde_json::Value`].
///
/// # Example
///
/// ```
/// use serde_json::json;
///
/// let rendered = jsonpath_fmt::render_value(
///     "id={id}; item={items.0}",
///     &json!({
///         "id": 42,
///         "items": ["first", "second"]
///     }),
/// )?;
///
/// assert_eq!(rendered, "id=42; item=first");
/// # Ok::<(), jsonpath_fmt::Error>(())
/// ```
pub fn render_value(template: impl AsRef<str>, data: &Value) -> Result<String, Error> {
    Template::parse(template)?.render_value(data)
}

/// Render a template with `{path}` placeholders against a value introspected with `facet`.
///
/// This avoids converting the entire input into a `serde_json::Value` before path resolution.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "facet")] {
/// use facet::Facet;
///
/// #[derive(Facet)]
/// struct Data {
///     interpolated: u8,
///     inner: Inner,
/// }
///
/// #[derive(Facet)]
/// struct Inner {
///     field: &'static str,
/// }
///
/// let rendered = jsonpath_fmt::render_facet(
///     "my-{interpolated}-string-{inner.field}",
///     &Data {
///         interpolated: 67,
///         inner: Inner { field: "woahg" },
///     },
/// )?;
///
/// assert_eq!(rendered, "my-67-string-woahg");
/// # }
/// # Ok::<(), jsonpath_fmt::Error>(())
/// ```
#[cfg(feature = "facet")]
pub fn render_facet<'facet, T>(template: impl AsRef<str>, data: &T) -> Result<String, Error>
where
    T: FacetIntrospect<'facet> + ?Sized,
{
    Template::parse(template)?.render_facet(data)
}

/// A parsed template that can be rendered repeatedly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    segments: Vec<Segment>,
}

impl Template {
    /// Parse a template containing `{path}` placeholders.
    ///
    /// # Example
    ///
    /// ```
    /// let template = jsonpath_fmt::Template::parse("{{prefix}}-{user.name}")?;
    ///
    /// let rendered = template.render_value(&serde_json::json!({
    ///     "user": { "name": "Ada" }
    /// }))?;
    ///
    /// assert_eq!(rendered, "{prefix}-Ada");
    /// # Ok::<(), jsonpath_fmt::Error>(())
    /// ```
    pub fn parse(template: impl AsRef<str>) -> Result<Self, Error> {
        let template = template.as_ref();
        let mut input = template;
        let mut segments = Vec::new();

        while !input.is_empty() {
            let literal =
                take_until_brace(&mut input).expect("complete string parsing cannot fail");

            push_literal(&mut segments, literal);

            if input.is_empty() {
                break;
            }

            let position = template.len() - input.len();

            if consume_escaped_open_brace(&mut input).is_ok() {
                push_literal(&mut segments, "{");
                continue;
            }

            if consume_escaped_close_brace(&mut input).is_ok() {
                push_literal(&mut segments, "}");
                continue;
            }

            if consume_open_brace(&mut input).is_ok() {
                let expr =
                    take_until_brace(&mut input).expect("complete string parsing cannot fail");

                if input.is_empty() {
                    return Err(ParseError::UnmatchedOpeningBrace { position }.into());
                }

                if consume_open_brace(&mut input).is_ok() {
                    return Err(ParseError::NestedOpeningBrace { position }.into());
                }

                consume_close_brace(&mut input)
                    .expect("placeholder contents must be followed by a brace");

                segments.push(Segment::Expression(Path::parse(position, expr)?));
                continue;
            }

            return Err(ParseError::UnmatchedClosingBrace { position }.into());
        }

        Ok(Self { segments })
    }

    /// Render a parsed template against any serializable value.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Data {
    ///     id: u32,
    /// }
    ///
    /// let template = jsonpath_fmt::Template::parse("item-{id}")?;
    /// let rendered = template.render(&Data { id: 7 })?;
    ///
    /// assert_eq!(rendered, "item-7");
    /// # Ok::<(), jsonpath_fmt::Error>(())
    /// ```
    pub fn render<T>(&self, data: &T) -> Result<String, Error>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(data)?;
        self.render_value(&value)
    }

    /// Render a parsed template against a [`serde_json::Value`].
    ///
    /// # Example
    ///
    /// ```
    /// let template = jsonpath_fmt::Template::parse("item={items.1.name}")?;
    /// let rendered = template.render_value(&serde_json::json!({
    ///     "items": [
    ///         { "name": "first" },
    ///         { "name": "second" }
    ///     ]
    /// }))?;
    ///
    /// assert_eq!(rendered, "item=second");
    /// # Ok::<(), jsonpath_fmt::Error>(())
    /// ```
    pub fn render_value(&self, data: &Value) -> Result<String, Error> {
        let mut rendered = String::new();

        for segment in &self.segments {
            match segment {
                Segment::Literal(text) => rendered.push_str(text),
                Segment::Expression(path) => {
                    let value = path.resolve(data)?;
                    rendered.push_str(&stringify_value(value)?);
                }
            }
        }

        Ok(rendered)
    }

    /// Render a parsed template against a value introspected with `facet`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "facet")] {
    /// use facet::Facet;
    ///
    /// #[derive(Facet)]
    /// struct Data {
    ///     items: Vec<Item>,
    /// }
    ///
    /// #[derive(Facet)]
    /// struct Item {
    ///     name: &'static str,
    /// }
    ///
    /// let template = jsonpath_fmt::Template::parse("item={items.1.name}")?;
    /// let rendered = template.render_facet(&Data {
    ///     items: vec![Item { name: "first" }, Item { name: "second" }],
    /// })?;
    ///
    /// assert_eq!(rendered, "item=second");
    /// # }
    /// # Ok::<(), jsonpath_fmt::Error>(())
    /// ```
    #[cfg(feature = "facet")]
    pub fn render_facet<'facet, T>(&self, data: &T) -> Result<String, Error>
    where
        T: FacetIntrospect<'facet> + ?Sized,
    {
        self.render_facet_peek(FacetPeek::new(data))
    }

    #[cfg(feature = "facet")]
    fn render_facet_peek<'mem, 'facet>(
        &self,
        data: FacetPeek<'mem, 'facet>,
    ) -> Result<String, Error> {
        let mut rendered = String::new();

        for segment in &self.segments {
            match segment {
                Segment::Literal(text) => rendered.push_str(text),
                Segment::Expression(path) => {
                    let value = path.resolve_facet(data)?;
                    rendered.push_str(&stringify_facet_value(value)?);
                }
            }
        }

        Ok(rendered)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Literal(String),
    Expression(Path),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Path {
    raw: String,
    segments: Vec<String>,
}

impl Path {
    fn parse(position: usize, expr: &str) -> Result<Self, Error> {
        let raw = expr.trim();
        if raw.is_empty() {
            return Err(ParseError::EmptyPlaceholder { position }.into());
        }

        let mut input = raw;
        let mut segments = Vec::new();

        loop {
            let segment = take_until_dot(&mut input).expect("complete string parsing cannot fail");
            let segment = segment.trim();

            if segment.is_empty() {
                return Err(ParseError::InvalidPath {
                    position,
                    path: raw.to_owned(),
                }
                .into());
            }

            segments.push(segment.to_owned());

            if consume_dot(&mut input).is_err() {
                break;
            }

            if input.is_empty() {
                return Err(ParseError::InvalidPath {
                    position,
                    path: raw.to_owned(),
                }
                .into());
            }
        }

        Ok(Self {
            raw: raw.to_owned(),
            segments,
        })
    }

    fn resolve<'a>(&self, data: &'a Value) -> Result<&'a Value, Error> {
        let mut current = data;

        for segment in &self.segments {
            current = match current {
                Value::Object(map) => map.get(segment),
                Value::Array(values) => segment
                    .parse::<usize>()
                    .ok()
                    .and_then(|idx| values.get(idx)),
                _ => None,
            }
            .ok_or_else(|| MissingPathError {
                path: self.raw.clone(),
            })?;
        }

        Ok(current)
    }

    #[cfg(feature = "facet")]
    fn resolve_facet<'mem, 'facet>(
        &self,
        data: FacetPeek<'mem, 'facet>,
    ) -> Result<FacetPeek<'mem, 'facet>, Error> {
        let mut current = normalize_facet_peek(data);

        for segment in &self.segments {
            current = if let Ok(struct_) = current.into_struct() {
                struct_.field_by_name(segment).ok()
            } else if let Ok(tuple) = current.into_tuple() {
                segment
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| tuple.field(index))
            } else if let Ok(list) = current.into_list_like() {
                segment
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| list.get(index))
            } else if let Ok(map) = current.into_map() {
                map.iter()
                    .find_map(
                        |(key, value)| match facet_key_matches_segment(key, segment) {
                            Ok(true) => Some(Ok(value)),
                            Ok(false) => None,
                            Err(err) => Some(Err(err)),
                        },
                    )
                    .transpose()?
            } else {
                None
            }
            .ok_or_else(|| MissingPathError {
                path: self.raw.clone(),
            })?;

            current = normalize_facet_peek(current);
        }

        Ok(current)
    }
}

fn take_until_brace<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_till(0.., ['{', '}']).parse_next(input)
}

fn take_until_dot<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_till(0.., '.').parse_next(input)
}

fn consume_escaped_open_brace<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    "{{".parse_next(input)
}

fn consume_escaped_close_brace<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    "}}".parse_next(input)
}

fn consume_open_brace<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    "{".parse_next(input)
}

fn consume_close_brace<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    "}".parse_next(input)
}

fn consume_dot<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    ".".parse_next(input)
}

fn push_literal(segments: &mut Vec<Segment>, literal: &str) {
    if literal.is_empty() {
        return;
    }

    match segments.last_mut() {
        Some(Segment::Literal(existing)) => existing.push_str(literal),
        _ => segments.push(Segment::Literal(literal.to_owned())),
    }
}

fn stringify_value(value: &Value) -> Result<String, serde_json::Error> {
    match value {
        Value::String(string) => Ok(string.clone()),
        _ => serde_json::to_string(value),
    }
}

#[cfg(feature = "facet")]
fn normalize_facet_peek<'mem, 'facet>(
    mut peek: FacetPeek<'mem, 'facet>,
) -> FacetPeek<'mem, 'facet> {
    loop {
        peek = peek.innermost_peek();

        if let Ok(option) = peek.into_option() {
            if let Some(inner) = option.value() {
                peek = inner;
                continue;
            }
        }

        return peek;
    }
}

#[cfg(feature = "facet")]
fn facet_key_matches_segment<'mem, 'facet>(
    key: FacetPeek<'mem, 'facet>,
    segment: &str,
) -> Result<bool, Error> {
    Ok(facet_key_to_string(key)? == segment)
}

#[cfg(feature = "facet")]
fn facet_key_to_string<'mem, 'facet>(key: FacetPeek<'mem, 'facet>) -> Result<String, Error> {
    let key = normalize_facet_peek(key);

    if let Some(string) = key.as_str() {
        return Ok(string.to_owned());
    }

    let value = facet_to_json_value(key)?;
    match value {
        Value::String(string) => Ok(string),
        Value::Number(number) => Ok(number.to_string()),
        Value::Bool(boolean) => Ok(boolean.to_string()),
        Value::Null => Ok("null".to_owned()),
        Value::Array(_) | Value::Object(_) => Err(FacetError::InvalidMapKey.into()),
    }
}

#[cfg(feature = "facet")]
fn stringify_facet_value<'mem, 'facet>(value: FacetPeek<'mem, 'facet>) -> Result<String, Error> {
    if let Some(string) = normalize_facet_peek(value).as_str() {
        return Ok(string.to_owned());
    }

    stringify_value(&facet_to_json_value(value)?).map_err(Error::Serialize)
}

#[cfg(feature = "facet")]
fn facet_to_json_value<'mem, 'facet>(value: FacetPeek<'mem, 'facet>) -> Result<Value, Error> {
    let value = normalize_facet_peek(value);

    if let Some(string) = value.as_str() {
        return Ok(Value::String(string.to_owned()));
    }

    if let Some(scalar) = value.scalar_type() {
        return scalar_to_json_value(value, scalar);
    }

    if let Ok(option) = value.into_option() {
        return match option.value() {
            Some(inner) => facet_to_json_value(inner),
            None => Ok(Value::Null),
        };
    }

    if let Ok(result) = value.into_result() {
        let mut object = serde_json::Map::new();
        if let Some(ok) = result.ok() {
            object.insert("Ok".to_owned(), facet_to_json_value(ok)?);
        } else if let Some(err) = result.err() {
            object.insert("Err".to_owned(), facet_to_json_value(err)?);
        }
        return Ok(Value::Object(object));
    }

    if let Ok(struct_) = value.into_struct() {
        let mut object = serde_json::Map::new();

        for (index, field) in struct_.ty().fields.iter().enumerate() {
            object.insert(
                field.name.to_owned(),
                facet_to_json_value(struct_.field(index).expect("field index is in bounds"))?,
            );
        }

        return Ok(Value::Object(object));
    }

    if let Ok(tuple) = value.into_tuple() {
        let mut items = Vec::with_capacity(tuple.len());

        for index in 0..tuple.len() {
            items.push(facet_to_json_value(
                tuple.field(index).expect("tuple index is in bounds"),
            )?);
        }

        return Ok(Value::Array(items));
    }

    if let Ok(list) = value.into_list_like() {
        let mut items = Vec::with_capacity(list.len());

        for index in 0..list.len() {
            items.push(facet_to_json_value(
                list.get(index).expect("list index is in bounds"),
            )?);
        }

        return Ok(Value::Array(items));
    }

    if let Ok(map) = value.into_map() {
        let mut object = serde_json::Map::new();

        for (key, value) in map.iter() {
            object.insert(facet_key_to_string(key)?, facet_to_json_value(value)?);
        }

        return Ok(Value::Object(object));
    }

    Err(FacetError::UnsupportedValue(value.shape().to_string()).into())
}

#[cfg(feature = "facet")]
fn scalar_to_json_value<'mem, 'facet>(
    value: FacetPeek<'mem, 'facet>,
    scalar: ScalarType,
) -> Result<Value, Error> {
    macro_rules! serialize_scalar {
        ($ty:ty) => {
            serde_json::to_value(value.get::<$ty>().map_err(FacetError::from)?)
                .map_err(Error::Serialize)
        };
    }

    match scalar {
        ScalarType::Unit => serialize_scalar!(()),
        ScalarType::Bool => serialize_scalar!(bool),
        ScalarType::Char => serialize_scalar!(char),
        ScalarType::Str => Ok(Value::String(
            value
                .as_str()
                .expect("string scalar types should be extractable")
                .to_owned(),
        )),
        ScalarType::String => Ok(Value::String(
            value
                .as_str()
                .expect("owned string scalar types should be extractable")
                .to_owned(),
        )),
        ScalarType::CowStr => Ok(Value::String(
            value
                .as_str()
                .expect("cow string scalar types should be extractable")
                .to_owned(),
        )),
        ScalarType::F32 => serialize_scalar!(f32),
        ScalarType::F64 => serialize_scalar!(f64),
        ScalarType::U8 => serialize_scalar!(u8),
        ScalarType::U16 => serialize_scalar!(u16),
        ScalarType::U32 => serialize_scalar!(u32),
        ScalarType::U64 => serialize_scalar!(u64),
        ScalarType::U128 => serialize_scalar!(u128),
        ScalarType::USize => serialize_scalar!(usize),
        ScalarType::I8 => serialize_scalar!(i8),
        ScalarType::I16 => serialize_scalar!(i16),
        ScalarType::I32 => serialize_scalar!(i32),
        ScalarType::I64 => serialize_scalar!(i64),
        ScalarType::I128 => serialize_scalar!(i128),
        ScalarType::ISize => serialize_scalar!(isize),
        _ => Err(FacetError::UnsupportedValue(value.shape().to_string()).into()),
    }
}

/// Errors that can happen while parsing or rendering templates.
#[derive(Debug, Display, Error, From)]
pub enum Error {
    /// Template parsing failed.
    Parse(ParseError),
    /// Serializing the input data failed.
    Serialize(serde_json::Error),
    /// A placeholder pointed at a missing path.
    MissingPath(MissingPathError),
    /// Rendering through the optional `facet` integration failed.
    #[cfg(feature = "facet")]
    Facet(FacetError),
}

/// Errors that can happen while parsing a template string.
#[derive(Debug, Display, Error, PartialEq, Eq)]
pub enum ParseError {
    /// The template contained an empty placeholder.
    #[display("empty placeholder at byte {position}")]
    EmptyPlaceholder {
        /// The byte offset of the opening `{`.
        position: usize,
    },
    /// The template contained a path with an empty segment.
    #[display("invalid path `{path}` at byte {position}")]
    InvalidPath {
        /// The byte offset of the opening `{`.
        position: usize,
        /// The placeholder path exactly as it appeared after trimming outer whitespace.
        path: String,
    },
    /// The template contained a `{` that was never closed.
    #[display("unmatched opening brace at byte {position}")]
    UnmatchedOpeningBrace {
        /// The byte offset of the unmatched `{`.
        position: usize,
    },
    /// The template contained a `}` that was not escaped or matched.
    #[display("unmatched closing brace at byte {position}")]
    UnmatchedClosingBrace {
        /// The byte offset of the unmatched `}`.
        position: usize,
    },
    /// The template attempted to open a nested placeholder.
    #[display("nested opening brace at byte {position}")]
    NestedOpeningBrace {
        /// The byte offset of the outer placeholder's opening `{`.
        position: usize,
    },
}

/// A placeholder referred to a path that was not present in the input data.
#[derive(Debug, Display, Error, PartialEq, Eq)]
#[display("missing path `{path}`")]
pub struct MissingPathError {
    path: String,
}

/// Errors that can happen while rendering through the optional `facet` integration.
#[cfg(feature = "facet")]
#[derive(Debug, Display, From)]
pub enum FacetError {
    /// `facet` reflection returned an error.
    Reflect(facet::ReflectError),
    /// A reflected value could not be represented with this crate's JSON-like rendering rules.
    #[display("unsupported reflected value `{_0}` for JSON-style rendering")]
    UnsupportedValue(String),
    /// A reflected map key could not be represented as a JSON object key.
    #[display("map key could not be represented as a JSON object key")]
    InvalidMapKey,
}

#[cfg(feature = "facet")]
impl std::error::Error for FacetError {}

impl MissingPathError {
    /// Returns the placeholder path that could not be resolved.
    ///
    /// # Example
    ///
    /// ```
    /// let err = jsonpath_fmt::render_value(
    ///     "{user.name}",
    ///     &serde_json::json!({ "user": {} }),
    /// )
    /// .unwrap_err();
    ///
    /// let missing = match err {
    ///     jsonpath_fmt::Error::MissingPath(missing) => missing,
    ///     other => panic!("unexpected error: {other}"),
    /// };
    ///
    /// assert_eq!(missing.path(), "user.name");
    /// ```
    pub fn path(&self) -> &str {
        &self.path
    }
}
