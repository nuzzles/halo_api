//! Borrowed delta-record traversal for native channel scans.
use super::*;

/// An anchored record, before position filtering or component interpretation.
/// `source.chunk_index` identifies the loaded buffer; `chunk` is the requested
/// native file number. The complete packet payload remains borrowed.
#[derive(Debug)]
pub struct ContextDeltaBipedRecord<'a> {
    pub payload: &'a [u8],
    pub chunk: i64,
    pub source: FilmPacket,
    pub packet_index: usize,
    pub start_bit: usize,
    pub position_bit: usize,
    pub position_end_bit: usize,
    pub slot: u32,
    pub generation: u8,
    pub component_indices: Vec<u8>,
}

/// Native walkDeltaBipedRecords over the exact supplied chunk order. Empty
/// selection means no visits; callers choose whether to pass chunk_numbers().
/// Missing chunks are skipped, duplicate selections revisited, and only delta
/// packets are admitted. This emits no observer hooks and applies no saturation,
/// speed, isolation or direction filters. Recognized records advance past i0;
/// rejected anchors advance one bit.
///
/// Malformed layouts outside this bounded reader's domain return an explicit
/// error. This helper does not reproduce native panics/nonprogress for those
/// layouts. Axis widths are not used to decode or reinterpret position values.
pub fn walk_context_delta_bipeds(
    context: &NativeFilmContext<'_>,
    chunks: &[i64],
    slots: &FilmSlotBand,
    layout: &I0Layout,
    mut visit: impl FnMut(ContextDeltaBipedRecord<'_>),
) -> Result<(), DecodeError> {
    let vector_bits = usize::try_from(layout.total_bits())
        .ok()
        .filter(|&n| n >= layout.gate_bits.max(4) as usize)
        .ok_or_else(|| DecodeError::Inconsistent("invalid delta biped layout width".into()))?;
    let minimum = vector_bits
        .checked_add(33)
        .ok_or_else(|| DecodeError::Inconsistent("delta biped record width overflow".into()))?;
    if !(4..=36).contains(&layout.gate_bits) {
        return Err(DecodeError::Inconsistent(
            "invalid delta biped gate width".into(),
        ));
    }
    for &chunk in chunks {
        let Some((data, packets)) = context.chunk_at(chunk) else {
            continue;
        };
        for (packet_index, &source) in packets.iter().enumerate() {
            if source.packet_type != 0 {
                continue;
            }
            let payload = &data[source.payload_offset..source.payload_offset + source.payload_size];
            let bits = super::bits::Bits(payload);
            let mut at = 0usize;
            while at.checked_add(minimum).is_some_and(|end| end <= bits.len()) {
                let header = super::biped_scan::match_biped_header_raw(
                    bits,
                    at,
                    [0, 8191],
                    true,
                    vector_bits,
                )
                .filter(|h| slots.contains(h.slot))
                .filter(|h| {
                    bits.read(h.position_bit, 4) == Some(0)
                        && bits.read(h.position_bit + 4, layout.gate_bits - 4)
                            == Some(u64::from(layout.region))
                });
                let Some(h) = header else {
                    at += 1;
                    continue;
                };
                let end = h.position_bit + vector_bits;
                visit(ContextDeltaBipedRecord {
                    payload,
                    chunk,
                    source,
                    packet_index,
                    start_bit: at,
                    position_bit: h.position_bit,
                    position_end_bit: end,
                    slot: h.slot,
                    generation: h.generation,
                    component_indices: h.indices,
                });
                at = end;
            }
        }
    }
    Ok(())
}

/// Native walkRecordTo with retained attempts. Hook publication precedes the
/// dispatch/bounds verdict; callers decide whether reaching the target is needed.
pub(super) fn walk_context_record_to(
    record: &ContextDeltaBipedRecord<'_>,
    arch: &FilmArchetype,
    context: &NativeReaderContext,
    target: u8,
    attempts: &mut Vec<BipedComponentAttempt>,
) -> Result<bool, NativeReaderProfileError> {
    let mut at = (record.position_end_bit as i64).wrapping_add(2);
    for &id in record.component_indices.iter().skip(1) {
        let Some(name) = arch
            .components
            .get(usize::from(id))
            .filter(|n| !n.is_empty())
        else {
            break;
        };
        let mut reader = NativeFilmReader::with_context(record.payload, context.clone());
        reader.set_native_bit_position(at);
        let (status, component) = reader.read_component(
            name,
            arch.levels.get(usize::from(id)).copied().unwrap_or(0),
            35,
        )?;
        let in_bounds =
            component.end_bit >= 0 && component.end_bit <= (record.payload.len() * 8) as i64;
        at = component.end_bit;
        attempts.push(BipedComponentAttempt {
            component_index: id,
            status,
            in_bounds,
            read: BipedChannelRead {
                packet_index: Some(record.packet_index),
                source: record.source,
                slot: record.slot,
                record_start_bit: record.start_bit,
                component,
            },
        });
        if status != Some(true) || !in_bounds {
            break;
        }
        if id == target {
            return Ok(true);
        }
    }
    Ok(false)
}
