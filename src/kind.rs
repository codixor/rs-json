/// JSON value kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonType {
    JsonNull,
    JsonTrue,
    JsonFalse,
    JsonString,
    JsonNumber,
    JsonMap,
    JsonArray,
    Empty,
}
