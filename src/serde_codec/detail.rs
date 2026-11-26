//! Safe diagnostic detail for typed codec errors.
//!
//! serde's default messages quote the input: rejected values
//! (`invalid type: string "hunter2", expected u64`), unknown object keys
//! (``unknown field `sk_live_x` ``) and unknown variants. A message from a
//! type's own impl (`Error::custom`) can hold anything, including the value
//! being decoded or encoded. Detail is therefore built from an allow-list:
//! it keeps only text that the target type declares statically or that the
//! codec writes as fixed text.
//!
//! | serde message | detail |
//! |---------------|--------|
//! | ``missing field `k` `` / ``duplicate field `k` `` | kept: `k` is a `&'static str` the type declares |
//! | ``unknown field `k`, expected …`` | `unknown field, expected …`: the key came from the input |
//! | `invalid type: string "…", expected …` | `invalid type: a string, expected …` |
//! | ``invalid value: integer `5`, expected …`` | `invalid value: an integer, expected …` |
//! | ``unknown variant `v`, expected …`` | `unknown variant, expected …` |
//! | `invalid length 3, expected …` | kept: a count, not input |
//! | `data did not match any variant of untagged enum E` | kept: `E` is the type name |
//! | `serde_json` grammar and number-range errors | kept: fixed text |
//! | I/O errors | none: the reader's own text may hold anything |
//! | any other message, including every `Error::custom` | none |
//!
//! The `expected …` part comes from the target type's `expecting`
//! implementation. Every detail drops the trailing `at line L column C`,
//! replaces control characters with `?` and is cut at [`MAX_DETAIL_CHARS`]
//! characters.

use core::fmt;

/// The longest detail kept, in characters.
pub const MAX_DETAIL_CHARS: usize = 256;

/// Sanitized diagnostic text. `Debug` never prints the text, so an error's
/// derived `Debug` stays as free of input as its `Display`.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Detail(Option<Box<str>>);

impl Detail {
    /// No detail.
    pub const NONE: Self = Self(None);

    /// The sanitized text, if any.
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_deref()
    }

    pub(super) fn decode(category: serde_json::error::Category, message: &str) -> Self {
        use serde_json::error::Category as C;
        match category {
            C::Io => Self::NONE,
            C::Syntax | C::Eof => Self::kept(strip_position(message)),
            C::Data => Self::data(strip_position(message)),
        }
    }

    pub(crate) fn decode_number() -> Self {
        Self::kept("number out of range")
    }

    pub(super) fn encode(is_io: bool, message: &str) -> Self {
        // `serde_json`'s own encode failures. Everything else is a custom
        // message from a `Serialize` impl, which may quote the value.
        const FIXED: [&str; 2] = [
            "key must be a string",
            "float key must be finite (got NaN or +/-inf)",
        ];
        let message = strip_position(message);
        if !is_io && FIXED.contains(&message) {
            Self::kept(message)
        } else {
            Self::NONE
        }
    }

    fn data(message: &str) -> Self {
        // Each prefix is followed only by text the target type declares.
        const KEEP: [&str; 6] = [
            "missing field `",
            "duplicate field `",
            "invalid length ",
            "number out of range",
            "data did not match any variant of untagged enum ",
            "no variant of enum ",
        ];
        if KEEP.iter().any(|prefix| message.starts_with(prefix)) {
            return Self::kept(message);
        }
        for prefix in ["invalid type: ", "invalid value: "] {
            if let Some(rest) = message.strip_prefix(prefix) {
                // The input sits before the expectation, so the text after the
                // last separator always comes from the target type.
                return match rest.rsplit_once(", expected ") {
                    Some((unexpected, expected)) => Self::kept(&format!(
                        "{prefix}{}, expected {expected}",
                        unexpected_kind(unexpected)
                    )),
                    None => Self::kept(&format!("{prefix}{}", unexpected_kind(rest))),
                };
            }
        }
        // The field or variant name came from the input. Keep only the
        // expectation, which the type declares.
        for (prefix, name, none) in [
            ("unknown field `", "unknown field", ", there are no fields"),
            (
                "unknown variant `",
                "unknown variant",
                ", there are no variants",
            ),
        ] {
            if message.starts_with(prefix) {
                return Self::kept(&match message.rsplit_once(", expected ") {
                    Some((_, expected)) => format!("{name}, expected {expected}"),
                    None if message.ends_with(none) => format!("{name}{none}"),
                    None => name.to_owned(),
                });
            }
        }
        // `Error::custom` from any impl: the text is not ours to vouch for.
        Self::NONE
    }

    fn kept(text: &str) -> Self {
        let clean: String = text
            .chars()
            .take(MAX_DETAIL_CHARS)
            .map(|c| if c.is_control() { '?' } else { c })
            .collect();
        Self((!clean.is_empty()).then(|| clean.into_boxed_str()))
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

/// serde's `Unexpected` display reduced to its kind. A form this table does
/// not know becomes "a value", so new serde wording cannot leak input.
fn unexpected_kind(unexpected: &str) -> &'static str {
    const WITH_VALUE: [(&str, &str); 6] = [
        ("string ", "a string"),
        ("integer ", "an integer"),
        ("floating point ", "a floating point number"),
        ("boolean ", "a boolean"),
        ("character ", "a character"),
        ("other ", "a value"),
    ];
    const FIXED: [(&str, &str); 12] = [
        ("byte array", "a byte array"),
        ("unit value", "a unit value"),
        ("null", "null"),
        ("Option value", "an Option value"),
        ("newtype struct", "a newtype struct"),
        ("sequence", "a sequence"),
        ("map", "a map"),
        ("enum", "an enum"),
        ("unit variant", "a unit variant"),
        ("newtype variant", "a newtype variant"),
        ("tuple variant", "a tuple variant"),
        ("struct variant", "a struct variant"),
    ];
    if let Some(&(_, kind)) = FIXED.iter().find(|(text, _)| unexpected == *text) {
        return kind;
    }
    WITH_VALUE
        .iter()
        .find(|(prefix, _)| unexpected.starts_with(prefix))
        .map_or("a value", |&(_, kind)| kind)
}

