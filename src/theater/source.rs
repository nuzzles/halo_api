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

/// Strict native `Decompresser`: distinguish header failures from later stream failures.
/// Trailing bytes after the first zlib stream are ignored.
pub fn decompress_film_chunk(raw: &[u8]) -> Result<Vec<u8>, FilmInflateError> {
    let header = zlib_header(raw)?;
    let (out, result) = inflate_body(raw, header);
    result.map(|()| out)
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

/// Native `WalkPackets` for a single already-decompressed chunk. The source
/// position is zero; per-chunk packet ordinal is the returned vector index.
pub fn walk_film_packets(data: &[u8]) -> Vec<FilmPacket> {
    native_packet_bytes(data, 0)
}

/// A native source packet with a caller-supplied tag and borrowed payload.
/// The tag is not necessarily a position in a `FilmSource`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaggedFilmPacket<'a> {
    pub chunk_index: i64,
    pub packet_index: usize,
    pub packet_type: u16,
    pub timestamp_us: u64,
    pub byte_2: u8,
    pub byte_3: u8,
    /// Byte offset within the supplied decompressed chunk, after the header.
    pub payload_offset: usize,
    pub payload: &'a [u8],
}

/// Native `Paquets`, including the full signed 64-bit caller tag range.
/// Payloads borrow the input. Walking stops at the native terminator or malformed
/// boundary; this result does not represent any trailing unparsed bytes.
pub fn walk_tagged_film_packets(data: &[u8], chunk_index: i64) -> Vec<TaggedFilmPacket<'_>> {
    native_packet_bytes(data, 0)
        .into_iter()
        .enumerate()
        .map(|(packet_index, packet)| TaggedFilmPacket {
            chunk_index,
            packet_index,
            packet_type: packet.packet_type,
            timestamp_us: packet.timestamp_us,
            byte_2: packet.byte_2,
            byte_3: packet.byte_3,
            payload_offset: packet.payload_offset,
            payload: &data[packet.payload_offset..packet.payload_offset + packet.payload_size],
        })
        .collect()
}

/// Native single-file wrapper (`ReadFilmChunk`), including tolerant inflation.
#[cfg(not(target_arch = "wasm32"))]
pub fn read_film_chunk(
    directory: impl AsRef<std::path::Path>,
    number: i64,
) -> Result<Vec<u8>, std::io::Error> {
    let raw = std::fs::read(directory.as_ref().join(format!("chunk_{number:02}.bin")))?;
    Ok(inflate_film_chunk(&raw).into_owned())
}

/// Native compatibility count (`CountFilmChunks`): stop at the first missing or
/// inaccessible numbered path, starting at one. This does not enumerate later files.
#[cfg(not(target_arch = "wasm32"))]
pub fn count_film_chunks(directory: impl AsRef<std::path::Path>) -> usize {
    let mut count = 0usize;
    while let Some(next) = count.checked_add(1) {
        if std::fs::metadata(directory.as_ref().join(format!("chunk_{next:02}.bin"))).is_err() {
            break;
        }
        count = next;
    }
    count
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
pub struct FilmDirectorySource {
    files: Vec<std::path::PathBuf>,
    numbers: Vec<i64>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, thiserror::Error)]
