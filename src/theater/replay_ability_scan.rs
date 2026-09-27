//! Native abilities phase. Raw scans survive nonfatal publication resets.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ReplayAbilityScan {
    pub ranks: ContextAbilityChannelScan,
    pub rank_error: Option<ContextAbilityChannelError>,
    pub equipment: ContextEquipmentScan,
    pub equipment_error: Option<ContextAbilityChannelError>,
    pub equipment_stats: FactsEquipmentChangeStats,
    pub camo: ContextAbilityChannelScan,
    pub camo_error: Option<ContextAbilityChannelError>,
    pub grapple: ContextAbilityChannelScan,
    pub grapple_error: Option<ContextAbilityChannelError>,
    pub impulses: ContextImpulseScan,
    pub impulse_error: Option<ContextAbilityChannelError>,
    pub impulse_stats: FactsAbilityImpulseStats,
    pub charges: ContextChargeScan,
    pub charge_error: Option<ContextAbilityChannelError>,
    pub charge_stats: FactsAbilityChargeStats,
}
impl ReplayAbilityScan {
    pub fn published_ranks(
        &self,
    ) -> impl Iterator<Item = (&ContextChannelObservation, FactsAbilityRank)> {
        self.ranks
            .ability_ranks()
            .filter(|_| self.rank_error.is_none())
    }
    pub fn published_equipment(
        &self,
    ) -> impl Iterator<Item = (&EquipmentChange, FactsEquipmentChange)> {
        self.equipment
            .facts()
            .filter(|_| self.equipment_error.is_none())
    }
    pub fn published_camo(
        &self,
    ) -> impl Iterator<Item = (&ContextChannelObservation, FactsCamoState)> {
        self.camo
            .camo_states()
            .filter(|_| self.camo_error.is_none())
    }
    pub fn published_grapple(
        &self,
    ) -> impl Iterator<Item = (&ContextChannelObservation, FactsGrappleRead)> {
        self.grapple
            .grapple_reads()
            .filter(|_| self.grapple_error.is_none())
    }
    pub fn published_impulses(
        &self,
    ) -> impl Iterator<Item = (&ContextChannelObservation, FactsAbilityImpulse)> {
        self.impulses
            .facts()
            .filter(|_| self.impulse_error.is_none())
    }
    pub fn published_charges(
        &self,
    ) -> impl Iterator<Item = (&ContextChannelObservation, FactsAbilityCharge)> {
        self.charges.facts().filter(|_| self.charge_error.is_none())
    }
    /// Replace only this phase's fields. Source evidence remains in raw scans;
    /// native facts capture still follows all six phases.
    pub fn apply_to_facts(&self, facts: &mut NativeFilmFacts) {
        facts.delta_channels.ability_ranks = self.published_ranks().map(|(_, v)| v).collect();
        facts.equipment_changes = self.published_equipment().map(|(_, v)| v).collect();
        facts.equipment_change_stats = self.equipment_stats.clone();
        facts.delta_channels.camo_states = self.published_camo().map(|(_, v)| v).collect();
        facts.delta_channels.grapple_reads = self.published_grapple().map(|(_, v)| v).collect();
        facts.abilities.impulses = self.published_impulses().map(|(_, v)| v).collect();
        facts.abilities.impulse_stats = self.impulse_stats.clone();
        facts.abilities.charges = self.published_charges().map(|(_, v)| v).collect();
        facts.abilities.charge_stats = self.charge_stats.clone();
    }
}
/// Borrowed native publication boundaries. Value/source pairs preserve access
/// to full scan evidence even where the native cache DTO omits source fields.
pub enum ReplayAbilityObservation<'a> {
    AbilityRanks(&'a [(&'a ContextChannelObservation, FactsAbilityRank)]),
    AbilityRankStats(&'a BipedChannelStats),
    EquipmentChanges(&'a [(&'a EquipmentChange, FactsEquipmentChange)]),
    EquipmentStats(&'a FactsEquipmentChangeStats),
    CamoStates(&'a [(&'a ContextChannelObservation, FactsCamoState)]),
    CamoStats(&'a BipedChannelStats),
    GrappleReads(&'a [(&'a ContextChannelObservation, FactsGrappleRead)]),
    GrappleStats(&'a GrappleStats),
    Impulses(&'a [(&'a ContextChannelObservation, FactsAbilityImpulse)]),
    Charges(&'a [(&'a ContextChannelObservation, FactsAbilityCharge)]),
}
impl ReplayAbilityObservation<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AbilityRanks(_) => "abilityRanks",
            Self::AbilityRankStats(_) => "abilityRanks.stats",
            Self::EquipmentChanges(_) => "equipmentChanges",
            Self::EquipmentStats(_) => "equipmentChanges.stats",
            Self::CamoStates(_) => "camoStates",
            Self::CamoStats(_) => "camoStates.stats",
            Self::GrappleReads(_) => "grappleReads",
            Self::GrappleStats(_) => "grappleReads.stats",
            Self::Impulses(_) => "abilityImpulses",
            Self::Charges(_) => "abilityCharges",
        }
    }
}
fn error_text(e: &ContextAbilityChannelError) -> String {
    match e {
        ContextAbilityChannelError::Registry(NativeContextRegistryError::NoRegistryChunk) => {
            "chunk_00 (registre) absent du film".into()
        }
        _ => e.to_string(),
    }
}
/// Native balayerCapacites, including birthOfLives over the preceding raw
/// position results. Every error clears published values. Equipment, impulse
/// and charge errors also reset published statistics; other statistics survive.
pub fn scan_replay_ability_inputs(
    match_id: &[u8],
    context: &NativeFilmContext<'_>,
    positions: &[FactsBipedPosition],
    mut observe: impl FnMut(ReplayAbilityObservation<'_>),
) -> ReplayAbilityScan {
    let match_id = ReplayByteString(match_id.to_vec()).json_text();
    let (ranks, rank_error) = scan_context_ability_emissions(context);
    if let Some(e) = &rank_error {
        tracing::warn!(
            err = error_text(e).as_str(),
            match_id = match_id.as_str(),
            "identites de capacite illisibles — rejeu sans rang complet"
        );
    } else {
        let s = &ranks.stats;
        tracing::info!(
            recordsDelta = s.records,
            masqueAvecI48 = s.with_component,
            lues = s.read,
            illisibles = s.unread,
            sansIdentite = s.gated,
            "capacites : lectures d i48"
        );
    }
    observe(ReplayAbilityObservation::AbilityRanks(
        &ranks
            .ability_ranks()
            .filter(|_| rank_error.is_none())
            .collect::<Vec<_>>(),
    ));
    observe(ReplayAbilityObservation::AbilityRankStats(&ranks.stats));
    let mut births = BTreeMap::new();
    for p in positions {
        births
            .entry(p.slot)
            .and_modify(|v: &mut u64| *v = (*v).min(p.timestamp_us))
            .or_insert(p.timestamp_us);
    }
    let (equipment, equipment_error) = scan_context_equipment_changes(context, &births);
    let equipment_stats = if let Some(e) = &equipment_error {
        tracing::warn!(
            err = error_text(e).as_str(),
            match_id = match_id.as_str(),
            "changements d equipement illisibles — rejeu sans ramassages d equipement"
        );
        FactsEquipmentChangeStats::default()
    } else {
        let s = equipment.facts_stats();
        tracing::info!(
            emissions = s.walk.read,
            vies = s.lives,
            ramassages = s.taken,
            consommations = s.spent,
            reapparitions = s.spawned,
            manqueesEstimees = s.missed_estimate,
            "equipement : changements lus"
        );
        s
    };
    observe(ReplayAbilityObservation::EquipmentChanges(
        &equipment
            .facts()
            .filter(|_| equipment_error.is_none())
            .collect::<Vec<_>>(),
    ));
    observe(ReplayAbilityObservation::EquipmentStats(&equipment_stats));
    let (camo, camo_error) = scan_context_camo_states(context);
    if let Some(e) = &camo_error {
        tracing::warn!(
            err = error_text(e).as_str(),
            match_id = match_id.as_str(),
            "etat de camouflage illisible — rejeu sans episodes de camo"
        );
    } else {
        let s = &camo.stats;
        tracing::info!(
            recordsDelta = s.records,
            masqueAvecI28 = s.with_component,
            lues = s.read,
            illisibles = s.unread,
            sansVoie = s.gated,
            "camouflage : lectures d i28 queue[1]"
        );
    }
    observe(ReplayAbilityObservation::CamoStates(
        &camo
            .camo_states()
            .filter(|_| camo_error.is_none())
            .collect::<Vec<_>>(),
    ));
    observe(ReplayAbilityObservation::CamoStats(&camo.stats));
    let (grapple, grapple_error) = scan_context_grapple_reads(context);
    let g = grapple.grapple_stats();
    if let Some(e) = &grapple_error {
        tracing::warn!(
            err = error_text(e).as_str(),
            match_id = match_id.as_str(),
            "evenements de grappin illisibles — rejeu sans tractions"
        );
    } else {
        tracing::info!(
            recordsDelta = g.records,
            masqueAvecI59 = g.with_component,
            lues = g.read,
            illisibles = g.unread,
            tag3 = g.tag_three,
            corpsCasses = g.body_broken,
            "grappin : lectures d i59 tag==3"
        );
    }
    observe(ReplayAbilityObservation::GrappleReads(
        &grapple
            .grapple_reads()
            .filter(|_| grapple_error.is_none())
            .collect::<Vec<_>>(),
    ));
    observe(ReplayAbilityObservation::GrappleStats(&g));
    let (impulses, impulse_error) = scan_context_ability_impulses(context);
    let impulse_stats = if let Some(e) = &impulse_error {
        tracing::warn!(
            err = error_text(e).as_str(),
            match_id = match_id.as_str(),
            "impulsions de capacite illisibles — rejeu sans impulsions"
        );
        FactsAbilityImpulseStats::default()
    } else {
        let s = &impulses.stats;
        tracing::info!(
            recordsDelta = s.records,
            masqueAvecI57 = s.with_predicted,
            masqueAvecI59 = s.with_non_predicted,
            lues = s.read,
            illisibles = s.unread,
            tag1 = s.tag_one,
            composantAbsent = s.absent,
            "capacites : lectures de tag d i57/i59"
        );
        FactsAbilityImpulseStats::from(s)
    };
    observe(ReplayAbilityObservation::Impulses(
        &impulses
            .facts()
            .filter(|_| impulse_error.is_none())
            .collect::<Vec<_>>(),
    ));
    let (charges, charge_error) = scan_context_ability_charges(context);
    let charge_stats = if let Some(e) = &charge_error {
        tracing::warn!(
            err = error_text(e).as_str(),
            match_id = match_id.as_str(),
            "charges d equipement illisibles — rejeu sans releve de charges"
        );
        FactsAbilityChargeStats::default()
    } else {
        let s = &charges.stats;
        tracing::info!(
            recordsDelta = s.records,
            masqueAvecI56 = s.with_component,
            lues = s.read,
            illisibles = s.unread,
            emplacementsArmes = s.armed,
            composantAbsent = s.absent,
            "capacites : lectures d i56"
        );
        FactsAbilityChargeStats::from(s)
    };
    observe(ReplayAbilityObservation::Charges(
        &charges
            .facts()
            .filter(|_| charge_error.is_none())
            .collect::<Vec<_>>(),
    ));
    ReplayAbilityScan {
        ranks,
        rank_error,
        equipment,
        equipment_error,
        equipment_stats,
        camo,
        camo_error,
        grapple,
        grapple_error,
        impulses,
        impulse_error,
        impulse_stats,
        charges,
        charge_error,
        charge_stats,
    }
}
