//! Occupant-driven boarding and exit episode state machine.
use super::{FactsVehicleEvent, ReplayPlayerPosition, VehicleEvent};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVehicleEpisode {
    pub slot: u32,
    pub start_us: u64,
    pub end_us: u64,
    pub borders: i64,
    pub open_end: bool,
    pub vehicle_slot: u32,
    pub vehicle_valid: bool,
    pub vehicle_at_us: u64,
    pub reappear_us: u64,
}
/// Only fields consumed by ride attribution; this is not a synthetic source record.
#[derive(Clone, Copy)]
pub(super) struct VehicleRideEvent {
    pub timestamp_us: u64,
    pub kind: i64,
    pub occupant_present: bool,
    pub occupant_in_band: bool,
    pub occupant_slot: u32,
    pub vehicle_slot: u32,
    pub vehicle_slot_valid: bool,
}
impl From<&VehicleEvent> for VehicleRideEvent {
    fn from(e: &VehicleEvent) -> Self {
        Self {
            timestamp_us: e.timestamp_us,
            kind: i64::from(e.kind),
            occupant_present: e.occupant_present,
            occupant_in_band: e.occupant_in_band,
            occupant_slot: e.occupant_slot,
            vehicle_slot: e.vehicle_slot,
            vehicle_slot_valid: e.vehicle_slot_valid,
        }
    }
}
impl From<&FactsVehicleEvent> for VehicleRideEvent {
    fn from(e: &FactsVehicleEvent) -> Self {
        Self {
            timestamp_us: e.timestamp_us,
            kind: e.kind,
            occupant_present: e.occupant_present,
            occupant_in_band: e.occupant_in_band,
            occupant_slot: e.occupant_slot,
            vehicle_slot: e.vehicle_slot,
            vehicle_slot_valid: e.vehicle_slot_valid,
        }
    }
}
/// First point at or after `at_us`, or last point strictly before it.
/// Positions must be sorted by timestamp.
pub fn replay_vehicle_anchor_at(
    positions: &[ReplayPlayerPosition],
    at_us: u64,
    after: bool,
) -> Option<&ReplayPlayerPosition> {
    let mut lo = 0;
    let mut hi = positions.len();
    while lo < hi {
        let m = lo + (hi - lo) / 2;
        if positions[m].timestamp_us >= at_us {
            hi = m;
        } else {
            lo = m + 1;
        }
    }
    if after {
        positions.get(lo)
    } else {
        lo.checked_sub(1).and_then(|i| positions.get(i))
    }
}
/// Merge already sorted lists, taking exits before boards at equal timestamps.
pub fn merge_replay_vehicle_events(
    boards: &[VehicleEvent],
    exits: &[VehicleEvent],
) -> Vec<VehicleEvent> {
    merge_vehicle_event_values(boards, exits, |e| e.timestamp_us)
}
/// Preserve complete cached events, with exits before boards at equal times.
pub fn merge_facts_replay_vehicle_events(
    boards: &[FactsVehicleEvent],
    exits: &[FactsVehicleEvent],
) -> Vec<FactsVehicleEvent> {
    merge_vehicle_event_values(boards, exits, |e| e.timestamp_us)
}
fn merge_vehicle_event_values<T: Clone>(
    boards: &[T],
    exits: &[T],
    time: impl Fn(&T) -> u64,
) -> Vec<T> {
    let mut out = Vec::with_capacity(boards.len() + exits.len());
    let (mut i, mut j) = (0, 0);
    while i < boards.len() && j < exits.len() {
        if time(&exits[j]) <= time(&boards[i]) {
            out.push(exits[j].clone());
            j += 1;
        } else {
            out.push(boards[i].clone());
            i += 1;
        }
    }
    out.extend_from_slice(&boards[i..]);
    out.extend_from_slice(&exits[j..]);
    out
}
/// A second board closes the previous episode. An unpaired exit starts at the
/// last prior position (or the exit itself). Open final rides retain reappearance
/// evidence for later closure against the vehicle's life window.
pub fn replay_vehicle_episodes_of_occupant(
    slot: u32,
    events: &[VehicleEvent],
    positions: &[ReplayPlayerPosition],
) -> Vec<ReplayVehicleEpisode> {
    vehicle_episodes_of_occupant(slot, events.iter().map(VehicleRideEvent::from), positions)
}
/// Positions are pure value projections; event kinds retain signed cache identity.
pub fn replay_facts_vehicle_episodes_of_occupant(
    slot: u32,
    events: &[FactsVehicleEvent],
    positions: &[ReplayPlayerPosition],
) -> Vec<ReplayVehicleEpisode> {
    vehicle_episodes_of_occupant(slot, events.iter().map(VehicleRideEvent::from), positions)
}
fn vehicle_episodes_of_occupant(
    slot: u32,
    events: impl Iterator<Item = VehicleRideEvent>,
    positions: &[ReplayPlayerPosition],
) -> Vec<ReplayVehicleEpisode> {
    let mut out = Vec::new();
    let mut open: Option<ReplayVehicleEpisode> = None;
    for event in events {
        if event.kind == 8 {
            if let Some(mut previous) = open.take() {
                previous.end_us = event.timestamp_us;
                previous.open_end = false;
                out.push(previous);
            }
            open = Some(ReplayVehicleEpisode {
                slot,
                start_us: event.timestamp_us,
                borders: 1,
                open_end: true,
                ..Default::default()
            });
            continue;
        }
        let mut episode = open.take().unwrap_or_else(|| ReplayVehicleEpisode {
            slot,
            start_us: replay_vehicle_anchor_at(positions, event.timestamp_us, false)
                .map_or(event.timestamp_us, |p| p.timestamp_us),
            ..Default::default()
        });
        episode.end_us = event.timestamp_us;
        episode.open_end = false;
        episode.borders += 1;
        if event.vehicle_slot_valid {
            episode.vehicle_slot = event.vehicle_slot;
            episode.vehicle_valid = true;
            episode.vehicle_at_us = event.timestamp_us;
        }
        out.push(episode);
    }
    if let Some(mut episode) = open {
        episode.reappear_us =
            replay_vehicle_anchor_at(positions, episode.start_us.wrapping_add(1), true)
                .map_or(0, |p| p.timestamp_us);
        out.push(episode);
    }
    out
}
/// Output is deterministic by occupant and event order. Inputs are already
/// grouped by occupant and sorted by timestamp.
pub fn build_replay_vehicle_event_episodes(
    boards: &BTreeMap<u32, Vec<VehicleEvent>>,
    exits: &BTreeMap<u32, Vec<VehicleEvent>>,
    positions: &BTreeMap<u32, Vec<ReplayPlayerPosition>>,
) -> Vec<ReplayVehicleEpisode> {
    vehicle_event_episode_values(&ride_event_map(boards), &ride_event_map(exits), positions)
}
pub fn build_facts_replay_vehicle_event_episodes(
    boards: &BTreeMap<u32, Vec<FactsVehicleEvent>>,
    exits: &BTreeMap<u32, Vec<FactsVehicleEvent>>,
    positions: &BTreeMap<u32, Vec<ReplayPlayerPosition>>,
) -> Vec<ReplayVehicleEpisode> {
    vehicle_event_episode_values(&ride_event_map(boards), &ride_event_map(exits), positions)
}
fn ride_event_map<T>(events: &BTreeMap<u32, Vec<T>>) -> BTreeMap<u32, Vec<VehicleRideEvent>>
where
    for<'a> VehicleRideEvent: From<&'a T>,
{
    events
        .iter()
        .map(|(&slot, events)| (slot, events.iter().map(VehicleRideEvent::from).collect()))
        .collect()
}
pub(super) fn vehicle_event_episode_values(
    boards: &BTreeMap<u32, Vec<VehicleRideEvent>>,
    exits: &BTreeMap<u32, Vec<VehicleRideEvent>>,
    positions: &BTreeMap<u32, Vec<ReplayPlayerPosition>>,
) -> Vec<ReplayVehicleEpisode> {
    let slots: BTreeSet<_> = boards.keys().chain(exits.keys()).copied().collect();
    let mut out = Vec::new();
    for slot in slots {
        let events = merge_vehicle_event_values(
            boards.get(&slot).map_or(&[], Vec::as_slice),
            exits.get(&slot).map_or(&[], Vec::as_slice),
            |e| e.timestamp_us,
        );
        out.extend(vehicle_episodes_of_occupant(
            slot,
            events.into_iter(),
            positions.get(&slot).map_or(&[], Vec::as_slice),
        ));
    }
    out
}
/// Published event episodes suppress overlapping gap fallbacks inclusively.
pub fn replay_vehicle_episode_covers(
    episodes: &[ReplayVehicleEpisode],
    slot: u32,
    start_us: u64,
    end_us: u64,
) -> bool {
    episodes
        .iter()
        .any(|e| e.slot == slot && start_us <= e.end_us && e.start_us <= end_us)
}
