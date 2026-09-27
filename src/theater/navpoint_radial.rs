//! Native radial-progress segments and monotonic rises, before mode interpretation.
use super::native_sort;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NavpointRadialRead {
    pub slot: u32,
    #[serde(rename = "TMS")]
    pub time_ms: i32,
    pub q: u8,
    pub chained: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavpointSegment {
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(rename = "StartMS")]
    pub start_ms: i32,
    #[serde(rename = "EndMS")]
    pub end_ms: i32,
    #[serde(rename = "QStart")]
    pub q_start: u8,
    #[serde(rename = "QEnd")]
    pub q_end: u8,
    #[serde(rename = "QMin")]
    pub q_min: u8,
    #[serde(rename = "QMax")]
    pub q_max: u8,
    #[serde(rename = "Samples")]
    pub samples: usize,
}
impl NavpointSegment {
    pub fn ends_at_summit(&self) -> bool {
        self.samples >= 3
            && i32::from(self.q_end) >= i32::from(self.q_max) - 4
            && i32::from(self.q_end) - i32::from(self.q_min) >= 16
    }
    pub fn is_disarm_hold(&self) -> bool {
        if self.samples < 2
            || self.q_end >= self.q_start
            || i32::from(self.q_max) > i32::from(self.q_start) + 4
        {
            return false;
        }
        let duration = self.end_ms.wrapping_sub(self.start_ms) as f64 / 1000.0;
        duration > 0.0 && f64::from(self.q_start - self.q_end) / duration < 60.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavpointRise {
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(rename = "StartMS")]
    pub start_ms: i32,
    #[serde(rename = "EndMS")]
    pub end_ms: i32,
    #[serde(rename = "QStart")]
    pub q_start: u8,
    #[serde(rename = "QEnd")]
    pub q_end: u8,
    #[serde(rename = "Samples")]
    pub samples: usize,
}
fn series(reads: &[NavpointRadialRead]) -> BTreeMap<u32, Vec<NavpointRadialRead>> {
    let mut out = BTreeMap::<u32, Vec<NavpointRadialRead>>::new();
    for r in reads {
        out.entry(r.slot).or_default().push(*r);
    }
    for s in out.values_mut() {
        native_sort::sort_by(s, |a, b| a.time_ms.cmp(&b.time_ms));
    }
    out
}
pub fn navpoint_radial_segments(reads: &[NavpointRadialRead]) -> Vec<NavpointSegment> {
    let mut out = Vec::new();
    for (slot, s) in series(reads) {
        let mut i = 0;
        while i < s.len() {
            let mut j = i;
            while j + 1 < s.len() && s[j + 1].time_ms.wrapping_sub(s[j].time_ms) <= 500 {
                j += 1;
            }
            out.push(NavpointSegment {
                slot,
                start_ms: s[i].time_ms,
                end_ms: s[j].time_ms,
                q_start: s[i].q,
                q_end: s[j].q,
                q_min: s[i..=j].iter().map(|r| r.q).min().unwrap(),
                q_max: s[i..=j].iter().map(|r| r.q).max().unwrap(),
                samples: j - i + 1,
            });
            i = j + 1;
        }
    }
    native_sort::sort_by(&mut out, |a, b| (a.end_ms, a.slot).cmp(&(b.end_ms, b.slot)));
    out
}
pub fn navpoint_contiguous_rises(reads: &[NavpointRadialRead]) -> Vec<NavpointRise> {
    let mut out = Vec::new();
    for (slot, s) in series(reads) {
        let mut i = 0;
        while i < s.len() {
            let mut j = i;
            while j + 1 < s.len()
                && s[j + 1].q >= s[j].q
                && s[j + 1].time_ms.wrapping_sub(s[j].time_ms) <= 500
            {
                j += 1;
            }
            if j - i + 1 >= 3 && i32::from(s[j].q) - i32::from(s[i].q) >= 16 {
                out.push(NavpointRise {
                    slot,
                    start_ms: s[i].time_ms,
                    end_ms: s[j].time_ms,
                    q_start: s[i].q,
                    q_end: s[j].q,
                    samples: j - i + 1,
                });
            }
            i = if j == i { i + 1 } else { j };
        }
    }
    native_sort::sort_by(&mut out, |a, b| {
        (a.end_ms, a.slot, a.start_ms, a.q_start, a.q_end, a.samples)
            .cmp(&(b.end_ms, b.slot, b.start_ms, b.q_start, b.q_end, b.samples))
    });
    out
}
