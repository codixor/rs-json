//! File and reader entry points with size-based strategy.
//!
//! | File size | Strategy | Why |
//! |-----------|----------|-----|
//! | ≤ `small_file_max` (1 MiB) | `read_to_end` into a reused `Vec`, tape DOM | one read, hot cache |
//! | > `mmap_min` (8 MiB) + `mmap` | memory-map, tape DOM (≤ 4 GiB) | no copy, page cache |
//! | otherwise | `read_to_end` with exact capacity from metadata | one allocation |
//! | caller wants bounded memory / > 4 GiB | `ReadLexer` + `PullParser` | constant memory |
//!
//! Thresholds are placeholders; `benches/stream_large.rs` sets the final values.

#[cfg(feature = "std")]
use crate::config::BufConfig;

/// Owned buffer + document pair returned by [`parse_file`].
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct FileDocument {
    /// Backing storage (heap or mmap).
    pub storage: Storage,
}

/// Where the document bytes live.
#[cfg(feature = "std")]
#[derive(Debug)]
pub enum Storage {
    /// Read into memory.
    Heap(alloc::vec::Vec<u8>),
    /// Memory-mapped (read-only). The file must not be truncated while mapped.
    #[cfg(feature = "mmap")]
    Mmap(memmap2::Mmap),
}

#[cfg(feature = "std")]
impl FileDocument {
    /// Bytes of the file.
    pub fn bytes(&self) -> &[u8] {
        match &self.storage {
            Storage::Heap(v) => v,
            #[cfg(feature = "mmap")]
            Storage::Mmap(m) => m,
        }
    }

    /// Parse into a tape document borrowing this storage.
    ///
    /// # Errors
    /// Invalid JSON / UTF-8 / limits.
    pub fn document(&self) -> Result<crate::Document<'_>, crate::Error> {
        crate::parse_bytes(self.bytes())
    }
}

/// Load a file using the size-based strategy above.
///
/// # Errors
/// I/O errors from open/stat/read/mmap.
#[cfg(feature = "std")]
pub fn parse_file(
    path: impl AsRef<std::path::Path>,
    cfg: BufConfig,
) -> std::io::Result<FileDocument> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let len = f.metadata()?.len();
    #[cfg(feature = "mmap")]
    if len >= cfg.mmap_min {
        // SAFETY note for agent: memmap2::Mmap::map is unsafe; isolate it in a
        // dedicated `#[allow(unsafe_code)]` fn with the truncation caveat documented.
        todo!("agent: mmap path");
    }
    let _ = cfg.small_file_max;
    let mut buf = alloc::vec::Vec::with_capacity(usize::try_from(len).unwrap_or(0) + 1);
    f.read_to_end(&mut buf)?;
    Ok(FileDocument {
        storage: Storage::Heap(buf),
    })
}
