//! Loaded-context biped creation scan and native facts projection.
use super::*;

#[derive(Debug, thiserror::Error)]
pub enum ContextCreationScanError {
    #[error("contexte de film absent")]
    AbsentContext,
    #[error("aucun chunk de donnees dans le film")]
    NoChunks,
    #[error("aucun slot biped (ti=35) dans les keyframes du film")]
    NoBipeds,
    #[error("bande de slots vide")]
    EmptyBand,
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error(transparent)]
    Band(#[from] DecodeError),
}

/// Native ScanBipedCreations / ForBand. With no explicit band, use the context's
/// cached whole-film biped band, unlike selected-chunk position scanning. Source
/// admission precedes band admission. This scanner resolves the scan profile but
/// emits no observer hooks. Output packet identities reference source buffers.
pub fn scan_context_biped_creations(
    context: Option<&NativeFilmContext<'_>>,
    band: Option<&FilmSlotBand>,
) -> Result<BipedCreationStream, ContextCreationScanError> {
    let context = context.ok_or(ContextCreationScanError::AbsentContext)?;
    let chunks = context.chunk_numbers();
    if chunks.is_empty() {
        return Err(ContextCreationScanError::NoChunks);
    }
    let discovered;
    let band = match band {
        Some(band) => band,
        None => {
            let [lo, hi] = context
                .biped_slot_band()
                .ok_or(ContextCreationScanError::NoBipeds)?;
            discovered = FilmSlotBand::from_slots(lo..=hi)?;
            &discovered
        }
    };
    if band.is_empty() {
        return Err(ContextCreationScanError::EmptyBand);
    }
    let _profile = context.scan_profile()?;
    let slots = band.slots();
    let mut out = BipedCreationStream {
        slot_band: Some([slots[0], *slots.last().unwrap()]),
        slots,
        stats: BipedCreationStats {
            slots: band.count(),
            ..Default::default()
        },
        records: Vec::new(),
    };
    for &number in chunks {
        let Some((data, packets)) = context.chunk_at(number) else {
            continue;
        };
        for (packet_index, &source) in packets.iter().enumerate() {
            if source.packet_type != 0 {
                continue;
            }
            let payload = &data[source.payload_offset..source.payload_offset + source.payload_size];
            for creation in super::biped_creation::scan_biped_creation_records_matching(
                payload,
                |slot| band.contains(slot),
                &mut out.stats,
            ) {
                out.records.push(FilmBipedCreation {
                    source,
                    creation,
                    packet_index: Some(packet_index),
                });
            }
        }
    }
    Ok(out)
}

/// Source scanners publish only accepted prologues with a recorded participant.
/// Ranges, version and representation remain in the source record; the native
/// cache stores only timestamp, lifetime and participant fields.
impl From<&FilmBipedCreation> for FactsBipedCreation {
    fn from(c: &FilmBipedCreation) -> Self {
        Self {
            timestamp_us: c.source.timestamp_us,
            slot: c.creation.slot,
            generation: u32::from(c.creation.generation),
            has_index: true,
            participant_index: u32::from(c.creation.participant_index),
        }
    }
}
