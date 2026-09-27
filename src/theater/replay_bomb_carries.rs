//! Bomb held-family filtering and publication through the shared presence gate.
use super::*;
use serde::{Deserialize, Serialize};
pub const BOMB_HELD_FAMILY: u32 = 0x3fee4fcf;
pub fn replay_bomb_held_events(
    changes: &[HeldWeaponChange],
    death_offset_ms: i64,
) -> Vec<ReplayHeldObjectEvent> {
    bomb_held_events(
        changes
            .iter()
            .map(|ch| (ch.timestamp_us, ch.slot, ch.family, ch.previous)),
        death_offset_ms,
    )
}
/// Filter cache transitions without interpreting raw kind bytes or slot indices.
pub fn replay_facts_bomb_held_events(
    changes: &[FactsWeaponChange],
    death_offset_ms: i64,
) -> Vec<ReplayHeldObjectEvent> {
    bomb_held_events(
        changes
            .iter()
            .map(|ch| (ch.timestamp_us, ch.slot, ch.family, ch.previous)),
        death_offset_ms,
    )
}
fn bomb_held_events(
    changes: impl Iterator<Item = (u64, u32, u32, u32)>,
    death_offset_ms: i64,
) -> Vec<ReplayHeldObjectEvent> {
    let mut out = Vec::new();
    for (timestamp_us, slot, family, previous) in changes {
        let time_ms = ((timestamp_us as i64) / 1000).wrapping_sub(death_offset_ms);
        if family == BOMB_HELD_FAMILY {
            out.push(ReplayHeldObjectEvent {
                time_ms,
                slot,
                xuid: 0,
                pickup: true,
            });
        }
        if previous == BOMB_HELD_FAMILY && family != BOMB_HELD_FAMILY {
            out.push(ReplayHeldObjectEvent {
                time_ms,
                slot,
                xuid: 0,
                pickup: false,
            });
        }
    }
    out
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayBombCarry {
    pub xuid: String,
    pub t0: i64,
    pub t1: i64,
    pub closed: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayBombCarryCoverage {
    pub bomb_film: bool,
    pub events: usize,
    pub periods: usize,
    pub carries: usize,
    pub closed: usize,
    pub open: usize,
    pub by_death: usize,
    pub no_bridge: usize,
    pub out_of_window: usize,
    pub carrier_absent: usize,
}
impl ReplayBombCarryCoverage {
    pub fn balanced(&self) -> bool {
        self.carries + self.no_bridge + self.out_of_window + self.carrier_absent == self.periods
            && self.closed + self.open == self.carries
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayBombCarries {
    pub carries: Vec<ReplayBombCarry>,
    pub coverage: Option<ReplayBombCarryCoverage>,
}
pub fn build_replay_bomb_carries(
    carry: &ReplayHeldObjectCarry,
    clock: ReplayMatchClock,
    presence: &ReplayCarrierPresence,
) -> ReplayBombCarries {
    let mut cov = ReplayBombCarryCoverage {
        bomb_film: true,
        events: carry.events.len(),
        periods: carry.periods.len(),
        ..Default::default()
    };
    let mut carries = Vec::new();
    for p in &carry.periods {
        if p.xuid == 0 {
            cov.no_bridge += 1;
            continue;
        }
        let t0 = clock.frame_of_match_ms(p.start_ms);
        if t0 < 0 || t0 >= clock.frames {
            cov.out_of_window += 1;
            continue;
        }
        let mut t1 = clock.frame_of_match_ms(p.end_ms);
        if t1 < 0 {
            t1 = 0;
        } else if t1 >= clock.frames {
            t1 = clock.frames.wrapping_sub(1);
        }
        t1 = t1.max(t0);
        let xuid = p.xuid.to_string();
        let Some(span) = presence.gate(&xuid, t0, t1) else {
            cov.carrier_absent += 1;
            continue;
        };
        let closed = !p.open;
        carries.push(ReplayBombCarry {
            xuid,
            t0: span.t0,
            t1: span.t1,
            closed,
        });
        if closed {
            cov.closed += 1;
            if p.by_death {
                cov.by_death += 1;
            }
        } else {
            cov.open += 1;
        }
    }
    cov.carries = carries.len();
    ReplayBombCarries {
        carries,
        coverage: Some(cov),
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmReplayBombCarries {
    pub raw: ReplayHeldObjectCarry,
    pub publication: ReplayBombCarries,
}
/// The complete raw carry timeline remains available for bomb statistics and
/// arming attribution, including unnamed periods and exact millisecond bounds.
#[allow(dead_code)]
pub(crate) fn build_film_replay_bomb_carries(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    bomb_recognized: bool,
) -> FilmReplayBombCarries {
    if !bomb_recognized {
        return FilmReplayBombCarries::default();
    }
    let offset = players.registry.owners.clock.offset_ms;
    let changes = film
        .weapon_changes
        .as_ref()
        .map_or(&[][..], |s| s.records.as_slice());
    let events = replay_bomb_held_events(changes, offset);
    assemble_bomb_carries(
        events,
        players,
        players.evidence.deaths.iter().map(|d| (d.xuid, d.time_ms)),
    )
}

/// Assemble cached bomb carries using the restored temporal identity bridge.
/// Carry recognition is independent of the bomb-arming mode gate.
pub fn build_facts_replay_bomb_carries(
    changes: &[FactsWeaponChange],
    deaths: &[FactsDeath],
    players: &FilmReplayPlayers,
    bomb_recognized: bool,
) -> FilmReplayBombCarries {
    if !bomb_recognized {
        return FilmReplayBombCarries::default();
    }
    let events = replay_facts_bomb_held_events(changes, players.registry.owners.clock.offset_ms);
    assemble_bomb_carries(events, players, deaths.iter().map(|d| (d.xuid, d.time_ms)))
}

fn assemble_bomb_carries(
    events: Vec<ReplayHeldObjectEvent>,
    players: &FilmReplayPlayers,
    deaths: impl Iterator<Item = (u64, i64)>,
) -> FilmReplayBombCarries {
    assemble_bomb_carry_values(
        events,
        &players.registry.owners.state,
        ReplayMatchClock {
            origin_us: players.clock.origin_us,
            step_us: players.clock.step_us,
            frames: players.clock.frame_count,
            death_offset_ms: players.registry.owners.clock.offset_ms,
        },
        &ReplayCarrierPresence::from_tracks(
            &players.players.publication.tracks,
            &players.players.naming.deduced,
        ),
        deaths,
    )
}

pub(super) fn assemble_bomb_carry_values(
    events: Vec<ReplayHeldObjectEvent>,
    state: &ReplayIdentityState,
    clock: ReplayMatchClock,
    presence: &ReplayCarrierPresence,
    deaths: impl Iterator<Item = (u64, i64)>,
) -> FilmReplayBombCarries {
    let offset = clock.death_offset_ms;
    if !state.bridge_established() {
        return FilmReplayBombCarries {
            publication: ReplayBombCarries {
                carries: Vec::new(),
                coverage: Some(ReplayBombCarryCoverage {
                    bomb_film: true,
                    events: events.len(),
                    ..Default::default()
                }),
            },
            ..Default::default()
        };
    }
    let occupant = |slot, time: i64| {
        let ms = time.wrapping_add(offset);
        if ms < 0 {
            0
        } else {
            state.xuid_at(slot, (ms as u64).wrapping_mul(1000))
        }
    };
    let raw = super::replay_held_object::held_object_carry(&events, Some(&occupant), deaths);
    let publication = build_replay_bomb_carries(&raw, clock, presence);
    FilmReplayBombCarries { raw, publication }
}
