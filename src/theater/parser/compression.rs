//! Strict zlib inflation for downloaded film chunks.
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::film::{InflatedFilmChunk, ParseError};
use flate2::{Decompress, FlushDecompress, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InflateError {
    #[error("invalid or incomplete zlib header")]
    Header,
    #[error("damaged or incomplete zlib stream")]
    Stream,
    #[error("decompressed chunk exceeds its output limit")]
    Limit,
}

/// Maximum inflated bytes per chunk, matching the pinned reference reader's ceiling.
/// This limits output, not merely the initial allocation.
const MAX_CHUNK_BYTES: usize = 256 << 20;

fn is_zlib_header(raw: &[u8]) -> bool {
    raw.get(..2).is_some_and(|h| {
        h[0] & 15 == 8 && h[0] >> 4 <= 7 && u16::from_be_bytes([h[0], h[1]]) % 31 == 0
    })
}

fn zlib_header(raw: &[u8]) -> Result<usize, InflateError> {
    if !is_zlib_header(raw) {
        return Err(InflateError::Header);
    }
    if raw[1] & 32 != 0 {
        // Reference NewReader uses a nil dictionary, whose Adler checksum is one.
        if raw.get(2..6) != Some(&[0, 0, 0, 1]) {
            return Err(InflateError::Header);
        }
        Ok(6)
    } else {
        Ok(2)
    }
}

fn inflate_body(raw: &[u8], header: usize, limit: usize) -> Result<Vec<u8>, InflateError> {
    // Let the backend validate the ordinary zlib header and Adler checksum.
    // A preset empty dictionary still follows the established raw-deflate path.
    let dictionary = header == 6;
    let input_start = if dictionary { header } else { 0 };
    let mut decoder = Decompress::new(!dictionary);
    let mut out = Vec::with_capacity(raw.len().saturating_mul(6).min(64 << 20).min(limit));
    let result = loop {
        let before = (decoder.total_in(), decoder.total_out());
        let mut block = [0; 8192];
        let remaining = limit - out.len();
        // One scratch byte beyond the limit distinguishes exact-limit StreamEnd
        // from real overflow, including when the checksum needs another call.
        let scratch_len = block.len().min(remaining.saturating_add(1));
        let status = decoder.decompress(
            &raw[input_start + before.0 as usize..],
            &mut block[..scratch_len],
            FlushDecompress::None,
        );
        let produced = (decoder.total_out() - before.1) as usize;
        out.extend_from_slice(&block[..produced.min(remaining)]);
        if produced > remaining {
            break Err(InflateError::Limit);
        }
        match status {
            Ok(Status::StreamEnd) => {
                break if dictionary {
                    let end = header + decoder.total_in() as usize;
                    let checksum = raw.get(end..end.saturating_add(4));
                    let (mut a, mut b) = (1u32, 0u32);
                    for &byte in &out {
                        a = (a + u32::from(byte)) % 65521;
                        b = (b + a) % 65521;
                    }
                    if checksum == Some(&(b << 16 | a).to_be_bytes()) {
                        Ok(())
                    } else {
                        Err(InflateError::Stream)
                    }
                } else {
                    Ok(())
                };
            }
            Err(_) => break Err(InflateError::Stream),
            _ if before == (decoder.total_in(), decoder.total_out()) => {
                break Err(InflateError::Stream);
            }
            _ => {}
        }
    };
    result.map(|()| out)
}

impl RawFilmChunk {
    /// Consume a downloaded chunk, retaining only its inflated bytes.
    /// Inflation failures include the chunk's manifest index in [`ParseError`].
    pub fn inflate(self) -> Result<InflatedFilmChunk, ParseError> {
        let index =
            usize::try_from(self.index).map_err(|_| ParseError::InvalidIndex(self.index))?;
        let kind = self.kind.try_into()?;
        let data = self
            .inflate_with_limit(MAX_CHUNK_BYTES)
            .map_err(|cause| ParseError::Inflate { index, cause })?;
        Ok(InflatedFilmChunk {
            kind,
            index,
            start_ms: self.start_ms,
            data,
        })
    }

    fn inflate_with_limit(&self, limit: usize) -> Result<Vec<u8>, InflateError> {
        let header = zlib_header(&self.data)?;
        inflate_body(&self.data, header, limit)
    }
}

#[cfg(test)]
mod tests {
    use super::{InflateError, inflate_body, zlib_header};
    use flate2::{Compression, write::ZlibEncoder};
    use std::io::Write;

    fn zlib(input: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(input).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn ordinary_zlib_validates_checksum_and_output_limit() {
        let data = b"theater film chunk".repeat(64);
        let compressed = zlib(&data);
        let header = zlib_header(&compressed).unwrap();
        assert_eq!(inflate_body(&compressed, header, data.len()).unwrap(), data);
        assert_eq!(
            inflate_body(&compressed, header, data.len() - 1),
            Err(InflateError::Limit)
        );

        let mut damaged = compressed.clone();
        *damaged.last_mut().unwrap() ^= 1;
        assert_eq!(
            inflate_body(&damaged, header, data.len()),
            Err(InflateError::Stream)
        );
        assert_eq!(
            inflate_body(&compressed[..compressed.len() - 1], header, data.len()),
            Err(InflateError::Stream)
        );
    }

    #[test]
    fn empty_preset_dictionary_header_preserves_raw_deflate_path() {
        let data = b"dictionary film chunk".repeat(32);
        let compressed = zlib(&data);
        let mut with_dictionary = vec![0x78, 0x20, 0, 0, 0, 1];
        with_dictionary.extend_from_slice(&compressed[2..]);
        let header = zlib_header(&with_dictionary).unwrap();
        assert_eq!(header, 6);
        assert_eq!(
            inflate_body(&with_dictionary, header, data.len()).unwrap(),
            data
        );

        *with_dictionary.last_mut().unwrap() ^= 1;
        assert_eq!(
            inflate_body(&with_dictionary, header, data.len()),
            Err(InflateError::Stream)
        );
        with_dictionary[5] = 2;
        assert_eq!(zlib_header(&with_dictionary), Err(InflateError::Header));
    }
}
