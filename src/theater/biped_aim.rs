//! Native aim-only biped observations, including occupants without world positions.
use super::{DecodeError, biped_capture::scan_companions_at, bits::Bits, fire_events};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BipedAim {
    pub slot: u32,
    pub chunk: i64,
    pub packet_index: usize,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub yaw_raw: u32,
    pub pitch_raw: u32,
}

impl BipedAim {
    /// Native midpoint conversion of the recorded 12-bit yaw quantum.
    pub fn aim_heading_degrees(&self) -> f32 {
        aim_heading_from_raw(self.yaw_raw)
    }

    /// Native measured pitch convention. Values outside the observed central
    /// half of the 11-bit range retain the reference's unverified extrapolation.
    pub fn aim_pitch_degrees(&self) -> f32 {
        aim_pitch_from_raw(self.pitch_raw)
    }
}

pub(super) fn aim_heading_from_raw(raw: u32) -> f32 {
    (360.0 * (f64::from(raw) + 0.5) / 4096.0) as f32
}

pub(super) fn aim_pitch_from_raw(raw: u32) -> f32 {
    (360.0 * (f64::from(raw) + 0.5) / 2048.0 - 180.0) as f32
}

/// Scan the native sparse, ascending masks without i0. Only generation one and
/// modeled components before i21 are accepted. Successful reads skip exactly the
/// primary aim fields, matching the native overlap rule even with optional tails.
pub fn scan_biped_aim_records(payload: &[u8], band: &BTreeSet<u32>) -> Vec<BipedAim> {
    scan_biped_aim_records_with_sources(payload, band)
        .into_iter()
        .map(|r| r.aim)
        .collect()
}
#[derive(Debug, Clone, PartialEq)]
pub struct BipedAimRecord {
    pub bit: usize,
    pub primary_end_bit: usize,
    pub aim: BipedAim,
    /// Other supported fields traversed en route to primary aim.
    pub companions: super::BipedCompanions,
}
pub fn scan_biped_aim_records_with_sources(
    payload: &[u8],
    band: &BTreeSet<u32>,
) -> Vec<BipedAimRecord> {
    let bits = Bits(payload);
    let mut p = 0;
    let mut out = Vec::new();
    while p + 21 <= bits.len() {
        let found = (|| {
            if bits.read(p, 1)? != 1 || bits.read(p + 14, 2)? != 1 || bits.read(p + 16, 2)? != 0 {
                return None;
            }
            let slot = bits.read(p + 1, 13)? as u32;
            if !band.contains(&slot) {
                return None;
            }
            let count = bits.read(p + 18, 3)? as usize;
            if count == 0 {
                return None;
            }
            let mut mask = [0u8; 7];
            let mut prev = 0;
            for (i, v) in mask.iter_mut().enumerate().take(count) {
                *v = bits.read(p + 21 + i * 6, 6)? as u8;
                if *v <= prev {
                    return None;
                }
                prev = *v;
            }
            let (companions, end) =
                scan_companions_at(payload, p + 21 + count * 6, &mask[..count], None, false);
            let [yaw_raw, pitch_raw] = companions.aim?;
            Some((
                BipedAim {
                    slot,
                    yaw_raw,
                    pitch_raw,
                    ..Default::default()
                },
                end?,
                companions,
            ))
        })();
        if let Some((aim, end, companions)) = found {
            out.push(BipedAimRecord {
                bit: p,
                primary_end_bit: end,
                aim,
                companions,
            });
            p = end;
        } else {
            p += 1;
        }
    }
    out
}

/// Scan loaded film chunks using the native biped census and publication order.
pub fn scan_film_biped_aim(chunks: &[FilmChunkData]) -> Result<Vec<BipedAim>, DecodeError> {
    if fire_events::native_chunk_prefix(chunks).is_err() {
        return Err(DecodeError::Missing("film data chunks"));
    }
    let band = super::biped_scan::biped_slot_band(chunks)?;
    let slots = band.map(|[lo, hi]| (lo..=hi).collect()).unwrap_or_default();
    scan_biped_aim_for_band(chunks, &slots)
}

pub fn scan_biped_aim_for_band(
    chunks: &[FilmChunkData],
    band: &BTreeSet<u32>,
) -> Result<Vec<BipedAim>, DecodeError> {
    let chunks = fire_events::native_chunk_prefix(chunks)?;
    if band.is_empty() {
        return Err(DecodeError::Missing("biped slot band"));
    }
    let mut out = Vec::new();
    for c in chunks {
        for (index, p) in fire_events::native_chunk_packets(c).into_iter().enumerate() {
            if p.packet_type != 0 {
                continue;
            }
            for mut aim in scan_biped_aim_records(
                &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                band,
            ) {
                aim.chunk = i64::from(c.metadata.index);
                aim.packet_index = index;
                aim.timestamp_us = p.timestamp_us;
                out.push(aim);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        payload: Vec<u8>,
        band: BTreeSet<u32>,
        aims: Vec<BipedAim>,
        headings: [f32; 2],
        pitch: f32,
    }
    #[test]
    fn native_loaded_aim_scan() {
        use crate::clients::hi::models::FilmChunk;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-aim-loaded-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 256);
        for (i, row) in cases.iter().enumerate() {
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let hex = c["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|k| u8::from_str_radix(&hex[k..k + 2], 16).unwrap())
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
            let band = super::super::biped_scan::biped_slot_band(&chunks).unwrap();
            let slots: Vec<u32> = band.map(|[lo, hi]| (lo..=hi).collect()).unwrap_or_default();
            assert_eq!(serde_json::json!(slots), row["slots"], "slots {i}");
            let result = scan_film_biped_aim(&chunks);
            let error = match &result {
                Ok(_) => "",
                Err(DecodeError::Missing("film data chunks")) => "chunks",
                Err(DecodeError::Missing("biped slot band")) => "band",
                Err(DecodeError::Missing("readable film chunks")) => "readable",
                Err(e) => panic!("case {i}: {e}"),
            };
            assert_eq!(error, row["error"].as_str().unwrap(), "error {i}");
            if let Ok(aims) = result {
                assert_eq!(
                    serde_json::to_value(&aims).unwrap(),
                    row["aims"],
                    "aims {i}"
                );
                assert_eq!(
                    serde_json::from_value::<Vec<BipedAim>>(serde_json::to_value(&aims).unwrap())
                        .unwrap(),
                    aims
                );
            }
        }
    }
    #[test]
    fn native_aim_only_scan() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/biped-aim-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            for (j, expected) in c.headings.into_iter().enumerate() {
                let aim = BipedAim {
                    yaw_raw: (i + j * 2048) as u32,
                    pitch_raw: i as u32,
                    ..Default::default()
                };
                assert_eq!(aim.aim_heading_degrees(), expected, "heading {i}/{j}");
                assert_eq!(aim.aim_pitch_degrees(), c.pitch, "pitch {i}");
            }
            assert_eq!(
                scan_biped_aim_records(&c.payload, &c.band),
                c.aims,
                "case {i}"
            );
        }
    }
}
