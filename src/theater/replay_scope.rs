//! Native scope episodes, closed by explicit exits, later entries or lifetime bounds.
use super::{BipedZoomEvent, IdentityLife, native_sort};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const REPLAY_ZOOM_HOLD_US: u64 = 3_500_000;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayScopePeriod {
    pub start_us: u64,
    pub end_us: u64,
    pub level: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayScopeLookup {
    pub by_slot: BTreeMap<u32, Vec<ReplayScopePeriod>>,
}
fn bounded_end(start: u64, next: u64, life_end: u64, hold: u64) -> u64 {
    let mut end = start.wrapping_add(hold);
    if next > start && next < end {
        end = next;
    }
    if life_end > start && life_end < end {
        end = life_end;
    }
    end
}
// Exact sort.Search traversal, including malformed nonmonotonic end times.
fn search(n: usize, predicate: impl Fn(usize) -> bool) -> usize {
    let (mut low, mut high) = (0, n);
    while low < high {
        let mid = low + (high - low) / 2;
        if predicate(mid) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    low
}
impl ReplayScopeLookup {
    /// Events retain their supplied order. Explicit exits close at their exact
    /// timestamp, even beyond a life/hold bound. Open periods use the earliest
    /// available later entry, life end or hold deadline. Both ends are inclusive.
    pub fn new(events: &[BipedZoomEvent], lives: &[IdentityLife], hold_us: u64) -> Self {
        Self::from_values(
            events
                .iter()
                .map(|e| (e.slot, e.source.timestamp_us, i64::from(e.level))),
            lives,
            hold_us,
        )
    }
    /// Cached zoom levels are native signed ints, not the narrower wire field.
    /// Nonpositive levels close a scope period; positive levels remain exact.
    pub fn from_facts(
        events: &[super::FactsZoomEvent],
        lives: &[IdentityLife],
        hold_us: u64,
    ) -> Self {
        Self::from_values(
            events.iter().map(|e| (e.slot, e.timestamp_us, e.level)),
            lives,
            hold_us,
        )
    }
    fn from_values(
        events: impl IntoIterator<Item = (u32, u64, i64)>,
        lives: &[IdentityLife],
        hold_us: u64,
    ) -> Self {
        let mut ends = BTreeMap::<u32, Vec<u64>>::new();
        for l in lives {
            ends.entry(l.slot).or_default().push(l.to as u64);
        }
        for list in ends.values_mut() {
            list.sort_unstable();
        }
        let life_end = |slot: u32, at: u64| {
            let Some(list) = ends.get(&slot) else {
                return 0;
            };
            list.get(search(list.len(), |i| list[i] >= at))
                .copied()
                .unwrap_or(0)
        };
        let mut out = Self::default();
        let mut open = BTreeMap::<u32, ReplayScopePeriod>::new();
        let mut close = |slot, mut p: ReplayScopePeriod, end| {
            if end > p.start_us {
                p.end_us = end;
                out.by_slot.entry(slot).or_default().push(p);
            }
        };
        for (slot, timestamp_us, level) in events {
            if level > 0 {
                if let Some(p) = open.remove(&slot) {
                    let end = bounded_end(
                        p.start_us,
                        timestamp_us,
                        life_end(slot, p.start_us),
                        hold_us,
                    );
                    close(slot, p, end);
                }
                open.insert(
                    slot,
                    ReplayScopePeriod {
                        start_us: timestamp_us,
                        end_us: 0,
                        level,
                    },
                );
            } else if let Some(p) = open.remove(&slot) {
                close(slot, p, timestamp_us);
            }
        }
        for (slot, p) in open {
            let end = bounded_end(p.start_us, 0, life_end(slot, p.start_us), hold_us);
            close(slot, p, end);
        }
        for periods in out.by_slot.values_mut() {
            native_sort::sort_by(periods, |a, b| a.start_us.cmp(&b.start_us));
        }
        out
    }
    pub fn at(&self, slot: u32, timestamp_us: u64) -> i64 {
        let Some(periods) = self.by_slot.get(&slot) else {
            return 0;
        };
        let i = search(periods.len(), |i| periods[i].end_us >= timestamp_us);
        periods
            .get(i)
            .filter(|p| timestamp_us >= p.start_us)
            .map_or(0, |p| p.level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Event {
        slot: u32,
        time: u64,
        level: u8,
    }
    #[derive(Deserialize)]
    struct Case {
        events: Vec<Event>,
        lives: Vec<IdentityLife>,
        hold: u64,
        queries: Vec<Event>,
    }
    #[test]
    fn native_scope_lifetime_lookup() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/replay-scope-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in rows.into_iter().enumerate() {
            let events: Vec<_> = c
                .events
                .into_iter()
                .map(|e| BipedZoomEvent {
                    slot: e.slot,
                    level: e.level,
                    source: super::super::FilmPacket {
                        chunk_index: 0,
                        packet_type: 5,
                        byte_2: 0,
                        byte_3: 0,
                        payload_offset: 0,
                        payload_size: 0,
                        timestamp_us: e.time,
                    },
                })
                .collect();
            let lookup = ReplayScopeLookup::new(&events, &c.lives, c.hold);
            for q in c.queries {
                assert_eq!(
                    lookup.at(q.slot, q.time),
                    i64::from(q.level),
                    "case {i}, slot {} time {}",
                    q.slot,
                    q.time
                );
            }
        }
    }
}
