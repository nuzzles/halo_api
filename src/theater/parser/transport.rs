//! Native source loading: compressed or clear chunks, indexed once in source order.
use flate2::{Decompress, FlushDecompress, Status};
use std::borrow::Cow;

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

// Legacy fixture adapters are test-only; production parsing consumes FilmChunk directly.
#[cfg(test)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FilmSourceMetadata {
    pub index: i64,
    pub chunk_type: i64,
    pub start_ms: i64,
}

#[cfg(test)]
pub(crate) struct FilmSource {
    original_chunks: Vec<Vec<u8>>,
    metadata: Vec<FilmSourceMetadata>,
}
#[cfg(test)]
impl FilmSource {
    pub(crate) fn load(
        chunks: &[impl AsRef<[u8]>],
        metadata: &[FilmSourceMetadata],
    ) -> Result<Self, &'static str> {
        if chunks.is_empty() {
            return Err("empty film input");
        }
        Ok(Self {
            original_chunks: chunks.iter().map(|c| c.as_ref().to_vec()).collect(),
            metadata: metadata.to_vec(),
        })
    }
    pub(crate) fn original_chunks(&self) -> &[Vec<u8>] {
        &self.original_chunks
    }
    pub(crate) fn metadata(&self) -> &[FilmSourceMetadata] {
        &self.metadata
    }
}

pub(crate) fn native_packet_bytes(
    data: &[u8],
    chunk_index: i32,
) -> Vec<crate::theater::film::FilmPacketHeader> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    while let Some(header) = data.get(offset..offset.saturating_add(16)) {
        let kind = u16::from_le_bytes(header[..2].try_into().unwrap());
        let size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        let stamp = u64::from_le_bytes(header[8..16].try_into().unwrap());
        let start = offset + 16;
        let Some(end) = start.checked_add(size).filter(|end| *end <= data.len()) else {
            break;
        };
        if size == 0 && kind != 7 {
            break;
        }
        out.push(crate::theater::film::FilmPacketHeader {
            chunk_index,
            packet_type: kind,
            byte_2: header[2],
            byte_3: header[3],
            payload_offset: start,
            payload_size: size,
            timestamp_us: stamp,
        });
        offset = end;
        if kind == 7 {
            break;
        }
    }
    out
}
