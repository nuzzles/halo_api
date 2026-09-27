//! Combine written occupancy with event and position-gap ride fallbacks.
use super::replay_vehicle_episodes::{VehicleRideEvent, vehicle_event_episode_values};
use super::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, num::NonZeroU64};

pub struct ReplayVehicleRidesContext<'a> {
    pub film: ReplayVehicleRideContext<'a>,
    pub events: &'a [VehicleEvent],
    pub vehicles: &'a BTreeMap<u32, Vec<ReplayVehiclePosition>>,
}
pub struct FactsReplayVehicleRidesContext<'a> {
    pub film: FactsReplayVehicleRideContext<'a>,
    pub events: &'a [FactsVehicleEvent],
    pub vehicles: &'a BTreeMap<u32, Vec<FactsBipedPosition>>,
}
struct VehicleRideValues<'a> {
    bipeds: &'a [ReplayPlayerPosition],
    events: &'a [VehicleRideEvent],
    vehicles: &'a BTreeMap<u32, Vec<ReplayPlayerPosition>>,
    lives: &'a [ReplayVehicleLife],
    drawable: &'a std::collections::BTreeSet<EquipmentLifeKey>,
    occupancy: &'a [VehicleOccupancy],
    origin_us: u64,
    step_us: u64,
    frames: i64,
}
fn facts_ride_position(p: &FactsBipedPosition) -> ReplayPlayerPosition {
    ReplayPlayerPosition {
        slot: p.slot,
        timestamp_us: p.timestamp_us,
        x: p.world[0],
        y: p.world[1],
        z: p.world[2],
        has_world: p.has_world,
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVehicleRideTally {
    pub episodes: usize,
    pub named: usize,
    pub lost: usize,
    pub by_event: usize,
    pub by_nearest_event: usize,
    pub by_geometry: usize,
    pub gap_fallbacks: usize,
    pub seats: usize,
    pub rejected: usize,
    pub borders_two: usize,
    pub borders_one: usize,
    pub borders_zero: usize,
    pub film: ReplayFilmVehicleRideTally,
}
impl ReplayVehicleRideTally {
    /// Emit event/geometric attribution observations without changing coverage.
    pub fn log(&self) {
        if self.episodes == 0 && self.gap_fallbacks == 0 {
            return;
        }
        tracing::info!(
            episodesEvenement = self.episodes,
            nommesParLEvenement = self.named,
            resolusParEvenement = self.by_event,
            resolusParVieLaPlusProche = self.by_nearest_event,
            resolusParGeometrie = self.by_geometry,
            nonRattaches = self.lost,
            episodesDeRepli = self.gap_fallbacks,
            siegesLusDansLeFilm = self.seats,
            "rejeu : rattachement des episodes d occupation"
        );
        if self.episodes > 0 && self.named == 0 {
            tracing::warn!(
                episodes = self.episodes,
                "rejeu : AUCUN episode d occupation nomme par son evenement de sortie — la reference de vehicule (domaine 1, ref 1) n est plus lue, le calque retombe sur la seule geometrie"
            );
        }
    }

    fn borders(&mut self, n: i64) {
        match n {
            2 => self.borders_two += 1,
            1 => self.borders_one += 1,
            _ => self.borders_zero += 1,
        }
    }
}
fn event_near(events: &[VehicleRideEvent], at: u64) -> Option<&VehicleRideEvent> {
    let mut best = None;
    let mut gap = u64::MAX;
    for event in events {
        let distance = event.timestamp_us.abs_diff(at);
        if distance / 1000 <= 2000 && (best.is_none() || distance < gap) {
            best = Some(event);
            gap = distance;
        }
    }
    best
}
fn nearest_vehicle(
    anchor: &ReplayPlayerPosition,
    vehicles: &BTreeMap<u32, Vec<ReplayPlayerPosition>>,
    radius: f64,
) -> Option<u32> {
    let mut best = None;
    let mut distance = 0.;
    for (&slot, points) in vehicles {
        if points.is_empty() {
            continue;
        }
        let mut lo = 0;
        let mut hi = points.len();
        while lo < hi {
            let m = lo + (hi - lo) / 2;
            if points[m].timestamp_us >= anchor.timestamp_us {
                hi = m;
            } else {
                lo = m + 1;
            }
        }
        let mut i = lo;
        if i == points.len()
            || (i > 0
                && points[i - 1].timestamp_us.abs_diff(anchor.timestamp_us)
                    < points[i].timestamp_us.abs_diff(anchor.timestamp_us))
        {
            i -= 1;
        }
        let point = &points[i];
        if point.timestamp_us.abs_diff(anchor.timestamp_us) > 1_000_000 {
            continue;
        }
        let d = super::replay_plan_distance([anchor.x, anchor.y], [point.x, point.y]);
        if d <= radius && (best.is_none() || d < distance) {
            best = Some(slot);
            distance = d;
        }
    }
    best
}
fn life_at(lives: &[ReplayVehicleLife], slot: u32, at: u64) -> Option<&ReplayVehicleLife> {
    lives
        .iter()
        .find(|l| l.slot == slot && at >= l.lo_us && at <= l.hi_us)
}
// Resolution codes match the native internal enum: event, nearest event, geometry.
fn resolve_episode<'a>(
    ep: &ReplayVehicleEpisode,
    points: &[ReplayPlayerPosition],
    context: &'a VehicleRideValues<'_>,
) -> Option<(&'a ReplayVehicleLife, u8)> {
    if ep.vehicle_valid {
        let mut best = None;
        let mut gap = u64::MAX;
        for life in context.lives.iter().filter(|l| l.slot == ep.vehicle_slot) {
            if ep.vehicle_at_us >= life.lo_us && ep.vehicle_at_us <= life.hi_us {
                best = Some((life, 1));
                break;
            }
            let g = if ep.vehicle_at_us < life.lo_us {
                life.lo_us - ep.vehicle_at_us
            } else {
                ep.vehicle_at_us - life.hi_us
            };
            if best.is_none() || g < gap {
                best = Some((life, 2));
                gap = g;
            }
        }
        if let Some((life, source)) = best.filter(|(l, _)| context.drawable.contains(&l.key())) {
            return Some((life, source));
        }
    }
    for (at, after) in [(ep.start_us, false), (ep.end_us, true)] {
        if after && ep.open_end {
            break;
        }
        if let Some(anchor) = replay_vehicle_anchor_at(points, at, after)
            && let Some(slot) = nearest_vehicle(anchor, context.vehicles, 3.)
            && let Some(life) = life_at(context.lives, slot, at)
        {
            return Some((life, 3));
        }
    }
    None
}
fn ride<P, A>(
    slot: u32,
    start: u64,
    end: u64,
    context: &ReplayVehicleRideContext<'_, P, A>,
    step: NonZeroU64,
    aim: impl Fn(&[A], u64, u64, u64, u64, i64) -> Vec<ReplayVehicleAim>,
) -> ReplayVehicleRide {
    let t0 = super::replay_vehicle_shots::frame(start, context.origin_us, step, context.frames);
    let t1 =
        super::replay_vehicle_shots::frame(end, context.origin_us, step, context.frames).max(t0);
    ReplayVehicleRide {
        slot,
        t0,
        t1,
        src: "proximity".into(),
        xuid: match context.identity.xuid_at(slot, start) {
            0 => String::new(),
            xuid => xuid.to_string(),
        },
        aim: aim(
            context.aim_by_slot.get(&slot).map_or(&[], Vec::as_slice),
            start,
            end,
            context.origin_us,
            context.step_us,
            context.frames,
        ),
        ..Default::default()
    }
}
/// Written occupancy wins. Resolved event episodes suppress overlapping gaps
/// even when explicit occupancy contradicts their publication.
pub fn build_replay_vehicle_rides(
    context: ReplayVehicleRidesContext<'_>,
) -> (
    BTreeMap<EquipmentLifeKey, Vec<ReplayVehicleRide>>,
    ReplayVehicleRideTally,
) {
    let vehicles = context
        .vehicles
        .iter()
        .map(|(&slot, points)| (slot, points.iter().map(|p| p.position.clone()).collect()))
        .collect();
    let events: Vec<_> = context.events.iter().map(VehicleRideEvent::from).collect();
    vehicle_ride_values(
        VehicleRideValues {
            bipeds: context.film.bipeds,
            events: &events,
            vehicles: &vehicles,
            lives: context.film.lives,
            drawable: context.film.drawable,
            occupancy: context.film.occupancy,
            origin_us: context.film.origin_us,
            step_us: context.film.step_us,
            frames: context.film.frames,
        },
        || build_replay_film_vehicle_rides(context.film),
        |slot, start, end, step| {
            ride(
                slot,
                start,
                end,
                &context.film,
                step,
                build_replay_vehicle_ride_aim,
            )
        },
    )
}
/// Combine cached written occupancy, events and geometric gaps with native precedence.
/// Position projections contain values only; cache events are never narrowed into
/// recording DTOs or given fabricated source coordinates.
pub fn build_facts_replay_vehicle_rides(
    context: FactsReplayVehicleRidesContext<'_>,
) -> (
    BTreeMap<EquipmentLifeKey, Vec<ReplayVehicleRide>>,
    ReplayVehicleRideTally,
) {
    let bipeds: Vec<_> = context
        .film
        .bipeds
        .iter()
        .map(facts_ride_position)
        .collect();
    let vehicles = context
        .vehicles
        .iter()
        .map(|(&slot, points)| (slot, points.iter().map(facts_ride_position).collect()))
        .collect();
    let events: Vec<_> = context.events.iter().map(VehicleRideEvent::from).collect();
    vehicle_ride_values(
        VehicleRideValues {
            bipeds: &bipeds,
            events: &events,
            vehicles: &vehicles,
            lives: context.film.lives,
            drawable: context.film.drawable,
            occupancy: context.film.occupancy,
            origin_us: context.film.origin_us,
            step_us: context.film.step_us,
            frames: context.film.frames,
        },
        || build_facts_replay_film_vehicle_rides(context.film),
        |slot, start, end, step| {
            ride(
                slot,
                start,
                end,
                &context.film,
                step,
                build_facts_replay_vehicle_ride_aim,
            )
        },
    )
}
fn vehicle_ride_values(
    context: VehicleRideValues<'_>,
    written: impl FnOnce() -> (ReplayFilmVehicleRides, ReplayFilmVehicleRideTally),
    make_ride: impl Fn(u32, u64, u64, NonZeroU64) -> ReplayVehicleRide,
) -> (
    BTreeMap<EquipmentLifeKey, Vec<ReplayVehicleRide>>,
    ReplayVehicleRideTally,
) {
    let mut tally = ReplayVehicleRideTally::default();
    let Some(step) = NonZeroU64::new(context.step_us) else {
        return (BTreeMap::new(), tally);
    };
    if context.lives.is_empty() {
        return (BTreeMap::new(), tally);
    }
    let (explicit, film_tally) = written();
    tally.film = film_tally;
    let mut out = explicit.rides.clone();
    let mut boards = BTreeMap::<u32, Vec<VehicleRideEvent>>::new();
    let mut exits = BTreeMap::<u32, Vec<VehicleRideEvent>>::new();
    for event in context
        .events
        .iter()
        .filter(|e| e.occupant_present && e.occupant_in_band)
    {
        let target = match event.kind {
            8 => &mut boards,
            22 => &mut exits,
            _ => continue,
        };
        target.entry(event.occupant_slot).or_default().push(*event);
    }
    for list in boards.values_mut().chain(exits.values_mut()) {
        list.sort_by_key(|e| e.timestamp_us);
    }
    let mut points = BTreeMap::<u32, Vec<ReplayPlayerPosition>>::new();
    for point in context.bipeds.iter().filter(|p| p.has_world) {
        points.entry(point.slot).or_default().push(point.clone());
    }
    for list in points.values_mut() {
        list.sort_by_key(|p| p.timestamp_us);
    }
    let mut kept = Vec::new();
    for mut ep in vehicle_event_episode_values(&boards, &exits, &points) {
        tally.episodes += 1;
        tally.named += usize::from(ep.vehicle_valid);
        let Some((life, source)) = resolve_episode(
            &ep,
            points.get(&ep.slot).map_or(&[], Vec::as_slice),
            &context,
        ) else {
            tally.lost += 1;
            continue;
        };
        match source {
            1 => tally.by_event += 1,
            2 => tally.by_nearest_event += 1,
            _ => tally.by_geometry += 1,
        }
        if ep.open_end {
            ep.end_us = life.hi_us;
            if ep.reappear_us > ep.start_us && ep.reappear_us < ep.end_us {
                ep.end_us = ep.reappear_us;
            }
        }
        let ride = make_ride(ep.slot, ep.start_us, ep.end_us, step);
        let borders = ep.borders;
        kept.push(ep);
        if explicit.contradicts(life.key(), &ride) {
            tally.rejected += 1;
            continue;
        }
        tally.borders(borders);
        out.entry(life.key()).or_default().push(ride);
    }
    for (&slot, list) in &points {
        for pair in list.windows(2) {
            let (start, end) = (pair[0].timestamp_us, pair[1].timestamp_us);
            if (end - start) / 1000 < 3000 || replay_vehicle_episode_covers(&kept, slot, start, end)
            {
                continue;
            }
            let Some(vehicle) = nearest_vehicle(&pair[0], context.vehicles, 1.5) else {
                continue;
            };
            let Some(life) = life_at(context.lives, vehicle, start) else {
                continue;
            };
            let board = event_near(boards.get(&slot).map_or(&[], Vec::as_slice), start);
            let exit = event_near(exits.get(&slot).map_or(&[], Vec::as_slice), end);
            let ride = make_ride(
                slot,
                board.map_or(start, |e| e.timestamp_us),
                exit.map_or(end, |e| e.timestamp_us),
                step,
            );
            if explicit.contradicts(life.key(), &ride) {
                tally.rejected += 1;
                continue;
            }
            tally.borders(i64::from(board.is_some()) + i64::from(exit.is_some()));
            tally.gap_fallbacks += 1;
            out.entry(life.key()).or_default().push(ride);
        }
    }
    tally.seats = assign_replay_vehicle_seats(
        &mut out,
        context.occupancy,
        context.origin_us,
        context.step_us,
        context.frames,
    );
    for list in out.values_mut() {
        list.sort_by_key(|r| (r.t0, r.slot));
    }
    (out, tally)
}
