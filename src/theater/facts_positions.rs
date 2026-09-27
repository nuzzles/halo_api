//! Shared biped/vehicle position section in the derived native facts cache.
use super::*;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsBipedPosition {
    pub timestamp_us: u64,
    pub slot: u32,
    pub has_world: bool,
    pub quantized: [u32; 3],
    pub world: [f32; 3],
    pub has_yaw: bool,
    pub yaw_raw: u32,
    pub pitch_raw: u32,
    pub directions: FactsPositionDirections,
    pub has_body: bool,
    pub health: f32,
    pub has_shield: bool,
    pub shield: f32,
    pub shield_quantum: u8,
}
pub fn encode_facts_positions(w: &mut NativeFactsWriter, positions: &[FactsBipedPosition]) {
    let mut index = BTreeMap::new();
    let mut slots = Vec::new();
    for p in positions {
        index.entry(p.slot).or_insert_with(|| {
            let ordinal = slots.len();
            slots.push(p.slot);
            ordinal
        });
    }
    w.unsigned(slots.len() as u64);
    for slot in slots {
        w.unsigned(u64::from(slot));
    }
    w.unsigned(positions.len() as u64);
    let mut timestamp = 0u64;
    let mut previous: BTreeMap<u32, [i64; 3]> = BTreeMap::new();
    for p in positions {
        w.unsigned(p.timestamp_us.wrapping_sub(timestamp));
        timestamp = p.timestamp_us;
        w.unsigned(index[&p.slot] as u64);
        let flags = u8::from(p.has_world)
            | (u8::from(p.has_yaw) << 1)
            | (u8::from(p.has_body) << 2)
            | (u8::from(p.has_shield) << 3);
        w.byte(flags);
        if p.has_world {
            let old = previous.entry(p.slot).or_default();
            for (axis, &q) in p.quantized.iter().enumerate() {
                w.signed(i64::from(q).wrapping_sub(old[axis]));
                old[axis] = i64::from(q);
            }
        }
        if p.has_yaw {
            w.unsigned(u64::from(p.yaw_raw));
            w.unsigned(u64::from(p.pitch_raw));
        }
        encode_facts_position_directions(w, &p.directions);
        if p.has_body {
            w.float32(p.health);
        }
        if p.has_shield {
            w.float32(p.shield);
            w.byte(p.shield_quantum);
        }
    }
}
/// Bounds are [minimum XYZ, maximum XYZ], rounded to the native f32 domain.
/// World values are recomputed from cache quanta; absent flags stay absent.
pub fn decode_facts_positions(
    r: &mut NativeFactsReader<'_>,
    layout: &I0Layout,
    bounds: [[f32; 3]; 2],
) -> Vec<FactsBipedPosition> {
    let count = r.count(1);
    let mut slots = Vec::new();
    for _ in 0..count {
        if r.error().is_some() {
            break;
        }
        slots.push(r.unsigned() as u32);
    }
    let count = r.count(3);
    let mut out = Vec::new();
    let mut timestamp = 0u64;
    let mut previous: BTreeMap<u32, [i64; 3]> = BTreeMap::new();
    for _ in 0..count {
        if r.error().is_some() {
            break;
        }
        timestamp = timestamp.wrapping_add(r.unsigned());
        let slot_index = r.unsigned() as i64;
        if slot_index < 0 || slot_index as u64 >= slots.len() as u64 {
            r.replace_error(format!(
                "index de slot {slot_index} hors table ({})",
                slots.len()
            ));
            return out;
        }
        let mut p = FactsBipedPosition {
            timestamp_us: timestamp,
            slot: slots[slot_index as usize],
            ..Default::default()
        };
        let flags = r.byte();
        if flags & 1 != 0 {
            p.has_world = true;
            let old = previous.entry(p.slot).or_default();
            for (axis, prior) in old.iter_mut().enumerate() {
                *prior = prior.wrapping_add(r.signed());
                p.quantized[axis] = *prior as u32;
                let width = layout.axis_widths[axis];
                let divisor = if width < 64 {
                    (1u64 << width) as f64
                } else {
                    0.
                };
                let step = (f64::from(bounds[1][axis]) - f64::from(bounds[0][axis])) / divisor;
                p.world[axis] = (f64::from(bounds[0][axis])
                    + step * (f64::from(p.quantized[axis]) + 0.5))
                    as f32;
            }
        }
        if flags & 2 != 0 {
            p.has_yaw = true;
            p.yaw_raw = r.unsigned() as u32;
            p.pitch_raw = r.unsigned() as u32;
        }
        decode_facts_position_directions(r, &mut p.directions);
        if flags & 4 != 0 {
            p.has_body = true;
            p.health = r.float32();
        }
        if flags & 8 != 0 {
            p.has_shield = true;
            p.shield = r.float32();
            p.shield_quantum = r.byte();
        }
        out.push(p);
    }
    out
}
