//! Experimental decoder for Halo Infinite Theater film chunks.

use std::collections::BTreeMap;

use super::models::FilmChunkData;

const PLAYER_MARKER: [u8; 2] = [0x2d, 0xc0];
const REGISTRY_SLOT_SIZE: usize = 260;
const REGISTRY_BLOCK_SLOTS: usize = 64;
const REGISTRY_BLOCK_SIZE: usize = REGISTRY_SLOT_SIZE * REGISTRY_BLOCK_SLOTS;

/// Ordered replication components for one ECS entity archetype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmArchetype {
    pub index: usize,
    pub components: Vec<String>,
}

/// Entity-component schema serialized in a Theater film's bootstrap chunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmRegistry {
    pub archetypes: Vec<FilmArchetype>,
}

impl FilmRegistry {
    pub fn archetype(&self, index: usize) -> Option<&FilmArchetype> {
        self.archetypes.get(index)
    }

    pub fn biped_archetypes(&self) -> impl Iterator<Item = &FilmArchetype> {
        self.archetypes.iter().filter(|archetype| {
            archetype
                .components
                .iter()
                .any(|name| name == "object-position-dynamic-precision-component")
                && archetype
                    .components
                    .iter()
                    .any(|name| name == "object-dead-state-component")
        })
    }
}

/// Parses the fixed-width ECS component registry from the film bootstrap chunk.
pub fn decode_registry(chunks: &[FilmChunkData]) -> Option<FilmRegistry> {
    let data = &chunks
        .iter()
        .find(|chunk| chunk.metadata.chunk_type == 1)?
        .data;
    let archetypes = data
        .chunks_exact(REGISTRY_BLOCK_SIZE)
        .enumerate()
        .map(|(index, block)| {
            let components = block
                .chunks_exact(REGISTRY_SLOT_SIZE)
                .map(registry_slot_name)
                .take_while(Option::is_some)
                .flatten()
                .collect();
            FilmArchetype { index, components }
        })
        .collect();
    Some(FilmRegistry { archetypes })
}

