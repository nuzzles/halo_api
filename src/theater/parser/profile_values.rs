//! Value-level profile declarations from the pinned native parser. Bounds below
//! identify specific reference ranges; none is a universal map fallback.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FilmMppWidths {
    pub lead: i64,
    pub index: i64,
}
impl Default for FilmMppWidths {
    fn default() -> Self {
        Self {
            lead: super::NATIVE_MPP_DEFAULT_WIDTHS[0] as i64,
            index: super::NATIVE_MPP_DEFAULT_WIDTHS[1] as i64,
        }
    }
}
impl std::fmt::Display for FilmMppWidths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.lead, self.index)
    }
}

/// Native profile invariant for delta-position observer dequantization.
pub const NATIVE_DELTA_QUANTUM: f32 = 0.01383;

pub type FilmQuantizationRange = [[f32; 2]; 3];
/// Native captured biped range for the reference map. This is not film-derived
/// calibration for arbitrary maps; consumers must preserve its provenance.
pub const NATIVE_QUANT_RANGE_CE_BIPED: FilmQuantizationRange = [
    [-41.10318, 72.10963],
    [-56.60697, 57.212566],
    [-84.37078, 53.18034],
];
