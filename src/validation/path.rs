use std::fmt;

#[derive(Debug, Clone)]
pub struct Path {
    parts: Vec<String>,
    separator: char,
}

impl Path {
    pub fn new(sep: char) -> Self {
        Self {
            parts: vec![],
            separator: sep,
        }
    }

    pub fn is_root(&self) -> bool {
        self.parts.is_empty()
    }

    /// Add path part
    pub fn push(&mut self, part: impl Into<String>) {
        self.parts.push(part.into());
    }

    /// Remove last path part, returning it. If The path is empty, return None.
    pub fn pop(&mut self) -> Option<String> {
        self.parts.pop()
    }

    /// Shorthand for extending path for passing down in recursive functions.
    pub fn extend(&self, part: impl Into<String>) -> Self {
        let mut new = self.clone();
        new.parts.push(part.into());
        new
    }
}

impl Default for Path {
    fn default() -> Self {
        Self {
            parts: vec![],
            separator: '/',
        }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_root() {
            f.write_str("{ROOT}")
        } else {
            let path = self.parts.join(&self.separator.to_string());
            f.write_str(&path)
        }
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.parts == other.parts
    }
}
