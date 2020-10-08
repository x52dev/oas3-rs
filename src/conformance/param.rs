use std::cell::Cell;

#[derive(Debug, Clone)]
pub struct ParamReplacement {
    pub name: String,
    pub value: String,
    pub used: Cell<bool>,
}

impl ParamReplacement {
    pub fn new<N, V>(name: N, val: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        Self {
            name: name.into(),
            value: val.into(),
            used: Cell::new(false),
        }
    }
}

impl ParamReplacement {
    pub fn used(&self) -> bool {
        self.used.get()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamPosition {
    Path,
    Query,
    Cookie,
    Header,
}

#[derive(Debug, Clone)]
pub struct TestParam {
    pub name: String,
    pub value: String,
    pub position: ParamPosition,
}

impl TestParam {
    pub fn new<N, V>(name: N, val: V, pos: ParamPosition) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        Self {
            name: name.into(),
            value: val.into(),
            position: pos,
        }
    }

    pub fn path<N, V>(name: N, val: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        Self::new(name, val, ParamPosition::Path)
    }

    pub fn query<N, V>(name: N, val: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        Self::new(name, val, ParamPosition::Query)
    }

    pub fn cookie<N, V>(name: N, val: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        Self::new(name, val, ParamPosition::Cookie)
    }

    pub fn header<N, V>(name: N, val: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        Self::new(name, val, ParamPosition::Header)
    }
}
