//! Loaded native placement pass: census, tracks, calibration, creations, confirmation.
use super::*;
#[derive(Debug, thiserror::Error)]
pub enum ContextPlacementError {
    #[error("bornes monde absentes : sans elles le décodeur ne rend que des quanta")]
    NoBounds,
    #[error("aucun chunk de donnees dans le film")]
    NoChunks,
    #[error("aucun slot d'archétype ti=37 dans les keyframes du film")]
    NoBand,
    #[error(transparent)]
    Tracks(#[from] ContextWorldObjectError),
    #[error(transparent)]
    Calibration(#[from] ContextMppCalibrationError),
    #[error(transparent)]
    Creations(#[from] ContextEquipmentCreationError),
}
pub enum ContextPlacementObservation<'a> {
    Calibration(&'a ContextMppCalibrationAttempt),
    Creation(&'a ContextEquipmentCreationAttempt),
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContextEquipmentPlacements {
    pub placements: Vec<EquipmentPlacement>,
    pub stats: EquipmentPlacementStats,
    pub census: Option<SourceWorldObjectCensus>,
    pub tracks: Option<ContextWorldObjectTracks>,
    pub calibration: Option<ContextMppCalibration>,
    pub creations: Option<ContextEquipmentCreationScan>,
    /// Actual creation-read widths, distinct from the measured calibration.
    pub creation_widths: Option<FilmMppWidths>,
}
struct RestoreMpp<'a, 's> {
    context: &'a mut NativeFilmContext<'s>,
    previous: FilmMppWidths,
}
impl Drop for RestoreMpp<'_, '_> {
    fn drop(&mut self) {
        self.context.set_mpp(self.previous);
    }
}
/// Native ScanEquipmentPlacements. Format-selected widths take precedence for
/// creation decoding, while statistics keep the independently measured widths.
/// Temporary profile installation is restored on normal return and unwinding.
pub fn scan_context_equipment_placements(
    context: &mut NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    mut observe: impl FnMut(ContextPlacementObservation<'_>),
) -> (ContextEquipmentPlacements, Option<ContextPlacementError>) {
    let mut out = ContextEquipmentPlacements::default();
    let result = (|| -> Result<(), ContextPlacementError> {
        let map = map.ok_or(ContextPlacementError::NoBounds)?;
        if context.chunk_numbers().is_empty() {
            return Err(ContextPlacementError::NoChunks);
        }
        let census = scan_source_world_object_keyframes(context.source(), &[37]);
        let band = census.archetypes[&37].band.clone();
        out.census = Some(census);
        if band.is_empty() {
            return Err(ContextPlacementError::NoBand);
        }
        out.stats.slots = band.len();
        let tracks = scan_context_world_object_tracks_for_band(context, Some(map), &band)?;
        let spans = equipment_life_spans(&tracks.tracks);
        out.stats.lives = spans.len();
        out.tracks = Some(tracks);
        // MPP format selection is independent of build-profile membership.
        let format = context
            .source()
            .and_then(FilmSource::registry_chunk)
            .and_then(|b| b.get(4..8))
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .unwrap_or(0);
        let resolution = resolve_film_mpp(format);
        out.stats.format_version = format;
        out.stats.format_sans_profil = resolution.unknown_format;
        let cal = calibrate_context_equipment_mpp(context, Some(map), &band, &spans, |a| {
            observe(ContextPlacementObservation::Calibration(&a))
        })?;
        out.stats.calibration = cal.calibration.clone();
        out.stats.scanned = true;
        let chosen = resolution.widths.or_else(|| {
            cal.conclusive
                .then_some([cal.calibration.widths.lead, cal.calibration.widths.index])
        });
        out.calibration = Some(cal);
        let Some([lead, index]) = chosen else {
            return Ok(());
        };
        let widths = FilmMppWidths {
            lead: lead as i64,
            index: index as i64,
        };
        out.creation_widths = Some(widths);
        let previous = context.set_mpp(widths);
        let (creations, error) = {
            let restore = RestoreMpp { context, previous };
            scan_context_equipment_creations_for_band(restore.context, Some(map), &band, |a| {
                observe(ContextPlacementObservation::Creation(a))
            })
        };
        out.creations = Some(creations);
        if let Some(error) = error {
            return Err(error.into());
        }
        let creations = out.creations.as_ref().unwrap();
        out.stats.anchors = creations.stats.anchors;
        out.stats.accepted = creations.stats.accepted;
        let raw: Vec<_> = creations
            .records
            .iter()
            .map(|r| r.read.creation.as_ref().unwrap().clone())
            .collect();
        let (placements, confirmed) =
            confirm_equipment_placements(&raw, &spans, equipment_position_epsilon(map));
        out.stats.confirmed = confirmed;
        out.stats.placements = placements.len();
        for p in &placements {
            *out.stats.by_id.entry(p.global_id).or_default() += 1;
        }
        out.placements = placements;
        Ok(())
    })();
    (out, result.err())
}
impl From<&EquipmentPlacement> for FactsPlacement {
    fn from(p: &EquipmentPlacement) -> Self {
        Self {
            start_us: p.t0_us,
            end_us: p.t1_us,
            life: FactsLifeKey {
                slot: p.life.slot,
                generation: p.life.generation,
            },
            position: [p.x, p.y, p.z],
            global_id: p.global_id,
            points: p.points as i64,
        }
    }
}
impl From<&EquipmentMppCalibration> for FactsMppCalibration {
    fn from(c: &EquipmentMppCalibration) -> Self {
        Self {
            widths: (c.widths.lead as i64, c.widths.index as i64),
            agree: c.agree as i64,
            runner: (c.runner.lead as i64, c.runner.index as i64),
            runner_agree: c.runner_agree as i64,
            anchors: c.anchors as i64,
            chunks: c.chunks as i64,
            lives: c.lives as i64,
            by_widths: Some(
                c.by_widths
                    .iter()
                    .map(|s| {
                        (
                            (s.widths.lead as i64, s.widths.index as i64),
                            s.agree as i64,
                        )
                    })
                    .collect(),
            ),
        }
    }
}
impl From<&EquipmentPlacementStats> for FactsPlacementStats {
    fn from(s: &EquipmentPlacementStats) -> Self {
        Self {
            scanned: s.scanned,
            calibration: {
                let mut c = FactsMppCalibration::from(&s.calibration);
                if !s.scanned {
                    c.by_widths = None;
                }
                c
            },
            lives: s.lives as i64,
            slots: s.slots as i64,
            anchors: s.anchors as i64,
            accepted: s.accepted as i64,
            confirmed: s.confirmed as i64,
            placements: s.placements as i64,
            by_id: Some(s.by_id.iter().map(|(&k, &v)| (k, v as i64)).collect()),
            format_version: s.format_version as i64,
            format_sans_profil: s.format_sans_profil,
        }
    }
}
