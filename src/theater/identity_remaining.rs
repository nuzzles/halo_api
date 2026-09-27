//! Final identity repair for published track spans, using fixed life evidence.
use super::{IdentityBridgeHealth, IdentityLife, ReplayIdentityState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Identity-bearing fields of a replay track, with inclusive frame bounds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayTrackIdentity {
    pub slot: u32,
    pub start_frame: i64,
    pub end_frame: i64,
    pub xuid: String,
    pub bot: super::ReplayByteString,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityOccupantCause {
    None,
    Previous,
    Next,
    Bridge,
    Contested,
}
/// Select from supplied named lives of ONE slot, preserving input-order ties.
/// Overlapping lives are not neighbors. A change of occupant across the span
/// refuses attribution even if the flat slot bridge has a name.
pub fn identity_slot_occupant_around(
    named: &[IdentityLife],
    from_us: i64,
    to_us: i64,
    bridge: &str,
) -> (String, IdentityOccupantCause) {
    let mut previous: Option<&IdentityLife> = None;
    let mut next: Option<&IdentityLife> = None;
    for life in named {
        if life.to <= from_us {
            if previous.is_none_or(|p| life.to > p.to) {
                previous = Some(life);
            }
        } else if life.from >= to_us && next.is_none_or(|n| life.from < n.from) {
            next = Some(life);
        }
    }
    if previous.zip(next).is_some_and(|(p, n)| p.xuid != n.xuid) {
        return (String::new(), IdentityOccupantCause::Contested);
    }
    if let Some(p) = previous {
        return (p.xuid.to_string(), IdentityOccupantCause::Previous);
    }
    if let Some(n) = next {
        return (n.xuid.to_string(), IdentityOccupantCause::Next);
    }
    if !bridge.is_empty() {
        return (bridge.into(), IdentityOccupantCause::Bridge);
    }
    (String::new(), IdentityOccupantCause::None)
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityRemainingReport {
    pub by_previous: usize,
    pub by_next: usize,
    pub by_bridge: usize,
    pub remaining: usize,
    pub contested: usize,
    /// Track indices named by inference, not by a direct identity reading.
    pub deduced: BTreeSet<usize>,
}
impl IdentityRemainingReport {
    pub fn total(&self) -> usize {
        self.by_previous + self.by_next + self.by_bridge + self.remaining
    }
    pub fn apply_to_health(&self, health: &mut IdentityBridgeHealth) {
        health.named_by_previous_life = self.by_previous;
        health.named_by_next_life = self.by_next;
        health.named_by_slot_bridge = self.by_bridge;
        health.unnamed_lives = self.remaining;
        health.unnamed_lives_contested = self.contested;
    }
}
/// Repair anonymous tracks after direct, bot, and succession naming. Deductions
/// never enter the life evidence used to name subsequent tracks.
pub fn name_remaining_identity_tracks(
    tracks: &mut [ReplayTrackIdentity],
    state: &ReplayIdentityState,
    origin_us: u64,
    step_us: u64,
) -> IdentityRemainingReport {
    let mut named: BTreeMap<u32, Vec<IdentityLife>> = BTreeMap::new();
    for life in state.lives().iter().filter(|l| l.xuid != 0) {
        named.entry(life.slot).or_default().push(life.clone());
    }
    let bridges = state.naming_bridge();
    let mut report = IdentityRemainingReport::default();
    for (i, track) in tracks.iter_mut().enumerate() {
        if !track.xuid.is_empty() || !track.bot.is_empty() {
            continue;
        }
        let time = |frame: i64| (origin_us as i64).wrapping_add(frame.wrapping_mul(step_us as i64));
        let bridge = bridges
            .get(&track.slot)
            .filter(|&&x| x != 0)
            .map(u64::to_string)
            .unwrap_or_default();
        let (xuid, cause) = identity_slot_occupant_around(
            named
                .get(&track.slot)
                .map(Vec::as_slice)
                .unwrap_or_default(),
            time(track.start_frame),
            time(track.end_frame),
            &bridge,
        );
        match cause {
            IdentityOccupantCause::Previous => report.by_previous += 1,
            IdentityOccupantCause::Next => report.by_next += 1,
            IdentityOccupantCause::Bridge => report.by_bridge += 1,
            IdentityOccupantCause::Contested => {
                report.contested += 1;
                report.remaining += 1;
            }
            IdentityOccupantCause::None => report.remaining += 1,
        }
        if !xuid.is_empty() {
            track.xuid = xuid;
            report.deduced.insert(i);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::{
        IdentityBot, identity_bot_names_by_seat, name_bot_identity_tracks,
        name_identity_tracks_by_lives,
    };
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        direct: Vec<ReplayTrackIdentity>,
        fallbacks: usize,
        bot_tracks: Vec<ReplayTrackIdentity>,
        owners: BTreeMap<u32, i64>,
        bots: Vec<IdentityBot>,
        bot_names: BTreeMap<i64, crate::theater::ReplayByteString>,
        shared: usize,
        state: ReplayIdentityState,
        tracks: Vec<ReplayTrackIdentity>,
        origin: u64,
        step: u64,
        output: Vec<ReplayTrackIdentity>,
        report: IdentityRemainingReport,
        total: usize,
    }
    #[test]
    fn native_final_track_identity_repair() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-remaining-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, mut c) in cases.into_iter().enumerate() {
            let mut direct = c.tracks.clone();
            assert_eq!(
                name_identity_tracks_by_lives(&mut direct, c.state.lives(), c.origin, c.step),
                c.fallbacks,
                "overlap fallback {i}"
            );
            assert_eq!(direct, c.direct, "direct names {i}");
            assert_eq!(
                identity_bot_names_by_seat(&c.bots),
                (c.bot_names, c.shared),
                "bot seats {i}"
            );
            let mut bots = c.tracks.clone();
            name_bot_identity_tracks(&mut bots, &c.owners, &c.bots);
            assert_eq!(bots, c.bot_tracks, "bot tracks {i}");
            let report = name_remaining_identity_tracks(&mut c.tracks, &c.state, c.origin, c.step);
            assert_eq!(report, c.report, "report {i}");
            assert_eq!(report.total(), c.total, "total {i}");
            assert_eq!(c.tracks, c.output, "tracks {i}");
        }
    }
}
