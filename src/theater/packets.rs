use super::{DecodeError, FilmPacket};
use crate::clients::hi::models::FilmChunkData;
use std::collections::BTreeSet;

pub(super) fn index(chunks: &[FilmChunkData]) -> Result<Vec<FilmPacket>, DecodeError> {
    let mut ids = BTreeSet::new();
    for c in chunks {
        if !ids.insert(c.metadata.index) {
            return Err(DecodeError::DuplicateChunk(c.metadata.index));
        }
    }
    let mut chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.metadata.chunk_type == 2)
        .collect();
    chunks.sort_by_key(|c| c.metadata.index);
    let mut result = Vec::new();
    for c in chunks {
        let mut offset = 0usize;
        while offset < c.data.len() {
            let bad = || DecodeError::Truncated {
                chunk: c.metadata.index,
                offset,
            };
            let h = c
                .data
                .get(offset..offset.saturating_add(16))
                .ok_or_else(bad)?;
            let size = u32::from_le_bytes([h[4], h[5], h[6], h[7]]) as usize;
            let stamp = u64::from_le_bytes([h[8], h[9], h[10], h[11], h[12], h[13], h[14], h[15]]);
            let payload = offset.checked_add(16).ok_or_else(bad)?;
            let end = payload.checked_add(size).ok_or_else(bad)?;
            if end > c.data.len() {
                return Err(bad());
            }
            result.push(FilmPacket {
                chunk_index: c.metadata.index,
                packet_type: u16::from_le_bytes([h[0], h[1]]),
                byte_2: h[2],
                byte_3: h[3],
                payload_offset: payload,
                payload_size: size,
                timestamp_us: stamp,
            });
            offset = end;
        }
    }
    if result.is_empty() {
        return Err(DecodeError::Missing("replication packets"));
    }
    Ok(result)
}
