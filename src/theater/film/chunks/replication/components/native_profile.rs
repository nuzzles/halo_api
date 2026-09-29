//! Native data models.
use crate::theater::film::*;
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NativePrecisionDescriptor {
    pub index_bits: u64,
    pub axis_bits: [u64; 3],
    pub region: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NativeMovementProfile {
    pub traversal: NativePrecisionDescriptor,
    pub world_object: NativePrecisionDescriptor,
    pub delta_quantum: f32,
    pub delta_axis_width: u64,
    pub range: FilmQuantizationRange,
    pub full_precision: bool,
    pub delta_has_handle_tail: bool,
    pub calibrated_skip: bool,
    pub mobility_action_extra_bits: i64,
}

impl Default for NativeMovementProfile {
    fn default() -> Self {
        Self {
            traversal: NativePrecisionDescriptor {
                index_bits: 1,
                axis_bits: [6; 3],
                region: 0,
            },
            world_object: NativePrecisionDescriptor {
                index_bits: 1,
                axis_bits: [13, 13, 14],
                region: 0,
            },
            delta_quantum: super::profile_values::NATIVE_DELTA_QUANTUM,
            delta_axis_width: 14,
            range: super::profile_values::NATIVE_QUANT_RANGE_CE_BIPED,
            full_precision: false,
            delta_has_handle_tail: false,
            calibrated_skip: false,
            mobility_action_extra_bits: 0,
        }
    }
}
