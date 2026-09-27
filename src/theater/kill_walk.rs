//! Native kill-source record walk and credible-death selection.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillDeadRecord {
    pub time_ms: i64,
    pub packet: KillPacketIdentity,
    pub slot: u32,
    pub bit: i64,
    pub dead: ObjectDeadState,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillWalkResult {
    pub deaths: Vec<KillDeadRecord>,
    pub credible: Vec<KillDeadRecord>,
    pub biped_low: i64,
    pub biped_high: i64,
    pub located: usize,
    pub with_events: usize,
}
impl KillWalkResult {
    pub fn select_credible(&mut self, participants: usize) {
        // Append semantics mirror the native pass; orchestration invokes it once.
        self.credible.extend(
            self.deaths
                .iter()
                .filter(|d| {
                    i64::from(d.slot) >= self.biped_low
                        && i64::from(d.slot) <= self.biped_high
                        && d.dead.enum_a >= 0
                        && (d.dead.enum_a as usize) < participants
                        && d.dead.enum_b >= 0
                        && (d.dead.enum_b as usize) < participants
                        && d.dead.val_0c <= 9
                })
                .cloned(),
        );
    }
    pub fn candidates(&self) -> (Vec<KillSourceCandidate>, usize) {
        let missing_bits = self.credible.iter().filter(|d| d.bit < 0).count();
        let candidates = self
            .credible
            .iter()
            .map(|d| KillSourceCandidate {
                packet: Some(d.packet),
                time_ms: d.time_ms,
                bit: d.bit,
                tag: d.dead.src_tag0,
                victim: d.dead.enum_a,
                killer: d.dead.enum_b,
                category: i32::from(d.dead.val_0c),
            })
            .collect();
        (deduplicate_kill_source_candidates(candidates), missing_bits)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillWalkProfile {
    pub encoding: FrameEncoding,
    pub simulation_complete: bool,
}
#[derive(Debug, Clone, Copy)]
pub struct KillPacketWalkOptions {
    pub start: usize,
    pub views: usize,
    pub time_ms: i64,
    pub packet: KillPacketIdentity,
}
/// Decode one packet against a snapshot: temporary record bindings never escape.
pub fn walk_kill_packet(
    data: &[u8],
    world: &FilmWorld,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    options: KillPacketWalkOptions,
) -> Vec<KillDeadRecord> {
    let KillPacketWalkOptions {
        start,
        views,
        time_ms,
        packet,
    } = options;
    let records = walk_march_with_policy(
        data,
        start,
        registry,
        &profile.encoding,
        world,
        MarchWalkPolicy {
            views,
            generation_strict: true,
            simulation_complete: profile.simulation_complete,
        },
    );
    let mut out = Vec::new();
    for r in records {
        if r.stop != EntityViewStop::Complete {
            continue;
        }
        let Some(attempt) = r
            .attempts
            .iter()
            .rev()
            .find(|a| a.span.name == "object-dead-state-component" && a.status == Some(true))
        else {
            continue;
        };
        let Some(dead) =
            ObjectDeadState::from_fields(&r.fields[attempt.field_start..attempt.field_end])
                .filter(|d| d.mort)
        else {
            continue;
        };
        let bit = r
            .components
            .iter()
            .find(|c| c.name == "object-dead-state-component")
            .map_or(-1, |c| c.start_bit);
        out.push(KillDeadRecord {
            time_ms,
            packet,
            slot: r.header.id.unwrap() & 0x3fff_ffff,
            bit,
            dead,
        });
    }
    out
}
/// Walk all supplied replication packets chronologically, with native source
/// positions for packet identity and the earliest replication time as origin.
pub fn run_kill_record_walk(
    chunks: &[FilmChunkData],
    timeline: &mut KillTimeline,
    profile: &KillWalkProfile,
    views: usize,
    participants: usize,
) -> Result<KillWalkResult, DecodeError> {
    timeline.rewind();
    let (low, high) = timeline.biped_range();
    let registry = timeline.registry.clone();
    let mut out = KillWalkResult {
        deaths: Vec::new(),
        credible: Vec::new(),
        biped_low: low,
        biped_high: high,
        located: 0,
        with_events: 0,
    };
    let mut packets = Vec::new();
    for (chunk, c) in chunks.iter().enumerate() {
        for (index, p) in fire_events::native_chunk_packets(c).into_iter().enumerate() {
            if p.packet_type == 0 {
                packets.push((
                    p.timestamp_us,
                    KillPacketIdentity {
                        chunk: chunk as i64,
                        packet: index as i64,
                    },
                    &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                ));
            }
        }
    }
    native_sort::sort_by(&mut packets, |a, b| a.0.cmp(&b.0));
    let origin = packets
        .first()
        .ok_or(DecodeError::Missing("replication packets"))?
        .0;
    for (timestamp, packet, data) in packets {
        let world = timeline.advance_to(timestamp);
        let mut start = 2;
        if bits::Bits(data).read(1, 1) == Some(1) {
            out.with_events += 1;
            let Some(at) = locate_march_event_records_with_simulation_policy(
                data,
                &registry,
                &profile.encoding,
                world,
                true,
                profile.simulation_complete,
            ) else {
                continue;
            };
            out.located += 1;
            start = at;
        }
        out.deaths.extend(walk_kill_packet(
            data,
            world,
            &registry,
            profile,
            KillPacketWalkOptions {
                start,
                views,
                time_ms: ((timestamp - origin) / 1000) as i64,
                packet,
            },
        ));
        timeline.detach_snapshot_alias();
    }
    native_sort::sort_by(&mut out.deaths, |a, b| a.time_ms.cmp(&b.time_ms));
    out.select_credible(participants);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use std::{fs, io::Read, path::Path};
    #[test]
    #[ignore = "requires downloaded v41 captures; calibration and full record walks"]
    fn local_kill_record_walk() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kill-walk-v41.json.zlib")[..])
            .read_to_end(&mut json)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&json).unwrap();
        let mut failures = Vec::new();
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
            let mut timeline = KillTimeline::from_chunks(&chunks).unwrap();
            timeline.rewind();
            let catalog = film_map_catalog();
            let map = &catalog.maps[row["map"].as_str().unwrap()];
            let calibration = calibrate_kill_walk(&chunks, &mut timeline, 8, Some(map));
            println!("{folder}: {calibration}");
            let walk = run_kill_record_walk(
                &chunks,
                &mut timeline,
                &calibration.profile,
                8,
                row["participants"].as_u64().unwrap() as usize,
            )
            .unwrap();
            let (candidates, no_bit) = walk.candidates();
            let mut cal = serde_json::to_value(&calibration).unwrap();
            cal.as_object_mut().unwrap().remove("profile");
            // Compare the complete returned native profile independently of
            // its Rust field names, then compare every original oracle field.
            assert_eq!(
                calibration.native_profile.as_ref().unwrap(),
                &native_scan_profile::tests::profile(&row["calibration"]["native_profile"]),
                "{folder}: calibrated native profile"
            );
            let restored: KillCalibration =
                serde_json::from_value(serde_json::to_value(&calibration).unwrap()).unwrap();
            assert_eq!(restored, calibration, "{folder}: calibration export");
            cal["native_profile"] = row["calibration"]["native_profile"].clone();
            let actual = serde_json::json!({"folder":folder,"map":row["map"],"participants":row["participants"],"calibration":cal,"calibration_string":calibration.to_string(),"walk":walk,"candidates":candidates,"no_bit":no_bit});
            if actual != row {
                let name = folder.replace('/', "-");
                fs::write(
                    format!("/private/tmp/halo-kill-walk-{name}-actual.json"),
                    serde_json::to_vec_pretty(&actual).unwrap(),
                )
                .unwrap();
                fs::write(
                    format!("/private/tmp/halo-kill-walk-{name}-expected.json"),
                    serde_json::to_vec_pretty(&row).unwrap(),
                )
                .unwrap();
                for (key, value) in row.as_object().unwrap() {
                    if actual[key] != *value {
                        failures.push(format!("{folder}/{key}"));
                    }
                }
            } else {
                println!("kill walk matches {folder}");
            }
        }
        assert!(failures.is_empty(), "mismatches: {failures:?}");
    }
}
