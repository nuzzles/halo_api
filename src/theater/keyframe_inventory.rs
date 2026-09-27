//! Native keyframe inventory inference: ability signatures, ammo boundaries and grenade windows.
//! These reference heuristics retain ambiguity counts and distinguish unread fields from zero.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", try_from = "SlotAmmoWire")]
pub struct KeyframeSlotAmmo {
    pub mag: Option<u32>,
    pub res: Option<u32>,
    pub gauge: Option<f64>,
    /// Exact transmitted R(12), retained so JSON decoding cannot round the gauge differently.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gauge_quantum: Option<u16>,
    pub overheat: u32,
    pub flags: u32,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SlotAmmoWire {
    mag: Option<u32>,
    res: Option<u32>,
    gauge: Option<f64>,
    #[serde(default)]
    gauge_quantum: Option<u16>,
    overheat: u32,
    flags: u32,
}
impl TryFrom<SlotAmmoWire> for KeyframeSlotAmmo {
    type Error = &'static str;
    fn try_from(wire: SlotAmmoWire) -> Result<Self, Self::Error> {
        if wire.gauge_quantum.is_some_and(|q| q > 4095) {
            return Err("gauge quantum exceeds 12 bits");
        }
        Ok(Self {
            mag: wire.mag,
            res: wire.res,
            gauge: wire
                .gauge_quantum
                .map(|q| f64::from(q) / 4095.)
                .or(wire.gauge),
            gauge_quantum: wire.gauge_quantum,
            overheat: wire.overheat,
            flags: wire.flags,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyframeInventory {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub chunk: i64,
    pub packet_index: usize,
    pub slot: u32,
    pub grenades: [u32; 4],
    pub grenades_read: bool,
    pub grenades_by_position: bool,
    pub selected_grenade_rank: i32,
    pub ability_rank: i32,
    pub ammo: [KeyframeSlotAmmo; 4],
    pub ammo_read: bool,
    pub drawn_slot: i32,
    pub ammo_candidates: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyframeInventoryStats {
    pub chunks: usize,
    pub chunks_unread: usize,
    pub keyframes: usize,
    pub records: usize,
    pub grenades_by_anchor: usize,
    pub grenades_by_position: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyframeInventoryStream {
    pub records: Vec<KeyframeInventory>,
    pub stats: KeyframeInventoryStats,
    /// The effective mode cap; zero requests the native default of two.
    pub grenade_max: u32,
    /// Reports use of the native default-cap fallback instead of silently applying it.
    pub default_grenade_max: bool,
}

pub fn scan_keyframe_inventory(
    chunks: &[FilmChunkData],
    grenade_max: u32,
) -> Result<KeyframeInventoryStream, DecodeError> {
    scan_keyframe_inventory_with_families(
        chunks,
        &v41_weapon_families().keys().copied().collect(),
        grenade_max,
    )
}
pub fn scan_keyframe_inventory_with_families(
    chunks: &[FilmChunkData],
    known: &BTreeSet<u32>,
    grenade_max: u32,
) -> Result<KeyframeInventoryStream, DecodeError> {
    let (stream, error) = scan_keyframe_inventory_with_diagnostics(chunks, known, grenade_max);
    match error {
        Some(error) => Err(error),
        None => Ok(stream),
    }
}

/// Preserve native scan counts and the default-cap fallback even when no readable
/// data prefix exists. An empty family catalog returns before source validation.
pub fn scan_keyframe_inventory_with_diagnostics(
    chunks: &[FilmChunkData],
    known: &BTreeSet<u32>,
    grenade_max: u32,
) -> (KeyframeInventoryStream, Option<DecodeError>) {
    let mut out = KeyframeInventoryStream::default();
    if known.is_empty() {
        return (out, None);
    }
    out.default_grenade_max = grenade_max == 0;
    out.grenade_max = if grenade_max == 0 { 2 } else { grenade_max };
    let selected = match super::fire_events::native_chunk_prefix(chunks) {
        Ok(selected) => selected,
        Err(error) => return (out, Some(error)),
    };
    out.stats.chunks = selected.len();
    for chunk in selected {
        for (index, packet) in super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .enumerate()
        {
            if packet.packet_type != 2 {
                continue;
            }
            out.stats.keyframes += 1;
            let payload =
                &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            for mut inv in decode_keyframe_inventories(payload, known, out.grenade_max) {
                inv.timestamp_us = packet.timestamp_us;
                inv.chunk = i64::from(packet.chunk_index);
                inv.packet_index = index;
                if inv.grenades_read {
                    if inv.grenades_by_position {
                        out.stats.grenades_by_position += 1;
                    } else {
                        out.stats.grenades_by_anchor += 1;
                    }
                }
                out.records.push(inv);
            }
        }
    }
    out.stats.records = out.records.len();
    (out, None)
}

// Inventory's native bit helper explicitly zero-pads both sides of the payload.
fn bits(pay: &[u8], at: isize, width: usize) -> u32 {
    let mut value = 0;
    for i in 0..width {
        let p = at + i as isize;
        let bit = usize::try_from(p)
            .ok()
            .and_then(|p| pay.get(p / 8).map(|b| (b >> (7 - p % 8)) & 1))
            .unwrap_or(0);
        value = (value << 1) | u32::from(bit);
    }
    value
}
fn ability_hits(pay: &[u8], from: isize, to: isize) -> Vec<(isize, u32)> {
    let mut out = vec![];
    let mut word = 0;
    for bit in from..to {
        word = ((word << 1) | bits(pay, bit, 1)) & 0x0fff_ffff;
        if bit - from < 27 || word != 0x8cac57a {
            continue;
        }
        let anchor = bit - 27;
        for off in 0..=60 {
            let p = anchor + 28 + off;
            if p + 20 > to {
                break;
            }
            if bits(pay, p, 20) == 0x12 {
                out.push((anchor, bits(pay, p + 20, 3)));
                break;
            }
        }
    }
    out
}
fn grenade_counts(pay: &[u8], at: isize, max: u32) -> Option<[u32; 4]> {
    if bits(pay, at, 3) != 4 {
        return None;
    }
    let values = std::array::from_fn(|i| bits(pay, at + 3 + 8 * i as isize, 8));
    values.iter().all(|v| *v <= max).then_some(values)
}
fn grenade_selection(pay: &[u8], family_end: isize, to: isize, grenades: [u32; 4]) -> i32 {
    let mask = grenades
        .iter()
        .enumerate()
        .fold(0, |mask, (i, v)| mask | if *v > 0 { 1 << i } else { 0 });
    if mask == 0 {
        return -1;
    }
    let mut selection = -1;
    for offset in 200..=210 {
        let at = family_end + offset;
        if at + 9 > to {
            break;
        }
        if bits(pay, at, 6) != mask {
            continue;
        }
        let rank = bits(pay, at + 6, 3);
        if !(1..=4).contains(&rank) || mask & (1 << (rank - 1)) == 0 {
            continue;
        }
        let selected = rank as i32 - 1;
        if selection >= 0 && selection != selected {
            return -1;
        }
        selection = selected;
    }
    selection
}
#[derive(Debug, Clone, PartialEq)]
struct AmmoParse {
    slots: [KeyframeSlotAmmo; 4],
    selected: i32,
    end: isize,
    complete: bool,
}
fn parse_ammo(pay: &[u8], start: isize, limit: isize) -> AmmoParse {
    let mut out = AmmoParse {
        slots: std::array::from_fn(|_| Default::default()),
        selected: -1,
        end: start,
        complete: false,
    };
    let parse = |out: &mut AmmoParse| -> Option<()> {
        fn read(pay: &[u8], pos: &mut isize, limit: isize, width: usize) -> Option<u32> {
            if *pos + width as isize > limit {
                return None;
            }
            let value = bits(pay, *pos, width);
            *pos += width as isize;
            Some(value)
        }
        for slot in &mut out.slots {
            if read(pay, &mut out.end, limit, 1)? == 0 {
                slot.mag = Some(read(pay, &mut out.end, limit, 8)?);
            }
            if read(pay, &mut out.end, limit, 1)? == 0 {
                let quantum = read(pay, &mut out.end, limit, 12)?;
                slot.gauge_quantum = Some(quantum as u16);
                slot.gauge = Some(f64::from(quantum) / 4095.);
            }
            if out.end + 20 > limit {
                return None;
            }
            slot.res = Some(read(pay, &mut out.end, limit, 11)?);
            slot.flags = read(pay, &mut out.end, limit, 2)?;
            slot.overheat = read(pay, &mut out.end, limit, 7)?;
        }
        read(pay, &mut out.end, limit, 3)?;
        for _ in 0..2 {
            if read(pay, &mut out.end, limit, 1)? == 0 {
                out.selected = read(pay, &mut out.end, limit, 2)? as i32;
            }
        }
        Some(())
    };
    out.complete = parse(&mut out).is_some();
    out
}
fn solve_ammo(pay: &[u8], end: isize, lo: isize) -> Vec<isize> {
    (lo..end)
        .filter(|&start| {
            let parsed = parse_ammo(pay, start, end + 1);
            parsed.complete
                && parsed.end == end
                && parsed.slots.iter().all(|s| {
                    !(s.mag.is_some() && s.gauge.is_some())
                        && (s.mag.is_some()
                            || s.gauge.is_some()
                            || (s.res == Some(0) && s.flags == 0 && s.overheat == 0))
                })
        })
        .collect()
}

pub fn decode_keyframe_inventories(
    pay: &[u8],
    known: &BTreeSet<u32>,
    grenade_max: u32,
) -> Vec<KeyframeInventory> {
    let anchors = recover_keyframe_anchors(pay);
    let mut out = vec![];
    for (i, anchor) in anchors.iter().enumerate() {
        if anchor.archetype != 35 {
            continue;
        }
        let from = anchor.bit as isize;
        let to = anchors.get(i + 1).map_or(pay.len() * 8, |a| a.bit) as isize;
        let mut inv = KeyframeInventory {
            timestamp_us: 0,
            chunk: 0,
            packet_index: 0,
            slot: anchor.id & 0x3fff_ffff,
            grenades: [0; 4],
            grenades_read: false,
            grenades_by_position: false,
            selected_grenade_rank: -1,
            ability_rank: -1,
            ammo: std::array::from_fn(|_| Default::default()),
            ammo_read: false,
            drawn_slot: -1,
            ammo_candidates: 0,
        };
        let hits = ability_hits(pay, from, to);
        if hits.len() == 1 {
            inv.ability_rank = (16 | hits[0].1) as i32;
            for at in hits[0].0..=to - 35 {
                if let Some(counts) = grenade_counts(pay, at, grenade_max)
                    && counts.iter().any(|v| *v > 0)
                {
                    inv.grenades = counts;
                    inv.grenades_read = true;
                    break;
                }
            }
        }
        let mut families = vec![];
        let mut word = 0;
        for bit in from..to {
            word = (word << 1) | bits(pay, bit, 1);
            if bit - from >= 31 && known.contains(&word) {
                families.push(bit - 31);
            }
        }
        if let Some(&first) = families.first() {
            let end = first - 1;
            let solutions = solve_ammo(pay, end, (end - 300).max(from));
            inv.ammo_candidates = solutions.len();
            if let Some(&start) = solutions.first() {
                let parsed = parse_ammo(pay, start, end + 1);
                inv.ammo = parsed.slots;
                inv.ammo_read = true;
                inv.drawn_slot = parsed.selected;
                if !inv.grenades_read {
                    for at in (start - 216).max(from)..=start - 127 {
                        if let Some(counts) = grenade_counts(pay, at, grenade_max) {
                            inv.grenades = counts;
                            inv.grenades_read = true;
                            inv.grenades_by_position = true;
                            break;
                        }
                    }
                }
            }
        }
        if inv.grenades_read
            && let Some(&last) = families.last()
        {
            inv.selected_grenade_rank = grenade_selection(pay, last + 32, to, inv.grenades);
        }
        out.push(inv);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn film_retains_inventory_source_failure_and_fallback() {
        use crate::clients::hi::models::FilmChunk;
        let inflate = |bytes: &[u8]| {
            let mut data = Vec::new();
            flate2::read::ZlibDecoder::new(bytes)
                .read_to_end(&mut data)
                .unwrap();
            data
        };
        let bootstrap = inflate(include_bytes!("fixtures/bootstrap-v41.zlib"));
        let payload = inflate(include_bytes!("fixtures/captured-keyframe-v41.zlib"));
        let mut packet = vec![2, 0, 0, 0];
        packet.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        packet.extend_from_slice(&1000u64.to_le_bytes());
        packet.extend(payload);
        // A valid packet exists, but native inventory requires the numbered data
        // prefix beginning at chunk 1. It still records the default-cap fallback.
        let chunks: Vec<_> = [(0, 1, bootstrap), (2, 2, packet)]
            .into_iter()
            .map(|(index, chunk_type, data)| FilmChunkData {
                metadata: FilmChunk {
                    index,
                    chunk_type,
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
        let film = LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap();
        assert!(film.keyframe_inventory_error.is_some());
        let scan = film.keyframe_inventory.as_ref().unwrap();
        assert!(scan.records.is_empty());
        assert_eq!(scan.stats, KeyframeInventoryStats::default());
        assert!(scan.default_grenade_max);
        assert_eq!(scan.grenade_max, 2);
        let restored: LegacyFilm =
            serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
        assert_eq!(restored.keyframe_inventory, film.keyframe_inventory);
        assert_eq!(
            restored.keyframe_inventory_error,
            film.keyframe_inventory_error
        );
    }

    #[test]
    fn native_inventory_packet_source() {
        use crate::clients::hi::models::FilmChunk;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/inventory-packet-source-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 120);
        let mut positive = 0;
        let mut errors = 0;
        for (case, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    let hex = v["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: v["index"].as_i64().unwrap() as i32,
                            chunk_type: v["chunk_type"].as_i64().unwrap() as i32,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let known = if row["known"] == true {
                [0xabcedf13, 0xdeadbeef].into()
            } else {
                BTreeSet::new()
            };
            let cap = row["cap"].as_u64().unwrap() as u32;
            let (result, error) = scan_keyframe_inventory_with_diagnostics(&chunks, &known, cap);
            assert_eq!(
                error.is_some(),
                row["error"].as_bool().unwrap(),
                "case {case}"
            );
            assert_eq!(
                scan_keyframe_inventory_with_families(&chunks, &known, cap).is_err(),
                error.is_some()
            );
            errors += usize::from(error.is_some());
            let mut expected: Vec<KeyframeInventory> =
                serde_json::from_value(row["records"].clone()).unwrap();
            for (inv, bits) in expected
                .iter_mut()
                .zip(row["gauge_bits"].as_array().unwrap())
            {
                for (slot, value) in inv.ammo.iter_mut().zip(bits.as_array().unwrap()) {
                    slot.gauge = value.as_u64().map(f64::from_bits);
                    slot.gauge_quantum = slot.gauge.map(|g| (g * 4095.).round() as u16);
                }
            }
            assert_eq!(result.records, expected, "records case {case}");
            assert_eq!(
                serde_json::json!(result.stats),
                row["stats"],
                "stats case {case}"
            );
            assert_eq!(
                usize::from(result.default_grenade_max),
                row["fallbacks"].as_u64().unwrap() as usize
            );
            positive += usize::from(!result.records.is_empty());
            assert_eq!(
                serde_json::from_value::<KeyframeInventoryStream>(serde_json::json!(result))
                    .unwrap(),
                result
            );
        }
        assert_eq!((positive, errors), (18, 30));
    }

    #[test]
    fn keyframe_inventory_and_partial_ammo_match_native() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/keyframe-inventory-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let oracle: serde_json::Value = serde_json::from_str(&json).unwrap();
        let decode = |row: &serde_json::Value| {
            let hex = row["hex"].as_str().unwrap();
            (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>()
        };
        let cases = oracle["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 256);
        for row in cases {
            let known = serde_json::from_value(row["known"].clone()).unwrap();
            let actual = decode_keyframe_inventories(
                &decode(row),
                &known,
                row["cap"].as_u64().unwrap() as u32,
            );
            let mut expected: Vec<KeyframeInventory> =
                serde_json::from_value(row["records"].clone()).unwrap();
            for (inv, bits) in expected
                .iter_mut()
                .zip(row["gauge_bits"].as_array().unwrap())
            {
                for (slot, value) in inv.ammo.iter_mut().zip(bits.as_array().unwrap()) {
                    slot.gauge = value.as_u64().map(f64::from_bits);
                    slot.gauge_quantum = slot.gauge.map(|g| (g * 4095.).round() as u16);
                }
            }
            assert_eq!(actual, expected);
            let restored: Vec<KeyframeInventory> =
                serde_json::from_slice(&serde_json::to_vec(&actual).unwrap()).unwrap();
            assert_eq!(actual, restored);
        }
        let cases = oracle["ammo"].as_array().unwrap();
        assert_eq!(cases.len(), 2048);
        for row in cases {
            let actual = parse_ammo(
                &decode(row),
                row["start"].as_i64().unwrap() as isize,
                row["limit"].as_i64().unwrap() as isize,
            );
            let mut slots: [KeyframeSlotAmmo; 4] =
                serde_json::from_value(row["slots"].clone()).unwrap();
            for (slot, value) in slots.iter_mut().zip(row["gauge_bits"].as_array().unwrap()) {
                slot.gauge = value.as_u64().map(f64::from_bits);
                slot.gauge_quantum = slot.gauge.map(|g| (g * 4095.).round() as u16);
            }
            assert_eq!(actual.slots, slots, "{row}");
            assert_eq!(
                serde_json::json!([actual.selected, actual.end as i32]),
                serde_json::json!([row["selected"], row["end"]])
            );
            assert_eq!(actual.complete, row["complete"].as_bool().unwrap());
        }
    }
}
