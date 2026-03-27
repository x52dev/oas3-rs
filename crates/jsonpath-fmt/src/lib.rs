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

/// Errors that can happen while parsing or rendering templates.
#[derive(Debug, Display, Error, From)]
pub enum Error {
    /// Template parsing failed.
    Parse(ParseError),
    /// Serializing the input data failed.
    Serialize(serde_json::Error),
    /// A placeholder pointed at a missing path.
    MissingPath(MissingPathError),
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
