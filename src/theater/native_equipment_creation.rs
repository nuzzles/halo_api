//! Native equipment creation body attempts used by scanning and MPP calibration.
use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeEquipmentCreationRefusal {
    DefaultOverflow,
    Mask,
    Position,
}
#[derive(Debug, Clone, PartialEq)]
pub struct NativeEquipmentCreationAttempt {
    pub default_state: DecodedComponent,
    pub mask: Vec<usize>,
    pub mask_full: bool,
    pub component_start: Option<i64>,
    pub refusal: Option<NativeEquipmentCreationRefusal>,
    pub creation: Option<EquipmentCreation>,
    pub ammo_read: Option<NativeGroundWeaponAmmoRead>,
}
/// Read after an already-admitted 24-bit ti37 NEW header. Retains default-state
/// fields and callbacks even when subsequent mask/position admission fails.
/// Uses a private observer, as the reference creation walker does.
pub fn read_native_equipment_creation(
    payload: &[u8],
    bit: usize,
    components: usize,
    map: &FilmMapBounds,
    profile: &NativeScanProfile,
) -> Result<NativeEquipmentCreationAttempt, NativeReaderProfileError> {
    read_native_world_creation(
        payload,
        bit,
        components,
        map,
        profile,
        WorldCreationKind::Equipment,
    )
}
/// Native ti42 body, with an independent complete ammo-component traversal.
pub fn read_native_ground_weapon_creation(
    payload: &[u8],
    bit: usize,
    arch: &FilmArchetype,
    map: &FilmMapBounds,
    profile: &NativeScanProfile,
) -> Result<NativeEquipmentCreationAttempt, NativeReaderProfileError> {
    read_native_world_creation(
        payload,
        bit,
        arch.components.len(),
        map,
        profile,
        WorldCreationKind::Ground(arch),
    )
}
/// Native ti40 body. The dynamic i0 layout is independent of media-frame widths.
pub fn read_native_vehicle_creation(
    payload: &[u8],
    bit: usize,
    components: usize,
    map: &FilmMapBounds,
    profile: &NativeScanProfile,
    layout: &I0Layout,
) -> Result<NativeEquipmentCreationAttempt, NativeReaderProfileError> {
    read_native_world_creation(
        payload,
        bit,
        components,
        map,
        profile,
        WorldCreationKind::Vehicle(layout),
    )
}
#[derive(Clone, Copy)]
enum WorldCreationKind<'a> {
    Equipment,
    Ground(&'a FilmArchetype),
    Vehicle(&'a I0Layout),
}
fn read_native_world_creation(
    payload: &[u8],
    bit: usize,
    components: usize,
    map: &FilmMapBounds,
    profile: &NativeScanProfile,
    kind: WorldCreationKind<'_>,
) -> Result<NativeEquipmentCreationAttempt, NativeReaderProfileError> {
    let mut reader = NativeFilmReader::with_context(
        payload,
        NativeReaderContext {
            profile: profile.clone(),
            observer: Some(NativeFilmObserver::default()),
        },
    );
    let start = (bit as i64).wrapping_add(24);
    reader.set_native_bit_position(start);
    let (_, default_state) = match kind {
        WorldCreationKind::Equipment => reader.read_equipment_default_state()?,
        WorldCreationKind::Ground(_) => reader.read_ground_weapon_default_state()?,
        WorldCreationKind::Vehicle(_) => reader.read_vehicle_default_state()?,
    };
    let total = (payload.len() as i64).wrapping_mul(8);
    let mut out = NativeEquipmentCreationAttempt {
        default_state,
        mask: Vec::new(),
        mask_full: false,
        component_start: None,
        refusal: None,
        creation: None,
        ammo_read: None,
    };
    if reader.native_bit_position() > total {
        out.refusal = Some(NativeEquipmentCreationRefusal::DefaultOverflow);
        return Ok(out);
    }
    reader.read_bits_wide(1);
    out.mask_full = reader.read_bits_wide(1) != 0;
    let mut valid = true;
    if out.mask_full {
        let mask = reader.read_bits_wide(64);
        for i in 0..64 {
            if mask & (1u64 << i) == 0 {
                continue;
            }
            if i >= components {
                valid = false;
                break;
            }
            out.mask.push(i);
        }
    } else {
        let count = reader.read_bits_wide(3);
        if count == 0 {
            valid = false;
        }
        for _ in 0..count {
            let i = reader.read_bits_wide(6) as usize;
            if i >= components || out.mask.last().is_some_and(|p| *p >= i) {
                valid = false;
                break;
            }
            out.mask.push(i);
        }
    }
    if !valid || out.mask.is_empty() || reader.native_bit_position() > total {
        out.refusal = Some(NativeEquipmentCreationRefusal::Mask);
        return Ok(out);
    }
    let at = reader.native_bit_position();
    out.component_start = Some(at);
    let (pos, position_bits) = if let WorldCreationKind::Vehicle(layout) = kind {
        (
            decode_vehicle_i0_position(payload, at, layout, &map.coordinate_bounds()),
            layout.total_bits(),
        )
    } else {
        let d = &profile.movement.world_object;
        let width = |n| {
            usize::try_from(n)
                .map_err(|_| NativeReaderProfileError::Width("world object precision"))
        };
        let precision = WorldObjectPrecision {
            index_bits: width(d.index_bits)?,
            axis_bits: [
                width(d.axis_bits[0])?,
                width(d.axis_bits[1])?,
                width(d.axis_bits[2])?,
            ],
            region: d.region,
        };
        (
            usize::try_from(at)
                .ok()
                .and_then(|at| {
                    decode_world_object_position_with_precision(payload, at, map, precision)
                })
                .map(|(xyz, _)| xyz),
            precision.bit_length().unwrap_or(0) as i64,
        )
    };
    let Some(xyz) = pos else {
        out.refusal = Some(NativeEquipmentCreationRefusal::Position);
        return Ok(out);
    };
    if let WorldCreationKind::Ground(arch) = kind {
        out.ammo_read = Some(read_native_ground_weapon_ammo(
            payload,
            at,
            &out.mask,
            arch,
            &reader.context(),
        )?);
    }
    let mut cre = EquipmentCreation {
        mask: out.mask.clone(),
        mask_full: out.mask_full,
        mask_has_i0: out.mask[0] == 0,
        default_state_bits: out.default_state.end_bit.wrapping_sub(start) as usize,
        after_bit: at.wrapping_add(position_bits) as usize,
        x: xyz[0],
        y: xyz[1],
        z: xyz[2],
        bit_pos: bit,
        ..Default::default()
    };
    if let Some(ammo) = out.ammo_read.as_ref().and_then(|r| r.ammo.as_ref()) {
        cre.has_ammo = true;
        cre.ammo = ammo.clone();
    }
    let mut header = NativeFilmBits::new(payload);
    header.set_position(bit as i64 + 3);
    cre.slot = header.read(13) as u32;
    cre.generation = header.read(2) as u32;
    for v in out
        .default_state
        .diagnostics
        .component_observations
        .iter()
        .chain(
            out.ammo_read
                .iter()
                .flat_map(|r| r.components.iter())
                .filter_map(|c| c.read.as_ref())
                .flat_map(|r| r.diagnostics.component_observations.iter()),
        )
    {
        match v {
            FilmComponentObservation::Mpp {
                field,
                value,
                present,
            } => {
                let i = *field as usize;
                cre.mpp_present[i] = *present;
                cre.mpp_val[i] = *value;
            }
            FilmComponentObservation::EquipmentCreation {
                field,
                value,
                present,
            } => match field {
                NativeEquipmentCreationField::Reference => {
                    cre.has_ref = *present;
                    cre.reference = *value as u32;
                }
                NativeEquipmentCreationField::AbilityId => {
                    cre.has_id = *present;
                    cre.ability_id = *value as u32;
                }
            },
            _ => {}
        }
    }
    out.creation = Some(cre);
    Ok(out)
}
