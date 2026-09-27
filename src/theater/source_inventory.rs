//! Native keyframe inventory pass over loaded buffers and a shared fallback count.
use super::*;
use std::collections::BTreeMap;

pub const DEFAULT_GRENADE_CAP_FALLBACK: &str = "repli_plafond_grenade_par_defaut";

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SourceInventoryScanError {
    #[error("aucun chunk film lisible")]
    NoReadableChunk,
}

/// Native ScanKeyframeInventory. An empty map returns before fallback or source
/// access; a nonempty all-false map still scans inventory records. The default
/// grenade cap is counted before source admission, even on error. Return scan
/// statistics alongside refusal rather than discarding the partial report.
/// Numbered chunks and packet ordinals retain native order; buffers are borrowed.
pub fn scan_source_keyframe_inventory(
    source: Option<&FilmSource>,
    known: &BTreeMap<u32, bool>,
    grenade_max: u32,
    fallbacks: Option<&FallbackCounter>,
) -> (KeyframeInventoryStream, Option<SourceInventoryScanError>) {
    let mut out = KeyframeInventoryStream::default();
    if known.is_empty() {
        return (out, None);
    }
    out.default_grenade_max = grenade_max == 0;
    out.grenade_max = if grenade_max == 0 {
        if let Some(counter) = fallbacks {
            counter.trigger(DEFAULT_GRENADE_CAP_FALLBACK);
        }
        2
    } else {
        grenade_max
    };
    let families = known.iter().filter_map(|(&k, &v)| v.then_some(k)).collect();
    let numbers = source.map_or_else(Vec::new, FilmSource::data_chunk_numbers);
    out.stats.chunks = numbers.len();
    for number in numbers {
        let Some((data, packets)) = source.and_then(|s| s.chunk_by_number(number)) else {
            out.stats.chunks_unread += 1;
            continue;
        };
        for (packet_index, packet) in packets.iter().enumerate() {
            if packet.packet_type != 2 {
                continue;
            }
            out.stats.keyframes += 1;
            let payload = &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            for mut inv in decode_keyframe_inventories(payload, &families, out.grenade_max) {
                inv.timestamp_us = packet.timestamp_us;
                inv.chunk = number;
                inv.packet_index = packet_index;
                if inv.grenades_read {
                    if inv.grenades_by_position {
                        out.stats.grenades_by_position += 1;
                    } else {
                        out.stats.grenades_by_anchor += 1;
                    }
                }
                out.records.push(inv);
            }
        }
    }
    out.stats.records = out.records.len();
    let error = (out.stats.chunks_unread == out.stats.chunks)
        .then_some(SourceInventoryScanError::NoReadableChunk);
    (out, error)
}

/// Project only native cache fields. Source locations, grenade-read method and
/// gauge quanta remain available in KeyframeInventory; cache encoding omits them.
impl From<&KeyframeInventory> for FactsKeyframeInventory {
    fn from(v: &KeyframeInventory) -> Self {
        Self {
            timestamp_us: v.timestamp_us,
            slot: v.slot,
            grenades_read: v.grenades_read,
            grenades: v.grenades,
            selected_grenade_rank: i64::from(v.selected_grenade_rank),
            ability_rank: i64::from(v.ability_rank),
            drawn_slot: i64::from(v.drawn_slot),
            ammo_candidates: v.ammo_candidates as i64,
            ammo_read: v.ammo_read,
            ammo: v.ammo.clone(),
        }
    }
}
