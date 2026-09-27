//! Native carried-weapon snapshots on the replay frame grid.
use super::{KeyframeLoadout, ReplayTrack, v41_weapon_families};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, num::NonZeroU64};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLoadout {
    pub t: i64,
    pub slot: u32,
    pub w: Vec<String>,
}
/// Fold aliases by canonical weapon name, preserving the first recorded family.
/// Unknown/empty names and pre-origin records are omitted. Equal time/slot rows
/// preserve input order and are not merged.
pub fn build_replay_loadouts(
    raw: &[KeyframeLoadout],
    origin_us: u64,
    step_us: NonZeroU64,
) -> Vec<ReplayLoadout> {
    build_loadout_values(
        raw.iter()
            .map(|l| (l.timestamp_us, l.slot, l.families.as_slice())),
        origin_us,
        step_us,
    )
}
/// The native cache carries no recording packet coordinates.
pub fn build_facts_replay_loadouts(
    raw: &[super::FactsLoadout],
    origin_us: u64,
    step_us: NonZeroU64,
) -> Vec<ReplayLoadout> {
    build_loadout_values(
        raw.iter().map(|l| {
            (
                l.timestamp_us,
                l.slot,
                l.families.as_deref().unwrap_or_default(),
            )
        }),
        origin_us,
        step_us,
    )
}
fn build_loadout_values<'a>(
    raw: impl IntoIterator<Item = (u64, u32, &'a [u32])>,
    origin_us: u64,
    step_us: NonZeroU64,
) -> Vec<ReplayLoadout> {
    let catalog = v41_weapon_families();
    let mut out = Vec::new();
    for (timestamp_us, slot, families) in raw {
        if timestamp_us < origin_us {
            continue;
        }
        let mut seen = BTreeSet::new();
        let mut ids = Vec::new();
        for family in families {
            if let Some(name) = catalog.get(family).filter(|n| !n.is_empty())
                && seen.insert(name)
            {
                ids.push(format!("0x{family:08X}"));
            }
        }
        if !ids.is_empty() {
            out.push(ReplayLoadout {
                t: ((timestamp_us - origin_us) / step_us.get()) as i64,
                slot,
                w: ids,
            });
        }
    }
    out.sort_by_key(|l| (l.t, l.slot));
    out
}
pub fn retain_replay_loadouts(loadouts: &mut Vec<ReplayLoadout>, tracks: &[ReplayTrack]) {
    let slots: BTreeSet<_> = tracks.iter().map(|t| t.slot).collect();
    loadouts.retain(|l| slots.contains(&l.slot));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        raw: Vec<KeyframeLoadout>,
        origin: u64,
        step: u64,
        slots: Vec<u32>,
        built: Vec<ReplayLoadout>,
        published: Vec<ReplayLoadout>,
    }
    #[test]
    fn native_loadout_alias_folding_and_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-loadouts-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in rows.into_iter().enumerate() {
            let mut got = build_replay_loadouts(&c.raw, c.origin, NonZeroU64::new(c.step).unwrap());
            assert_eq!(got, c.built, "built {i}");
            let tracks: Vec<_> = c
                .slots
                .into_iter()
                .map(|slot| ReplayTrack {
                    slot,
                    ..Default::default()
                })
                .collect();
            retain_replay_loadouts(&mut got, &tracks);
            assert_eq!(got, c.published, "published {i}");
        }
    }
}
