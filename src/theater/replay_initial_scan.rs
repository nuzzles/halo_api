//! First native scan phase; later inventory/world phases and assembly are separate.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayInitialScan {
    pub major_version: Option<u32>,
    pub translocations: Vec<FilmNativeTranslocatorEvent>,
    pub positions: SourceWorldPositionReport,
    pub creations: BipedCreationStream,
    pub fire: Vec<SourceFireEvent>,
    pub loadouts: Vec<KeyframeLoadout>,
}
impl ReplayInitialScan {
    /// Replace only fields produced by this phase. This is not a complete scan
    /// or a captured facts file; capture follows all remaining scan phases.
    pub fn apply_to_facts(&self, facts: &mut NativeFilmFacts) {
        facts.header.film_major_version = self.major_version.map(i64::from);
        facts.delta_channels.translocations = self
            .translocations
            .iter()
            .map(FactsTranslocation::from)
            .collect();
        facts.positions = facts_from_world_position_scan(&self.positions, true);
        facts.biped_creations = self
            .creations
            .records
            .iter()
            .map(FactsBipedCreation::from)
            .collect();
        facts.events.fire = self
            .fire
            .iter()
            .map(|e| FactsFireEvent::from(&e.event))
            .collect();
        facts.events.loadouts = self
            .loadouts
            .iter()
            .map(|l| FactsLoadout {
                timestamp_us: l.timestamp_us,
                slot: l.slot,
                families: Some(l.families.clone()),
            })
            .collect();
    }
}

pub enum ReplayInitialObservation<'a> {
    Translocations(&'a [FilmNativeTranslocatorEvent]),
    Positions(&'a SourceWorldPositionReport),
    BipedCreations(&'a BipedCreationStream),
    Fire(&'a [SourceFireEvent]),
    Loadouts(&'a [KeyframeLoadout]),
}
impl ReplayInitialObservation<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Translocations(_) => "translocations",
            Self::Positions(_) => "positions",
            Self::BipedCreations(_) => "bipedCreations",
            Self::Fire(_) => "fire",
            Self::Loadouts(_) => "loadouts",
        }
    }
}
#[derive(Default)]
pub struct ReplayInitialScanOptions<'a> {
    pub chunks: &'a [i64],
    /// None uses native scalar defaults. Capture and teleport exemptions are
    /// replaced by this phase's authoritative values; bounds come from the map.
    pub scan: Option<&'a SourceWorldScanOptions>,
}

/// Native balayerPositions, including entry scan-option preparation. The caller
/// must already have installed its calibrated profile then map precision on the
/// shared context. This phase forces direction capture, installs actual recorded
/// teleport exemptions, and passes ImposedLayout explicitly to position scanning.
/// A position refusal stops before creation/fire/loadout observations; later
/// scanner failures log and clear only their own outputs. Observation values are
/// borrowed at the native boundaries. Wall-clock debug timing is not collected.
/// This is not the complete scanFilmInputs or BuildFromFilmAvecFaits entry.
pub fn scan_replay_initial_inputs(
    match_id: &[u8],
    context: &NativeFilmContext<'_>,
    map: &FilmMapBounds,
    options: ReplayInitialScanOptions<'_>,
    mut observe: impl FnMut(ReplayInitialObservation<'_>),
) -> Result<ReplayInitialScan, ContextPositionScanError> {
    let match_id = ReplayByteString(match_id.to_vec()).json_text();
    let major_version = context.profile()?.major_version;
    if major_version.is_none() {
        tracing::warn!(
            match_id = match_id.as_str(),
            "rejeu : version de film illisible — le fil des morts retombe sur le decoupage historique du gamertag"
        );
    }
    let translocations = scan_source_translocator_events(context.source(), Some(map));
    let mut scan = options
        .scan
        .cloned()
        .unwrap_or_else(|| SourceWorldScanOptions {
            scan: BipedScanOptions::native_defaults(),
            dynamic_forward_level: None,
            teleport_exemptions: BTreeMap::new(),
        });
    scan.scan.capture_dirs = true;
    scan.teleport_exemptions.clear();
    for t in &translocations {
        scan.teleport_exemptions
            .entry(t.read.event.slot)
            .or_default()
            .push(t.source.timestamp_us);
    }
    for times in scan.teleport_exemptions.values_mut() {
        times.sort_unstable();
    }
    if !translocations.is_empty() {
        tracing::info!(
            evenements = translocations.len(),
            "translocateur : teleportations lues"
        );
    }
    observe(ReplayInitialObservation::Translocations(&translocations));
    let positions = scan_context_world_positions(
        context,
        options.chunks,
        None,
        context.imposed_layout().as_ref(),
        [map.min, map.max],
        &scan,
    )?;
    observe(ReplayInitialObservation::Positions(&positions));
    let creations = match scan_context_biped_creations(Some(context), None) {
        Ok(creations) => {
            let s = &creations.stats;
            let (word, count) = s.most_common_other().unwrap_or_default();
            tracing::info!(
                corps = s.slots,
                ancres = s.anchors,
                acceptes = s.accepted,
                formeRefusee = s.shape_bad,
                signatureEtrangere = s.signature_mismatch,
                motAlternatif = word,
                motAlternatifCompte = count,
                porteFermee = s.gate_closed,
                tronques = s.truncated,
                "creation de bipede : records lus"
            );
            if s.anchors > 0 && s.accepted == 0 {
                tracing::warn!(
                    match_id = match_id.as_str(),
                    ancres = s.anchors,
                    motAlternatifModal = word,
                    compte = count,
                    "creation de bipede : aucune signature reconnue sur des ancres presentes — ce film porte-t-il une autre representation ?"
                );
            }
            creations
        }
        Err(error) => {
            tracing::warn!(
                err = error.to_string().as_str(),
                match_id = match_id.as_str(),
                "creations de bipede illisibles — le registre degrade sur le pont par morts"
            );
            BipedCreationStream::default()
        }
    };
    observe(ReplayInitialObservation::BipedCreations(&creations));
    let fire = scan_source_fire_events(context.source()).unwrap_or_else(|_| {
        tracing::warn!(
            err = "aucun chunk film lisible",
            match_id = match_id.as_str(),
            "events de tir illisibles — rejeu sans tirs"
        );
        Vec::new()
    });
    observe(ReplayInitialObservation::Fire(&fire));
    let families = v41_weapon_families().keys().map(|&k| (k, true)).collect();
    let loadouts = context
        .source()
        .map_or(Err(KeyframeLoadoutScanError::NoReadableChunk), |source| {
            scan_source_keyframe_loadouts(source, &families)
        })
        .unwrap_or_else(|_| {
            tracing::warn!(
                err = "aucun chunk film lisible",
                match_id = match_id.as_str(),
                "keyframes illisibles — rejeu sans armes portées"
            );
            Vec::new()
        });
    observe(ReplayInitialObservation::Loadouts(&loadouts));
    Ok(ReplayInitialScan {
        major_version,
        translocations,
        positions,
        creations,
        fire,
        loadouts,
    })
}
