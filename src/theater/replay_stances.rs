//! Movement stances folded with the same life/death-aware accumulator as equipment.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
fn zero(v: &i64) -> bool {
    *v == 0
}
fn no(v: &bool) -> bool {
    !*v
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayStanceCoverage<K: Ord + Default = String> {
    pub scanned: bool,
    #[serde(skip_serializing_if = "no")]
    pub absent: bool,
    pub records: i64,
    pub desyncs: i64,
    pub reads: i64,
    pub intervals: i64,
    #[serde(skip_serializing_if = "zero")]
    pub jump_episodes: i64,
    #[serde(skip_serializing_if = "zero")]
    pub jumps_derived: i64,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub by_kind: BTreeMap<K, i64>,
    pub lives: i64,
    pub tracks_total: i64,
    #[serde(skip_serializing_if = "zero")]
    pub dropped: i64,
    #[serde(skip_serializing_if = "zero")]
    pub event_packets_unlocated: i64,
    // Go's fixed-size array is emitted even when every width is zero.
    pub map_widths: [u64; 3],
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayStances<K: Ord + Default = String> {
    pub stances: Vec<ReplayStance<K>>,
    pub coverage: ReplayStanceCoverage<K>,
}
pub fn build_replay_stances(
    reads: &[MovementStateRead],
    stats: &MovementStateStats,
    origin: u64,
    step: u64,
    tracks: &[ReplayTrack],
    closed_by_death: &BTreeSet<usize>,
) -> ReplayStances {
    build_stance_values(
        reads
            .iter()
            .map(|r| (r.timestamp_us, r.slot, r.kind.clone(), r.on)),
        ReplayStanceCoverage {
            scanned: stats.scanned,
            absent: stats.absent,
            records: stats.records as i64,
            desyncs: stats.desyncs as i64,
            reads: reads.len() as i64,
            tracks_total: tracks.len() as i64,
            dropped: stats.slot_unbound as i64,
            event_packets_unlocated: stats.event_packets_unlocated as i64,
            map_widths: stats.map_widths,
            jump_episodes: stats.jump_episodes as i64,
            jumps_derived: stats.jumps_derived as i64,
            ..Default::default()
        },
        origin,
        step,
        tracks,
        closed_by_death,
    )
}
/// Fold decoded cache readings without normalizing native kind identities.
/// The cache omits the two jump derivation counters; they remain zero here.
pub fn build_facts_replay_stances(
    reads: &[FactsMovementState],
    stats: &FactsMovementStats,
    origin: u64,
    step: u64,
    tracks: &[ReplayTrack],
    closed_by_death: &BTreeSet<usize>,
) -> ReplayStances<ReplayByteString> {
    build_stance_values(
        reads.iter().map(|r| {
            (
                r.timestamp_us,
                r.slot,
                ReplayByteString(r.kind.clone()),
                r.on,
            )
        }),
        ReplayStanceCoverage {
            scanned: stats.scanned,
            absent: stats.absent,
            records: stats.records,
            desyncs: stats.desyncs,
            reads: reads.len() as i64,
            tracks_total: tracks.len() as i64,
            dropped: stats.slot_unbound,
            event_packets_unlocated: stats.event_packets_unlocated,
            map_widths: stats.map_widths,
            ..Default::default()
        },
        origin,
        step,
        tracks,
        closed_by_death,
    )
}
fn build_stance_values<K: Ord + Clone + Default>(
    reads: impl ExactSizeIterator<Item = (u64, u32, K, bool)>,
    coverage: ReplayStanceCoverage<K>,
    origin: u64,
    step: u64,
    tracks: &[ReplayTrack],
    closed_by_death: &BTreeSet<usize>,
) -> ReplayStances<K> {
    let mut out = ReplayStances {
        coverage,
        stances: Vec::new(),
    };
    if tracks.is_empty() || step == 0 || reads.len() == 0 {
        return out;
    }
    let windows =
        super::replay_equipment_episodes::equipment_episode_windows(tracks, closed_by_death);
    let mut groups = BTreeMap::<(u32, K), Vec<(u64, bool)>>::new();
    for (timestamp, slot, kind, on) in reads {
        if !windows.contains_key(&slot) {
            out.coverage.dropped = out.coverage.dropped.wrapping_add(1);
            continue;
        }
        groups
            .entry((slot, kind))
            .or_default()
            .push((timestamp, on));
    }
    for ((slot, kind), mut records) in groups {
        records.sort_by_key(|r| r.0);
        let samples: Vec<_> = records
            .iter()
            .map(|r| {
                (
                    super::replay_equipment_episodes::equipment_episode_frame(r.0, origin, step),
                    r.1,
                )
            })
            .collect();
        out.stances.extend(
            super::replay_equipment_episodes::stance_episodes(slot, "", &samples, &windows[&slot])
                .into_iter()
                .map(|e| ReplayStance {
                    slot: e.slot,
                    kind: kind.clone(),
                    t0: e.t0,
                    t1: e.t1,
                }),
        );
    }
    out.stances.sort_by(|a, b| {
        a.t0.cmp(&b.t0)
            .then(a.slot.cmp(&b.slot))
            .then(a.kind.cmp(&b.kind))
    });
    let mut lives = BTreeSet::new();
    for s in &out.stances {
        *out.coverage.by_kind.entry(s.kind.clone()).or_default() += 1;
        lives.insert(s.slot);
    }
    out.coverage.intervals = out.stances.len() as i64;
    out.coverage.lives = lives.len() as i64;
    out
}
fn replay_movement(film: &LegacyFilm) -> Option<&MovementStateStream> {
    // Native replay clears both readings and counters on scan failure. LegacyFilm
    // retains the failed report for inspection, independently of publication.
    film.movement_states
        .as_ref()
        .filter(|_| film.movement_states_error.is_none())
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_stances(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> ReplayStances {
    let empty = MovementStateStats::default();
    let (reads, stats) =
        replay_movement(film).map_or((&[][..], &empty), |s| (s.records.as_slice(), &s.stats));
    build_replay_stances(
        reads,
        stats,
        players.clock.origin_us,
        players.clock.step_us,
        &players.players.publication.tracks,
        &players.players.death_closed_tracks,
    )
}

// The document retains native byte identities. Source decoders currently produce
// UTF-8 kind names; converting them to bytes is lossless and preserves ordering.
impl From<ReplayStance> for ReplayStance<ReplayByteString> {
    fn from(value: ReplayStance) -> Self {
        Self {
            slot: value.slot,
            kind: ReplayByteString(value.kind.into_bytes()),
            t0: value.t0,
            t1: value.t1,
        }
    }
}
impl From<ReplayStanceCoverage> for ReplayStanceCoverage<ReplayByteString> {
    fn from(value: ReplayStanceCoverage) -> Self {
        let ReplayStanceCoverage {
            scanned,
            absent,
            records,
            desyncs,
            reads,
            intervals,
            jump_episodes,
            jumps_derived,
            by_kind,
            lives,
            tracks_total,
            dropped,
            event_packets_unlocated,
            map_widths,
        } = value;
        Self {
            scanned,
            absent,
            records,
            desyncs,
            reads,
            intervals,
            jump_episodes,
            jumps_derived,
            by_kind: by_kind
                .into_iter()
                .map(|(k, v)| (ReplayByteString(k.into_bytes()), v))
                .collect(),
            lives,
            tracks_total,
            dropped,
            event_packets_unlocated,
            map_widths,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn retained_failed_movement_is_not_published() {
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut bootstrap)
            .unwrap();
        let chunk = crate::clients::hi::models::FilmChunkData {
            metadata: crate::clients::hi::models::FilmChunk {
                index: 0,
                chunk_type: 1,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: bootstrap.len() as i64,
                file_relative_path: String::new(),
            },
            data: bootstrap,
        };
        let data = super::super::fire_events::test_payload_after_keyframe(&[], 1);
        let mut packets = chunk.clone();
        packets.metadata.index = 1;
        packets.metadata.chunk_type = 2;
        packets.metadata.size = data.len() as i64;
        packets.data = data;
        let mut film =
            LegacyFilm::try_from_chunks(&[chunk, packets], DecodeOptions::v41()).unwrap();
        assert!(replay_movement(&film).is_none());
        film.movement_states = Some(MovementStateStream {
            records: Vec::new(),
            stats: MovementStateStats {
                map_widths: [13, 12, 11],
                ..Default::default()
            },
            ..Default::default()
        });
        assert_eq!(
            replay_movement(&film).unwrap().stats.map_widths,
            [13, 12, 11]
        );
        film.movement_states_error = Some("missing biped archetype".into());
        assert!(replay_movement(&film).is_none());
        let restored: LegacyFilm =
            serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
        assert!(replay_movement(&restored).is_none());
        assert_eq!(
            restored.movement_states.unwrap().stats.map_widths,
            [13, 12, 11]
        );
    }
    #[derive(Deserialize)]
    struct Case {
        reads: Vec<MovementStateRead>,
        stats: MovementStateStats,
        origin: u64,
        step: u64,
        tracks: Vec<ReplayTrack>,
        deaths: Vec<usize>,
        stances: Vec<ReplayStance>,
        coverage: ReplayStanceCoverage,
    }
    #[test]
    fn native_stance_publication() {
        let mut b = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-stances-v41.json.zlib")[..],
        )
        .read_to_end(&mut b)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&b).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let got = build_replay_stances(
                &c.reads,
                &c.stats,
                c.origin,
                c.step,
                &c.tracks,
                &c.deaths.into_iter().collect(),
            );
            assert_eq!(got.stances, c.stances, "stances {i}");
            assert_eq!(got.coverage, c.coverage, "coverage {i}");
        }
    }
}
