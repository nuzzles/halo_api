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
impl FilmMppWidths {
    /// Native value-level validity only requires positive widths. Individual
    /// readers separately enforce the widths they can consume.
    pub fn is_valid(self) -> bool {
        self.lead > 0 && self.index > 0
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
pub const NATIVE_QUANT_RANGE_UNIT3: FilmQuantizationRange = [[-3., 3.]; 3];
pub const NATIVE_QUANT_RANGE_NORM: FilmQuantizationRange = [[-0.7, 0.7]; 3];
pub const NATIVE_QUANT_RANGE_WORLD100: FilmQuantizationRange = [[-100., 100.]; 3];
/// Rejected historical bounds retained by the native reference as evidence of
/// an earlier error. Do not use for position decoding or as a map fallback.
pub const REJECTED_HISTORICAL_CLIFFHANGER_RANGE: FilmQuantizationRange = [
    [-973.867, 179.377],
    [-361.439, 1047.008],
    [-86.552, 489.092],
];
/// Native captured biped range for the reference map. This is not film-derived
/// calibration for arbitrary maps; consumers must preserve its provenance.
pub const NATIVE_QUANT_RANGE_CE_BIPED: FilmQuantizationRange = [
    [-41.10318, 72.10963],
    [-56.60697, 57.212566],
    [-84.37078, 53.18034],
];
/// Native index==-1 build range only. Indexed map positions use map ranges.
pub fn native_build_quantization_range() -> FilmQuantizationRange {
    super::BUILD_DEFAULT_POSITION_BOUNDS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_profile_values() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/profile-values-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let oracle: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let expected: FilmMppWidths = serde_json::from_value(oracle["default"].clone()).unwrap();
        assert_eq!(FilmMppWidths::default(), expected);
        let rows = oracle["widths"].as_array().unwrap();
        assert_eq!(rows.len(), 81);
        for row in rows {
            let widths: FilmMppWidths = serde_json::from_value(row["widths"].clone()).unwrap();
            assert_eq!(widths.is_valid(), row["valid"].as_bool().unwrap());
            assert_eq!(widths.to_string(), row["text"].as_str().unwrap());
            assert_eq!(serde_json::to_value(widths).unwrap(), row["widths"]);
        }
        for (name, range) in [
            ("unit3", NATIVE_QUANT_RANGE_UNIT3),
            ("norm", NATIVE_QUANT_RANGE_NORM),
            ("world100", NATIVE_QUANT_RANGE_WORLD100),
            ("rejected", REJECTED_HISTORICAL_CLIFFHANGER_RANGE),
            ("biped", NATIVE_QUANT_RANGE_CE_BIPED),
            ("build", native_build_quantization_range()),
        ] {
            let expected: [[u32; 2]; 3] =
                serde_json::from_value(oracle["ranges"][name].clone()).unwrap();
            assert_eq!(range.map(|axis| axis.map(f32::to_bits)), expected, "{name}");
        }
    }
}
