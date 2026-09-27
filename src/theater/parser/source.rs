//! Native source loading: compressed or clear chunks, indexed once in source order.
use super::{FilmPacket, fire_events::native_packet_bytes};
use flate2::{Decompress, FlushDecompress, Status};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, ops::Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FilmInflateError {
    #[error("invalid or incomplete zlib header")]
    Header,
    #[error("damaged or incomplete zlib stream")]
    Stream,
}

fn zlib_header(raw: &[u8]) -> Result<usize, FilmInflateError> {
    let h = raw.get(..2).ok_or(FilmInflateError::Header)?;
    if h[0] & 15 != 8 || h[0] >> 4 > 7 || u16::from_be_bytes([h[0], h[1]]) % 31 != 0 {
        return Err(FilmInflateError::Header);
    }
    if h[1] & 32 != 0 {
        // Native NewReader uses a nil dictionary, whose Adler checksum is one.
        if raw.get(2..6) != Some(&[0, 0, 0, 1]) {
            return Err(FilmInflateError::Header);
        }
        Ok(6)
    } else {
        Ok(2)
    }
}

fn inflate_body(raw: &[u8], header: usize) -> (Vec<u8>, Result<(), FilmInflateError>) {
    let mut decoder = Decompress::new(false);
    let mut out = Vec::with_capacity(raw.len().saturating_mul(6).min(64 << 20));
    let result = loop {
        let before = (decoder.total_in(), decoder.total_out());
        let mut block = [0; 8192];
        let status = decoder.decompress(
            &raw[header + before.0 as usize..],
            &mut block,
            FlushDecompress::None,
        );
        out.extend_from_slice(&block[..(decoder.total_out() - before.1) as usize]);
        match status {
            Ok(Status::StreamEnd) => {
                let end = header + decoder.total_in() as usize;
                let checksum = raw.get(end..end.saturating_add(4));
                let (mut a, mut b) = (1u32, 0u32);
                for &byte in &out {
                    a = (a + u32::from(byte)) % 65521;
                    b = (b + a) % 65521;
                }
                break if checksum == Some(&(b << 16 | a).to_be_bytes()) {
                    Ok(())
                } else {
                    Err(FilmInflateError::Stream)
                };
            }
            Err(_) => break Err(FilmInflateError::Stream),
            _ if before == (decoder.total_in(), decoder.total_out()) => {
                break Err(FilmInflateError::Stream);
            }
            _ => {}
        }
    };
    (out, result)
}

/// Native `Inflate`: clear/invalid input passes through; a damaged stream keeps
/// its decompressed prefix, unless it produced no bytes, in which case raw input survives.
pub fn inflate_film_chunk(raw: &[u8]) -> Cow<'_, [u8]> {
    if raw.len() < 2 || raw[0] != 0x78 {
        return Cow::Borrowed(raw);
    }
    let Ok(header) = zlib_header(raw) else {
        return Cow::Borrowed(raw);
    };
    let (out, status) = inflate_body(raw, header);
    if status.is_err() && out.is_empty() {
        Cow::Borrowed(raw)
    } else {
        Cow::Owned(out)
    }
}

/// Positional metadata used by the reference source layer; file number and
/// loaded chunk position are deliberately separate. Native manifest integers
/// retain the pinned reference's signed 64-bit range, even for unknown types.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmSourceMetadata {
    pub index: i64,
    pub chunk_type: i64,
    pub start_ms: i64,
}

/// Ordered, fallible raw chunk access, matching the native Source contract.
/// The count is queried once. Chunks may be borrowed or owned, compressed or raw.
/// A negative or zero count denotes an empty source; an empty chunk is valid.
pub trait FilmChunkProvider {
    type Error: std::error::Error + Send + Sync + 'static;
    fn num_chunks(&mut self) -> i64;
    fn chunk(&mut self, position: usize) -> Result<Cow<'_, [u8]>, Self::Error>;
}

impl<T: AsRef<[u8]>> FilmChunkProvider for &[T] {
    type Error = std::io::Error;
    fn num_chunks(&mut self) -> i64 {
        i64::try_from(self.len()).unwrap_or(i64::MAX)
    }
    fn chunk(&mut self, position: usize) -> Result<Cow<'_, [u8]>, Self::Error> {
        self.get(position)
            .map(|data| Cow::Borrowed(data.as_ref()))
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "source: chunk {position} hors bornes ({} chunks)",
                        self.len()
                    ),
                )
            })
    }
}

/// A directory snapshot of chunk paths in native numeric order. File contents
/// are read on each `chunk` call, not when the provider is opened. Newly added
/// paths do not change this provider's count; removed paths produce read errors.
/// The directory argument is a literal path, not a glob expression.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub(crate) struct FilmDirectorySource {
    files: Vec<std::path::PathBuf>,
    numbers: Vec<i64>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, thiserror::Error)]