fn registry_slot_name(slot: &[u8]) -> Option<String> {
    let bytes = slot.get(8..)?;
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let name = bytes.get(..end)?;
    if name.is_empty() || !name.iter().all(|byte| byte.is_ascii_graphic()) {
        return None;
    }
    Some(String::from_utf8(name.to_vec()).expect("validated ASCII component name"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FilmEntityState {
    full_id: u32,
    archetype_index: u8,
}

/// Persistent entity bindings required to interpret FRAME delta records.
#[derive(Debug, Clone, Default)]
pub struct FilmWorld {
    entities: BTreeMap<u32, FilmEntityState>,
}

impl FilmWorld {
    pub fn bind(&mut self, full_id: u32, archetype_index: u8) {
        self.entities.insert(
            full_id & 0x3fff_ffff,
            FilmEntityState {
                full_id,
                archetype_index,
            },
        );
    }

    pub fn unbind(&mut self, slot: u32) {
        self.entities.remove(&slot);
    }

    pub fn archetype_index(&self, slot: u32) -> Option<u8> {
        self.entities.get(&slot).map(|state| state.archetype_index)
    }

    pub fn full_id(&self, slot: u32) -> Option<u32> {
        self.entities.get(&slot).map(|state| state.full_id)
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct FilmBitReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> FilmBitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub fn read(&mut self, width: usize) -> Option<u64> {
        if width > 64 || self.position.checked_add(width)? > self.data.len() * 8 {
            return None;
        }
        let mut value = 0u64;
        for _ in 0..width {
            value = (value << 1) | u64::from(bit_at(self.data, self.position));
            self.position += 1;
        }
        Some(value)
    }

    pub const fn position(&self) -> usize {
        self.position
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilmRecordKind {
    End,
    New,
    Delete,
    Delta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilmRecordHeader {
    pub kind: FilmRecordKind,
    pub full_id: Option<u32>,
    pub slot: Option<u32>,
    pub header_bits: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilmDeltaHeader {
    pub record: FilmRecordHeader,
    pub component_mask: u64,
    pub data_bit_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilmPositionEncoding {
    Raw,
    Absolute,
    AbsoluteFallback,
    Delta8,
    DeltaAxis,
    Unchanged,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilmPositionUpdate {
    pub slot: u32,
    pub encoding: FilmPositionEncoding,
    pub vector: Option<[f32; 3]>,
}

/// Decodes position component 0 from the first DELTA record in a FRAME.
///
/// Delta vectors are relative and must be accumulated against the entity's
/// latest absolute position. Multiplayer films observed so far use 14-bit axes.
pub fn decode_first_position_update(
    payload: &[u8],
    axis_width: usize,
) -> Option<FilmPositionUpdate> {
    let delta = decode_first_delta_header(payload)?;
    if delta.component_mask & 1 == 0 || !(1..=24).contains(&axis_width) {
        return None;
    }
    let slot = delta.record.slot?;
    let mut reader = FilmBitReader::new(payload);
    reader.read(delta.data_bit_offset)?;
    let use_prediction = reader.read(1)? != 0;
    let is_delta = reader.read(1)? != 0;

    if use_prediction {
        reader.read(1)?; // handle-present; tail follows the vector
        let mut vector = [0.0; 3];
        for value in &mut vector {
            *value = f32::from_bits(reader.read(32)? as u32);
        }
        return Some(FilmPositionUpdate {
            slot,
            encoding: FilmPositionEncoding::Raw,
            vector: Some(vector),
        });
    }
    if !is_delta {
        return decode_absolute_position(&mut reader, slot, axis_width, false);
    }

    let special = reader.read(1)? != 0;
    if special {
        if reader.read(1)? != 0 {
            return Some(FilmPositionUpdate {
                slot,
                encoding: FilmPositionEncoding::Unchanged,
                vector: None,
            });
        }
        return decode_absolute_body(&mut reader, slot, axis_width, false);
    }
    if reader.read(1)? != 0 {
        return decode_absolute_position(&mut reader, slot, axis_width, true);
    }
    if reader.read(1)? != 0 {
        let step = position_step(axis_width);
        let mut vector = [0.0; 3];
        for value in &mut vector {
            *value = f32::from(reader.read(8)? as u8 as i8) * step;
        }
        return Some(FilmPositionUpdate {
            slot,
            encoding: FilmPositionEncoding::Delta8,
            vector: Some(vector),
        });
    }

    let mut vector = [0.0; 3];
    for value in &mut vector {
        *value = reader.read(axis_width)? as f32 * position_step(axis_width);
    }
    Some(FilmPositionUpdate {
        slot,
        encoding: FilmPositionEncoding::DeltaAxis,
        vector: Some(vector),
    })
}

fn decode_absolute_position(
    reader: &mut FilmBitReader<'_>,
    slot: u32,
    axis_width: usize,
    fallback: bool,
) -> Option<FilmPositionUpdate> {
    if reader.read(1)? != 0 {
        return Some(FilmPositionUpdate {
            slot,
            encoding: FilmPositionEncoding::Unchanged,
            vector: None,
        });
    }
    decode_absolute_body(reader, slot, axis_width, fallback)
}

fn decode_absolute_body(
    reader: &mut FilmBitReader<'_>,
    slot: u32,
    axis_width: usize,
    fallback: bool,
) -> Option<FilmPositionUpdate> {
    if reader.read(1)? == 0 {
        reader.read(1)?; // precision index
    }
    let mut vector = [0.0; 3];
    for value in &mut vector {
        *value = dequant_position_axis(reader.read(axis_width)?, axis_width);
    }
    Some(FilmPositionUpdate {
        slot,
        encoding: if fallback {
            FilmPositionEncoding::AbsoluteFallback
        } else {
            FilmPositionEncoding::Absolute
        },
        vector: Some(vector),
    })
}

fn position_step(axis_width: usize) -> f32 {
    200.0 / (1u64 << axis_width) as f32
}

fn dequant_position_axis(value: u64, axis_width: usize) -> f32 {
    let step = position_step(axis_width);
    value as f32 * step - 100.0 + step * 0.5
}

/// Decodes the first DELTA record through its component-presence mask.
pub fn decode_first_delta_header(payload: &[u8]) -> Option<FilmDeltaHeader> {
    let record = decode_frame_record_header(payload)?;
    if record.kind != FilmRecordKind::Delta {
        return None;
    }
    let mut reader = FilmBitReader::new(payload);
    reader.read(record.header_bits)?;
    let component_mask = if reader.read(1)? != 0 {
        reader.read(64)?
    } else {
        let count = reader.read(3)? as usize;
        let mut mask = 0u64;
        for _ in 0..count {
            mask |= 1u64 << reader.read(6)?;
        }
        mask
    };
    Some(FilmDeltaHeader {
        record,
        component_mask,
        data_bit_offset: reader.position(),
    })
}

/// Decodes the prefix-coded header of the first entity record in a FRAME payload.
pub fn decode_frame_record_header(payload: &[u8]) -> Option<FilmRecordHeader> {
    decode_frame_record_header_with_id_width(payload, 11)
}

fn decode_frame_record_header_with_id_width(
    payload: &[u8],
    id_width: usize,
) -> Option<FilmRecordHeader> {
    let mut reader = FilmBitReader::new(payload);
    let kind = if reader.read(1)? != 0 {
        FilmRecordKind::Delta
    } else {
        match reader.read(2)? {
            0 => FilmRecordKind::End,
            1 => FilmRecordKind::New,
            2 => FilmRecordKind::Delete,
            3 => FilmRecordKind::Delta,
            _ => unreachable!(),
        }
    };
    if kind == FilmRecordKind::End {
        return Some(FilmRecordHeader {
            kind,
            full_id: None,
            slot: None,
            header_bits: reader.position(),
        });
    }
    let low = reader.read(id_width)? as u32;
    let tag = reader.read(2)? as u32;
    let full_id = (tag << 30) | low;
    Some(FilmRecordHeader {
        kind,
        full_id: Some(full_id),
        slot: Some(low),
        header_bits: reader.position(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilmPacket {
    pub chunk_index: i32,
    pub packet_type: u16,
    pub byte_2: u8,
    pub byte_3: u8,
    pub payload_offset: usize,
    pub payload_size: usize,
    pub timestamp_us: u64,
}

/// Indexes the byte-aligned packet stream inside decompressed replication chunks.
pub fn index_packets(chunks: &[FilmChunkData]) -> Vec<FilmPacket> {
    let mut packets = Vec::new();
    for chunk in chunks.iter().filter(|chunk| chunk.metadata.chunk_type == 2) {
        let mut header_offset = 0usize;
        while let Some(header) = chunk.data.get(header_offset..header_offset + 16) {
            let packet_type = u16::from_le_bytes([header[0], header[1]]);
            let payload_size =
                u32::from_le_bytes(header[4..8].try_into().expect("four size bytes")) as usize;
            let timestamp_us =
                u64::from_le_bytes(header[8..16].try_into().expect("eight timestamp bytes"));
            let payload_offset = header_offset + 16;
            let Some(next_header) = payload_offset.checked_add(payload_size) else {
                break;
            };
            if next_header > chunk.data.len() {
                break;
            }
            packets.push(FilmPacket {
                chunk_index: chunk.metadata.index,
                packet_type,
                byte_2: header[2],
                byte_3: header[3],
                payload_offset,
                payload_size,
                timestamp_us,
            });
            header_offset = next_header;
            if packet_type == 7 {
                break;
            }
        }
    }
    packets
}

/// An absolute position found in a Theater keyframe.
///
/// Keyframes currently do not expose a reliable entity-to-player binding, so
/// these samples intentionally remain unattributed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilmPosition {
    pub chunk_index: i32,
    pub timestamp_ms: i64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Decodes absolute XYZ samples from the full-state keyframes in replication chunks.
///
/// This is useful for map-level spatial analysis. It does not decode the denser
/// per-frame position deltas or associate a sample with a player yet.
pub fn decode_keyframe_positions(chunks: &[FilmChunkData]) -> Vec<FilmPosition> {
    const COMB_OFFSET_BITS: usize = 273;
    let mut positions = Vec::new();
    for chunk in chunks.iter().filter(|chunk| chunk.metadata.chunk_type == 2) {
        for packet in index_packets(std::slice::from_ref(chunk))
            .into_iter()
            .filter(|packet| packet.packet_type == 2)
        {
            let payload =
                &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            let end = payload.len().saturating_mul(8).saturating_sub(96);
            let mut bit = COMB_OFFSET_BITS;
            while bit <= end {
                if position_comb_matches(payload, bit) {
                    let start = bit - COMB_OFFSET_BITS;
                    let x = read_f32_le_bits(payload, start);
                    let y = read_f32_le_bits(payload, start + 32);
                    let z = read_f32_le_bits(payload, start + 64);
                    if plausible_position(x, y, z) {
                        positions.push(FilmPosition {
                            chunk_index: chunk.metadata.index,
                            timestamp_ms: chunk.metadata.start_time_offset_ms,
                            x,
                            y,
                            z,
                        });
                    }
                    bit += 96;
                } else {
                    bit += 1;
                }
            }
        }
    }
    positions
}

fn position_comb_matches(data: &[u8], start: usize) -> bool {
    (0..4).all(|repeat| {
        let base = start + repeat * 24;
        (0..8).all(|offset| bit_at(data, base + offset))
            && (8..24).all(|offset| !bit_at(data, base + offset))
    })
}

fn read_f32_le_bits(data: &[u8], start: usize) -> f32 {
    let mut bytes = [0u8; 4];
    for (byte_index, byte) in bytes.iter_mut().enumerate() {
        for bit_index in 0..8 {
            *byte |= u8::from(bit_at(data, start + byte_index * 8 + bit_index)) << (7 - bit_index);
        }
    }
    f32::from_bits(u32::from_be_bytes(bytes).swap_bytes())
}

fn plausible_position(x: f32, y: f32, z: f32) -> bool {
    let magnitude = x.abs() + y.abs() + z.abs();
    let artifact = (x.abs() <= 0.1 && (y - 2.0).abs() <= 0.1 && z.abs() <= 0.1)
        || ((x + 2.1).abs() <= 0.1 && y.abs() <= 0.1 && z.abs() <= 0.1);
    [x, y, z]
        .iter()
        .all(|value| value.is_finite() && value.abs() < 200.0)
        && magnitude >= 1.0
        && !artifact
}

fn bit_at(data: &[u8], position: usize) -> bool {
    data.get(position / 8)
        .is_some_and(|byte| byte & (1 << (7 - position % 8)) != 0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmPlayer {
    pub xuid: u64,
    pub gamertag: String,
}

/// Finds the film's 5-bit replication index for each known human player.
pub fn decode_player_indices(
    chunks: &[FilmChunkData],
    players: &[FilmPlayer],
) -> BTreeMap<u64, u8> {
    let mut indices = BTreeMap::new();
    for chunk in chunks.iter().filter(|chunk| chunk.metadata.chunk_type == 2) {
        for player in players {
            if indices.contains_key(&player.xuid) {
                continue;
            }
            let xuid = player.xuid.to_le_bytes();
            let Some(position) = find_bit_pattern(&chunk.data, &xuid).into_iter().next() else {
                continue;
            };
            let Some(index_position) = position.checked_sub(5) else {
                continue;
            };
            if let Some(bits) = extract_bits(&chunk.data, index_position, 5) {
                indices.insert(player.xuid, bits[0] >> 3);
            }
        }
    }
    indices
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilmEventKind {
    Mode,
    Death,
    Kill,
    Medal,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilmMedal {
    DoubleKill,
    TripleKill,
    Overkill,
    Killtacular,
    KillingSpree,
    Killjoy,
    Wingman,
    Rifleman,
    Boxer,
    BackSmack,
    Snipe,
    Perfect,
    NoScope,
    CounterSnipe,
    Unknown(u8),
}

impl FilmMedal {
    pub const fn from_id(id: u8) -> Self {
        match id {
            0 => Self::DoubleKill,
            1 => Self::TripleKill,
            2 => Self::Overkill,
            3 => Self::Killtacular,
            9 => Self::KillingSpree,
            26 => Self::Killjoy,
            65 => Self::Wingman,
            71 => Self::Rifleman,
            74 => Self::Boxer,
            82 => Self::BackSmack,
            108 => Self::Snipe,
            109 => Self::Perfect,
            114 => Self::NoScope,
            168 => Self::CounterSnipe,
            other => Self::Unknown(other),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::DoubleKill => "Double Kill",
            Self::TripleKill => "Triple Kill",
            Self::Overkill => "Overkill",
            Self::Killtacular => "Killtacular",
            Self::KillingSpree => "Killing Spree",
            Self::Killjoy => "Killjoy",
            Self::Wingman => "Wingman",
            Self::Rifleman => "Rifleman",
            Self::Boxer => "Boxer",
            Self::BackSmack => "Back Smack",
            Self::Snipe => "Snipe",
            Self::Perfect => "Perfect",
            Self::NoScope => "No Scope",
            Self::CounterSnipe => "Counter-snipe",
            Self::Unknown(_) => "Unknown medal",
        }
    }
}

impl FilmEventKind {
    pub const fn from_fields(code: u8, medal_flag: u8) -> Self {
        if medal_flag != 0 {
            return Self::Medal;
        }
        match code {
            10 => Self::Mode,
            20 => Self::Death,
            50 => Self::Kill,
            other => Self::Other(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmEvent {
    pub xuid: u64,
    pub gamertag: String,
    pub timestamp_ms: u32,
    pub kind: FilmEventKind,
    pub medal_flag: u8,
    pub metadata: u8,
}

impl FilmEvent {
    pub const fn medal(&self) -> Option<FilmMedal> {
        if matches!(self.kind, FilmEventKind::Medal) {
            Some(FilmMedal::from_id(self.metadata))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FilmDecodeDiagnostics {
    pub player_markers: usize,
    pub markers_with_zero_padding: usize,
    pub decoded_gamertags: usize,
}

pub fn decode_diagnostics(chunks: &[FilmChunkData]) -> FilmDecodeDiagnostics {
    let mut diagnostics = FilmDecodeDiagnostics::default();
    for chunk in chunks.iter().filter(|chunk| chunk.metadata.chunk_type != 3) {
        for marker_position in find_bit_pattern(&chunk.data, &PLAYER_MARKER) {
            diagnostics.player_markers += 1;
            let Some(xuid_position) = marker_position.checked_sub(8 * 8) else {
                continue;
            };
            let Some(padding_position) = xuid_position.checked_sub(21 * 8) else {
                continue;
            };
            if bits_are_zero(&chunk.data, padding_position, 21 * 8) {
                diagnostics.markers_with_zero_padding += 1;
                let Some(gamertag_position) = padding_position.checked_sub(32 * 8) else {
                    continue;
                };
                if extract_bits(&chunk.data, gamertag_position, 32 * 8)
                    .is_some_and(|bytes| !decode_utf16(&bytes).is_empty())
                {
                    diagnostics.decoded_gamertags += 1;
                }
            }
        }
    }
    diagnostics
}

pub fn decode_players(chunks: &[FilmChunkData]) -> Vec<FilmPlayer> {
    let mut players = BTreeMap::new();
    for chunk in chunks.iter().filter(|chunk| chunk.metadata.chunk_type != 3) {
        for marker_position in find_bit_pattern(&chunk.data, &PLAYER_MARKER) {
            let Some(xuid_position) = marker_position.checked_sub(8 * 8) else {
                continue;
            };
            let Some(padding_position) = xuid_position.checked_sub(21 * 8) else {
                continue;
            };
            let Some(gamertag_position) = padding_position.checked_sub(32 * 8) else {
                continue;
            };
            let Some(xuid_bytes) = extract_bits(&chunk.data, xuid_position, 8 * 8) else {
                continue;
            };
            if !bits_are_zero(&chunk.data, padding_position, 21 * 8) {
                continue;
            }
            let xuid = u64::from_le_bytes(xuid_bytes.try_into().expect("eight XUID bytes"));
            if xuid == 0 {
                continue;
            }
            let Some(gamertag_bytes) = extract_bits(&chunk.data, gamertag_position, 32 * 8) else {
                continue;
            };
            let gamertag = decode_utf16(&gamertag_bytes);
            if !gamertag.is_empty() {
                players.entry(xuid).or_insert(gamertag);
            }
        }
    }
    players
        .into_iter()
        .map(|(xuid, gamertag)| FilmPlayer { xuid, gamertag })
        .collect()
}

pub fn decode_events(chunks: &[FilmChunkData], players: &[FilmPlayer]) -> Vec<FilmEvent> {
    let mut events = Vec::new();
    for chunk in chunks.iter().filter(|chunk| chunk.metadata.chunk_type == 3) {
        for player in players {
            let pattern = player
                .gamertag
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>();
            for position in find_bit_pattern(&chunk.data, &pattern) {
                let Some(data) = extract_bits(&chunk.data, position, 60 * 8) else {
                    continue;
                };
                events.push(FilmEvent {
                    xuid: player.xuid,
                    gamertag: player.gamertag.clone(),
                    timestamp_ms: u32::from_be_bytes(data[48..52].try_into().unwrap()),
                    kind: FilmEventKind::from_fields(data[47], data[55]),
                    medal_flag: data[55],
                    metadata: data[59],
                });
            }
        }
    }
    events.sort_by_key(|event| event.timestamp_ms);
    events.dedup();
    events
}

fn decode_utf16(bytes: &[u8]) -> String {
    let values = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    String::from_utf16_lossy(&values)
        .trim_matches('\0')
        .trim()
        .to_string()
}

fn bits_are_zero(data: &[u8], position: usize, length: usize) -> bool {
    extract_bits(data, position, length).is_some_and(|bytes| bytes.iter().all(|byte| *byte == 0))
}

fn find_bit_pattern(data: &[u8], pattern: &[u8]) -> Vec<usize> {
    if pattern.is_empty() || pattern.len() > data.len() {
        return Vec::new();
    }
    let end = data.len() * 8 - pattern.len() * 8;
    (0..=end)
        .filter(|position| bit_pattern_matches(data, pattern, *position))
        .collect()
}

fn bit_pattern_matches(data: &[u8], pattern: &[u8], position: usize) -> bool {
    let byte_offset = position / 8;
    let shift = position % 8;
    pattern.iter().enumerate().all(|(index, expected)| {
        let mut actual = data[byte_offset + index] << shift;
        if shift > 0 && byte_offset + index + 1 < data.len() {
            actual |= data[byte_offset + index + 1] >> (8 - shift);
        }
        actual == *expected
    })
}

fn extract_bits(data: &[u8], position: usize, length: usize) -> Option<Vec<u8>> {
    if length == 0 || position.checked_add(length)? > data.len() * 8 {
        return None;
    }
    let byte_count = length.div_ceil(8);
    let byte_offset = position / 8;
    let shift = position % 8;
    let mut output = Vec::with_capacity(byte_count);
    for index in 0..byte_count {
        let mut byte = data[byte_offset + index] << shift;
        if shift > 0 && byte_offset + index + 1 < data.len() {
            byte |= data[byte_offset + index + 1] >> (8 - shift);
        }
        output.push(byte);
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;

    #[test]
    fn finds_unaligned_patterns() {
        let data = [0b1011_0011, 0b0101_1110];
        assert_eq!(find_bit_pattern(&data, &[0b1100_1101]), vec![2]);
    }

    #[test]
    fn reads_unaligned_little_endian_float() {
        let value = 12.5f32;
        let mut data = vec![0u8; 5];
        for (index, bit) in value
            .to_le_bytes()
            .iter()
            .flat_map(|byte| (0..8).map(move |shift| byte & (1 << (7 - shift)) != 0))
            .enumerate()
        {
            if bit {
                let position = index + 3;
                data[position / 8] |= 1 << (7 - position % 8);
            }
        }
        assert_eq!(read_f32_le_bits(&data, 3), value);
    }

    #[test]
    fn recognizes_position_comb() {
        let mut data = [0u8; 12];
        for repeat in 0..4 {
            data[repeat * 3] = 0xff;
        }
        assert!(position_comb_matches(&data, 0));
        data[4] = 1;
        assert!(!position_comb_matches(&data, 0));
    }

    #[test]
    fn reads_msb_first_bits() {
        let mut reader = FilmBitReader::new(&[0b1011_0010, 0b0110_0000]);
        assert_eq!(reader.read(3), Some(0b101));
        assert_eq!(reader.read(7), Some(0b1001001));
        assert_eq!(reader.position(), 10);
    }

    #[test]
    fn parses_registry_slots() {
        let mut data = vec![0; REGISTRY_BLOCK_SIZE];
        let name = b"object-position-dynamic-precision-component";
        data[8..8 + name.len()].copy_from_slice(name);
        let chunks = [FilmChunkData {
            metadata: FilmChunk {
                index: 0,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: data.len() as i64,
                file_relative_path: String::new(),
                chunk_type: 1,
            },
            data,
        }];
        let registry = decode_registry(&chunks).unwrap();
        assert_eq!(
            registry.archetypes[0].components,
            [String::from_utf8_lossy(name)]
        );
    }

    #[test]
    fn decodes_prefix_coded_record_headers() {
        assert_eq!(
            decode_frame_record_header(&[0]),
            Some(FilmRecordHeader {
                kind: FilmRecordKind::End,
                full_id: None,
                slot: None,
                header_bits: 3,
            })
        );
        let header =
            decode_frame_record_header_with_id_width(&[0b1010_1010, 0b1010_1010], 13).unwrap();
        assert_eq!(header.kind, FilmRecordKind::Delta);
        assert_eq!(header.slot, Some(0b0_1010_1010_1010));
        assert_eq!(header.header_bits, 16);
    }

    #[test]
    fn decodes_sparse_component_mask() {
        // DELTA, slot 0, tag 0, sparse mask with indices 0 and 11.
        let bits = "1 00000000000 00 0 010 000000 001011"
            .chars()
            .filter(|character| *character != ' ')
            .collect::<String>();
        let mut data = vec![0u8; bits.len().div_ceil(8)];
        for (index, character) in bits.bytes().enumerate() {
            if character == b'1' {
                data[index / 8] |= 1 << (7 - index % 8);
            }
        }
        let delta = decode_first_delta_header(&data).unwrap();
        assert_eq!(delta.record.slot, Some(0));
        assert_eq!(delta.component_mask, 1 | (1 << 11));
        assert_eq!(delta.data_bit_offset, bits.len());
    }
}
