//! Native vehicle replay pass over a single loaded context.
use super::*;
#[derive(Debug, thiserror::Error)]
pub enum ReplayVehicleScanError {
    #[error("no vehicle slots in keyframes")]
    NoBand,
    #[error(transparent)]
    Creations(ContextEquipmentCreationError),
    #[error(transparent)]
    Positions(ContextPositionScanError),
    #[error(transparent)]
    Events(ContextVehicleChannelError),
    #[error(transparent)]
    Aims(ContextVehicleChannelError),
    #[error(transparent)]
    March(NativeMarchError),
}
#[derive(Debug)]
pub struct ReplayVehicleInputScan {
    pub format: FilmMppResolution,
    pub installed_widths: Option<FilmMppWidths>,
    pub census: SourceWorldObjectCensus,
    pub creations: Option<ContextEquipmentCreationScan>,
    pub positions: Option<SourceWorldPositionReport>,
    pub events: Vec<SourceVehicleEvent>,
    pub aims: Vec<SourceBipedAim>,
    pub march: Option<FilmMarchFacts>,
    pub errors: Vec<ReplayVehicleScanError>,
    pub published: FactsVehicleScan,
}
struct Restore<'a, 's> {
    context: &'a mut NativeFilmContext<'s>,
    previous: Option<FilmMppWidths>,
}
impl Drop for Restore<'_, '_> {
    fn drop(&mut self) {
        if let Some(w) = self.previous {
            self.context.set_mpp(w);
        }
    }
}
/// Native format fallback selection precedes census admission. The outer owner
/// must publish an unknown-format counter event when `format.unknown_format`.
/// Failures in creation/positions clear publication; auxiliary failures do not.
pub fn scan_replay_vehicle_inputs(
    match_id: &[u8],
    context: &mut NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    calibrated: FilmMppWidths,
    mut observe: impl FnMut(&ContextEquipmentCreationAttempt),
) -> ReplayVehicleInputScan {
    scan_replay_vehicle_inputs_observed(match_id, context, map, calibrated, |event| {
        if let ReplayWorldObjectScanObservation::Creation { attempt, .. } = event {
            observe(attempt);
        }
    })
}
/// Includes the native unknown-format counter boundary before census/width changes.
pub fn scan_replay_vehicle_inputs_observed(
    match_id: &[u8],
    context: &mut NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    calibrated: FilmMppWidths,
    mut observe: impl FnMut(ReplayWorldObjectScanObservation<'_>),
) -> ReplayVehicleInputScan {
    let match_id = ReplayByteString(match_id.to_vec()).json_text();
    let match_id = match_id.as_str();
    let format = context
        .source()
        .and_then(FilmSource::registry_chunk)
        .and_then(|b| b.get(4..8))
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .unwrap_or(0);
    let format = resolve_film_mpp(format);
    if format.unknown_format {
        observe(ReplayWorldObjectScanObservation::UnknownFormat {
            archetype: 40,
            format: format.format_version,
        });
    }
    let selected = format
        .widths
        .map(|[lead, index]| FilmMppWidths {
            lead: lead as i64,
            index: index as i64,
        })
        .unwrap_or(calibrated);
    let installed_widths = selected.is_valid().then_some(selected);
    let previous = installed_widths.map(|w| context.set_mpp(w));
    let restore = Restore { context, previous };
    let context = &*restore.context;
    let census = scan_source_world_object_keyframes(context.source(), &[40]);
    let kf = &census.archetypes[&40];
    let band = kf.band.clone();
    let mut out = ReplayVehicleInputScan {
        format,
        installed_widths,
        census,
        creations: None,
        positions: None,
        events: Vec::new(),
        aims: Vec::new(),
        march: None,
        errors: Vec::new(),
        published: FactsVehicleScan::default(),
    };
    if band.is_empty() {
        tracing::info!(
            match_id,
            imagesCles = out.census.archetypes[&40].times_us.len(),
            "vehicules : aucun slot ti=40 aux images-cles — rejeu sans ce calque"
        );
        out.errors.push(ReplayVehicleScanError::NoBand);
        return out;
    }
    let (creations, error) =
        scan_context_vehicle_creations_for_band(context, map, &band, |attempt| {
            observe(ReplayWorldObjectScanObservation::Creation {
                archetype: 40,
                attempt,
            })
        });
    out.creations = Some(creations);
    if let Some(error) = error {
        tracing::warn!(err=%error,match_id,"vehicules : records de creation illisibles — rejeu sans ce calque");
        out.errors.push(ReplayVehicleScanError::Creations(error));
        return out;
    }
    let map = map.expect("creation admission checked map");
    let options = SourceWorldScanOptions {
        scan: BipedScanOptions {
            require_generation_one: false,
            capture_dirs: true,
            ..BipedScanOptions::native_defaults()
        },
        dynamic_forward_level: Some(2),
        teleport_exemptions: Default::default(),
    };
    let positions = match scan_context_world_positions(
        context,
        &[],
        Some(&band),
        context.imposed_layout().as_ref(),
        [map.min, map.max],
        &options,
    ) {
        Ok(p) => p,
        Err(error) => {
            tracing::warn!(err=%error,match_id,"vehicules : nuage de positions illisible — AUCUN vehicule publie (sans lui, une vie recensee n aurait ni trajectoire ni cap)");
            out.errors.push(ReplayVehicleScanError::Positions(error));
            return out;
        }
    };
    out.published.positions = facts_from_world_position_scan(&positions, true);
    out.positions = Some(positions);
    let (events, error) = scan_context_vehicle_events(context);
    out.events = events;
    if let Some(error) = error {
        tracing::warn!(err=%error,match_id,"vehicules : liste d evenements illisible — episodes d occupation bornes par le seul trou de position");
        out.errors.push(ReplayVehicleScanError::Events(error));
    } else {
        out.published.events = out
            .events
            .iter()
            .map(|e| FactsVehicleEvent::from(&e.event))
            .collect();
    }
    let (aims, error) = scan_context_biped_aim(context);
    out.aims = aims;
    if let Some(error) = error {
        tracing::warn!(err=%error,match_id,"vehicules : visees sans position illisibles — episodes d occupation sans serie de visee, le cone retombe sur le cap du chassis");
        out.errors.push(ReplayVehicleScanError::Aims(error));
    } else {
        out.published.aims = out
            .aims
            .iter()
            .map(|a| FactsVehicleAim::from(&a.record.aim))
            .collect();
    }
    match context.scan_march_facts() {
        Ok(m) => {
            super::film_vehicles::log_vehicle_death_reads(match_id, &m);
            out.published.occupancy = m.facts.occupancy.clone();
            out.published.deaths = facts_from_march_scan(Some(&m), Some(40));
            out.march = Some(m);
        }
        Err(error) => {
            tracing::warn!(err=%error,match_id,"vehicules : morts ecrites illisibles — fins de vie bornees par le seul recensement");
            out.published.deaths = facts_from_march_scan(None, Some(40));
            out.errors.push(ReplayVehicleScanError::March(error));
        }
    }
    let c = out.creations.as_ref().unwrap();
    let st = &c.stats;
    let kf = &out.census.archetypes[&40];
    tracing::info!(
        slots = st.slots,
        ancres = st.anchors,
        creationsAcceptees = st.accepted,
        imagesCles = kf.times_us.len(),
        viesRecensees = kf.seen_us.len(),
        echantillons = out.published.positions.len(),
        evenements = out.published.events.len(),
        viseesSansPosition = out.published.aims.len(),
        mortsEcrites = out.published.deaths.deaths.as_ref().map_or(0, Vec::len),
        lecturesDOccupation = out.published.occupancy.len(),
        "vehicules : balayage ti=40"
    );
    out.published.scanned = true;
    out.published.keyframes = FactsWorldKeyframes::from(kf);
    out.published.creations = c
        .records
        .iter()
        .map(|r| FactsEquipmentCreation::from(r.read.creation.as_ref().unwrap()))
        .collect();
    out.published.stats = FactsCreationStats::from(st);
    out
}
impl From<&VehicleEvent> for FactsVehicleEvent {
    fn from(v: &VehicleEvent) -> Self {
        Self {
            timestamp_us: v.timestamp_us,
            kind: i64::from(v.kind),
            occupant_present: v.occupant_present,
            occupant_sonde: i64::from(v.occupant_sonde),
            occupant_slot: v.occupant_slot,
            occupant_in_band: v.occupant_in_band,
            vehicle_slot: v.vehicle_slot,
            vehicle_slot_valid: v.vehicle_slot_valid,
            vehicle_gen: v.vehicle_gen,
            seat: v.seat,
            seat_valid: v.seat_valid,
        }
    }
}
impl From<&BipedAim> for FactsVehicleAim {
    fn from(v: &BipedAim) -> Self {
        Self {
            timestamp_us: v.timestamp_us,
            slot: v.slot,
            yaw_raw: v.yaw_raw,
            pitch_raw: v.pitch_raw,
        }
    }
}
