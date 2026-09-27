//! Keyframe presence lists in the native derived facts cache.
use super::{NativeFactsReader, NativeFactsWriter, WorldObjectKeyframes};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FactsWorldKeyframes {
    pub times_us: Vec<u64>,
    pub seen_us: BTreeMap<(u32, u32), Vec<u64>>,
}
impl From<&WorldObjectKeyframes> for FactsWorldKeyframes {
    fn from(value: &WorldObjectKeyframes) -> Self {
        Self {
            times_us: value.times_us.clone(),
            seen_us: value
                .seen_us
                .iter()
                .map(|life| ((life.slot, life.generation), life.times_us.clone()))
                .collect(),
        }
    }
}
pub fn encode_facts_keyframes(writer: &mut NativeFactsWriter, frames: &FactsWorldKeyframes) {
    writer.unsigned(frames.times_us.len() as u64);
    let mut previous = 0u64;
    for &timestamp in &frames.times_us {
        writer.unsigned(timestamp.wrapping_sub(previous));
        previous = timestamp;
    }
    writer.unsigned(frames.seen_us.len() as u64);
    for (&(slot, generation), times) in &frames.seen_us {
        writer.unsigned(u64::from(slot));
        writer.unsigned(u64::from(generation));
        writer.unsigned(times.len() as u64);
        previous = 0;
        for &timestamp in times {
            writer.unsigned(timestamp.wrapping_sub(previous));
            previous = timestamp;
        }
    }
}
/// Preserve native partial results and the count(2) guard, including its dependence
/// on trailing bytes. Negative slice counts are safely refused instead of panicking.
pub fn decode_facts_keyframes(reader: &mut NativeFactsReader<'_>) -> FactsWorldKeyframes {
    let mut out = FactsWorldKeyframes::default();
    let count = reader.unsigned() as i64;
    if count < 0 {
        reader.fail(format!(
            "invalid signed keyframe count {count} at offset {}",
            reader.offset()
        ));
        return out;
    }
    let mut previous = 0u64;
    for _ in 0..count {
        if reader.error().is_some() {
            break;
        }
        previous = previous.wrapping_add(reader.unsigned());
        out.times_us.push(previous);
    }
    let count = reader.unsigned() as i64;
    for _ in 0..count {
        if reader.error().is_some() {
            break;
        }
        let key = (reader.unsigned() as u32, reader.unsigned() as u32);
        let count = reader.count(2);
        let mut times = Vec::new();
        previous = 0;
        for _ in 0..count {
            if reader.error().is_some() {
                break;
            }
            previous = previous.wrapping_add(reader.unsigned());
            times.push(previous);
        }
        out.seen_us.insert(key, times);
    }
    out
}
