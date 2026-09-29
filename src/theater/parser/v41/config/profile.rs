//! Reference data models.
use super::values::FilmQuantizationRange;
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PrecisionDescriptor {
    pub index_bits: u64,
    pub axis_bits: [u64; 3],
    pub region: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MovementProfile {
    pub traversal: PrecisionDescriptor,
    pub world_object: PrecisionDescriptor,
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
                index_bits: 1,
                axis_bits: [6; 3],
                region: 0,
            },
            world_object: PrecisionDescriptor {
                index_bits: 1,
                axis_bits: [13, 13, 14],
                region: 0,
            },
            delta_quantum: crate::theater::parser::v41::REFERENCE_DELTA_QUANTUM,
            delta_axis_width: 14,
            range: crate::theater::parser::v41::REFERENCE_QUANT_RANGE_CE_BIPED,
            full_precision: false,
            delta_has_handle_tail: false,
            calibrated_skip: false,
            mobility_action_extra_bits: 0,
        }
    }
}
