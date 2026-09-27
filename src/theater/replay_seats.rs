//! Native roster seat reuse, ordinal replacement fallback, and presence coverage.
use super::{ReplayByteString, ReplayFilmPlayerTable, ReplayRosterEntry, ReplayTrack};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplaySeatCoverage {
    pub entrees: usize,
    pub sieges: usize,
    pub lus: usize,
    pub apparies: usize,
    pub reprises_ecrites: usize,
    pub arrivants: usize,
    pub presences_closes: usize,
    pub sans_presence: usize,
    pub occupants_max: usize,
    #[serde(skip_serializing_if = "is_false")]
    pub sans_table_du_film: bool,
}
fn is_false(v: &bool) -> bool {
    !*v
}
fn roster_key(e: &ReplayRosterEntry) -> ReplayByteString {
    if !e.xuid.is_empty() {
        e.xuid.clone().into()
    } else if e.bot && !e.name.is_empty() {
        ReplayByteString([b"bot:".as_slice(), e.name.as_ref()].concat())
    } else {
        ReplayByteString::default()
    }
}
/// Native presence is the envelope of all lives for an identity, including the
/// gaps between them. Bots join through their names at this publication stage.
pub fn replay_presence_envelopes(tracks: &[ReplayTrack]) -> BTreeMap<ReplayByteString, [i64; 2]> {
    let mut out = BTreeMap::<ReplayByteString, [i64; 2]>::new();
    for t in tracks {
        let key = if !t.xuid.is_empty() {
            t.xuid.clone().into()
        } else if !t.bot.is_empty() {
            ReplayByteString([b"bot:".as_slice(), t.bot.as_ref()].concat())
        } else {
            continue;
        };
        out.entry(key)
            .and_modify(|w| {
                w[0] = w[0].min(t.start_frame);
                w[1] = w[1].max(t.end_frame);
            })
            .or_insert([t.start_frame, t.end_frame]);
    }
    out
}
#[derive(Clone, Copy)]
struct SeatState {
    index: usize,
    from: i64,
    to: i64,
    present: bool,
    origin: bool,
}
/// Set every seat from its written film index first. Only unmatched new indices
/// may inherit a departed original seat of the same recorded team. `apparies`
/// is also the native ordinal-fallback count.
pub fn assign_replay_seats(
    roster: &mut [ReplayRosterEntry],
    tracks: &[ReplayTrack],
    table: &ReplayFilmPlayerTable,
    frames: i64,
) -> ReplaySeatCoverage {
    let mut cov = ReplaySeatCoverage {
        entrees: roster.len(),
        ..Default::default()
    };
    if roster.is_empty() {
        return cov;
    }
    let origins = table.is_read().then(|| {
        table
            .seats
            .iter()
            .map(|s| s.film_index)
            .collect::<BTreeSet<_>>()
    });
    cov.sans_table_du_film = origins.is_none();
    let presence = replay_presence_envelopes(tracks);
    let mut states = Vec::new();
    let mut shared = BTreeMap::<i64, usize>::new();
    for (index, e) in roster.iter_mut().enumerate() {
        e.seat = e.film_index;
        e.seat_source = "lu".into();
        *shared.entry(e.film_index).or_default() += 1;
        let w = presence.get(&roster_key(e));
        let s = SeatState {
            index,
            from: w.map_or(0, |w| w[0]),
            to: w.map_or(0, |w| w[1]),
            present: w.is_some(),
            origin: origins.as_ref().is_none_or(|o| o.contains(&e.film_index)),
        };
        if !s.present {
            cov.sans_presence += 1;
        } else if frames > 0 && s.to < frames - 1 {
            cov.presences_closes += 1;
        }
        if !s.origin {
            cov.arrivants += 1;
        }
        states.push(s);
    }
    cov.reprises_ecrites = shared.values().filter(|&&n| n > 1).count();
    if origins.is_some() {
        let teams: BTreeSet<_> = roster.iter().filter_map(|e| e.team).collect();
        for team in teams {
            let mut leaving = Vec::new();
            let mut arriving = Vec::new();
            for s in &states {
                let e = &roster[s.index];
                if e.team != Some(team) || !s.present {
                    continue;
                }
                if s.origin && frames > 0 && s.to < frames - 1 {
                    leaving.push(*s);
                } else if !s.origin && shared[&e.film_index] == 1 {
                    arriving.push(*s);
                }
            }
            super::native_sort::sort_by(&mut leaving, |a, b| a.to.cmp(&b.to));
            super::native_sort::sort_by(&mut arriving, |a, b| a.from.cmp(&b.from));
            let mut k = 0;
            for a in arriving {
                while k < leaving.len() && leaving[k].to >= a.from {
                    k += 1;
                }
                if k == leaving.len() {
                    break;
                }
                roster[a.index].seat = roster[leaving[k].index].seat;
                roster[a.index].seat_source = "apparie".into();
                k += 1;
                cov.apparies += 1;
            }
        }
    }
    cov.lus = cov.entrees - cov.apparies;
    cov.sieges = roster.iter().map(|e| e.seat).collect::<BTreeSet<_>>().len();
    let mut edges: Vec<_> = states
        .iter()
        .filter(|s| s.present)
        .flat_map(|s| [(s.from, 1i64), (s.to.wrapping_add(1), -1)])
        .collect();
    edges.sort_unstable();
    let mut n = 0i64;
    for (_, delta) in edges {
        n += delta;
        cov.occupants_max = cov.occupants_max.max(n.max(0) as usize);
    }
    cov
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        roster: Vec<ReplayRosterEntry>,
        tracks: Vec<ReplayTrack>,
        table: ReplayFilmPlayerTable,
        frames: i64,
        output: Vec<ReplayRosterEntry>,
        coverage: ReplaySeatCoverage,
        presence: BTreeMap<ReplayByteString, [i64; 2]>,
        fallbacks: usize,
    }
    #[test]
    fn native_seat_assignment_and_coverage() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/replay-seats-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, mut c) in cases.into_iter().enumerate() {
            assert_eq!(
                replay_presence_envelopes(&c.tracks),
                c.presence,
                "presence {i}"
            );
            let cov = assign_replay_seats(&mut c.roster, &c.tracks, &c.table, c.frames);
            assert_eq!(cov, c.coverage, "coverage {i}");
            assert_eq!(cov.apparies, c.fallbacks, "fallbacks {i}");
            assert_eq!(c.roster, c.output, "seats {i}");
        }
    }
}
