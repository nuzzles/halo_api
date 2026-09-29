//! Position observations and optional reader-local accumulation from the reference i0 reader.
use crate::theater::parser::v41::*;
use serde::{Deserialize, Serialize};

/// Serializable dequantization context for frame/keyframe readers. Floating point
/// bits preserve exact profile values while keeping encoding equality deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PositionCaptureEncoding {
    pub min_bits: [u32; 3],
    pub max_bits: [u32; 3],
    pub quantum_bits: u32,
    pub region: u32,
    pub axis_widths: [usize; 3],
    pub region_index_bits: usize,
}
impl PositionCaptureEncoding {
    pub(crate) fn map(&self) -> FilmMapBounds {
        FilmMapBounds {
            module: String::new(),
            min: self.min_bits.map(f32::from_bits),
            max: self.max_bits.map(f32::from_bits),
            region: self.region,
            axis_widths: self.axis_widths,
            region_index_bits: self.region_index_bits,
        }
    }
    pub(crate) fn reader<'a>(&self, map: &'a FilmMapBounds, slot: u32) -> PositionCapture<'a> {
        PositionCapture {
            map,
            slot,
            quantum: f32::from_bits(self.quantum_bits),
            world: None,
            emit: true,
        }
    }
}

/// Map and reader-local state used by reference position publications. Suppressing
/// emission still updates an explicitly supplied accumulator.
#[derive(Debug)]
pub(crate) struct PositionCapture<'a> {
    pub map: &'a FilmMapBounds,
    pub quantum: f32,
    pub slot: u32,
    pub world: Option<&'a mut FilmWorld>,
    pub emit: bool,
}

// Go uint64 shifts yield zero at widths >= 64. A zero delta width
// subtracts one in unsigned arithmetic, so its half-range is also zero.
pub(crate) fn reference_shift_one(width: u64) -> u64 {
    if width < 64 { 1u64 << width } else { 0 }
}
