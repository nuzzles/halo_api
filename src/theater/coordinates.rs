use super::CoordinateLayout;
use serde::{Deserialize, Serialize};

/// Map BSP quantization bounds, supplied independently of the film. Do not
/// substitute playable-area bounds or assign another map's bounds by width.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CoordinateBounds {
    /// Minimum X/Y/Z in world units.
    pub min: [f32; 3],
    /// Maximum X/Y/Z in world units.
    pub max: [f32; 3],
}

impl CoordinateBounds {
    /// Axis widths for the observed precision level 16: ceil(log2(ceil(60 *
    /// extent))), capped at 26. Invalid or degenerate bounds return None.
    pub fn axis_bits(self) -> Option<[usize; 3]> {
        let mut bits = [0; 3];
        for (i, width) in bits.iter_mut().enumerate() {
            let extent = f64::from(self.max[i]) - f64::from(self.min[i]);
            if !self.min[i].is_finite() || !self.max[i].is_finite() || extent <= 0.0 {
                return None;
            }
            *width = (extent * 60.0).ceil().log2().ceil().min(26.0) as usize;
        }
        Some(bits)
    }

    /// Midpoint dequantization. Requires bounds whose predicted axis widths
    /// match the independently checked layout. Units are not assumed to be metres.
    pub fn world_position(self, raw: [u32; 3], layout: CoordinateLayout) -> Option<[f32; 3]> {
        let bits = self.axis_bits()?;
        if bits != layout.axis_bits() {
            return None;
        }
        let mut result = [0.0; 3];
        for i in 0..3 {
            let levels = 1u32 << bits[i];
            if raw[i] >= levels {
                return None;
            }
            result[i] = (f64::from(self.min[i])
                + (f64::from(raw[i]) + 0.5) * (f64::from(self.max[i]) - f64::from(self.min[i]))
                    / f64::from(levels)) as f32;
        }
        Some(result)
    }
}
