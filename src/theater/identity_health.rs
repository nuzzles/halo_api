//! Native identity bridge health and publication verdicts.
use super::IdentityRegistryOutput;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityBridgeHealth {
    pub slots: usize,
    pub from_reading: usize,
    pub lives_named: usize,
    pub lives_total: usize,
    pub index_readings: i64,
    pub index_disagreements: i64,
    pub slot_collisions: usize,
    pub concordant: usize,
    pub discordant: usize,
    pub bridge_named_lives: usize,
    pub direct_by_creation: usize,
    pub direct_by_creation_propagated: usize,
    pub bodies_with_creation: usize,
    pub named_by_previous_life: usize,
    pub named_by_next_life: usize,
    pub named_by_slot_bridge: usize,
    pub unnamed_lives: usize,
    pub unnamed_lives_contested: usize,
    pub death_offset_matched: usize,
    pub death_offset_runner_up: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_offset_ms: Option<i64>,
    pub closed_by_shot: usize,
    pub closed_by_respawn: usize,
    pub closed_contested: usize,
    pub closed_refused: usize,
}
impl IdentityRegistryOutput {
    /// Native registry summary and refusal observations, using final coverage after
    /// scoreboard and deduction passes. Does not recompute identity resolution.
    pub fn log(&self, match_id: &str, coverage: &super::IdentityCoverage) {
        let total = coverage.total();
        let h = self.bridge_health();
        tracing::info!(
            match_id = match_id,
            slots = h.slots,
            viesNommees = h.lives_named,
            viesTotal = h.lives_total,
            lecturesIndex = h.index_readings,
            desaccordsIndex = h.index_disagreements,
            collisionsSlot = h.slot_collisions,
            parCreation = h.direct_by_creation,
            parCreationPropagee = h.direct_by_creation_propagated,
            corpsAvecCreation = h.bodies_with_creation,
            pontConcordant = h.concordant,
            pontDiscordant = h.discordant,
            viesNommeesParLePont = h.bridge_named_lives,
            parElimination = self.owners.state.eliminated,
            parExclusionTemporelle = self.owners.state.excluded,
            viesSansAucunCandidat = self.owners.state.excluded_contradictions,
            parTableauAPI = self.scoreboard.named(),
            conflitsAuTableau = self.scoreboard.conflicts,
            sansCandidatAuTableau = self.scoreboard.no_candidate,
            liensDirects = total.direct,
            liensDeduits = total.inferred,
            liensNonResolus = total.unresolved,
            "rejeu : registre d'identite"
        );
        self.owners
            .creation
            .log_refusals(match_id, &coverage.biped_slot.unresolved_by_cause);
        if h.discordant > 0 {
            tracing::warn!(
                match_id = match_id,
                discordances = h.discordant,
                concordances = h.concordant,
                "rejeu : le pont par morts contredit le lien direct sur des vies — le film fait foi, les noms du pont sont ecartes"
            );
        }
        if h.bridge_named_lives > 0 {
            tracing::warn!(
                match_id = match_id,
                vies = h.bridge_named_lives,
                "rejeu : AUCUNE lecture directe corps -> joueur — le pont par morts a nomme en degradation complete (cf. identity_registry_bridge.go)"
            );
        }
        if h.index_disagreements > 0 {
            tracing::warn!(
                match_id = match_id,
                desaccords = h.index_disagreements,
                "rejeu : desaccord de lecture de l'index de joueur — liens directs NON publies"
            );
        }
        if total.unresolved > 0 {
            tracing::warn!(
                match_id = match_id,
                liens = total.unresolved,
                "rejeu : liens d'identite NON RESOLUS — publies et comptes, jamais inventes"
            );
        }
    }
    /// Health before the separate final track-naming repair stage. A measured
    /// zero clock offset remains present; an unmatched clock is absent.
    pub fn bridge_health(&self) -> IdentityBridgeHealth {
        let o = &self.owners;
        IdentityBridgeHealth {
            slots: o.state.indices_by_slot().len(),
            from_reading: o.from_deaths,
            lives_named: o.deaths_named,
            lives_total: o.lives_total,
            index_readings: o.index_readings,
            index_disagreements: o.index_disagreements,
            slot_collisions: o.state.ambiguous_slots().len(),
            concordant: o.verification.concordant,
            discordant: o.verification.discordant,
            bridge_named_lives: o.verification.named_by_bridge,
            direct_by_creation: o.creation.direct,
            direct_by_creation_propagated: o.creation.propagated,
            bodies_with_creation: o.creation.slots,
            death_offset_matched: o.clock.matched,
            death_offset_runner_up: o.clock.runner_up,
            death_offset_ms: (o.clock.matched > 0).then_some(o.clock.offset_ms),
            closed_by_shot: o.closures.by_shot,
            closed_by_respawn: o.closures.by_respawn,
            closed_contested: o.closures.contested,
            closed_refused: o.closures.refused,
            ..Default::default()
        }
    }
}
impl IdentityBridgeHealth {
    /// Native uncertainty warning. A zero match count is unavailable alignment,
    /// and an exact two-to-one margin is sufficient to suppress this warning.
    pub fn warn_if_death_offset_ambiguous(&self) {
        if self.death_offset_matched == 0
            || (self.death_offset_matched as i64)
                >= (self.death_offset_runner_up as i64).wrapping_mul(2)
        {
            return;
        }
        tracing::warn!(
            apparies = self.death_offset_matched,
            second_candidat = self.death_offset_runner_up,
            marge_minimale = 2,
            vies = self.lives_total,
            slots = self.slots,
            "rejeu : calage du fil des morts trop peu distinct du bruit — nommage et origine suspects"
        );
    }
    pub fn verdict(&self) -> &'static str {
        if self.slots == 0 {
            "non publiable : aucun pont"
        } else if self.slot_collisions > 0 {
            "non publiable : un slot change de porteur"
        } else if self.from_reading + self.closed_by_shot + self.closed_by_respawn != self.slots {
            "non publiable : une source non comptée a alimenté le pont"
        } else if self.index_disagreements > 0 {
            "non publiable : une identité est lue de deux façons"
        } else if self.index_readings < 2 {
            "partiel : la table d'index n'est confirmée par aucun second chunk"
        } else {
            "nominal"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_bridge_alignment_warning() {
        #[derive(Deserialize)]
        struct Row {
            bridge: IdentityBridgeHealth,
            events: Vec<serde_json::Value>,
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/bridge-warning-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Row> = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.into_iter().enumerate() {
            assert_eq!(
                crate::theater::log_test_support::capture_logs(|| row
                    .bridge
                    .warn_if_death_offset_ambiguous()),
                row.events,
                "case {i}"
            );
        }
    }
}