pub(crate) enum FilmDirectoryChunkError {
    #[error("source: chunk {position} hors bornes ({count} chunks)")]
    OutOfBounds { position: usize, count: usize },
    #[error("source: {path}: {source}", path = path.display())]
    Read {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(not(target_arch = "wasm32"))]
impl FilmDirectorySource {
    /// Discover `chunk_*.bin` entries without reading their contents. An empty
    /// directory is an error. Entries that are directories remain in the source
    /// and fail when read, matching native discovery rather than being skipped.
    pub fn open(directory: impl AsRef<std::path::Path>) -> Result<Self, FilmSourceError> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let name = entry.file_name();
            let bytes = name.as_encoded_bytes();
            if bytes.starts_with(b"chunk_") && bytes.ends_with(b".bin") {
                let number = name.to_str().map(film_chunk_number).unwrap_or(-1);
                files.push((number, name, entry.path()));
            }
        }
        // filepath.Glob supplies lexicographic order before native sort.Slice.
        files.sort_by(|a, b| a.1.cmp(&b.1));
        super::native_sort::sort_by(&mut files, |a, b| {
            let key = |n| if n == -1 { i64::MAX } else { n };
            if a.0 != b.0 {
                key(a.0).cmp(&key(b.0))
            } else {
                a.1.cmp(&b.1)
            }
        });