pub enum FilmDirectoryChunkError {
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
    /// Move loaded bytes into the existing LegacyFilm constructors. Metadata must
    /// explicitly provide every chunk's role and file number; neither is guessed.
    pub fn into_chunks(
        self,
    ) -> Result<Vec<crate::clients::hi::models::FilmChunkData>, FilmSourceError> {
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        self.chunks
            .into_iter()
            .enumerate()
            .map(|(position, data)| {
                let m = self
                    .metadata
                    .get(position)
                    .ok_or(FilmSourceError::MissingMetadata(position))?;
                Ok(FilmChunkData {
                    metadata: FilmChunk {
                        index: i32::try_from(m.index)
                            .map_err(|_| FilmSourceError::ChunkNumber(m.index))?,
                        start_time_offset_ms: m.start_ms,
                        chunk_type: i32::try_from(m.chunk_type)
                            .map_err(|_| FilmSourceError::ChunkType(m.chunk_type))?,
                        duration_ms: 0,
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data,
                })
            })
            .collect()
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
    pub fn all_packets(&self) -> &[FilmPacket] {
        &self.packets
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
pub fn film_chunk_number(name: &str) -> i64 {
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
pub fn align_film_source_metadata(
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_fallible_source_provider() {
        use serde_json::{Value, json};
        #[derive(Debug, thiserror::Error)]
        #[error("provider read rejected")]
        struct ProviderFault(i64);
        struct Provider {
            count: i64,
            fail: i64,
            tag: i64,
            chunks: Vec<Vec<u8>>,
            calls: Vec<i64>,
        }
        impl FilmChunkProvider for Provider {
            type Error = ProviderFault;
            fn num_chunks(&mut self) -> i64 {
                self.calls.push(-1);
                self.count
            }
            fn chunk(&mut self, position: usize) -> Result<Cow<'_, [u8]>, ProviderFault> {
                self.calls.push(position as i64);
                if position as i64 == self.fail {
                    return Err(ProviderFault(self.tag));
                }
                Ok(Cow::Borrowed(&self.chunks[position]))
            }
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/source-provider-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(rows.len(), 256);
        let hex = |data: &[u8]| data.iter().map(|b| format!("{b:02x}")).collect::<String>();
        for (i, row) in rows.iter().enumerate() {
            let chunks = row["raw"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    let text = v.as_str().unwrap();
                    (0..text.len())
                        .step_by(2)
                        .map(|j| u8::from_str_radix(&text[j..j + 2], 16).unwrap())
                        .collect()
                })
                .collect();
            let mut provider = Provider {
                count: row["count"].as_i64().unwrap(),
                fail: row["fail"].as_i64().unwrap(),
                tag: i as i64,
                chunks,
                calls: vec![],
            };
            let mut meta: Vec<_> = row["meta"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| FilmSourceMetadata {
                    index: m["Index"].as_i64().unwrap(),
                    chunk_type: m["ChunkType"].as_i64().unwrap(),
                    start_ms: m["StartMS"].as_i64().unwrap(),
                })
                .collect();
            let erased: &mut dyn FilmChunkProvider<Error = ProviderFault> = &mut provider;
            let result = FilmSource::load_from(erased, &meta);
            assert_eq!(json!(provider.calls), row["calls"], "read order {i}");
            match result {
                Err(FilmSourceError::Empty) => {
                    assert!(provider.count <= 0);
                    assert_eq!(row["cause"], false);
                }
                Err(error @ FilmSourceError::ChunkRead { .. }) => {
                    assert_eq!(row["cause"], true);
                    assert_eq!(error.to_string(), row["error"].as_str().unwrap());
                    let FilmSourceError::ChunkRead { position, source } = error else {
                        unreachable!()
                    };
                    assert_eq!(position as i64, provider.fail);
                    assert_eq!(source.downcast_ref::<ProviderFault>().unwrap().0, i as i64);
                }
                Err(error) => panic!("unexpected {i}: {error}"),
                Ok(source) => {
                    assert_eq!(row["error"], "");
                    assert_eq!(row["cause"], false);
                    // Loaded bytes and metadata must not depend on provider storage.
                    provider.chunks.clear();
                    meta.clear();
                    assert_eq!(
                        json!(
                            (0..source.num_chunks())
                                .map(|p| hex(source.chunk(p).unwrap()))
                                .collect::<Vec<_>>()
                        ),
                        row["loaded"]
                    );
                    let copied:Vec<_>=source.metadata().iter().map(|m|json!({"Index":m.index,"ChunkType":m.chunk_type,"StartMS":m.start_ms})).collect();
                    let expected = if row["copied"].is_null() {
                        json!([])
                    } else {
                        row["copied"].clone()
                    };
                    assert_eq!(json!(copied), expected);
                    let mut packets = Vec::new();
                    for p in 0..source.num_chunks() {
                        for (index, packet) in source.packets(p).iter().enumerate() {
                            packets.push(json!({"chunk":p,"index":index,"type":packet.packet_type,"timestamp":packet.timestamp_us,"hex":hex(source.payload(packet).unwrap())}));
                        }
                    }
                    assert_eq!(json!(packets), row["packets"], "packets {i}");
                }
            }
        }
    }

    #[test]
    fn native_source_loading() {
        #[derive(Deserialize)]
        struct Packet {
            chunk: i64,
            index: usize,
            #[serde(rename = "type")]
            kind: u16,
            timestamp: u64,
            hex: String,
        }
        #[derive(Deserialize)]
        struct TaggedPacket {
            #[serde(flatten)]
            packet: Packet,
            offset: usize,
            byte_2: u8,
            byte_3: u8,
        }
        #[derive(Deserialize)]
        struct Tagged {
            tag: i64,
            packets: Vec<TaggedPacket>,
        }
        #[derive(Deserialize)]
        struct Case {
            raw: String,
            clear: String,
            strict: String,
            error: String,
            chunks: Vec<String>,
            counts: Vec<usize>,
            packets: Vec<Packet>,
            metadata: Vec<FilmSourceMetadata>,
            aligned: Vec<FilmSourceMetadata>,
            tagged: Vec<Tagged>,
        }
        fn unhex(s: &str) -> Vec<u8> {
            s.as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
                .collect()
        }
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/source-v41.json.zlib")[..])
            .read_to_end(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&json).unwrap();
        assert_eq!(cases.len(), 512);
        for (i, c) in cases.into_iter().enumerate() {
            let raw = unhex(&c.raw);
            let clear = unhex(&c.clear);
            assert_eq!(c.tagged.len(), 9);
            for tagged in &c.tagged {
                let actual = walk_tagged_film_packets(&clear, tagged.tag);
                assert_eq!(actual.len(), tagged.packets.len(), "tagged count {i}");
                for (a, e) in actual.iter().zip(&tagged.packets) {
                    assert_eq!(a.chunk_index, e.packet.chunk);
                    assert_eq!(a.packet_index, e.packet.index);
                    assert_eq!(a.packet_type, e.packet.kind);
                    assert_eq!(a.timestamp_us, e.packet.timestamp);
                    assert_eq!(a.payload, unhex(&e.packet.hex));
                    assert_eq!(a.payload_offset, e.offset);
                    assert_eq!((a.byte_2, a.byte_3), (e.byte_2, e.byte_3));
                    assert_eq!(a.payload.as_ptr(), clear[e.offset..].as_ptr());
                }
            }
            let strict = decompress_film_chunk(&raw);
            match c.error.as_str() {
                "header" => assert_eq!(strict, Err(FilmInflateError::Header), "strict {i}"),
                "stream" => assert_eq!(strict, Err(FilmInflateError::Stream), "strict {i}"),
                "" => assert_eq!(strict, Ok(unhex(&c.strict)), "strict {i}"),
                s => panic!("unknown error {s}"),
            }
            let source = FilmSource::load(&[raw, clear, vec![]], &c.metadata).unwrap();
            assert_eq!(source.num_chunks(), 3);
            assert_eq!(source.metadata(), c.metadata);
            assert_eq!(
                align_film_source_metadata(&[7, 99, -1, 7], &c.metadata),
                c.aligned
            );
            assert_eq!(source.chunk(4), None);
            assert!(source.packets(4).is_empty());
            for j in 0..3 {
                assert_eq!(
                    source.chunk(j).unwrap(),
                    unhex(&c.chunks[j]),
                    "chunk {i}/{j}"
                );
                assert_eq!(source.packets(j).len(), c.counts[j], "count {i}/{j}");
            }
            assert_eq!(source.all_packets().len(), c.packets.len(), "packets {i}");
            for (actual, expected) in source.all_packets().iter().zip(&c.packets) {
                assert_eq!(
                    (
                        i64::from(actual.chunk_index),
                        actual.packet_type,
                        actual.timestamp_us
                    ),
                    (expected.chunk, expected.kind, expected.timestamp),
                    "packet {i}"
                );
                assert_eq!(
                    source.packets(expected.chunk as usize)[expected.index],
                    *actual
                );
                assert_eq!(
                    source.payload(actual).unwrap(),
                    unhex(&expected.hex),
                    "payload {i}"
                );
            }
        }
        assert!(matches!(
            FilmSource::load(&[] as &[Vec<u8>], &[]),
            Err(FilmSourceError::Empty)
        ));
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod directory_tests {
    use super::*;
    #[test]
    fn native_directory_source_lifecycle() {
        use serde_json::{Value, json};
        use std::io::Read;
        struct Trace {
            inner: FilmDirectorySource,
            calls: Vec<i64>,
        }
        impl FilmChunkProvider for Trace {
            type Error = FilmDirectoryChunkError;
            fn num_chunks(&mut self) -> i64 {
                self.calls.push(-1);
                self.inner.num_chunks()
            }
            fn chunk(&mut self, position: usize) -> Result<Cow<'_, [u8]>, Self::Error> {
                self.calls.push(position as i64);
                self.inner.chunk(position)
            }
        }
        fn kind(error: &FilmDirectoryChunkError) -> &'static str {
            match error {
                FilmDirectoryChunkError::Read { source, .. } => match source.kind() {
                    std::io::ErrorKind::NotFound => "missing",
                    std::io::ErrorKind::IsADirectory => "directory",
                    _ => "other",
                },
                _ => "other",
            }
        }
        fn unhex(text: &str) -> Vec<u8> {
            (0..text.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&text[j..j + 2], 16).unwrap())
                .collect()
        }
        fn hex(bytes: &[u8]) -> String {
            bytes.iter().map(|b| format!("{b:02x}")).collect()
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/source-directory-lifecycle-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(rows.len(), 64);
        for (case, row) in rows.iter().enumerate() {
            let directory = std::env::temp_dir().join(format!(
                "halo-directory-lifecycle-{}-{case}",
                std::process::id()
            ));
            std::fs::create_dir(&directory).unwrap();
            let inputs = row["inputs"].as_array().unwrap();
            for input in inputs {
                std::fs::write(
                    directory.join(input["name"].as_str().unwrap()),
                    unhex(input["hex"].as_str().unwrap()),
                )
                .unwrap();
            }
            let mut source = FilmDirectorySource::open(&directory).unwrap();
            assert_eq!(json!(source.chunk_numbers()), row["numbers"]);
            let first = source.chunk(0).unwrap().into_owned();
            std::fs::write(
                directory.join(inputs[0]["name"].as_str().unwrap()),
                [0xa5, case as u8],
            )
            .unwrap();
            assert_eq!(
                hex(&source.chunk(0).unwrap()),
                row["second"].as_str().unwrap()
            );
            assert_eq!(hex(&first), row["first"].as_str().unwrap());
            std::fs::write(directory.join("chunk_999.bin"), [0xff]).unwrap();
            assert_eq!(source.num_chunks(), inputs.len() as i64);
            let fail = row["fail"].as_i64().unwrap();
            if fail >= 0 {
                let path = directory.join(inputs[fail as usize]["name"].as_str().unwrap());
                std::fs::remove_file(&path).unwrap();
                if row["action"] == "directory" {
                    std::fs::create_dir(&path).unwrap();
                }
            }
            for position in 0..inputs.len() {
                let observed = match source.chunk(position) {
                    Ok(bytes) => json!({"hex":hex(&bytes),"kind":""}),
                    Err(error) => {
                        let FilmDirectoryChunkError::Read { path, .. } = &error else {
                            panic!("unexpected bounds error")
                        };
                        assert_eq!(
                            path,
                            &directory.join(inputs[position]["name"].as_str().unwrap())
                        );
                        assert!(
                            std::error::Error::source(&error)
                                .unwrap()
                                .downcast_ref::<std::io::Error>()
                                .is_some()
                        );
                        json!({"hex":"","kind":kind(&error)})
                    }
                };
                assert_eq!(
                    observed, row["reads"][position],
                    "case {case} read {position}"
                );
            }
            assert_eq!(
                source.chunk(inputs.len()).unwrap_err().to_string(),
                row["bounds"].as_str().unwrap()
            );
            let mut traced = Trace {
                inner: source,
                calls: vec![],
            };
            let result = FilmSource::load_from(&mut traced, &[]);
            assert_eq!(json!(traced.calls), row["calls"], "case {case} calls");
            match result {
                Ok(film) => {
                    assert_eq!(row["load_kind"], "");
                    let loaded: Vec<_> = (0..film.num_chunks())
                        .map(|p| hex(film.chunk(p).unwrap()))
                        .collect();
                    assert_eq!(json!(loaded), row["loaded"]);
                }
                Err(FilmSourceError::ChunkRead { position, source }) => {
                    assert_eq!(position as i64, fail);
                    assert_eq!(
                        kind(source.downcast_ref::<FilmDirectoryChunkError>().unwrap()),
                        row["load_kind"].as_str().unwrap()
                    );
                }
                Err(error) => panic!("case {case}: {error}"),
            }
            std::fs::remove_dir_all(directory).unwrap();
        }
    }

    #[test]
    fn native_source_directory() {
        #[derive(Deserialize)]
        struct Input {
            name: String,
            hex: String,
        }
        #[derive(Deserialize)]
        struct Number {
            name: String,
            number: i64,
        }
        #[derive(Deserialize)]
        struct Case {
            inputs: Vec<Input>,
            metadata: Vec<FilmSourceMetadata>,
            chunks: Vec<String>,
            aligned: Vec<FilmSourceMetadata>,
            numbers: Vec<Number>,
        }
        fn unhex(s: &str) -> Vec<u8> {
            s.as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
                .collect()
        }
        let c: Case =
            serde_json::from_slice(include_bytes!("fixtures/source-directory-v41.json")).unwrap();
        for n in &c.numbers {
            assert_eq!(film_chunk_number(&n.name), n.number, "{}", n.name);
        }
        let path =
            std::env::temp_dir().join(format!("halo-source-directory-{}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        for input in &c.inputs {
            std::fs::write(path.join(&input.name), unhex(&input.hex)).unwrap();
        }
        let source = FilmSource::load_directory(&path, &c.metadata).unwrap();
        std::fs::remove_dir_all(&path).unwrap();
        assert_eq!(source.metadata(), c.aligned);
        for (i, bytes) in c.chunks.iter().enumerate() {
            assert_eq!(source.chunk(i).unwrap(), unhex(bytes));
        }
        assert_eq!(source.num_chunks(), c.chunks.len());
        let chunks = source.into_chunks().unwrap();
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.data, unhex(&c.chunks[i]));
            assert_eq!(i64::from(chunk.metadata.index), c.aligned[i].index);
            assert_eq!(
                i64::from(chunk.metadata.chunk_type),
                c.aligned[i].chunk_type
            );
            assert_eq!(chunk.metadata.start_time_offset_ms, c.aligned[i].start_ms);
        }
        let missing = FilmSource::load(&[vec![0u8]], &[]).unwrap();
        assert!(matches!(
            missing.into_chunks(),
            Err(FilmSourceError::MissingMetadata(0))
        ));
    }
}
