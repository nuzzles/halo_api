//! Native weapon-hit event readers, including their explicit zero-tail provenance.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponDamageRead {
    /// Loaded packet provenance. None for payload-only reads and older exports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<FilmPacket>,
    /// Ordinal among all packets in the chunk, before filtering by packet type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub damage: WeaponDamage,
    /// Second five-bit scalar from the damage body.
    pub secondary_mag_raw: u64,
    /// Body victim reference, separate from the packet-header victim.
    pub body_victim_idx: Option<u64>,
    pub end_bit: usize,
    pub padding_bits: usize,
}
/// Decode a complete type-36 head. Unreadable identity pairs remain explicit.
pub fn decode_weapon_shot(payload: &[u8], timestamp_us: u64) -> Option<WeaponShot> {
    let fire = decode_fire_event(payload)?;
    let mut shot = WeaponShot {
        timestamp_us,
        ..Default::default()
    };
    if fire.unit.present && fire.has_shooter {
        shot.attacker = u64::from(fire.unit.slot - 512);
        shot.weapon_id = fire.weapon_id;
        shot.film_index = i64::from(fire.film_index);
        shot.has_pair = true;
    }
    Some(shot)
}
/// Read the first damage_aftermath exactly as the native statistics scanner.
/// Synthetic tail bits are retained separately rather than hidden as physical data.
pub fn decode_weapon_damage(payload: &[u8], timestamp_us: u64) -> Option<WeaponDamageRead> {
    native_weapon_damage::decode_with_fields(payload, timestamp_us, false).read
}

