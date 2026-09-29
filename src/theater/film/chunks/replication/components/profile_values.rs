//! Native data models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FilmMppWidths {
    pub lead: i64,
    pub index: i64,
}

pub type FilmQuantizationRange = [[f32; 2]; 3];

impl Default for FilmMppWidths {
    fn default() -> Self {
        Self {
            lead: crate::theater::film::chunks::replication::components::profile::NATIVE_MPP_DEFAULT_WIDTHS[0] as i64,
            index: crate::theater::film::chunks::replication::components::profile::NATIVE_MPP_DEFAULT_WIDTHS[1] as i64,
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

/// Native captured biped range for the reference map. This is not film-derived
/// calibration for arbitrary maps; consumers must preserve its provenance.
pub const NATIVE_QUANT_RANGE_CE_BIPED: FilmQuantizationRange = [
    [-41.10318, 72.10963],
    [-56.60697, 57.212566],
    [-84.37078, 53.18034],
];
