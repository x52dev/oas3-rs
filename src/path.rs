use std::fmt;

#[derive(Debug, Clone)]
pub struct Path {
    parts: Vec<String>,
    seperator: char,
}

impl Path {
    pub fn new() -> Self { Self::default() }
    pub fn push<T: Into<String>>(&mut self, part: T) { self.parts.push(part.into()); }
    pub fn pop(&mut self) -> Option<String> { self.parts.pop() }
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let path = self.parts.join(&self.seperator.to_string());
        write!(f, "{}", path)
    }
}
