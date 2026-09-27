//! Native placement denominators and deterministic map codecs for the facts cache.
use super::{NativeFactsReader, NativeFactsWriter};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FactsMppCalibration {
    pub widths: (i64, i64),
    pub agree: i64,
    pub runner: (i64, i64),
    pub runner_agree: i64,
    pub anchors: i64,
    pub chunks: i64,
    pub lives: i64,
    pub by_widths: Option<BTreeMap<(i64, i64), i64>>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FactsPlacementStats {
    pub scanned: bool,
    pub calibration: FactsMppCalibration,
    pub lives: i64,
    pub slots: i64,
    pub anchors: i64,
    pub accepted: i64,
    pub confirmed: i64,
    pub placements: i64,
    pub by_id: Option<BTreeMap<u32, i64>>,
    pub format_version: i64,
    pub format_sans_profil: bool,
}
pub fn encode_facts_width_pairs(
    w: &mut NativeFactsWriter,
    pairs: Option<&BTreeMap<(i64, i64), i64>>,
) {
    w.unsigned(pairs.map_or(0, |p| p.len()) as u64);
    if let Some(pairs) = pairs {
        for (&(lead, index), &value) in pairs {
            w.signed(lead);
            w.signed(index);
            w.signed(value);
        }
    }
}
pub fn decode_facts_width_pairs(
    r: &mut NativeFactsReader<'_>,
) -> Option<BTreeMap<(i64, i64), i64>> {
    let count = r.count(3);
    if count == 0 {
        return None;
    }
    let mut out = BTreeMap::new();
    for _ in 0..count {
        if r.error().is_some() {
            break;
        }
        let key = (r.signed(), r.signed());
        out.insert(key, r.signed());
    }
    Some(out)
}
pub fn encode_facts_id_pairs(w: &mut NativeFactsWriter, pairs: Option<&BTreeMap<u32, i64>>) {
    w.unsigned(pairs.map_or(0, |p| p.len()) as u64);
    if let Some(pairs) = pairs {
        for (&id, &value) in pairs {
            w.unsigned(id.into());
            w.signed(value);
        }
    }
}
pub fn decode_facts_id_pairs(r: &mut NativeFactsReader<'_>) -> Option<BTreeMap<u32, i64>> {
    let count = r.count(2);
    if count == 0 {
        return None;
    }
    let mut out = BTreeMap::new();
    for _ in 0..count {
        if r.error().is_some() {
            break;
        }
        let key = r.unsigned() as u32;
        out.insert(key, r.signed());
    }
    Some(out)
}
pub fn encode_facts_placement_stats(w: &mut NativeFactsWriter, s: &FactsPlacementStats) {
    w.boolean(s.scanned);
    let c = &s.calibration;
    for value in [
        c.widths.0,
        c.widths.1,
        c.agree,
        c.runner.0,
        c.runner.1,
        c.runner_agree,
        c.anchors,
        c.chunks,
        c.lives,
    ] {
        w.signed(value);
    }
    encode_facts_width_pairs(w, c.by_widths.as_ref());
    for value in [
        s.lives,
        s.slots,
        s.anchors,
        s.accepted,
        s.confirmed,
        s.placements,
    ] {
        w.signed(value);
    }
    encode_facts_id_pairs(w, s.by_id.as_ref());
    w.signed(s.format_version);
    w.boolean(s.format_sans_profil);
}
pub fn decode_facts_placement_stats(r: &mut NativeFactsReader<'_>) -> FactsPlacementStats {
    FactsPlacementStats {
        scanned: r.boolean(),
        calibration: FactsMppCalibration {
            widths: (r.signed(), r.signed()),
            agree: r.signed(),
            runner: (r.signed(), r.signed()),
            runner_agree: r.signed(),
            anchors: r.signed(),
            chunks: r.signed(),
            lives: r.signed(),
            by_widths: decode_facts_width_pairs(r),
        },
        lives: r.signed(),
        slots: r.signed(),
        anchors: r.signed(),
        accepted: r.signed(),
        confirmed: r.signed(),
        placements: r.signed(),
        by_id: decode_facts_id_pairs(r),
        format_version: r.signed(),
        format_sans_profil: r.boolean(),
    }
}
