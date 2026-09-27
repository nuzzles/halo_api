//! Native v41 vitality values. Serialized health may be negative; shields may exceed one.
use super::FilmBitReader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodedBodyVitality {
    pub quantum: u8,
    pub health: f32,
    /// Flags at object offsets 0x5c, 0x5d and 0x5e; meanings remain unknown.
    pub flags: [bool; 3],
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodedShieldVitality {
    pub quantum: u8,
    pub shield: f32,
    pub regen_present: bool,
    pub regen: [Option<u16>; 2],
    pub block_64: u16,
    /// Binary order: 0x66, 0x67, 0x69, 0x68.
    pub flags: [bool; 4],
}

/// Native endpoint/midpoint dequantization. Quanta outside the stated domain
/// are not clamped. Shift/subtraction and midpoint comparisons preserve native
/// unsigned wrapping; degenerate widths retain IEEE infinities/NaNs.
pub fn dequantize_native_endpoint(
    q: u64,
    min: f32,
    max: f32,
    width: u32,
    exclude_level: bool,
    exact_endpoints: bool,
) -> f32 {
    let mut levels = 1_u64.checked_shl(width).unwrap_or(0);
    if exclude_level {
        levels = levels.wrapping_sub(1);
    }
    let value = if exact_endpoints && q == 0 {
        min
    } else if exact_endpoints && q == levels.wrapping_sub(1) {
        max
    } else if exact_endpoints {
        let step = (max - min) / levels.wrapping_sub(2) as f32;
        (q.wrapping_sub(1) as f32).mul_add(step, step.mul_add(0.5, min))
    } else {
        let step = (max - min) / levels as f32;
        (q as f32).mul_add(step, min) + step * 0.5
    };
    if exclude_level && q.wrapping_mul(2) == levels.wrapping_sub(1) {
        ((f64::from(min) + f64::from(max)) * 0.5) as f32
    } else {
        value
    }
}

/// Native manual-navpoint duration in seconds, including its 0.1ms dead zone.
/// This conversion does not assign elapsed/remaining gameplay meaning.
pub fn native_navpoint_manual_timer_seconds(q: u64) -> f32 {
    // Keep the reference's decimal literal visible for source comparison.
    #[allow(clippy::excessive_precision)]
    let value = dequantize_native_endpoint(q, -0.025, 6553.5752, 17, false, false);
    if (-1.0e-4..=1.0e-4).contains(&value) {
        0.
    } else {
        value
    }
}

pub(super) fn vitality_value(q: u8, body: bool) -> f32 {
    let (min, max) = if body { (-1., 1.) } else { (0., 4.) };
    dequantize_native_endpoint(u64::from(q), min, max, 8, body, true)
}

/// Read i4 atomically: truncation returns None and leaves the cursor unchanged.
pub fn read_body_vitality(cursor: &mut FilmBitReader<'_>) -> Option<DecodedBodyVitality> {
    let mut c = *cursor;
    let quantum = c.read(8)? as u8;
    let flags = [c.bit()?, c.bit()?, c.bit()?];
    *cursor = c;
    Some(DecodedBodyVitality {
        quantum,
        health: vitality_value(quantum, true),
        flags,
    })
}

/// Read i5 atomically, preserving optional regeneration words and all flags.
pub fn read_shield_vitality(cursor: &mut FilmBitReader<'_>) -> Option<DecodedShieldVitality> {
    let mut c = *cursor;
    let quantum = c.read(8)? as u8;
    let regen_present = c.bit()?;
    let mut regen = [None; 2];
    if regen_present {
        for value in &mut regen {
            if c.bit()? {
                *value = Some(c.read(12)? as u16);
            }
        }
    }
    let block_64 = c.read(16)? as u16;
    let flags = [c.bit()?, c.bit()?, c.bit()?, c.bit()?];
    *cursor = c;
    Some(DecodedShieldVitality {
        quantum,
        shield: vitality_value(quantum, false),
        regen_present,
        regen,
        block_64,
        flags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_generic_endpoint_and_navpoint_timer() {
        #[derive(Deserialize)]
        struct Case {
            q: u64,
            width: u32,
            min: u32,
            max: u32,
            exclude: bool,
            exact: bool,
            value: u32,
        }
        #[derive(Deserialize)]
        struct Oracle {
            cases: Vec<Case>,
            navpoint: Vec<u32>,
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/endpoint-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let oracle: Oracle = serde_json::from_slice(&raw).unwrap();
        assert_eq!(oracle.cases.len(), 4096);
        for (i, c) in oracle.cases.into_iter().enumerate() {
            let actual = dequantize_native_endpoint(
                c.q,
                f32::from_bits(c.min),
                f32::from_bits(c.max),
                c.width,
                c.exclude,
                c.exact,
            );
            if f32::from_bits(c.value).is_nan() {
                assert!(actual.is_nan(), "NaN {i}");
            } else {
                assert_eq!(actual.to_bits(), c.value, "case {i}");
            }
        }
        assert_eq!(oracle.navpoint.len(), 1 << 17);
        for (q, expected) in oracle.navpoint.into_iter().enumerate() {
            assert_eq!(
                native_navpoint_manual_timer_seconds(q as u64).to_bits(),
                expected,
                "navpoint {q}"
            );
        }
    }
    #[test]
    fn every_vitality_quantum_matches_native_go() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-scan-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut json)
        .unwrap();
        let oracle: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rows = oracle["vitality"].as_array().unwrap();
        assert_eq!(rows.len(), 256);
        for row in rows {
            let q = row["q"].as_u64().unwrap() as u8;
            for (body, key) in [(true, "health"), (false, "shield")] {
                assert_eq!(
                    vitality_value(q, body).to_bits(),
                    (row[key].as_f64().unwrap() as f32).to_bits(),
                    "{key} q={q}"
                );
            }
        }
        let reads = oracle["vitality_reads"].as_array().unwrap();
        assert_eq!(reads.len(), 512);
        for row in reads {
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let start = row["start"].as_u64().unwrap() as usize;
            let mut c = FilmBitReader::new(&bytes, start).unwrap();
            let b = read_body_vitality(&mut c).unwrap();
            assert_eq!(c.position as u64, row["body_end"].as_u64().unwrap());
            let mut e = row["body"].clone();
            e["Health"] = serde_json::json!(e["Health"].as_f64().unwrap() as f32);
            assert_eq!(
                serde_json::json!({"Q":b.quantum,"Health":b.health,"F5c":b.flags[0],"F5d":b.flags[1],"F5e":b.flags[2]}),
                e
            );
            c.position = start;
            let s = read_shield_vitality(&mut c).unwrap();
            assert_eq!(c.position as u64, row["shield_end"].as_u64().unwrap());
            // JSON float spelling differs across languages; compare rounded binary values.
            let mut expected = row["shield"].clone();
            expected["Shield"] = serde_json::json!(expected["Shield"].as_f64().unwrap() as f32);
            assert_eq!(
                serde_json::json!({"Q":s.quantum,"Shield":s.shield,"RegenPresent":s.regen_present,"HasRegen0":s.regen[0].is_some(),"HasRegen1":s.regen[1].is_some(),"Regen0":s.regen[0].unwrap_or(0),"Regen1":s.regen[1].unwrap_or(0),"Block64":s.block_64,"F66":s.flags[0],"F67":s.flags[1],"F69":s.flags[2],"F68":s.flags[3]}),
                expected
            );
            for end in 0..bytes.len() {
                if let Some(mut short) = FilmBitReader::new(&bytes[..end], start) {
                    if read_body_vitality(&mut short).is_none() {
                        assert_eq!(short.position, start);
                    }
                    short.position = start;
                    if read_shield_vitality(&mut short).is_none() {
                        assert_eq!(short.position, start);
                    }
                }
            }
        }
    }
}
