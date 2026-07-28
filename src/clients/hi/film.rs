//! Experimental decoder for Halo Infinite Theater film chunks.

use std::collections::BTreeMap;

use super::models::FilmChunkData;

const PLAYER_MARKER: [u8; 2] = [0x2d, 0xc0];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmPlayer {
    pub xuid: u64,
    pub gamertag: String,
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

    #[test]
    fn finds_unaligned_patterns() {
        let data = [0b1011_0011, 0b0101_1110];
        assert_eq!(find_bit_pattern(&data, &[0b1100_1101]), vec![2]);
    }
}
