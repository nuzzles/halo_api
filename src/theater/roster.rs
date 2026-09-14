//! Recorded player identities and their replication roster indices.

use std::collections::BTreeMap;

use super::bits::Bits;
use crate::clients::hi::models::FilmChunkData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmPlayer {
    pub xuid: u64,
    pub gamertag: String,
}

/// Read recorded identities from the checked name/padding/XUID/marker layout.
/// Summary identities are decoded independently by `decode_summary_events`.
pub fn decode_players(chunks: &[FilmChunkData]) -> Vec<FilmPlayer> {
    let mut players = BTreeMap::new();
    for chunk in chunks.iter().filter(|c| c.metadata.chunk_type != 3) {
        let bits = Bits(&chunk.data);
        // Preserve marker precedence when a recording repeats an identity.
        for marker in [0x2dc0, 0x25c0] {
            for (position, word) in bits.windows() {
                if word >> 48 != marker || bits.read(position, 16) != Some(marker) {
                    continue;
                }
                let Some(name_start) = position.checked_sub((32 + 21 + 8) * 8) else {
                    continue;
                };
                let padding = name_start + 32 * 8;
                if bits.read(padding, 64) != Some(0)
                    || bits.read(padding + 64, 64) != Some(0)
                    || bits.read(padding + 128, 40) != Some(0)
                {
                    continue;
                }
                let xuid = bits.read(position - 64, 64).unwrap().swap_bytes();
                if xuid == 0 {
                    continue;
                }
                let units: Vec<_> = (0..16)
                    .map(|i| (bits.read(name_start + i * 16, 16).unwrap() as u16).swap_bytes())
                    .collect();
                let name = String::from_utf16_lossy(&units)
                    .trim_matches('\0')
                    .trim()
                    .to_owned();
                if !name.is_empty() {
                    players.entry(xuid).or_insert(name);
                }
            }
        }
    }
    players
        .into_iter()
        .map(|(xuid, gamertag)| FilmPlayer { xuid, gamertag })
        .collect()
}

/// Find the recorded five-bit replication index preceding each known XUID.
pub fn decode_player_indices(
    chunks: &[FilmChunkData],
    players: &[FilmPlayer],
) -> BTreeMap<u64, u8> {
    let mut indices = BTreeMap::new();
    for chunk in chunks.iter().filter(|c| c.metadata.chunk_type == 2) {
        let bits = Bits(&chunk.data);
        for player in players {
            if indices.contains_key(&player.xuid) {
                continue;
            }
            let Some((position, _)) = bits.windows().find(|&(position, word)| {
                word == player.xuid.swap_bytes() && position + 64 <= bits.len()
            }) else {
                continue;
            };
            if let Some(start) = position.checked_sub(5) {
                indices.insert(player.xuid, bits.read(start, 5).unwrap() as u8);
            }
        }
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;

    #[test]
    fn both_markers_and_roster_indices_work_at_every_bit_alignment() {
        for shift in 0..8 {
            let mut data = vec![0u8; 140];
            for (i, (xuid, name, marker)) in [
                (1u64, "Nuzzles", [0x2d, 0xc0]),
                (2, "MsNuzzles", [0x25, 0xc0]),
            ]
            .into_iter()
            .enumerate()
            {
                let start = shift + i * 64 * 8;
                let mut record = vec![0; 32 + 21];
                for (j, unit) in name.encode_utf16().enumerate() {
                    record[j * 2..j * 2 + 2].copy_from_slice(&unit.to_le_bytes());
                }
                // Roster index directly precedes the XUID; padding ends with zero here.
                record.extend(xuid.to_le_bytes());
                record.extend(marker);
                for (bit, set) in record
                    .iter()
                    .flat_map(|byte| (0..8).map(move |i| (byte >> (7 - i)) & 1))
                    .enumerate()
                {
                    data[(start + bit) / 8] |= set << (7 - (start + bit) % 8);
                }
            }
            let chunks = [FilmChunkData {
                data,
                metadata: FilmChunk {
                    index: 1,
                    chunk_type: 2,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: 0,
                    file_relative_path: String::new(),
                },
            }];
            let players = decode_players(&chunks);
            assert_eq!(
                players,
                [
                    FilmPlayer {
                        xuid: 1,
                        gamertag: "Nuzzles".into()
                    },
                    FilmPlayer {
                        xuid: 2,
                        gamertag: "MsNuzzles".into()
                    }
                ]
            );
            assert_eq!(
                decode_player_indices(&chunks, &players),
                BTreeMap::from([(1, 0), (2, 0)])
            );
        }
    }

    #[test]
    fn first_recorded_nonzero_index_wins_at_every_bit_alignment() {
        let players = [
            FilmPlayer {
                xuid: 2535472547643888,
                gamertag: "Nuzzles".into(),
            },
            FilmPlayer {
                xuid: 2535443507298499,
                gamertag: "Yet".into(),
            },
        ];
        for shift in 0..8 {
            let mut data = vec![0u8; 40];
            let mut put = |start: usize, width: usize, value: u64| {
                for i in 0..width {
                    data[(start + i) / 8] |=
                        (((value >> (width - i - 1)) & 1) as u8) << (7 - (start + i) % 8);
                }
            };
            for (i, (xuid, index)) in [
                (players[0].xuid, 3),
                (players[1].xuid, 19),
                (players[0].xuid, 7),
            ]
            .into_iter()
            .enumerate()
            {
                let start = shift + i * 80;
                put(start, 5, index);
                put(start + 5, 64, xuid.swap_bytes());
            }
            let chunk = FilmChunkData {
                data,
                metadata: FilmChunk {
                    index: 1,
                    chunk_type: 2,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: 0,
                    file_relative_path: String::new(),
                },
            };
            assert_eq!(
                decode_player_indices(&[chunk], &players),
                BTreeMap::from([(players[0].xuid, 3), (players[1].xuid, 19)])
            );
        }
    }
}
