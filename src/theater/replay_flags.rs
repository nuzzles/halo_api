//! Complete flag carry layer assembly before managed-property return gauges.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
fn flag_zero(n: &usize) -> bool {
    *n == 0
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayFlagCoverage {
    pub flag_film: bool,
    pub bursts: usize,
    pub captures: usize,
    pub steals: usize,
    pub openings: usize,
    pub carries: usize,
    pub closed: usize,
    pub open: usize,
    pub no_bridge: usize,
    pub no_track: usize,
    pub out_of_window: usize,
    pub ambiguous_slot: usize,
    pub marker_observed: usize,
    pub marker_confirmed: usize,
    pub open_observed: usize,
    pub open_confirmed: usize,
    pub overlaps: usize,
    pub closed_overlaps: usize,
    pub ambiguous_carrier_kills: usize,
    pub ambiguous_returns: usize,
    pub home_by_object: usize,
    pub ambiguous_homecomings: usize,
    pub neutral_flag: bool,
    pub neutral_births: usize,
    pub team_births: usize,
    pub spawns: usize,
    pub object_lives: usize,
    pub closed_by_object: usize,
    #[serde(default, skip_serializing_if = "flag_zero")]
    pub closed_by_handoff: usize,
    #[serde(default, skip_serializing_if = "flag_zero")]
    pub carrier_team_unknown: usize,
    #[serde(default, skip_serializing_if = "flag_zero")]
    pub closed_by_return: usize,
    #[serde(default, skip_serializing_if = "flag_zero")]
    pub closed_by_home: usize,
    pub drops_repositioned: usize,
    pub assigned_by_play: usize,
    pub drops_withheld: usize,
    pub own_flag_refused: usize,
    pub unresolved: usize,
    pub gauge_scanned: bool,
    pub gauge_slots: usize,
    pub gauge_reads: usize,
    pub gauge_paired: usize,
    pub gauge_spans: usize,
    pub gauge_points: usize,
}
impl ReplayFlagCoverage {
    pub fn balanced(&self) -> bool {
        self.carries + self.no_bridge + self.no_track + self.out_of_window == self.openings
            && self.closed + self.open == self.carries
            && self.closed_by_handoff
                + self.closed_by_return
                + self.closed_by_home
                + self.closed_by_object
                <= self.closed
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayFlags {
    pub raw: Vec<ReplayFlagCarryRaw>,
    pub carries: Vec<ReplayFlagCarry>,
    pub coverage: Option<ReplayFlagCoverage>,
    pub tracks_without_bridge: usize,
    pub drops_using_pickup: usize,
}
pub struct ReplayFlagScan<'a, M = CarrierMarkScan> {
    pub scanned: bool,
    pub signals: ReplayFlagFilmSignals,
    pub events: &'a [StatborgNamedEvent],
    pub identity: &'a StatborgRoundIdentity,
    pub teams: &'a BTreeMap<String, i64>,
    pub marks: &'a M,
    pub spawns: &'a [ReplayFlagSpawn],
    pub free: &'a [FreeObjectiveLife],
}
pub type FactsReplayFlagScan<'a> = ReplayFlagScan<'a, FactsCarrierMarkScan>;
pub struct ReplayFlagContext<'a, D = IdentityDeath> {
    pub clock: ReplayMatchClock,
    pub tracks: &'a [ReplayTrack],
    pub deaths: &'a [D],
    pub bridge: &'a BTreeMap<u32, u64>,
    pub ambiguous_slots: &'a BTreeMap<u32, bool>,
}
pub fn build_replay_flags(scan: ReplayFlagScan<'_>, ctx: ReplayFlagContext<'_>) -> ReplayFlags {
    build_flag_values(
        scan,
        ctx,
        mark_replay_flag_carries,
        bound_replay_flag_carries,
    )
}
pub fn build_facts_replay_flags(
    scan: FactsReplayFlagScan<'_>,
    ctx: ReplayFlagContext<'_>,
) -> ReplayFlags {
    build_flag_values(
        scan,
        ctx,
        mark_facts_replay_flag_carries,
        bound_replay_flag_carries,
    )
}
pub(super) fn build_flag_values<M, D>(
    scan: ReplayFlagScan<'_, M>,
    ctx: ReplayFlagContext<'_, D>,
    mark: impl FnOnce(&mut [ReplayFlagCarryRaw], &M, &BTreeMap<u32, u64>, i64),
    bound: impl FnOnce(
        &[ReplayFlagOpening],
        &[StatborgNamedEvent],
        &[D],
        ReplayMatchClock,
    ) -> Vec<ReplayFlagCarryRaw>,
) -> ReplayFlags {
    if !scan.scanned {
        return ReplayFlags::default();
    }
    let choice = choose_replay_flag_spawns(scan.spawns, scan.free);
    let spawns = &choice.spawns;
    let mut cov = ReplayFlagCoverage {
        flag_film: scan.signals.is_flag_film(),
        bursts: scan.signals.bursts,
        captures: scan.signals.captures,
        steals: scan.signals.steals,
        spawns: spawns.len(),
        neutral_flag: choice.neutral,
        neutral_births: choice.neutral_births,
        team_births: choice.team_births,
        ..Default::default()
    };
    if !cov.flag_film {
        return ReplayFlags {
            coverage: Some(cov),
            ..Default::default()
        };
    }
    let openings = replay_flag_openings(scan.events, scan.identity);
    cov.openings = openings.len();
    let unresolved: Vec<_> = openings
        .iter()
        .filter(|o| o.xuid.is_empty())
        .map(|o| o.slot)
        .collect();
    if !unresolved.is_empty() {
        super::replay_diagnostic_sink::emit_replay_diagnostic(&StatborgDiagnostic {
            level: "INFO".into(),
            message: "rejeu : prises de drapeau sans pont d'identite".into(),
            attributes: vec![
                ("slots".into(), serde_json::json!(unresolved)),
                ("sansPont".into(), serde_json::json!(unresolved.len())),
                ("prises".into(), serde_json::json!(openings.len())),
            ],
        });
    }
    let named: Vec<_> = openings
        .into_iter()
        .filter(|o| !o.xuid.is_empty())
        .collect();
    cov.no_bridge = cov.openings - named.len();
    let mut raw = bound(&named, scan.events, ctx.deaths, ctx.clock);
    cov.carrier_team_unknown = close_replay_flags_by_handoff(&mut raw, &named, spawns, scan.teams);
    let returns = replay_flag_returns(scan.events, scan.identity, spawns, scan.teams);
    let homes = replay_flag_object_homecomings(scan.free, spawns, ctx.clock);
    close_replay_flags_by_homecoming(&mut raw, &returns, &homes, spawns, scan.teams);
    cov.ambiguous_carrier_kills =
        close_replay_flags_by_carrier_kills(&mut raw, scan.events, scan.identity);
    let index = replay_flag_carrier_tracks(ctx.tracks, ctx.bridge, ctx.ambiguous_slots);
    close_replay_flags_by_free_lives(&mut raw, &index, scan.free, spawns, ctx.clock);
    let (mut raw, position) = attach_replay_flag_positions(raw, &index, ctx.clock);
    cov.ambiguous_slot = position.ambiguous_slot;
    cov.no_track = position.no_track;
    cov.out_of_window = position.out_of_window;
    cov.drops_repositioned = reposition_replay_flag_drops(&mut raw, scan.free, spawns, ctx.clock);
    let times: Vec<_> = returns.iter().map(|h| h.at).collect();
    let assignment = assign_replay_flags(&mut raw, spawns, scan.teams, &times, &homes);
    cov.own_flag_refused = assignment.own_flag_refused;
    cov.unresolved = assignment.unresolved;
    cov.assigned_by_play = assignment.assigned_by_play;
    mark(&mut raw, scan.marks, ctx.bridge, ctx.clock.death_offset_ms);
    let tally = tally_replay_flag_carries(&raw);
    macro_rules! copy {($src:ident,$($field:ident),*)=>{$(cov.$field=$src.$field;)*}}
    copy!(
        tally,
        carries,
        closed,
        open,
        marker_observed,
        marker_confirmed,
        open_observed,
        open_confirmed,
        closed_by_handoff,
        closed_by_return,
        closed_by_home,
        closed_by_object,
        overlaps,
        closed_overlaps
    );
    let lives = assemble_replay_flag_lives(&raw, spawns, &returns, &homes, ctx.clock);
    let life_cov = lives.coverage;
    copy!(
        life_cov,
        drops_withheld,
        ambiguous_returns,
        home_by_object,
        ambiguous_homecomings
    );
    // Native builds the carrier index once for position attachment and once more
    // for free-life closure when free lives exist. Each rejected track is counted.
    ReplayFlags {
        raw,
        carries: lives.carries,
        coverage: Some(cov),
        tracks_without_bridge: index.without_bridge * (1 + usize::from(!scan.free.is_empty())),
        drops_using_pickup: position.drop_uses_pickup,
    }
}
