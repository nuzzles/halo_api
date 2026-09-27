//! Event scans over already-loaded buffers, without legacy chunk copies.
use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFireEvent {
    /// Source buffer position and payload range; event.chunk is the file number.
    pub source: FilmPacket,
    pub event: FilmFireEvent,
}

/// Native ScanFireEvents: contiguous numbered prefix, all-packet ordinals,
/// grammar-based type-36 admission, no timestamp sort. Readable empty chunks succeed.
pub fn scan_source_fire_events(
    source: Option<&FilmSource>,
) -> Result<Vec<SourceFireEvent>, DecodeError> {
    let source = source.ok_or(DecodeError::Missing("readable film chunks"))?;
    let mut out = Vec::new();
    let mut read = false;
    for number in source.data_chunk_numbers() {
        let Some((data, packets)) = source.chunk_by_number(number) else {
            continue;
        };
        read = true;
        for (packet_index, &packet) in packets.iter().enumerate() {
            if packet.packet_type != 0 {
                continue;
            }
            let payload = &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            if let Some(mut event) = decode_fire_event(payload) {
                event.chunk = number;
                event.packet_index = packet_index;
                event.timestamp_us = packet.timestamp_us;
                out.push(SourceFireEvent {
                    source: packet,
                    event,
                });
            }
        }
    }
    if !read {
        return Err(DecodeError::Missing("readable film chunks"));
    }
    Ok(out)
}

/// Native ScanTranslocatorTeleports with retained source ranges and padding
/// diagnostics. Missing coordinates do not discard an admitted event. Equal-time
/// ordering uses the pinned native sort. Source packet identities are buffer
/// positions, not file numbers; use FilmSource metadata to resolve the latter.
pub fn scan_source_translocator_events(
    source: Option<&FilmSource>,
    map: Option<&FilmMapBounds>,
) -> Vec<FilmNativeTranslocatorEvent> {
    let mut out = Vec::new();
    let Some(source) = source else {
        return out;
    };
    for number in source.data_chunk_numbers() {
        let Some((data, packets)) = source.chunk_by_number(number) else {
            continue;
        };
        for (packet_index, &packet) in packets.iter().enumerate() {
            if packet.packet_type != 0 || packet.payload_size < 2 {
                continue;
            }
            let payload = &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            if payload[0] != 0xfa {
                continue;
            }
            if let Some(read) = decode_native_translocator_head(payload, map) {
                out.push(FilmNativeTranslocatorEvent {
                    source: packet,
                    packet_index,
                    read,
                });
            }
        }
    }
    super::native_sort::sort_by(&mut out, |a, b| {
        a.source.timestamp_us.cmp(&b.source.timestamp_us)
    });
    out
}

/// Native cache retains the recorded five-bit FilmIndex, including -1 when absent.
/// Full event fields and source references remain available in SourceFireEvent.
impl From<&FilmFireEvent> for FactsFireEvent {
    fn from(e: &FilmFireEvent) -> Self {
        Self {
            timestamp_us: e.timestamp_us,
            film_index: i64::from(e.film_index),
            weapon_id: e.weapon_id,
            has_aim: e.has_aim,
            aim: e.aim,
        }
    }
}

/// Native facts publish coordinates only when both vectors were read. Partial
/// vectors and padding diagnostics remain in the original source event.
impl From<&FilmNativeTranslocatorEvent> for FactsTranslocation {
    fn from(e: &FilmNativeTranslocatorEvent) -> Self {
        let positions = e.read.event.positions();
        let [from, to] = positions.unwrap_or_default();
        Self {
            timestamp_us: e.source.timestamp_us,
            slot: e.read.event.slot,
            has_positions: positions.is_some(),
            from,
            to,
        }
    }
}
