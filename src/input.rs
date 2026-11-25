//! File snapshots for DOM parsing. Use `PullParser::from_reader` with the
//! `read` feature when the entire input should not live in memory.

#[cfg(feature = "std")]
use crate::config::BufConfig;
#[cfg(all(feature = "mmap", not(any(unix, windows))))]
use rs_mmap::ReadOnlyMmap as Mmap;
#[cfg(all(feature = "mmap", any(unix, windows)))]
use rs_mmap::compat::{Mmap, MmapMut};

/// Owned source bytes. Call [`FileDocument::document`] to validate and parse them.
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct FileDocument {
    /// Backing storage for the snapshot.
    pub storage: Storage,
}

/// Storage owned by a [`FileDocument`].
#[cfg(feature = "std")]
#[derive(Debug)]
pub enum Storage {
    /// Bytes read into a growable buffer.
    Heap(alloc::vec::Vec<u8>),
    /// Read-only mapping of a snapshot or a caller-guaranteed immutable file.
    #[cfg(feature = "mmap")]
    Mmap(Mmap),
}

#[cfg(feature = "std")]
impl FileDocument {
    /// File bytes, independent of later changes to the source file.
    pub fn bytes(&self) -> &[u8] {
        match &self.storage {
            Storage::Heap(v) => v,
            #[cfg(feature = "mmap")]
            Storage::Mmap(m) => m,
        }
    }

    /// Parse a tape document borrowing this snapshot.
    ///
    /// # Errors
    /// Returns a parse error for invalid JSON, invalid UTF-8 or exceeded limits.
    pub fn document(&self) -> Result<crate::Document<'_>, crate::Error> {
        crate::parse_bytes(self.bytes())
    }
}

/// Read a snapshot. With `mmap`, regular files larger than `small_file_max`
/// and at least `mmap_min` bytes are copied into anonymous read-only mappings.
/// A writer modifying the file during the read can produce a mixed snapshot.
///
/// # Errors
/// Reports allocation, mapping and file I/O errors.
#[cfg(feature = "std")]
pub fn parse_file(
    path: impl AsRef<std::path::Path>,
    cfg: BufConfig,
) -> std::io::Result<FileDocument> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let meta = file.metadata()?;
    #[cfg(feature = "mmap")]
    if meta.is_file() && meta.len() > cfg.small_file_max && meta.len() >= cfg.mmap_min {
        #[cfg(any(unix, windows))]
        return mapped_snapshot(file, meta.len());
        #[cfg(not(any(unix, windows)))]
        return Err(std::io::ErrorKind::Unsupported.into());
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(usize::try_from(meta.len().min(cfg.small_file_max)).unwrap_or(0))
        .map_err(std::io::Error::other)?;
    file.read_to_end(&mut bytes)?;
    Ok(FileDocument {
        storage: Storage::Heap(bytes),
    })
}

#[cfg(all(feature = "mmap", any(unix, windows)))]
fn mapped_snapshot(mut file: std::fs::File, len: u64) -> std::io::Result<FileDocument> {
    use std::io::Read;
    let len = usize::try_from(len).map_err(std::io::Error::other)?;
    let mut mapping = MmapMut::map_anon(len)?;
    let mut used = 0;
    while let Some(rest) = mapping.get_mut(used..).filter(|s| !s.is_empty()) {
        match file.read(rest) {
            Ok(0) => {
                return Ok(FileDocument {
                    storage: Storage::Heap(mapping.get(..used).unwrap_or_default().to_vec()),
                });
            }
            Ok(n) => used += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    // If the file grew after stat, keep the tail too.
    let mut tail = Vec::new();
    file.read_to_end(&mut tail)?;
    if tail.is_empty() {
        return Ok(FileDocument {
            storage: Storage::Mmap(mapping.make_read_only()?),
        });
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve(
            len.checked_add(tail.len())
                .ok_or(std::io::ErrorKind::OutOfMemory)?,
        )
        .map_err(std::io::Error::other)?;
    bytes.extend_from_slice(&mapping);
    bytes.extend_from_slice(&tail);
    Ok(FileDocument {
        storage: Storage::Heap(bytes),
    })
}

/// Map the bytes of an immutable file directly. Empty files use empty heap storage.
///
/// # Safety
/// The caller must ensure that no process modifies or truncates the underlying
/// file from before this call until the returned mapping is dropped. Use
/// [`parse_file`] for files that other writers can access.
///
/// # Errors
/// Returns file open, metadata or memory mapping errors.
#[cfg(feature = "mmap")]
#[allow(
    unsafe_code,
    reason = "the caller guarantees immutable backing for the lifetime of the mapping"
)]
pub unsafe fn map_immutable_file(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<FileDocument> {
    let file = std::fs::File::open(path)?;
    let storage = if file.metadata()?.len() == 0 {
        Storage::Heap(Vec::new())
    } else {
        #[cfg(any(unix, windows))]
        {
            // SAFETY: the caller guarantees the file's contents and length are stable.
            Storage::Mmap(unsafe { Mmap::map(&file)? })
        }
        #[cfg(not(any(unix, windows)))]
        return Err(std::io::ErrorKind::Unsupported.into());
    };
    Ok(FileDocument { storage })
}