/// A loaded shot read, including shots whose identity pair could not be read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponShotRead {
    pub source: FilmPacket,
    /// Ordinal before filtering by packet type.
    pub packet_index: usize,
    pub shot: WeaponShot,
}
pub fn scan_weapon_shot_reads(
    chunks: &[FilmChunkData],
) -> Result<Vec<WeaponShotRead>, DecodeError> {
    Ok(scan_weapon_shots_selected(
        fire_events::native_chunk_prefix(chunks)?,
    ))
}
pub fn scan_weapon_shots(chunks: &[FilmChunkData]) -> Result<Vec<WeaponShot>, DecodeError> {
    Ok(scan_weapon_shot_reads(chunks)?
        .into_iter()
        .map(|r| r.shot)
        .collect())
}
/// Native ScanFilmWeaponShots 1..=last_chunk contract. Missing requested chunks
/// are errors, regardless of later available chunks; nonpositive limits are empty.
pub fn scan_weapon_shots_through(
    chunks: &[FilmChunkData],
    last_chunk: i32,
) -> Result<Vec<WeaponShot>, DecodeError> {
    Ok(scan_weapon_shot_reads_through(chunks, last_chunk)?
        .into_iter()
        .map(|r| r.shot)
        .collect())
}
/// Source-retaining counterpart of `scan_weapon_shots_through`.
pub fn scan_weapon_shot_reads_through(
    chunks: &[FilmChunkData],
    last_chunk: i32,
) -> Result<Vec<WeaponShotRead>, DecodeError> {
    Ok(scan_weapon_shots_selected(weapon_chunk_range(
        chunks, last_chunk,
    )?))
}
fn weapon_chunk_range(
    chunks: &[FilmChunkData],
    last: i32,
) -> Result<Vec<&FilmChunkData>, DecodeError> {
    (1..=last)
        .map(|index| {
            chunks
                .iter()
                .find(|c| c.metadata.index == index)
                .ok_or(DecodeError::Missing("requested weapon scan chunk"))
        })
        .collect()
}
fn scan_weapon_shots_selected(chunks: Vec<&FilmChunkData>) -> Vec<WeaponShotRead> {
    let mut out = Vec::new();
    for c in chunks {
        for (packet_index, p) in fire_events::native_chunk_packets(c).into_iter().enumerate() {
            if p.packet_type == 0
                && let Some(shot) = decode_weapon_shot(
                    &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                    p.timestamp_us,
                )
            {
                out.push(WeaponShotRead {
                    source: p,
                    packet_index,
                    shot,
                });
            }
        }
    }
    out
}
/// Reconstruct each chunk's native final bindings before attributing its damages.
/// The base is selected from both victim and responsible biped landings.
pub fn scan_weapon_damages(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
) -> Result<(Vec<WeaponDamageRead>, i64), DecodeError> {
    if registry.major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(
            registry.major_version as i32,
        ));
    }
    scan_weapon_damages_selected(fire_events::native_chunk_prefix(chunks)?, registry)
}
/// Native ScanFilmWeaponDamages explicit range, retaining full damage reads.
pub fn scan_weapon_damages_through(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    last_chunk: i32,
) -> Result<(Vec<WeaponDamageRead>, i64), DecodeError> {
    scan_weapon_damages_selected(weapon_chunk_range(chunks, last_chunk)?, registry)
}
fn scan_weapon_damages_selected(
    chunks: Vec<&FilmChunkData>,
    registry: &FilmRegistry,
) -> Result<(Vec<WeaponDamageRead>, i64), DecodeError> {
    if registry.major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(
            registry.major_version as i32,
        ));
    }
    let encoding = FrameEncoding {
        keyframe_layout: Default::default(),
        keyframe_simulation_complete: None,
        native_id_low_bits: None,
        component_widths: Default::default(),
        new_record: Default::default(),
        position_capture: None,
        ids: RecordIdLayout {
            low_bits: 13,
            base: 0,
        },
        mpp_widths: [9, 5],
        position: None,
        extra_fields: false,
        corruption_check: false,
    };
    let mut hits = [0_u64; 15];
    let mut out = Vec::new();
    for c in chunks {
        let packets = fire_events::native_chunk_packets(c);
        let payload = |p: &FilmPacket| &c.data[p.payload_offset..p.payload_offset + p.payload_size];
        let mut world = FilmWorld::default();
        for p in packets.iter().filter(|p| p.packet_type == 2) {
            for a in recover_keyframe_anchors(payload(p)) {
                world.bind_full(a.id, a.archetype);
            }
        }
        for p in packets.iter().filter(|p| p.packet_type == 0) {
            let pay = payload(p);
            if pay.first().is_some_and(|b| b & 0x40 == 0) {
                let _ = decode_production_frame(pay, 2, registry, &encoding, &mut world);
            }
        }
        for (packet_index, p) in packets
            .iter()
            .enumerate()
            .filter(|(_, p)| p.packet_type == 0)
        {
            if let Some(mut read) = decode_weapon_damage(payload(p), p.timestamp_us) {
                read.source = Some(*p);
                read.packet_index = Some(packet_index);
                for (i, base) in WEAPON_HIT_SLOT_BASES.iter().enumerate() {
                    for idx in [read.damage.victim_idx, read.damage.responsible_idx] {
                        let slot = base + idx;
                        if idx >= 0
                            && (0..8192).contains(&slot)
                            && world.archetype(slot as u32) == Some(35)
                        {
                            hits[i] += 1;
                        }
                    }
                }
                out.push(read);
            }
        }
    }
    let mut best = 0;
    for i in 1..hits.len() {
        if hits[i] > hits[best] {
            best = i;
        }
    }
    Ok((out, WEAPON_HIT_SLOT_BASES[best]))
}

