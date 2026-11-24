//! Compatibility shim (feature `legacy`) so existing rust-misc callers of
//! `rs_json::parse(&str) -> JsonItem` compile during migration.
//! Agent: implement `JsonItem<'d,'a>` as a thin wrapper over `dom::Value`
//! with the old method names (`as_str` raw, `as_f64`, `as_i128`, `as_bool`,
//! `is_null`, `exists`, `kind`/`get_type`, `elements`, `entries`, `Index<usize>`,
//! `Index<&str>`), marked `#[deprecated]`.
