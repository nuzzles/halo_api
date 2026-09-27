//! Ordered loaded world phase; facts capture belongs after all six phases.
use super::*;
#[derive(Debug)]
pub struct ReplayWorldScan {
    pub zoom: Vec<SourceZoomEvent>,
    pub placements: ContextEquipmentPlacements,
    pub placement_error: Option<ContextPlacementError>,
    pub spawns: SourceSpawnScan,
    pub spawn_error: Option<SourceSpawnScanError>,
    pub pads: ReplayPadScans,
    pub vehicles: ReplayVehicleInputScan,
    pub guarded: ReplayGuardedScan,
    pub diagnostics: Vec<StatborgDiagnostic>,
    pub published_zoom: Vec<FactsZoomEvent>,
    pub published: FactsWorldSection,
}
impl ReplayWorldScan {
    /// Project this phase only; mode guards remain in the facts file's guard section.
    pub fn apply_to_facts(&self, facts: &mut NativeFilmFacts) {
        facts.zoom_events = self.published_zoom.clone();
        facts.world = self.published.clone();
        facts.vehicles = self.vehicles.published.clone();
    }
}
pub enum ReplayWorldObservation<'a> {
    ZoomEvents(Option<&'a [FactsZoomEvent]>),
    Placements(Option<&'a [FactsPlacement]>),
    PlacementStats(&'a FactsPlacementStats),
    SpawnEvents(Option<&'a [FactsSpawnEvent]>),
    Pads(&'a ReplayPadScans),
    Vehicles(&'a FactsVehicleScan),
    Guarded(ReplayGuardedObservation<'a>),
    PlacementAttempt(ContextPlacementObservation<'a>),
    Creation {
        archetype: u32,
        attempt: &'a ContextEquipmentCreationAttempt,
    },
    Diagnostic(&'a StatborgDiagnostic),
    Counter {
        name: &'a str,
        amount: usize,
    },
}
impl ReplayWorldObservation<'_> {
    pub fn channel_name(&self) -> Option<&'static str> {
        match self {
            Self::ZoomEvents(_) => Some("zoomEvents"),
            Self::Placements(_) => Some("placements"),
            Self::PlacementStats(_) => Some("placements.stats"),
            Self::SpawnEvents(_) => Some("spawnEvents"),
            Self::Pads(_) => Some("pads"),
            Self::Vehicles(_) => Some("vehicles"),
            Self::Guarded(e) => e.channel_name(),
            _ => None,
        }
    }
}
fn unknown_format(format: u32, observe: &mut impl FnMut(ReplayWorldObservation<'_>)) {
    let name = format!("filmdec_unknown_format_{format}");
    observe(ReplayWorldObservation::Counter {
        name: &name,
        amount: 1,
    });
}
fn object_observation(
    event: ReplayWorldObjectScanObservation<'_>,
    observe: &mut impl FnMut(ReplayWorldObservation<'_>),
) {
    match event {
        ReplayWorldObjectScanObservation::UnknownFormat { format, .. } => {
            unknown_format(format, observe)
        }
        ReplayWorldObjectScanObservation::Creation { archetype, attempt } => {
            observe(ReplayWorldObservation::Creation { archetype, attempt })
        }
    }
}
macro_rules! log {
    ($diagnostics:expr, $observe:expr, $level:literal, $message:literal $(, $key:literal => $value:expr)* $(,)?) => {{
        let diagnostic = StatborgDiagnostic { level: $level.into(), message: $message.into(),
            attributes: vec![$(($key.into(), serde_json::json!($value))),*] };
        $observe(ReplayWorldObservation::Diagnostic(&diagnostic));
        $diagnostics.push(diagnostic);
    }};
}
/// Native balayerMonde. Diagnostics/counters use the embedding observer, while
/// existing pad/vehicle tracing stays at its native scan boundary. No final
/// facts capture or interpretation of objective modes occurs here.
pub fn scan_replay_world_inputs(
    match_id: &[u8],
    context: &mut NativeFilmContext<'_>,
    map: &FilmMapBounds,
    guarded_options: ReplayGuardedScanOptions<'_>,
    mut observe: impl FnMut(ReplayWorldObservation<'_>),
) -> ReplayWorldScan {
    let id = ReplayByteString(match_id.to_vec()).json_text();
    let mut diagnostics = Vec::new();
    let zoom = scan_source_zoom_events(context.source());
    let published_zoom: Vec<_> = zoom.iter().map(FactsZoomEvent::from).collect();
    observe(ReplayWorldObservation::ZoomEvents(
        (!published_zoom.is_empty()).then_some(published_zoom.as_slice()),
    ));
    let (placements, placement_error) =
        scan_context_equipment_placements(context, Some(map), |a| {
            observe(ReplayWorldObservation::PlacementAttempt(a))
        });
    let stats = &placements.stats;
    if stats.format_sans_profil {
        unknown_format(stats.format_version, &mut observe);
    }
    let calibrated = FilmMppWidths {
        lead: stats.calibration.widths.lead as i64,
        index: stats.calibration.widths.index as i64,
    };
    if let Some(e) = &placement_error {
        log!(diagnostics, observe, "WARN", "poses d'equipement illisibles — rejeu sans equipement pose", "err" => e.to_string(), "match_id" => id);
    } else if !calibrated.is_valid() {
        log!(diagnostics, observe, "WARN", "poses d'equipement : le decoupage du bloc de replication n'a pas ete tranche sur ce film — AUCUNE pose publiee plutot que du bruit",
            "match_id" => id, "ancres" => stats.calibration.anchors, "vies" => stats.calibration.lives, "chunksLus" => stats.calibration.chunks);
    } else {
        log!(diagnostics, observe, "INFO", "poses d'equipement : records de creation ti=37",
            "decoupage" => calibrated.to_string(), "accords" => stats.calibration.agree,
            "ancres" => stats.anchors, "acceptes" => stats.accepted, "confirmes" => stats.confirmed, "poses" => stats.placements);
    }
    let mut published = FactsWorldSection {
        placements: if placement_error.is_none() {
            placements
                .placements
                .iter()
                .map(FactsPlacement::from)
                .collect()
        } else {
            vec![]
        },
        placement_stats: FactsPlacementStats::from(stats),
        ..Default::default()
    };
    observe(ReplayWorldObservation::Placements(
        (placement_error.is_none() && placements.creations.is_some())
            .then_some(published.placements.as_slice()),
    ));
    observe(ReplayWorldObservation::PlacementStats(
        &published.placement_stats,
    ));
    let (spawns, spawn_error) = scan_context_equipment_spawn_events(context);
    let s = &spawns.stats;
    if let Some(e) = &spawn_error {
        log!(diagnostics, observe, "WARN", "evenements de piece engendree illisibles — l'origine des poses retombe sur ses replis", "err" => e.to_string(), "match_id" => id);
    } else {
        log!(diagnostics, observe, "INFO", "poses d'equipement : evenements 103 (piece engendree)", "match_id" => id,
            "chunks" => s.chunks, "paquetsDelta" => s.packets, "listesNonVides" => s.lists,
            "evenements" => s.events, "refSource" => s.with_source, "refEngendree" => s.with_spawned, "ref2" => s.reference_2);
        published.spawn_events = spawns.records.iter().map(FactsSpawnEvent::from).collect();
    }
    published.spawn_stats = FactsSpawnStats::from(s);
    observe(ReplayWorldObservation::SpawnEvents(
        (!published.spawn_events.is_empty()).then_some(published.spawn_events.as_slice()),
    ));
    let pads = scan_replay_pad_inputs_observed(match_id, context, Some(map), calibrated, |e| {
        object_observation(e, &mut observe)
    });
    published.weapons = pads.weapons.published.clone();
    published.powerups = pads.powerups.published.clone();
    observe(ReplayWorldObservation::Pads(&pads));
    let vehicles =
        scan_replay_vehicle_inputs_observed(match_id, context, Some(map), calibrated, |e| {
            object_observation(e, &mut observe)
        });
    observe(ReplayWorldObservation::Vehicles(&vehicles.published));
    let guarded = scan_replay_guarded_inputs(match_id, context, guarded_options, |e| {
        observe(ReplayWorldObservation::Guarded(e))
    });
    ReplayWorldScan {
        zoom,
        placements,
        placement_error,
        spawns,
        spawn_error,
        pads,
        vehicles,
        guarded,
        diagnostics,
        published_zoom,
        published,
    }
}