/// Build unsmoothed biped position tracks for the first `n` chunks. Missing
/// chunks are tolerated for explicit positive limits. A zero (or negative)
/// limit selects the contiguous numbered prefix, as the native automatic scan
/// does for zero. The following keyframe contributes to the slot band,
/// matching native allocation-gap coverage; it contributes no position samples.
pub fn build_weapon_hit_tracks(
    chunks: &[FilmChunkData],
    map: &FilmMapBounds,
    n: i32,
) -> Result<WeaponHitTracks, DecodeError> {
    // Validate the map even when no film data is available.
    scan_biped_position_records(&[], [0, 0], map, true)?;
    let mut selected: Vec<_> = chunks
        .iter()
        .filter(|c| c.metadata.index >= 1 && (n <= 0 || c.metadata.index <= n))
        .collect();
    // The native directory adapter sorts filenames numerically before automatic
    // FilmChunkNumbers selection stops at the first missing number.
    selected.sort_by_key(|c| c.metadata.index);
    if n <= 0 {
        let prefix = selected
            .iter()
            .enumerate()
            .take_while(|(i, c)| i64::from(c.metadata.index) == *i as i64 + 1)
            .count();
        selected.truncate(prefix);
    }
    if selected.is_empty() {
        return Err(DecodeError::Missing("readable film chunks"));
    }
    let last = if n > 0 {
        n
    } else {
        selected.iter().map(|c| c.metadata.index).max().unwrap()
    };
    let mut band: Option<[u32; 2]> = None;
    for c in chunks
        .iter()
        .filter(|c| c.metadata.index >= 1 && c.metadata.index <= last.saturating_add(1))
    {
        if let Some(p) = fire_events::native_chunk_packets(c)
            .into_iter()
            .find(|p| p.packet_type == 2)
        {
            for a in recover_keyframe_anchors(
                &c.data[p.payload_offset..p.payload_offset + p.payload_size],
            )
            .into_iter()
            .filter(|a| a.archetype == 35)
            {
                let slot = a.id & 0x3fff_ffff;
                band = Some(band.map_or([slot, slot], |[lo, hi]| [lo.min(slot), hi.max(slot)]));
            }
        }
    }
    let band = band.ok_or(DecodeError::Missing("biped keyframe slot band"))?;
    let mut tracks = WeaponHitTracks::new();
    for c in selected {
        for p in fire_events::native_chunk_packets(c)
            .into_iter()
            .filter(|p| p.packet_type == 0)
        {
            for r in scan_biped_position_records(
                &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                band,
                map,
                true,
            )? {
                if r.quantized
                    .iter()
                    .zip(map.axis_widths)
                    .any(|(&q, w)| q == 0 || q == (1u32 << w) - 1)
                {
                    continue;
                }
                tracks
                    .entry(r.slot)
                    .or_default()
                    .push(WeaponHitPositionSample {
                        timestamp_us: p.timestamp_us,
                        position: r.world,
                    });
            }
        }
    }
    for t in tracks.values_mut() {
        native_sort::sort_by(t, |a, b| a.timestamp_us.cmp(&b.timestamp_us));
    }
    Ok(tracks)
}

/// Complete weapon-hit scan output, retaining source events and optional
/// distance evidence alongside the accuracy summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmWeaponHits {
    /// Complete loaded shot provenance. None identifies older exports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot_reads: Option<Vec<WeaponShotRead>>,
    pub shots: Vec<WeaponShot>,
    pub damages: Vec<WeaponDamageRead>,
    pub biped_landing_base: i64,
    pub distance_base: Option<i64>,
    pub tracks: Option<WeaponHitTracks>,
    pub distance_error: Option<String>,
    pub stats: Vec<WeaponHitStats>,
}
/// Scan direct weapon hits. Missing map/position evidence preserves counted hits
/// and leaves distance buckets empty, matching the native best-effort contract.
pub fn scan_film_weapon_hits(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    map: Option<&FilmMapBounds>,
) -> Result<FilmWeaponHits, DecodeError> {
    let shot_reads = scan_weapon_shot_reads(chunks)?;
    let shots: Vec<_> = shot_reads.iter().map(|r| r.shot.clone()).collect();
    let (damages, biped_landing_base) = scan_weapon_damages(chunks, registry)?;
    let values: Vec<_> = damages.iter().map(|r| r.damage.clone()).collect();
    let n = fire_events::native_chunk_prefix(chunks)?
        .last()
        .unwrap()
        .metadata
        .index;
    let (tracks, distance_error) = match map.map(|m| build_weapon_hit_tracks(chunks, m, n)) {
        Some(Ok(tracks)) => (Some(tracks), None),
        Some(Err(error)) => (None, Some(error.to_string())),
        None => (None, None),
    };
    let distance_base = tracks
        .as_ref()
        .map(|t| resolve_hit_distance_base(&values, t));
    let distance = |d: &WeaponDamage| weapon_hit_distance(tracks.as_ref()?, distance_base?, d);
    let stats = pair_weapon_hits(&shots, &values, WEAPON_HIT_PAIR_WINDOW_US, Some(&distance));
    Ok(FilmWeaponHits {
        shot_reads: Some(shot_reads),
        shots,
        damages,
        biped_landing_base,
        distance_base,
        tracks,
        distance_error,
        stats,
    })
}

