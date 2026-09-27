//! Team designators read from managed-player full-state keyframe records.
use super::{
    ComponentDecode, FilmRegistry, FrameEncoding, decode_default_state,
    decode_native_keyframe_record, recover_keyframe_anchors,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
const COMPONENT: &str = "managed-player-team-designator-component";
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerTeamScanReport {
    pub archetype_absent: bool,
    pub component_mismatch: bool,
    pub component: super::ReplayByteString,
    pub packets: i64,
    pub records: i64,
    pub read: i64,
    pub unreached: i64,
    pub out_of_domain_index: i64,
    pub out_of_domain_value: i64,
    pub entities: i64,
    pub entity_divergences: i64,
    pub index_divergences: i64,
    pub indices: i64,
    pub no_team: i64,
}
impl PlayerTeamScanReport {
    pub fn has_readings(&self) -> bool {
        !self.archetype_absent && !self.component_mismatch && self.read > 0
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmPlayerTeams {
    /// Speculative full-state attempts, including rejected team readings. Older
    /// exports without this trace cannot establish that no attempts occurred.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<PlayerTeamAttempt>,
    /// -1 is a recorded lack of team; a missing index is an unread assignment.
    pub by_index: BTreeMap<i64, i64>,
    pub report: PlayerTeamScanReport,
}
/// A candidate from the team scan, not a validated entity or gameplay event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTeamAttempt {
    /// Absent for payload-only scans.
    pub source: Option<super::FilmPacket>,
    /// Packet ordinal before filtering for keyframes.
    pub packet_index: Option<usize>,
    pub record_start_bit: usize,
    pub slot: u32,
    pub record: Option<super::KeyframeRecord>,
    /// Independent boundary-checked player index and raw team designator;
    /// domain and conflict admission happen after this observation.
    pub reading: Option<(i64, i64)>,
}
/// Re-derive the default-state boundary independently of the full-state walk.
/// Even a truncated later component/corruption trailer does not erase a read i0.
pub fn read_player_team_record(
    payload: &[u8],
    bit: usize,
    registry: &FilmRegistry,
    corruption: bool,
) -> Option<(i64, i64)> {
    read_player_team_record_with_context(payload, bit, registry, &team_encoding(corruption))
}

fn team_encoding(corruption_check: bool) -> FrameEncoding {
    FrameEncoding {
        keyframe_layout: Default::default(),
        keyframe_simulation_complete: None,
        native_id_low_bits: None,
        component_widths: Default::default(),
        new_record: Default::default(),
        position_capture: None,
        ids: super::RecordIdLayout {
            low_bits: 13,
            base: 0,
        },
        mpp_widths: [9, 5],
        position: None,
        extra_fields: false,
        corruption_check,
    }
}

/// Native team boundary check under the caller's complete keyframe context.
/// A calibrated component skip still permits reading the four recorded team bits
/// once the independent default-state boundary agrees with the component start.
pub fn read_player_team_record_with_context(
    payload: &[u8],
    bit: usize,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> Option<(i64, i64)> {
    let record = decode_native_keyframe_record(payload, bit, registry, encoding)?;
    team_reading_from_record(payload, bit, &record, encoding)
}

fn team_reading_from_record(
    payload: &[u8],
    bit: usize,
    record: &super::KeyframeRecord,
    encoding: &FrameEncoding,
) -> Option<(i64, i64)> {
    let first = record.components.first()?;
    if first.name != COMPONENT {
        return None;
    }
    let layout = encoding.keyframe_layout;
    let ComponentDecode::Decoded(default) = decode_default_state(
        payload,
        bit.checked_add(layout.header_bits)?
            .checked_add(layout.size_word_bits)?,
        9,
        encoding.mpp_widths,
        encoding.position.as_ref(),
    ) else {
        return None;
    };
    let expected = default
        .end_bit
        .checked_add(layout.size_word_bits as i64)?
        .checked_add(if encoding.corruption_check {
            layout.size_word_bits as i64
        } else {
            0
        })?;
    if expected != first.start_bit {
        return None;
    }
    let raw = super::bits::Cursor::new(payload, usize::try_from(first.start_bit).ok()?)?.read(4)?;
    let index = default
        .fields
        .iter()
        .find(|f| f.name == "player_index")?
        .raw;
    Some((index as i64, raw as i64))
}
/// Consume keyframe payloads in source order; conflicting indices are withheld.
pub fn collect_player_teams<'a>(
    payloads: impl IntoIterator<Item = &'a [u8]>,
    registry: Option<&FilmRegistry>,
    corruption: bool,
) -> FilmPlayerTeams {
    collect_player_teams_with_context(payloads, registry, &team_encoding(corruption))
}

/// Collect team readings with the same complete context used by the record reader.
pub fn collect_player_teams_with_context<'a>(
    payloads: impl IntoIterator<Item = &'a [u8]>,
    registry: Option<&FilmRegistry>,
    encoding: &FrameEncoding,
) -> FilmPlayerTeams {
    collect_player_teams_observed(payloads.into_iter().map(|p| (p, None)), registry, encoding)
}

fn collect_player_teams_observed<'a>(
    payloads: impl IntoIterator<Item = (&'a [u8], Option<(super::FilmPacket, usize)>)>,
    registry: Option<&FilmRegistry>,
    encoding: &FrameEncoding,
) -> FilmPlayerTeams {
    let mut out = FilmPlayerTeams::default();
    let Some(reg) = registry.filter(|r| r.archetype(9).is_some_and(|a| !a.components.is_empty()))
    else {
        out.report.archetype_absent = true;
        return out;
    };
    out.report.component = reg.archetype(9).unwrap().components[0].clone().into();
    if out.report.component.as_ref() != COMPONENT.as_bytes() {
        out.report.component_mismatch = true;
        return out;
    }
    let mut entities = BTreeMap::<u32, BTreeSet<i64>>::new();
    let mut indices = BTreeMap::<i64, BTreeSet<i64>>::new();
    for (payload, source) in payloads {
        let mut carrier = false;
        for anchor in recover_keyframe_anchors(payload)
            .into_iter()
            .filter(|a| a.archetype == 9)
        {
            if !carrier {
                out.report.packets = out.report.packets.wrapping_add(1);
                carrier = true;
            }
            out.report.records = out.report.records.wrapping_add(1);
            let record = decode_native_keyframe_record(payload, anchor.bit, reg, encoding);
            let reading = record
                .as_ref()
                .and_then(|r| team_reading_from_record(payload, anchor.bit, r, encoding));
            out.attempts.push(PlayerTeamAttempt {
                source: source.as_ref().map(|(p, _)| *p),
                packet_index: source.as_ref().map(|(_, i)| *i),
                record_start_bit: anchor.bit,
                slot: anchor.id & 0x3fff_ffff,
                record,
                reading,
            });
            match reading {
                None => out.report.unreached = out.report.unreached.wrapping_add(1),
                Some((index, _)) if !(0..32).contains(&index) => {
                    out.report.out_of_domain_index = out.report.out_of_domain_index.wrapping_add(1)
                }
                Some((_, raw)) if !(0..=9).contains(&raw) => {
                    out.report.out_of_domain_value = out.report.out_of_domain_value.wrapping_add(1)
                }
                Some((index, raw)) => {
                    out.report.read = out.report.read.wrapping_add(1);
                    entities
                        .entry(anchor.id & 0x3fff_ffff)
                        .or_default()
                        .insert(raw - 1);
                    indices.entry(index).or_default().insert(raw - 1);
                }
            }
        }
    }
    out.report.entities = entities.len() as i64;
    out.report.entity_divergences = entities.values().filter(|s| s.len() > 1).count() as i64;
    for (index, seen) in indices {
        if seen.len() != 1 {
            out.report.index_divergences = out.report.index_divergences.wrapping_add(1);
            continue;
        }
        let team = *seen.first().unwrap();
        out.report.no_team = out.report.no_team.wrapping_add(i64::from(team == -1));
        out.by_index.insert(index, team);
    }
    out.report.indices = out.by_index.len() as i64;
    out
}
pub fn scan_film_player_teams(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: Option<&FilmRegistry>,
    corruption: bool,
) -> FilmPlayerTeams {
    scan_film_player_teams_with_context(chunks, registry, &team_encoding(corruption))
}

pub fn scan_film_player_teams_with_context(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: Option<&FilmRegistry>,
    encoding: &FrameEncoding,
) -> FilmPlayerTeams {
    collect_player_teams_observed(
        chunks
            .iter()
            .filter(|c| c.metadata.chunk_type != 0)
            .flat_map(|c| {
                super::fire_events::native_chunk_packets(c)
                    .into_iter()
                    .enumerate()
                    .filter(|(_, p)| p.packet_type == 2)
                    .map(move |(i, p)| {
                        (
                            &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                            Some((p, i)),
                        )
                    })
            }),
        registry,
        encoding,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Reading {
        bit: usize,
        result: Option<(i64, i64)>,
    }
    #[derive(Deserialize)]
    struct Packet {
        hex: String,
        reads: Vec<Reading>,
    }
    #[derive(Deserialize)]
    struct Case {
        mode: usize,
        #[serde(default)]
        names: Option<Vec<String>>,
        emissions: Vec<super::super::FilmComponentObservation>,
        attempts: Vec<serde_json::Value>,
        check: bool,
        #[serde(default)]
        keyframe_layout: super::super::KeyframeLayout,
        #[serde(default)]
        component_widths: super::super::ComponentWidthOverrides,
        packets: Vec<Packet>,
        output: FilmPlayerTeams,
        read: bool,
    }
    #[test]
    fn native_keyframe_team_readings() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/player-teams-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let mut reg = FilmRegistry {
                archetypes: (0..10)
                    .map(|index| super::super::FilmArchetype {
                        index,
                        components: vec![],
                        levels: vec![],
                    })
                    .collect(),
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            reg.archetypes[9].components = match c.mode {
                1 => vec![],
                2 => vec!["different".into(), "unsupported".into()],
                _ => vec![COMPONENT.into(), "unsupported".into()],
            };
            if let Some(names) = c.names {
                reg.archetypes[9].components = names;
            }
            reg.archetypes[9].levels = vec![0; reg.archetypes[9].components.len()];
            let payloads: Vec<Vec<u8>> = c
                .packets
                .iter()
                .map(|p| {
                    (0..p.hex.len())
                        .step_by(2)
                        .map(|j| u8::from_str_radix(&p.hex[j..j + 2], 16).unwrap())
                        .collect()
                })
                .collect();
            let mut encoding = team_encoding(c.check);
            encoding.keyframe_layout = c.keyframe_layout;
            encoding.component_widths = c.component_widths;
            for (p, payload) in c.packets.iter().zip(&payloads) {
                for r in &p.reads {
                    assert_eq!(
                        read_player_team_record_with_context(payload, r.bit, &reg, &encoding),
                        r.result,
                        "record {i} bit {}",
                        r.bit
                    );
                }
            }
            let actual = collect_player_teams_with_context(
                payloads.iter().map(Vec::as_slice),
                (c.mode != 0).then_some(&reg),
                &encoding,
            );
            let emissions: Vec<_> = actual
                .attempts
                .iter()
                .filter_map(|a| a.record.as_ref())
                .flat_map(|r| r.diagnostics.component_observations.iter().cloned())
                .collect();
            assert_eq!(emissions, c.emissions, "team walk observations {i}");
            assert_eq!(
                actual.attempts.len() as i64,
                actual.report.records,
                "team attempts {i}"
            );
            let projected: Vec<_> = actual
                .attempts
                .iter()
                .map(|a| {
                    let r = a.record.as_ref().unwrap();
                    serde_json::json!({"bit":a.record_start_bit,"slot":a.slot,"end":r.end_bit,
                    "reading":a.reading,"emissions":r.diagnostics.component_observations})
                })
                .collect();
            assert_eq!(
                projected, c.attempts,
                "native attempt boundaries and observations {i}"
            );
            let mut accepted = actual.clone();
            accepted.attempts.clear();
            assert_eq!(accepted, c.output, "teams {i}");
            let restored: FilmPlayerTeams =
                serde_json::from_slice(&serde_json::to_vec(&actual).unwrap()).unwrap();
            assert_eq!(restored, actual, "team trace export {i}");
            assert_eq!(actual.report.has_readings(), c.read, "read {i}");
            if [1025, 1029, 1031, 1537, 1545].contains(&i) {
                use crate::clients::hi::models::{FilmChunk, FilmChunkData};
                use crate::theater::{DecodeOptions, LegacyFilm};
                // Encode the native registry and packet inputs without manufacturing
                // expectations from the Rust scanner under test.
                let mut bootstrap = vec![0; 8 + 11 * 64 * 260];
                bootstrap[..4].copy_from_slice(&41u32.to_le_bytes());
                bootstrap[4..8].copy_from_slice(&27u32.to_le_bytes());
                for (j, name) in reg.archetypes[9].components.iter().enumerate() {
                    let offset = 8 + (9 * 64 + j) * 260;
                    bootstrap[offset..offset + name.len()].copy_from_slice(name.as_bytes());
                }
                bootstrap[8 + 10 * 64 * 260] = 0xff;
                let mut data = Vec::new();
                for payload in &payloads {
                    data.extend_from_slice(&2u16.to_le_bytes());
                    data.extend_from_slice(&[0, 0]);
                    data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
                    data.extend_from_slice(&1000u64.to_le_bytes());
                    data.extend_from_slice(payload);
                }
                let chunks: Vec<_> = [bootstrap, data]
                    .into_iter()
                    .enumerate()
                    .map(|(j, data)| FilmChunkData {
                        metadata: FilmChunk {
                            index: j as i32,
                            chunk_type: if j == 0 { 1 } else { 2 },
                            start_time_offset_ms: 0,
                            duration_ms: 1,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    })
                    .collect();
                let baseline = LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap();
                let mut baseline_teams = baseline.player_teams.unwrap();
                baseline_teams.attempts.clear();
                assert_ne!(
                    Some(&baseline_teams),
                    Some(&c.output),
                    "context must matter {i}"
                );
                let film = LegacyFilm::try_from_chunks_with_encoding(
                    &chunks,
                    DecodeOptions::v41(),
                    encoding.clone(),
                )
                .unwrap();
                let teams = film.player_teams.as_ref().unwrap();
                let mut accepted = teams.clone();
                accepted.attempts.clear();
                assert_eq!(accepted, c.output, "Film teams {i}");
                assert!(!teams.attempts.is_empty());
                let source_emissions: Vec<_> = teams
                    .attempts
                    .iter()
                    .filter_map(|a| a.record.as_ref())
                    .flat_map(|r| r.diagnostics.component_observations.iter().cloned())
                    .collect();
                assert_eq!(source_emissions, c.emissions, "Film callback retention {i}");
                if i >= 1536 {
                    assert!(!source_emissions.is_empty());
                }
                for a in &teams.attempts {
                    let source = a.source.as_ref().unwrap();
                    assert_eq!(source.chunk_index, 1);
                    assert_eq!(source.packet_type, 2);
                    let ordinal = a.packet_index.unwrap();
                    assert!(ordinal < payloads.len());
                    assert_eq!(
                        &chunks[1].data
                            [source.payload_offset..source.payload_offset + source.payload_size],
                        payloads[ordinal]
                    );
                    assert!(a.record_start_bit < source.payload_size * 8);
                }
                let restored: LegacyFilm =
                    serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
                assert_eq!(restored.player_teams, film.player_teams, "export teams {i}");
            }
        }
    }
}
