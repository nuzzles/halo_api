//! Managed-property zone channels on the published frame axis, before attribution.
use super::ManagedPropertyRead;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayZoneSample {
    pub t: i64,
    pub v: u64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayZoneSeries {
    pub gauge: BTreeMap<u32, Vec<ReplayZoneSample>>,
    pub owner: BTreeMap<u32, Vec<ReplayZoneSample>>,
    pub keys: BTreeMap<u32, u32>,
    pub desig: BTreeMap<u32, Vec<ReplayZoneSample>>,
    pub owner_chained: BTreeMap<u32, Vec<ReplayZoneSample>>,
    pub slots: usize,
}
impl ReplayZoneSeries {
    pub fn from_reads(reads: &[ManagedPropertyRead], origin: u64, step: u64, frames: i64) -> Self {
        let mut out = Self::default();
        let mut seen = BTreeSet::new();
        for r in reads {
            if r.field != 0 || !r.has_value {
                continue;
            }
            // Speaking slots include out-of-window reads and unrelated scalar tags.
            seen.insert(r.slot);
            if r.timestamp_us < origin || step == 0 {
                continue;
            }
            let t = ((r.timestamp_us - origin) / step) as i64;
            if t >= frames {
                continue;
            }
            let sample = ReplayZoneSample { t, v: r.value };
            match r.tag {
                3 => out.gauge.entry(r.slot).or_default().push(sample),
                4 => {
                    out.owner.entry(r.slot).or_default().push(sample);
                    if r.chained {
                        out.owner_chained.entry(r.slot).or_default().push(sample);
                    }
                }
                5 => {
                    out.keys.insert(r.slot, r.value as u32);
                    if r.chained {
                        out.desig.entry(r.slot).or_default().push(sample);
                    }
                }
                _ => {}
            }
        }
        out.slots = seen.len();
        for map in [
            &mut out.gauge,
            &mut out.owner,
            &mut out.desig,
            &mut out.owner_chained,
        ] {
            for samples in map.values_mut() {
                samples.sort_by_key(|s| s.t);
            }
        }
        out
    }
    pub fn ramps(&self) -> Vec<ReplayZoneRamp> {
        let mut out = Vec::new();
        for (&slot, samples) in &self.gauge {
            let mut i = 0;
            while i < samples.len() {
                let mut j = i;
                while j + 1 < samples.len() && samples[j + 1].v >= samples[j].v {
                    j += 1;
                }
                if j - i + 1 >= 3 && samples[j].v - samples[i].v >= 4096 {
                    out.push(ReplayZoneRamp {
                        slot,
                        t0: samples[i].t,
                        t_peak: samples[j].t,
                        start: samples[i].v,
                        top: samples[j].v,
                    });
                }
                i = j + 1;
            }
        }
        out.sort_by_key(|r| r.t_peak);
        out
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayZoneRamp {
    pub slot: u32,
    pub t0: i64,
    pub t_peak: i64,
    pub start: u64,
    pub top: u64,
}
pub fn replay_zone_window_frames(interval_ms: i64) -> i64 {
    if interval_ms <= 0 {
        1
    } else {
        (2000 / interval_ms).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        reads: Vec<ManagedPropertyRead>,
        origin: u64,
        step: u64,
        frames: i64,
        interval: i64,
        window: i64,
        series: ReplayZoneSeries,
        ramps: Vec<ReplayZoneRamp>,
    }
    #[test]
    fn native_zone_series_and_ramps() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-zone-series-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for (i, c) in cases.iter().enumerate() {
            let got = ReplayZoneSeries::from_reads(&c.reads, c.origin, c.step, c.frames);
            assert_eq!(got, c.series, "series {i}");
            assert_eq!(got.ramps(), c.ramps, "ramps {i}");
            assert_eq!(replay_zone_window_frames(c.interval), c.window);
        }
    }
}
