//! Native padded zoom-head decoding, separate from bounded gameplay observations.
use super::{DecodedHeadEvent, FilmPacket, bits::Cursor, event_heads, fire_events};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeZoomRead {
    pub head: DecodedHeadEvent,
    pub source_bits: usize,
    pub padded_bits: usize,
    /// Native domain-four slot; check unit_reference_recorded before treating
    /// every reference bit as recorded rather than padded.
    pub slot: u32,
    /// Native level; check level_recorded before interpreting it as recorded.
    pub level: u8,
}
impl NativeZoomRead {
    pub fn unit_reference_recorded(&self) -> bool {
        self.head
            .references
            .first()
            .is_some_and(|r| r.end_bit <= self.source_bits)
    }
    pub fn level_recorded(&self) -> bool {
        self.head.end_bit <= self.source_bits
    }
}

/// Native decodeZoomHead semantics. The head's references and logical extent
/// retain provenance for the reader's synthetic zero tail. A missing first
/// reference still refuses publication; the config bit is a source-scanner gate.
pub fn decode_native_zoom_head(data: &[u8]) -> Option<NativeZoomRead> {
    let head = event_heads::decode_head_with_cursor(Cursor::new_padded(data, 0))?;
    if head.kind != 21 {
        return None;
    }
    let (slot, level) = head.zoom()?;
    Some(NativeZoomRead {
        source_bits: data.len() * 8,
        padded_bits: head.end_bit.saturating_sub(data.len() * 8),
        slot,
        level,
        head,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmNativeZoomEvent {
    pub source: FilmPacket,
    pub packet_index: usize,
    pub read: NativeZoomRead,
}

pub fn scan_native_zoom_events(chunks: &[FilmChunkData]) -> Vec<FilmNativeZoomEvent> {
    let mut out = Vec::new();
    for chunk in fire_events::native_chunk_prefix(chunks).unwrap_or_default() {
        for (packet_index, source) in fire_events::native_chunk_packets(chunk)
            .into_iter()
            .enumerate()
        {
            if source.packet_type != 0 || source.payload_size < 2 {
                continue;
            }
            let data =
                &chunk.data[source.payload_offset..source.payload_offset + source.payload_size];
            if data[0] != 0xca {
                continue;
            }
            if let Some(read) = decode_native_zoom_head(data) {
                out.push(FilmNativeZoomEvent {
                    source,
                    packet_index,
                    read,
                });
            }
        }
    }
    super::native_sort::sort_by(&mut out, |a, b| {
        a.source.timestamp_us.cmp(&b.source.timestamp_us)
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use std::io::Read;
    fn bytes(h: &str) -> Vec<u8> {
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect()
    }
    #[test]
    fn native_zoom_padding_and_source_scanner() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/zoom-padding-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let oracle: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let rows = oracle["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 2304);
        let mut incomplete_refs = 0;
        let mut padded_levels = 0;
        for (case, row) in rows.iter().enumerate() {
            let pay = bytes(row["hex"].as_str().unwrap());
            let read = decode_native_zoom_head(&pay);
            assert_eq!(read.is_some(), row["ok"].as_bool().unwrap(), "case {case}");
            if let Some(read) = read {
                assert_eq!(u64::from(read.slot), row["event"]["Slot"].as_u64().unwrap());
                assert_eq!(
                    u64::from(read.level),
                    row["event"]["Level"].as_u64().unwrap()
                );
                assert_eq!(read.head.end_bit as u64, row["end"].as_u64().unwrap());
                assert_eq!(read.source_bits, pay.len() * 8);
                assert_eq!(
                    read.padded_bits,
                    read.head.end_bit.saturating_sub(pay.len() * 8)
                );
                incomplete_refs += usize::from(!read.unit_reference_recorded());
                padded_levels += usize::from(!read.level_recorded());
                let restored: NativeZoomRead =
                    serde_json::from_slice(&serde_json::to_vec(&read).unwrap()).unwrap();
                assert_eq!(restored, read);
            }
        }
        assert!(incomplete_refs > 0 && padded_levels > 0);
        let data = bytes(oracle["source"].as_str().unwrap());
        let chunks = [FilmChunkData {
            metadata: FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 90000,
                duration_ms: 1,
                size: data.len() as i64,
                file_relative_path: String::new(),
            },
            data,
        }];
        let reads = scan_native_zoom_events(&chunks);
        let expected = oracle["output"].as_array().unwrap();
        assert_eq!(reads.len(), expected.len());
        for (read, e) in reads.iter().zip(expected) {
            assert_eq!(read.packet_index as u64, e["packet"].as_u64().unwrap());
            assert_eq!(
                read.source.payload_offset as u64,
                e["start"].as_u64().unwrap()
            );
            assert_eq!(read.source.payload_size as u64, e["size"].as_u64().unwrap());
            assert_eq!(
                read.source.timestamp_us,
                e["event"]["TimestampUS"].as_u64().unwrap()
            );
            assert_eq!(
                u64::from(read.read.slot),
                e["event"]["Slot"].as_u64().unwrap()
            );
            assert_eq!(
                u64::from(read.read.level),
                e["event"]["Level"].as_u64().unwrap()
            );
        }
        let restored: Vec<FilmNativeZoomEvent> =
            serde_json::from_slice(&serde_json::to_vec(&reads).unwrap()).unwrap();
        assert_eq!(restored, reads);
    }
}
