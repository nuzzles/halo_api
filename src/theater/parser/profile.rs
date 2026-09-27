//! v41 profile resolution from recorded version/build and the pinned map catalog.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Effective component corruption checks and whether identification declared them.
/// Standalone profile composition preserves the inherited value when identification
/// is absent. Parser contexts start from the invariant false and retain that decision
/// across later custom profile replacements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FilmCorruptionControl {
    pub enabled: bool,
    pub declared: bool,
}

impl FilmCorruptionControl {
    pub(crate) fn from_recorded(recorded: Option<bool>, inherited: bool) -> Self {
        Self {
            enabled: recorded.unwrap_or(inherited),
            declared: recorded.is_some(),
        }
    }
}

/// Native scan invariant used when no declared/calibrated MPP layout is installed.
pub const NATIVE_MPP_DEFAULT_WIDTHS: [usize; 2] = [9, 5];

/// Format membership and declared MPP widths are separate native facts. Historical
/// formats remain known metadata even though their MPP widths are undetermined;
/// this does not enable decoding any major version other than 41.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FilmMppResolution {
    pub format_version: u32,
    pub widths: Option<[usize; 2]>,
    pub unknown_format: bool,
}

pub(crate) fn resolve_film_mpp(format_version: u32) -> FilmMppResolution {
    FilmMppResolution {
        format_version,
        widths: (format_version == 27).then_some([9, 5]),
        unknown_format: !matches!(format_version, 20 | 21 | 24 | 25 | 27),
    }
}
