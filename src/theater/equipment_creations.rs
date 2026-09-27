//! Equipment NEW records, with fields emitted by the shared default-state reader.
use super::{
    ComponentDecode, FilmMapBounds, PositionEncoding, WorldObjectPrecision,
    bits::{Bits, Cursor},
    decode_default_state, decode_world_object_position_with_precision,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GroundWeaponAmmo {
    pub mag: u32,
    pub res: u32,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EquipmentCreation {
    pub slot: u32,
    #[serde(rename = "Gen")]
    pub generation: u32,
    pub chunk: i64,
    pub packet_index: usize,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub bit_pos: usize,
    pub has_ref: bool,
    #[serde(rename = "Ref")]
    pub reference: u32,
    #[serde(rename = "HasID")]
    pub has_id: bool,
    #[serde(rename = "AbilityID")]
    pub ability_id: u32,
    #[serde(rename = "MPPPresent")]
    pub mpp_present: [bool; 4],
    #[serde(rename = "MPPVal")]
    pub mpp_val: [u64; 4],
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub mask: Vec<usize>,
    pub mask_full: bool,
    #[serde(rename = "MaskHasI0")]
    pub mask_has_i0: bool,
    pub default_state_bits: usize,
    pub has_ammo: bool,
    pub ammo: GroundWeaponAmmo,
    pub after_bit: usize,
}
impl EquipmentCreation {
    /// Number of zero-padded bits used to decode the position. The two trailing
    /// position flags are skipped by the reference and are not counted as reads.
    pub fn position_padded_bits(&self, payload_bytes: usize) -> usize {
        self.after_bit
            .saturating_sub(2)
            .saturating_sub(payload_bytes.saturating_mul(8))
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EquipmentCreationStats {
    pub slots: usize,
    pub anchors: usize,
    pub overflow: usize,
    pub mask_bad: usize,
    pub pos_bad: usize,
    pub accepted: usize,
    pub mask_sparse: usize,
    pub mask_full: usize,
    #[serde(rename = "NoI0")]
    pub no_i0: usize,
    pub with_ref: usize,
    #[serde(rename = "WithID")]
    pub with_id: usize,
    pub with_ammo: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquipmentCreationStream {
    pub records: Vec<EquipmentCreation>,
    pub stats: EquipmentCreationStats,
}
pub(super) fn equipment_new_header(
    pay: &[u8],
    p: usize,
    band: &BTreeSet<u32>,
) -> Option<(u32, u32)> {
    world_object_new_header(pay, p, band, 37)
}
pub(super) fn world_object_new_header(
    pay: &[u8],
    p: usize,
    band: &BTreeSet<u32>,
    ti: u32,
) -> Option<(u32, u32)> {
    let b = Bits(pay);
    if b.read(p, 3) != Some(1) || b.read(p + 18, 6) != Some(ti as u64) {
        return None;
    }
    let slot = b.read(p + 3, 13)? as u32;
    band.contains(&slot)
        .then_some((slot, b.read(p + 16, 2)? as u32))
}
pub(super) fn read_equipment_creation_with_position(
    pay: &[u8],
    p: usize,
    components: usize,
    widths: [usize; 2],
    map: &FilmMapBounds,
    stats: &mut EquipmentCreationStats,
    position: &PositionEncoding,
) -> Option<EquipmentCreation> {
    CreationWalk {
        vehicle_layout: None,
        component_policy: None,
        position: Some(position),
        ti: 37,
        components,
        widths,
        map,
        ammo_arch: None,
        corruption_check: false,
    }
    .read(pay, p, stats)
}
struct CreationWalk<'a> {
    vehicle_layout: Option<&'a super::I0Layout>,
    ti: u32,
    components: usize,
    widths: [usize; 2],
    map: &'a FilmMapBounds,
    ammo_arch: Option<&'a super::FilmArchetype>,
    corruption_check: bool,
    position: Option<&'a PositionEncoding>,
    component_policy: Option<(&'a super::ComponentWidthOverrides, bool)>,
}
/// Native vehicle creation i0 gate: absolute spine, expected region and no
/// saturated axis. Its width excludes the position tail, unlike world objects.
fn decode_vehicle_creation_position(
    pay: &[u8],
    at: usize,
    map: &FilmMapBounds,
) -> Option<[f32; 3]> {
    let layout = super::I0Layout {
        gate_bits: (4 + map.region_index_bits.max(1)) as i64,
        axis_widths: map.axis_widths.map(|w| w as u64),
        region: map.region,
    };
    decode_vehicle_i0_position(pay, at as i64, &layout, &map.coordinate_bounds())
}

/// Native dynamic i0 admission gate with an explicit layout, independent of map
/// catalog normalization. Requires the absolute spine, expected region and
/// nonsaturated axis quanta. The position tail is not part of this read.
/// Wider fields retain the native reader's low 32-bit value and saturation rule.
pub fn decode_vehicle_i0_position(
    pay: &[u8],
    at: i64,
    layout: &super::I0Layout,
    bounds: &super::CoordinateBounds,
) -> Option<[f32; 3]> {
    let at = usize::try_from(at).ok()?;
    let gate = usize::try_from(layout.gate_bits).ok()?;
    let region_bits = gate.checked_sub(4)?;
    let [x, y, z] = layout.axis_widths;
    let widths = [
        usize::try_from(x).ok()?,
        usize::try_from(y).ok()?,
        usize::try_from(z).ok()?,
    ];
    if widths.iter().any(|&w| w >= 64) {
        return None;
    }
    let total = widths.iter().try_fold(gate, |n, w| n.checked_add(*w))?;
    let bits = Bits(pay);
    if at.checked_add(total)? > bits.len()
        || bits.read(at, 4)? != 0
        || bits.read(at.checked_add(4)?, region_bits)? != u64::from(layout.region)
    {
        return None;
    }
    let mut cursor = at.checked_add(gate)?;
    let mut out = [0.; 3];
    for (i, v) in out.iter_mut().enumerate() {
        let width = widths[i];
        let q = bits.read(cursor, width)? as u32;
        let steps = 1_u64.checked_shl(width as u32)?;
        let saturated = 1_u32.checked_shl(width as u32).unwrap_or(0).wrapping_sub(1);
        if q == 0 || q == saturated {
            return None;
        }
        let step = (f64::from(bounds.max[i]) - f64::from(bounds.min[i])) / steps as f64;
        *v = (f64::from(bounds.min[i]) + step * (q as f64 + 0.5)) as f32;
        cursor += width;
    }
    Some(out)
}

/// Vehicle NEW records use the shared creation walk with ti=40 default state,
/// explicit map precision and the stricter dynamic-position admission gate.
pub fn decode_vehicle_creations(
    pay: &[u8],
    band: &BTreeSet<u32>,
    components: usize,
    widths: [usize; 2],
    map: &FilmMapBounds,
) -> EquipmentCreationStream {
    decode_vehicle_creations_with_position(
        pay,
        band,
        components,
        widths,
        map,
        &map.position_encoding(),
    )
}
pub fn decode_vehicle_creations_with_position(
    pay: &[u8],
    band: &BTreeSet<u32>,
    components: usize,
    widths: [usize; 2],
    map: &FilmMapBounds,
    position: &PositionEncoding,
) -> EquipmentCreationStream {
    CreationWalk {
        vehicle_layout: None,
        component_policy: None,
        position: Some(position),
        ti: 40,
        components,
        widths,
        map,
        ammo_arch: None,
        corruption_check: false,
    }
    .scan(pay, band)
}
/// Vehicle creation scan with the resolved dynamic i0 layout kept separate
/// from default-state position precision and coordinate bounds.
pub fn decode_vehicle_creations_with_layout(
    pay: &[u8],
    band: &BTreeSet<u32>,
    components: usize,
    widths: [usize; 2],
    map: &FilmMapBounds,
    position: &PositionEncoding,
    layout: &super::I0Layout,
) -> EquipmentCreationStream {
    CreationWalk {
        vehicle_layout: Some(layout),
        component_policy: None,
        position: Some(position),
        ti: 40,
        components,
        widths,
        map,
        ammo_arch: None,
        corruption_check: false,
    }
    .scan(pay, band)
}
/// Resolved native context inputs. Layout and registry failures remain deferred
/// until their native setup step, preserving error precedence and partial stats.
#[derive(Debug)]
pub struct VehicleCreationScanConfig<'a> {
    pub map: Option<&'a FilmMapBounds>,
    pub layout: Result<&'a super::I0Layout, super::DecodeError>,
    pub registry: Result<&'a super::FilmRegistry, super::DecodeError>,
    pub position: &'a PositionEncoding,
    pub mpp_widths: [usize; 2],
}

#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct VehicleCreationScanFailure {
    pub scan: Box<EquipmentCreationStream>,
    #[source]
    pub error: super::DecodeError,
}
fn vehicle_creation_failure(slots: usize, error: super::DecodeError) -> VehicleCreationScanFailure {
    VehicleCreationScanFailure {
        scan: Box::new(EquipmentCreationStream {
            records: Vec::new(),
            stats: EquipmentCreationStats {
                slots,
                ..Default::default()
            },
        }),
        error,
    }
}

/// Native offline wrapper: detect i0 from the recording and install successful
/// measurements on a copy of the inherited component precision. Bounds remain
/// external; a failed detection never replaces inherited widths with zeroes.
pub fn scan_vehicle_creations_with_detected_layout(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: Option<&FilmMapBounds>,
    registry: Result<&super::FilmRegistry, super::DecodeError>,
    mpp_widths: [usize; 2],
    inherited_position: &PositionEncoding,
) -> Result<EquipmentCreationStream, VehicleCreationScanFailure> {
    let detection = super::detect_film_i0_layout(chunks);
    let mut position = inherited_position.clone();
    let layout = if detection.refusal.is_none() {
        detection
            .layout
            .as_ref()
            .ok_or(super::DecodeError::Missing("vehicle i0 layout"))
    } else {
        Err(super::DecodeError::Inconsistent(format!(
            "vehicle i0 layout: {:?}",
            detection.refusal
        )))
    };
    if let Ok(layout) = layout.as_ref() {
        let axes = layout.axis_widths.map(|w| w as usize);
        if axes.iter().all(|&w| w != 0) {
            position.world_axis_bits = Some(axes);
            if layout.gate_bits > 4 {
                position.index_bits = (layout.gate_bits - 4) as usize;
            }
            position.region_axis_bits.clear();
            position.region_axis_bits.insert(layout.region, axes);
        }
    }
    scan_vehicle_creations(
        chunks,
        VehicleCreationScanConfig {
            map,
            layout,
            registry,
            position: &position,
            mpp_widths,
        },
    )
}

/// Native top-level vehicle scan: data-prefix and census guards precede bounds.
pub fn scan_vehicle_creations(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    config: VehicleCreationScanConfig<'_>,
) -> Result<EquipmentCreationStream, VehicleCreationScanFailure> {
    if super::fire_events::native_chunk_prefix(chunks).is_err() {
        return Err(vehicle_creation_failure(
            0,
            super::DecodeError::Missing("film data chunks"),
        ));
    }
    let band = super::scan_world_object_keyframes(chunks, 40).band;
    if band.is_empty() {
        return Err(vehicle_creation_failure(
            0,
            super::DecodeError::Missing("vehicle slot band"),
        ));
    }
    scan_vehicle_creations_for_band_report(chunks, &band, config)
}

/// Native ForBand scan, retaining Slots when layout or registry setup fails.
/// An explicitly empty band is valid; unlike the top-level scan it has no census guard.
pub fn scan_vehicle_creations_for_band_report(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    band: &BTreeSet<u32>,
    config: VehicleCreationScanConfig<'_>,
) -> Result<EquipmentCreationStream, VehicleCreationScanFailure> {
    let map = config.map.ok_or_else(|| {
        vehicle_creation_failure(0, super::DecodeError::Missing("vehicle coordinate bounds"))
    })?;
    if super::fire_events::native_chunk_prefix(chunks).is_err() {
        return Err(vehicle_creation_failure(
            0,
            super::DecodeError::Missing("film data chunks"),
        ));
    }
    let fail = |error| vehicle_creation_failure(band.len(), error);
    let layout = config.layout.map_err(fail)?;
    let registry = config.registry.map_err(fail)?;
    let arch = registry
        .archetype(40)
        .ok_or_else(|| fail(super::DecodeError::Missing("vehicle archetype")))?;
    CreationWalk {
        vehicle_layout: Some(layout),
        component_policy: None,
        position: Some(config.position),
        ti: 40,
        components: arch.components.len(),
        widths: config.mpp_widths,
        map,
        ammo_arch: None,
        corruption_check: false,
    }
    .scan_chunks(chunks, band)
    .map_err(fail)
}

pub fn scan_vehicle_creations_for_band(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &BTreeSet<u32>,
    registry: &super::FilmRegistry,
    widths: [usize; 2],
) -> Result<EquipmentCreationStream, super::DecodeError> {
    scan_vehicle_creations_for_band_with_position(
        chunks,
        map,
        band,
        registry,
        widths,
        &map.position_encoding(),
    )
}
pub fn scan_vehicle_creations_for_band_with_position(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &BTreeSet<u32>,
    registry: &super::FilmRegistry,
    widths: [usize; 2],
    position: &PositionEncoding,
) -> Result<EquipmentCreationStream, super::DecodeError> {
    let layout = super::I0Layout {
        gate_bits: (4 + map.region_index_bits.max(1)) as i64,
        axis_widths: map.axis_widths.map(|w| w as u64),
        region: map.region,
    };
    // Compatibility result; callers needing partial counters use the report API.
    scan_vehicle_creations_for_band_report(
        chunks,
        band,
        VehicleCreationScanConfig {
            map: Some(map),
            layout: Ok(&layout),
            registry: Ok(registry),
            position,
            mpp_widths: widths,
        },
    )
    .map_err(|failure| failure.error)
}
impl CreationWalk<'_> {
    fn read(
        &self,
        pay: &[u8],
        p: usize,
        stats: &mut EquipmentCreationStats,
    ) -> Option<EquipmentCreation> {
        let Self {
            ti,
            components,
            widths,
            map,
            ..
        } = *self;
        let start = p + 24;
        let map_encoding = map.position_encoding();
        let encoding = self.position.unwrap_or(&map_encoding);
        let ComponentDecode::Decoded(default) =
            decode_default_state(pay, start, ti, widths, (ti == 40).then_some(encoding))
        else {
            stats.overflow += 1;
            return None;
        };
        let mut r = Cursor::new_padded(pay, usize::try_from(default.end_bit).ok()?);
        r.read(1)?;
        let full = r.bit()?;
        let mut idx = Vec::new();
        let mut valid = true;
        if full {
            let mask = r.read(64)?;
            for i in 0..64 {
                if mask & (1 << i) != 0 {
                    if i >= components {
                        valid = false;
                        break;
                    }
                    idx.push(i);
                }
            }
        } else {
            let count = r.read(3)? as usize;
            if count == 0 {
                valid = false;
            }
            for _ in 0..count {
                let i = r.read(6)? as usize;
                if i >= components || idx.last().is_some_and(|prev| *prev >= i) {
                    valid = false;
                    break;
                }
                idx.push(i);
            }
        }
        if !valid || idx.is_empty() || r.position > pay.len() * 8 {
            stats.mask_bad += 1;
            return None;
        }
        let precision = self
            .position
            .and_then(WorldObjectPrecision::from_position)
            .unwrap_or_else(|| WorldObjectPrecision::from_map(map));
        let position = if ti == 40 {
            self.vehicle_layout
                .map_or_else(
                    || decode_vehicle_creation_position(pay, r.position, map),
                    |layout| {
                        decode_vehicle_i0_position(
                            pay,
                            r.position as i64,
                            layout,
                            &map.coordinate_bounds(),
                        )
                    },
                )
                .map(|v| (v, 0))
        } else {
            decode_world_object_position_with_precision(pay, r.position, map, precision)
        };
        let Some((xyz, _)) = position else {
            stats.pos_bad += 1;
            return None;
        };
        let field = |name: &str| {
            default
                .fields
                .iter()
                .find(|f| f.name == name)
                .map(|f| f.raw)
        };
        let names = [
            "mpp.lead",
            "mpp.object_tag",
            "mpp.variant_name",
            "mpp.tail.name",
        ];
        let ammo = self.ammo_arch.and_then(|arch| {
            super::components::read_ground_weapon_ammo(
                pay,
                r.position,
                &idx,
                arch,
                encoding,
                self.corruption_check,
                self.component_policy,
            )
        });
        Some(EquipmentCreation {
            slot: 0,
            generation: 0,
            chunk: 0,
            packet_index: 0,
            timestamp_us: 0,
            bit_pos: p,
            has_ref: field("player_index").is_some(),
            reference: field("player_index").unwrap_or(0) as u32,
            has_id: field("ability_enabled_id").is_some(),
            ability_id: field("ability_enabled_id").unwrap_or(0) as u32,
            mpp_present: std::array::from_fn(|i| field(names[i]).is_some()),
            mpp_val: std::array::from_fn(|i| field(names[i]).unwrap_or(0)),
            x: xyz[0],
            y: xyz[1],
            z: xyz[2],
            mask_has_i0: idx[0] == 0,
            mask: idx,
            mask_full: full,
            default_state_bits: usize::try_from(default.end_bit).ok()?.checked_sub(start)?,
            has_ammo: ammo.is_some(),
            ammo: ammo.unwrap_or_default(),
            after_bit: r.position
                + if ti == 40 {
                    match self.vehicle_layout {
                        Some(layout) => usize::try_from(layout.total_bits()).ok()?,
                        None => WorldObjectPrecision::from_map(map).bit_length()?,
                    }
                } else {
                    precision.bit_length()?
                },
        })
    }
}
/// Bounded header scan, retaining the reference's permissive position tail reads.
/// Creation observations remain unconfirmed until compared with mobile lifetimes.
pub fn decode_equipment_creations(
    pay: &[u8],
    band: &BTreeSet<u32>,
    components: usize,
    widths: [usize; 2],
    map: &FilmMapBounds,
) -> EquipmentCreationStream {
    decode_equipment_creations_with_position(
        pay,
        band,
        components,
        widths,
        map,
        &map.position_encoding(),
    )
}

pub fn decode_equipment_creations_with_position(
    pay: &[u8],
    band: &BTreeSet<u32>,
    components: usize,
    widths: [usize; 2],
    map: &FilmMapBounds,
    position: &PositionEncoding,
) -> EquipmentCreationStream {
    CreationWalk {
        vehicle_layout: None,
        component_policy: None,
        position: Some(position),
        ti: 37,
        components,
        widths,
        map,
        ammo_arch: None,
        corruption_check: false,
    }
    .scan(pay, band)
}
/// Independent power-up pad scan. Unlike placement confirmation, this still runs
/// with the inherited widths when format lookup and calibration are inconclusive.
pub fn scan_equipment_creations_for_band(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &BTreeSet<u32>,
    registry: &super::FilmRegistry,
    widths: [usize; 2],
) -> Result<EquipmentCreationStream, super::DecodeError> {
    scan_equipment_creations_for_band_with_position(
        chunks,
        map,
        band,
        registry,
        widths,
        &map.position_encoding(),
    )
}

pub fn scan_equipment_creations_for_band_with_position(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &BTreeSet<u32>,
    registry: &super::FilmRegistry,
    widths: [usize; 2],
    position: &PositionEncoding,
) -> Result<EquipmentCreationStream, super::DecodeError> {
    let arch = registry
        .archetype(37)
        .ok_or(super::DecodeError::Missing("equipment archetype"))?;
    CreationWalk {
        vehicle_layout: None,
        component_policy: None,
        position: Some(position),
        ti: 37,
        components: arch.components.len(),
        widths,
        map,
        ammo_arch: None,
        corruption_check: false,
    }
    .scan_chunks(chunks, band)
}

/// Ground-weapon NEW records reuse the equipment walk with archetype 42 defaults.
/// Ammunition is read through the registry component grammar, not fixed offsets.
pub fn decode_ground_weapon_creations(
    pay: &[u8],
    band: &BTreeSet<u32>,
    arch: &super::FilmArchetype,
    widths: [usize; 2],
    map: &FilmMapBounds,
    corruption_check: bool,
) -> EquipmentCreationStream {
    decode_ground_weapon_creations_with_position(
        pay,
        band,
        arch,
        widths,
        map,
        corruption_check,
        &map.position_encoding(),
    )
}
pub fn decode_ground_weapon_creations_with_position(
    pay: &[u8],
    band: &BTreeSet<u32>,
    arch: &super::FilmArchetype,
    widths: [usize; 2],
    map: &FilmMapBounds,
    corruption_check: bool,
    position: &PositionEncoding,
) -> EquipmentCreationStream {
    CreationWalk {
        vehicle_layout: None,
        component_policy: None,
        position: Some(position),
        ti: 42,
        components: arch.components.len(),
        widths,
        map,
        ammo_arch: Some(arch),
        corruption_check,
    }
    .scan(pay, band)
}
/// Ground-weapon creation scan with the complete component traversal policy.
/// MPP widths and position precision come from the same inherited film profile.
pub fn decode_ground_weapon_creations_with_encoding(
    pay: &[u8],
    band: &BTreeSet<u32>,
    arch: &super::FilmArchetype,
    map: &FilmMapBounds,
    encoding: &super::FrameEncoding,
) -> EquipmentCreationStream {
    CreationWalk {
        vehicle_layout: None,
        component_policy: Some((
            &encoding.component_widths,
            encoding.keyframe_simulation_complete.unwrap_or(true),
        )),
        position: encoding.position.as_ref(),
        ti: 42,
        components: arch.components.len(),
        widths: encoding.mpp_widths,
        map,
        ammo_arch: Some(arch),
        corruption_check: encoding.corruption_check,
    }
    .scan(pay, band)
}

/// LegacyFilm-wide counterpart of `decode_ground_weapon_creations_with_encoding`.
pub fn scan_ground_weapon_creations_for_band_with_encoding(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &BTreeSet<u32>,
    registry: &super::FilmRegistry,
    encoding: &super::FrameEncoding,
) -> Result<EquipmentCreationStream, super::DecodeError> {
    let arch = registry
        .archetype(42)
        .ok_or(super::DecodeError::Missing("ground weapon archetype"))?;
    CreationWalk {
        vehicle_layout: None,
        component_policy: Some((
            &encoding.component_widths,
            encoding.keyframe_simulation_complete.unwrap_or(true),
        )),
        position: encoding.position.as_ref(),
        ti: 42,
        components: arch.components.len(),
        widths: encoding.mpp_widths,
        map,
        ammo_arch: Some(arch),
        corruption_check: encoding.corruption_check,
    }
    .scan_chunks(chunks, band)
}

impl CreationWalk<'_> {
    fn scan(&self, pay: &[u8], band: &BTreeSet<u32>) -> EquipmentCreationStream {
        let mut out = EquipmentCreationStream {
            records: Vec::new(),
            stats: EquipmentCreationStats {
                slots: band.len(),
                ..Default::default()
            },
        };
        let Some(limit) = pay.len().saturating_mul(8).checked_sub(24) else {
            return out;
        };
        let mut p = 0;
        while p <= limit {
            let at = p;
            p += 1;
            let Some((slot, generation)) = world_object_new_header(pay, at, band, self.ti) else {
                continue;
            };
            out.stats.anchors += 1;
            let Some(mut e) = self.read(pay, at, &mut out.stats) else {
                continue;
            };
            e.slot = slot;
            e.generation = generation;
            out.stats.accepted += 1;
            if e.mask_full {
                out.stats.mask_full += 1;
            } else {
                out.stats.mask_sparse += 1;
            }
            out.stats.no_i0 += usize::from(!e.mask_has_i0);
            out.stats.with_ref += usize::from(e.has_ref);
            out.stats.with_id += usize::from(e.has_id);
            out.stats.with_ammo += usize::from(e.has_ammo);
            p = e.after_bit;
            out.records.push(e);
        }
        out
    }
}
/// Scan native metadata-order chunks using caller-resolved MPP widths and the
/// film's corruption-check flag. This exports raw creations, not confirmed drops.
pub fn scan_ground_weapon_creations_for_band(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &BTreeSet<u32>,
    registry: &super::FilmRegistry,
    widths: [usize; 2],
    corruption_check: bool,
) -> Result<EquipmentCreationStream, super::DecodeError> {
    scan_ground_weapon_creations_for_band_with_position(
        chunks,
        map,
        band,
        registry,
        widths,
        corruption_check,
        &map.position_encoding(),
    )
}
pub fn scan_ground_weapon_creations_for_band_with_position(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    band: &BTreeSet<u32>,
    registry: &super::FilmRegistry,
    widths: [usize; 2],
    corruption_check: bool,
    position: &PositionEncoding,
) -> Result<EquipmentCreationStream, super::DecodeError> {
    let arch = registry
        .archetype(42)
        .ok_or(super::DecodeError::Missing("ground weapon archetype"))?;
    let walk = CreationWalk {
        vehicle_layout: None,
        component_policy: None,
        position: Some(position),
        ti: 42,
        components: arch.components.len(),
        widths,
        map,
        ammo_arch: Some(arch),
        corruption_check,
    };
    walk.scan_chunks(chunks, band)
}
impl CreationWalk<'_> {
    fn scan_chunks(
        &self,
        chunks: &[crate::clients::hi::models::FilmChunkData],
        band: &BTreeSet<u32>,
    ) -> Result<EquipmentCreationStream, super::DecodeError> {
        let mut out = EquipmentCreationStream {
            records: Vec::new(),
            stats: EquipmentCreationStats {
                slots: band.len(),
                ..Default::default()
            },
        };
        for chunk in super::fire_events::native_chunk_prefix(chunks)? {
            for (index, p) in super::fire_events::native_chunk_packets(chunk)
                .into_iter()
                .enumerate()
            {
                if p.packet_type != 0 {
                    continue;
                }
                let mut stream = self.scan(
                    &chunk.data[p.payload_offset..p.payload_offset + p.payload_size],
                    band,
                );
                for c in &mut stream.records {
                    c.chunk = i64::from(chunk.metadata.index);
                    c.packet_index = index;
                    c.timestamp_us = p.timestamp_us;
                }
                let a = &mut out.stats;
                let b = stream.stats;
                a.anchors += b.anchors;
                a.overflow += b.overflow;
                a.mask_bad += b.mask_bad;
                a.pos_bad += b.pos_bad;
                a.accepted += b.accepted;
                a.mask_sparse += b.mask_sparse;
                a.mask_full += b.mask_full;
                a.no_i0 += b.no_i0;
                a.with_ref += b.with_ref;
                a.with_id += b.with_id;
                a.with_ammo += b.with_ammo;
                out.records.extend(stream.records);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct VehicleCase {
        #[serde(default)]
        layout: Option<super::super::I0Layout>,
        #[serde(default)]
        profile_axes: Option<[usize; 3]>,
        map: FilmMapBounds,
        widths: [usize; 2],
        hex: String,
        records: Vec<EquipmentCreation>,
        stats: EquipmentCreationStats,
    }
    #[test]
    fn native_creation_world_profile() {
        #[derive(Deserialize)]
        struct Case {
            map: FilmMapBounds,
            ti: u32,
            index: usize,
            region: u32,
            axes: [usize; 3],
            widths: [usize; 2],
            hex: String,
            records: Vec<EquipmentCreation>,
            stats: EquipmentCreationStats,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/creation-world-profile-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cases.len(), 512);
        let (mut total, mut zero, mut wide) = (0, 0, 0);
        for (i, c) in cases.into_iter().enumerate() {
            let pay: Vec<u8> = c
                .hex
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
                .collect();
            let mut position = c.map.position_encoding();
            position.index_bits = c.index;
            position.world_axis_bits = Some(c.axes);
            position.region_axis_bits = [(c.region, c.axes)].into();
            let band = [512].into();
            let result = if c.ti == 37 {
                decode_equipment_creations_with_position(
                    &pay, &band, 1, c.widths, &c.map, &position,
                )
            } else {
                let arch = crate::theater::FilmArchetype {
                    index: 42,
                    components: vec!["object-position-component".into()],
                    levels: vec![0],
                };
                decode_ground_weapon_creations_with_position(
                    &pay, &band, &arch, c.widths, &c.map, false, &position,
                )
            };
            assert_eq!(result.records, c.records, "records {i}");
            assert_eq!(result.stats, c.stats, "stats {i}");
            assert_eq!(
                serde_json::from_value::<EquipmentCreationStream>(
                    serde_json::to_value(&result).unwrap()
                )
                .unwrap(),
                result
            );
            total += result.records.len();
            zero += if c.index == 0 {
                result.records.len()
            } else {
                0
            };
            wide += if c.axes.iter().any(|&w| w > 30) {
                result.records.len()
            } else {
                0
            };
        }
        assert_eq!((total, zero, wide), (452, 117, 92));
    }

    #[test]
    #[ignore = "requires five captured v41 films, including the hour-long raid"]
    fn local_vehicle_creation_corpus() {
        use super::super::{DecodeError, I0Layout};
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/vehicle-creation-corpus-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 9);
        let mut total = 0;
        for row in rows {
            let folder = row["folder"].as_str().unwrap();
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
            let meta: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join("film.json")).unwrap()).unwrap();
            assert_eq!(meta["film_major_version"], 41);
            let chunks: Vec<_> = meta["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| FilmChunkData {
                    metadata: FilmChunk {
                        index: c["index"].as_i64().unwrap() as i32,
                        chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                        start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                        duration_ms: 0,
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: std::fs::read(root.join(c["file"].as_str().unwrap())).unwrap(),
                })
                .collect();
            let registry = super::super::parse_registry(
                &chunks.iter().find(|c| c.metadata.index == 0).unwrap().data,
            )
            .unwrap();
            assert_eq!(
                registry.archetypes.len(),
                row["archetypes"].as_u64().unwrap() as usize
            );
            let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
            let automatic = row["automatic"].as_bool().unwrap();
            let result = if automatic {
                let detection = super::super::detect_film_i0_layout(&chunks);
                assert_eq!(
                    detection.refusal.is_some(),
                    row["layout_error"].as_bool().unwrap(),
                    "layout status {folder}"
                );
                if let Some(layout) = detection.layout {
                    assert_eq!(
                        layout,
                        serde_json::from_value(row["layout"].clone()).unwrap(),
                        "layout {folder}"
                    );
                }
                let inherited =
                    super::super::film_map_catalog().maps["cliffhanger"].position_encoding();
                scan_vehicle_creations_with_detected_layout(
                    &chunks,
                    Some(&map),
                    Ok(&registry),
                    super::super::NATIVE_MPP_DEFAULT_WIDTHS,
                    &inherited,
                )
            } else {
                let layout = I0Layout {
                    gate_bits: (4 + map.region_index_bits.max(1)) as i64,
                    axis_widths: map.axis_widths.map(|w| w as u64),
                    region: map.region,
                };
                assert_eq!(
                    layout,
                    serde_json::from_value(row["layout"].clone()).unwrap()
                );
                let position = map.position_encoding();
                scan_vehicle_creations(
                    &chunks,
                    VehicleCreationScanConfig {
                        map: Some(&map),
                        layout: Ok(&layout),
                        registry: Ok(&registry),
                        position: &position,
                        mpp_widths: super::super::NATIVE_MPP_DEFAULT_WIDTHS,
                    },
                )
            };
            assert_eq!(
                result.is_err(),
                row["error"].as_bool().unwrap(),
                "error {folder} auto={automatic}"
            );
            let out = match result {
                Ok(out) => out,
                Err(failure) => {
                    assert!(
                        matches!(failure.error, DecodeError::Missing("vehicle slot band")),
                        "unexpected error {folder}: {}",
                        failure.error
                    );
                    *failure.scan
                }
            };
            assert_eq!(
                out.records,
                serde_json::from_value::<Vec<EquipmentCreation>>(row["records"].clone()).unwrap(),
                "records {folder} auto={automatic}"
            );
            assert_eq!(
                out.stats,
                serde_json::from_value(row["stats"].clone()).unwrap(),
                "stats {folder} auto={automatic}"
            );
            total += out.records.len();
            eprintln!(
                "vehicle creations {folder} auto={automatic}: {} records",
                out.records.len()
            );
        }
        assert_eq!(
            total, 150,
            "captured test must include positive creation records"
        );
    }
    #[test]
    fn native_loaded_vehicle_creation_scan() {
        use super::super::{DecodeError, FilmArchetype, FilmRegistry, I0Layout};
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/vehicle-creation-loaded-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 256);
        for (i, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let hex = c["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|k| u8::from_str_radix(&hex[k..k + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 2,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
            let layout: I0Layout = serde_json::from_value(row["layout"].clone()).unwrap();
            let position = map.position_encoding();
            let registry = FilmRegistry {
                archetypes: (0..if row["arch_ok"].as_bool().unwrap() {
                    41
                } else {
                    40
                })
                    .map(|index| FilmArchetype {
                        index,
                        components: if index == 40 {
                            vec![String::new(); 33]
                        } else {
                            vec![]
                        },
                        levels: if index == 40 { vec![0; 33] } else { vec![] },
                    })
                    .collect(),
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            if i == 9 {
                let facts = super::super::scan_film_vehicle_facts_with_position(
                    &chunks,
                    &registry,
                    &map,
                    serde_json::from_value(row["widths"].clone()).unwrap(),
                    &position,
                );
                assert!(!facts.scanned);
                assert_eq!(facts.issues.len(), 1);
                assert_eq!(facts.creations.unwrap().stats.slots, 4);
            }
            let config = VehicleCreationScanConfig {
                map: row["bounds_ok"].as_bool().unwrap().then_some(&map),
                layout: if row["layout_ok"].as_bool().unwrap() {
                    Ok(&layout)
                } else {
                    Err(DecodeError::Missing("layout oracle refusal"))
                },
                registry: if row["registry_ok"].as_bool().unwrap() {
                    Ok(&registry)
                } else {
                    Err(DecodeError::Missing("registry oracle refusal"))
                },
                position: &position,
                mpp_widths: serde_json::from_value(row["widths"].clone()).unwrap(),
            };
            let band: BTreeSet<u32> = serde_json::from_value(row["band"].clone()).unwrap();
            let result = if row["top"].as_bool().unwrap() {
                scan_vehicle_creations(&chunks, config)
            } else {
                scan_vehicle_creations_for_band_report(&chunks, &band, config)
            };
            let (out, error) = match result {
                Ok(out) => (out, ""),
                Err(failure) => {
                    let error = match failure.error {
                        DecodeError::Missing("film data chunks") => "chunks",
                        DecodeError::Missing("vehicle coordinate bounds") => "bounds",
                        DecodeError::Missing("vehicle slot band") => "band",
                        DecodeError::Missing("layout oracle refusal") => "layout",
                        DecodeError::Missing("registry oracle refusal") => "registry",
                        DecodeError::Missing("vehicle archetype") => "archetype",
                        e => panic!("case {i}: {e}"),
                    };
                    (*failure.scan, error)
                }
            };
            assert_eq!(error, row["error"].as_str().unwrap(), "error {i}");
            assert_eq!(
                out.records,
                serde_json::from_value::<Vec<EquipmentCreation>>(row["records"].clone()).unwrap(),
                "records {i}"
            );
            assert_eq!(
                serde_json::to_value(&out.stats).unwrap(),
                row["stats"],
                "stats {i}"
            );
            assert_eq!(
                serde_json::from_value::<EquipmentCreationStream>(
                    serde_json::to_value(&out).unwrap()
                )
                .unwrap(),
                out
            );
        }
    }
    #[test]
    fn native_vehicle_i0_layouts() {
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            at: i64,
            layout: super::super::I0Layout,
            point: Option<[f32; 3]>,
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/vehicle-i0-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 2560);
        let bounds = super::super::CoordinateBounds {
            min: [-123.5, 0.125, -109.75],
            max: [908.25, 2777.75, 141.5],
        };
        let mut accepted = 0;
        let mut no_region = 0;
        for (i, c) in cases.into_iter().enumerate() {
            let pay: Vec<_> = (0..c.hex.len())
                .step_by(2)
                .map(|p| u8::from_str_radix(&c.hex[p..p + 2], 16).unwrap())
                .collect();
            let actual = decode_vehicle_i0_position(&pay, c.at, &c.layout, &bounds);
            assert_eq!(actual, c.point, "case {i}");
            accepted += usize::from(actual.is_some());
            no_region += usize::from(actual.is_some() && c.layout.gate_bits == 4);
        }
        assert_eq!(accepted, 538);
        assert_eq!(no_region, 105);
    }
    #[test]
    fn native_vehicle_creations() {
        compare_vehicle_creations(
            include_bytes!("fixtures/vehicle-creations-v41.json.zlib"),
            false,
        );
    }
    #[test]
    fn native_vehicle_creation_layout() {
        compare_vehicle_creations(
            include_bytes!("fixtures/vehicle-creation-layout-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_vehicle_creation_profile() {
        compare_vehicle_creations(
            include_bytes!("fixtures/vehicle-creation-profile-v41.json.zlib"),
            true,
        );
    }
    fn compare_vehicle_creations(fixture: &[u8], nondefault: bool) {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_end(&mut raw)
            .unwrap();
        let mut differs = 0;
        let cases: Vec<VehicleCase> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<_> = (0..c.hex.len())
                .step_by(2)
                .map(|p| u8::from_str_radix(&c.hex[p..p + 2], 16).unwrap())
                .collect();
            let baseline =
                decode_vehicle_creations(&data, &[512, 514, 515].into(), 33, c.widths, &c.map);
            let mut position = c.map.position_encoding();
            if let Some(axes) = c.profile_axes {
                position.world_axis_bits = Some(axes);
                position.region_axis_bits.insert(c.map.region, axes);
            }
            let out = if let Some(layout) = &c.layout {
                decode_vehicle_creations_with_layout(
                    &data,
                    &[512, 514, 515].into(),
                    33,
                    c.widths,
                    &c.map,
                    &position,
                    layout,
                )
            } else {
                decode_vehicle_creations_with_position(
                    &data,
                    &[512, 514, 515].into(),
                    33,
                    c.widths,
                    &c.map,
                    &position,
                )
            };
            differs += usize::from(out != baseline);
            assert_eq!(out.stats, c.stats, "stats {i}");
            assert_eq!(out.records, c.records, "records {i}");
        }
        if nondefault {
            assert!(differs > 100, "profile must affect observed decoding");
        }
    }
    #[test]
    fn native_ground_weapon_creations() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/ground-creations-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let films: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for film in films.as_array().unwrap() {
            let a = &film["arch"];
            let arch = super::super::FilmArchetype {
                index: 42,
                components: serde_json::from_value(a["Components"].clone()).unwrap(),
                levels: serde_json::from_value(a["Levels"].clone()).unwrap(),
            };
            let map: FilmMapBounds = serde_json::from_value(film["map"].clone()).unwrap();
            let band: BTreeSet<u32> = serde_json::from_value(film["band"].clone()).unwrap();
            for (i, row) in film["cases"].as_array().unwrap().iter().enumerate() {
                let hex = row["hex"].as_str().unwrap();
                let pay: Vec<_> = (0..hex.len())
                    .step_by(2)
                    .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                    .collect();
                let actual = decode_ground_weapon_creations(
                    &pay,
                    &band,
                    &arch,
                    [9, 5],
                    &map,
                    row["corruption"].as_bool().unwrap(),
                );
                let records: Vec<EquipmentCreation> =
                    serde_json::from_value(row["records"].clone()).unwrap();
                let stats: EquipmentCreationStats =
                    serde_json::from_value(row["stats"].clone()).unwrap();
                assert_eq!(actual.stats, stats, "{} case {i} stats", film["folder"]);
                assert_eq!(
                    actual.records, records,
                    "{} case {i} records",
                    film["folder"]
                );
            }
        }
    }
    #[test]
    fn native_equipment_creation_records_and_diagnostics() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/equipment-creations-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let map = FilmMapBounds {
            module: String::new(),
            min: [-100., -200., -300.],
            max: [300., 400., 500.],
            axis_widths: [16, 17, 18],
            region: 0,
            region_index_bits: 1,
        };
        let mut accepted = 0;
        for (i, row) in rows.as_array().unwrap().iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let pay: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                .collect();
            let actual =
                decode_equipment_creations(&pay, &[512, 513, 515].into(), 31, [9, 5], &map);
            let expected: Vec<EquipmentCreation> =
                serde_json::from_value(row["records"].clone()).unwrap();
            let stats: EquipmentCreationStats =
                serde_json::from_value(row["stats"].clone()).unwrap();
            assert_eq!(actual.stats, stats, "stats {i}");
            assert_eq!(actual.records, expected, "records {i}");
            accepted += actual.records.len();
        }
        assert!(accepted > 1000);
    }
}
