//! Native carried-inventory phase, retaining raw scan results alongside failures.
use super::*;

#[derive(Debug)]
pub struct ReplayCarriedScan {
    pub weapon_changes: HeldWeaponChangeStream,
    pub weapon_error: Option<ContextWeaponScanError>,
    pub pickups: NativePickupStream,
    pub published_pickups: Vec<FactsPickup>,
    pub pickup_stats: FactsPickupStats,
    pub inventory: KeyframeInventoryStream,
    pub inventory_error: Option<SourceInventoryScanError>,
    pub deltas: InventoryDeltaStream,
    pub delta_error: Option<ContextInventoryScanError>,
}
impl ReplayCarriedScan {
    pub fn published_weapons(&self) -> &[HeldWeaponChange] {
        if self.weapon_error.is_some() {
            &[]
        } else {
            &self.weapon_changes.records
        }
    }
    pub fn published_inventory(&self) -> &[KeyframeInventory] {
        if self.inventory_error.is_some() {
            &[]
        } else {
            &self.inventory.records
        }
    }
    pub fn published_deltas(&self) -> &[InventoryDeltaRead] {
        if self.delta_error.is_some() {
            &[]
        } else {
            &self.deltas.records
        }
    }
    /// Native facts projection for this phase only. Capture still follows all
    /// scan phases. Error resets affect publication, preserving raw scan evidence.
    pub fn apply_to_facts(&self, facts: &mut NativeFilmFacts) {
        facts.weapon_changes = self
            .published_weapons()
            .iter()
            .map(FactsWeaponChange::from)
            .collect();
        facts.pickups = self.published_pickups.clone();
        facts.pickup_stats = self.pickup_stats.clone();
        facts.inventory.inventory = self
            .published_inventory()
            .iter()
            .map(FactsKeyframeInventory::from)
            .collect();
        facts.inventory.deltas = self
            .published_deltas()
            .iter()
            .map(FactsInventoryDelta::from)
            .collect();
        facts.header.inventory_delta_ammo_refused = self.deltas.stats.ammo_refused;
    }
}
pub enum ReplayCarriedObservation<'a> {
    HeldWeapons(&'a [HeldWeaponChange]),
    HeldWeaponStats(&'a HeldWeaponChangeStats),
    Pickups(&'a [FactsPickup]),
    PickupStats(&'a FactsPickupStats),
    Inventory(&'a [KeyframeInventory]),
    InventoryStats(&'a KeyframeInventoryStats),
    Deltas(&'a [InventoryDeltaRead]),
    DeltaStats(&'a InventoryDeltaStats),
}
impl ReplayCarriedObservation<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            Self::HeldWeapons(_) => "heldWeaponChanges",
            Self::HeldWeaponStats(_) => "heldWeaponChanges.stats",
            Self::Pickups(_) => "pickups",
            Self::PickupStats(_) => "pickups.stats",
            Self::Inventory(_) => "inventory",
            Self::InventoryStats(_) => "inventory.stats",
            Self::Deltas(_) => "inventoryDeltas",
            Self::DeltaStats(_) => "inventoryDeltas.stats",
        }
    }
}
/// Native balayerPortage/balayerInventaire. All scanner failures are nonfatal,
/// with native publication resets, logs and eight borrowed observation boundaries.
/// Initial loadouts come from the preceding position phase. The shared fallback
/// counter is incremented at the keyframe inventory boundary, even on refusal.
pub fn scan_replay_carried_inputs(
    match_id: &[u8],
    context: &NativeFilmContext<'_>,
    loadouts: &[KeyframeLoadout],
    fallbacks: Option<&FallbackCounter>,
    mut observe: impl FnMut(ReplayCarriedObservation<'_>),
) -> ReplayCarriedScan {
    let match_id = ReplayByteString(match_id.to_vec()).json_text();
    let (weapon_changes, weapon_error) = scan_context_held_weapon_changes(context, loadouts);
    if let Some(error) = &weapon_error {
        let message = match error {
            ContextWeaponScanError::Registry(NativeContextRegistryError::NoRegistryChunk) => {
                "chunk_00 (registre) absent du film".to_owned()
            }
            _ => error.to_string(),
        };
        tracing::warn!(
            err = message.as_str(),
            match_id = match_id.as_str(),
            "changements d arme illisibles — rejeu sans ramassages"
        );
    } else {
        let s = &weapon_changes.stats;
        tracing::info!(
            recordsDelta = s.records,
            masquePorteur = s.with_component,
            emissions = s.emissions,
            repetitions = s.repeats,
            "ramassage : changements d arme lus"
        );
    }
    observe(ReplayCarriedObservation::HeldWeapons(
        if weapon_error.is_some() {
            &[]
        } else {
            &weapon_changes.records
        },
    ));
    observe(ReplayCarriedObservation::HeldWeaponStats(
        &weapon_changes.stats,
    ));
    let pickups = scan_context_biped_pickups(context);
    let (published_pickups, pickup_stats) = if pickups.no_film_chunks {
        tracing::warn!(
            err = "aucun chunk de donnees dans le film",
            match_id = match_id.as_str(),
            "ramassages natifs illisibles — rejeu sans ramassages natifs"
        );
        (vec![], FactsPickupStats::default())
    } else {
        let s = &pickups.stats;
        tracing::info!(
            paquets = s.packets,
            type9 = s.type_9,
            type8 = s.type_8,
            publies = s.published,
            listesMultiples = s.multi_event,
            refusesSansRef = s.refused_no_ref,
            refusesSansIdentifiant = s.refused_no_catalog,
            refusesHorsBande = s.refused_off_band,
            refLargeInattendue = s.unexpected_wide_ref,
            "ramassage natif : evenements lus"
        );
        facts_from_pickup_scan(&pickups)
    };
    observe(ReplayCarriedObservation::Pickups(&published_pickups));
    observe(ReplayCarriedObservation::PickupStats(&pickup_stats));
    let families = v41_weapon_families().keys().map(|&k| (k, true)).collect();
    let (inventory, inventory_error) =
        scan_source_keyframe_inventory(context.source(), &families, 0, fallbacks);
    if let Some(error) = &inventory_error {
        tracing::warn!(
            err = error.to_string().as_str(),
            match_id = match_id.as_str(),
            "inventaire illisible — rejeu sans grenades ni munitions"
        );
    } else {
        let s = &inventory.stats;
        tracing::info!(
            chunks = s.chunks,
            chunksIllisibles = s.chunks_unread,
            imagesCles = s.keyframes,
            records = s.records,
            grenadesParAncre = s.grenades_by_anchor,
            grenadesParPosition = s.grenades_by_position,
            "inventaire : lectures de keyframe"
        );
    }
    observe(ReplayCarriedObservation::Inventory(
        if inventory_error.is_some() {
            &[]
        } else {
            &inventory.records
        },
    ));
    observe(ReplayCarriedObservation::InventoryStats(&inventory.stats));
    let (deltas, delta_error) = scan_context_inventory_deltas(context);
    if let Some(error) = &delta_error {
        let message = match error {
            ContextInventoryScanError::Registry(NativeContextRegistryError::NoRegistryChunk) => {
                "chunk_00 (registre) absent du film".to_owned()
            }
            _ => error.to_string(),
        };
        tracing::warn!(
            err = message.as_str(),
            match_id = match_id.as_str(),
            "inventaire delta illisible — grenades sans rafraichissement entre images-cles"
        );
    } else {
        let s = &deltas.stats;
        tracing::info!(
            recordsDelta = s.records,
            masqueAvecI22 = s.with_i22,
            i22Lues = s.i22_read,
            i22Implausibles = s.implausible,
            masqueAvecI47 = s.with_i47,
            i47Lues = s.i47_read,
            accord = s.accord,
            accordVerifies = s.accord_checked,
            canalMunitionsRefuse = s.ammo_refused,
            "inventaire delta : lectures"
        );
    }
    observe(ReplayCarriedObservation::Deltas(if delta_error.is_some() {
        &[]
    } else {
        &deltas.records
    }));
    observe(ReplayCarriedObservation::DeltaStats(&deltas.stats));
    ReplayCarriedScan {
        weapon_changes,
        weapon_error,
        pickups,
        published_pickups,
        pickup_stats,
        inventory,
        inventory_error,
        deltas,
        delta_error,
    }
}