#[allow(dead_code)]
impl LegacyFilm {
    /// Scan and retain native direct weapon-hit statistics for portable export.
    /// Supply the same chunks used to construct this LegacyFilm. Resolved sampling
    /// precision is preferred; absent map evidence still preserves hit counts.
    /// A failed scan leaves previously retained results unchanged and records
    /// weapon_hits_error. Success replaces the result and clears that error.
    pub fn retain_weapon_hits(&mut self, chunks: &[FilmChunkData]) -> Result<(), DecodeError> {
        let map = self
            .scan_precision
            .as_ref()
            .and_then(|p| p.sampling_map.as_ref())
            .or_else(|| self.profile.as_ref().and_then(|p| p.map.as_ref()));
        match scan_film_weapon_hits(chunks, &self.registry, map) {
            Ok(hits) => {
                self.weapon_hits = Some(hits);
                self.weapon_hits_error = None;
                Ok(())
            }
            Err(error) => {
                self.weapon_hits_error = Some(error.to_string());
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    fn assert_shot_sources(chunks: &[FilmChunkData], reads: &[WeaponShotRead]) {
        for read in reads {
            let chunk = chunks
                .iter()
                .find(|c| c.metadata.index == read.source.chunk_index)
                .unwrap();
            let packets = fire_events::native_chunk_packets(chunk);
            assert_eq!(packets[read.packet_index], read.source);
            let payload = &chunk.data
                [read.source.payload_offset..read.source.payload_offset + read.source.payload_size];
            assert_eq!(
                decode_weapon_shot(payload, read.source.timestamp_us).as_ref(),
                Some(&read.shot)
            );
            assert_eq!(
                serde_json::from_value::<WeaponShotRead>(serde_json::to_value(read).unwrap())
                    .unwrap(),
                *read
            );
        }
    }
    fn assert_damage_sources(chunks: &[FilmChunkData], reads: &[WeaponDamageRead]) {
        for read in reads {
            let source = read.source.as_ref().unwrap();
            let chunk = chunks
                .iter()
                .find(|c| c.metadata.index == source.chunk_index)
                .unwrap();
            let packets = fire_events::native_chunk_packets(chunk);
            assert_eq!(&packets[read.packet_index.unwrap()], source);
            let payload =
                &chunk.data[source.payload_offset..source.payload_offset + source.payload_size];
            let mut direct = decode_weapon_damage(payload, source.timestamp_us).unwrap();
            assert!(direct.source.is_none() && direct.packet_index.is_none());
            direct.source = Some(*source);
            direct.packet_index = read.packet_index;
            assert_eq!(&direct, read);
            let json = serde_json::to_value(read).unwrap();
            assert_eq!(
                serde_json::from_value::<WeaponDamageRead>(json.clone()).unwrap(),
                *read
            );
            let mut legacy = json;
            legacy.as_object_mut().unwrap().remove("source");
            legacy.as_object_mut().unwrap().remove("packet_index");
            let legacy: WeaponDamageRead = serde_json::from_value(legacy).unwrap();
            assert!(legacy.source.is_none() && legacy.packet_index.is_none());
            let mut expected = read.clone();
            expected.source = None;
            expected.packet_index = None;
            assert_eq!(legacy, expected);
        }
    }

    #[test]
    fn native_weapon_explicit_ranges() {
        use crate::clients::hi::models::FilmChunk;
        let rows: Vec<serde_json::Value> =
            serde_json::from_slice(include_bytes!("fixtures/weapon-range-v41.json")).unwrap();
        let registry = FilmRegistry {
            archetypes: vec![],
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        for (case, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let hex = c["hex"].as_str().unwrap();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 2,
                            start_time_offset_ms: 0,
                            duration_ms: 1,
                            size: 0,
                            file_relative_path: String::new(),
                        },
                        data: (0..hex.len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                            .collect(),
                    }
                })
                .collect();
            let last = row["last"].as_i64().unwrap() as i32;
            let shots = scan_weapon_shots_through(&chunks, last);
            assert_eq!(
                shots.is_err(),
                row["shot_error"].as_bool().unwrap(),
                "shots {case}"
            );
            if let Ok(shots) = shots {
                let reads = scan_weapon_shot_reads_through(&chunks, last).unwrap();
                assert_shot_sources(&chunks, &reads);
                assert_eq!(reads.into_iter().map(|r| r.shot).collect::<Vec<_>>(), shots);
                assert_eq!(
                    serde_json::to_value(shots).unwrap(),
                    row["shots"]
                        .as_array()
                        .map_or_else(|| serde_json::json!([]), |a| serde_json::json!(a)),
                    "shots {case}"
                );
            }
            let damages = scan_weapon_damages_through(&chunks, &registry, last);
            assert_eq!(
                damages.is_err(),
                row["damage_error"].as_bool().unwrap(),
                "damages {case}"
            );
            if let Ok((damages, base)) = damages {
                assert_damage_sources(&chunks, &damages);
                let values: Vec<_> = damages.into_iter().map(|d| d.damage).collect();
                let expected: Vec<WeaponDamage> = serde_json::from_value(
                    row["damages"]
                        .as_array()
                        .map_or_else(|| serde_json::json!([]), |a| serde_json::json!(a)),
                )
                .unwrap();
                assert_eq!(values, expected, "damages {case}");
                assert_eq!(base, row["base"].as_i64().unwrap(), "base {case}");
            }
        }
    }

    #[test]
    fn native_weapon_track_ranges() {
        use crate::clients::hi::models::FilmChunk;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/weapon-track-range-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let hex = fixture["chunk_hex"].as_str().unwrap();
        let data: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let map: FilmMapBounds = serde_json::from_value(fixture["map"].clone()).unwrap();
        let cases = fixture["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 16);
        for (i, case) in cases.iter().enumerate() {
            let chunks: Vec<_> = case["indices"]
                .as_array()
                .unwrap()
                .iter()
                .map(|index| FilmChunkData {
                    metadata: FilmChunk {
                        index: index.as_i64().unwrap() as i32,
                        chunk_type: 2,
                        start_time_offset_ms: 0,
                        duration_ms: 0,
                        size: data.len() as i64,
                        file_relative_path: String::new(),
                    },
                    data: data.clone(),
                })
                .collect();
            let actual =
                build_weapon_hit_tracks(&chunks, &map, case["last"].as_i64().unwrap() as i32);
            assert_eq!(
                actual.is_err(),
                case["error"].as_bool().unwrap(),
                "error {i}"
            );
            if let Ok(actual) = actual {
                let expected: WeaponHitTracks =
                    serde_json::from_value(case["tracks"].clone()).unwrap();
                assert_eq!(actual, expected, "tracks {i}");
            }
        }
    }

    #[test]
    fn native_weapon_hit_event_readers() {
        #[derive(Deserialize)]
        struct Case {
            damage_bytes: String,
            damage: WeaponDamage,
            secondary: u64,
            body_victim: Option<u64>,
            end: usize,
            shot_bytes: String,
            shot: Option<WeaponShot>,
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/weapon-hit-scan-d61443e-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 2048);
        let bytes = |s: &str| {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>()
        };
        let mut padded = 0;
        for (i, c) in cases.into_iter().enumerate() {
            let d = decode_weapon_damage(&bytes(&c.damage_bytes), i as u64).unwrap();
            assert_eq!(d.damage, c.damage, "damage {i}");
            assert_eq!(d.secondary_mag_raw, c.secondary, "secondary {i}");
            assert_eq!(d.body_victim_idx, c.body_victim, "body victim {i}");
            assert_eq!(d.end_bit, c.end, "end {i}");
            padded += usize::from(d.padding_bits > 0);
            assert_eq!(
                decode_weapon_shot(&bytes(&c.shot_bytes), i as u64),
                c.shot,
                "shot {i}"
            );
        }
        assert!(padded > 100);
        assert!(decode_weapon_damage(&[0xc0], 0).is_none());
        assert!(decode_weapon_damage(&[0xc0, 0x80], 0).is_none());
        assert!(decode_weapon_shot(&[0xd3; 20], 0).is_none());
    }
    #[test]
    fn missing_distance_preserves_aggregate_hits() {
        use crate::clients::hi::models::FilmChunk;
        let mut shot = vec![0; 15];
        shot[0] = 0xd2;
        shot[1] = 0x40;
        let mut damage = vec![0; 32];
        damage[0] = 0xc0;
        damage[1] = 0x30;
        let mut data = Vec::new();
        for payload in [shot, damage] {
            data.extend_from_slice(&[0; 4]);
            data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            data.extend_from_slice(&1000_u64.to_le_bytes());
            data.extend(payload);
        }
        let chunks = [FilmChunkData {
            metadata: FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 0,
                duration_ms: 1,
                size: 0,
                file_relative_path: String::new(),
            },
            data,
        }];
        let registry = FilmRegistry {
            archetypes: vec![],
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let output = scan_film_weapon_hits(&chunks, &registry, None).unwrap();
        assert_eq!(output.stats.len(), 1);
        assert_eq!(output.stats[0].hits, 1);
        assert_eq!(output.stats[0].dist_buckets, [0; 7]);
        assert!(output.tracks.is_none());
        assert!(output.distance_error.is_none());
        let reads = output.shot_reads.as_ref().unwrap();
        assert_eq!(reads.len(), 1);
        assert_shot_sources(&chunks, reads);
        assert_eq!(reads[0].shot, output.shots[0]);
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(include_bytes!("fixtures/bootstrap-v41.zlib").as_slice())
            .read_to_end(&mut bootstrap)
            .unwrap();
        let mut film_chunks = vec![FilmChunkData {
            metadata: FilmChunk {
                index: 0,
                chunk_type: 1,
                start_time_offset_ms: 0,
                duration_ms: 1,
                size: bootstrap.len() as i64,
                file_relative_path: String::new(),
            },
            data: bootstrap,
        }];
        film_chunks.extend(chunks.iter().cloned());
        let mut film = LegacyFilm::try_from_chunks(&film_chunks, DecodeOptions::v41()).unwrap();
        assert!(film.weapon_hits_error.is_none());
        let retained = film.weapon_hits.as_ref().unwrap();
        assert_eq!(retained.shot_reads, output.shot_reads);
        let mut json = serde_json::to_value(&film).unwrap();
        let restored: LegacyFilm = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(restored.weapon_hits, film.weapon_hits);
        json["weapon_hits"]
            .as_object_mut()
            .unwrap()
            .remove("shot_reads");
        let legacy: LegacyFilm = serde_json::from_value(json).unwrap();
        let mut expected = retained.clone();
        expected.shot_reads = None;
        assert_eq!(legacy.weapon_hits, Some(expected));
        let retained = film.weapon_hits.clone();
        assert!(film.retain_weapon_hits(&[]).is_err());
        assert_eq!(film.weapon_hits, retained);
        assert!(film.weapon_hits_error.is_some());
        let failed: LegacyFilm =
            serde_json::from_value(serde_json::to_value(&film).unwrap()).unwrap();
        assert_eq!(failed.weapon_hits_error, film.weapon_hits_error);
        film.retain_weapon_hits(&film_chunks).unwrap();
        assert!(film.weapon_hits_error.is_none());
        assert_eq!(film.weapon_hits, retained);
        let mut old = serde_json::to_value(&film).unwrap();
        old.as_object_mut().unwrap().remove("weapon_hits");
        old.as_object_mut().unwrap().remove("weapon_hits_error");
        let old: LegacyFilm = serde_json::from_value(old).unwrap();
        assert!(old.weapon_hits.is_none() && old.weapon_hits_error.is_none());
        let mut missing_prefix = film_chunks.clone();
        missing_prefix[1].metadata.index = 2;
        let partial = LegacyFilm::try_from_chunks(&missing_prefix, DecodeOptions::v41()).unwrap();
        assert!(partial.weapon_hits.is_none());
        assert!(partial.weapon_hits_error.is_some());
        assert_eq!(partial.registry, film.registry);
        assert!(!partial.packets.is_empty());
        let map = FilmMapBounds {
            module: String::new(),
            min: [0.; 3],
            max: [1.; 3],
            axis_widths: [0, 12, 12],
            region: 0,
            region_index_bits: 1,
        };
        let refused = scan_film_weapon_hits(&chunks, &registry, Some(&map)).unwrap();
        assert_eq!(refused.stats, output.stats);
        assert!(refused.distance_error.is_some());
        assert!(refused.distance_base.is_none());
    }
    #[test]
    #[ignore = "requires six downloaded v41 films"]
    fn local_weapon_hit_corpus() {
        compare_weapon_hit_corpus(0);
    }
    #[test]
    #[ignore = "requires six downloaded v41 films"]
    fn local_weapon_hit_tracks() {
        compare_weapon_hit_corpus(1);
    }
    #[test]
    #[ignore = "requires six downloaded v41 films"]
    fn local_weapon_hit_aggregate() {
        compare_weapon_hit_corpus(2);
    }
    #[test]
    #[ignore = "requires six downloaded v41 films"]
    fn local_weapon_hit_prefix_tracks() {
        compare_weapon_hit_corpus(3);
    }
    #[test]
    #[ignore = "requires six downloaded v41 films; verifies automatic Film weapon-hit retention"]
    fn local_weapon_hit_film_retention() {
        compare_weapon_hit_corpus(4);
    }

    #[test]
    #[ignore = "requires six downloaded v41 films; verifies automatic map Film weapon-hit retention"]
    fn local_weapon_hit_film_map_retention() {
        compare_weapon_hit_corpus(5);
    }

    fn compare_weapon_hit_corpus(mode: u8) {
        use crate::clients::hi::models::FilmChunk;
        use std::{fs, path::Path};
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/weapon-hit-corpus-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for row in rows {
            let folder = row["folder"].as_str().unwrap();
            let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
            let meta: serde_json::Value =
                serde_json::from_slice(&fs::read(dir.join("film.json")).unwrap()).unwrap();
            let chunks: Vec<_> = meta["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| c["index"].as_i64().unwrap() <= row["n"].as_i64().unwrap())
                .map(|c| FilmChunkData {
                    metadata: FilmChunk {
                        index: c["index"].as_i64().unwrap() as i32,
                        chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                        start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                        duration_ms: c["duration_ms"].as_i64().unwrap(),
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: fs::read(dir.join(c["file"].as_str().unwrap())).unwrap(),
                })
                .collect();
            if mode == 4 || mode == 5 {
                let expected_map: FilmMapBounds =
                    serde_json::from_value(row["map"].clone()).unwrap();
                let film = if mode == 5 {
                    LegacyFilm::try_from_chunks_with_map_bounds(
                        &chunks,
                        DecodeOptions::v41(),
                        &expected_map.module,
                        expected_map.clone(),
                        RecordIdLayout {
                            low_bits: 13,
                            base: 0,
                        },
                    )
                    .unwrap()
                } else {
                    LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap()
                };
                assert!(film.weapon_hits_error.is_none(), "{folder}");
                let output = film.weapon_hits.as_ref().unwrap();
                let shots: Vec<WeaponShot> = serde_json::from_value(row["shots"].clone()).unwrap();
                let damages: Vec<WeaponDamage> =
                    serde_json::from_value(row["damages"].clone()).unwrap();
                let stats: Vec<WeaponHitStats> = serde_json::from_value(
                    row[if mode == 5 { "stats" } else { "without" }].clone(),
                )
                .unwrap();
                assert_eq!(output.shots, shots, "shots {folder}");
                assert_eq!(
                    output
                        .shot_reads
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|r| r.shot.clone())
                        .collect::<Vec<_>>(),
                    shots
                );
                assert_eq!(
                    output
                        .damages
                        .iter()
                        .map(|r| r.damage.clone())
                        .collect::<Vec<_>>(),
                    damages,
                    "damages {folder}"
                );
                assert_eq!(
                    output.biped_landing_base,
                    row["base"].as_i64().unwrap(),
                    "base {folder}"
                );
                assert_eq!(output.stats, stats, "stats {folder}");
                assert_shot_sources(&chunks, output.shot_reads.as_ref().unwrap());
                assert_damage_sources(&chunks, &output.damages);
                if mode == 5 {
                    assert_eq!(
                        film.scan_precision.as_ref().unwrap().sampling_map.as_ref(),
                        Some(&expected_map),
                        "native sampling context {folder}"
                    );
                    assert_eq!(
                        output.distance_error.is_some(),
                        row["track_error"].as_bool().unwrap(),
                        "distance error {folder}"
                    );
                    assert_eq!(
                        output.distance_base,
                        row["distance_base"].as_i64(),
                        "distance base {folder}"
                    );
                    let tracks: WeaponHitTracks =
                        serde_json::from_value(row["tracks"].clone()).unwrap();
                    assert_eq!(output.tracks.as_ref(), Some(&tracks), "all tracks {folder}");
                    let restored: FilmWeaponHits =
                        serde_json::from_slice(&serde_json::to_vec(output).unwrap()).unwrap();
                    assert_eq!(&restored, output, "map hit export {folder}");
                    eprintln!(
                        "automatic map Film weapon-hit retention matches {folder}: {} track points",
                        tracks.values().map(Vec::len).sum::<usize>()
                    );
                } else {
                    assert!(
                        output.tracks.is_none()
                            && output.distance_base.is_none()
                            && output.distance_error.is_none()
                    );
                    let restored: LegacyFilm =
                        serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
                    assert_eq!(restored.weapon_hits, film.weapon_hits, "roundtrip {folder}");
                    eprintln!("automatic Film weapon-hit retention matches {folder}");
                }
                continue;
            }
            if mode == 3 {
                let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
                let tracks = build_weapon_hit_tracks(&chunks, &map, 1);
                assert_eq!(
                    tracks.is_err(),
                    row["prefix_error"].as_bool().unwrap(),
                    "prefix error {folder}"
                );
                if let Ok(tracks) = tracks {
                    let expected: WeaponHitTracks =
                        serde_json::from_value(row["prefix_tracks"].clone()).unwrap();
                    assert_eq!(tracks, expected, "prefix tracks {folder}");
                }
                eprintln!("weapon-hit prefix tracks match {folder}");
                continue;
            }
            if mode == 2 {
                let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
                let registry = parse_registry(&chunks[0].data).unwrap();
                let output = scan_film_weapon_hits(&chunks, &registry, Some(&map)).unwrap();
                assert!(output.distance_error.is_none(), "distance error {folder}");
                assert_eq!(
                    output.distance_base,
                    row["distance_base"].as_i64(),
                    "distance base {folder}"
                );
                let stats: Vec<WeaponHitStats> =
                    serde_json::from_value(row["stats"].clone()).unwrap();
                assert_eq!(output.stats, stats, "stats {folder}");
                let damages: Vec<_> = output.damages.iter().map(|r| r.damage.clone()).collect();
                let without: Vec<WeaponHitStats> =
                    serde_json::from_value(row["without"].clone()).unwrap();
                assert_eq!(
                    pair_weapon_hits(&output.shots, &damages, WEAPON_HIT_PAIR_WINDOW_US, None),
                    without,
                    "without distances {folder}"
                );
                eprintln!("weapon-hit aggregate matches {folder}");
                continue;
            }
            if mode == 1 {
                let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
                let tracks =
                    build_weapon_hit_tracks(&chunks, &map, row["n"].as_i64().unwrap() as i32);
                assert_eq!(
                    tracks.is_err(),
                    row["track_error"].as_bool().unwrap(),
                    "track error {folder}"
                );
                if let Ok(tracks) = tracks {
                    let expected: WeaponHitTracks =
                        serde_json::from_value(row["tracks"].clone()).unwrap();
                    assert_eq!(tracks, expected, "tracks {folder}");
                }
                eprintln!("weapon-hit tracks match {folder}");
                continue;
            }
            let registry = parse_registry(&chunks[0].data).unwrap();
            let shots: Vec<WeaponShot> = serde_json::from_value(row["shots"].clone()).unwrap();
            assert_eq!(scan_weapon_shots(&chunks).unwrap(), shots, "shots {folder}");
            let shot_reads = scan_weapon_shot_reads(&chunks).unwrap();
            assert_shot_sources(&chunks, &shot_reads);
            assert_eq!(
                shot_reads.into_iter().map(|r| r.shot).collect::<Vec<_>>(),
                shots
            );
            let (reads, base) = scan_weapon_damages(&chunks, &registry).unwrap();
            assert_damage_sources(&chunks, &reads);
            let damages: Vec<WeaponDamage> =
                serde_json::from_value(row["damages"].clone()).unwrap();
            assert_eq!(
                reads.into_iter().map(|r| r.damage).collect::<Vec<_>>(),
                damages,
                "damages {folder}"
            );
            assert_eq!(base, row["base"].as_i64().unwrap(), "base {folder}");
            eprintln!("weapon-hit scan matches {folder}");
        }
    }
}
