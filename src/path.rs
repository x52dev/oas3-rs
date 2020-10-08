use std::fmt;

#[derive(Debug, Clone)]
pub struct Path {
    parts: Vec<String>,
    seperator: char,
}

impl Path {
    pub fn new(sep: char) -> Self {
        Self {
            parts: vec![],
            seperator: sep,
        }
    }

    /// Add path part
    pub fn push<T: Into<String>>(&mut self, part: T) {
        self.parts.push(part.into());
    }

    /// Remove last path part, returning it. If The path is empty, return None.
    pub fn pop(&mut self) -> Option<String> {
        self.parts.pop()
    }

    /// Shorthand for extending path for passing down in recursive functions.
    pub fn extend<T: Into<String>>(&self, part: T) -> Self {
        let mut new = self.clone();
        new.parts.push(part.into());
        new
    }
}

impl Default for Path {
    fn default() -> Self {
        Self {
            parts: vec![],
            seperator: '/',
        }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.parts.join(&self.seperator.to_string());
        write!(f, "{}", path)
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.parts == other.parts
    }
}
