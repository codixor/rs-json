//! Streaming RFC 6901 JSON Pointer selection (feature `pointer`).
//! Compile a set of pointers into a trie; while pulling events, track the
//! current path and call `skip_value` for every subtree no pointer can match.

/// Compiled pointer set.
#[derive(Debug, Default)]
pub struct PointerSet {
    _todo: (),
}

impl PointerSet {
    /// Parse pointers like `/data/0/price` (`~0`/`~1` escapes).
    ///
    /// # Errors
    /// Malformed pointer text.
    pub fn compile<'s>(pointers: impl IntoIterator<Item = &'s str>) -> Result<Self, &'static str> {
        let _ = pointers.into_iter();
        todo!("agent: pointer trie")
    }
}
