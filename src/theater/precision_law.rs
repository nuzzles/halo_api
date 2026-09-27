//! Native float32 map precision law. Catalog entries remain authoritative for
//! decoding; these helpers expose the engine's bounds-to-width calculation.

pub const OBJECT_POSITION_PRECISION_LEVEL: i64 = 16;
pub const MAX_POSITION_AXIS_BITS: u32 = 26;
pub const BUILD_DEFAULT_POSITION_BOUNDS: [[f32; 2]; 3] = [[-20000., 20000.]; 3];

/// Quantization step, preserving native uint32 shift semantics at extreme levels.
pub fn position_precision_step(level: i64) -> f32 {
    let shift = if level <= OBJECT_POSITION_PRECISION_LEVEL {
        OBJECT_POSITION_PRECISION_LEVEL.wrapping_sub(level)
    } else {
        level.wrapping_sub(OBJECT_POSITION_PRECISION_LEVEL)
    } as u64;
    let factor = if shift < 32 {
        (1_u32 << shift) as f32
    } else {
        0.
    };
    let step = 1_f32 / 120.;
    if level <= OBJECT_POSITION_PRECISION_LEVEL {
        factor * step
    } else {
        step / factor
    }
}
/// Axis widths from min/max pairs, with float32 subtraction/division and the
/// engine's counting cap. No validity or map identity is inferred from the result.
pub fn position_axis_widths(bounds: [[f32; 2]; 3], level: i64) -> [u32; 3] {
    let step = position_precision_step(level);
    if step < 1e-4 {
        return [MAX_POSITION_AXIS_BITS; 3];
    }
    let twice = step + step;
    let limit = twice * ((1_u32 << 22) as f32);
    bounds.map(|[min, max]| {
        let extent = max - min;
        let bins = if extent < limit {
            // Native converts through the machine integer register, retaining
            // the low 32 bits even for negative extents.
            ((extent / twice).ceil() as i64) as u32
        } else {
            1 << 22
        };
        super::native_quantized_bit_len(bins).min(MAX_POSITION_AXIS_BITS)
    })
}
pub fn default_position_axis_widths(level: i64) -> [u32; 3] {
    position_axis_widths(BUILD_DEFAULT_POSITION_BOUNDS, level)
}
/// The raw region count, not the count of geometrically valid regions.
pub fn position_region_index_width(count: i64) -> u32 {
    if count == 1 {
        1
    } else {
        super::native_quantized_bit_len(count.max(0) as u32).min(MAX_POSITION_AXIS_BITS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[test]
    fn native_precision_law() {
        #[derive(Deserialize)]
        struct Case {
            level: i64,
            count: i64,
            bounds_bits: [[u32; 2]; 3],
            step_bits: u32,
            widths: [u32; 3],
            defaults: [u32; 3],
            index: u32,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/precision-law-v41.json.zlib")[..])
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 2061);
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                position_precision_step(c.level).to_bits(),
                c.step_bits,
                "step {i}"
            );
            assert_eq!(
                position_axis_widths(c.bounds_bits.map(|b| b.map(f32::from_bits)), c.level),
                c.widths,
                "widths {i}"
            );
            assert_eq!(
                default_position_axis_widths(c.level),
                c.defaults,
                "defaults {i}"
            );
            assert_eq!(position_region_index_width(c.count), c.index, "index {i}");
        }
    }
}
