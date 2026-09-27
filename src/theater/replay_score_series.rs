//! Replay score emissions on the document's millisecond frame grid.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayScoreClock {
    pub origin_ms: i64,
    pub interval_ms: i64,
    pub frames: i64,
}
impl ReplayScoreClock {
    pub fn frame_of(self, time_ms: i64) -> Option<i64> {
        if self.interval_ms <= 0 || self.frames <= 0 || time_ms < self.origin_ms {
            return None;
        }
        let t = time_ms.wrapping_sub(self.origin_ms) / self.interval_ms;
        (t < self.frames).then_some(t)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayScoreTick {
    pub t: i64,
    pub v: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayScoreRound {
    pub round: i64,
    pub points: Vec<ReplayScoreTick>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayScoreSeries {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rounds: Vec<ReplayScoreRound>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub total: Vec<ReplayScoreTick>,
}
/// Keep the first emission of a plateau and the last changed value within a
/// frame. Filtering is ordered: an out-of-window value cannot start a plateau.
pub fn replay_score_ticks(
    points: &[StatborgScorePoint],
    clock: ReplayScoreClock,
) -> Vec<ReplayScoreTick> {
    let mut out: Vec<ReplayScoreTick> = Vec::new();
    for p in points {
        let Some(t) = clock.frame_of(p.time_ms) else {
            continue;
        };
        if let Some(last) = out.last_mut() {
            if last.v == p.value {
                continue;
            }
            if last.t == t {
                last.v = p.value;
                continue;
            }
        }
        out.push(ReplayScoreTick { t, v: p.value });
    }
    out
}
pub fn replay_score_rounds(
    rounds: &BTreeMap<i64, Vec<StatborgScorePoint>>,
    clock: ReplayScoreClock,
) -> Vec<ReplayScoreRound> {
    rounds
        .iter()
        .filter_map(|(round, points)| {
            let points = replay_score_ticks(points, clock);
            (!points.is_empty()).then_some(ReplayScoreRound {
                round: *round,
                points,
            })
        })
        .collect()
}
/// Recompose an identified player's round segments before applying the clock.
/// Chronological rejection uses the raw cumulative points, before frame clipping.
pub fn replay_score_series_of_rounds(
    rounds: &BTreeMap<i64, Vec<StatborgScorePoint>>,
    clock: ReplayScoreClock,
) -> ReplayScoreSeries {
    score_series_of_rounds_impl(rounds, clock, None)
}
/// Retain the native chronology warning before clock clipping or frame merging.
pub fn replay_score_series_of_rounds_with_diagnostics(
    rounds: &BTreeMap<i64, Vec<StatborgScorePoint>>,
    clock: ReplayScoreClock,
) -> (ReplayScoreSeries, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out = score_series_of_rounds_impl(rounds, clock, Some(&mut diagnostics));
    (out, diagnostics)
}
pub(super) fn score_series_of_rounds_impl(
    rounds: &BTreeMap<i64, Vec<StatborgScorePoint>>,
    clock: ReplayScoreClock,
    diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> ReplayScoreSeries {
    let mut total = Vec::new();
    let mut offset = 0i64;
    for points in rounds.values() {
        for p in points {
            total.push(StatborgScorePoint {
                value: p.value.wrapping_add(offset),
                ..p.clone()
            });
        }
        if let Some(p) = points.last() {
            offset = offset.wrapping_add(p.value);
        }
    }
    ReplayScoreSeries {
        rounds: replay_score_rounds(rounds, clock),
        total: replay_score_ticks(
            &super::statborg_series::statborg_chronological_total_observed(&total, diagnostics),
            clock,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_score_series_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-score-series-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.as_array().unwrap().iter().enumerate() {
            let clock: ReplayScoreClock = serde_json::from_value(row["clock"].clone()).unwrap();
            let points: Vec<StatborgScorePoint> =
                serde_json::from_value(row["points"].clone()).unwrap();
            let rounds: BTreeMap<i64, Vec<StatborgScorePoint>> =
                serde_json::from_value(row["rounds"].clone()).unwrap();
            assert_eq!(
                replay_score_ticks(&points, clock),
                serde_json::from_value::<Vec<ReplayScoreTick>>(row["ticks"].clone()).unwrap(),
                "ticks {i}"
            );
            let (series, diagnostics) =
                replay_score_series_of_rounds_with_diagnostics(&rounds, clock);
            assert_eq!(
                diagnostics,
                serde_json::from_value::<Vec<StatborgDiagnostic>>(row["diagnostics"].clone())
                    .unwrap(),
                "diagnostics {i}"
            );
            assert_eq!(series, replay_score_series_of_rounds(&rounds, clock));
            assert_eq!(
                replay_score_series_of_rounds(&rounds, clock),
                serde_json::from_value::<ReplayScoreSeries>(row["series"].clone()).unwrap(),
                "series {i}"
            );
        }
    }
}