/// Drop `serde_json`'s trailing ` at line L column C`.
fn strip_position(message: &str) -> &str {
    let Some((head, tail)) = message.rsplit_once(" at line ") else {
        return message;
    };
    let mut parts = tail.splitn(3, ' ');
    let line = parts.next().unwrap_or_default();
    let word = parts.next().unwrap_or_default();
    let column = parts.next().unwrap_or_default();
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if digits(line) && word == "column" && digits(column) {
        head
    } else {
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::error::Category as C;

    fn d(category: C, message: &str) -> Option<String> {
        Detail::decode(category, message)
            .as_str()
            .map(str::to_owned)
    }

    #[test]
    fn values_are_reduced_to_their_kind() {
        assert_eq!(
            d(
                C::Data,
                "invalid type: string \"hunter2\", expected u64 at line 1 column 9"
            )
            .as_deref(),
            Some("invalid type: a string, expected u64")
        );
        assert_eq!(
            d(C::Data, "invalid value: integer `482913`, expected a port").as_deref(),
            Some("invalid value: an integer, expected a port")
        );
        // A value that itself contains the separator never survives.
        assert_eq!(
            d(
                C::Data,
                "invalid type: string \"x, expected hunter2\", expected u64"
            )
            .as_deref(),
            Some("invalid type: a string, expected u64")
        );
        assert_eq!(
            d(C::Data, "unknown variant `sk_live_x`, expected `a` or `b`").as_deref(),
            Some("unknown variant, expected `a` or `b`")
        );
        assert_eq!(
            d(C::Data, "invalid type: map, expected a string").as_deref(),
            Some("invalid type: a map, expected a string")
        );
    }

    #[test]
    fn input_keys_and_custom_messages_are_dropped() {
        assert_eq!(
            d(
                C::Data,
                "unknown field `sk_live_x`, expected `token` at line 1 column 7"
            )
            .as_deref(),
            Some("unknown field, expected `token`")
        );
        // A key that itself contains the separator never survives.
        assert_eq!(
            d(
                C::Data,
                "unknown field `a, there are no fields`, expected one of `a`, `b`"
            )
            .as_deref(),
            Some("unknown field, expected one of `a`, `b`")
        );
        assert_eq!(
            d(C::Data, "unknown field `sk_live_x`, there are no fields").as_deref(),
            Some("unknown field, there are no fields")
        );
        assert_eq!(
            d(
                C::Data,
                "unknown variant `sk_live_x`, there are no variants"
            )
            .as_deref(),
            Some("unknown variant, there are no variants")
        );
        assert_eq!(d(C::Data, "bad token sk_live_x"), None);
        assert_eq!(
            Detail::encode(false, "cannot encode sk_live_x").as_str(),
            None
        );
        assert_eq!(
            Detail::encode(false, "key must be a string").as_str(),
            Some("key must be a string")
        );
        assert_eq!(Detail::encode(true, "key must be a string").as_str(), None);
    }

    #[test]
    fn declared_names_and_grammar_are_kept() {
        assert_eq!(
            d(C::Data, "missing field `path`").as_deref(),
            Some("missing field `path`")
        );
        assert_eq!(
            d(
                C::Data,
                "data did not match any variant of untagged enum Rule"
            )
            .as_deref(),
            Some("data did not match any variant of untagged enum Rule")
        );
        assert_eq!(
            d(C::Syntax, "expected `,` or `}` at line 1 column 4").as_deref(),
            Some("expected `,` or `}`")
        );
        assert_eq!(d(C::Io, "disk said hunter2"), None);
    }

    #[test]
    fn text_is_cleaned_and_bounded() {
        assert_eq!(
            d(C::Data, "missing field `a\nb`").as_deref(),
            Some("missing field `a?b`")
        );
        let long = format!("missing field `{}`", "x".repeat(1000));
        assert_eq!(
            d(C::Data, &long).map(|s| s.chars().count()),
            Some(MAX_DETAIL_CHARS)
        );
        // A trailing phrase that is not a position stays.
        assert_eq!(
            d(C::Syntax, "expected value at line one").as_deref(),
            Some("expected value at line one")
        );
        assert_eq!(
            format!("{:?}", Detail::decode(C::Data, "missing field `secret`")),
            "Detail(..)"
        );
    }
}
