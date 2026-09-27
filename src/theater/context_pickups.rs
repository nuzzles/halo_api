//! Native pickup head scanning through the shared source context.
use super::*;

/// Native ScanBipedPickups. A missing biped band disables only the off-band
/// sentinel; it does not disable scanning. NoFilmChunks remains distinct from
/// an empty successful result. Attempts retain padded fields and refusal points.
pub fn scan_context_biped_pickups(context: &NativeFilmContext<'_>) -> NativePickupStream {
    let chunks = context.chunk_numbers();
    if chunks.is_empty() {
        return NativePickupStream {
            no_film_chunks: true,
            ..Default::default()
        };
    }
    let mut out = NativePickupStream {
        slot_band: context.biped_slot_band(),
        ..Default::default()
    };
    for &number in chunks {
        let Some((data, packets)) = context.chunk_at(number) else {
            continue;
        };
        for (packet_index, &source) in packets.iter().enumerate() {
            if source.packet_type != 0 || source.payload_size < 2 {
                continue;
            }
            let payload = &data[source.payload_offset..source.payload_offset + source.payload_size];
            if payload[0] != 0xc4 {
                continue;
            }
            out.stats.packets += 1;
            let read = decode_native_biped_pickup(payload, &mut out.stats);
            let mut published = read.outcome == NativePickupOutcome::Accepted;
            if published
                && out
                    .slot_band
                    .is_some_and(|[lo, hi]| !(lo..=hi).contains(&read.slot().unwrap()))
            {
                published = false;
                out.stats.refused_off_band += 1;
            }
            out.stats.published += usize::from(published);
            out.attempts.push(FilmNativePickupAttempt {
                source,
                packet_index,
                read,
                published,
            });
        }
    }
    out
}

/// Native cache projection includes published padded values. Consumers needing
/// recordedness must inspect the original attempts, which this does not alter.
pub fn facts_from_pickup_scan(scan: &NativePickupStream) -> (Vec<FactsPickup>, FactsPickupStats) {
    let records = scan
        .attempts
        .iter()
        .filter(|a| a.published)
        .map(|a| FactsPickup {
            timestamp_us: a.source.timestamp_us,
            slot: a.read.slot().expect("published unit reference"),
            catalog_id: a.read.catalog_id.expect("published catalog"),
            class: a.read.class.expect("published class"),
        })
        .collect();
    let s = &scan.stats;
    (
        records,
        FactsPickupStats {
            packets: s.packets as i64,
            type9: s.type_9 as i64,
            type8: s.type_8 as i64,
            other_type: s.other_type as i64,
            published: s.published as i64,
            multi_event: s.multi_event as i64,
            refused_no_ref: s.refused_no_ref as i64,
            refused_no_catalog: s.refused_no_catalog as i64,
            refused_off_band: s.refused_off_band as i64,
            unexpected_wide_ref: s.unexpected_wide_ref as i64,
        },
    )
}
