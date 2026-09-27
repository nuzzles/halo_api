//! Loaded-context world-object position scans, retaining pre-segmentation reads.
use super::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub struct ContextWorldObjectPosition {
    pub source: FilmPacket,
    pub packet_index: usize,
    pub record: WorldObjectPositionRecord,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContextWorldObjectTracks {
    /// Includes reads discarded by minimum-length and rest/gap segmentation.
    pub records: Vec<ContextWorldObjectPosition>,
    pub tracks: Vec<WorldObjectTrack>,
}
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ContextWorldObjectError {
    #[error("bornes monde absentes : sans elles le décodeur ne rend que des quanta")]
    NoBounds,
    #[error("aucun chunk de donnees dans le film")]
    NoChunks,
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error("aucun slot d'archétype ti={0} dans les keyframes du film")]
    NoSlotBand(i64),
}
/// Native ScanWorldObjectsForBand. Bounds are checked before profile resolution,
/// then chunk availability. The descriptor comes from the shared scan context.
/// Empty bands succeed when bounds and chunks exist.
pub fn scan_context_world_object_tracks_for_band(
    context: &NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    band: &BTreeSet<u32>,
) -> Result<ContextWorldObjectTracks, ContextWorldObjectError> {
    let map = map.ok_or(ContextWorldObjectError::NoBounds)?;
    let descriptor = context.scan_profile()?.movement.world_object;
    if context.chunk_numbers().is_empty() {
        return Err(ContextWorldObjectError::NoChunks);
    }
    let mut out = ContextWorldObjectTracks::default();
    for &chunk in context.chunk_numbers() {
        let Some((data, packets)) = context.chunk_at(chunk) else {
            continue;
        };
        for (packet_index, &source) in packets.iter().enumerate() {
            if source.packet_type != 0 {
                continue;
            }
            let payload = &data[source.payload_offset..source.payload_offset + source.payload_size];
            for mut record in
                decode_world_object_positions_with_descriptor(payload, band, map, &descriptor)
            {
                record.sample.timestamp_us = source.timestamp_us;
                record.sample.chunk = chunk;
                out.records.push(ContextWorldObjectPosition {
                    source,
                    packet_index,
                    record,
                });
            }
        }
    }
    out.tracks =
        assemble_world_object_tracks(out.records.iter().map(|r| r.record.clone()).collect());
    Ok(out)
}

/// Native ScanWorldObjects admission order: chunks, recovered slot band, then
/// bounds/profile validation. Retains the keyframes used for band selection.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextWorldObjectScan {
    pub census: SourceWorldObjectCensus,
    pub positions: ContextWorldObjectTracks,
}

pub fn scan_context_world_objects(
    context: &NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    archetype: i64,
) -> Result<ContextWorldObjectScan, ContextWorldObjectError> {
    if context.chunk_numbers().is_empty() {
        return Err(ContextWorldObjectError::NoChunks);
    }
    // Native keyframe type indices are nonnegative and fit in u32; an out-of-
    // domain requested type cannot match one. Do not narrow and alias it.
    let ti =
        u32::try_from(archetype).map_err(|_| ContextWorldObjectError::NoSlotBand(archetype))?;
    let census = scan_source_world_object_keyframes(context.source(), &[ti]);
    let band = &census.archetypes[&ti].band;
    if band.is_empty() {
        return Err(ContextWorldObjectError::NoSlotBand(archetype));
    }
    let positions = scan_context_world_object_tracks_for_band(context, map, band)?;
    Ok(ContextWorldObjectScan { census, positions })
}

/// Native ScanProjectiles uses archetype 41, independently of grenade name votes.
/// Successful empty trajectories remain an allocated list; no impact is inferred.
pub fn scan_context_projectiles(
    context: &NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
) -> Result<ContextWorldObjectScan, ContextWorldObjectError> {
    scan_context_world_objects(context, map, 41)
}
