//! Native weapon/powerup pad scans over one loaded context.
use super::*;
#[derive(Debug, thiserror::Error)]
pub enum ReplayPadScanError {
    #[error("no archetype slots in keyframes")]
    NoBand,
    #[error(transparent)]
    Creations(ContextEquipmentCreationError),
    #[error(transparent)]
    Tracks(ContextWorldObjectError),
}
#[derive(Debug)]
pub struct ReplayPadScan {
    pub archetype: u32,
    pub format: FilmMppResolution,
    pub installed_widths: Option<FilmMppWidths>,
    pub census: SourceWorldObjectCensus,
    pub creations: Option<ContextEquipmentCreationScan>,
    pub tracks: Option<ContextWorldObjectTracks>,
    pub error: Option<ReplayPadScanError>,
    /// Native publication is wholly empty after any failed stage.
    pub published: FactsWorldObjectScan,
}
#[derive(Debug)]
pub struct ReplayPadScans {
    pub weapons: ReplayPadScan,
    pub powerups: ReplayPadScan,
    /// One native counter increment per unknown-format selection, in call order.
    pub unknown_formats: Vec<u32>,
}
struct PadMppRestore<'a, 's> {
    context: &'a mut NativeFilmContext<'s>,
    previous: Option<FilmMppWidths>,
}
impl Drop for PadMppRestore<'_, '_> {
    fn drop(&mut self) {
        if let Some(w) = self.previous {
            self.context.set_mpp(w);
        }
    }
}
/// Native format fallback boundary precedes profile installation and census.
pub enum ReplayWorldObjectScanObservation<'a> {
    UnknownFormat {
        archetype: u32,
        format: u32,
    },
    Creation {
        archetype: u32,
        attempt: &'a ContextEquipmentCreationAttempt,
    },
}
/// Native decodeFilmPadScans: weapons precede powerups. Recorded creation and
/// position evidence remains available even when native publication is cleared.
pub fn scan_replay_pad_inputs(
    match_id: &[u8],
    context: &mut NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    calibrated: FilmMppWidths,
    mut observe: impl FnMut(u32, &ContextEquipmentCreationAttempt),
) -> ReplayPadScans {
    scan_replay_pad_inputs_observed(match_id, context, map, calibrated, |event| {
        if let ReplayWorldObjectScanObservation::Creation { archetype, attempt } = event {
            observe(archetype, attempt);
        }
    })
}
/// Includes immediate native unknown-format counter boundaries.
pub fn scan_replay_pad_inputs_observed(
    match_id: &[u8],
    context: &mut NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    calibrated: FilmMppWidths,
    mut observe: impl FnMut(ReplayWorldObjectScanObservation<'_>),
) -> ReplayPadScans {
    let match_id = ReplayByteString(match_id.to_vec()).json_text();
    let weapons = scan_pad(context, map, calibrated, 42, &match_id, &mut observe);
    let powerups = scan_pad(context, map, calibrated, 37, &match_id, &mut observe);
    let unknown_formats = [&weapons, &powerups]
        .into_iter()
        .filter(|s| s.format.unknown_format)
        .map(|s| s.format.format_version)
        .collect();
    ReplayPadScans {
        weapons,
        powerups,
        unknown_formats,
    }
}
fn scan_pad(
    context: &mut NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    calibrated: FilmMppWidths,
    archetype: u32,
    match_id: &str,
    observe: &mut impl FnMut(ReplayWorldObjectScanObservation<'_>),
) -> ReplayPadScan {
    let format = context
        .source()
        .and_then(FilmSource::registry_chunk)
        .and_then(|b| b.get(4..8))
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .unwrap_or(0);
    let format = resolve_film_mpp(format);
    if format.unknown_format {
        observe(ReplayWorldObjectScanObservation::UnknownFormat {
            archetype,
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
    let restore = PadMppRestore { context, previous };
    let context = &*restore.context;
    let census = scan_source_world_object_keyframes(context.source(), &[archetype]);
    let kf = &census.archetypes[&archetype];
    let band = kf.band.clone();
    let label = if archetype == 42 {
        "armes au sol (ti=42)"
    } else {
        "power-ups de socle (ti=37)"
    };
    let mut out = ReplayPadScan {
        archetype,
        format,
        installed_widths,
        census,
        creations: None,
        tracks: None,
        error: None,
        published: FactsWorldObjectScan::default(),
    };
    let kf = &out.census.archetypes[&archetype];
    if band.is_empty() {
        tracing::warn!(
            archetype = label,
            match_id,
            imagesCles = kf.times_us.len(),
            "socles : aucun slot de l archetype aux images-cles — rejeu sans ce calque"
        );
        out.error = Some(ReplayPadScanError::NoBand);
        return out;
    }
    let (creations, error) = if archetype == 42 {
        scan_context_ground_weapon_creations_for_band(context, map, &band, |a| {
            observe(ReplayWorldObjectScanObservation::Creation {
                archetype,
                attempt: a,
            })
        })
    } else {
        scan_context_equipment_creations_for_band(context, map, &band, |a| {
            observe(ReplayWorldObjectScanObservation::Creation {
                archetype,
                attempt: a,
            })
        })
    };
    out.creations = Some(creations);
    if let Some(error) = error {
        tracing::warn!(archetype=label,err=%error,match_id,"socles : records de creation illisibles — rejeu sans ce calque");
        out.error = Some(ReplayPadScanError::Creations(error));
        return out;
    }
    let tracks = match scan_context_world_object_tracks_for_band(context, map, &band) {
        Ok(t) => t,
        Err(error) => {
            tracing::warn!(archetype=label,err=%error,match_id,"socles : pistes delta illisibles — AUCUN socle publie (sans elles, toute apparition passerait pour un objet apparu au repos)");
            out.error = Some(ReplayPadScanError::Tracks(error));
            return out;
        }
    };
    let c = out.creations.as_ref().unwrap();
    let st = &c.stats;
    tracing::info!(
        archetype = label,
        slots = st.slots,
        ancres = st.anchors,
        acceptees = st.accepted,
        imagesCles = kf.times_us.len(),
        viesRecensees = kf.seen_us.len(),
        pistesDelta = tracks.tracks.len(),
        "socles : balayage d archetype"
    );
    out.published = FactsWorldObjectScan {
        scanned: true,
        creations: c
            .records
            .iter()
            .map(|r| FactsEquipmentCreation::from(r.read.creation.as_ref().unwrap()))
            .collect(),
        stats: FactsCreationStats::from(st),
        keyframes: FactsWorldKeyframes::from(kf),
        tracks: tracks
            .tracks
            .iter()
            .map(FactsProjectileTrack::from)
            .collect(),
    };
    out.tracks = Some(tracks);
    out
}
impl From<&EquipmentCreationStats> for FactsCreationStats {
    fn from(s: &EquipmentCreationStats) -> Self {
        Self {
            slots: s.slots as i64,
            anchors: s.anchors as i64,
            overflow: s.overflow as i64,
            mask_bad: s.mask_bad as i64,
            pos_bad: s.pos_bad as i64,
            accepted: s.accepted as i64,
            mask_sparse: s.mask_sparse as i64,
            mask_full: s.mask_full as i64,
            no_i0: s.no_i0 as i64,
            with_ref: s.with_ref as i64,
            with_id: s.with_id as i64,
            with_ammo: s.with_ammo as i64,
        }
    }
}
