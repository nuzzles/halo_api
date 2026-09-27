//! Native equipment creation scanning over the loaded source and shared profile.
use super::*;
use std::collections::BTreeSet;
#[derive(Debug, thiserror::Error)]
pub enum ContextEquipmentCreationError {
    #[error("bornes monde absentes : sans elles le décodeur ne rend que des quanta")]
    NoBounds,
    #[error("bornes absentes : sans elles le decodeur ne rend que des quanta")]
    NoVehicleBounds,
    #[error("archetype vehicule 40 absent du registre")]
    NoVehicleArchetype,
    #[error("decoupage i0 illisible : {0}")]
    VehicleLayout(String),
    #[error("aucun chunk de donnees dans le film")]
    NoChunks,
    #[error("chunk_00 (registre) absent du film")]
    NoRegistry,
    #[error("archétype équipement 37 absent du registre")]
    NoArchetype,
    #[error("archétype arme au sol 42 absent du registre")]
    NoGroundArchetype,
    #[error(transparent)]
    Registry(NativeContextRegistryError),
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error(transparent)]
    Reader(#[from] NativeReaderProfileError),
}
#[derive(Debug, Clone, PartialEq)]
pub struct ContextEquipmentCreationAttempt {
    pub chunk: i64,
    pub source: FilmPacket,
    pub packet_index: usize,
    pub bit: usize,
    pub read: NativeEquipmentCreationAttempt,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContextEquipmentCreationScan {
    /// Accepted creations, including their full default-state read.
    pub records: Vec<ContextEquipmentCreationAttempt>,
    pub stats: EquipmentCreationStats,
}
/// Native ScanEquipmentCreationsForBand. Empty bands still perform registry and
/// profile admission. Rejected attempts stream to the observer, while accepted
/// reads also remain in the result. Accepted bodies are skipped during scanning.
pub fn scan_context_equipment_creations_for_band(
    context: &NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    band: &BTreeSet<u32>,
    mut observe: impl FnMut(&ContextEquipmentCreationAttempt),
) -> (
    ContextEquipmentCreationScan,
    Option<ContextEquipmentCreationError>,
) {
    scan_context_world_creations_for_band(context, map, band, 37, &mut observe)
}
/// Loaded native ground-weapon creations, including the separate ammo trace.
pub fn scan_context_ground_weapon_creations_for_band(
    context: &NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    band: &BTreeSet<u32>,
    mut observe: impl FnMut(&ContextEquipmentCreationAttempt),
) -> (
    ContextEquipmentCreationScan,
    Option<ContextEquipmentCreationError>,
) {
    scan_context_world_creations_for_band(context, map, band, 42, &mut observe)
}
/// Loaded ti40 creations, admitting the cached dynamic i0 layout before registry.
pub fn scan_context_vehicle_creations_for_band(
    context: &NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    band: &BTreeSet<u32>,
    mut observe: impl FnMut(&ContextEquipmentCreationAttempt),
) -> (
    ContextEquipmentCreationScan,
    Option<ContextEquipmentCreationError>,
) {
    scan_context_world_creations_for_band(context, map, band, 40, &mut observe)
}
fn vehicle_layout_error(d: &I0LayoutDetection) -> String {
    match d.refusal.as_ref().unwrap_or(&I0LayoutRefusal::Inconclusive) {
        I0LayoutRefusal::NoChunks => "aucun chunk de donnees dans le film".into(),
        I0LayoutRefusal::NoBipeds => "aucun slot biped (ti=35) dans le film".into(),
        I0LayoutRefusal::Inconclusive => format!(
            "profil i0 non concluant : {} frontière(s) détectée(s) sur {} paires",
            d.report.boundaries.len(),
            d.report.pairs
        ),
        I0LayoutRefusal::Implausible => {
            format!("découpage i0 implausible : {}", d.layout.as_ref().unwrap())
        }
    }
}
fn scan_context_world_creations_for_band(
    context: &NativeFilmContext<'_>,
    map: Option<&FilmMapBounds>,
    band: &BTreeSet<u32>,
    archetype: u32,
    observe: &mut impl FnMut(&ContextEquipmentCreationAttempt),
) -> (
    ContextEquipmentCreationScan,
    Option<ContextEquipmentCreationError>,
) {
    let mut out = ContextEquipmentCreationScan::default();
    let result = (|| -> Result<(), ContextEquipmentCreationError> {
        let map = map.ok_or(if archetype == 40 {
            ContextEquipmentCreationError::NoVehicleBounds
        } else {
            ContextEquipmentCreationError::NoBounds
        })?;
        let numbers = context.chunk_numbers();
        if numbers.is_empty() {
            return Err(ContextEquipmentCreationError::NoChunks);
        }
        out.stats.slots = band.len();
        let vehicle_layout = if archetype == 40 {
            let d = context.i0_layout();
            if d.refusal.is_some() || d.layout.is_none() {
                return Err(ContextEquipmentCreationError::VehicleLayout(
                    vehicle_layout_error(d),
                ));
            }
            d.layout.as_ref()
        } else {
            None
        };
        let registry = context.registry().map_err(|e| match e {
            NativeContextRegistryError::NoRegistryChunk => {
                ContextEquipmentCreationError::NoRegistry
            }
            _ => ContextEquipmentCreationError::Registry(*e),
        })?;
        let arch = registry
            .registry
            .archetype(archetype as usize)
            .ok_or(if archetype == 37 {
                ContextEquipmentCreationError::NoArchetype
            } else if archetype == 40 {
                ContextEquipmentCreationError::NoVehicleArchetype
            } else {
                ContextEquipmentCreationError::NoGroundArchetype
            })?;
        let profile = context.scan_profile()?;
        for &chunk in numbers {
            let Some((data, packets)) = context.chunk_at(chunk) else {
                continue;
            };
            for (packet_index, &source) in packets.iter().enumerate() {
                if source.packet_type != 0 {
                    continue;
                }
                let payload =
                    &data[source.payload_offset..source.payload_offset + source.payload_size];
                let Some(limit) = payload.len().saturating_mul(8).checked_sub(24) else {
                    continue;
                };
                let mut bit = 0;
                while bit <= limit {
                    let at = bit;
                    bit += 1;
                    if super::equipment_creations::world_object_new_header(
                        payload, at, band, archetype,
                    )
                    .is_none()
                    {
                        continue;
                    }
                    out.stats.anchors += 1;
                    let mut read = if archetype == 37 {
                        read_native_equipment_creation(
                            payload,
                            at,
                            arch.components.len(),
                            map,
                            &profile,
                        )?
                    } else if let Some(layout) = vehicle_layout {
                        read_native_vehicle_creation(
                            payload,
                            at,
                            arch.components.len(),
                            map,
                            &profile,
                            layout,
                        )?
                    } else {
                        read_native_ground_weapon_creation(payload, at, arch, map, &profile)?
                    };
                    match read.refusal {
                        Some(NativeEquipmentCreationRefusal::DefaultOverflow) => {
                            out.stats.overflow += 1
                        }
                        Some(NativeEquipmentCreationRefusal::Mask) => out.stats.mask_bad += 1,
                        Some(NativeEquipmentCreationRefusal::Position) => out.stats.pos_bad += 1,
                        None => {}
                    }
                    if let Some(c) = &mut read.creation {
                        c.chunk = chunk;
                        c.packet_index = packet_index;
                        c.timestamp_us = source.timestamp_us;
                        out.stats.accepted += 1;
                        out.stats.mask_full += usize::from(c.mask_full);
                        out.stats.mask_sparse += usize::from(!c.mask_full);
                        out.stats.no_i0 += usize::from(!c.mask_has_i0);
                        out.stats.with_ref += usize::from(c.has_ref);
                        out.stats.with_id += usize::from(c.has_id);
                        out.stats.with_ammo += usize::from(c.has_ammo);
                        bit = c.after_bit;
                    }
                    let attempt = ContextEquipmentCreationAttempt {
                        chunk,
                        source,
                        packet_index,
                        bit: at,
                        read,
                    };
                    observe(&attempt);
                    if attempt.read.creation.is_some() {
                        out.records.push(attempt);
                    }
                }
            }
        }
        Ok(())
    })();
    (out, result.err())
}
impl From<&EquipmentCreation> for FactsEquipmentCreation {
    fn from(c: &EquipmentCreation) -> Self {
        Self {
            timestamp_us: c.timestamp_us,
            slot: c.slot,
            generation: c.generation,
            chunk: c.chunk,
            packet_index: c.packet_index as i64,
            bit_pos: c.bit_pos as i64,
            has_ref: c.has_ref,
            reference: c.reference,
            has_id: c.has_id,
            ability_id: c.ability_id,
            mpp_present: c.mpp_present,
            mpp_val: c.mpp_val,
            position: [c.x, c.y, c.z],
            mask: Some(c.mask.iter().map(|&v| v as i64).collect()),
            mask_full: c.mask_full,
            mask_has_i0: c.mask_has_i0,
            default_state_bits: c.default_state_bits as i64,
            has_ammo: c.has_ammo,
            ammo: c.ammo.clone(),
            after_bit: c.after_bit as i64,
        }
    }
}
