use crate::error::{JsonError, JsonErrorKind};
use crate::item::JsonItem;
use crate::key::Key;
use crate::kind::JsonType;
use crate::kind::JsonType::{JsonFalse, JsonNull, JsonNumber, JsonString, JsonTrue};
use crate::slice::Slice;

const DEFAULT_VEC_CAPACITY: usize = 4;

pub(crate) fn parse(source: &str) -> Result<JsonItem<'_>, JsonError> {
    Parser::new(source).parse_root()
}

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
        }
    }

    fn parse_root(&self) -> Result<JsonItem<'a>, JsonError> {
        let index = self.skip_spaces(0);
        if index >= self.bytes.len() {
            return Err(self.error(JsonErrorKind::UnexpectedEof, index));
        }

        let (item, next) = self.parse_item(index)?;
        let trailing = self.skip_spaces(next);
        if trailing != self.bytes.len() {
            return Err(self.error(JsonErrorKind::TrailingCharacters, trailing));
        }

        Ok(item)
    }

    fn parse_item(&self, index: usize) -> Result<(JsonItem<'a>, usize), JsonError> {
        if index >= self.bytes.len() {
            return Err(self.error(JsonErrorKind::UnexpectedEof, index));
        }

        match self.bytes[index] {
            b'n' => self.parse_keyword(index, b"null", JsonNull),
            b't' => self.parse_keyword(index, b"true", JsonTrue),
            b'f' => self.parse_keyword(index, b"false", JsonFalse),
            b'-' | b'0'..=b'9' => self.parse_number(index),
            b'"' => self.parse_string_value(index),
            b'{' => self.parse_map(index),
            b'[' => self.parse_array(index),
            _ => Err(self.error(JsonErrorKind::InvalidToken, index)),
        }
    }

    fn parse_keyword(
        &self,
        index: usize,
        keyword: &[u8],
        kind: JsonType,
    ) -> Result<(JsonItem<'a>, usize), JsonError> {
        let end = index + keyword.len();
        if self.bytes.get(index..end) != Some(keyword) {
            return Err(self.error(JsonErrorKind::InvalidToken, index));
        }

        Ok((JsonItem::new(self.slice(index, end), kind), end))
    }

    fn parse_number(&self, index: usize) -> Result<(JsonItem<'a>, usize), JsonError> {
        let mut cursor = index;

        if self.bytes[cursor] == b'-' {
            cursor += 1;
            if cursor >= self.bytes.len() {
                return Err(self.error(JsonErrorKind::UnexpectedEof, cursor));
            }
        }

        match self.bytes[cursor] {
            b'0' => {
                cursor += 1;
                if matches!(self.bytes.get(cursor), Some(b'0'..=b'9')) {
                    return Err(self.error(JsonErrorKind::InvalidNumber, cursor));
                }
            }
            b'1'..=b'9' => {
                cursor += 1;
                while matches!(self.bytes.get(cursor), Some(b'0'..=b'9')) {
                    cursor += 1;
                }
            }
            _ => return Err(self.error(JsonErrorKind::InvalidNumber, cursor)),
        }

        if matches!(self.bytes.get(cursor), Some(b'.')) {
            cursor += 1;
            if !matches!(self.bytes.get(cursor), Some(b'0'..=b'9')) {
                return Err(self.error(JsonErrorKind::InvalidNumber, cursor));
            }
            while matches!(self.bytes.get(cursor), Some(b'0'..=b'9')) {
                cursor += 1;
            }
        }

        if matches!(self.bytes.get(cursor), Some(b'e' | b'E')) {
            cursor += 1;
            if matches!(self.bytes.get(cursor), Some(b'+' | b'-')) {
                cursor += 1;
            }
            if !matches!(self.bytes.get(cursor), Some(b'0'..=b'9')) {
                return Err(self.error(JsonErrorKind::InvalidNumber, cursor));
            }
            while matches!(self.bytes.get(cursor), Some(b'0'..=b'9')) {
                cursor += 1;
            }
        }

        Ok((JsonItem::new(self.slice(index, cursor), JsonNumber), cursor))
    }

    fn parse_string_value(&self, index: usize) -> Result<(JsonItem<'a>, usize), JsonError> {
        let (slice, next) = self.parse_string_slice(index)?;
        Ok((JsonItem::new(slice, JsonString), next))
    }

    fn parse_string_slice(&self, index: usize) -> Result<(Slice<'a>, usize), JsonError> {
        debug_assert_eq!(self.bytes[index], b'"');

        let mut cursor = index + 1;
        while cursor < self.bytes.len() {
            match self.bytes[cursor] {
                b'"' => return Ok((self.slice(index + 1, cursor), cursor + 1)),
                b'\\' => {
                    cursor += 1;
                    if cursor >= self.bytes.len() {
                        return Err(self.error(JsonErrorKind::UnexpectedEof, cursor));
                    }
                    match self.bytes[cursor] {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                            cursor += 1;
                        }
                        b'u' => {
                            if cursor + 4 >= self.bytes.len() {
                                return Err(self.error(JsonErrorKind::UnexpectedEof, cursor));
                            }
                            if self.bytes[cursor + 1..cursor + 5]
                                .iter()
                                .any(|byte| !byte.is_ascii_hexdigit())
                            {
                                return Err(self.error(JsonErrorKind::InvalidString, cursor));
                            }
                            cursor += 5;
                        }
                        _ => return Err(self.error(JsonErrorKind::InvalidString, cursor)),
                    }
                }
                0x00..=0x1f => return Err(self.error(JsonErrorKind::InvalidString, cursor)),
                _ => cursor += 1,
            }
        }

        Err(self.error(JsonErrorKind::UnexpectedEof, cursor))
    }

    fn parse_map(&self, index: usize) -> Result<(JsonItem<'a>, usize), JsonError> {
        let mut cursor = self.skip_spaces(index + 1);
        let mut entries = Vec::with_capacity(DEFAULT_VEC_CAPACITY);

        if matches!(self.bytes.get(cursor), Some(b'}')) {
            return Ok((
                JsonItem::new_map(self.slice(index, cursor + 1), entries),
                cursor + 1,
            ));
        }

        loop {
            if cursor >= self.bytes.len() {
                return Err(self.error(JsonErrorKind::UnexpectedEof, cursor));
            }
            if self.bytes[cursor] != b'"' {
                return Err(self.error(JsonErrorKind::InvalidObject, cursor));
            }

            let (key_slice, next) = self.parse_string_slice(cursor)?;
            cursor = self.skip_spaces(next);
            if !matches!(self.bytes.get(cursor), Some(b':')) {
                return Err(self.error(JsonErrorKind::InvalidObject, cursor));
            }

            cursor = self.skip_spaces(cursor + 1);
            let (item, next) = self.parse_item(cursor)?;
            entries.push((Key::from_slice(key_slice), item));
            cursor = self.skip_spaces(next);

            match self.bytes.get(cursor) {
                Some(b',') => cursor = self.skip_spaces(cursor + 1),
                Some(b'}') => {
                    return Ok((
                        JsonItem::new_map(self.slice(index, cursor + 1), entries),
                        cursor + 1,
                    ));
                }
                Some(_) => return Err(self.error(JsonErrorKind::InvalidObject, cursor)),
                None => return Err(self.error(JsonErrorKind::UnexpectedEof, cursor)),
            }
        }
    }

    fn parse_array(&self, index: usize) -> Result<(JsonItem<'a>, usize), JsonError> {
        let mut cursor = self.skip_spaces(index + 1);
        let mut items = Vec::with_capacity(DEFAULT_VEC_CAPACITY);

        if matches!(self.bytes.get(cursor), Some(b']')) {
            return Ok((
                JsonItem::new_array(self.slice(index, cursor + 1), items),
                cursor + 1,
            ));
        }

        loop {
            let (item, next) = self.parse_item(cursor)?;
            items.push(item);
            cursor = self.skip_spaces(next);

            match self.bytes.get(cursor) {
                Some(b',') => cursor = self.skip_spaces(cursor + 1),
                Some(b']') => {
                    return Ok((
                        JsonItem::new_array(self.slice(index, cursor + 1), items),
                        cursor + 1,
                    ));
                }
                Some(_) => return Err(self.error(JsonErrorKind::InvalidArray, cursor)),
                None => return Err(self.error(JsonErrorKind::UnexpectedEof, cursor)),
            }
        }
    }

    fn skip_spaces(&self, mut index: usize) -> usize {
        while index < self.bytes.len() {
            match self.bytes[index] {
                b' ' | b'\n' | b'\r' | b'\t' => index += 1,
                _ => break,
            }
        }
        index
    }

    fn error(&self, kind: JsonErrorKind, index: usize) -> JsonError {
        JsonError::new(self.source, index, kind)
    }

    fn slice(&self, start: usize, end: usize) -> Slice<'a> {
        Slice::from_range(self.source, start, end)
    }
}
