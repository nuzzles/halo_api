//! Native data models.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FilmMapBounds {
    pub module: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    #[serde(rename = "axisWidths")]
    pub axis_widths: [usize; 3],
    #[serde(default)]
    pub region: u32,
    #[serde(default, rename = "regionIndexBits")]
    pub region_index_bits: usize,
}

/// Fixed v41 format-27 component widths.
pub const NATIVE_MPP_DEFAULT_WIDTHS: [usize; 2] = [9, 5];
