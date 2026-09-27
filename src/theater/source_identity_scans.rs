//! Loaded identity-index and packet-clock reads for the native replay bridge.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SourcePlayerIndexError {
    #[error("roster vide : rien à résoudre")]
    EmptyRoster,
    #[error("aucun chunk film lisible")]
    NoChunks,
    #[error("aucun chunk de réplication n'a livré d'index de joueur")]
    NoReadings,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePlayerIndexChunk {
    pub chunk: i64,
    pub source_index: usize,
    pub reads: Vec<PlayerIndexPatternRead>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourcePlayerIndexScan {
    /// Native ScanPlayerIndices always initializes ByXUID, including on error.
    pub table: PlayerIndexTable,
    /// Every readable replication chunk visited, including ones without matches.
    pub chunks: Vec<SourcePlayerIndexChunk>,
    pub error: Option<SourcePlayerIndexError>,
}
/// Native ScanPlayerIndices. Rejects an empty roster before examining the source;
/// scans the numbered prefix except its final highlight chunk. Conflicting
/// readings remove only that identity. Injectivity is a later bridge operation.
pub fn scan_source_player_indices(
    source: Option<&FilmSource>,
    roster: &[u64],
) -> SourcePlayerIndexScan {
    let mut out = SourcePlayerIndexScan::default();
    if roster.is_empty() {
        out.error = Some(SourcePlayerIndexError::EmptyRoster);
        return out;
    }
    let nums = source
        .map(FilmSource::data_chunk_numbers)
        .unwrap_or_default();
    if nums.is_empty() {
        out.error = Some(SourcePlayerIndexError::NoChunks);
        return out;
    }
    let source = source.unwrap();
    let mut seen = BTreeMap::<u64, BTreeSet<i64>>::new();
    for &chunk in &nums[..nums.len() - 1] {
        let Some((raw, _)) = source.chunk_by_number(chunk) else {
            continue;
        };
        let reads = resolve_player_index_reads(roster, raw);
        if !reads.is_empty() {
            out.table.readings = out.table.readings.wrapping_add(1);
        }
        for r in &reads {
            seen.entry(r.xuid).or_default().insert(r.index);
        }
        out.chunks.push(SourcePlayerIndexChunk {
            chunk,
            source_index: source.chunk_position(chunk).unwrap(),
            reads,
        });
    }
    if out.table.readings == 0 {
        out.error = Some(SourcePlayerIndexError::NoReadings);
        return out;
    }
    for (xuid, indices) in seen {
        if indices.len() > 1 {
            out.table.disagreements = out.table.disagreements.wrapping_add(1);
        } else {
            out.table.by_xuid.insert(xuid, *indices.first().unwrap());
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SourceClockOriginError {
    #[error("chunk 1 (origine d'horloge) : absent du film")]
    NoChunk,
    #[error("chunk 1 (origine d'horloge) : aucun paquet lisible")]
    NoPacket,
}
/// Native ScanClockOrigin, retaining the packet that established the timestamp.
/// This direct number-one lookup is independent of contiguous-prefix discovery.
/// Zero is a successful recorded timestamp; source buffer identity stays intact.
pub fn scan_source_clock_origin(
    source: Option<&FilmSource>,
) -> Result<FilmPacket, SourceClockOriginError> {
    let (_, packets) = source
        .and_then(|s| s.chunk_by_number(1))
        .ok_or(SourceClockOriginError::NoChunk)?;
    packets
        .first()
        .copied()
        .ok_or(SourceClockOriginError::NoPacket)
}
