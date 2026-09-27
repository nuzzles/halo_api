//! Native world-phase event heads over loaded source buffers.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceZoomEvent {
    pub chunk: i64,
    pub event: FilmNativeZoomEvent,
}
/// Native ScanZoomEvents: absent sources degrade to an empty result, and ties
/// follow the pinned native timestamp sort. Padded reference/level values retain
/// their recorded-versus-padded diagnostics in the original NativeZoomRead.
pub fn scan_source_zoom_events(source: Option<&FilmSource>) -> Vec<SourceZoomEvent> {
    let mut out = Vec::new();
    let Some(source) = source else {
        return out;
    };
    for chunk in source.data_chunk_numbers() {
        let Some((data, packets)) = source.chunk_by_number(chunk) else {
            continue;
        };
        for (packet_index, &packet) in packets.iter().enumerate() {
            if packet.packet_type != 0 || packet.payload_size < 2 {
                continue;
            }
            let payload = &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            if payload[0] != 0xca {
                continue;
            }
            if let Some(read) = decode_native_zoom_head(payload) {
                out.push(SourceZoomEvent {
                    chunk,
                    event: FilmNativeZoomEvent {
                        source: packet,
                        packet_index,
                        read,
                    },
                });
            }
        }
    }
    super::native_sort::sort_by(&mut out, |a, b| {
        a.event
            .source
            .timestamp_us
            .cmp(&b.event.source.timestamp_us)
    });
    out
}
impl From<&SourceZoomEvent> for FactsZoomEvent {
    fn from(r: &SourceZoomEvent) -> Self {
        Self {
            timestamp_us: r.event.source.timestamp_us,
            slot: r.event.read.slot,
            level: i64::from(r.event.read.level),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpawnEvent {
    pub chunk: i64,
    pub event: EquipmentSpawnEvent,
    /// All three reference values and their source bit boundaries. The third
    /// reference is retained even though the native DTO only publishes its gate.
    pub head: DecodedHeadEvent,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceSpawnScan {
    pub records: Vec<SourceSpawnEvent>,
    pub stats: EquipmentSpawnStats,
    /// Truncated type-103 heads are explicit refusals rather than native raw
    /// indexing panics. Source bytes remain in the loaded film.
    pub rejected_heads: Vec<FilmHeadEvent>,
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SourceSpawnScanError {
    #[error("aucun chunk de donnees dans le film")]
    NoChunks,
    #[error("aucun chunk film lisible")]
    NoReadableChunks,
}
/// Native ScanEquipmentSpawnEvents. Scans only the first event in each list,
/// preserves caller file ordering and all-packet ordinals, and does not infer
/// deployment from the source/spawned identities.
pub fn scan_context_equipment_spawn_events(
    context: &NativeFilmContext<'_>,
) -> (SourceSpawnScan, Option<SourceSpawnScanError>) {
    let mut out = SourceSpawnScan::default();
    let numbers = context.chunk_numbers();
    if numbers.is_empty() {
        return (out, Some(SourceSpawnScanError::NoChunks));
    }
    for &chunk in numbers {
        let Some((data, packets)) = context.chunk_at(chunk) else {
            continue;
        };
        out.stats.chunks += 1;
        for (packet_index, &packet) in packets.iter().enumerate() {
            if packet.packet_type != 0 || packet.payload_size == 0 {
                continue;
            }
            let payload = &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            out.stats.packets += 1;
            out.stats.lists += usize::from(payload[0] & 0x40 != 0);
            let Some(head) = decode_packet_head_event(payload).filter(|h| h.kind == 103) else {
                continue;
            };
            if head.references.len() != 3 {
                out.stats.truncated += 1;
                out.rejected_heads.push(FilmHeadEvent {
                    source: packet,
                    event: head,
                });
                continue;
            }
            let life = |i: usize| {
                head.references[i].value.as_ref().map(|r| ObjectLife {
                    slot: r.index + 512,
                    generation: r.generation,
                })
            };
            let event = EquipmentSpawnEvent {
                packet_index: Some(packet_index),
                packet,
                source: life(0),
                spawned: life(1),
                reference_2_present: head.references[2].value.is_some(),
            };
            out.stats.events += 1;
            out.stats.with_source += usize::from(event.source.is_some());
            out.stats.with_spawned += usize::from(event.spawned.is_some());
            out.stats.reference_2 += usize::from(event.reference_2_present);
            out.records.push(SourceSpawnEvent { chunk, event, head });
        }
    }
    let error = (out.stats.chunks == 0).then_some(SourceSpawnScanError::NoReadableChunks);
    (out, error)
}
impl From<&SourceSpawnEvent> for FactsSpawnEvent {
    fn from(r: &SourceSpawnEvent) -> Self {
        let e = &r.event;
        let key = |v: Option<ObjectLife>| {
            v.map_or_else(FactsLifeKey::default, |v| FactsLifeKey {
                slot: v.slot,
                generation: u32::from(v.generation),
            })
        };
        Self {
            timestamp_us: e.packet.timestamp_us,
            chunk: r.chunk,
            packet_index: e.packet_index.unwrap() as i64,
            spawned: key(e.spawned),
            spawned_valid: e.spawned.is_some(),
            source: key(e.source),
            source_valid: e.source.is_some(),
            ref2_present: e.reference_2_present,
        }
    }
}
impl From<&EquipmentSpawnStats> for FactsSpawnStats {
    fn from(s: &EquipmentSpawnStats) -> Self {
        Self {
            chunks: s.chunks as i64,
            packets: s.packets as i64,
            lists: s.lists as i64,
            events: s.events as i64,
            with_spawned: s.with_spawned as i64,
            with_source: s.with_source as i64,
            ref2: s.reference_2 as i64,
        }
    }
}
