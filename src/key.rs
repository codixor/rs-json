use crate::slice::Slice;

/// A zero-copy JSON object key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key<'a> {
    slice: Slice<'a>,
}

impl<'a> Key<'a> {
    pub(crate) fn from_slice(slice: Slice<'a>) -> Self {
        Self { slice }
    }

    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Self {
            slice: Slice::from_str(source),
        }
    }

    /// Key text value.
    #[must_use]
    pub fn as_str(&self) -> &'a str {
        self.slice.as_str()
    }
}
