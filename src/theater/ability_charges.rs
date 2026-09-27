//! Equipment-energy publication. Unarmed slots carry no transmitted value.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityCharge {
    /// Native packet ordinal; unknown for unframed caller-supplied input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub source: FilmPacket,
    pub slot: u32,
    /// Energy-mask bit, not an equipment identity.
    pub emplacement: u8,
    pub charges: u8,
    pub low: u8,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AbilityChargeStats {
    pub records: usize,
    #[serde(rename = "WithI56")]
    pub with_component: usize,
    pub read: usize,
    pub unread: usize,
    pub armed: usize,
    pub absent: bool,
    pub scanned: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityChargeStream {
    /// Failed intermediate/target reads with raw fields, source and callbacks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_components: Vec<BipedChannelRejection>,
    /// Every attempted component in scan order, including intermediate fields and callbacks.
    /// Absence in old exports means this trace was unavailable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_attempts: Vec<super::BipedComponentAttempt>,
    pub records: Vec<AbilityCharge>,
    /// Includes mask-zero reads; no synthetic full-charge values are inserted.
    pub components: Vec<BipedChannelRead>,
    pub stats: AbilityChargeStats,
}

pub fn scan_ability_charges(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    encoding: &PositionEncoding,
) -> Result<AbilityChargeStream, DecodeError> {
    scan_ability_charges_impl(
        chunks,
        registry,
        positions,
        super::biped_channels::BipedComponentReader::Legacy(encoding),
    )
}

/// Native direct biped walk under a complete inherited reader context.
/// Traversal width overrides and corruption guards do not apply here.
pub fn scan_ability_charges_with_context(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    context: &super::FrameEncoding,
) -> Result<AbilityChargeStream, DecodeError> {
    scan_ability_charges_impl(
        chunks,
        registry,
        positions,
        super::biped_channels::BipedComponentReader::Context(context),
    )
}

fn scan_ability_charges_impl(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    reader: super::biped_channels::BipedComponentReader<'_>,
) -> Result<AbilityChargeStream, DecodeError> {
    let arch = registry
        .archetype(35)
        .ok_or(DecodeError::Missing("biped archetype"))?;
    let mut out = AbilityChargeStream::default();
    let Some(target) = super::ability_states::component_index(arch, "biped-spartan-ability-energy")
    else {
        out.stats.absent = true;
        out.stats.scanned = true;
        return Ok(out);
    };
    let packet_indices = super::fire_events::native_packet_indices(chunks);
    let bytes = super::fire_events::native_chunk_data(chunks);
    for candidate in &positions.candidates {
        // Native delta walking skips absent chunks before counting records.
        if !bytes.contains_key(&candidate.source.chunk_index) {
            continue;
        }
        out.stats.records += 1;
        let r = &candidate.record;
        if !r.component_indices.contains(&target) {
            continue;
        }
        out.stats.with_component += 1;
        let source = candidate.source;
        let data = bytes
            .get(&source.chunk_index)
            .and_then(|b| {
                b.get(
                    source.payload_offset
                        ..source.payload_offset.checked_add(source.payload_size)?,
                )
            })
            .ok_or(DecodeError::Truncated {
                chunk: source.chunk_index,
                offset: source.payload_offset,
            })?;
        let read = super::biped_channels::walk_biped_components_retaining_rejections(
            data,
            r,
            arch,
            reader,
            target,
            (
                source,
                packet_indices
                    .get(&(source.chunk_index, source.payload_offset))
                    .copied(),
            ),
            (&mut out.rejected_components, &mut out.component_attempts),
        )
        .into_iter()
        .find(|(id, _)| *id == target);
        let Some((_, component)) = read else {
            out.stats.unread += 1;
            continue;
        };
        let read = BipedChannelRead {
            packet_index: packet_indices
                .get(&(source.chunk_index, source.payload_offset))
                .copied(),
            source,
            slot: r.slot,
            record_start_bit: r.start_bit,
            component,
        };
        out.stats.read += 1;
        for i in 0..3 {
            if let Some(raw) = read.value(&format!("charge[{i}]")) {
                out.stats.armed += 1;
                out.records.push(AbilityCharge {
                    packet_index: read.packet_index,
                    source,
                    slot: r.slot,
                    emplacement: i,
                    charges: ((raw >> 4) & 15) as u8,
                    low: (raw & 15) as u8,
                });
            }
        }
        out.components.push(read);
    }
    out.records
        .sort_by_key(|r| (r.source.timestamp_us, r.slot, r.emplacement));
    out.stats.scanned = true;
    Ok(out)
}
