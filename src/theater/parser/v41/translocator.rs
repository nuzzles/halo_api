//! Type-117 teleport events from LevelUp's transloc_events.go.
//! See docs/CREDIT.md. Failed position reads retain the known event.
use super::{FilmMapBounds, bits::Cursor};
pub(crate) use crate::theater::film::chunks::replication::replication_stream::models::translocator::{
    NativeTranslocatorEvent, TeleportPosition, TranslocatorEvent, TranslocatorStop,
};

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
    // Native readTranslocVec consumes padded axes, then rejects the vector
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

pub(crate) fn decode_native_translocator_head(
    data: &[u8],
    map: Option<&FilmMapBounds>,
) -> Option<NativeTranslocatorEvent> {
    let event = decode_translocator_with_cursor(data, map, Cursor::new_padded(data, 0))?;
    Some(NativeTranslocatorEvent {
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
