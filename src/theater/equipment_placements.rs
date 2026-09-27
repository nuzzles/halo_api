//! Equipment placements confirmed against independently decoded mobile lifetimes.
//! The end time is the last movement observation, never a disappearance time.
use super::{EquipmentCreation, FilmMapBounds, WorldObjectTrack};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EquipmentLifeKey {
    pub slot: u32,
    #[serde(rename = "Gen")]
    pub generation: u32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EquipmentLifeSpan {
    #[serde(rename = "T0US")]
    pub t0_us: u64,
    #[serde(rename = "T1US")]
    pub t1_us: u64,
    pub first: [f32; 3],
    pub points: usize,
}
pub type EquipmentLifeSpans = BTreeMap<EquipmentLifeKey, Vec<EquipmentLifeSpan>>;
pub fn equipment_life_spans(tracks: &[WorldObjectTrack]) -> EquipmentLifeSpans {
    let mut out = EquipmentLifeSpans::new();
    for tr in tracks {
        let Some(p) = tr.pts.first() else { continue };
        out.entry(EquipmentLifeKey {
            slot: tr.slot,
            generation: tr.generation,
        })
        .or_default()
        .push(EquipmentLifeSpan {
            t0_us: p.timestamp_us,
            t1_us: tr.pts.last().unwrap().timestamp_us,
            first: [p.x, p.y, p.z],
            points: tr.pts.len(),
        });
    }
    for spans in out.values_mut() {
        super::native_sort::sort_by(spans, |a, b| a.t0_us.cmp(&b.t0_us));
    }
    out
}
pub fn match_equipment_life(
    spans: &[EquipmentLifeSpan],
    pos: [f32; 3],
    eps: [f32; 3],
    at_us: u64,
) -> Option<&EquipmentLifeSpan> {
    let mut best: Option<&EquipmentLifeSpan> = None;
    for s in spans {
        if (0..3).all(|i| (s.first[i] - pos[i]).abs() <= eps[i])
            && best.is_none_or(|b| s.t0_us.abs_diff(at_us) < b.t0_us.abs_diff(at_us))
        {
            best = Some(s);
        }
    }
    best
}
pub fn equipment_position_epsilon(map: &FilmMapBounds) -> [f32; 3] {
    std::array::from_fn(|i| 0.002 * (map.max[i] - map.min[i]))
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EquipmentPlacement {
    pub life: EquipmentLifeKey,
    #[serde(rename = "T0US")]
    pub t0_us: u64,
    #[serde(rename = "T1US")]
    pub t1_us: u64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(rename = "GlobalID")]
    pub global_id: u32,
    pub points: usize,
}
/// Return one earliest creation per matching lifetime, and the count before deduplication.
pub fn confirm_equipment_placements(
    creations: &[EquipmentCreation],
    spans: &EquipmentLifeSpans,
    eps: [f32; 3],
) -> (Vec<EquipmentPlacement>, usize) {
    let mut best = BTreeMap::<(EquipmentLifeKey, u64), EquipmentPlacement>::new();
    let mut confirmed = 0;
    for c in creations {
        let key = EquipmentLifeKey {
            slot: c.slot,
            generation: c.generation,
        };
        let Some(life) = spans
            .get(&key)
            .and_then(|s| match_equipment_life(s, [c.x, c.y, c.z], eps, c.timestamp_us))
        else {
            continue;
        };
        confirmed += 1;
        let id = (key, life.t0_us);
        if best.get(&id).is_some_and(|cur| cur.t0_us <= c.timestamp_us) {
            continue;
        }
        best.insert(
            id,
            EquipmentPlacement {
                life: key,
                t0_us: c.timestamp_us,
                t1_us: life.t1_us,
                x: c.x,
                y: c.y,
                z: c.z,
                global_id: c.mpp_val[1] as u32,
                points: life.points,
            },
        );
    }
    let mut out: Vec<_> = best.into_values().collect();
    out.sort_by(|a, b| {
        a.t0_us
            .cmp(&b.t0_us)
            .then(a.life.cmp(&b.life))
            .then(a.t1_us.cmp(&b.t1_us))
            .then_with(|| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.global_id.cmp(&b.global_id))
            .then(a.points.cmp(&b.points))
    });
    (out, confirmed)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EquipmentMppWidths {
    pub lead: usize,
    pub index: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentMppScore {
    #[serde(rename = "key")]
    pub widths: EquipmentMppWidths,
    #[serde(rename = "value")]
    pub agree: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EquipmentMppCalibration {
    pub widths: EquipmentMppWidths,
    pub agree: usize,
    pub runner: EquipmentMppWidths,
    pub runner_agree: usize,
    pub anchors: usize,
    pub chunks: usize,
    pub lives: usize,
    /// Nonzero scores in native candidate preference order; struct-keyed maps
    /// cannot be represented directly as JSON objects.
    pub by_widths: Vec<EquipmentMppScore>,
}
pub(super) fn candidates() -> impl Iterator<Item = EquipmentMppWidths> {
    [5, 2, 3, 4, 6, 7, 8].into_iter().flat_map(|index| {
        [9, 6, 5, 7, 8, 10, 11, 12, 13]
            .into_iter()
            .map(move |lead| EquipmentMppWidths { lead, index })
    })
}
pub(super) fn verdict(cal: &mut EquipmentMppCalibration) -> bool {
    cal.widths = Default::default();
    cal.runner = Default::default();
    cal.agree = 0;
    cal.runner_agree = 0;
    for w in candidates() {
        let n = cal
            .by_widths
            .iter()
            .find(|s| s.widths == w)
            .map_or(0, |s| s.agree);
        if n > cal.agree {
            cal.runner = cal.widths;
            cal.runner_agree = cal.agree;
            cal.widths = w;
            cal.agree = n;
        } else if n > cal.runner_agree {
            cal.runner = w;
            cal.runner_agree = n;
        }
    }
    let ok = cal.agree >= 12 && cal.agree >= 3 * cal.runner_agree;
    if !ok {
        cal.widths = Default::default();
    }
    ok
}
pub fn calibrate_equipment_mpp(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &std::collections::BTreeSet<u32>,
    components: usize,
    spans: &EquipmentLifeSpans,
) -> EquipmentMppCalibration {
    calibrate_equipment_mpp_with_position(
        chunks,
        map,
        band,
        components,
        spans,
        &map.position_encoding(),
    )
}
/// Calibrate only MPP widths, retaining the inherited world-object descriptor.
pub fn calibrate_equipment_mpp_with_position(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &std::collections::BTreeSet<u32>,
    components: usize,
    spans: &EquipmentLifeSpans,
    position: &super::PositionEncoding,
) -> EquipmentMppCalibration {
    let mut cal = EquipmentMppCalibration {
        lives: spans.len(),
        ..Default::default()
    };
    if band.is_empty() || spans.is_empty() {
        return cal;
    }
    let Ok(chunks) = super::fire_events::native_chunk_prefix(chunks) else {
        return cal;
    };
    let eps = equipment_position_epsilon(map);
    let mut scores = [0usize; 63];
    for chunk in chunks {
        cal.chunks += 1;
        for p in super::fire_events::native_chunk_packets(chunk) {
            if p.packet_type != 0 {
                continue;
            }
            let pay = &chunk.data[p.payload_offset..p.payload_offset + p.payload_size];
            let Some(limit) = pay.len().saturating_mul(8).checked_sub(24) else {
                continue;
            };
            for at in 0..=limit {
                let Some((slot, generation)) =
                    super::equipment_creations::equipment_new_header(pay, at, band)
                else {
                    continue;
                };
                let Some(lives) = spans
                    .get(&EquipmentLifeKey { slot, generation })
                    .filter(|s| !s.is_empty())
                else {
                    continue;
                };
                cal.anchors += 1;
                for (i, w) in candidates().enumerate() {
                    let Some(c) = super::equipment_creations::read_equipment_creation_with_position(
                        pay,
                        at,
                        components,
                        [w.lead, w.index],
                        map,
                        &mut Default::default(),
                        position,
                    ) else {
                        continue;
                    };
                    if match_equipment_life(lives, [c.x, c.y, c.z], eps, p.timestamp_us).is_some() {
                        scores[i] += 1;
                    }
                }
            }
        }
        cal.by_widths = candidates()
            .zip(scores)
            .filter(|(_, n)| *n > 0)
            .map(|(widths, agree)| EquipmentMppScore { widths, agree })
            .collect();
        if verdict(&mut cal) {
            break;
        }
    }
    cal
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EquipmentPlacementStats {
    pub scanned: bool,
    pub calibration: EquipmentMppCalibration,
    pub lives: usize,
    pub slots: usize,
    pub anchors: usize,
    pub accepted: usize,
    pub confirmed: usize,
    pub placements: usize,
    #[serde(rename = "ByID")]
    pub by_id: BTreeMap<u32, usize>,
    pub format_version: u32,
    pub format_sans_profil: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquipmentPlacementStream {
    pub placements: Vec<EquipmentPlacement>,
    pub stats: EquipmentPlacementStats,
    pub creations: super::EquipmentCreationStream,
}
pub fn scan_equipment_placements_for_band(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &std::collections::BTreeSet<u32>,
    registry: &super::FilmRegistry,
) -> Result<EquipmentPlacementStream, super::DecodeError> {
    scan_equipment_placements_for_band_with_position(
        chunks,
        map,
        band,
        registry,
        &map.position_encoding(),
    )
}
/// Use the same inherited descriptor for mobile lifetimes, calibration and creations.
pub fn scan_equipment_placements_for_band_with_position(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &std::collections::BTreeSet<u32>,
    registry: &super::FilmRegistry,
    position: &super::PositionEncoding,
) -> Result<EquipmentPlacementStream, super::DecodeError> {
    let prefix = super::fire_events::native_chunk_prefix(chunks)?;
    if band.is_empty() {
        return Err(super::DecodeError::Missing("equipment slot band"));
    }
    let components = registry
        .archetype(37)
        .ok_or(super::DecodeError::Missing("equipment archetype"))?
        .components
        .len();
    let tracks =
        super::scan_world_object_tracks_for_band_with_position(chunks, map, band, position)?;
    let spans = equipment_life_spans(&tracks.tracks);
    let calibration =
        calibrate_equipment_mpp_with_position(chunks, map, band, components, &spans, position);
    let widths = if registry.format_version == 27 {
        EquipmentMppWidths { lead: 9, index: 5 }
    } else {
        calibration.widths
    };
    let mut out = EquipmentPlacementStream {
        placements: Vec::new(),
        stats: EquipmentPlacementStats {
            scanned: true,
            lives: spans.len(),
            slots: band.len(),
            format_version: registry.format_version,
            format_sans_profil: !matches!(registry.format_version, 20 | 21 | 24 | 25 | 27),
            calibration,
            ..Default::default()
        },
        creations: super::EquipmentCreationStream {
            records: Vec::new(),
            stats: super::EquipmentCreationStats {
                slots: band.len(),
                ..Default::default()
            },
        },
    };
    if widths.lead == 0 || widths.index == 0 {
        return Ok(out);
    }
    for chunk in prefix {
        for (index, p) in super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .enumerate()
        {
            if p.packet_type != 0 {
                continue;
            }
            let mut stream = super::decode_equipment_creations_with_position(
                &chunk.data[p.payload_offset..p.payload_offset + p.payload_size],
                band,
                components,
                [widths.lead, widths.index],
                map,
                position,
            );
            for c in &mut stream.records {
                c.chunk = i64::from(chunk.metadata.index);
                c.packet_index = index;
                c.timestamp_us = p.timestamp_us;
            }
            let a = &mut out.creations.stats;
            let b = stream.stats;
            a.anchors += b.anchors;
            a.overflow += b.overflow;
            a.mask_bad += b.mask_bad;
            a.pos_bad += b.pos_bad;
            a.accepted += b.accepted;
            a.mask_sparse += b.mask_sparse;
            a.mask_full += b.mask_full;
            a.no_i0 += b.no_i0;
            a.with_ref += b.with_ref;
            a.with_id += b.with_id;
            a.with_ammo += b.with_ammo;
            out.creations.records.extend(stream.records);
        }
    }
    out.stats.anchors = out.creations.stats.anchors;
    out.stats.accepted = out.creations.stats.accepted;
    (out.placements, out.stats.confirmed) = confirm_equipment_placements(
        &out.creations.records,
        &spans,
        equipment_position_epsilon(map),
    );
    out.stats.placements = out.placements.len();
    for p in &out.placements {
        *out.stats.by_id.entry(p.global_id).or_default() += 1;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_equipment_calibration_profile() {
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/equipment-calibration-profile-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 256);
        for (i, row) in rows.into_iter().enumerate() {
            let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
            let mut position = map.position_encoding();
            position.index_bits = row["index"].as_u64().unwrap() as usize;
            position.world_axis_bits = Some(serde_json::from_value(row["axes"].clone()).unwrap());
            position.region_axis_bits = [(
                row["region"].as_u64().unwrap() as u32,
                position.world_axis_bits.unwrap(),
            )]
            .into();
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(j, hex)| {
                    let hex = hex.as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|k| u8::from_str_radix(&hex[k..k + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: j as i32 + 1,
                            chunk_type: 0,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let spans: EquipmentLifeSpans = row["spans"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    (
                        serde_json::from_value(r["key"].clone()).unwrap(),
                        serde_json::from_value(r["spans"].clone()).unwrap(),
                    )
                })
                .collect();
            let components = row["components"].as_u64().unwrap() as usize;
            let band = if row["empty_band"].as_bool().unwrap() {
                Default::default()
            } else {
                [512].into()
            };
            let actual = calibrate_equipment_mpp_with_position(
                &chunks, &map, &band, components, &spans, &position,
            );
            let expected: EquipmentMppCalibration =
                serde_json::from_value(row["calibration"].clone()).unwrap();
            assert_eq!(actual, expected, "calibration {i}");
            let tracks = super::super::scan_world_object_tracks_for_band_with_position(
                &chunks, &map, &band, &position,
            )
            .unwrap();
            let expected: Vec<WorldObjectTrack> =
                serde_json::from_value(row["tracks"].clone()).unwrap();
            assert_eq!(tracks.tracks, expected, "tracks {i}");
            assert_eq!(
                serde_json::from_value::<EquipmentMppCalibration>(
                    serde_json::to_value(&actual).unwrap()
                )
                .unwrap(),
                actual
            );
            if i == 0 {
                let mut legacy_map = map.clone();
                legacy_map.region_index_bits = position.index_bits;
                legacy_map.axis_widths = position.world_axis_bits.unwrap();
                legacy_map.region = 0;
                assert_eq!(actual.agree, 7);
                assert_eq!(
                    calibrate_equipment_mpp(&chunks, &legacy_map, &[512].into(), 1, &spans).agree,
                    0
                );
            }
        }
    }
    #[test]
    fn native_equipment_lifetimes_and_placements() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/equipment-placements-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.as_array().unwrap().iter().enumerate() {
            let mut cal = EquipmentMppCalibration {
                by_widths: serde_json::from_value(row["scores"].clone()).unwrap(),
                ..Default::default()
            };
            assert_eq!(
                verdict(&mut cal),
                row["conclusive"].as_bool().unwrap(),
                "verdict {i}"
            );
            assert_eq!(
                cal.widths,
                serde_json::from_value(row["verdict"]["Widths"].clone()).unwrap()
            );
            assert_eq!(
                cal.runner,
                serde_json::from_value(row["verdict"]["Runner"].clone()).unwrap()
            );
            assert_eq!(
                cal.agree,
                row["verdict"]["Agree"].as_u64().unwrap() as usize
            );
            assert_eq!(
                cal.runner_agree,
                row["verdict"]["RunnerAgree"].as_u64().unwrap() as usize
            );
            let tracks: Vec<WorldObjectTrack> =
                serde_json::from_value(row["tracks"].clone()).unwrap();
            let spans = equipment_life_spans(&tracks);
            let expected: EquipmentLifeSpans = row["spans"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    (
                        serde_json::from_value(r["key"].clone()).unwrap(),
                        serde_json::from_value(r["spans"].clone()).unwrap(),
                    )
                })
                .collect();
            assert_eq!(spans, expected, "spans {i}");
            let mut c = row["creations"].clone();
            for c in c.as_array_mut().unwrap() {
                if c["Mask"].is_null() {
                    c["Mask"] = serde_json::json!([]);
                }
            }
            let creations: Vec<EquipmentCreation> = serde_json::from_value(c).unwrap();
            let (out, confirmed) =
                confirm_equipment_placements(&creations, &spans, [0.8, 1.2, 1.6]);
            let expected: Vec<EquipmentPlacement> =
                serde_json::from_value(row["placements"].clone()).unwrap();
            assert_eq!(
                confirmed,
                row["confirmed"].as_u64().unwrap() as usize,
                "confirmed {i}"
            );
            assert_eq!(out, expected, "placements {i}");
        }
    }
}
