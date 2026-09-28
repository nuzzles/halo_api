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

/// Private decompression and packet index used only by the version dispatcher.
#[derive(Debug, Clone)]
pub(crate) struct FilmSource {
    #[cfg(test)]
    original_chunks: Vec<Vec<u8>>,
    chunks: Vec<Vec<u8>>,
    packets: Vec<FilmPacket>,
    bounds: Vec<Range<usize>>,
    metadata: Vec<FilmSourceMetadata>,
}
impl FilmSource {
    pub(crate) fn load(
        chunks: &[impl AsRef<[u8]>],
        metadata: &[FilmSourceMetadata],
    ) -> Result<Self, &'static str> {
        if chunks.is_empty() {
            return Err("empty film input");
        }
        if chunks.len() > i32::MAX as usize {
            return Err("too many chunks");
        }
        let mut result = Self {
            #[cfg(test)]
            original_chunks: chunks.iter().map(|c| c.as_ref().to_vec()).collect(),
            chunks: Vec::with_capacity(chunks.len()),
            packets: Vec::new(),
            bounds: Vec::with_capacity(chunks.len()),
            metadata: metadata.to_vec(),
        };
        for (position, raw) in chunks.iter().enumerate() {
            let data = inflate_film_chunk(raw.as_ref()).into_owned();
            let start = result.packets.len();
            result
                .packets
                .extend(native_packet_bytes(&data, position as i32));
            result.bounds.push(start..result.packets.len());
            result.chunks.push(data);
        }
        Ok(result)
    }
    #[cfg(test)]
    pub(crate) fn original_chunks(&self) -> &[Vec<u8>] {
        &self.original_chunks
    }
    pub(crate) fn num_chunks(&self) -> usize {
        self.chunks.len()
    }
    pub(crate) fn chunk(&self, position: usize) -> Option<&[u8]> {
        self.chunks.get(position).map(Vec::as_slice)
    }
    pub(crate) fn packets(&self, position: usize) -> &[FilmPacket] {
        self.bounds
            .get(position)
            .map_or(&[], |range| &self.packets[range.clone()])
    }
    pub(crate) fn metadata(&self) -> &[FilmSourceMetadata] {
        &self.metadata
    }
    pub(crate) fn payload(&self, packet: &FilmPacket) -> Option<&[u8]> {
        self.chunk(usize::try_from(packet.chunk_index).ok()?)?
            .get(packet.payload_offset..packet.payload_offset.checked_add(packet.payload_size)?)
    }
}
