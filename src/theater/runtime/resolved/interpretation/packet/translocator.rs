//! Reference data models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TranslocatorStop {
    PositionsRead,
    Truncated,
    UnsupportedReferences,
    MissingMap,
    InvalidMap,
    UnknownRegion(u32),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TeleportPosition {
    pub start_bit: usize,
    pub end_bit: usize,
    /// None is the engine's default box (22 bits, +/-20000), not an inferred map.
    pub region: Option<u32>,
    pub axis_bits: [usize; 3],
    pub quantized: [u32; 3],
    pub world: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TranslocatorEvent {
    pub config: bool,
    pub slot: u32,
    pub generation: u8,
    pub other_reference_gates: Option<[bool; 2]>,
    pub effect_present: Option<bool>,
    pub effect: Option<u32>,
    pub from: Option<TeleportPosition>,
    pub to: Option<TeleportPosition>,
    pub end_bit: usize,
    pub stop: TranslocatorStop,
}

/// Reference decoder outcome, including its synthetic zero-tail convention.
/// Slot/reference values crossing source_bits are not fully recorded identities.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TranslocatorRead {
    pub event: TranslocatorEvent,
    pub source_bits: usize,
    /// Logical reader extent beyond the payload, including skipped tail fields.
    pub padded_bits: usize,
}

use crate::theater::parser::bits::Cursor;
use crate::theater::parser::v41::FilmMapBounds;

fn decode_translocator_with_cursor(
    data: &[u8],
    map: Option<&FilmMapBounds>,
    mut r: Cursor<'_>,
) -> Option<TranslocatorEvent> {
    let config = r.read(1)? != 0;
    if r.read(1)? == 0 || r.read(7)? != 117 || r.read(1)? == 0 {
        return None;
    }
    let slot = 512 + r.read(8)? as u32;
    let generation = r.read(2)? as u8;
    let mut event = TranslocatorEvent {
        config,
        slot,
        generation,
        other_reference_gates: None,
        effect_present: None,
        effect: None,
        from: None,
        to: None,
        end_bit: r.position,
        stop: TranslocatorStop::Truncated,
    };
    event.stop =
        payload(&mut r, map, &mut event, data.len() * 8).unwrap_or(TranslocatorStop::Truncated);
    event.end_bit = r.position;
    Some(event)
}

fn payload(
    r: &mut Cursor<'_>,
    map: Option<&FilmMapBounds>,
    event: &mut TranslocatorEvent,
    source_bits: usize,
) -> Option<TranslocatorStop> {
    let gates = [r.read(1)? != 0, r.read(1)? != 0];
    event.other_reference_gates = Some(gates);
    if gates.iter().any(|v| *v) {
        return Some(TranslocatorStop::UnsupportedReferences);
    }
    let effect_present = r.read(1)? != 0;
    event.effect_present = Some(effect_present);
    if effect_present {
        event.effect = Some(r.read(32)? as u32);
    }
    match position(r, map, source_bits) {
        Ok(from) => event.from = Some(from),
        Err(stop) => return Some(stop),
    }
    match position(r, map, source_bits) {
        Ok(to) => event.to = Some(to),
        Err(stop) => return Some(stop),
    }
    Some(TranslocatorStop::PositionsRead)
}

fn position(
    r: &mut Cursor<'_>,
    map: Option<&FilmMapBounds>,
    source_bits: usize,
) -> Result<TeleportPosition, TranslocatorStop> {
    let start_bit = r.position;
    let mut axis_bits = [22; 3];
    let mut min = [-20000.0f32; 3];
    let mut max = [20000.0f32; 3];
    let region = if r.read(1).ok_or(TranslocatorStop::Truncated)? == 0 {
        let map = map.ok_or(TranslocatorStop::MissingMap)?;
        if map.region_index_bits.max(1) > 8
            || (0..3).any(|i| {
                !(1..=26).contains(&map.axis_widths[i])
                    || !map.min[i].is_finite()
                    || !map.max[i].is_finite()
                    || map.max[i] <= map.min[i]
            })
        {
            return Err(TranslocatorStop::InvalidMap);
        }
        let region = r
            .read(map.region_index_bits.max(1))
            .ok_or(TranslocatorStop::Truncated)? as u32;
        if region != map.region {
            return Err(TranslocatorStop::UnknownRegion(region));
        }
        axis_bits = map.axis_widths;
        min = map.min;
        max = map.max;
        Some(region)
    } else {
        None
    };
    let mut quantized = [0; 3];
    let mut world = [0.0; 3];
    for i in 0..3 {
        quantized[i] = r.read(axis_bits[i]).ok_or(TranslocatorStop::Truncated)? as u32;
        let step = (f64::from(max[i]) - f64::from(min[i])) / ((1u64 << axis_bits[i]) as f64);
        world[i] = (f64::from(min[i]) + step * (f64::from(quantized[i]) + 0.5)) as f32;
    }
    // Reference readTranslocVec consumes padded axes, then rejects the vector
    // when its logical end lies beyond the physical source.
    if r.position > source_bits {
        return Err(TranslocatorStop::Truncated);
    }
    Ok(TeleportPosition {
        start_bit,
        end_bit: r.position,
        region,
        axis_bits,
        quantized,
        world,
    })
}

pub(super) fn decode_translocator_head(
    data: &[u8],
    map: Option<&FilmMapBounds>,
) -> Option<TranslocatorRead> {
    let event = decode_translocator_with_cursor(data, map, Cursor::new_padded(data, 0))?;
    Some(TranslocatorRead {
        padded_bits: event.end_bit.saturating_sub(data.len() * 8),
        source_bits: data.len() * 8,
        event,
    })
}

#[cfg(test)]
impl TranslocatorEvent {
    /// A jump is usable only after both positions were read. A partial first
    /// position is retained for inspection but never supplied as a complete jump.
    pub fn positions(&self) -> Option<[[f32; 3]; 2]> {
        if self.stop != TranslocatorStop::PositionsRead {
            return None;
        }
        Some([self.from.as_ref()?.world, self.to.as_ref()?.world])
    }
}
