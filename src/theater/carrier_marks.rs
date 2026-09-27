//! Keyframe carrier-marker observations; these corroborate flag carry evidence.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CarrierMark {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub slot: u32,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CarrierMarkScan {
    pub marks: Vec<CarrierMark>,
    #[serde(rename = "KeyframeUS")]
    pub keyframe_us: Vec<u64>,
    pub records: usize,
    pub biped_records: usize,
}
impl CarrierMarkScan {
    /// Attribute a matching window to the record containing its first bit, and
    /// emit at most one mark per record even when several shifted views match.
    pub fn append_keyframe(&mut self, payload: &[u8], timestamp_us: u64) {
        self.keyframe_us.push(timestamp_us);
        let records = recover_keyframe_anchors(payload);
        self.records += records.len();
        self.biped_records += records.iter().filter(|r| r.archetype == 35).count();
        let known: BTreeSet<_> = [0x00010005, 0x0002000b, 0x00040017, 0x0008002f]
            .into_iter()
            .collect();
        self.marks.extend(
            keyframe_loadouts::families_by_record(payload, &known, 35)
                .into_iter()
                .map(|(slot, _)| CarrierMark { timestamp_us, slot }),
        );
    }
}
pub fn scan_carrier_marks(chunks: &[FilmChunkData]) -> Result<CarrierMarkScan, DecodeError> {
    let mut out = CarrierMarkScan::default();
    for chunk in fire_events::native_chunk_prefix(chunks)? {
        for p in fire_events::native_chunk_packets(chunk) {
            if p.packet_type == 2 {
                out.append_keyframe(
                    &chunk.data[p.payload_offset..p.payload_offset + p.payload_size],
                    p.timestamp_us,
                );
            }
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCarrierMarkScan {
    pub scan: CarrierMarkScan,
    /// Ordered keyframe packets, including packets that produced no marks.
    pub keyframes: Vec<FilmPacket>,
}
/// Native ScanCarrierMarks over loaded buffers. A readable empty chunk succeeds;
/// absent readable chunks are a refusal, not a successfully scanned empty film.
pub fn scan_source_carrier_marks(
    source: Option<&FilmSource>,
) -> Result<SourceCarrierMarkScan, SourceCarrierMarkError> {
    let mut out = SourceCarrierMarkScan {
        scan: CarrierMarkScan::default(),
        keyframes: Vec::new(),
    };
    let source = source.ok_or(SourceCarrierMarkError::NoReadableChunk)?;
    let mut read = false;
    for number in source.data_chunk_numbers() {
        let Some((data, packets)) = source.chunk_by_number(number) else {
            continue;
        };
        read = true;
        for &packet in packets {
            if packet.packet_type != 2 {
                continue;
            }
            out.keyframes.push(packet);
            out.scan.append_keyframe(
                &data[packet.payload_offset..packet.payload_offset + packet.payload_size],
                packet.timestamp_us,
            );
        }
    }
    if !read {
        return Err(SourceCarrierMarkError::NoReadableChunk);
    }
    Ok(out)
}
impl From<&CarrierMarkScan> for FactsCarrierMarkScan {
    fn from(v: &CarrierMarkScan) -> Self {
        Self {
            marks: (!v.marks.is_empty()).then(|| v.marks.clone()),
            keyframe_us: (!v.keyframe_us.is_empty()).then(|| v.keyframe_us.clone()),
            records: v.records as i64,
            biped_records: v.biped_records as i64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SourceCarrierMarkError {
    #[error("aucun chunk film lisible")]
    NoReadableChunk,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_carrier_marker_attribution() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/carrier-marks-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let mut actual = CarrierMarkScan::default();
            for input in r["inputs"].as_array().unwrap() {
                let hex = input["hex"].as_str().unwrap();
                let data: Vec<_> = (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                    .collect();
                actual.append_keyframe(&data, input["timestamp_us"].as_u64().unwrap());
            }
            assert_eq!(
                actual,
                serde_json::from_value::<CarrierMarkScan>(r["expected"].clone()).unwrap(),
                "carrier case {i}"
            );
        }
        assert!(scan_carrier_marks(&[]).is_err());
    }
}
