//! Signature-guarded biped creation observations; not complete NEW records.
use super::{DecodeError, FilmPacket, bits::Bits};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const BIPED_REPRESENTATION_NAME: u32 = 0x1876_bda0;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedCreation {
    pub start_bit: usize,
    /// End of the participant prologue, not the full creation record.
    pub prologue_end_bit: usize,
    pub slot: u32,
    pub generation: u8,
    pub participant_index: u8,
    pub version: u8,
    pub representation: u32,
}
impl BipedCreation {
    pub fn life_key(&self) -> u32 {
        self.slot | (u32::from(self.generation) << 16)
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedCreationStats {
    /// Selected slot cardinality. Standalone payload scans do not set this.
    #[serde(default)]
    pub slots: usize,
    /// Native prologue overflow counter. The current scan loop admits only
    /// complete prologues, so this remains zero for that entry point.
    #[serde(default)]
    pub truncated: usize,
    pub anchors: usize,
    pub accepted: usize,
    pub shape_bad: usize,
    pub signature_mismatch: usize,
    pub gate_closed: usize,
    /// Full histogram retained; the reference exports only its most common entry.
    pub other_representations: BTreeMap<u32, usize>,
}
impl BipedCreationStats {
    pub fn most_common_other(&self) -> Option<(u32, usize)> {
        self.other_representations
            .iter()
            .map(|(&k, &n)| (k, n))
            .min_by_key(|&(k, n)| (std::cmp::Reverse(n), k))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmBipedCreation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub source: FilmPacket,
    pub creation: BipedCreation,
}

/// The reference's 24-bit NEW anchor plus 48-bit participant prologue. A missing
/// participant is rejected, never represented as index zero. `stats` accumulates
/// across calls, including alternate representation names.
pub fn scan_biped_creation_records(
    data: &[u8],
    band: [u32; 2],
    stats: &mut BipedCreationStats,
) -> Result<Vec<BipedCreation>, DecodeError> {
    if band[0] > band[1] || band[1] >= 8192 {
        return Err(DecodeError::Inconsistent(
            "invalid biped creation slot band".into(),
        ));
    }
    Ok(scan_biped_creation_records_matching(
        data,
        |slot| slot >= band[0] && slot <= band[1],
        stats,
    ))
}

pub(super) fn scan_biped_creation_records_matching(
    data: &[u8],
    contains: impl Fn(u32) -> bool,
    stats: &mut BipedCreationStats,
) -> Vec<BipedCreation> {
    let bits = Bits(data);
    let mut at = 0usize;
    let mut out = Vec::new();
    while at.saturating_add(72) <= bits.len() {
        let slot = bits.read(at + 3, 13).unwrap() as u32;
        if bits.read(at, 3) != Some(1) || bits.read(at + 18, 6) != Some(35) || !contains(slot) {
            at += 1;
            continue;
        }
        stats.anchors += 1;
        let start = at + 24;
        if bits.read(start, 1) != Some(1)
            || bits.read(start + 1, 8) != Some(13)
            || bits.read(start + 9, 1) != Some(1)
        {
            stats.shape_bad += 1;
            at += 1;
            continue;
        }
        let representation = bits.read(start + 10, 32).unwrap() as u32;
        if representation != BIPED_REPRESENTATION_NAME {
            stats.signature_mismatch += 1;
            *stats
                .other_representations
                .entry(representation)
                .or_default() += 1;
            at += 1;
            continue;
        }
        if bits.read(start + 42, 1) != Some(0) {
            stats.gate_closed += 1;
            at += 1;
            continue;
        }
        out.push(BipedCreation {
            start_bit: at,
            prologue_end_bit: at + 72,
            slot,
            generation: bits.read(at + 16, 2).unwrap() as u8,
            participant_index: bits.read(start + 43, 5).unwrap() as u8,
            version: 13,
            representation,
        });
        stats.accepted += 1;
        at += 72;
    }
    out
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedCreationStream {
    /// Exact selected membership; slot_band below is only its bounding range.
    #[serde(default)]
    pub slots: Vec<u32>,
    pub slot_band: Option<[u32; 2]>,
    pub stats: BipedCreationStats,
    pub records: Vec<FilmBipedCreation>,
}
/// Scan replication packets with a previously recovered biped slot band.
pub fn scan_biped_creations_for_band(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    band: Option<[u32; 2]>,
) -> Result<BipedCreationStream, DecodeError> {
    let Some(band) = band else {
        return Ok(BipedCreationStream::default());
    };
    if band[0] > band[1] || band[1] >= 8192 {
        return Err(DecodeError::Inconsistent(
            "invalid biped creation slot band".into(),
        ));
    }
    scan_biped_creations_for_slots(chunks, &super::FilmSlotBand::from_slots(band[0]..=band[1])?)
}

/// Native selected-prefix scanner, including sparse slot membership. Source
/// errors precede empty-band errors, matching ScanBipedCreationsForBand.
pub fn scan_biped_creations_for_slots(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    band: &super::FilmSlotBand,
) -> Result<BipedCreationStream, DecodeError> {
    let selected = super::fire_events::native_chunk_prefix(chunks)?;
    if band.is_empty() {
        return Err(DecodeError::Missing("biped creation slot band"));
    }
    let slots = band.slots();
    let mut out = BipedCreationStream {
        slot_band: Some([slots[0], *slots.last().unwrap()]),
        slots,
        stats: BipedCreationStats {
            slots: band.count(),
            ..Default::default()
        },
        records: Vec::new(),
    };
    for chunk in selected {
        for (packet_index, source) in super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .enumerate()
        {
            if source.packet_type != 0 {
                continue;
            }
            let payload =
                &chunk.data[source.payload_offset..source.payload_offset + source.payload_size];
            for creation in scan_biped_creation_records_matching(
                payload,
                |slot| band.contains(slot),
                &mut out.stats,
            ) {
                out.records.push(FilmBipedCreation {
                    source,
                    creation,
                    packet_index: Some(packet_index),
                });
            }
        }
    }
    Ok(out)
}

/// Recover the ti35 band and scan creation prologues without requiring map bounds.
pub fn scan_biped_creations(
    chunks: &[crate::clients::hi::models::FilmChunkData],
) -> Result<BipedCreationStream, DecodeError> {
    super::fire_events::native_chunk_prefix(chunks)?;
    let band = super::biped_scan::biped_slot_band(chunks)?
        .ok_or(DecodeError::Missing("biped creation slot band"))?;
    scan_biped_creations_for_band(chunks, Some(band))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn creation_prologue_and_rejection_counters_match_go() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-scan-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut json)
        .unwrap();
        let oracle: serde_json::Value = serde_json::from_str(&json).unwrap();
        let cases = oracle["creations"].as_array().unwrap();
        assert_eq!(cases.len(), 2048);
        for row in cases {
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let mut stats = BipedCreationStats::default();
            let records = scan_biped_creation_records(&bytes, [512, 514], &mut stats).unwrap();
            let expected = row["records"].as_array().cloned().unwrap_or_default();
            assert_eq!(records.len(), expected.len());
            for (a, e) in records.iter().zip(&expected) {
                assert_eq!(
                    serde_json::json!({"Slot":a.slot,"Generation":a.generation,"ParticipantIndex":a.participant_index,"HasIndex":true,"Chunk":0,"PacketIndex":0,"TimestampUS":0,"BitPos":a.start_bit,"Version":a.version,"Representation":a.representation}),
                    *e
                );
                assert_eq!(a.prologue_end_bit, a.start_bit + 72);
            }
            let e = &row["stats"];
            for (key, value) in [
                ("Slots", stats.slots),
                ("Truncated", stats.truncated),
                ("Anchors", stats.anchors),
                ("Accepted", stats.accepted),
                ("ShapeBad", stats.shape_bad),
                ("SignatureMismatch", stats.signature_mismatch),
                ("GateClosed", stats.gate_closed),
            ] {
                assert_eq!(value as u64, e[key].as_u64().unwrap(), "{key}");
            }
            let (word, count) = stats.most_common_other().unwrap_or((0, 0));
            assert_eq!(serde_json::json!(word), e["OtherWord"]);
            assert_eq!(serde_json::json!(count), e["OtherWordCount"]);
            assert_eq!(e["Truncated"], 0); // Native loop requires the entire prologue before anchoring.
            let data = super::super::fire_events::test_payload_after_keyframe(&bytes, 0);
            let chunk = crate::clients::hi::models::FilmChunkData {
                metadata: crate::clients::hi::models::FilmChunk {
                    index: 1,
                    chunk_type: 2,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            };
            let stream = scan_biped_creations_for_band(&[chunk], Some([512, 514])).unwrap();
            assert_eq!(stream.stats.slots, 3);
            let mut payload_stats = stream.stats.clone();
            payload_stats.slots = 0;
            assert_eq!(payload_stats, stats);
            assert_eq!(
                stream
                    .records
                    .iter()
                    .map(|r| &r.creation)
                    .collect::<Vec<_>>(),
                records.iter().collect::<Vec<_>>()
            );
            let restored: BipedCreationStream =
                serde_json::from_value(serde_json::to_value(&stream).unwrap()).unwrap();
            assert_eq!(restored, stream);
        }
    }
    #[test]
    fn native_creation_source_and_sparse_bands() {
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/creation-source-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 512);
        let mut published = 0;
        for (i, case) in cases.iter().enumerate() {
            let slots: Vec<u32> = serde_json::from_value(case["slots"].clone()).unwrap();
            let band = super::super::FilmSlotBand::from_slots(slots.clone()).unwrap();
            let chunks: Vec<_> = case["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let hex = c["hex"].as_str().unwrap();
                    let data: Vec<u8> = (0..hex.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 2,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            for (result, case) in [
                (scan_biped_creations_for_slots(&chunks, &band), case),
                (scan_biped_creations(&chunks), &case["automatic"]),
            ] {
                let error = match &result {
                    Ok(_) => "",
                    Err(DecodeError::Missing("readable film chunks")) => "source",
                    Err(DecodeError::Missing("biped creation slot band")) => "band",
                    Err(e) => panic!("case {i}: {e}"),
                };
                assert_eq!(error, case["error"], "case {i}");
                let out = result.unwrap_or_default();
                if error.is_empty() {
                    assert_eq!(serde_json::json!(out.slots), case["slots"]);
                }
                let stats = &out.stats;
                let (word, count) = stats.most_common_other().unwrap_or((0, 0));
                assert_eq!(
                    serde_json::json!({"Slots":stats.slots,"Anchors":stats.anchors,"Truncated":stats.truncated,"ShapeBad":stats.shape_bad,"SignatureMismatch":stats.signature_mismatch,"OtherWord":word,"OtherWordCount":count,"GateClosed":stats.gate_closed,"Accepted":stats.accepted}),
                    case["stats"],
                    "stats {i}"
                );
                let records: Vec<_> = out.records.iter().map(|r| {
                let c = &r.creation;
                assert!(out.slots.contains(&c.slot));
                assert_eq!(c.prologue_end_bit, c.start_bit + 72);
                serde_json::json!({"Slot":c.slot,"Generation":c.generation,"ParticipantIndex":c.participant_index,"HasIndex":true,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.unwrap(),"TimestampUS":r.source.timestamp_us,"BitPos":c.start_bit,"Version":c.version,"Representation":c.representation})
            }).collect();
                assert_eq!(
                    records,
                    case["records"].as_array().cloned().unwrap_or_default(),
                    "records {i}"
                );
                published += records.len();
                let restored: BipedCreationStream =
                    serde_json::from_value(serde_json::to_value(&out).unwrap()).unwrap();
                assert_eq!(restored, out);
            }
        }
        assert_eq!(published, 1220);
    }
}
