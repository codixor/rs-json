//! Optional diagnostic labels for typed codec errors. Serde exposes error
//! categories but reports value kinds and expectations through formatted text.
//! Treat every part of that text as untrusted, including a visitor's expectation.

use core::fmt;
use serde_json::error::Category;
use std::borrow::Cow;
use std::io::Write;

/// Upper bound on the character count of a diagnostic detail.
pub const MAX_DETAIL_CHARS: usize = 256;

/// A diagnostic assembled exclusively from the codec's fixed labels.
/// Its `Debug` representation indicates only whether a detail is present.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Detail(Option<Cow<'static, str>>);

const NUMBER_RANGE: &str = "number out of range";
const KEY_ERRORS: &[&str] = &[
    "key must be a string",
    "float key must be finite (got NaN or +/-inf)",
];
const SYNTAX_ERRORS: &[&str] = &[
    "expected `:`",
    "expected `,` or `]`",
    "expected `,` or `}`",
    "expected ident",
    "expected value",
    "expected `\"`",
    "invalid escape",
    "invalid number",
    "invalid unicode code point",
    "control character (\\u0000-\\u001F) found while parsing a string",
    "invalid value: expected key to be a number in quotes",
    "lone leading surrogate in hex escape",
    "trailing comma",
    "trailing characters",
    "unexpected end of hex escape",
    "recursion limit exceeded",
];
const EOF_ERRORS: &[&str] = &[
    "EOF while parsing a list",
    "EOF while parsing an object",
    "EOF while parsing a string",
    "EOF while parsing a value",
];
const EXPECTATIONS: &[&str] = &[
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "usize",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "isize",
    "f32",
    "f64",
    "a boolean",
    "a character",
    "a string",
    "a borrowed string",
    "a borrowed byte array",
    "byte array",
    "a sequence",
    "a map",
    "unit",
];

// Return a reference into the table, never the message that matched it.
fn lookup(message: &str, labels: &[&'static str]) -> Option<&'static str> {
    labels.iter().copied().find(|label| *label == message)
}

impl Detail {
    /// No detail.
    pub const NONE: Self = Self(None);

    /// The diagnostic labels, if any.
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_deref()
    }

    fn fixed(label: &'static str) -> Self {
        Self(Some(Cow::Borrowed(label)))
    }

    pub(crate) fn decode_number() -> Self {
        Self::fixed(NUMBER_RANGE)
    }

    pub(super) fn decode(error: &serde_json::Error) -> (Category, Self) {
        let category = error.classify();
        if category == Category::Io {
            return (category, Self::NONE);
        }
        let mut scratch = [0; MAX_DETAIL_CHARS];
        let Some(message) = message_without_position(error, &mut scratch) else {
            return (category, Self::NONE);
        };
        let label = match category {
            // serde_json groups an out-of-range number with grammar failures.
            Category::Syntax if message == NUMBER_RANGE => {
                return (Category::Data, Self::decode_number());
            }
            Category::Syntax => {
                lookup(message, SYNTAX_ERRORS).or_else(|| lookup(message, KEY_ERRORS))
            }
            Category::Eof => lookup(message, EOF_ERRORS),
            Category::Data => return (category, Self::data(message)),
            Category::Io => None,
        };
        (category, label.map_or(Self::NONE, Self::fixed))
    }

    pub(super) fn encode(error: &serde_json::Error) -> Self {
        // Built-in map key errors are Syntax. Skip I/O errors and custom
        // serialization errors (Data) without invoking their Display.
        if !error.is_syntax() {
            return Self::NONE;
        }
        let mut scratch = [0; MAX_DETAIL_CHARS];
        message_without_position(error, &mut scratch)
            .and_then(|message| lookup(message, KEY_ERRORS))
            .map_or(Self::NONE, Self::fixed)
    }

    fn data(message: &str) -> Self {
        for prefix in ["invalid type: ", "invalid value: "] {
            if let Some(rest) = message.strip_prefix(prefix) {
                let (unexpected, expectation) =
                    rest.rsplit_once(", expected ").unwrap_or((rest, ""));
                let kind = unexpected_kind(unexpected);
                let detail = match lookup(expectation, EXPECTATIONS) {
                    Some(label) => format!("{prefix}{kind}, expected {label}"),
                    None => format!("{prefix}{kind}"),
                };
                return Self(Some(Cow::Owned(detail)));
            }
        }
        if message == NUMBER_RANGE {
            return Self::decode_number();
        }
        for (prefix, label) in [
            ("missing field `", "missing field"),
            ("duplicate field `", "duplicate field"),
            ("unknown field `", "unknown field"),
            ("unknown variant `", "unknown variant"),
            ("invalid length ", "invalid length"),
            (
                "data did not match any variant of untagged enum ",
                "no matching enum variant",
            ),
            ("no variant of enum ", "no matching enum variant"),
        ] {
            if message.starts_with(prefix) {
                return Self::fixed(label);
            }
        }
        Self::NONE
    }
}

impl fmt::Debug for Detail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if self.0.is_some() {
            "Detail(..)"
        } else {
            "None"
        })
    }
}

// serde_json exposes no public message accessor. Inspect only diagnostics that
// fit in the scratch buffer. A partial message must never be classified.
fn message_without_position<'a>(
    error: &serde_json::Error,
    scratch: &'a mut [u8],
) -> Option<&'a str> {
    let capacity = scratch.len();
    let mut writer = &mut *scratch;
    write!(writer, "{error}").ok()?;
    let used = capacity - writer.len();
    let text = str::from_utf8(scratch.get(..used)?).ok()?;
    if error.line() == 0 {
        Some(text)
    } else {
        text.rsplit_once(" at line ").map(|(message, _)| message)
    }
}

fn unexpected_kind(unexpected: &str) -> &'static str {
    for (prefix, kind) in [
        ("string ", "a string"),
        ("integer ", "an integer"),
        ("floating point ", "a floating point number"),
        ("boolean ", "a boolean"),
        ("character ", "a character"),
    ] {
        if unexpected.starts_with(prefix) {
            return kind;
        }
    }
    match unexpected {
        "null" => "null",
        "byte array" => "a byte array",
        "unit value" => "a unit value",
        "Option value" => "an Option value",
        "newtype struct" => "a newtype struct",
        "sequence" => "a sequence",
        "map" => "a map",
        "enum" => "an enum",
        "unit variant" => "a unit variant",
        "newtype variant" => "a newtype variant",
        "tuple variant" => "a tuple variant",
        "struct variant" => "a struct variant",
        _ => "a value",
    }
}
