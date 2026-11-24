//! Flat objects with one or two string values fit in the inline tape on 64-bit
//! targets. After the first key, index up to 64 bytes with one bit per byte.
//! The ordinary parser handles anything outside this grammar or window.

use super::{Inline, NodeTag, PackedNode, string};
use wide::u8x16;

#[derive(Default)]
struct Masks {
    quotes: u64,
    slashes: u64,
    controls: u32,
}

impl Masks {
    #[inline]
    fn include(&mut self, block: &[u8; 16], offset: usize) {
        let bytes = u8x16::new(*block);
        self.quotes |= u64::from(bytes.simd_eq(u8x16::splat(b'"')).to_bitmask()) << offset;
        self.slashes |= u64::from(bytes.simd_eq(u8x16::splat(b'\\')).to_bitmask()) << offset;
        self.controls |= bytes.simd_lt(u8x16::splat(32)).to_bitmask();
    }

    fn scan(body: &[u8]) -> Option<Self> {
        let last = body.len().checked_sub(16)?;
        if last > 48 {
            return None;
        }
        let mut masks = Self::default();
        masks.include(body.first_chunk()?, 0);
        if last > 16 {
            masks.include(body.get(16..)?.first_chunk()?, 16);
        }
        if last > 32 {
            masks.include(body.get(32..)?.first_chunk()?, 32);
        }
        // The last load can overlap a previous one. OR keeps shared byte positions.
        masks.include(body.last_chunk()?, last);
        Some(masks)
    }

    fn structural_quotes(&self, body: &[u8]) -> Option<u64> {
        let slashes = self.slashes;
        // For runs of one or two backslashes, only the first escapes its
        // successor. Longer runs take the general parser's parity handling.
        if self.controls != 0 || slashes & (slashes << 1) & (slashes << 2) != 0 {
            return None;
        }
        let escaped = (slashes << 1) & !(slashes << 2);
        // Escaped quotes and backslashes are valid by construction.
        let mut other = escaped & !(self.quotes | slashes);
        while other != 0 {
            let i = other.trailing_zeros() as usize;
            other &= other - 1;
            match *body.get(i)? {
                b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
                b'u' => {
                    if crate::unescape::validated_unicode_len(body.get(i + 1..)?)? == 10 {
                        // Both UTF-16 code units were checked together.
                        other &= !(1 << (i + 6));
                    }
                }
                _ => return None,
            }
        }
        Some(self.quotes & !escaped)
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "the caller bounds the document by INLINE_BYTES, which fits in u8"
)]
fn field(start: usize, end: usize, escaped: bool) -> PackedNode {
    PackedNode {
        tag: NodeTag::Str,
        start: start as u8,
        len: (end - start) as u8,
        aux: u8::from(escaped),
    }
}

impl Inline {
    #[inline]
    pub(in crate::dom) fn string_object(src: &[u8]) -> Option<Self> {
        debug_assert!(src.len() <= super::INLINE_BYTES);
        // The smallest envelope adds six bytes to the 16-byte scan window.
        if src.len() < 22 || !src.starts_with(b"{\"") || !src.ends_with(b"\"}") {
            return None;
        }
        Self::string_fields(src)
    }

    fn string_fields(src: &[u8]) -> Option<Self> {
        let mut cursor = 1;
        let first_key = string(src, &mut cursor)?;
        let start = cursor + 2;
        if src.get(cursor..start)? != b":\"" {
            return None;
        }
        // Include the final closing quote so a dangling backslash is rejected.
        let body = src.get(start..src.len() - 1)?;
        let masks = Masks::scan(body)?;
        let quotes = masks.structural_quotes(body)?;
        let end = body.len() - 1;
        let last_quote = 1u64 << end;
        if quotes & last_quote == 0 {
            return None;
        }
        let interior = quotes ^ last_quote;
        let mut out = Self::new();
        *out.nodes.get_mut(1)? = field(
            first_key.a as usize,
            (first_key.a + first_key.b) as usize,
            first_key.flags != 0,
        );
        let count = if interior == 0 {
            *out.nodes.get_mut(2)? = field(start, start + end, masks.slashes != 0);
            1
        } else {
            // Two entries have exactly two interior delimiters: "," and ":".
            // Locate their outer quotes independently, then check the complete
            // mask, separator bytes and room for the second key (possibly empty).
            let first_end = interior.trailing_zeros() as usize;
            let last_open = 63 - interior.leading_zeros() as usize;
            let key_end = last_open.checked_sub(2)?;
            let quote_pair = 0b101;
            if key_end <= first_end + 2
                || interior != ((quote_pair << first_end) | (quote_pair << key_end))
                || body.get(first_end + 1) != Some(&b',')
                || body.get(key_end + 1) != Some(&b':')
            {
                return None;
            }
            let before_comma = (1 << first_end) - 1;
            let before_colon = (1 << key_end) - 1;
            *out.nodes.get_mut(2)? =
                field(start, start + first_end, masks.slashes & before_comma != 0);
            *out.nodes.get_mut(3)? = field(
                start + first_end + 3,
                start + key_end,
                masks.slashes & (before_colon ^ before_comma) != 0,
            );
            *out.nodes.get_mut(4)? = field(
                start + last_open + 1,
                start + end,
                masks.slashes >> last_open != 0,
            );
            2
        };
        out.used = 1 + 2 * count;
        *out.nodes.first_mut()? = PackedNode {
            tag: NodeTag::Obj,
            start: 0,
            len: count,
            aux: out.used,
        };
        Some(out)
    }
}
