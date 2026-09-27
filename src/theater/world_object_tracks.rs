//! Native projectile/equipment/world-object position scans and mobile lifetimes.
//! A trajectory endpoint is the last replicated position, not an impact event.
use super::{
    DecodeError, FilmMapBounds,
    bits::Cursor,
    fire_events::{native_chunk_packets, native_chunk_prefix},
};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorldObjectSample {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub chunk: i64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub at_rest: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorldObjectTrack {
    pub slot: u32,
    #[serde(rename = "Gen")]
    pub generation: u32,
    pub pts: Vec<WorldObjectSample>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldObjectPositionRecord {
    pub slot: u32,
    pub generation: u32,
    pub sample: WorldObjectSample,
    pub bit: usize,
    pub padded_bits: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldObjectPadding {
    pub chunk: i64,
    pub packet_index: usize,
    pub record: WorldObjectPositionRecord,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldObjectTrackStream {
    pub band: BTreeSet<u32>,
    pub tracks: Vec<WorldObjectTrack>,
    pub padding: Vec<WorldObjectPadding>,
}
fn read(pay: &[u8], p: usize, n: usize) -> u64 {
    Cursor::new_padded(pay, p).read(n).unwrap_or(0)
}
/// Fill the observed archetype range, then remove every slot observed with another
/// archetype. Sparse keyframe observations alone miss short projectile lives.
pub fn world_object_slot_band(
    chunks: &[FilmChunkData],
    archetype: u32,
) -> Result<BTreeSet<u32>, DecodeError> {
    native_chunk_prefix(chunks)?;
    Ok(super::scan_world_object_keyframes(chunks, archetype).band)
}
/// Scan the reference dominant absolute-position path. Unsupported gates, foreign
/// regions and saturated axes are rejected. Native zero-tail reads are reported.
pub fn decode_world_object_positions(
    pay: &[u8],
    band: &BTreeSet<u32>,
    map: &FilmMapBounds,
) -> Vec<WorldObjectPositionRecord> {
    decode_world_object_positions_with_precision(
        pay,
        band,
        map,
        WorldObjectPrecision::from_map(map),
    )
}
/// Scan with the native descriptor, including zero-bit indices and 32-bit axes.
pub fn decode_world_object_positions_with_precision(
    pay: &[u8],
    band: &BTreeSet<u32>,
    map: &FilmMapBounds,
    precision: WorldObjectPrecision,
) -> Vec<WorldObjectPositionRecord> {
    let Some(pos_bits) = precision.bit_length() else {
        return Vec::new();
    };
    let Some(limit) = pay.len().saturating_mul(8).checked_sub(27 + pos_bits) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut p = 0;
    while p <= limit {
        let start = p;
        p += 1;
        if read(pay, start, 1) != 1 {
            continue;
        }
        let slot = read(pay, start + 1, 13) as u32;
        if !band.contains(&slot) || read(pay, start + 16, 2) != 0 {
            continue;
        }
        let count = read(pay, start + 18, 3) as usize;
        if count == 0 {
            continue;
        }
        let mut idx = Vec::new();
        for i in 0..count {
            idx.push(read(pay, start + 21 + 6 * i, 6));
        }
        if idx[0] != 0 || idx.windows(2).any(|a| a[0] >= a[1]) {
            continue;
        }
        let at = start + 21 + 6 * count;
        let Some((xyz, off)) = decode_world_object_position_with_precision(pay, at, map, precision)
        else {
            continue;
        };
        out.push(WorldObjectPositionRecord {
            slot,
            generation: read(pay, start + 14, 2) as u32,
            bit: start,
            padded_bits: off.saturating_sub(pay.len() * 8),
            sample: WorldObjectSample {
                timestamp_us: 0,
                chunk: 0,
                x: xyz[0],
                y: xyz[1],
                z: xyz[2],
                at_rest: idx.contains(&18),
            },
        });
        p += pos_bits;
    }
    out
}
/// Explicit native world-object descriptor, independent of coordinate bounds.
/// A zero-bit region index is meaningful here; catalog maps use at least one bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldObjectPrecision {
    pub index_bits: usize,
    pub axis_bits: [usize; 3],
    pub region: u32,
}
impl WorldObjectPrecision {
    pub fn from_map(map: &FilmMapBounds) -> Self {
        map.absolute_precision()
    }
    /// Native profiles contain one world region; use the same first-region rule
    /// as replay map-precision installation when adapting the general encoding.
    pub fn from_position(position: &super::PositionEncoding) -> Option<Self> {
        Some(Self {
            index_bits: position.index_bits,
            axis_bits: position.world_axis_bits?,
            region: position
                .region_axis_bits
                .keys()
                .next()
                .copied()
                .unwrap_or(0),
        })
    }
    fn valid(self) -> bool {
        self.index_bits <= 32 && self.axis_bits.iter().all(|w| (1..=32).contains(w))
    }
    pub fn bit_length(self) -> Option<usize> {
        self.valid()
            .then(|| 4 + self.index_bits + self.axis_bits.iter().sum::<usize>())
    }
}

/// Native selective world-position read, including zero-padded input tails.
/// Returns coordinates and the offset before the two unconsumed tail flags.
pub fn decode_world_object_position_with_precision(
    pay: &[u8],
    at: usize,
    bounds: &FilmMapBounds,
    precision: WorldObjectPrecision,
) -> Option<([f32; 3], usize)> {
    at.checked_add(precision.bit_length()?)?;
    let iw = precision.index_bits;
    if read(pay, at, 2) != 0 || read(pay, at + 2, iw) != precision.region as u64 {
        return None;
    }
    let mut off = at + 2 + iw;
    let mut xyz = [0.; 3];
    for (a, value) in xyz.iter_mut().enumerate() {
        let w = precision.axis_bits[a];
        let q = read(pay, off, w);
        off += w;
        if q == 0 || q == (1u64 << w) - 1 {
            return None;
        }
        *value =
            bounds.min[a] + (q as f32 + 0.5) * (bounds.max[a] - bounds.min[a]) / (1u64 << w) as f32;
    }
    Some((xyz, off))
}
/// Loaded native descriptor path. Uses the pinned Go 64-bit int/uint arithmetic
/// on host and WASM, including wide reads, signed width casts and uint32 region
/// truncation. Unlike the bounded convenience descriptor, no 32-bit width cap
/// is imposed. Non-finite coordinates are retained when native dequantization
/// produces them (for example, a shift of 64 yields a zero divisor).
pub fn decode_world_object_positions_with_descriptor(
    pay: &[u8],
    band: &BTreeSet<u32>,
    bounds: &FilmMapBounds,
    precision: &super::NativePrecisionDescriptor,
) -> Vec<WorldObjectPositionRecord> {
    use super::native_bits_tolerant as peek;
    let pos_bits = precision
        .axis_bits
        .iter()
        .fold(precision.index_bits, |a, b| a.wrapping_add(*b))
        .wrapping_add(4) as i64;
    let pay_bits = (pay.len() as i64).wrapping_mul(8);
    let limit = pay_bits.wrapping_sub(27_i64.wrapping_add(pos_bits));
    let mut p = 0_i64;
    let mut out = Vec::new();
    // Past the payload every prefix bit is zero. Skipping that padded suffix
    // also avoids iterating huge synthetic ranges after descriptor overflow.
    while p <= limit && p < pay_bits {
        let start = p;
        p = p.wrapping_add(1);
        if peek(pay, start, 1) != 1 {
            continue;
        }
        let slot = peek(pay, start.wrapping_add(1), 13) as u32;
        if !band.contains(&slot) || peek(pay, start.wrapping_add(16), 2) != 0 {
            continue;
        }
        let count = peek(pay, start.wrapping_add(18), 3) as i64;
        if count == 0 {
            continue;
        }
        let idx: Vec<_> = (0..count)
            .map(|i| peek(pay, start.wrapping_add(21 + 6 * i), 6))
            .collect();
        if idx[0] != 0 || idx.windows(2).any(|a| a[0] >= a[1]) {
            continue;
        }
        let at = start.wrapping_add(21 + 6 * count);
        let Some((xyz, off)) =
            decode_world_object_position_with_descriptor(pay, at, bounds, precision)
        else {
            continue;
        };
        out.push(WorldObjectPositionRecord {
            slot,
            generation: peek(pay, start.wrapping_add(14), 2) as u32,
            bit: start as usize,
            padded_bits: off.saturating_sub(pay_bits).max(0) as usize,
            sample: WorldObjectSample {
                timestamp_us: 0,
                chunk: 0,
                x: xyz[0],
                y: xyz[1],
                z: xyz[2],
                at_rest: idx.contains(&18),
            },
        });
        p = p.wrapping_add(pos_bits);
    }
    out
}

/// Selective native position primitive; the offset excludes two tail flags.
pub fn decode_world_object_position_with_descriptor(
    pay: &[u8],
    at: i64,
    bounds: &FilmMapBounds,
    precision: &super::NativePrecisionDescriptor,
) -> Option<([f32; 3], i64)> {
    use super::native_bits_tolerant as peek;
    if peek(pay, at, 2) != 0
        || peek(pay, at.wrapping_add(2), precision.index_bits as i64) as u32 != precision.region
    {
        return None;
    }
    let mut off = at.wrapping_add(2).wrapping_add(precision.index_bits as i64);
    let mut xyz = [0.; 3];
    for (a, value) in xyz.iter_mut().enumerate() {
        let width = precision.axis_bits[a];
        let q = peek(pay, off, width as i64);
        let levels = if width < 64 { 1_u64 << width } else { 0 };
        if q == 0 || q == levels.wrapping_sub(1) {
            return None;
        }
        *value = bounds.min[a] + (q as f32 + 0.5) * (bounds.max[a] - bounds.min[a]) / levels as f32;
        off = off.wrapping_add(width as i64);
    }
    Some((xyz, off))
}

fn sample_order(a: &WorldObjectSample, b: &WorldObjectSample) -> Ordering {
    if a.timestamp_us != b.timestamp_us {
        return a.timestamp_us.cmp(&b.timestamp_us);
    }
    // Native comparisons stop at the first unequal axis, including NaN.
    for (a, b) in [(a.x, b.x), (a.y, b.y), (a.z, b.z)] {
        if a != b {
            return a.partial_cmp(&b).unwrap_or(Ordering::Equal);
        }
    }
    Ordering::Equal
}
/// Group by slot/generation, split after rest or gaps greater than 250ms, and
/// retain only mobile segments containing at least three samples.
pub fn assemble_world_object_tracks(
    records: Vec<WorldObjectPositionRecord>,
) -> Vec<WorldObjectTrack> {
    let mut lives = BTreeMap::<(u32, u32), Vec<WorldObjectSample>>::new();
    for r in records {
        lives
            .entry((r.slot, r.generation))
            .or_default()
            .push(r.sample);
    }
    let mut out = Vec::new();
    for ((slot, generation), mut pts) in lives {
        super::native_sort::sort_by(&mut pts, sample_order);
        let mut start = 0;
        for i in 1..=pts.len() {
            if i == pts.len()
                || pts[i].timestamp_us.wrapping_sub(pts[i - 1].timestamp_us) > 250000
                || pts[i - 1].at_rest
            {
                if i - start >= 3 {
                    out.push(WorldObjectTrack {
                        slot,
                        generation,
                        pts: pts[start..i].to_vec(),
                    });
                }
                start = i;
            }
        }
    }
    super::native_sort::sort_by(&mut out, |a, b| {
        a.pts[0]
            .timestamp_us
            .cmp(&b.pts[0].timestamp_us)
            .then(a.slot.cmp(&b.slot))
            .then(a.generation.cmp(&b.generation))
            .then(a.pts.len().cmp(&b.pts.len()))
            .then_with(|| {
                a.pts
                    .iter()
                    .zip(&b.pts)
                    .map(|(a, b)| {
                        if a.timestamp_us != b.timestamp_us {
                            return a.timestamp_us.cmp(&b.timestamp_us);
                        }
                        for (a, b) in [(a.x, b.x), (a.y, b.y), (a.z, b.z)] {
                            if a != b {
                                return if a < b {
                                    Ordering::Less
                                } else {
                                    Ordering::Greater
                                };
                            }
                        }
                        a.at_rest.cmp(&b.at_rest).then(a.chunk.cmp(&b.chunk))
                    })
                    .find(|c| *c != Ordering::Equal)
                    .unwrap_or(Ordering::Equal)
            })
    });
    out
}
pub fn scan_world_object_tracks(
    chunks: &[FilmChunkData],
    map: &FilmMapBounds,
    archetype: u32,
) -> Result<WorldObjectTrackStream, DecodeError> {
    let band = world_object_slot_band(chunks, archetype)?;
    if band.is_empty() {
        return Err(DecodeError::Missing("world object slot band"));
    }
    scan_world_object_tracks_for_band(chunks, map, &band)
}
pub fn scan_world_object_tracks_for_band(
    chunks: &[FilmChunkData],
    map: &FilmMapBounds,
    band: &BTreeSet<u32>,
) -> Result<WorldObjectTrackStream, DecodeError> {
    scan_world_object_tracks_for_band_with_position(chunks, map, band, &map.position_encoding())
}
pub fn scan_world_object_tracks_for_band_with_position(
    chunks: &[FilmChunkData],
    map: &FilmMapBounds,
    band: &BTreeSet<u32>,
    position: &super::PositionEncoding,
) -> Result<WorldObjectTrackStream, DecodeError> {
    let precision = WorldObjectPrecision::from_position(position)
        .ok_or(DecodeError::Missing("world object axis precision"))?;
    let mut records = Vec::new();
    let mut padding = Vec::new();
    for c in native_chunk_prefix(chunks)? {
        for (index, p) in native_chunk_packets(c).into_iter().enumerate() {
            if p.packet_type != 0 {
                continue;
            }
            for mut r in decode_world_object_positions_with_precision(
                &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                band,
                map,
                precision,
            ) {
                r.sample.timestamp_us = p.timestamp_us;
                r.sample.chunk = i64::from(c.metadata.index);
                if r.padded_bits > 0 {
                    padding.push(WorldObjectPadding {
                        chunk: i64::from(c.metadata.index),
                        packet_index: index,
                        record: r.clone(),
                    });
                }
                records.push(r);
            }
        }
    }
    Ok(WorldObjectTrackStream {
        band: band.clone(),
        tracks: assemble_world_object_tracks(records),
        padding,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    fn oracle() -> serde_json::Value {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/world-tracks-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        serde_json::from_slice(&raw).unwrap()
    }
    #[test]
    fn native_world_object_positions() {
        let rows = oracle();
        for (i, row) in rows["cases"].as_array().unwrap().iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let pay: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                .collect();
            let map = FilmMapBounds {
                module: String::new(),
                min: [-100., -200., -300.],
                max: [300., 400., 500.],
                axis_widths: serde_json::from_value(row["widths"].clone()).unwrap(),
                region: row["region"].as_u64().unwrap() as u32,
                region_index_bits: row["index_bits"].as_u64().unwrap() as usize,
            };
            let actual = decode_world_object_positions(&pay, &[512, 513, 515].into(), &map);
            assert_eq!(
                actual.len(),
                row["records"].as_array().unwrap().len(),
                "count {i}"
            );
            for (a, e) in actual.iter().zip(row["records"].as_array().unwrap()) {
                let expected: WorldObjectSample =
                    serde_json::from_value(e["sample"].clone()).unwrap();
                assert_eq!(a.sample, expected, "position {i}");
                assert_eq!(a.slot, e["slot"].as_u64().unwrap() as u32);
                assert_eq!(a.generation, e["generation"].as_u64().unwrap() as u32);
            }
        }
    }
    #[test]
    fn native_world_object_lifetime_order() {
        let rows = oracle();
        for (i, row) in rows["tracks"].as_array().unwrap().iter().enumerate() {
            let pts: Vec<WorldObjectSample> = serde_json::from_value(row["input"].clone()).unwrap();
            let actual = assemble_world_object_tracks(
                pts.into_iter()
                    .map(|sample| WorldObjectPositionRecord {
                        slot: 512,
                        generation: 1,
                        sample,
                        bit: 0,
                        padded_bits: 0,
                    })
                    .collect(),
            );
            let expected: Vec<WorldObjectTrack> =
                serde_json::from_value(row["tracks"].clone()).unwrap();
            assert_eq!(actual, expected, "tracks {i}");
        }
    }
}
