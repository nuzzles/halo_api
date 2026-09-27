//! Identity construction from decoded film evidence and optional match participants.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub struct IdentityRegistryInput<'a> {
    pub positions: &'a [ReplayPlayerPosition],
    pub creations: &'a [IdentityCreationRecord],
    pub deaths: &'a [IdentityDeath],
    pub replication_indices: &'a PlayerIndexTable,
    pub film_table: &'a ReplayFilmPlayerTable,
    pub bots: &'a [IdentityBot],
    pub fire: &'a [IdentityFireReference],
    pub roster_xuids: &'a [u64],
    pub participants: &'a [IdentityParticipant],
    pub statborg_records: &'a [StatborgRecord],
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityRegistryOutput {
    pub owners: IdentityOwnerOutput,
    pub tables: IdentityTableLinks,
    pub scoreboard: IdentityScoreboardReport,
}
/// Compose one effective index table before constructing owners. Scoreboard,
/// roster elimination and temporal exclusion consume that same table in order.
/// Round boundaries are resolved from the supplied decoded statborg records.
/// Publication of the frame-based identity section is a separate stage.
pub fn build_identity_registry(input: IdentityRegistryInput<'_>) -> IdentityRegistryOutput {
    build_identity_registry_observed(input, None)
}
pub(super) fn build_identity_registry_observed(
    input: IdentityRegistryInput<'_>,
    match_id: Option<&str>,
) -> IdentityRegistryOutput {
    let round_starts = resolve_statborg_round_bounds(input.statborg_records).starts();
    let tables = compose_identity_tables(input.film_table, input.replication_indices);
    if let Some(id) = match_id {
        super::replay_player_document_logs::table(id, &tables.coverage);
    }
    let bot_indices: BTreeSet<_> = input.bots.iter().map(|b| b.film_index).collect();
    let mut owners = build_identity_owners(IdentityOwnerInput {
        positions: input.positions,
        creations: input.creations,
        deaths: input.deaths,
        indices: &tables.table,
        bot_indices: &bot_indices,
        fire: input.fire,
        round_starts_ms: &round_starts,
    });
    let scoreboard = resolve_identity_scoreboard(
        &mut owners.state,
        &owners.creation,
        IdentityScoreboardInput {
            participants: input.participants,
            bots: input.bots,
            indices: &tables.table,
            clock: &owners.clock,
        },
    );
    owners.state.resolve_roster_elimination(
        input.roster_xuids,
        &tables.table.by_xuid,
        owners.deaths_named,
    );
    owners.state.resolve_temporal_exclusion(
        input.roster_xuids,
        &tables.table.by_xuid,
        !input.bots.is_empty(),
    );
    IdentityRegistryOutput {
        owners,
        tables,
        scoreboard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        statborg_records: Vec<StatborgRecord>,
        positions: Vec<ReplayPlayerPosition>,
        creations: Vec<IdentityCreationRecord>,
        deaths: Vec<IdentityDeath>,
        fire: Vec<IdentityFireReference>,
        bots: Vec<IdentityBot>,
        film_table: ReplayFilmPlayerTable,
        roster_xuids: Vec<u64>,
        participants: Vec<IdentityParticipant>,
        indices: PlayerIndexTable,
        output: IdentityRegistryOutput,
        health: IdentityBridgeHealth,
        clock: IdentityClock,
        section: serde_json::Value,
        statborg: IdentityStatborgPublication,
        total: IdentityLinkCounts,
        verdict: String,
        logs: Vec<serde_json::Value>,
    }
    #[test]
    fn native_registry_construction_order() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-registry-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let section = build_identity_section(
                &c.output,
                IdentitySectionInput {
                    clock: c.clock,
                    deaths: &c.deaths,
                    bots: &c.bots,
                    roster_xuids: &c.roster_xuids,
                    statborg: &c.statborg,
                    statborg_records: &c.statborg_records,
                },
            );
            assert_eq!(
                serde_json::to_value(&section).unwrap(),
                c.section,
                "section {i}"
            );
            assert_eq!(
                super::super::log_test_support::capture_logs(|| c
                    .output
                    .log(&format!("registry-{i}"), &section.coverage)),
                c.logs,
                "registry logs {i}"
            );
            assert_eq!(section.coverage.total(), c.total, "coverage total {i}");
            assert_eq!(c.output.bridge_health(), c.health, "health {i}");
            assert_eq!(c.health.verdict(), c.verdict, "verdict {i}");
            assert_eq!(
                build_identity_registry(IdentityRegistryInput {
                    positions: &c.positions,
                    creations: &c.creations,
                    deaths: &c.deaths,
                    fire: &c.fire,
                    bots: &c.bots,
                    film_table: &c.film_table,
                    roster_xuids: &c.roster_xuids,
                    participants: &c.participants,
                    replication_indices: &c.indices,
                    statborg_records: &c.statborg_records,
                }),
                c.output,
                "owner construction {i}"
            );
        }
    }
}
