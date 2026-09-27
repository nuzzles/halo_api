//! Active camouflage and overshield periods bounded by published lives and read deaths.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEquipmentEpisode {
    pub slot: u32,
    pub fam: String,
    pub t0: i64,
    pub t1: i64,
    #[serde(default, rename = "endRead", skip_serializing_if = "episode_false")]
    pub end_read: bool,
    #[serde(default, skip_serializing_if = "episode_zero")]
    pub k: usize,
    #[serde(default, skip_serializing_if = "episode_zero")]
    pub a: usize,
}
fn episode_false(v: &bool) -> bool {
    !v
}
fn episode_zero(v: &usize) -> bool {
    *v == 0
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayEquipmentCoverage {
    pub tracks_total: usize,
    pub camo_lives: usize,
    pub camo_episodes: usize,
    pub overshield_lives: usize,
    pub overshield_episodes: usize,
    pub kills_read: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEquipmentEpisodes {
    pub episodes: Vec<ReplayEquipmentEpisode>,
    pub non_binary: usize,
    pub coverage: ReplayEquipmentCoverage,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayShieldRead {
    pub slot: u32,
    pub timestamp_us: u64,
    pub quantum: u8,
}
#[derive(Clone, Copy)]
pub(super) struct EpisodeLifeWindow {
    from: i64,
    to: i64,
    dead: bool,
}
pub(super) fn equipment_episode_windows(
    tracks: &[ReplayTrack],
    deaths: &BTreeSet<usize>,
) -> BTreeMap<u32, Vec<EpisodeLifeWindow>> {
    let mut out = BTreeMap::<u32, Vec<EpisodeLifeWindow>>::new();
    for (i, t) in tracks.iter().enumerate() {
        out.entry(t.slot).or_default().push(EpisodeLifeWindow {
            from: t.start_frame,
            to: t.end_frame,
            dead: deaths.contains(&i),
        });
    }
    for list in out.values_mut() {
        super::native_sort::sort_by(list, |a, b| a.from.cmp(&b.from));
    }
    out
}
fn window_for(windows: &[EpisodeLifeWindow], from: i64, to: i64) -> Option<EpisodeLifeWindow> {
    let mut best = None;
    let mut overlap = 0;
    for w in windows {
        let ov = to.min(w.to).wrapping_sub(from.max(w.from)).wrapping_add(1);
        if ov > 0 && (best.is_none() || ov > overlap) {
            best = Some(*w);
            overlap = ov;
        }
    }
    best
}
fn span_for(windows: &[EpisodeLifeWindow], from: i64, to: i64) -> Option<EpisodeLifeWindow> {
    let list: Vec<_> = windows
        .iter()
        .filter(|w| to.min(w.to) >= from.max(w.from))
        .collect();
    if list.is_empty() {
        return None;
    }
    let anchor = list
        .iter()
        .position(|w| from >= w.from && from <= w.to)
        .unwrap_or(0);
    let (mut lo, mut hi) = (anchor, anchor);
    while lo > 0 && !list[lo - 1].dead {
        lo -= 1;
    }
    while hi + 1 < list.len() && !list[hi].dead {
        hi += 1;
    }
    Some(EpisodeLifeWindow {
        from: list[lo].from,
        to: list[hi].to,
        dead: list[hi].dead,
    })
}
pub(super) fn equipment_episode_frame(time: u64, origin: u64, step: u64) -> i64 {
    if time >= origin {
        ((time - origin) / step) as i64
    } else {
        (((origin - time).wrapping_add(step).wrapping_sub(1) / step) as i64).wrapping_neg()
    }
}
struct Accum<'a> {
    slot: u32,
    family: &'a str,
    windows: &'a [EpisodeLifeWindow],
    open: Option<i64>,
}
impl Accum<'_> {
    fn sample(&mut self, frame: i64, active: bool, out: &mut Vec<ReplayEquipmentEpisode>) {
        if active {
            if self.open.is_none() {
                self.open = Some(frame);
            }
        } else if self.open.is_some() {
            self.close(frame, true, out);
        }
    }
    fn close(&mut self, end: i64, read: bool, out: &mut Vec<ReplayEquipmentEpisode>) {
        let Some(start) = self.open.take() else {
            return;
        };
        let Some(w) = span_for(self.windows, start, end) else {
            return;
        };
        let t0 = start.max(w.from);
        let t1 = end.min(w.to);
        if t1 >= t0 {
            out.push(ReplayEquipmentEpisode {
                slot: self.slot,
                fam: self.family.into(),
                t0,
                t1,
                end_read: read,
                ..Default::default()
            });
        }
    }
    fn finish(&mut self, out: &mut Vec<ReplayEquipmentEpisode>) {
        if let Some(start) = self.open {
            if let Some(w) = window_for(self.windows, start, start) {
                self.close(w.to, false, out);
            } else {
                self.open = None;
            }
        }
    }
}
/// Shield readings must be supplied in timeline order. Camo readings are stably
/// sorted per slot. Replication gaps may be spanned, but read deaths stop the span.
/// Nonbinary camo samples are counted only for published slots and never toggle state.
pub fn build_replay_equipment_episodes(
    shields: &[ReplayShieldRead],
    camo: &[BipedCamoState],
    origin: u64,
    step: u64,
    tracks: &[ReplayTrack],
    deaths: &BTreeSet<usize>,
) -> ReplayEquipmentEpisodes {
    build_equipment_episode_values(
        shields,
        camo.iter().map(|r| CamoValue {
            timestamp_us: r.source.timestamp_us,
            slot: r.slot,
            quantum: r.quantum,
        }),
        origin,
        step,
        tracks,
        deaths,
    )
}
/// Cache position presence gates shield quanta. Sort the shield timeline as the
/// native replay assembly does before folding episodes.
pub fn build_facts_replay_equipment_episodes(
    positions: &[FactsBipedPosition],
    camo: &[FactsCamoState],
    origin: u64,
    step: u64,
    tracks: &[ReplayTrack],
    deaths: &BTreeSet<usize>,
) -> ReplayEquipmentEpisodes {
    let mut shields: Vec<_> = positions
        .iter()
        .filter(|p| p.has_shield)
        .map(|p| ReplayShieldRead {
            timestamp_us: p.timestamp_us,
            slot: p.slot,
            quantum: p.shield_quantum,
        })
        .collect();
    shields.sort_by_key(|p| p.timestamp_us);
    build_equipment_episode_values(
        &shields,
        camo.iter().map(|r| CamoValue {
            timestamp_us: r.timestamp_us,
            slot: r.slot,
            quantum: r.quantum,
        }),
        origin,
        step,
        tracks,
        deaths,
    )
}
struct CamoValue {
    timestamp_us: u64,
    slot: u32,
    quantum: u16,
}
fn build_equipment_episode_values(
    shields: &[ReplayShieldRead],
    camo: impl Iterator<Item = CamoValue>,
    origin: u64,
    step: u64,
    tracks: &[ReplayTrack],
    deaths: &BTreeSet<usize>,
) -> ReplayEquipmentEpisodes {
    let mut out = ReplayEquipmentEpisodes {
        coverage: ReplayEquipmentCoverage {
            tracks_total: tracks.len(),
            ..Default::default()
        },
        ..Default::default()
    };
    if tracks.is_empty() || step == 0 {
        return out;
    }
    let windows = equipment_episode_windows(tracks, deaths);
    let mut by_slot = BTreeMap::<u32, Vec<CamoValue>>::new();
    for r in camo {
        if windows.contains_key(&r.slot) {
            by_slot.entry(r.slot).or_default().push(r);
        }
    }
    for (slot, mut list) in by_slot {
        list.sort_by_key(|r| r.timestamp_us);
        let mut a = Accum {
            slot,
            family: "camo",
            windows: &windows[&slot],
            open: None,
        };
        for r in list {
            match r.quantum {
                0 | 4095 => a.sample(
                    equipment_episode_frame(r.timestamp_us, origin, step),
                    r.quantum == 4095,
                    &mut out.episodes,
                ),
                _ => out.non_binary += 1,
            }
        }
        a.finish(&mut out.episodes);
    }
    let mut accs = BTreeMap::new();
    let mut order = Vec::new();
    for r in shields {
        let Some(w) = windows.get(&r.slot) else {
            continue;
        };
        let a = accs.entry(r.slot).or_insert_with(|| {
            order.push(r.slot);
            Accum {
                slot: r.slot,
                family: "overshield",
                windows: w,
                open: None,
            }
        });
        a.sample(
            equipment_episode_frame(r.timestamp_us, origin, step),
            r.quantum > 64,
            &mut out.episodes,
        );
    }
    for slot in order {
        accs.get_mut(&slot).unwrap().finish(&mut out.episodes);
    }
    out.episodes
        .sort_by(|a, b| (a.t0, a.slot, &a.fam).cmp(&(b.t0, b.slot, &b.fam)));
    out.coverage = build_replay_equipment_coverage(&out.episodes, tracks, deaths);
    out
}
/// Native late coverage measurement over the document's current episodes and
/// tracks. Unknown families do not contribute to either supported family. A
/// missing life window uses (slot, -1) so a published episode remains counted.
pub fn build_replay_equipment_coverage(
    episodes: &[ReplayEquipmentEpisode],
    tracks: &[ReplayTrack],
    deaths: &BTreeSet<usize>,
) -> ReplayEquipmentCoverage {
    let windows = equipment_episode_windows(tracks, deaths);
    let mut coverage = ReplayEquipmentCoverage {
        tracks_total: tracks.len(),
        ..Default::default()
    };
    let (mut camo_lives, mut os_lives) = (BTreeSet::new(), BTreeSet::new());
    for e in episodes {
        let key = (
            e.slot,
            windows
                .get(&e.slot)
                .and_then(|w| window_for(w, e.t0, e.t1))
                .map_or(-1, |w| w.from),
        );
        match e.fam.as_str() {
            "camo" => {
                coverage.camo_episodes += 1;
                camo_lives.insert(key);
            }
            "overshield" => {
                coverage.overshield_episodes += 1;
                os_lives.insert(key);
            }
            _ => {}
        }
    }
    coverage.camo_lives = camo_lives.len();
    coverage.overshield_lives = os_lives.len();
    coverage
}
/// Construct measured episodes; kill/assist attribution is a subsequent layer.
#[allow(dead_code)]
pub(crate) fn build_film_replay_equipment_episodes(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> ReplayEquipmentEpisodes {
    let mut shields: Vec<_> = film
        .biped_positions
        .as_ref()
        .into_iter()
        .flat_map(|s| s.accepted())
        .filter_map(|p| {
            Some(ReplayShieldRead {
                slot: p.record.slot,
                timestamp_us: p.source.timestamp_us,
                quantum: p.record.companions.shield.as_ref()?.quantum,
            })
        })
        .collect();
    shields.sort_by_key(|s| s.timestamp_us);
    let camo: Vec<_> = film
        .biped_channels
        .as_ref()
        .map(|c| c.camo_states().collect())
        .unwrap_or_default();
    build_replay_equipment_episodes(
        &shields,
        &camo,
        players.clock.origin_us,
        players.clock.step_us,
        &players.players.publication.tracks,
        &players.players.death_closed_tracks,
    )
}

/// Shared native episode accumulator used by equipment and movement stances.
pub(super) fn stance_episodes(
    slot: u32,
    family: &str,
    samples: &[(i64, bool)],
    windows: &[EpisodeLifeWindow],
) -> Vec<ReplayEquipmentEpisode> {
    let mut acc = Accum {
        slot,
        family,
        windows,
        open: None,
    };
    let mut out = Vec::new();
    for &(frame, on) in samples {
        acc.sample(frame, on, &mut out);
    }
    acc.finish(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Camo {
        #[serde(rename = "TimestampUS")]
        time: u64,
        slot: u32,
        q: u16,
    }
    #[derive(Deserialize)]
    struct Case {
        camo: Vec<Camo>,
        shields: Vec<ReplayShieldRead>,
        origin: u64,
        step: u64,
        tracks: Vec<ReplayTrack>,
        deaths: BTreeSet<usize>,
        output: ReplayEquipmentEpisodes,
        kills: Vec<ReplayEquipmentKill>,
        kill_origin: Option<i64>,
        kill_interval: i64,
        kill_read: bool,
        has_occupant: bool,
        kill_measured: bool,
        credited: Vec<ReplayEquipmentEpisode>,
    }
    #[test]
    fn native_equipment_episode_publication() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-equipment-episodes-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let mut credited = c.output.episodes.clone();
            let occupant = |slot: u32, frame: i64| {
                if frame < 0 {
                    0
                } else {
                    (u64::from(slot) + (frame / 10) as u64) % 5
                }
            };
            let measured = attach_replay_equipment_kills(
                &mut credited,
                &c.kills,
                c.kill_read,
                c.has_occupant
                    .then_some(&occupant as &dyn Fn(u32, i64) -> u64),
                c.kill_origin,
                c.kill_interval,
            );
            assert_eq!(measured, c.kill_measured, "kill measurement {i}");
            assert_eq!(credited, c.credited, "kill attribution {i}");

            let camo: Vec<_> = c
                .camo
                .into_iter()
                .map(|r| BipedCamoState {
                    packet_index: None,
                    source: FilmPacket {
                        chunk_index: 1,
                        packet_type: 0,
                        byte_2: 0,
                        byte_3: 0,
                        payload_offset: 0,
                        payload_size: 0,
                        timestamp_us: r.time,
                    },
                    slot: r.slot,
                    quantum: r.q,
                })
                .collect();
            assert_eq!(
                build_replay_equipment_episodes(
                    &c.shields, &camo, c.origin, c.step, &c.tracks, &c.deaths
                ),
                c.output,
                "case {i}"
            );
        }
    }
}
