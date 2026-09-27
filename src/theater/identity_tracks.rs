//! Direct life-overlap and unambiguous bot-seat naming for replay tracks.
use super::{IdentityBot, IdentityLife, ReplayTrackIdentity};
use std::collections::{BTreeMap, BTreeSet};

/// Names tracks by their greatest positive overlap with a named life of the
/// same slot. Returns how often a later candidate replaced an earlier winner.
/// Equal overlaps preserve the first life; existing XUIDs may be replaced.
pub fn name_identity_tracks_by_lives(
    tracks: &mut [ReplayTrackIdentity],
    lives: &[IdentityLife],
    origin_us: u64,
    step_us: u64,
) -> usize {
    let mut fallbacks = 0;
    for track in tracks {
        let from = (origin_us as i64).wrapping_add(track.start_frame.wrapping_mul(step_us as i64));
        let to = (origin_us as i64)
            .wrapping_add(track.end_frame.wrapping_mul(step_us as i64))
            .wrapping_add(step_us as i64)
            .wrapping_sub(1);
        let mut best_xuid = 0;
        let mut best_overlap = 0;
        for life in lives {
            if life.slot != track.slot || life.xuid == 0 {
                continue;
            }
            let overlap = to.min(life.to).wrapping_sub(from.max(life.from));
            if overlap > best_overlap {
                if best_xuid != 0 {
                    fallbacks += 1;
                }
                best_overlap = overlap;
                best_xuid = life.xuid;
            }
        }
        if best_xuid != 0 {
            track.xuid = best_xuid.to_string();
        }
    }
    fallbacks
}
/// Distinct names sharing a seat make that seat ambiguous; repeated declarations
/// of the same name do not. Empty names supply no identity.
pub fn identity_bot_names_by_seat(
    bots: &[IdentityBot],
) -> (BTreeMap<i64, super::ReplayByteString>, usize) {
    let mut names: BTreeMap<i64, BTreeSet<&super::ReplayByteString>> = BTreeMap::new();
    for bot in bots {
        if !bot.name.is_empty() {
            names.entry(bot.film_index).or_default().insert(&bot.name);
        }
    }
    let mut shared = 0;
    let mut out = BTreeMap::new();
    for (seat, names) in names {
        if names.len() > 1 {
            shared += 1;
        } else if let Some(name) = names.first() {
            out.insert(seat, (*name).clone());
        }
    }
    (out, shared)
}
/// Fill anonymous tracks through their resolved owner index; preserve every
/// existing human/bot name. Returns the number of shared bot seats refused.
pub fn name_bot_identity_tracks(
    tracks: &mut [ReplayTrackIdentity],
    owners: &BTreeMap<u32, i64>,
    bots: &[IdentityBot],
) -> usize {
    if owners.is_empty() || bots.is_empty() {
        return 0;
    }
    let (names, shared) = identity_bot_names_by_seat(bots);
    for track in tracks {
        if !track.xuid.is_empty() || !track.bot.is_empty() {
            continue;
        }
        if let Some(name) = owners.get(&track.slot).and_then(|seat| names.get(seat)) {
            track.bot = name.clone();
        }
    }
    shared
}

impl super::ReplayIdentityState {
    fn tracks_matching_lives(
        &self,
        tracks: &[ReplayTrackIdentity],
        origin: u64,
        step: u64,
        keep: impl Fn(usize, &IdentityLife) -> bool,
    ) -> BTreeSet<usize> {
        tracks
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                let from = (origin as i64).wrapping_add(t.start_frame.wrapping_mul(step as i64));
                let to = (origin as i64).wrapping_add(t.end_frame.wrapping_mul(step as i64));
                self.lives()
                    .iter()
                    .enumerate()
                    .any(|(li, l)| {
                        l.slot == t.slot && keep(li, l) && to.min(l.to) >= from.max(l.from)
                    })
                    .then_some(i)
            })
            .collect()
    }
    /// Inclusive overlap with any inferred life, irrespective of its name.
    pub fn deduced_tracks(
        &self,
        tracks: &[ReplayTrackIdentity],
        origin: u64,
        step: u64,
    ) -> BTreeSet<usize> {
        self.tracks_matching_lives(tracks, origin, step, |i, _| self.life_is_deduced(i))
    }
    pub fn death_closed_tracks(
        &self,
        tracks: &[ReplayTrackIdentity],
        origin: u64,
        step: u64,
    ) -> BTreeSet<usize> {
        self.tracks_matching_lives(tracks, origin, step, |_, l| l.cause == "death")
    }
}

/// Inputs for the ordered naming stages after replay track segmentation.
pub struct IdentityTrackNamingInput<'a> {
    pub state: &'a super::ReplayIdentityState,
    pub bots: &'a [IdentityBot],
    pub successions: &'a [super::IdentitySuccession],
    pub fire: &'a [super::IdentityFireReference],
    pub origin_us: u64,
    pub step_us: u64,
    pub death_offset_ms: i64,
    pub offset_matches: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IdentityTrackNamingReport {
    pub overlap_fallbacks: usize,
    pub shared_bot_seats: usize,
    pub succession: super::IdentitySuccessionReport,
    pub remaining: super::IdentityRemainingReport,
    pub death_closed: BTreeSet<usize>,
}
/// Native replay order: direct life overlap, bot seats, replacement chains,
/// final neighbor repair, then union the registry's inferred-life track indices.
pub fn name_replay_identity_tracks(
    tracks: &mut [ReplayTrackIdentity],
    input: IdentityTrackNamingInput<'_>,
) -> IdentityTrackNamingReport {
    let overlap_fallbacks =
        name_identity_tracks_by_lives(tracks, input.state.lives(), input.origin_us, input.step_us);
    let shared_bot_seats =
        name_bot_identity_tracks(tracks, input.state.indices_by_slot(), input.bots);
    let succession = super::attribute_identity_successions(
        tracks,
        input.successions,
        input.origin_us,
        input.step_us,
        input.death_offset_ms,
        input.offset_matches,
        input.fire,
    );
    let mut remaining =
        super::name_remaining_identity_tracks(tracks, input.state, input.origin_us, input.step_us);
    remaining.deduced.extend(
        input
            .state
            .deduced_tracks(tracks, input.origin_us, input.step_us),
    );
    let death_closed = input
        .state
        .death_closed_tracks(tracks, input.origin_us, input.step_us);
    IdentityTrackNamingReport {
        overlap_fallbacks,
        shared_bot_seats,
        succession,
        remaining,
        death_closed,
    }
}
