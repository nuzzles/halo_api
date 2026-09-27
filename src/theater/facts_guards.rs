//! Mode-guarded channels in the derived native facts cache.
use super::{
    CarrierMark, ManagedPropertyRead, NativeFactsReader, NativeFactsWriter, NavpointRadialRead,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FactsCarrierMarkScan {
    pub marks: Option<Vec<CarrierMark>>,
    #[serde(rename = "KeyframeUS")]
    pub keyframe_us: Option<Vec<u64>>,
    pub records: i64,
    pub biped_records: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FactsModeGuards {
    pub flag_marks: FactsCarrierMarkScan,
    pub zone_reads: Option<Vec<ManagedPropertyRead>>,
    pub zone_scanned: bool,
    pub flag_gauge: Option<Vec<ManagedPropertyRead>>,
    pub flag_gauge_scanned: bool,
    pub bomb_reads: Option<Vec<NavpointRadialRead>>,
}
// Native uses signed int counts and eager slice capacities. Avoid allocations
// from untrusted counts; retain its reads and partial append behavior. Negative
// capacities are safely refused at the native panic cursor.
fn slice_count(r: &mut NativeFactsReader<'_>) -> i64 {
    let n = r.unsigned() as i64;
    if n < 0 {
        r.fail(format!(
            "invalid signed guarded channel count {n} at offset {}",
            r.offset()
        ));
    }
    n
}
fn nonempty<T>(v: Vec<T>) -> Option<Vec<T>> {
    if v.is_empty() { None } else { Some(v) }
}
pub fn encode_facts_carrier_marks(w: &mut NativeFactsWriter, s: &FactsCarrierMarkScan) {
    let marks = s.marks.as_deref().unwrap_or_default();
    w.unsigned(marks.len() as u64);
    let mut last = 0;
    for m in marks {
        w.unsigned(m.timestamp_us.wrapping_sub(last));
        last = m.timestamp_us;
        w.unsigned(m.slot.into());
    }
    let times = s.keyframe_us.as_deref().unwrap_or_default();
    w.unsigned(times.len() as u64);
    last = 0;
    for &ts in times {
        w.unsigned(ts.wrapping_sub(last));
        last = ts;
    }
    w.unsigned(s.records as u64);
    w.unsigned(s.biped_records as u64);
}
pub fn decode_facts_carrier_marks(r: &mut NativeFactsReader<'_>) -> FactsCarrierMarkScan {
    let mut marks = Vec::new();
    let mut last = 0u64;
    for _ in 0..slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        marks.push(CarrierMark {
            timestamp_us: last,
            slot: r.unsigned() as u32,
        });
    }
    let mut times = Vec::new();
    last = 0;
    for _ in 0..slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        times.push(last);
    }
    FactsCarrierMarkScan {
        marks: nonempty(marks),
        keyframe_us: nonempty(times),
        records: r.unsigned() as i64,
        biped_records: r.unsigned() as i64,
    }
}
pub fn encode_facts_managed_reads(w: &mut NativeFactsWriter, reads: &[ManagedPropertyRead]) {
    w.unsigned(reads.len() as u64);
    for z in reads {
        w.unsigned(z.slot.into());
        w.unsigned(z.timestamp_us);
        w.signed(z.field);
        w.signed(z.film_index);
        w.signed(z.tag);
        w.unsigned(z.value);
        w.boolean(z.has_value);
        w.boolean(z.chained);
    }
}
pub fn decode_facts_managed_reads(
    r: &mut NativeFactsReader<'_>,
) -> Option<Vec<ManagedPropertyRead>> {
    let mut out = Vec::new();
    for _ in 0..slice_count(r) {
        if r.error().is_some() {
            break;
        }
        out.push(ManagedPropertyRead {
            slot: r.unsigned() as u32,
            timestamp_us: r.unsigned(),
            field: r.signed(),
            film_index: r.signed(),
            tag: r.signed(),
            value: r.unsigned(),
            has_value: r.boolean(),
            chained: r.boolean(),
        });
    }
    nonempty(out)
}
pub fn encode_facts_mode_guards(w: &mut NativeFactsWriter, s: &FactsModeGuards) {
    encode_facts_carrier_marks(w, &s.flag_marks);
    encode_facts_managed_reads(w, s.zone_reads.as_deref().unwrap_or_default());
    w.boolean(s.zone_scanned);
    encode_facts_managed_reads(w, s.flag_gauge.as_deref().unwrap_or_default());
    w.boolean(s.flag_gauge_scanned);
    let bombs = s.bomb_reads.as_deref().unwrap_or_default();
    w.unsigned(bombs.len() as u64);
    for b in bombs {
        w.unsigned(b.slot.into());
        w.signed(b.time_ms.into());
        w.byte(b.q);
        w.boolean(b.chained);
    }
}
pub fn decode_facts_mode_guards(r: &mut NativeFactsReader<'_>) -> FactsModeGuards {
    let mut s = FactsModeGuards {
        flag_marks: decode_facts_carrier_marks(r),
        zone_reads: decode_facts_managed_reads(r),
        zone_scanned: r.boolean(),
        flag_gauge: decode_facts_managed_reads(r),
        flag_gauge_scanned: r.boolean(),
        bomb_reads: None,
    };
    let mut bombs = Vec::new();
    for _ in 0..slice_count(r) {
        if r.error().is_some() {
            break;
        }
        bombs.push(NavpointRadialRead {
            slot: r.unsigned() as u32,
            time_ms: r.signed() as i32,
            q: r.byte(),
            chained: r.boolean(),
        });
    }
    s.bomb_reads = nonempty(bombs);
    s
}
