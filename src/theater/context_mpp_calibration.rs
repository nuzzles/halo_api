//! Loaded-source MPP calibration with native candidate ordering and evidence.
use super::*;
use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMppCalibrationRefusal {
    NoBounds,
    EmptyBand,
    NoLives,
    Registry(NativeContextRegistryError),
    MissingArchetype,
    NoChunks,
}
#[derive(Debug, thiserror::Error)]
pub enum ContextMppCalibrationError {
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error(transparent)]
    Reader(#[from] NativeReaderProfileError),
}
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMppCalibrationAttempt {
    pub chunk: i64,
    pub source: FilmPacket,
    pub packet_index: usize,
    pub bit: usize,
    pub widths: EquipmentMppWidths,
    pub read: NativeEquipmentCreationAttempt,
    /// Index in the supplied life's span list; None means no positional witness.
    pub matched_span: Option<usize>,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContextMppCalibration {
    pub calibration: EquipmentMppCalibration,
    pub conclusive: bool,
    pub refusal: Option<ContextMppCalibrationRefusal>,
}
/// Calibrate against supplied mobile lifetimes. Each speculative attempt is
/// delivered in native traversal/candidate order; callers can retain or stream
/// that evidence without forcing all 63 attempts per anchor into memory.
/// Candidate profiles are local copies, so the context's installed MPP and hooks
/// remain intact on every return, including errors.
pub fn calibrate_context_equipment_mpp(
    context: &NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    band: &BTreeSet<u32>,
    spans: &EquipmentLifeSpans,
    mut observe: impl FnMut(ContextMppCalibrationAttempt),
) -> Result<ContextMppCalibration, ContextMppCalibrationError> {
    let mut out = ContextMppCalibration {
        calibration: EquipmentMppCalibration {
            lives: spans.len(),
            ..Default::default()
        },
        ..Default::default()
    };
    let refusal = if map.is_none() {
        Some(ContextMppCalibrationRefusal::NoBounds)
    } else if band.is_empty() {
        Some(ContextMppCalibrationRefusal::EmptyBand)
    } else if spans.is_empty() {
        Some(ContextMppCalibrationRefusal::NoLives)
    } else {
        None
    };
    if refusal.is_some() {
        out.refusal = refusal;
        return Ok(out);
    }
    let map = map.unwrap();
    let registry = match context.registry() {
        Ok(r) => r,
        Err(e) => {
            out.refusal = Some(ContextMppCalibrationRefusal::Registry(*e));
            return Ok(out);
        }
    };
    let Some(arch) = registry.registry.archetype(37) else {
        out.refusal = Some(ContextMppCalibrationRefusal::MissingArchetype);
        return Ok(out);
    };
    let numbers = context.chunk_numbers();
    if numbers.is_empty() {
        out.refusal = Some(ContextMppCalibrationRefusal::NoChunks);
        return Ok(out);
    }
    let mut profile = context.scan_profile()?;
    let eps = equipment_position_epsilon(map);
    let mut scores = [0usize; 63];
    for &chunk in numbers {
        let Some((data, packets)) = context.chunk_at(chunk) else {
            continue;
        };
        out.calibration.chunks += 1;
        for (packet_index, &source) in packets.iter().enumerate() {
            if source.packet_type != 0 {
                continue;
            }
            let payload = &data[source.payload_offset..source.payload_offset + source.payload_size];
            let Some(limit) = payload.len().saturating_mul(8).checked_sub(24) else {
                continue;
            };
            for bit in 0..=limit {
                let Some((slot, generation)) =
                    super::equipment_creations::equipment_new_header(payload, bit, band)
                else {
                    continue;
                };
                let Some(lives) = spans
                    .get(&EquipmentLifeKey { slot, generation })
                    .filter(|v| !v.is_empty())
                else {
                    continue;
                };
                out.calibration.anchors += 1;
                for (i, widths) in super::equipment_placements::candidates().enumerate() {
                    profile.mpp = FilmMppWidths {
                        lead: widths.lead as i64,
                        index: widths.index as i64,
                    };
                    let read = read_native_equipment_creation(
                        payload,
                        bit,
                        arch.components.len(),
                        map,
                        &profile,
                    )?;
                    let matched_span = read
                        .creation
                        .as_ref()
                        .and_then(|c| {
                            match_equipment_life(lives, [c.x, c.y, c.z], eps, source.timestamp_us)
                        })
                        .and_then(|matched| lives.iter().position(|v| std::ptr::eq(v, matched)));
                    scores[i] += usize::from(matched_span.is_some());
                    observe(ContextMppCalibrationAttempt {
                        chunk,
                        source,
                        packet_index,
                        bit,
                        widths,
                        read,
                        matched_span,
                    });
                }
            }
        }
        out.calibration.by_widths = super::equipment_placements::candidates()
            .zip(scores)
            .filter(|(_, n)| *n > 0)
            .map(|(widths, agree)| EquipmentMppScore { widths, agree })
            .collect();
        out.conclusive = super::equipment_placements::verdict(&mut out.calibration);
        if out.conclusive {
            break;
        }
    }
    Ok(out)
}
