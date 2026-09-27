//! Geometric objective attribution with explicit missing/ambiguous evidence.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayZoneAttribution {
    pub action: ReplayObjectiveAction,
    pub attributed: bool,
    pub spatial_rank: usize,
    #[serde(rename = "InstanceID")]
    pub instance_id: i32,
    pub sample_gap_frames: i64,
    pub sample: ReplayPoint,
    pub has_sample: bool,
    #[serde(rename = "DistanceM")]
    pub distance_m: f64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayZoneAttributionCoverage {
    pub actions: usize,
    pub attributed: usize,
    pub no_position: usize,
    pub outside: usize,
    pub ambiguous: usize,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayZoneAttributeOptions {
    pub max_gap_frames: i64,
    #[serde(rename = "MaxDistanceM")]
    pub max_distance_m: f64,
}
pub fn replay_zone_samples_by_xuid(
    tracks: &[ReplayTrack],
    bridge: &BTreeMap<u32, u64>,
) -> BTreeMap<String, Vec<ReplayPoint>> {
    let mut out = BTreeMap::<String, Vec<ReplayPoint>>::new();
    for track in tracks {
        if let Some(xuid) = replay_published_track_xuid(track, bridge) {
            out.entry(xuid)
                .or_default()
                .extend_from_slice(track.points());
        }
    }
    for points in out.values_mut() {
        points.sort_by_key(|p| p.t);
    }
    out
}
/// Exact frames select the first equal-frame sample; equal-distance frames select
/// the prior sample (the last point on that prior frame).
pub fn replay_zone_nearest_sample(
    points: &[ReplayPoint],
    frame: i64,
    max_gap: i64,
) -> Option<(&ReplayPoint, i64)> {
    let i = points.partition_point(|p| p.t < frame);
    let mut best = None;
    let mut gap = max_gap.saturating_add(1);
    if i > 0 {
        best = Some(&points[i - 1]);
        gap = frame - points[i - 1].t;
    }
    if i < points.len() && points[i].t - frame < gap {
        best = Some(&points[i]);
        gap = points[i].t - frame;
    }
    if gap > max_gap {
        None
    } else {
        best.map(|p| (p, gap))
    }
}
pub fn attribute_replay_zones(
    actions: &[ReplayObjectiveAction],
    tracks: &[ReplayTrack],
    zones: &[ObjectiveZone],
    bridge: &BTreeMap<u32, u64>,
    options: ReplayZoneAttributeOptions,
) -> (Vec<ReplayZoneAttribution>, ReplayZoneAttributionCoverage) {
    let max_gap = if options.max_gap_frames > 0 {
        options.max_gap_frames
    } else {
        2
    };
    let samples = replay_zone_samples_by_xuid(tracks, bridge);
    let mut out = Vec::with_capacity(actions.len());
    let mut cov = ReplayZoneAttributionCoverage {
        actions: actions.len(),
        ..Default::default()
    };
    for action in actions {
        let mut att = ReplayZoneAttribution {
            action: action.clone(),
            attributed: false,
            spatial_rank: 0,
            instance_id: 0,
            sample_gap_frames: 0,
            sample: ReplayPoint::default(),
            has_sample: false,
            distance_m: 0.,
        };
        let nearest = samples
            .get(&action.xuid)
            .and_then(|p| replay_zone_nearest_sample(p, action.t, max_gap));
        if let Some((point, gap)) = nearest {
            att.sample = point.clone();
            att.sample_gap_frames = gap;
            att.has_sample = true;
            let position = ObjectiveVec3 {
                x: point.x as f64,
                y: point.y as f64,
                z: point.z as f64,
            };
            let mut best = f64::INFINITY;
            let mut hits = Vec::new();
            for zone in zones {
                let d = zone.volume.distance_to(position);
                if d < best {
                    best = d;
                    hits.clear();
                    hits.push(zone);
                } else if d == best {
                    hits.push(zone);
                }
            }
            if !hits.is_empty() {
                att.distance_m = best;
            }
            if hits.is_empty() || best > options.max_distance_m {
                cov.outside += 1;
            } else if hits.len() == 1 {
                att.attributed = true;
                att.spatial_rank = hits[0].spatial_rank;
                att.instance_id = hits[0].instance_id;
                cov.attributed += 1;
            } else {
                cov.ambiguous += 1;
            }
        } else {
            cov.no_position += 1;
        }
        out.push(att);
    }
    (out, cov)
}
pub fn translate_replay_zones(zones: &[ObjectiveZone], delta: ObjectiveVec3) -> Vec<ObjectiveZone> {
    zones
        .iter()
        .map(|z| {
            let mut z = z.clone();
            z.volume = z.volume.translated(delta);
            z.center = ObjectiveVec3 {
                x: z.center.x + delta.x,
                y: z.center.y + delta.y,
                z: z.center.z + delta.z,
            };
            z
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        entry: MapObjectivesEntry,
        actions: Vec<ReplayObjectiveAction>,
        tracks: Vec<ReplayTrack>,
        bridge: BTreeMap<u32, u64>,
        options: ReplayZoneAttributeOptions,
        delta: ObjectiveVec3,
        output: Vec<ReplayZoneAttribution>,
        coverage: ReplayZoneAttributionCoverage,
        translated: Vec<ReplayZoneAttribution>,
        translated_coverage: ReplayZoneAttributionCoverage,
    }
    #[test]
    fn native_zone_attribution() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-zone-attribution-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for (i, c) in cases.iter().enumerate() {
            let zones = c.entry.zones_of_role("strongholds_zone").zones;
            for (zones, expected, coverage) in [
                (zones.clone(), &c.output, &c.coverage),
                (
                    translate_replay_zones(&zones, c.delta),
                    &c.translated,
                    &c.translated_coverage,
                ),
            ] {
                let (got, cov) =
                    attribute_replay_zones(&c.actions, &c.tracks, &zones, &c.bridge, c.options);
                assert_eq!(got.len(), expected.len(), "action count {i}");
                assert_eq!(&cov, coverage, "coverage {i}");
                assert_eq!(
                    cov.actions,
                    cov.attributed + cov.no_position + cov.outside + cov.ambiguous
                );
                for (mut a, e) in got.into_iter().zip(expected) {
                    assert!(
                        (a.distance_m - e.distance_m).abs() <= 1e-12 * e.distance_m.abs().max(1.),
                        "distance {i}"
                    );
                    a.distance_m = e.distance_m;
                    assert_eq!(&a, e, "attribution {i}");
                }
            }
        }
    }
}
