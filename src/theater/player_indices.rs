//! Native bit-pattern player index reads, restricted to replication chunks.
use super::{DecodeError, PlayerIndexTable, bits::Bits, fire_events::native_chunk_prefix};
use crate::clients::hi::models::FilmChunkData;
use std::collections::{BTreeMap, BTreeSet};

/// Native participant-index width immediately preceding an XUID pattern.
pub const PLAYER_INDEX_BITS: usize = 5;

/// First occurrence of each little-endian XUID, with the five preceding bits.
/// Patterns near the beginning read zero for the prefix outside the buffer.
pub fn resolve_player_indices(roster: &[u64], chunk: &[u8]) -> BTreeMap<u64, i64> {
    resolve_player_index_reads(roster, chunk)
        .into_iter()
        .map(|r| (r.xuid, r.index))
        .collect()
}

/// First matching identity pattern and its five-bit index prefix. Ordering is
/// by source bit position, not roster order; repeated roster entries collapse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerIndexPatternRead {
    pub xuid: u64,
    pub index: i64,
    pub xuid_start_bit: usize,
    /// The prefix may begin before byte zero; those bits are synthetic zeros.
    pub index_start_bit: i64,
    pub leading_padding_bits: usize,
}

pub fn resolve_player_index_reads(roster: &[u64], chunk: &[u8]) -> Vec<PlayerIndexPatternRead> {
    if roster.is_empty() {
        return Vec::new();
    }
    let mut wanted: BTreeMap<_, _> = roster.iter().map(|&x| (x.swap_bytes(), x)).collect();
    let mut out = Vec::new();
    let bits = Bits(chunk);
    for (position, word) in bits.windows().take(bits.len().saturating_sub(63)) {
        if let Some(xuid) = wanted.remove(&word) {
            let width = position.min(PLAYER_INDEX_BITS);
            let index = bits.read(position - width, width).unwrap();
            out.push(PlayerIndexPatternRead {
                xuid,
                index: index as i64,
                xuid_start_bit: position,
                index_start_bit: position as i64 - PLAYER_INDEX_BITS as i64,
                leading_padding_bits: PLAYER_INDEX_BITS - width,
            });
            if wanted.is_empty() {
                break;
            }
        }
    }
    out
}
/// Native decimal-string roster projection. Numeric overflow wraps as in Go;
/// equivalent numeric spellings retain the last original string as their key.
pub fn resolve_player_index_strings(roster: &[String], chunk: &[u8]) -> BTreeMap<String, i64> {
    let mut back = BTreeMap::new();
    for original in roster {
        let s = original.trim();
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let value = s.bytes().fold(0_u64, |v, b| {
            v.wrapping_mul(10).wrapping_add(u64::from(b - b'0'))
        });
        back.insert(value, original.clone());
    }
    resolve_player_indices(&back.keys().copied().collect::<Vec<_>>(), chunk)
        .into_iter()
        .map(|(x, pi)| (back[&x].clone(), pi))
        .collect()
}
/// Merge bit-pattern identities with first-chunk wins, matching ResolveBest.
/// This differs from the disagreement-refusing replication identity scan.
pub fn resolve_best_player_indices(roster: &[u64], chunks: &[&[u8]]) -> BTreeMap<u64, i64> {
    let mut merged = BTreeMap::new();
    for chunk in chunks {
        for (x, pi) in resolve_player_indices(roster, chunk) {
            merged.entry(x).or_insert(pi);
        }
    }
    merged
}

/// Read contiguous numbered chunks in source order, excluding registry and the
/// final highlight chunk. Disagreement removes only that identity, never votes.
pub fn scan_player_indices(
    chunks: &[FilmChunkData],
    roster: &[u64],
) -> Result<PlayerIndexTable, DecodeError> {
    if roster.is_empty() {
        return Err(DecodeError::Missing("player index roster"));
    }
    let selected = native_chunk_prefix(chunks)?;
    let mut out = PlayerIndexTable::default();
    let mut seen = BTreeMap::<u64, BTreeSet<i64>>::new();
    for chunk in &selected[..selected.len() - 1] {
        let got = resolve_player_indices(roster, &chunk.data);
        if got.is_empty() {
            continue;
        }
        out.readings = out.readings.wrapping_add(1);
        for (x, pi) in got {
            seen.entry(x).or_default().insert(pi);
        }
    }
    if out.readings == 0 {
        return Err(DecodeError::Missing("replication player indices"));
    }
    for (x, indices) in seen {
        if indices.len() > 1 {
            out.disagreements = out.disagreements.wrapping_add(1);
        } else {
            out.by_xuid.insert(x, *indices.first().unwrap());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        roster: Vec<u64>,
        chunks: Vec<Chunk>,
        output: PlayerIndexTable,
        error: bool,
    }
    #[derive(Deserialize)]
    struct Chunk {
        index: i32,
        hex: String,
        resolved: BTreeMap<u64, i64>,
    }
    #[test]
    fn native_player_index_scanning() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/player-indices-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let chunks: Vec<_> = c
                .chunks
                .into_iter()
                .map(|ch| {
                    let data: Vec<_> = (0..ch.hex.len())
                        .step_by(2)
                        .map(|p| u8::from_str_radix(&ch.hex[p..p + 2], 16).unwrap())
                        .collect();
                    assert_eq!(
                        resolve_player_indices(&c.roster, &data),
                        ch.resolved,
                        "raw {i}/{}",
                        ch.index
                    );
                    FilmChunkData {
                        metadata: crate::clients::hi::models::FilmChunk {
                            index: ch.index,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                            chunk_type: 0,
                        },
                        data,
                    }
                })
                .collect();
            let out = scan_player_indices(&chunks, &c.roster);
            assert_eq!(out.is_err(), c.error, "error {i}");
            assert_eq!(out.unwrap_or_default(), c.output, "output {i}");
        }
    }
}
