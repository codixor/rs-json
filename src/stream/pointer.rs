//! Select complete values by RFC 6901 JSON Pointer while streaming events.
use super::{Event, PullParser};
use crate::{Error, lex::Lexer};
use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};

#[derive(Debug, Default)]
struct Node {
    keys: BTreeMap<String, usize>,
    indices: BTreeMap<usize, usize>,
    matches: Vec<usize>,
}

/// A flat trie of pointers. The empty string selects the root.
#[derive(Debug)]
pub struct PointerSet {
    nodes: Vec<Node>,
}

impl Default for PointerSet {
    fn default() -> Self {
        Self {
            nodes: vec![Node::default()],
        }
    }
}

fn token(text: &str) -> Result<String, &'static str> {
    let mut decoded = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        decoded.push(if c == '~' {
            match chars.next() {
                Some('0') => '~',
                Some('1') => '/',
                _ => return Err("invalid JSON Pointer escape"),
            }
        } else {
            c
        });
    }
    Ok(decoded)
}

struct Frame {
    node: Option<usize>,
    array_index: Option<usize>,
    member: Option<usize>,
    outer_matches: usize,
}

impl PointerSet {
    /// Compile pointers in order. A match reports its zero-based input index.
    ///
    /// # Errors
    /// A nonempty pointer must start with a slash. Only `~0` and `~1` are valid escapes.
    pub fn compile<'s>(pointers: impl IntoIterator<Item = &'s str>) -> Result<Self, &'static str> {
        let mut set = Self::default();
        for (id, pointer) in pointers.into_iter().enumerate() {
            let mut at = 0;
            if !pointer.is_empty() {
                for part in pointer
                    .strip_prefix('/')
                    .ok_or("expected leading slash")?
                    .split('/')
                {
                    at = set.child(at, token(part)?);
                }
            }
            set.node_mut(at).matches.push(id);
        }
        Ok(set)
    }

    fn node_mut(&mut self, at: usize) -> &mut Node {
        self.nodes.get_mut(at).expect("compiled node exists")
    }

    fn child(&mut self, at: usize, key: String) -> usize {
        if let Some(id) = self.nodes.get(at).and_then(|n| n.keys.get(&key)) {
            return *id;
        }
        let id = self.nodes.len();
        self.nodes.push(Node::default());
        let parent = self.node_mut(at);
        if (key == "0" || !key.starts_with('0'))
            && key.bytes().all(|c| c.is_ascii_digit())
            && let Ok(index) = key.parse::<usize>()
        {
            parent.indices.insert(index, id);
        }
        parent.keys.insert(key, id);
        id
    }

    /// Read from the start of a parser to EOF and emit the complete event stream
    /// of each selected value. Overlapping or repeated pointers each receive
    /// their own events. Key, number and string bytes borrow the current token.
    /// A missing member produces no events. If a member occurs more than once,
    /// each occurrence is delivered. Multi-document parsing selects from every root.
    ///
    /// # Errors
    /// Returns parser errors even inside unmatched values. Earlier matches may
    /// already have been delivered when a later error is found.
    pub fn select<L: Lexer>(
        &self,
        parser: &mut PullParser<L>,
        mut emit: impl FnMut(usize, Event, &[u8]),
    ) -> Result<(), Error> {
        let mut frames: Vec<Frame> = Vec::new();
        let mut active = Vec::new();
        loop {
            let event = parser.next_event()?;
            match event {
                Event::Eof => return Ok(()),
                Event::End => {}
                Event::ArrayEnd | Event::ObjectEnd => {
                    for &id in &active {
                        emit(id, event, &[]);
                    }
                    if let Some(frame) = frames.pop() {
                        active.truncate(frame.outer_matches);
                    }
                }
                Event::Key { escaped } => {
                    let content = parser.content();
                    for &id in &active {
                        emit(id, event, content);
                    }
                    if let Some(frame) = frames.last_mut() {
                        frame.member =
                            frame.node.and_then(|n| self.nodes.get(n)).and_then(|node| {
                                let raw = core::str::from_utf8(content).ok()?;
                                if escaped {
                                    node.keys.get(&crate::unescape::unescape(raw)).copied()
                                } else {
                                    node.keys.get(raw).copied()
                                }
                            });
                    }
                }
                value => {
                    let next = match frames.last_mut() {
                        None => Some(0),
                        Some(frame) => {
                            if let Some(i) = frame.array_index {
                                frame.array_index = i.checked_add(1);
                                frame
                                    .node
                                    .and_then(|n| self.nodes.get(n)?.indices.get(&i).copied())
                            } else {
                                frame.member.take()
                            }
                        }
                    };
                    let outer_matches = active.len();
                    if let Some(node) = next.and_then(|n| self.nodes.get(n)) {
                        active.extend_from_slice(&node.matches);
                    }
                    let bytes = if matches!(value, Event::Num | Event::Str { .. }) {
                        parser.content()
                    } else {
                        &[]
                    };
                    for &id in &active {
                        emit(id, value, bytes);
                    }
                    if matches!(value, Event::ObjectStart | Event::ArrayStart) {
                        if active.is_empty() && next.is_none() {
                            parser.skip_open()?;
                        } else {
                            frames.push(Frame {
                                node: next,
                                array_index: (value == Event::ArrayStart).then_some(0),
                                member: None,
                                outer_matches,
                            });
                        }
                    } else {
                        active.truncate(outer_matches);
                    }
                }
            }
        }
    }
}
