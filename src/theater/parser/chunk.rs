//! Validated chunk kinds, strict inflation, and packet framing during film parsing.
use crate::clients::hi::models::{RawChunkKind, RawFilmChunk};
use crate::theater::film::ParseError;
use crate::theater::parser::packet::{
    ByteRange, FilmPacketHeader, Packet, PacketBody, PacketRead, PacketSource, PacketStream,
};
use flate2::{Decompress, FlushDecompress, Status};
use serde::{Deserialize, Serialize};

/// Recognized film chunk category after input validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkKind {
    /// Type 1: bootstrap data and the entity-component registry.
    Registry,
    /// Type 2: ordered replay packets and replication state changes.
    Replication,
    /// Type 3: terminal summary/highlights packets.
    Summary,
}

impl From<ChunkKind> for RawChunkKind {
    fn from(value: ChunkKind) -> Self {
        match value {
            ChunkKind::Registry => Self::REGISTRY,
            ChunkKind::Replication => Self::REPLICATION,
            ChunkKind::Summary => Self::SUMMARY,
        }
    }
}

/// One chunk after inflation and before kind-specific parsing.
/// Its downloaded byte buffer is no longer retained alongside `data`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InflatedFilmChunk {
    /// Recognized chunk category established from the raw manifest code.
    pub kind: ChunkKind,
    /// Manifest index carried from the raw chunk.
    pub index: usize,
    /// Manifest start-time offset in milliseconds.
    pub start_ms: i64,
    /// Inflated chunk bytes used by the section reader.
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InflateError {
    #[error("invalid or incomplete zlib header")]
    Header,
    #[error("damaged or incomplete zlib stream")]
    Stream,
    #[error("decompressed chunk exceeds its output limit")]
    Limit,
}
impl TryFrom<RawChunkKind> for ChunkKind {
    type Error = ParseError;
    fn try_from(value: RawChunkKind) -> Result<Self, Self::Error> {
        match value.raw() {
            1 => Ok(Self::Registry),
            2 => Ok(Self::Replication),
            3 => Ok(Self::Summary),
            other => Err(ParseError::UnsupportedChunkKind(other)),
        }
    }
}

