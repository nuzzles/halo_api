//! Halo Infinite map identities and coordinate calibrations.
//!
//! The default glossary registers verified asset IDs for every developer map
//! with published reference analysis bounds, plus known Forge assets sharing one definition.
//! Additional verified assets or custom bounds can be registered with
//! [`MapGlossary::insert`]. Unknown IDs never default to Forge.
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

pub mod known_maps;
pub mod levels;

pub use known_maps::KnownMap;

/// Quantization region from a separately verified map asset or game module.
/// These are compression bounds, not the playable area or navigation mesh bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionBounds {
    /// Quantization region selected by the map.
    pub region: u32,
    /// Lower bounds in engine world coordinates.
    pub min: [f32; 3],
    /// Upper bounds in engine world coordinates.
    pub max: [f32; 3],
}

impl PositionBounds {
    /// Widths at object-position precision level 16. This follows the reference
    /// engine's single-precision extent/count arithmetic, including its 2^22 cap.
    /// Bounds must be finite and increasing on all three axes. A single quantization
    /// bin has width zero: its explicit code is zero and consumes no source bits.
    pub fn axis_bits(&self) -> Option<[usize; 3]> {
        self.axis_bits_at_precision(16)
    }

    pub(crate) fn axis_bits_at_precision(&self, level: u8) -> Option<[usize; 3]> {
        let mut result = [0; 3];
        let step = (1.0_f32 / 120.0) * 2.0_f32.powi(16 - i32::from(level));
        let quantum = step + step;
        for (axis, width) in result.iter_mut().enumerate() {
            let extent = self.max[axis] - self.min[axis];
            if !self.min[axis].is_finite()
                || !self.max[axis].is_finite()
                || !extent.is_finite()
                || extent <= 0.0
            {
                return None;
            }
            let bins = (extent / quantum).ceil().min((1 << 22) as f32) as u32;
            *width = if step < 1e-4 {
                26
            } else {
                (32 - bins.saturating_sub(1).leading_zeros()) as usize
            };
        }
        Some(result)
    }

    /// Midpoint conversion, admitted only when the decoded widths match these
    /// bounds. World axes/units remain those of the supplied map; no Monet transform.
    pub fn dequantize(&self, raw: [u32; 3], widths: [usize; 3]) -> Option<[f64; 3]> {
        self.dequantize_at_precision(raw, widths, 16)
    }

    /// Convert a quantized map position with widths verified at the given wire precision.
    pub(crate) fn dequantize_at_precision(
        &self,
        raw: [u32; 3],
        widths: [usize; 3],
        level: u8,
    ) -> Option<[f64; 3]> {
        if widths != self.axis_bits_at_precision(level)? {
            return None;
        }
        let mut result = [0.0; 3];
        for axis in 0..3 {
            let bins = 1_u64 << widths[axis];
            if u64::from(raw[axis]) >= bins {
                return None;
            }
            result[axis] = f64::from(self.min[axis])
                + (f64::from(raw[axis]) + 0.5)
                    * (f64::from(self.max[axis]) - f64::from(self.min[axis]))
                    / bins as f64;
        }
        Some(result)
    }
}

/// Shared coordinate calibration for a known map, independent of asset variants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapDefinition {
    /// Canonical developer map or shared Forge-canvas classification.
    pub map: KnownMap,
    /// Canonical map name; an individual map asset can have a different name.
    pub name: &'static str,
    /// Coordinate compression bounds.
    pub bounds: PositionBounds,
    /// Number of bits used to identify the quantization region.
    pub region_index_bits: usize,
}

/// Calibration for one map asset, independent of any particular match fixture.
#[derive(Debug, Clone, PartialEq)]
pub struct MapDetails {
    /// Human-readable map name, for diagnostics and export metadata.
    pub name: String,
    /// Coordinate compression bounds, not playable-area or camera bounds.
    pub bounds: PositionBounds,
    /// Number of bits used to identify a quantization region in the film.
    pub region_index_bits: usize,
    /// Verified absolute-position payload lengths for regions without recovered bounds.
    /// These preserve record boundaries only; they must not produce XYZ coordinates.
    pub opaque_position_region_bits: BTreeMap<u32, u64>,
}

/// A map asset cannot be resolved with a supported position calibration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MapError {
    /// No calibration was registered for this map asset.
    #[error(
        "Unknown map asset {0}; register verified bounds in MapGlossary before resolving replay"
    )]
    UnknownMap(Uuid),
    /// Coordinate bounds are nonfinite, empty, reversed or excessively wide.
    #[error("Invalid bounds for map asset {0}")]
    InvalidBounds(Uuid),
    /// The v41 replay resolver does not support this region calibration.
    #[error("Unsupported region calibration for map asset {0}")]
    UnsupportedRegion(Uuid),
}

/// Map-asset-ID lookup used by the contextual Theater replay resolver.
///
/// [`Default`] and [`Self::new`] include all built-in maps. Custom definitions
/// can be added or replace a built-in entry without changing other calibrations.
#[derive(Debug, Clone)]
pub struct MapGlossary {
    maps: HashMap<Uuid, MapDetails>,
}

impl MapGlossary {
    /// Create a glossary containing all built-in map calibrations.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a calibration, returning any previous definition for this asset.
    /// The calibration is validated by [`Self::resolve`] before replay decoding.
    pub fn insert(&mut self, asset_id: Uuid, details: MapDetails) -> Option<MapDetails> {
        self.maps.insert(asset_id, details)
    }

    /// Inspect a registered calibration without validating it for replay decoding.
    pub fn get(&self, asset_id: &Uuid) -> Option<&MapDetails> {
        self.maps.get(asset_id)
    }

    /// Resolve and validate a map calibration for the contextual v41 decoder.
    pub fn resolve(&self, asset_id: &Uuid) -> Result<&MapDetails, MapError> {
        let map = self.get(asset_id).ok_or(MapError::UnknownMap(*asset_id))?;
        // Bound caller-supplied replay calibration here, not the canonical
        // quantization width law, which also covers wider recorded test boxes.
        if map.bounds.axis_bits().is_none()
            || (0..3).any(|axis| map.bounds.max[axis] - map.bounds.min[axis] >= 1_000_000.0)
        {
            return Err(MapError::InvalidBounds(*asset_id));
        }
        if map.bounds.region > 1 || !(1..=8).contains(&map.region_index_bits) {
            return Err(MapError::UnsupportedRegion(*asset_id));
        }
        if map
            .opaque_position_region_bits
            .iter()
            .any(|(&region, &bits)| {
                region == map.bounds.region
                    || region >= (1 << map.region_index_bits)
                    || !(1..=96).contains(&bits)
            })
        {
            return Err(MapError::UnsupportedRegion(*asset_id));
        }
        Ok(map)
    }
}

impl From<MapDefinition> for MapDetails {
    fn from(map: MapDefinition) -> Self {
        Self {
            name: map.name.into(),
            bounds: map.bounds,
            region_index_bits: map.region_index_bits,
            opaque_position_region_bits: map
                .map
                .opaque_position_region_bits()
                .iter()
                .copied()
                .collect(),
        }
    }
}

impl Default for MapGlossary {
    fn default() -> Self {
        Self {
            maps: known_maps::ASSETS
                .iter()
                .map(|&(id, name, map)| {
                    (
                        id,
                        MapDetails {
                            name: name.into(),
                            ..map.definition().into()
                        },
                    )
                })
                .collect(),
        }
    }
}
