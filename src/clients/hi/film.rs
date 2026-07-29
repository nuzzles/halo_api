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

/// Decodes the first DELTA record through its component-presence mask.
pub fn decode_first_delta_header(payload: &[u8]) -> Option<FilmDeltaHeader> {
    let record = decode_frame_record_header(payload)?;
    if record.kind != FilmRecordKind::Delta {
        return None;
    }
    let mut reader = FilmBitReader::new(payload);
    reader.read(record.header_bits)?;
    let component_mask = if reader.read(1)? != 0 {
        // Component bitsets are serialized component-0 first. `FilmBitReader`
        // returns that first bit as the high bit of a u64, whereas the sparse
        // representation below uses component indexes as normal bit positions.
        reader.read(64)?.reverse_bits()
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
    Known { id: u8, name: &'static str },
    Unknown(u8),
}

/// Known film-medal IDs, including the SPNKr mappings catalogued in
/// <https://den.dev/blog/extracting-stats-film-files-halo-infinite/>.
pub const KNOWN_FILM_MEDALS: &[(u8, &str)] = &[
    (0, "Double Kill"),
    (1, "Triple Kill"),
    (2, "Overkill"),
    (3, "Killtacular"),
    (4, "Killtrocity"),
    (5, "Killamanjaro"),
    (6, "Killtastrophe"),
    (7, "Killpocalypse"),
    (8, "Killionaire"),
    (9, "Killing Spree"),
    (10, "Killing Frenzy"),
    (11, "Running Riot"),
    (12, "Rampage"),
    (13, "Perfection"),
    (26, "Killjoy"),
    (27, "Nightmare"),
    (28, "Boogeyman"),
    (29, "Grim Reaper"),
    (30, "Demon"),
    (31, "Flawless Victory"),
    (32, "Steaktacular"),
    (36, "Stopped Short"),
    (37, "Flag Joust"),
    (38, "Goal Line Stand"),
    (39, "Necromancer"),
    (43, "Ace"),
    (44, "Extermination"),
    (45, "Sole Survivor"),
    (46, "Untainted"),
    (47, "Blight"),
    (48, "Disease"),
    (49, "Plague"),
    (51, "Pestilence"),
    (53, "Culling"),
    (54, "Cleansing"),
    (55, "Purge"),
    (56, "Purification"),
    (57, "Divine Intervention"),
    (58, "Zombie Slayer"),
    (59, "Undead Hunter"),
    (60, "Hell's Janitor"),
    (61, "The Sickness"),
    (62, "Spotter"),
    (63, "Treasure Hunter"),
    (64, "Saboteur"),
    (65, "Wingman"),
    (66, "Wheelman"),
    (67, "Gunner"),
    (68, "Driver"),
    (69, "Pilot"),
    (70, "Tanker"),
    (71, "Rifleman"),
    (72, "Bomber"),
    (73, "Grenadier"),
    (74, "Boxer"),
    (75, "Warrior"),
    (76, "Gunslinger"),
    (77, "Scattergunner"),
    (78, "Sharpshooter"),
    (79, "Marksman"),
    (80, "Heavy"),
    (81, "Bodyguard"),
    (82, "Back Smack"),
    (83, "Nuclear Football"),
    (84, "Boom Block"),
    (85, "Bulltrue"),
    (86, "Cluster Luck"),
    (87, "Dogfight"),
    (88, "Harpoon"),
    (89, "Mind the Gap"),
    (90, "Ninja"),
    (91, "Odin's Raven"),
    (92, "Pancake"),
    (93, "Quigley"),
    (94, "Remote Detonation"),
    (95, "Return to Sender"),
    (96, "Rideshare"),
    (97, "Skyjack"),
    (98, "Stick"),
    (99, "Tag & Bag"),
    (108, "Snipe"),
    (109, "Perfect"),
    (114, "No Scope"),
    (127, "From the Grave"),
    (128, "From the Void"),
    (129, "Grapple-jack"),
    (130, "Hold This"),
    (131, "Last Shot"),
    (132, "Lawnmower"),
    (133, "Mount Up"),
    (134, "Off the Rack"),
    (135, "Quick Draw"),
    (137, "Pineapple Express"),
    (138, "Ramming Speed"),
    (139, "Reclaimer"),
    (140, "Shot Caller"),
    (141, "Yard Sale"),
    (142, "Special Delivery"),
    (146, "Fumble"),
    (148, "Straight Balling"),
    (151, "Always Rotating"),
    (152, "Hill Guardian"),
    (153, "Clock Stop"),
    (154, "Secure Line"),
    (156, "Splatter"),
    (162, "All That Juice"),
    (163, "Great Journey"),
    (165, "Breacher"),
    (166, "Mounted"),
    (168, "Counter-snipe"),
];

impl FilmMedal {
    pub fn from_id(id: u8) -> Self {
        KNOWN_FILM_MEDALS
            .iter()
            .find(|(known_id, _)| *known_id == id)
            .map_or(Self::Unknown(id), |(_, name)| Self::Known { id, name })
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Known { name, .. } => name,
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
    pub fn medal(&self) -> Option<FilmMedal> {
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
    const EVENT_HEADER_BYTES: usize = 12;
    const EVENT_GAMERTAG_BYTES: usize = 32;
    const EVENT_TAIL_BYTES: usize = 60;
    const PREFIX_PADDING_BYTES: usize = 3;

    let mut events = Vec::new();
    for chunk in chunks.iter().filter(|chunk| chunk.metadata.chunk_type == 3) {
        for player in players {
            let Some(gamertag_field) = padded_gamertag_field(&player.gamertag) else {
                continue;
            };
            for gamertag_position in find_bit_pattern(&chunk.data, &gamertag_field) {
                let Some(event_position) = gamertag_position.checked_sub(EVENT_HEADER_BYTES * 8)
                else {
                    continue;
                };
                let Some(prefix_position) = event_position.checked_sub(PREFIX_PADDING_BYTES * 8)
                else {
                    continue;
                };
                if !bits_are_zero(&chunk.data, prefix_position, PREFIX_PADDING_BYTES * 8) {
                    continue;
                }
                let Some(data) = extract_bits(&chunk.data, gamertag_position, EVENT_TAIL_BYTES * 8)
                else {
                    continue;
                };
                if data[..EVENT_GAMERTAG_BYTES] != gamertag_field {
                    continue;
                }
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

fn padded_gamertag_field(gamertag: &str) -> Option<[u8; 32]> {
    let encoded = gamertag
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    let mut field = [0; 32];
    field.get_mut(..encoded.len())?.copy_from_slice(&encoded);
    Some(field)
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
    fn reads_msb_first_bits() {
        let mut reader = FilmBitReader::new(&[0b1011_0010, 0b0110_0000]);
        assert_eq!(reader.read(3), Some(0b101));
        assert_eq!(reader.read(7), Some(0b1001001));
        assert_eq!(reader.position(), 10);
    }

    #[test]
    fn parses_registry_slots() {
        let mut data = vec![0; REGISTRY_BLOCK_SIZE];
        let name = b"example-component";
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
    fn decodes_only_complete_summary_event_envelopes() {
        let mut data = vec![0; 3];
        let mut event = [0u8; 72];
        event[..12].fill(0xa5);
        event[12..44].copy_from_slice(&padded_gamertag_field("MsNuzzles").unwrap());
        event[59] = 50;
        event[60..64].copy_from_slice(&12_345u32.to_be_bytes());
        event[71] = 42;
        data.extend_from_slice(&event);
        let chunks = [FilmChunkData {
            metadata: FilmChunk {
                index: 0,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: data.len() as i64,
                file_relative_path: String::new(),
                chunk_type: 3,
            },
            data,
        }];
        let players = [
            FilmPlayer {
                xuid: 1,
                gamertag: "Nuzzles".into(),
            },
            FilmPlayer {
                xuid: 2,
                gamertag: "MsNuzzles".into(),
            },
        ];

        assert_eq!(
            decode_events(&chunks, &players),
            [FilmEvent {
                xuid: 2,
                gamertag: "MsNuzzles".into(),
                timestamp_ms: 12_345,
                kind: FilmEventKind::Kill,
                medal_flag: 0,
                metadata: 42,
            }]
        );
    }

    #[test]
    fn maps_article_medal_ids() {
        assert_eq!(
            FilmMedal::from_id(166),
            FilmMedal::Known {
                id: 166,
                name: "Mounted"
            }
        );
        assert_eq!(FilmMedal::from_id(255), FilmMedal::Unknown(255));
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

    #[test]
    fn decodes_dense_component_mask_component_zero_first() {
        // DELTA, slot 0, tag 0, dense mask with components 0 and 63 set.
        let bits = format!("1{:011b}0011{}1", 0, "0".repeat(62))
            .chars()
            .collect::<Vec<_>>();
        let mut data = vec![0u8; bits.len().div_ceil(8)];
        for (index, bit) in bits.into_iter().enumerate() {
            if bit == '1' {
                data[index / 8] |= 1 << (7 - index % 8);
            }
        }
        let delta = decode_first_delta_header(&data).unwrap();
        assert_eq!(delta.component_mask, 1 | (1 << 63));
    }
}