/// Maximum inflated bytes per chunk, matching pinned the reference reader's ceiling.
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
    let mut decoder = Decompress::new(false);
    let mut out = Vec::with_capacity(raw.len().saturating_mul(6).min(64 << 20).min(limit));
    let result = loop {
        let before = (decoder.total_in(), decoder.total_out());
        let mut block = [0; 8192];
        let remaining = limit - out.len();
        // One scratch byte beyond the limit distinguishes exact-limit StreamEnd
        // from real overflow, including when the checksum needs another call.
        let scratch_len = block.len().min(remaining.saturating_add(1));
        let status = decoder.decompress(
            &raw[header + before.0 as usize..],
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
                    Err(InflateError::Stream)
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

impl InflatedFilmChunk {
    /// Frame packet envelopes once, retaining any malformed trailing bytes.
    pub(crate) fn read_packets<T: PacketBody>(
        &self,
        max_packets: usize,
        mut decode: impl FnMut(usize, FilmPacketHeader, &[u8]) -> PacketRead<T>,
    ) -> Result<PacketStream<T>, ParseError> {
        let mut packets = Vec::new();
        let mut offset = 0usize;
        while offset < self.data.len() {
            if packets.len() == max_packets {
                return Err(ParseError::PacketLimit {
                    index: self.index,
                    limit: max_packets,
                });
            }
            let Some(bytes) = self.data.get(offset..offset.saturating_add(16)) else {
                break;
            };
            let size = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
            let start = offset + 16;
            let Some(end) = start
                .checked_add(size as usize)
                .filter(|end| *end <= self.data.len())
            else {
                break;
            };
            let source = PacketSource {
                header: ByteRange {
                    start: offset,
                    end: start,
                },
                payload: ByteRange { start, end },
            };
            let header = FilmPacketHeader {
                packet_type: u16::from_le_bytes(bytes[..2].try_into().unwrap()),
                unknown_2: [bytes[2], bytes[3]],
                payload_size: size,
                timestamp_us: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            };
            let body = decode(packets.len(), header, &self.data[start..end]);
            let packet = Packet {
                source,
                header,
                body,
            };
            debug_assert!(packet.payload(&self.data).is_some());
            packets.push(packet);
            offset = end;
        }
        let opaque = if offset < self.data.len() {
            tracing::warn!(kind = ?self.kind, chunk_index = self.index,
                byte_offset = offset, remaining_bytes = self.data.len() - offset,
                "stopped parsing film chunk before the end");
            vec![ByteRange {
                start: offset,
                end: self.data.len(),
            }]
        } else {
            Vec::new()
        };
        Ok(PacketStream { packets, opaque })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 1950 stored block with independently computed Adler checksum.
    fn stored(payload: &[u8], window: u8) -> Vec<u8> {
        let cmf = (window << 4) | 8;
        let flg = (31 - ((u16::from(cmf) << 8) % 31)) % 31;
        let length = u16::try_from(payload.len()).unwrap();
        let mut wire = vec![cmf, flg as u8, 1];
        wire.extend(length.to_le_bytes());
        wire.extend((!length).to_le_bytes());
        wire.extend(payload);
        let (mut a, mut b) = (1u32, 0u32);
        for &byte in payload {
            a = (a + u32::from(byte)) % 65521;
            b = (b + a) % 65521;
        }
        wire.extend((b << 16 | a).to_be_bytes());
        wire
    }

    fn raw(data: Vec<u8>) -> RawFilmChunk {
        RawFilmChunk {
            kind: crate::theater::film::ChunkKind::Replication.into(),
            index: 3,
            start_ms: 42,
            data,
        }
    }

    #[test]
    fn all_rfc_windows_and_limits_return_one_owned_buffer() {
        for window in 0..=7 {
            for length in [0usize, 1, 8191, 8192, 8193, 16384] {
                let payload: Vec<_> = (0..length).map(|i| (i * 37) as u8).collect();
                let wire = stored(&payload, window);
                let inflated = raw(wire.clone()).inflate().unwrap();
                assert_eq!(inflated.data, payload);
                assert_eq!(inflated.index, 3);
                assert_eq!(inflated.start_ms, 42);
                for limit in [0, 1, length.saturating_sub(1), length, length + 1] {
                    let result = raw(wire.clone()).inflate_with_limit(limit);
                    if limit < length {
                        assert_eq!(result, Err(InflateError::Limit));
                    } else {
                        assert_eq!(result.unwrap(), payload);
                    }
                }
            }
        }
    }

    #[test]
    fn malformed_zlib_is_an_error_instead_of_partial_output() {
        let wire = stored(b"film chunk", 7);
        for cut in 2..wire.len() {
            assert!(raw(wire[..cut].to_vec()).inflate().is_err());
        }
        let mut damaged = wire;
        *damaged.last_mut().unwrap() ^= 1;
        assert!(matches!(
            raw(damaged).inflate(),
            Err(ParseError::Inflate {
                index: 3,
                cause: InflateError::Stream
            })
        ));
        assert!(matches!(
            raw(b"\x78\x00bad".to_vec()).inflate(),
            Err(ParseError::Inflate {
                index: 3,
                cause: InflateError::Header
            })
        ));
    }

    #[test]
    fn uncompressed_input_is_rejected() {
        assert!(matches!(
            raw(vec![1, 2, 3, 4]).inflate(),
            Err(ParseError::Inflate {
                index: 3,
                cause: InflateError::Header
            })
        ));
        assert!(matches!(
            raw(Vec::new()).inflate(),
            Err(ParseError::Inflate {
                index: 3,
                cause: InflateError::Header
            })
        ));
    }
}
