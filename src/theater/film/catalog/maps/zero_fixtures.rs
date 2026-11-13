//! Shared test inputs from the unchanged original width law and point reader.
use super::PositionBounds;
#[derive(serde::Deserialize)]
pub(crate) struct Case {
    pub name: String,
    pub precision: u8,
    pub min: [u32; 3],
    pub max: [u32; 3],
    pub widths: [usize; 3],
    pub codes: [u32; 3],
    pub hex: String,
    pub bit_len: usize,
    pub trace: Vec<(usize, usize)>,
}
pub(crate) fn cases() -> Vec<Case> {
    #[derive(serde::Deserialize)]
    struct Reference {
        cases: Vec<Case>,
    }
    serde_json::from_str::<Reference>(include_str!(
        "../../../fixtures/zero-position-axes-v41/reference.json"
    ))
    .unwrap()
    .cases
}
impl Case {
    pub fn bounds(&self) -> PositionBounds {
        PositionBounds {
            region: 3,
            min: self.min.map(f32::from_bits),
            max: self.max.map(f32::from_bits),
        }
    }
    pub fn bytes(&self) -> Vec<u8> {
        self.hex
            .as_bytes()
            .chunks(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect()
    }
    pub fn cut(&self, n: usize) -> (Vec<u8>, usize) {
        let start = (8 - n % 8) % 8;
        let source = self.bytes();
        let mut out = vec![0; (start + n) / 8];
        for i in 0..n {
            out[(start + i) / 8] |= ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (start + i) % 8);
        }
        (out, start)
    }
}

#[test]
fn original_zero_axis_width_law_matches_all_profiles_and_midpoints() {
    let cases = cases();
    assert_eq!(cases.len(), 80);
    assert!(cases.iter().any(|c| c.widths == [0; 3]));
    for c in cases {
        let b = c.bounds();
        assert_eq!(
            b.axis_bits_at_precision(c.precision),
            Some(c.widths),
            "{}",
            c.name
        );
        let p = b
            .dequantize_at_precision(c.codes, c.widths, c.precision)
            .unwrap();
        for i in 0..3 {
            if c.widths[i] == 0 {
                assert_eq!(p[i], (f64::from(b.min[i]) + f64::from(b.max[i])) / 2.0);
                let mut bad = c.codes;
                bad[i] = 1;
                assert!(
                    b.dequantize_at_precision(bad, c.widths, c.precision)
                        .is_none()
                );
            }
        }
    }
}
