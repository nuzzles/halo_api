//! Reference source loading: compressed or clear chunks, indexed once in source order.
use crate::theater::film::ChunkTransport;
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
        // Reference NewReader uses a nil dictionary, whose Adler checksum is one.
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

/// Reference `Inflate`: clear/invalid input passes through; a damaged stream keeps
/// its decompressed prefix, unless it produced no bytes, in which case raw input survives.
pub fn inflate_film_chunk(raw: &[u8]) -> (Cow<'_, [u8]>, ChunkTransport) {
    if raw.len() < 2 || raw[0] != 0x78 {
        return (Cow::Borrowed(raw), ChunkTransport::Clear);
    }
    let Ok(header) = zlib_header(raw) else {
        return (Cow::Borrowed(raw), ChunkTransport::ZlibRejected);
    };
    let (out, status) = inflate_body(raw, header);
    if status.is_err() && out.is_empty() {
        (Cow::Borrowed(raw), ChunkTransport::ZlibRejected)
    } else {
        let transport = if status.is_ok() {
            ChunkTransport::ZlibComplete
        } else {
            ChunkTransport::ZlibPartial
        };
        (Cow::Owned(out), transport)
    }
}
