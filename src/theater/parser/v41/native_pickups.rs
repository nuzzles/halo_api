//! Native pickup attempts, including refusal points and synthetic zero-tail reads.
use super::{BipedPickupStats, bits::Cursor, event_heads};
pub(crate) use crate::theater::film::chunks::replication::native_pickups::{
    NativePickupOutcome, NativePickupRead,
};

/// Reproduce decodeBipedPickup, including its early returns. Direct decoding
/// updates only decoder counters; packet, published and off-band counts belong
/// to the scanner. The config bit is not an admission gate here.
pub(crate) fn decode_native_biped_pickup(
    data: &[u8],
    stats: &mut BipedPickupStats,
) -> NativePickupRead {
    let mut r = Cursor::new_padded(data, 0);
    let mut out = NativePickupRead {
        config: r.bit().unwrap(),
        more: r.bit().unwrap(),
        kind: r.read(7).unwrap() as u8,
        references: Vec::new(),
        class: None,
        catalog_id: None,
        more_events: None,
        source_bits: data.len() * 8,
        end_bit: 9,
        padded_bits: 0,
        outcome: NativePickupOutcome::EmptyList,
    };
    out.outcome = (|| {
        if !out.more {
            return NativePickupOutcome::EmptyList;
        }
        match out.kind {
            9 => stats.type_9 += 1,
            8 => {
                stats.type_8 += 1;
                return NativePickupOutcome::BoardVehicle;
            }
            _ => {
                stats.other_type += 1;
                return NativePickupOutcome::OtherType;
            }
        }
        out.references
            .push(event_heads::reference(&mut r, 2).unwrap());
        if out.references[0].value.is_none() {
            stats.refused_no_ref += 1;
            return NativePickupOutcome::NoReference;
        }
        for domain in [8, 7] {
            out.references
                .push(event_heads::reference(&mut r, domain).unwrap());
        }
        if out.references[1..].iter().any(|r| r.value.is_some()) {
            stats.unexpected_wide_ref += 1;
        }
        out.class = Some(r.read(3).unwrap() as u8);
        if !r.bit().unwrap() {
            stats.refused_no_catalog += 1;
            return NativePickupOutcome::NoCatalog;
        }
        out.catalog_id = Some(r.read(32).unwrap() as u32);
        out.more_events = Some(r.bit().unwrap());
        if out.more_events == Some(true) {
            stats.multi_event += 1;
        }
        NativePickupOutcome::Accepted
    })();
    out.end_bit = r.position;
    out.padded_bits = r.position.saturating_sub(out.source_bits);
    out
}

#[cfg(test)]
impl NativePickupRead {
    /// Native slot, which may include synthetic tail bits. This is not a player identity.
    pub fn slot(&self) -> Option<u32> {
        Some(512 + self.references.first()?.value.as_ref()?.index)
    }
}
