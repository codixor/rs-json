//! Bit-packed nesting stack: 1 bit per level (0 = array, 1 = object).
//! The first 128 levels live inline; deeper input spills to a `Vec<u64>`.
//! Replaces the old recursive descent that overflowed the native stack on
//! ~100k-deep input.

use alloc::vec::Vec;

/// Container kind at one nesting level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    /// Inside `[`.
    Array,
    /// Inside `{`.
    Object,
}

/// Nesting stack with inline storage.
#[derive(Debug, Default, Clone)]
pub struct NestStack {
    inline: [u64; 2],
    spill: Vec<u64>,
    depth: u32,
}

impl NestStack {
    /// Empty stack.
    pub const fn new() -> Self {
        Self { inline: [0; 2], spill: Vec::new(), depth: 0 }
    }

    /// Current depth.
    pub const fn depth(&self) -> u32 {
        self.depth
    }

    /// Clear for reuse without freeing the spill allocation.
    pub fn clear(&mut self) {
        self.inline = [0; 2];
        self.spill.clear();
        self.depth = 0;
    }

    fn word(&mut self, idx: usize) -> &mut u64 {
        if idx < 2 {
            &mut self.inline[idx]
        } else {
            let s = idx - 2;
            if s >= self.spill.len() {
                self.spill.resize(s + 1, 0);
            }
            &mut self.spill[s]
        }
    }

    /// Push a frame; `Err(())` when `max_depth` would be exceeded.
    #[allow(clippy::result_unit_err)]
    pub fn push(&mut self, f: Frame, max_depth: u32) -> Result<(), ()> {
        if self.depth >= max_depth {
            return Err(());
        }
        let d = self.depth as usize;
        let w = self.word(d / 64);
        let bit = 1u64 << (d % 64);
        if f == Frame::Object { *w |= bit } else { *w &= !bit }
        self.depth += 1;
        Ok(())
    }

    /// Pop the top frame.
    pub fn pop(&mut self) -> Option<Frame> {
        let top = self.top()?;
        self.depth -= 1;
        Some(top)
    }

    /// Peek the top frame.
    pub fn top(&self) -> Option<Frame> {
        let d = self.depth.checked_sub(1)? as usize;
        let w = if d / 64 < 2 { self.inline[d / 64] } else { self.spill[d / 64 - 2] };
        Some(if w >> (d % 64) & 1 == 1 { Frame::Object } else { Frame::Array })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_push_pop() {
        let mut s = NestStack::new();
        for i in 0..10_000u32 {
            let f = if i % 3 == 0 { Frame::Object } else { Frame::Array };
            s.push(f, u32::MAX).unwrap();
        }
        for i in (0..10_000u32).rev() {
            let f = if i % 3 == 0 { Frame::Object } else { Frame::Array };
            assert_eq!(s.pop(), Some(f));
        }
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn depth_limit() {
        let mut s = NestStack::new();
        assert!(s.push(Frame::Array, 1).is_ok());
        assert!(s.push(Frame::Array, 1).is_err());
    }
}
