//! Native non-weapon pickup origins from map points and bounded dropped placements.
use super::ReplayPlayerPosition;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PICKUP_ORIGIN_MATCH_M: f64 = 1.0;
pub const PICKUP_ORIGIN_POSITION_MAX_US: u64 = 100_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MapSpawnPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub kind: String,
}
/// Projection of a published equipment placement, on the replay frame clock.
/// Movement observations alone cannot establish its origin or disappearance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PickupOriginPlacement {
    pub t0: i64,
    pub until_max: i64,
    pub end: String,
    pub origin: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[derive(Debug, Clone)]
pub struct PickupOriginJudge {
    pub state: String,
    points: Vec<MapSpawnPoint>,
    positions: BTreeMap<u32, Vec<ReplayPlayerPosition>>,
    dropped: Vec<PickupOriginPlacement>,
}
impl PickupOriginJudge {
    pub fn new(
        state: &str,
        points: Vec<MapSpawnPoint>,
        positions: &[ReplayPlayerPosition],
        placements: &[PickupOriginPlacement],
    ) -> Self {
        let mut by_slot = BTreeMap::<_, Vec<_>>::new();
        for p in positions.iter().filter(|p| p.has_world) {
            by_slot.entry(p.slot).or_default().push(p.clone());
        }
        Self {
            state: if state.is_empty() {
                "map_absent"
            } else {
                state
            }
            .into(),
            points,
            positions: by_slot,
            dropped: placements
                .iter()
                .filter(|p| p.origin == "dropped")
                .cloned()
                .collect(),
        }
    }
    pub fn catalog_points(&self) -> usize {
        self.points.len()
    }
    /// Return (origin, matched point kind), suitable for ReplayPickupOrigin.
    /// Equal time distances select the first input sample, even if it is later.
    pub fn resolve(&self, slot: u32, timestamp_us: u64, frame: i64) -> (String, String) {
        let mut best = None;
        let mut delta = 1_u64 << 62;
        if let Some(samples) = self.positions.get(&slot) {
            for p in samples {
                let d = p.timestamp_us.abs_diff(timestamp_us);
                if d < delta {
                    delta = d;
                    best = Some(p);
                }
            }
        }
        let Some(p) = best.filter(|_| delta <= PICKUP_ORIGIN_POSITION_MAX_US) else {
            return (String::new(), String::new());
        };
        let near = |x: f32, y: f32, z: f32| {
            let dx = (p.x - x) as f64;
            let dy = (p.y - y) as f64;
            let dz = (p.z - z) as f64;
            (dx * dx + dy * dy + dz * dz).sqrt() < PICKUP_ORIGIN_MATCH_M
        };
        for point in &self.points {
            if near(point.x, point.y, point.z) {
                return ("spawner".into(), point.kind.clone());
            }
        }
        for d in &self.dropped {
            if d.t0 > frame || (d.end == "seen" && d.until_max >= 0 && frame > d.until_max) {
                continue;
            }
            if near(d.x, d.y, d.z) {
                return ("ground".into(), String::new());
            }
        }
        (String::new(), String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        state: String,
        resolved_state: String,
        points: Vec<MapSpawnPoint>,
        positions: Vec<ReplayPlayerPosition>,
        placements: Vec<PickupOriginPlacement>,
        queries: Vec<Query>,
    }
    #[derive(Deserialize)]
    struct Query {
        slot: u32,
        ts: u64,
        frame: i64,
        origin: String,
        kind: String,
    }
    #[test]
    fn native_pickup_origin_decisions() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/pickup-origin-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let count = c.points.len();
            let judge = PickupOriginJudge::new(&c.state, c.points, &c.positions, &c.placements);
            assert_eq!(judge.state, c.resolved_state, "state {i}");
            assert_eq!(judge.catalog_points(), count);
            for (n, q) in c.queries.into_iter().enumerate() {
                assert_eq!(
                    judge.resolve(q.slot, q.ts, q.frame),
                    (q.origin, q.kind),
                    "case {i} query {n}"
                );
            }
        }
    }
}
