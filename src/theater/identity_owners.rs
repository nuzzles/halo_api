//! Native owner construction from decoded evidence, before scoreboard deductions.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub struct IdentityOwnerInput<'a> {
    pub positions: &'a [ReplayPlayerPosition],
    pub creations: &'a [IdentityCreationRecord],
    pub deaths: &'a [IdentityDeath],
    /// Effective, injective table after film/replication composition.
    pub indices: &'a PlayerIndexTable,
    pub bot_indices: &'a BTreeSet<i64>,
    pub fire: &'a [IdentityFireReference],
    /// Round starts resolved from statborg, on the match clock in milliseconds.
    pub round_starts_ms: &'a [i64],
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityOwnerOutput {
    pub state: ReplayIdentityState,
    pub creation: IdentityCreationReport,
    pub verification: IdentityDeathVerification,
    pub clock: IdentityDeathClock,
    pub closures: IdentityClosureReport,
    pub from_deaths: usize,
    pub deaths_named: usize,
    pub lives_total: usize,
    pub index_readings: i64,
    pub index_disagreements: i64,
    pub replication_gap_fallbacks: usize,
}
/// Preserve the native order: name initial stays, calibrate once, refine lives,
/// name again, verify deaths, then complete the bridge and apply closures.
pub fn build_identity_owners(input: IdentityOwnerInput<'_>) -> IdentityOwnerOutput {
    let mut out = IdentityOwnerOutput::default();
    if input.positions.is_empty() || input.indices.by_xuid.is_empty() {
        return out;
    }
    let mut lives = build_identity_life_spans(input.positions);
    name_lives_from_creations(
        &mut lives,
        input.creations,
        input.indices,
        input.bot_indices,
    );
    out.clock = calibrate_identity_death_clock(&lives, input.deaths);
    let pairs = match_identity_deaths(&lives, input.deaths, out.clock.offset_ms);
    mark_identity_deaths(&mut lives, &pairs);
    let refined = refine_identity_lives(
        &lives,
        &IdentityLifetimeBoundaries {
            creations: input.creations.to_vec(),
            rounds_us: input
                .round_starts_ms
                .iter()
                .map(|ms| ms.wrapping_add(out.clock.offset_ms).wrapping_mul(1000))
                .collect(),
            deaths_by_player: identity_deaths_by_player(input.deaths, out.clock.offset_ms),
        },
    );
    lives = refined.lives;
    out.replication_gap_fallbacks = refined.replication_gap_fallbacks;
    out.lives_total = lives.len();
    out.creation = name_lives_from_creations(
        &mut lives,
        input.creations,
        input.indices,
        input.bot_indices,
    );
    let pairs = match_identity_deaths(&lives, input.deaths, out.clock.offset_ms);
    mark_identity_deaths(&mut lives, &pairs);
    out.verification = verify_identity_deaths(&lives, input.deaths, &pairs);
    if out.creation.slots == 0 {
        out.verification.named_by_bridge =
            name_identity_lives_by_deaths(&mut lives, input.deaths, &pairs);
    }
    out.deaths_named = out.verification.matched;
    if out.creation.named_lives() == 0 && out.verification.named_by_bridge == 0 {
        out.state = ReplayIdentityState::from_lives(lives, &BTreeMap::new());
        return out;
    }
    out.index_readings = input.indices.readings;
    out.index_disagreements = input.indices.disagreements;
    out.state = ReplayIdentityState::from_lives(lives.clone(), &input.indices.by_xuid);
    let mut owners = out.state.indices_by_slot().clone();
    for (slot, index) in owners_from_creations(input.creations) {
        owners.entry(slot).or_insert(index);
    }
    out.from_deaths = owners.len();
    let closed = close_identity_bridge(
        &owners,
        IdentityClosureInput {
            positions: input.positions,
            lives: &lives,
            deaths: input.deaths,
            offset_ms: out.clock.offset_ms,
            indices: &input.indices.by_xuid,
            fire: input.fire,
        },
    );
    name_identity_closed_lives(
        &mut lives,
        &closed.owners,
        &closed.report.closed_life,
        &input.indices.by_xuid,
    );
    out.state
        .install_closure_bridge(lives, closed.owners, &input.indices.by_xuid);
    out.closures = closed.report;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        round_starts_ms: Vec<i64>,
        positions: Vec<ReplayPlayerPosition>,
        creations: Vec<IdentityCreationRecord>,
        deaths: Vec<IdentityDeath>,
        fire: Vec<IdentityFireReference>,
        bot_indices: BTreeSet<i64>,
        indices: PlayerIndexTable,
        output: IdentityOwnerOutput,
    }
    #[test]
    fn native_owner_construction_order() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-owners-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                build_identity_owners(IdentityOwnerInput {
                    positions: &c.positions,
                    creations: &c.creations,
                    deaths: &c.deaths,
                    fire: &c.fire,
                    bot_indices: &c.bot_indices,
                    indices: &c.indices,
                    round_starts_ms: &c.round_starts_ms,
                }),
                c.output,
                "owner construction {i}"
            );
        }
    }
}
