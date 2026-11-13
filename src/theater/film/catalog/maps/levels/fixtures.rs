//! Authored point bodies and unchanged upstream primitive traces.
use serde::Deserialize;
#[derive(Deserialize)]
pub(crate) struct Case {
    pub name: String,
    pub tag: u32,
    pub region: Option<u32>,
    pub kind: String,
    pub precision: u8,
    pub codes: [u32; 3],
    pub widths: [usize; 3],
    pub hex: String,
    pub bit_len: usize,
    pub upstream: bool,
}
impl Case {
    pub fn bytes(&self) -> Vec<u8> {
        self.hex
            .as_bytes()
            .chunks(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect()
    }
    pub fn component(&self) -> &'static str {
        match self.kind.as_str() {
            "dynamic" => "object-position-dynamic-precision-component",
            "object" => "object-position-component",
            _ => panic!("not a component"),
        }
    }
    /// Shift a prefix so its final bit coincides exactly with the slice end.
    pub fn cut(&self, cut: usize) -> (Vec<u8>, usize) {
        let source = self.bytes();
        let shift = (8 - cut % 8) % 8;
        let mut out = vec![0; (cut + shift) / 8];
        for i in 0..cut {
            out[(i + shift) / 8] |= ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
        }
        (out, shift)
    }
}
#[derive(Deserialize)]
pub(crate) struct Oracle {
    pub name: String,
    pub end: usize,
    pub marker: u32,
    pub trace: Vec<(usize, usize)>,
}
pub(crate) fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!("../../../../fixtures/levels-v41/points.json")).unwrap()
}
pub(crate) fn oracles() -> Vec<Oracle> {
    #[derive(Deserialize)]
    struct Export {
        commit: String,
        cases: Vec<Oracle>,
    }
    let result: Export = serde_json::from_str(include_str!(
        "../../../../fixtures/levels-v41/reference.json"
    ))
    .unwrap();
    assert_eq!(result.commit, "18067987ebc3628ad499c819c6aee814692df43b");
    result.cases
}
