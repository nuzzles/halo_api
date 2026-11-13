//! Reference data models.
use crate::theater::parser::v41::context::values::{
    FilmQuantizationRange, REFERENCE_DELTA_QUANTUM, REFERENCE_QUANT_RANGE_CE_BIPED,
};
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PrecisionDescriptor {
    pub index_bits: u64,
    pub axis_bits: [u64; 3],
    pub region: u32,
    /// A complete verified region table when present, including the primary.
    /// Empty preserves older single-region/reference profile serialization.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub regions: std::collections::BTreeMap<u32, [u64; 3]>,
}

impl PrecisionDescriptor {
    pub(crate) fn axes_for_region(&self, region: u32) -> Option<[u64; 3]> {
        if self.regions.is_empty() {
            (region == self.region).then_some(self.axis_bits)
        } else {
            self.regions.get(&region).copied()
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MovementProfile {
    pub traversal: PrecisionDescriptor,
    pub world_object: PrecisionDescriptor,
    /// Fixture-verified vector lengths where the region's coordinate bounds are unknown.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub opaque_position_region_bits: std::collections::BTreeMap<u32, u64>,
    pub delta_quantum: f32,
    pub delta_axis_width: u64,
    pub range: FilmQuantizationRange,
    pub full_precision: bool,
    pub delta_has_handle_tail: bool,
    pub calibrated_skip: bool,
    pub mobility_action_extra_bits: i64,
}

impl Default for MovementProfile {
    fn default() -> Self {
        Self {
            traversal: PrecisionDescriptor {
                regions: Default::default(),
                index_bits: 1,
                axis_bits: [6; 3],
                region: 0,
            },
            world_object: PrecisionDescriptor {
                regions: Default::default(),
                index_bits: 1,
                axis_bits: [13, 13, 14],
                region: 0,
            },
            opaque_position_region_bits: Default::default(),
            delta_quantum: REFERENCE_DELTA_QUANTUM,
            delta_axis_width: 14,
            range: REFERENCE_QUANT_RANGE_CE_BIPED,
            full_precision: false,
            delta_has_handle_tail: false,
            calibrated_skip: false,
            mobility_action_extra_bits: 0,
        }
    }
}
