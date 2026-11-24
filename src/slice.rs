use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Slice<'a> {
    source: &'a str,
}

impl<'a> Slice<'a> {
    pub(crate) const fn from_str(source: &'a str) -> Self {
        Self { source }
    }

    pub(crate) fn from_range(source: &'a str, start: usize, end: usize) -> Self {
        Self {
            source: &source[start..end],
        }
    }

    pub(crate) const fn empty() -> Self {
        Self { source: "" }
    }

    pub(crate) const fn as_str(self) -> &'a str {
        self.source
    }
}

impl Debug for Slice<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.source, f)
    }
}
