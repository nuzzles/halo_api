//! Cached objective assembly with explicit caller mode and identity precedence.
use super::*;
use std::collections::BTreeMap;

pub struct FactsReplayFlagLayerInput<'a> {
    pub scanned: bool,
    pub records: &'a [StatborgRecord],
    pub bursts: &'a [i64],
    pub identity: &'a StatborgRoundIdentity,
    pub teams: &'a BTreeMap<String, i64>,
    pub marks: &'a FactsCarrierMarkScan,
    pub spawns: &'a [ReplayFlagSpawn],
    /// Unpublished free-object evidence from the cached weapon scan.
    pub free: &'a [FreeObjectiveLife],
    pub gauge: &'a [ManagedPropertyRead],
    pub gauge_scanned: bool,
}

/// Preserve both named-event observation passes even when the mode gate is off.
/// A resolved empty caller identity is authoritative; only unresolved identity
/// falls back to the native death-instant resolver. Raw death names are unused.
pub fn build_facts_replay_flag_layer(
    input: FactsReplayFlagLayerInput<'_>,
    ctx: ReplayFlagContext<'_, FactsDeath>,
) -> (ReplayFlags, Vec<StatborgDiagnostic>) {
    let free = input.free;
    build_facts_replay_flag_layer_with_free(input, ctx, || free.to_vec())
}

pub(super) fn build_facts_replay_flag_layer_with_free(
    input: FactsReplayFlagLayerInput<'_>,
    ctx: ReplayFlagContext<'_, FactsDeath>,
    free: impl FnOnce() -> Vec<FreeObjectiveLife>,
) -> (ReplayFlags, Vec<StatborgDiagnostic>) {
    build_facts_replay_flag_layer_observed(input, ctx, free, |_| {})
}

pub(super) fn build_facts_replay_flag_layer_observed(
    input: FactsReplayFlagLayerInput<'_>,
    ctx: ReplayFlagContext<'_, FactsDeath>,
    free: impl FnOnce() -> Vec<FreeObjectiveLife>,
    observe: impl FnOnce(&[StatborgDiagnostic]),
) -> (ReplayFlags, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let (signals, events) = super::replay_flags_film::flag_named_inputs(
        input.records,
        input.bursts,
        Some(&mut diagnostics),
    );
    observe(&diagnostics);
    let recognized = input.scanned && signals.is_flag_film();
    let resolved;
    let identity = if recognized && !input.identity.resolved() {
        resolved = resolve_facts_objective_identity(input.records, ctx.deaths);
        &resolved
    } else {
        input.identity
    };
    let free = if recognized { free() } else { Vec::new() };
    let clock = ctx.clock;
    let mut out = super::replay_flags::build_flag_values(
        FactsReplayFlagScan {
            scanned: input.scanned,
            signals,
            events: &events,
            identity,
            teams: input.teams,
            marks: input.marks,
            spawns: input.spawns,
            free: &free,
        },
        ctx,
        mark_facts_replay_flag_carries,
        |ops, events, deaths, clock| {
            let deaths: Vec<_> = deaths.iter().map(|d| (d.xuid, d.time_ms)).collect();
            super::replay_flag_carries::bound_flag_values(ops, events, &deaths, clock)
        },
    );
    if recognized {
        if let Some(cov) = out.coverage.as_mut() {
            cov.object_lives = free.len();
        }
        attach_replay_flag_return_gauges(
            &mut out.carries,
            input.gauge,
            input.gauge_scanned,
            clock,
            out.coverage.as_mut(),
        );
    }
    (out, diagnostics)
}

fn resolve_facts_objective_identity(
    records: &[StatborgRecord],
    deaths: &[FactsDeath],
) -> StatborgRoundIdentity {
    let instants: Vec<_> = deaths
        .iter()
        .map(|d| StatborgDeathInstant {
            xuid: d.xuid.to_string(),
            time_ms: d.time_ms,
        })
        .collect();
    resolve_statborg_round_identity(records, &instants)
}

/// Native VIP always resolves identity from these records and death instants;
/// unlike flags and skulls it accepts no completed caller identity override.
pub fn build_facts_replay_vip_crown(
    records: &[StatborgRecord],
    deaths: &[FactsDeath],
    clock: ReplayMatchClock,
    scanned: bool,
) -> (ReplayVipCrown, Vec<StatborgDiagnostic>) {
    if !scanned {
        return (ReplayVipCrown::default(), Vec::new());
    }
    let mut diagnostics = Vec::new();
    let events =
        super::statborg_named::named_events_with_sink(records, "vip", Some(&mut diagnostics));
    let identity = resolve_facts_objective_identity(records, deaths);
    let deaths: Vec<_> = deaths.iter().map(|d| (d.xuid, d.time_ms)).collect();
    (
        super::replay_vip::build_vip_values(true, &events, &identity, &deaths, clock),
        diagnostics,
    )
}

pub struct FactsReplaySkullInput<'a> {
    pub scanned: bool,
    pub records: &'a [StatborgRecord],
    pub identity: &'a StatborgRoundIdentity,
    pub deaths: &'a [FactsDeath],
}
/// Preserve an explicitly resolved caller identity, including an empty one.
/// Resolve deaths locally only for an enabled scan with unresolved identity.
pub fn build_facts_replay_skull_carries(
    input: FactsReplaySkullInput<'_>,
    clock: ReplayMatchClock,
    presence: &ReplayCarrierPresence,
) -> ReplaySkullCarries {
    if !input.scanned {
        return ReplaySkullCarries::default();
    }
    let resolved;
    let identity = if input.identity.resolved() {
        input.identity
    } else {
        resolved = resolve_facts_objective_identity(input.records, input.deaths);
        &resolved
    };
    build_replay_skull_carries(true, input.records, identity, clock, presence)
}