        if files.is_empty() {
            return Err(FilmSourceError::Empty);
        }
        Ok(Self {
            numbers: files.iter().map(|v| v.0).collect(),
            files: files.into_iter().map(|v| v.2).collect(),
        })
    }

    /// Native file numbers in source-position order, including -1 for unknowns.
    pub fn chunk_numbers(&self) -> &[i64] {
        &self.numbers
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FilmChunkProvider for FilmDirectorySource {
    type Error = FilmDirectoryChunkError;

    fn num_chunks(&mut self) -> i64 {
        i64::try_from(self.files.len()).unwrap_or(i64::MAX)
    }

    fn chunk(&mut self, position: usize) -> Result<Cow<'_, [u8]>, Self::Error> {
        let path = self
            .files
            .get(position)
            .ok_or(FilmDirectoryChunkError::OutOfBounds {
                position,
                count: self.files.len(),
            })?;
        std::fs::read(path)
            .map(Cow::Owned)
            .map_err(|source| FilmDirectoryChunkError::Read {
                path: path.clone(),
                source,
            })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FilmSourceError {
    #[error("source: chunk {position}: {source}")]
    ChunkRead {
        position: usize,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("no film chunks in source")]
    Empty,
    #[error("film source contains too many chunks")]
    TooManyChunks,
    #[error("source metadata is missing for chunk position {0}")]
    MissingMetadata(usize),
    #[error("source chunk number {0} cannot be represented by FilmChunkData")]
    ChunkNumber(i64),
    #[error("source chunk type {0} cannot be represented by FilmChunkData")]
    ChunkType(i64),
    #[error("film source I/O: {0}")]
    Io(#[from] std::io::Error),
}

/// Read-only loaded bytes and packet ranges. Packets use source positions;
/// metadata numbers are not substituted for those positions.
#[derive(Debug, Clone)]
pub struct FilmSource {
    original_chunks: Vec<Vec<u8>>,
    chunks: Vec<Vec<u8>>,
    packets: Vec<FilmPacket>,
    bounds: Vec<Range<usize>>,
    metadata: Vec<FilmSourceMetadata>,
}
impl FilmSource {
    /// Load raw or zlib chunks once. Metadata is optional and copied as supplied,
    /// including a shorter/longer metadata slice, matching native `source.Load`.
    pub fn load(
        chunks: &[impl AsRef<[u8]>],
        metadata: &[FilmSourceMetadata],
    ) -> Result<Self, FilmSourceError> {
        let mut provider = chunks;
        Self::load_from(&mut provider, metadata)
    }

    /// Read each chunk once in source order, stopping at the first provider error.
    /// Errors preserve the failing position and original cause; no partial FilmSource
    /// is published. Source positions and supplied metadata numbers stay distinct.
    pub fn load_from<P: FilmChunkProvider + ?Sized>(
        provider: &mut P,
        metadata: &[FilmSourceMetadata],
    ) -> Result<Self, FilmSourceError> {
        let count = provider.num_chunks();
        if count <= 0 {
            return Err(FilmSourceError::Empty);
        }
        if count > i64::from(i32::MAX) {
            return Err(FilmSourceError::TooManyChunks);
        }
        let count = usize::try_from(count).map_err(|_| FilmSourceError::TooManyChunks)?;
        let mut result = Self {
            original_chunks: Vec::with_capacity(count),
            chunks: Vec::with_capacity(count),
            packets: Vec::new(),
            bounds: Vec::with_capacity(count),
            metadata: Vec::new(),
        };
        for position in 0..count {
            let raw = provider
                .chunk(position)
                .map_err(|source| FilmSourceError::ChunkRead {
                    position,
                    source: Box::new(source),
                })?;
            result.original_chunks.push(raw.as_ref().to_vec());
            let data = inflate_film_chunk(raw.as_ref()).into_owned();
            let start = result.packets.len();
            result
                .packets
                .extend(native_packet_bytes(&data, position as i32));
            result.bounds.push(start..result.packets.len());
            result.chunks.push(data);
        }
        result.metadata = metadata.to_vec();
        Ok(result)
    }

    /// Read `chunk_*.bin` in numeric file order, aligning manifest rows by
    /// file number. Unknown names sort last. No upper chunk-number limit is imposed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_directory(
        directory: impl AsRef<std::path::Path>,
        metadata: &[FilmSourceMetadata],
    ) -> Result<Self, FilmSourceError> {
        let mut provider = FilmDirectorySource::open(directory)?;
        let aligned = align_film_source_metadata(provider.chunk_numbers(), metadata);
        Self::load_from(&mut provider, &aligned)
    }

    /// Native `FilmChunkNumbers`: enumerate the metadata-order prefix 1, 2, ...,
    /// stopping at its first gap. With no metadata, position is the native number.
    /// This compatibility scan policy does not remove any loaded source bytes.
    pub fn data_chunk_numbers(&self) -> Vec<i64> {
        if self.metadata.is_empty() {
            return (1..self.chunks.len()).map(|i| i as i64).collect();
        }
        let mut result = Vec::new();
        let mut wanted = 1;
        for metadata in self.metadata.iter().take(self.chunks.len()) {
            if metadata.index < 1 {
                continue;
            }
            if metadata.index != wanted {
                break;
            }
            result.push(metadata.index);
            wanted += 1;
        }
        result
    }

    /// Native number-to-position translation. The first matching metadata entry
    /// wins, including negative numbers; entries beyond the loaded chunks are ignored.
    pub fn chunk_position(&self, number: i64) -> Option<usize> {
        if self.metadata.is_empty() {
            usize::try_from(number)
                .ok()
                .filter(|&i| i < self.chunks.len())
        } else {
            self.metadata
                .iter()
                .take(self.chunks.len())
                .position(|m| m.index == number)
        }
    }

    /// Borrow a numbered chunk and its cached packet ranges (`FilmChunkAt`).
    /// Packet `chunk_index` remains the source position, as in `all_packets`;
    /// callers already have the requested file number. Payload ranges are unchanged.
    pub fn chunk_by_number(&self, number: i64) -> Option<(&[u8], &[FilmPacket])> {
        let position = self.chunk_position(number)?;
        Some((self.chunk(position)?, self.packets(position)))
    }

    /// Native `FilmRegistryChunk`: retrieve number zero without parsing its bytes.
    pub fn registry_chunk(&self) -> Option<&[u8]> {
        self.chunk_by_number(0).map(|(bytes, _)| bytes)
    }

    /// Exact input bytes, including compression, damage and trailing transport bytes.
    pub fn original_chunks(&self) -> &[Vec<u8>] {
        &self.original_chunks
    }

    pub fn num_chunks(&self) -> usize {
        self.chunks.len()
    }
    pub fn chunk(&self, position: usize) -> Option<&[u8]> {
        self.chunks.get(position).map(Vec::as_slice)
    }
    pub fn packets(&self, position: usize) -> &[FilmPacket] {
        self.bounds
            .get(position)
            .map_or(&[], |range| &self.packets[range.clone()])
    }
    pub fn metadata(&self) -> &[FilmSourceMetadata] {
        &self.metadata
    }
    pub fn payload(&self, packet: &FilmPacket) -> Option<&[u8]> {
        self.chunk(usize::try_from(packet.chunk_index).ok()?)?
            .get(packet.payload_offset..packet.payload_offset.checked_add(packet.payload_size)?)
    }
}

/// Native file-number parsing; unknown or negative numbers return -1.
#[cfg(not(target_arch = "wasm32"))]
fn film_chunk_number(name: &str) -> i64 {
    let base = std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name);
    let stem = base.strip_suffix(".bin").unwrap_or(base);
    stem.strip_prefix("chunk_")
        .and_then(|n| n.parse::<i64>().ok())
        .filter(|&n| n >= 0)
        .unwrap_or(-1)
}

/// Native directory metadata alignment: first duplicate wins; unknown numbers
/// are retained as -1 without inheriting metadata from an unknown manifest row.
#[cfg(not(target_arch = "wasm32"))]
fn align_film_source_metadata(
    numbers: &[i64],
    metadata: &[FilmSourceMetadata],
) -> Vec<FilmSourceMetadata> {
    numbers
        .iter()
        .map(|&index| {
            metadata
                .iter()
                .find(|m| index != -1 && m.index == index)
                .copied()
                .unwrap_or(FilmSourceMetadata {
                    index,
                    ..Default::default()
                })
        })
        .collect()
}
