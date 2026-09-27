//! Loaded vehicle boarding/exit and biped aim-only evidence.
use super::*;
use std::collections::BTreeSet;
#[derive(Debug, thiserror::Error)]
pub enum ContextVehicleChannelError {
    #[error("aucun chunk de donnees dans le film")]
    NoChunks,
    #[error("aucun chunk film lisible")]
    NoReadableChunk,
    #[error("aucun slot biped (ti=35) dans les keyframes du film")]
    NoBipeds,
    #[error("truncated vehicle references in chunk {chunk}, packet {packet_index}")]
    TruncatedReferences {
        chunk: i64,
        packet_index: usize,
        source_packet: FilmPacket,
    },
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
}
#[derive(Debug, Clone, PartialEq)]
pub struct SourceVehicleEvent {
    pub source: FilmPacket,
    pub event: VehicleEvent,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SourceBipedAim {
    pub source: FilmPacket,
    pub record: BipedAimRecord,
}
fn biped_band(context: &NativeFilmContext<'_>) -> BTreeSet<u32> {
    context
        .biped_slot_band()
        .map(|[lo, hi]| (lo..=hi).collect())
        .unwrap_or_default()
}
/// Native ScanVehicleEvents allows an empty biped band. Refused references retain
/// the source packet and preceding events instead of reproducing the native panic.
pub fn scan_context_vehicle_events(
    context: &NativeFilmContext<'_>,
) -> (Vec<SourceVehicleEvent>, Option<ContextVehicleChannelError>) {
    let numbers = context.chunk_numbers();
    let band = biped_band(context);
    let base = band.first().copied().unwrap_or(0);
    let mut out = Vec::new();
    let mut read = false;
    for &chunk in numbers {
        let Some((data, packets)) = context.chunk_at(chunk) else {
            continue;
        };
        read = true;
        for (packet_index, &source) in packets.iter().enumerate() {
            if source.packet_type != 0 || source.payload_size == 0 {
                continue;
            }
            let payload = &data[source.payload_offset..source.payload_offset + source.payload_size];
            match decode_vehicle_event(payload, base, &band) {
                Ok(Some(mut event)) => {
                    event.chunk = chunk;
                    event.packet_index = packet_index;
                    event.timestamp_us = source.timestamp_us;
                    out.push(SourceVehicleEvent { source, event });
                }
                Ok(None) => {}
                Err(_) => {
                    return (
                        out,
                        Some(ContextVehicleChannelError::TruncatedReferences {
                            chunk,
                            packet_index,
                            source_packet: source,
                        }),
                    );
                }
            }
        }
    }
    (
        out,
        (!read).then_some(ContextVehicleChannelError::NoReadableChunk),
    )
}
/// Native ScanBipedAimOnly resolves the shared read context for each delta
/// packet after bounds/band admission. Vitality and companion readers themselves
/// have no profile-dependent fields or native observer publications.
pub fn scan_context_biped_aim(
    context: &NativeFilmContext<'_>,
) -> (Vec<SourceBipedAim>, Option<ContextVehicleChannelError>) {
    let mut out = Vec::new();
    let result = (|| -> Result<(), ContextVehicleChannelError> {
        let numbers = context.chunk_numbers();
        if numbers.is_empty() {
            return Err(ContextVehicleChannelError::NoChunks);
        }
        let band = biped_band(context);
        if band.is_empty() {
            return Err(ContextVehicleChannelError::NoBipeds);
        }
        let mut read = false;
        for &chunk in numbers {
            let Some((data, packets)) = context.chunk_at(chunk) else {
                continue;
            };
            read = true;
            for (packet_index, &source) in packets.iter().enumerate() {
                if source.packet_type != 0 {
                    continue;
                }
                let _context = context.reader_context()?;
                let payload =
                    &data[source.payload_offset..source.payload_offset + source.payload_size];
                for mut record in scan_biped_aim_records_with_sources(payload, &band) {
                    record.aim.chunk = chunk;
                    record.aim.packet_index = packet_index;
                    record.aim.timestamp_us = source.timestamp_us;
                    out.push(SourceBipedAim { source, record });
                }
            }
        }
        if !read {
            return Err(ContextVehicleChannelError::NoReadableChunk);
        }
        Ok(())
    })();
    (out, result.err())
}
