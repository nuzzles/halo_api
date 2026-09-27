//! Reference biped position scan and its explicit acceptance filters.
//! These are guarded bit-scan observations, not proof of a complete entity record.
use super::{
    DecodeError, FilmMapBounds, FilmPacket, FilmTranslocatorEvent, bits::Bits, packets,
    recover_keyframe_anchors,
};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Native compact component mask plus overflow flag. Negative indices and
/// indices >=64 set overflow and contribute no bits. Keep the original index
/// sequence alongside this projection to preserve ordering and out-of-range data.
pub fn native_component_mask(indices: impl IntoIterator<Item = i64>) -> (u64, bool) {
    let mut mask = 0;
    let mut overflow = false;
    for index in indices {
        if (0..64).contains(&index) {
            mask |= 1_u64 << index;
        } else {
            overflow = true;
        }
    }
    (mask, overflow)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BipedScanOptions {
    /// Publish component masks and decode direction/vitality companions.
    #[serde(default = "capture_dirs_default")]
    pub capture_dirs: bool,
    pub require_generation_one: bool,
    pub drop_saturated: bool,
    /// Convenience threshold; zero disables isolation unless a native override is set.
    pub isolation_gap_us: u64,
    /// Raw native IsolationGapMS. When present, overrides isolation_gap_us.
    /// Nonpositive values disable filtering; positive conversion wraps in u64.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_isolation_gap_ms: Option<i64>,
    pub max_speed: f64,
}
fn capture_dirs_default() -> bool {
    true
}
impl Default for BipedScanOptions {
    fn default() -> Self {
        Self {
            capture_dirs: true,
            require_generation_one: true,
            drop_saturated: true,
            isolation_gap_us: 15_000_000,
            native_isolation_gap_ms: None,
            max_speed: 100.0,
        }
    }
}
impl BipedScanOptions {
    /// None disables the filter. Some(0) is an active zero-gap filter, which
    /// can result from native positive-millisecond conversion overflow.
    pub fn isolation_threshold_us(&self) -> Option<u64> {
        match self.native_isolation_gap_ms {
            Some(ms) => (ms > 0).then(|| (ms as u64).wrapping_mul(1000)),
            None => (self.isolation_gap_us > 0).then_some(self.isolation_gap_us),
        }
    }
    /// Scalar defaults from native DefaultScanFilmOptions. Unlike the historical
    /// Rust convenience default, companion capture is disabled.
    pub fn native_defaults() -> Self {
        Self {
            capture_dirs: false,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BipedPositionRecord {
    pub start_bit: usize,
    pub position_bit: usize,
    /// End of the vector, before any position tail or later component.
    pub end_bit: usize,
    pub slot: u32,
    pub generation: u8,
    pub component_indices: Vec<u8>,
    pub quantized: [u32; 3],
    pub world: [f32; 3],
    #[serde(default)]
    pub companions: super::BipedCompanions,
}
/// Absolute-position wire observation without world-coordinate interpretation.
/// Map bounds are deliberately absent. Companion fields retain their native units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantizedBipedPositionRecord {
    pub start_bit: usize,
    pub position_bit: usize,
    pub end_bit: usize,
    pub slot: u32,
    pub generation: u8,
    pub component_indices: Vec<u8>,
    pub quantized: [u32; 3],
    pub companions: super::BipedCompanions,
}

/// Native ScanBipedRecords without WorldRange. Spatial speed filters do not
/// apply; this payload-level API has no timestamps for isolation filtering.
/// The supplied wire layout is required, but no map bounds are fabricated.
pub fn scan_quantized_biped_position_records(
    data: &[u8],
    band: [u32; 2],
    layout: &super::I0Layout,
    options: &BipedScanOptions,
    dynamic_forward_level: Option<u32>,
) -> Result<Vec<QuantizedBipedPositionRecord>, DecodeError> {
    scan_quantized_biped_position_records_observed(
        data,
        band,
        layout,
        options,
        dynamic_forward_level,
        None,
    )
}

/// Native payload scan with live RecordMaskHook delivery after companion decoding.
/// Saturated records are excluded before publication; temporal filters belong to
/// source-level scans. The hook borrows the original complete packet payload.
pub fn scan_quantized_biped_position_records_observed(
    data: &[u8],
    band: [u32; 2],
    layout: &super::I0Layout,
    options: &BipedScanOptions,
    dynamic_forward_level: Option<u32>,
    observer: Option<&super::NativeFilmObserver>,
) -> Result<Vec<QuantizedBipedPositionRecord>, DecodeError> {
    scan_quantized_records_contextual(
        data,
        band,
        None,
        layout,
        options.require_generation_one,
        dynamic_forward_level,
        BipedCapturePolicy {
            directions: options.capture_dirs,
            drop_saturated: options.drop_saturated,
            observer,
        },
    )
}

pub(crate) fn scan_quantized_band_observed(
    data: &[u8],
    allowed: &BTreeSet<u32>,
    layout: &super::I0Layout,
    options: &BipedScanOptions,
    dynamic_forward_level: Option<u32>,
    observer: Option<&super::NativeFilmObserver>,
) -> Result<Vec<QuantizedBipedPositionRecord>, DecodeError> {
    scan_quantized_records_contextual(
        data,
        [0, 8191],
        Some(allowed),
        layout,
        options.require_generation_one,
        dynamic_forward_level,
        BipedCapturePolicy {
            directions: options.capture_dirs,
            drop_saturated: options.drop_saturated,
            observer,
        },
    )
}

struct BipedCapturePolicy<'a> {
    directions: bool,
    drop_saturated: bool,
    observer: Option<&'a super::NativeFilmObserver>,
}

impl BipedPositionRecord {
    /// Native MaskBits/MaskOver projection, retaining the complete source list.
    pub fn compact_component_mask(&self) -> (u64, bool) {
        native_component_mask(self.component_indices.iter().map(|&i| i64::from(i)))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BipedPositionRejection {
    Saturated,
    Isolated,
    ExcessiveSpeed,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BipedPositionCandidate {
    /// Ordinal among all packet types in the source chunk. None marks legacy
    /// exports or caller-created observations without framed source data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub source: FilmPacket,
    pub record: BipedPositionRecord,
    pub rejection: Option<BipedPositionRejection>,
}
/// Native RecordMaskHook publication. The source packet identifies the original
/// payload; `after_position_bit` is relative to that payload, before any i0 tail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedRecordMaskObservation {
    pub source: FilmPacket,
    pub component_indices: Vec<u8>,
    pub after_position_bit: usize,
}
impl BipedRecordMaskObservation {
    /// Native compact projection of this callback's complete component list.
    pub fn compact_component_mask(&self) -> (u64, bool) {
        native_component_mask(self.component_indices.iter().map(|&i| i64::from(i)))
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BipedPositionStream {
    /// Ordered callbacks after saturation rejection, before isolation/speed filters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub record_masks: Vec<BipedRecordMaskObservation>,
    /// Contiguous min/max of ti=35 anchors in each chunk's first keyframe, as in Go.
    /// None means no biped slot band was established; no positions are invented.
    pub slot_band: Option<[u32; 2]>,
    pub options: BipedScanOptions,
    /// Every matching position is retained, including those rejected by filtering.
    pub candidates: Vec<BipedPositionCandidate>,
}
impl BipedPositionStream {
    pub fn accepted(&self) -> impl Iterator<Item = &BipedPositionCandidate> {
        self.candidates.iter().filter(|c| c.rejection.is_none())
    }
}
fn validate_map(map: &FilmMapBounds) -> Result<(), DecodeError> {
    let index_bits = map.region_index_bits.max(1);
    if index_bits > 8
        || map.region >= 1 << index_bits
        || (0..3).any(|i| {
            !(1..=26).contains(&map.axis_widths[i])
                || !map.min[i].is_finite()
                || !map.max[i].is_finite()
                || map.max[i] <= map.min[i]
        })
    {
        return Err(DecodeError::Inconsistent(
            "invalid biped position map context".into(),
        ));
    }
    Ok(())
}
/// Port of matchBipedHeader and walkDeltaBipedPayload. Advances only through the
/// established i0 vector; other components and record boundaries are not guessed.
pub fn scan_biped_position_records(
    data: &[u8],
    band: [u32; 2],
    map: &FilmMapBounds,
    require_generation_one: bool,
) -> Result<Vec<BipedPositionRecord>, DecodeError> {
    scan_position_records_inner(data, band, None, map, require_generation_one, None, true)
}
fn scan_position_records_inner(
    data: &[u8],
    band: [u32; 2],
    allowed: Option<&BTreeSet<u32>>,
    map: &FilmMapBounds,
    require_generation_one: bool,
    dynamic_forward_level: Option<u32>,
    capture_dirs: bool,
) -> Result<Vec<BipedPositionRecord>, DecodeError> {
    validate_map(map)?;
    let layout = super::I0Layout {
        gate_bits: (4 + map.region_index_bits.max(1)) as i64,
        axis_widths: map.axis_widths.map(|w| w as u64),
        region: map.region,
    };
    Ok(scan_quantized_records_inner(
        data,
        band,
        allowed,
        &layout,
        require_generation_one,
        dynamic_forward_level,
        capture_dirs,
    )?
    .into_iter()
    .map(|r| {
        let world = std::array::from_fn(|i| {
            let step = (f64::from(map.max[i]) - f64::from(map.min[i]))
                / ((1u64 << map.axis_widths[i]) as f64);
            (f64::from(map.min[i]) + step * (f64::from(r.quantized[i]) + 0.5)) as f32
        });
        BipedPositionRecord {
            start_bit: r.start_bit,
            position_bit: r.position_bit,
            end_bit: r.end_bit,
            slot: r.slot,
            generation: r.generation,
            component_indices: r.component_indices,
            quantized: r.quantized,
            companions: r.companions,
            world,
        }
    })
    .collect())
}
pub(crate) fn scan_quantized_records_inner(
    data: &[u8],
    band: [u32; 2],
    allowed: Option<&BTreeSet<u32>>,
    layout: &super::I0Layout,
    require_generation_one: bool,
    dynamic_forward_level: Option<u32>,
    capture_dirs: bool,
) -> Result<Vec<QuantizedBipedPositionRecord>, DecodeError> {
    scan_quantized_records_contextual(
        data,
        band,
        allowed,
        layout,
        require_generation_one,
        dynamic_forward_level,
        BipedCapturePolicy {
            directions: capture_dirs,
            drop_saturated: false,
            observer: None,
        },
    )
}

fn scan_quantized_records_contextual(
    data: &[u8],
    band: [u32; 2],
    allowed: Option<&BTreeSet<u32>>,
    layout: &super::I0Layout,
    require_generation_one: bool,
    dynamic_forward_level: Option<u32>,
    capture: BipedCapturePolicy<'_>,
) -> Result<Vec<QuantizedBipedPositionRecord>, DecodeError> {
    if !(4..=36).contains(&layout.gate_bits)
        || layout.axis_widths.iter().any(|w| !(1..=26).contains(w))
        || u64::from(layout.region) >= (1u64 << (layout.gate_bits - 4))
    {
        return Err(DecodeError::Inconsistent(
            "invalid biped wire layout".into(),
        ));
    }
    if band[0] > band[1] || band[1] >= 8192 {
        return Err(DecodeError::Inconsistent("invalid biped slot band".into()));
    }
    let allowed = allowed.map(super::FilmSlotBand::from_set).transpose()?;
    let bits = Bits(data);
    let gate = layout.gate_bits as usize;
    let vector_bits = gate + layout.axis_widths.iter().sum::<u64>() as usize;
    let minimum = 21 + 12 + vector_bits;
    let mut out = Vec::new();
    let mut at: usize = 0;
    while at.saturating_add(minimum) <= bits.len() {
        if allowed.as_ref().is_some_and(|slots| {
            bits.read(at + 1, 13)
                .is_none_or(|s| !slots.contains(s as u32))
        }) {
            at += 1;
            continue;
        }
        let Some(mut record) = match_position(
            bits,
            at,
            band,
            layout,
            gate,
            vector_bits,
            require_generation_one,
        ) else {
            at += 1;
            continue;
        };
        if capture.drop_saturated
            && record
                .quantized
                .iter()
                .zip(layout.axis_widths)
                .any(|(&q, w)| q == 0 || u64::from(q) == (1u64 << w) - 1)
        {
            at = record.end_bit;
            continue;
        }
        if capture.directions {
            record.companions = super::scan_biped_companions_with_grammar(
                data,
                record.end_bit,
                &record.component_indices,
                dynamic_forward_level,
                dynamic_forward_level.is_some(),
            );
        }
        if capture.directions
            && let Some(observer) = capture.observer
        {
            let indices: Vec<_> = record
                .component_indices
                .iter()
                .map(|&i| usize::from(i))
                .collect();
            observer.publish(super::NativeHookPublication::RecordMask {
                indices: &indices,
                payload: data,
                after_i0: record.end_bit,
            });
        }
        at = record.end_bit;
        out.push(record);
    }
    Ok(out)
}
pub(super) struct RawBipedHeader {
    pub slot: u32,
    pub generation: u8,
    pub position_bit: usize,
    pub indices: Vec<u8>,
}
pub(super) fn match_biped_header_raw(
    bits: Bits<'_>,
    at: usize,
    band: [u32; 2],
    require_generation_one: bool,
    vector_bits: usize,
) -> Option<RawBipedHeader> {
    if bits.read(at, 1)? != 1 {
        return None;
    }
    let slot = bits.read(at + 1, 13)? as u32;
    if slot < band[0] || slot > band[1] {
        return None;
    }
    let generation = bits.read(at + 14, 2)? as u8;
    if require_generation_one && generation != 1 || bits.read(at + 16, 2)? != 0 {
        return None;
    }
    let count = bits.read(at + 18, 3)? as usize;
    if !(2..=7).contains(&count) {
        return None;
    }
    let position_bit = at + 21 + 6 * count;
    let end_bit = position_bit + vector_bits;
    if end_bit > bits.len() {
        return None;
    }
    let mut indices = Vec::with_capacity(count);
    for i in 0..count {
        let index = bits.read(at + 21 + 6 * i, 6)? as u8;
        if i == 0 && index != 0 || indices.last().is_some_and(|previous| *previous >= index) {
            return None;
        }
        indices.push(index);
    }
    Some(RawBipedHeader {
        slot,
        generation,
        position_bit,
        indices,
    })
}
fn match_position(
    bits: Bits<'_>,
    at: usize,
    band: [u32; 2],
    layout: &super::I0Layout,
    gate: usize,
    vector_bits: usize,
    require_generation_one: bool,
) -> Option<QuantizedBipedPositionRecord> {
    let header = match_biped_header_raw(bits, at, band, require_generation_one, vector_bits)?;
    let RawBipedHeader {
        slot,
        generation,
        position_bit,
        indices,
    } = header;
    let end_bit = position_bit + vector_bits;
    if bits.read(position_bit, 4)? != 0
        || bits.read(position_bit + 4, gate - 4)? != u64::from(layout.region)
    {
        return None;
    }
    let mut quantized = [0; 3];
    let mut pos = position_bit + gate;
    for (value, width) in quantized.iter_mut().zip(layout.axis_widths) {
        *value = bits.read(pos, width as usize)? as u32;
        pos += width as usize;
    }
    Some(QuantizedBipedPositionRecord {
        start_bit: at,
        position_bit,
        end_bit,
        slot,
        generation,
        companions: Default::default(),
        component_indices: indices,
        quantized,
    })
}

pub(crate) fn biped_slot_band(chunks: &[FilmChunkData]) -> Result<Option<[u32; 2]>, DecodeError> {
    // Native uses the contiguous data prefix plus the following chunk, whose
    // first keyframe can contain bipeds created during the preceding chunk.
    let selected = super::fire_events::native_chunk_prefix(chunks).unwrap_or_default();
    let Some(last) = selected.last() else {
        return Ok(None);
    };
    let next = last
        .metadata
        .index
        .checked_add(1)
        .and_then(|index| chunks.iter().find(|c| c.metadata.index == index));
    let mut band: Option<[u32; 2]> = None;
    for chunk in selected.into_iter().chain(next) {
        let Some(packet) = super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .find(|p| p.packet_type == 2)
        else {
            continue;
        };
        let payload =
            &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size];
        for anchor in recover_keyframe_anchors(payload)
            .into_iter()
            .filter(|a| a.archetype == 35)
        {
            let slot = anchor.id & 0x3fff_ffff;
            band = Some(band.map_or([slot, slot], |[min, max]| [min.min(slot), max.max(slot)]));
        }
    }
    Ok(band)
}

/// Scan with the reference's band, saturation, isolation and speed rules. Teleport
/// exemptions use attributed type-117 events even when their positions are absent.
/// Companion capture follows the reference biped direction/vitality stopping rules.
pub fn scan_biped_positions(
    chunks: &[FilmChunkData],
    map: &FilmMapBounds,
    options: BipedScanOptions,
    teleports: &[FilmTranslocatorEvent],
) -> Result<BipedPositionStream, DecodeError> {
    validate_map(map)?;
    if !options.max_speed.is_finite() {
        return Err(DecodeError::Inconsistent(
            "invalid biped speed limit".into(),
        ));
    }
    let packets = packets::index(chunks)?;
    let bytes: BTreeMap<_, _> = chunks.iter().map(|c| (c.metadata.index, &c.data)).collect();
    let band = biped_slot_band(chunks)?;
    let mut stream = BipedPositionStream {
        record_masks: vec![],
        slot_band: band,
        options,
        candidates: Vec::new(),
    };
    let Some(band) = band else { return Ok(stream) };
    let mut ordinals = BTreeMap::<i32, usize>::new();
    for source in packets {
        let next = ordinals.entry(source.chunk_index).or_default();
        let packet_index = *next;
        *next += 1;
        if source.packet_type != 0 {
            continue;
        }
        let data = &bytes[&source.chunk_index]
            [source.payload_offset..source.payload_offset + source.payload_size];
        for record in scan_position_records_inner(
            data,
            band,
            None,
            map,
            stream.options.require_generation_one,
            None,
            stream.options.capture_dirs,
        )? {
            let saturated = record
                .quantized
                .iter()
                .zip(map.axis_widths)
                .any(|(&q, w)| q == 0 || q == (1u32 << w) - 1);
            let rejection = (stream.options.drop_saturated && saturated)
                .then_some(BipedPositionRejection::Saturated);
            if stream.options.capture_dirs && rejection.is_none() {
                stream.record_masks.push(BipedRecordMaskObservation {
                    source,
                    component_indices: record.component_indices.clone(),
                    after_position_bit: record.end_bit,
                });
            }
            stream.candidates.push(BipedPositionCandidate {
                packet_index: Some(packet_index),
                source,
                record,
                rejection,
            });
        }
    }
    filter_positions(&mut stream, teleports);
    Ok(stream)
}
/// Scan an exact world-object slot band with the anchored biped position grammar.
/// Map precision is explicit. Dynamic orientation uses the registry's i2 level;
/// vehicle callers pass Some(2) for the pinned v41 profile. Unlike the biped entry
/// point, packet traversal follows the native readable chunk prefix.
pub fn scan_positions_for_band(
    chunks: &[FilmChunkData],
    band: &BTreeSet<u32>,
    map: &FilmMapBounds,
    options: BipedScanOptions,
    dynamic_forward_level: Option<u32>,
    teleports: &[FilmTranslocatorEvent],
) -> Result<BipedPositionStream, DecodeError> {
    validate_map(map)?;
    if !options.max_speed.is_finite() {
        return Err(DecodeError::Inconsistent(
            "invalid biped speed limit".into(),
        ));
    }
    let chunks = super::fire_events::native_chunk_prefix(chunks)?;
    let (&lo, &hi) = band
        .first()
        .zip(band.last())
        .ok_or(DecodeError::Missing("world-object slot band"))?;
    let mut stream = BipedPositionStream {
        record_masks: vec![],
        slot_band: Some([lo, hi]),
        options,
        candidates: Vec::new(),
    };
    for c in chunks {
        for (packet_index, source) in super::fire_events::native_chunk_packets(c)
            .into_iter()
            .enumerate()
            .filter(|(_, p)| p.packet_type == 0)
        {
            let data = &c.data[source.payload_offset..source.payload_offset + source.payload_size];
            for record in scan_position_records_inner(
                data,
                [lo, hi],
                Some(band),
                map,
                stream.options.require_generation_one,
                dynamic_forward_level,
                stream.options.capture_dirs,
            )? {
                let saturated = record
                    .quantized
                    .iter()
                    .zip(map.axis_widths)
                    .any(|(&q, w)| q == 0 || q == (1u32 << w) - 1);
                let rejection = (stream.options.drop_saturated && saturated)
                    .then_some(BipedPositionRejection::Saturated);
                if stream.options.capture_dirs && rejection.is_none() {
                    stream.record_masks.push(BipedRecordMaskObservation {
                        source,
                        component_indices: record.component_indices.clone(),
                        after_position_bit: record.end_bit,
                    });
                }
                stream.candidates.push(BipedPositionCandidate {
                    packet_index: Some(packet_index),
                    source,
                    record,
                    rejection,
                });
            }
        }
    }
    filter_positions(&mut stream, teleports);
    Ok(stream)
}

pub(crate) fn isolated_position_indices(
    samples: impl Iterator<Item = (usize, u32, u64)>,
    gap: Option<u64>,
) -> Vec<usize> {
    let Some(gap) = gap else {
        return Vec::new();
    };
    let mut by_slot = BTreeMap::<u32, Vec<(usize, u64)>>::new();
    for (i, slot, time) in samples {
        by_slot.entry(slot).or_default().push((i, time));
    }
    let mut rejected = Vec::new();
    for positions in by_slot.values() {
        for (k, &(i, time)) in positions.iter().enumerate() {
            let mut nearest = if k > 0 {
                time.wrapping_sub(positions[k - 1].1)
            } else {
                u64::MAX
            };
            if let Some(&(_, next)) = positions.get(k + 1) {
                nearest = nearest.min(next.wrapping_sub(time));
            }
            if nearest > gap {
                rejected.push(i);
            }
        }
    }
    rejected
}

fn filter_positions(stream: &mut BipedPositionStream, teleports: &[FilmTranslocatorEvent]) {
    let isolated = isolated_position_indices(
        stream
            .candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.rejection.is_none())
            .map(|(i, c)| (i, c.record.slot, c.source.timestamp_us)),
        stream.options.isolation_threshold_us(),
    );
    for i in isolated {
        stream.candidates[i].rejection = Some(BipedPositionRejection::Isolated);
    }
    if stream.options.max_speed <= 0.0 {
        return;
    }
    let mut exemptions = BTreeMap::<u32, Vec<u64>>::new();
    for t in teleports {
        exemptions
            .entry(t.event.slot)
            .or_default()
            .push(t.source.timestamp_us);
    }
    for times in exemptions.values_mut() {
        times.sort_unstable();
    }
    let rejected = speed_rejected_indices(
        stream
            .candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.rejection.is_none())
            .map(|(i, c)| (i, c.record.slot, c.source.timestamp_us, c.record.world)),
        stream.options.max_speed,
        &exemptions,
    );
    for i in rejected {
        stream.candidates[i].rejection = Some(BipedPositionRejection::ExcessiveSpeed);
    }
}

/// Native DropTeleportsExcept. Exemption arrays retain caller order, just as the
/// native map API does; event-based callers sort when constructing the map.
pub(crate) fn speed_rejected_indices(
    samples: impl Iterator<Item = (usize, u32, u64, [f32; 3])>,
    max_speed: f64,
    exemptions: &BTreeMap<u32, Vec<u64>>,
) -> Vec<usize> {
    let mut rejected = Vec::new();
    if max_speed <= 0.0 {
        return rejected;
    }
    let mut anchors = BTreeMap::<u32, (u64, [f32; 3], u8)>::new();
    for (i, slot, time, world) in samples {
        let exempt = exemptions.get(&slot).is_some_and(|times| {
            let index = times.partition_point(|&t| t.wrapping_add(200_000) < time);
            times
                .get(index)
                .is_some_and(|&t| t <= time.wrapping_add(200_000))
        });
        if let Some(&(previous_time, previous_world, streak)) = anchors.get(&slot) {
            let distance = (0..3)
                .map(|axis| f64::from(world[axis] - previous_world[axis]).powi(2))
                .sum::<f64>()
                .sqrt();
            let speed = if time <= previous_time {
                if distance == 0.0 { 0.0 } else { f64::INFINITY }
            } else {
                distance / ((time - previous_time) as f64 / 1e6)
            };
            if streak < 3 && speed > max_speed && !exempt {
                rejected.push(i);
                anchors.insert(slot, (previous_time, previous_world, streak + 1));
                continue;
            }
        }
        anchors.insert(slot, (time, world, 0));
    }
    rejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    fn oracle() -> serde_json::Value {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-scan-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut json)
        .unwrap();
        serde_json::from_str(&json).unwrap()
    }
    fn packet(time: u64, index: usize) -> FilmPacket {
        FilmPacket {
            chunk_index: 1,
            packet_type: 0,
            byte_2: 0,
            byte_3: 0,
            payload_offset: index,
            payload_size: 0,
            timestamp_us: time,
        }
    }
    #[test]
    fn native_record_mask_hook() {
        #[derive(Deserialize)]
        struct Mask {
            time: u64,
            mask: Vec<u8>,
            end: usize,
            hex: String,
        }
        #[derive(Deserialize)]
        struct Case {
            quanta: Vec<serde_json::Value>,
            map: FilmMapBounds,
            hex: String,
            capture: bool,
            drop: bool,
            gap: u64,
            speed: f64,
            dynamic: bool,
            masks: Vec<Mask>,
            accepted: Vec<u64>,
            accepted_records: Vec<serde_json::Value>,
        }
        let unhex = |s: &str| -> Vec<u8> {
            s.as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                .collect()
        };
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/record-mask-hook-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 256);
        let mut before_filter = 0;
        let mut publications = 0;
        for (i, c) in cases.into_iter().enumerate() {
            let layout = super::super::I0Layout {
                gate_bits: (4 + c.map.region_index_bits.max(1)) as i64,
                axis_widths: c.map.axis_widths.map(|w| w as u64),
                region: c.map.region,
            };
            let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let observer = super::super::NativeFilmObserver::default();
            observer.set_hook(
                super::super::NativeHookKind::RecordMask,
                Some({
                    let observed = observed.clone();
                    std::sync::Arc::new(move |publication| {
                        let super::super::NativeHookPublication::RecordMask {
                            indices,
                            payload,
                            after_i0,
                        } = publication
                        else {
                            panic!("wrong hook");
                        };
                        observed.lock().unwrap().push((
                            indices.to_vec(),
                            payload.to_vec(),
                            after_i0,
                        ));
                    })
                }),
            );
            for quantum in &c.quanta {
                let raw = scan_quantized_biped_position_records_observed(
                    &unhex(quantum["hex"].as_str().unwrap()),
                    [512, 513],
                    &layout,
                    &BipedScanOptions {
                        capture_dirs: c.capture,
                        drop_saturated: c.drop,
                        ..Default::default()
                    },
                    c.dynamic.then_some(2),
                    Some(&observer),
                )
                .unwrap();
                let expected = quantum["records"].as_array().unwrap();
                assert_eq!(raw.len(), expected.len(), "quantized count {i}");
                for (r, e) in raw.iter().zip(expected) {
                    assert_eq!(r.slot, e["Slot"]);
                    assert_eq!(serde_json::json!(r.quantized), e["Q"]);
                    assert_eq!(e["HasWorld"], false);
                    for axis in ["X", "Y", "Z"] {
                        assert_eq!(e[axis].as_f64(), Some(0.0));
                    }
                    let value = serde_json::to_value(r).unwrap();
                    assert!(value.get("world").is_none());
                    assert_eq!(
                        *r,
                        serde_json::from_value::<QuantizedBipedPositionRecord>(value).unwrap()
                    );
                }
            }

            let expected_masks: Vec<_> = c
                .masks
                .iter()
                .map(|m| {
                    (
                        m.mask.iter().map(|&i| i as usize).collect::<Vec<_>>(),
                        unhex(&m.hex),
                        m.end,
                    )
                })
                .collect();
            assert_eq!(*observed.lock().unwrap(), expected_masks, "live masks {i}");
            let data = unhex(&c.hex);
            let chunk = FilmChunkData {
                metadata: crate::clients::hi::models::FilmChunk {
                    index: 1,
                    chunk_type: 2,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            };
            let out = scan_positions_for_band(
                std::slice::from_ref(&chunk),
                &BTreeSet::from([512, 513]),
                &c.map,
                BipedScanOptions {
                    capture_dirs: c.capture,
                    drop_saturated: c.drop,
                    require_generation_one: true,
                    isolation_gap_us: c.gap,
                    native_isolation_gap_ms: None,
                    max_speed: c.speed,
                },
                c.dynamic.then_some(2),
                &[],
            )
            .unwrap();
            for (actual, expected) in out.accepted().zip(&c.accepted_records) {
                assert_eq!(
                    actual.packet_index.unwrap(),
                    expected["PacketIndex"],
                    "ordinal {i}"
                );
                assert_eq!(actual.source.timestamp_us, expected["TimestampUS"]);
                assert_eq!(actual.record.slot, expected["Slot"]);
                assert_eq!(serde_json::json!(actual.record.quantized), expected["Q"]);
            }
            assert_eq!(out.accepted().count(), c.accepted_records.len());
            let mut exported = serde_json::to_value(&out).unwrap();
            assert_eq!(
                serde_json::from_value::<BipedPositionStream>(exported.clone()).unwrap(),
                out
            );
            for candidate in exported["candidates"].as_array_mut().unwrap() {
                candidate.as_object_mut().unwrap().remove("packet_index");
            }
            let legacy: BipedPositionStream = serde_json::from_value(exported).unwrap();
            assert!(legacy.candidates.iter().all(|c| c.packet_index.is_none()));
            if i == 1 {
                // A keyframe preceding the deltas counts toward the native
                // ordinal. Its byte length must never become that ordinal.
                let mut keyframe = vec![0u8; 17];
                let mut bit = 1;
                for value in [512u32 | (1 << 30), 35, 513 | (1 << 30), 35] {
                    for shift in (0..32).rev() {
                        keyframe[bit / 8] |= (((value >> shift) & 1) as u8) << (7 - bit % 8);
                        bit += 1;
                    }
                }
                let mut framed = vec![0u8; 16];
                framed[..2].copy_from_slice(&2u16.to_le_bytes());
                framed[4..8].copy_from_slice(&(keyframe.len() as u32).to_le_bytes());
                framed.extend(keyframe);
                framed.extend_from_slice(&chunk.data);
                let prefixed = FilmChunkData {
                    metadata: chunk.metadata.clone(),
                    data: framed,
                };
                let automatic = scan_biped_positions(
                    std::slice::from_ref(&prefixed),
                    &c.map,
                    out.options.clone(),
                    &[],
                )
                .unwrap();
                let explicit = scan_positions_for_band(
                    std::slice::from_ref(&prefixed),
                    &BTreeSet::from([512, 513]),
                    &c.map,
                    out.options.clone(),
                    None,
                    &[],
                )
                .unwrap();
                assert_eq!(automatic, explicit);
                assert!(!automatic.candidates.is_empty());
                assert_eq!(automatic.candidates.len(), out.candidates.len());
                for (a, b) in automatic.candidates.iter().zip(&out.candidates) {
                    assert_eq!(a.packet_index, b.packet_index.map(|n| n + 1));
                    assert_eq!(a.record, b.record);
                    assert_ne!(a.packet_index.unwrap(), a.source.payload_offset);
                }
            }
            assert_eq!(out.record_masks.len(), c.masks.len(), "mask count {i}");
            for (actual, expected) in out.record_masks.iter().zip(c.masks) {
                assert_eq!(actual.source.timestamp_us, expected.time, "time {i}");
                assert_eq!(actual.component_indices, expected.mask, "mask {i}");
                assert_eq!(actual.after_position_bit, expected.end, "offset {i}");
                let source = actual.source;
                assert_eq!(
                    chunk.data[source.payload_offset..source.payload_offset + source.payload_size],
                    unhex(&expected.hex),
                    "payload {i}"
                );
                publications += 1;
            }
            let accepted: Vec<_> = out.accepted().map(|r| r.source.timestamp_us).collect();
            assert_eq!(accepted, c.accepted, "filtered records {i}");
            if c.capture && out.record_masks.len() > accepted.len() {
                before_filter += 1;
            }
            if !c.capture {
                assert!(out.record_masks.is_empty());
                assert!(
                    out.candidates
                        .iter()
                        .all(|r| r.record.companions == Default::default())
                );
            }
            let restored: BipedPositionStream =
                serde_json::from_slice(&serde_json::to_vec(&out).unwrap()).unwrap();
            assert_eq!(restored, out);
        }
        assert!(publications > 1000);
        assert!(before_filter > 0);
    }
    #[test]
    fn position_headers_regions_values_and_boundaries_match_go() {
        let cases = oracle();
        let scans = cases["scans"].as_array().unwrap();
        assert_eq!(scans.len(), 1264);
        for row in scans {
            let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let need = row["require_generation_one"].as_bool().unwrap();
            let actual = scan_biped_position_records(&bytes, [512, 513], &map, need).unwrap();
            let expected = row["records"].as_array().unwrap();
            assert_eq!(actual.len(), expected.len(), "{row}");
            for (a, e) in actual.iter().zip(expected) {
                assert_eq!(a.start_bit as u64, e["start"].as_u64().unwrap());
                assert_eq!(a.position_bit as u64, e["position"].as_u64().unwrap());
                assert_eq!(a.end_bit as u64, e["end"].as_u64().unwrap());
                assert_eq!(u64::from(a.slot), e["slot"].as_u64().unwrap());
                assert_eq!(u64::from(a.generation), e["generation"].as_u64().unwrap());
                assert_eq!(serde_json::json!(a.component_indices), e["mask"]);
                assert_eq!(serde_json::json!(a.quantized), e["q"]);
                for i in 0..3 {
                    assert_eq!(
                        a.world[i].to_bits(),
                        (e["world"][i].as_f64().unwrap() as f32).to_bits()
                    );
                }
            }
            for n in 0..bytes.len() {
                let short =
                    scan_biped_position_records(&bytes[..n], [512, 513], &map, need).unwrap();
                assert!(short.iter().all(|r| r.end_bit <= n * 8));
            }
        }
    }
    #[test]
    fn native_isolation_gap_domain() {
        use std::io::Read;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/isolation-gap-domain-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 64);
        for (case, row) in rows.iter().enumerate() {
            let options = BipedScanOptions {
                native_isolation_gap_ms: Some(row["gap_ms"].as_i64().unwrap()),
                // The native value overrides this deliberately different convenience value.
                isolation_gap_us: 42,
                max_speed: 0.0,
                ..Default::default()
            };
            let candidates = row["samples"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, p)| BipedPositionCandidate {
                    packet_index: None,
                    source: packet(p["TimestampUS"].as_u64().unwrap(), i),
                    record: BipedPositionRecord {
                        start_bit: 0,
                        position_bit: 0,
                        end_bit: 0,
                        slot: p["Slot"].as_u64().unwrap() as u32,
                        generation: 1,
                        component_indices: vec![],
                        quantized: [0; 3],
                        world: [0.0; 3],
                        companions: Default::default(),
                    },
                    rejection: None,
                })
                .collect();
            let mut stream = BipedPositionStream {
                record_masks: vec![],
                slot_band: None,
                candidates,
                options,
            };
            filter_positions(&mut stream, &[]);
            let kept: Vec<_> = stream.accepted().map(|c| c.source.payload_offset).collect();
            assert_eq!(
                serde_json::json!(kept),
                row["kept"],
                "native isolation {case}"
            );
            let restored: BipedPositionStream =
                serde_json::from_value(serde_json::to_value(&stream).unwrap()).unwrap();
            assert_eq!(restored, stream);
        }
    }
    #[test]
    fn isolation_speed_reanchoring_and_teleport_exemptions_match_go() {
        let cases = oracle();
        let filters = cases["filters"].as_array().unwrap();
        assert_eq!(filters.len(), 64);
        for row in filters {
            let candidates = row["input"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, p)| BipedPositionCandidate {
                    packet_index: None,
                    source: packet(p["time"].as_u64().unwrap(), i),
                    record: BipedPositionRecord {
                        start_bit: 0,
                        position_bit: 0,
                        end_bit: 0,
                        slot: p["slot"].as_u64().unwrap() as u32,
                        generation: 1,
                        component_indices: vec![0],
                        quantized: [0; 3],
                        companions: Default::default(),
                        world: std::array::from_fn(|i| p["world"][i].as_f64().unwrap() as f32),
                    },
                    rejection: None,
                })
                .collect();
            let mut stream = BipedPositionStream {
                record_masks: vec![],
                slot_band: Some([512, 515]),
                candidates,
                options: BipedScanOptions {
                    capture_dirs: true,
                    require_generation_one: true,
                    drop_saturated: false,
                    isolation_gap_us: row["gap_us"].as_u64().unwrap(),
                    native_isolation_gap_ms: None,
                    max_speed: row["max_speed"].as_f64().unwrap(),
                },
            };
            let teleports: Vec<_> = row["teleports"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|t| FilmTranslocatorEvent {
                    source: packet(t["TimestampUS"].as_u64().unwrap(), 0),
                    event: super::super::TranslocatorEvent {
                        config: true,
                        slot: t["Slot"].as_u64().unwrap() as u32,
                        generation: 1,
                        other_reference_gates: None,
                        effect_present: None,
                        effect: None,
                        from: None,
                        to: None,
                        end_bit: 20,
                        stop: super::super::TranslocatorStop::MissingMap,
                    },
                })
                .collect();
            filter_positions(&mut stream, &teleports);
            let isolated: Vec<_> = stream
                .candidates
                .iter()
                .filter(|c| c.rejection != Some(BipedPositionRejection::Isolated))
                .map(|c| c.source.payload_offset)
                .collect();
            let accepted: Vec<_> = stream.accepted().map(|c| c.source.payload_offset).collect();
            assert_eq!(serde_json::json!(isolated), row["after_isolation"]);
            assert_eq!(serde_json::json!(accepted), row["accepted"]);
            let json = serde_json::to_vec(&stream).unwrap();
            assert_eq!(
                serde_json::from_slice::<BipedPositionStream>(&json).unwrap(),
                stream
            );
        }
    }
}
