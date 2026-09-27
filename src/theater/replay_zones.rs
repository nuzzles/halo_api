//! Top-level zone dispatch with explicit scan, geometry, clock and hill gates.
use super::*;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReplayZones {
    pub states: Vec<ReplayZoneState>,
    pub coverage: Option<ReplayZonesCoverage>,
    pub inferred_capture_teams: usize,
    pub hill_fallbacks: ReplayHillFallbacks,
}
pub struct ReplayZoneInput<'a> {
    pub scanned: bool,
    pub reads: &'a [ManagedPropertyRead],
    pub zones: &'a [ObjectiveZone],
    pub roles: &'a str,
    pub teams: &'a BTreeMap<String, i64>,
    pub hill: bool,
}
pub struct ReplayZoneContext<'a> {
    pub origin_us: u64,
    pub step_us: u64,
    pub frames: i64,
    pub interval_ms: i64,
    pub tracks: &'a [ReplayTrack],
    pub actions: &'a [ReplayObjectiveAction],
    pub bridge: &'a BTreeMap<u32, u64>,
}
pub fn build_replay_zones(
    input: ReplayZoneInput<'_>,
    context: ReplayZoneContext<'_>,
) -> ReplayZones {
    let mut out = ReplayZones::default();
    if !input.scanned {
        return out;
    }
    let mut cov = ReplayZonesCoverage {
        method: "captures+geometry".into(),
        roles: input.roles.into(),
        catalog: input.zones.len(),
        ..Default::default()
    };
    if !input.zones.is_empty() && context.frames > 0 && context.step_us != 0 {
        let zones = replay_zone_catalog(input.zones);
        let series = ReplayZoneSeries::from_reads(
            input.reads,
            context.origin_us,
            context.step_us,
            context.frames,
        );
        cov.slots = series.slots;
        let captures: Vec<_> = context
            .actions
            .iter()
            .filter(|a| a.stat == "zone_captures" || a.stat == "zone_secures")
            .cloned()
            .collect();
        cov.captures = captures.len();
        let (attributions, ac) = attribute_replay_zones(
            &captures,
            context.tracks,
            &zones,
            context.bridge,
            ReplayZoneAttributeOptions {
                max_distance_m: 5.,
                ..Default::default()
            },
        );
        cov.no_position = ac.no_position;
        cov.outside = ac.outside;
        cov.ambiguous_zone = ac.ambiguous;
        let pairs = replay_zone_pairs(&attributions);
        cov.attributed = pairs.len();
        if pairs.is_empty() {
            if input.hill {
                (out.states, out.hill_fallbacks) = build_replay_hill_states(
                    &zones,
                    &series,
                    &replay_zone_team_set(input.teams),
                    context.tracks,
                    context.frames,
                    &mut cov,
                );
            }
        } else {
            (out.states, out.inferred_capture_teams) = build_replay_zone_owner_layer(
                ReplayZoneOwnerInput {
                    series: &series,
                    pairs: &pairs,
                    teams: input.teams,
                    frames: context.frames,
                    interval_ms: context.interval_ms,
                    catalog: zones.len(),
                    hill: input.hill,
                },
                &mut cov,
            );
            tally_replay_zone_states(&out.states, &mut cov);
        }
    }
    out.coverage = Some(cov);
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        entry: MapObjectivesEntry,
        reads: Vec<ManagedPropertyRead>,
        tracks: Vec<ReplayTrack>,
        actions: Vec<ReplayObjectiveAction>,
        bridge: BTreeMap<u32, u64>,
        teams: BTreeMap<String, i64>,
        frames: i64,
        interval: i64,
        origin: u64,
        step: u64,
        scanned: bool,
        hill: bool,
        output: ReplayZones,
    }
    #[test]
    fn native_complete_zone_dispatch() {
        let mut b = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/replay-zones-v41.json.zlib")[..])
            .read_to_end(&mut b)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&b).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let zones = c.entry.zones_of_role("strongholds_zone").zones;
            let out = build_replay_zones(
                ReplayZoneInput {
                    scanned: c.scanned,
                    reads: &c.reads,
                    zones: &zones,
                    roles: "strongholds_zone",
                    teams: &c.teams,
                    hill: c.hill,
                },
                ReplayZoneContext {
                    origin_us: c.origin,
                    step_us: c.step,
                    frames: c.frames,
                    interval_ms: c.interval,
                    tracks: &c.tracks,
                    actions: &c.actions,
                    bridge: &c.bridge,
                },
            );
            assert_eq!(out, c.output, "dispatch {i}");
        }
    }
}
