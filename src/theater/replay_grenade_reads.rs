//! Carried-grenade counters from keyframes and delta records, without value merging.
use super::{InventoryDeltaRead, KeyframeInventory, ReplayTrack};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, num::NonZeroU64};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayGrenadeRead {
    pub t: i64,
    pub slot: u32,
    /// Native append-to-nil emits null for a present but empty delta count list.
    pub g: Option<Vec<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gs: Option<i64>,
    pub src: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayGrenadeReadCoverage {
    pub from_keyframe: usize,
    pub from_delta: usize,
    pub unpublished: usize,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ammo_refused: bool,
}
fn is_false(b: &bool) -> bool {
    !*b
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayGrenadeReads {
    pub reads: Vec<ReplayGrenadeRead>,
    pub coverage: Option<ReplayGrenadeReadCoverage>,
}
/// Selection-only deltas do not replace measured counters. At equal time/slot,
/// source sorts lexically (delta before kf), with stable order inside each source.
pub fn build_replay_grenade_reads(
    kf: &[KeyframeInventory],
    deltas: &[InventoryDeltaRead],
    origin: u64,
    step: NonZeroU64,
) -> Vec<ReplayGrenadeRead> {
    let keyframes = kf.iter().filter(|r| r.grenades_read).map(|r| GrenadeInput {
        timestamp_us: r.timestamp_us,
        slot: r.slot,
        counts: &r.grenades,
        selection: (r.selected_grenade_rank >= 0).then_some(i64::from(r.selected_grenade_rank)),
        source: "kf",
    });
    let delta = deltas.iter().filter_map(|d| {
        d.grenades.as_ref().map(|counts| GrenadeInput {
            timestamp_us: d.source.timestamp_us,
            slot: d.slot,
            counts,
            selection: d.selection.as_ref().and_then(|s| s.rank).map(i64::from),
            source: "delta",
        })
    });
    build_grenade_values(keyframes.chain(delta), origin, step)
}
/// Cache-native reads preserve signed selection ranks and variable-length
/// counter lists. A present empty list remains a published native null G value.
pub fn build_facts_replay_grenade_reads(
    kf: &[super::FactsKeyframeInventory],
    deltas: &[super::FactsInventoryDelta],
    origin: u64,
    step: NonZeroU64,
) -> Vec<ReplayGrenadeRead> {
    let keyframes = kf.iter().filter(|r| r.grenades_read).map(|r| GrenadeInput {
        timestamp_us: r.timestamp_us,
        slot: r.slot,
        counts: &r.grenades,
        selection: (r.selected_grenade_rank >= 0).then_some(r.selected_grenade_rank),
        source: "kf",
    });
    let delta = deltas.iter().filter_map(|d| {
        d.grenades.as_deref().map(|counts| GrenadeInput {
            timestamp_us: d.timestamp_us,
            slot: d.slot,
            counts,
            selection: (d.selection_read && d.selection != -1).then_some(d.selection),
            source: "delta",
        })
    });
    build_grenade_values(keyframes.chain(delta), origin, step)
}
struct GrenadeInput<'a> {
    timestamp_us: u64,
    slot: u32,
    counts: &'a [u32],
    selection: Option<i64>,
    source: &'static str,
}
fn build_grenade_values<'a>(
    values: impl IntoIterator<Item = GrenadeInput<'a>>,
    origin: u64,
    step: NonZeroU64,
) -> Vec<ReplayGrenadeRead> {
    let mut out = Vec::new();
    for r in values {
        if r.timestamp_us < origin {
            continue;
        }
        out.push(ReplayGrenadeRead {
            t: ((r.timestamp_us - origin) / step.get()) as i64,
            slot: r.slot,
            g: (!r.counts.is_empty()).then(|| r.counts.to_vec()),
            gs: r.selection,
            src: r.source.into(),
        });
    }
    out.sort_by(|a, b| {
        a.t.cmp(&b.t)
            .then(a.slot.cmp(&b.slot))
            .then(a.src.cmp(&b.src))
    });
    out
}
/// Coverage is absent when nothing was built, even if ammunition was refused.
/// If every built read is filtered, coverage still records the unpublished reads.
pub fn publish_replay_grenade_reads(
    mut reads: Vec<ReplayGrenadeRead>,
    tracks: &[ReplayTrack],
    ammo_refused: bool,
) -> ReplayGrenadeReads {
    if reads.is_empty() {
        return ReplayGrenadeReads {
            reads,
            coverage: None,
        };
    }
    let before = reads.len();
    let slots: BTreeSet<_> = tracks.iter().map(|t| t.slot).collect();
    reads.retain(|r| slots.contains(&r.slot));
    let mut coverage = ReplayGrenadeReadCoverage {
        unpublished: before - reads.len(),
        ammo_refused,
        ..Default::default()
    };
    for r in &reads {
        if r.src == "delta" {
            coverage.from_delta += 1;
        } else {
            coverage.from_keyframe += 1;
        }
    }
    ReplayGrenadeReads {
        reads,
        coverage: Some(coverage),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Delta {
        time: u64,
        slot: u32,
        counts: Option<[u32; 4]>,
        selection_read: bool,
        selection: i32,
    }
    #[derive(Deserialize)]
    struct Case {
        kf: Vec<KeyframeInventory>,
        deltas: Vec<Delta>,
        origin: u64,
        step: u64,
        slots: Vec<u32>,
        refused: bool,
        built: Vec<ReplayGrenadeRead>,
        output: ReplayGrenadeReads,
    }
    #[test]
    fn native_carried_grenade_reads_and_coverage() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-grenade-reads-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in rows.into_iter().enumerate() {
            let deltas: Vec<_> = c
                .deltas
                .into_iter()
                .map(|d| InventoryDeltaRead {
                    chunk_number: None,
                    packet_index: None,
                    slot: d.slot,
                    grenades: d.counts,
                    ammo: vec![],
                    selection: d
                        .selection_read
                        .then(|| super::super::InventoryGrenadeSelection {
                            mask: 0,
                            rank: u8::try_from(d.selection).ok(),
                        }),
                    source: super::super::FilmPacket {
                        chunk_index: 0,
                        packet_type: 3,
                        byte_2: 0,
                        byte_3: 0,
                        payload_offset: 0,
                        payload_size: 0,
                        timestamp_us: d.time,
                    },
                })
                .collect();
            let built = build_replay_grenade_reads(
                &c.kf,
                &deltas,
                c.origin,
                NonZeroU64::new(c.step).unwrap(),
            );
            assert_eq!(built, c.built, "built {i}");
            let tracks: Vec<_> = c
                .slots
                .into_iter()
                .map(|slot| ReplayTrack {
                    slot,
                    ..Default::default()
                })
                .collect();
            assert_eq!(
                publish_replay_grenade_reads(built, &tracks, c.refused),
                c.output,
                "publication {i}"
            );
        }
    }
}
