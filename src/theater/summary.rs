//! Version-41 summary events, decoded using only the decompressed footer chunks.
//!
//! The 3,667 events in the 32-film corpus share the same identity-to-tail offset.
//! This is a guarded captured layout, not a general decoder for the intervening state.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{
    DecodeError, FilmEventCounts, MedalAward, SourceSpan, SummaryEvent, SummaryKind, bits::Bits,
};
use crate::clients::hi::models::{FilmChunkData, MatchStats};

const IDENTITY_TO_TAIL_BITS: usize = 14_926;
const TAIL_BITS: usize = 60 * 8;
const END_MARKER: u64 = 0x0000_2ee0;

/// Counts declared by a summary packet, compared with accepted event records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryPacketDiagnostics {
    pub chunk: i32,
    pub payload_byte: usize,
    pub declared_events: u32,
    pub decoded_events: usize,
}

/// Footer-only decode, independent of the bootstrap registry or motion decoder.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryEventReport {
    pub events: Vec<SummaryEvent>,
    pub packets: Vec<SummaryPacketDiagnostics>,
    /// Unsupported packet types in footer chunks. Their payloads are not interpreted.
    pub unparsed_packet_types: BTreeMap<u16, usize>,
}

impl SummaryEventReport {
    /// All declared summary events were found. A missing footer is not a complete empty match.
    /// This does not claim to decode every event payload or all gameplay actions.
    pub fn matches_declared_counts(&self) -> bool {
        !self.packets.is_empty()
            && self
                .packets
                .iter()
                .all(|p| p.decoded_events == p.declared_events as usize)
    }
}

/// One medal identity compared with the independent stats API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MedalCountComparison {
    pub name_id: u32,
    pub decoded: usize,
    pub match_stats: usize,
}

/// Per-player totals and medal identities. Equal total medal counts alone are insufficient.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryPlayerValidation {
    pub xuid: String,
    pub name: Option<String>,
    pub decoded: FilmEventCounts,
    pub match_stats: FilmEventCounts,
    pub medals: Vec<MedalCountComparison>,
    pub unknown_medal_codes: Vec<u8>,
}

impl SummaryPlayerValidation {
    pub fn matches_stats(&self) -> bool {
        self.decoded == self.match_stats
            && self.unknown_medal_codes.is_empty()
            && self.medals.iter().all(|m| m.decoded == m.match_stats)
    }
}

/// Independent count and medal-identity validation; mismatches remain inspectable.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryEventValidation {
    pub declared_counts_match: bool,
    pub players: Vec<SummaryPlayerValidation>,
}

/// Stored highlights with their independent match-statistics comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedSummaryEventReport {
    pub summary: SummaryEventReport,
    pub validation: SummaryEventValidation,
}

impl SummaryEventValidation {
    pub fn matches_stats(&self) -> bool {
        self.declared_counts_match
            && self
                .players
                .iter()
                .all(SummaryPlayerValidation::matches_stats)
    }
}

/// Compare every human player's kills, deaths and per-NameId medal counts.
/// Mode highlights do not imply a particular objective action or assist count.
pub fn validate_summary_events(
    report: &SummaryEventReport,
    stats: &MatchStats,
) -> SummaryEventValidation {
    #[derive(Default)]
    struct Counts {
        name: Option<String>,
        decoded: FilmEventCounts,
        expected: FilmEventCounts,
        medals: BTreeMap<u32, (usize, usize)>,
        unknown: BTreeSet<u8>,
    }
    let mut players = BTreeMap::<String, Counts>::new();
    for event in &report.events {
        let p = players.entry(event.xuid.clone()).or_default();
        p.name = Some(event.name.clone());
        match event.kind {
            SummaryKind::Kill => p.decoded.kills += 1,
            SummaryKind::Death => p.decoded.deaths += 1,
            SummaryKind::Medal => {
                p.decoded.medals += 1;
                if let Some(id) =
                    super::film_medal_definition(event.metadata).and_then(|m| m.name_id)
                {
                    p.medals.entry(id).or_default().0 += 1;
                } else {
                    p.unknown.insert(event.metadata);
                }
            }
            _ => {}
        }
    }
    for player in stats.players.iter().filter(|p| p.is_human()) {
        let Some(id) = player
            .player_id
            .strip_prefix("xuid(")
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.parse::<u64>().ok())
        else {
            continue;
        };
        let p = players.entry(id.to_string()).or_default();
        for team in &player.team_stats {
            let core = &team.stats.core;
            p.expected.kills += usize::try_from(core.kills).unwrap_or(0);
            p.expected.deaths += usize::try_from(core.deaths).unwrap_or(0);
            for medal in &core.medals {
                let count = usize::try_from(medal.count).unwrap_or(0);
                p.expected.medals += count;
                if let Ok(id) = u32::try_from(medal.name_id) {
                    p.medals.entry(id).or_default().1 += count;
                }
            }
        }
    }
    SummaryEventValidation {
        declared_counts_match: report.matches_declared_counts(),
        players: players
            .into_iter()
            .map(|(xuid, p)| SummaryPlayerValidation {
                xuid,
                name: p.name,
                decoded: p.decoded,
                match_stats: p.expected,
                medals: p
                    .medals
                    .into_iter()
                    .map(|(name_id, (decoded, match_stats))| MedalCountComparison {
                        name_id,
                        decoded,
                        match_stats,
                    })
                    .collect(),
                unknown_medal_codes: p.unknown.into_iter().collect(),
            })
            .collect(),
    }
}

fn source(chunk: i32, payload_byte: usize, bit: usize, length: usize) -> SourceSpan {
    SourceSpan {
        chunk,
        payload_byte,
        bit,
        end_bit: bit + length,
    }
}

fn read_event(
    bits: Bits<'_>,
    marker_bit: usize,
    chunk: i32,
    payload_byte: usize,
) -> Option<SummaryEvent> {
    let identity_bit = marker_bit.checked_sub(64)?;
    let xuid = bits.read(identity_bit, 64)?.swap_bytes();
    if xuid == 0 {
        return None;
    }
    let tail = identity_bit.checked_add(IDENTITY_TO_TAIL_BITS)?;
    let end = tail.checked_add(TAIL_BITS)?;
    if bits.read(end, 32)? != END_MARKER {
        return None;
    }
    // The two reserved three-byte fields and boolean medal flag independently guard the tail.
    if bits.read(tail + 52 * 8, 24)? != 0 || bits.read(tail + 56 * 8, 24)? != 0 {
        return None;
    }
    let medal_flag = bits.read(tail + 55 * 8, 8)? as u8;
    if medal_flag > 1 {
        return None;
    }
    let mut units = Vec::with_capacity(16);
    let mut terminated = false;
    for i in 0..16 {
        let unit = (bits.read(tail + i * 16, 16)? as u16).swap_bytes();
        if unit == 0 {
            terminated = true;
        } else if terminated {
            return None;
        } else {
            units.push(unit);
        }
    }
    let name = String::from_utf16(&units).ok()?;
    if name.is_empty() || name.chars().any(char::is_control) {
        return None;
    }
    let type_code = bits.read(tail + 47 * 8, 8)? as u8;
    let metadata = bits.read(tail + 59 * 8, 8)? as u8;
    let time_us = bits.read(tail + 48 * 8, 32)? * 1000;
    let kind = SummaryKind::from_fields(type_code, medal_flag);
    Some(SummaryEvent {
        xuid: xuid.to_string(),
        player: None,
        name,
        time_us,
        kind,
        metadata,
        medal_flag,
        type_code: Some(type_code),
        medal: (medal_flag == 1).then(|| MedalAward::from_film_id(metadata)),
        source: Some(source(chunk, payload_byte, tail, TAIL_BITS)),
        identity_source: Some(source(chunk, payload_byte, identity_bit, 64)),
    })
}

/// Decode the stored human-player highlight timeline from version-41 footer chunks.
///
/// Input must be decompressed. Only manifest chunk type 3 is needed. Events retain
/// their recorded XUID and gamertag without a network/roster lookup. Kills and deaths
/// are not paired; mode subtypes and the state between identity and event remain opaque.
/// Unknown event and medal codes are retained. Distinct source records are never deduplicated
/// merely because their player, time and medal happen to match.
pub fn decode_summary_events(
    chunks: &[FilmChunkData],
    major_version: i32,
) -> Result<SummaryEventReport, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    let mut report = SummaryEventReport::default();
    let mut ids = BTreeSet::new();
    let mut footers: Vec<_> = chunks
        .iter()
        .filter(|c| c.metadata.chunk_type == 3)
        .collect();
    footers.sort_by_key(|c| c.metadata.index);
    for chunk in footers {
        let id = chunk.metadata.index;
        if !ids.insert(id) {
            return Err(DecodeError::DuplicateChunk(id));
        }
        let mut offset = 0usize;
        while offset < chunk.data.len() {
            let bad = || DecodeError::Truncated { chunk: id, offset };
            let header = chunk
                .data
                .get(offset..offset.saturating_add(16))
                .ok_or_else(bad)?;
            let kind = u16::from_le_bytes([header[0], header[1]]);
            let size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
            let payload_byte = offset.checked_add(16).ok_or_else(bad)?;
            let end = payload_byte.checked_add(size).ok_or_else(bad)?;
            let payload = chunk.data.get(payload_byte..end).ok_or_else(bad)?;
            match kind {
                9 => {
                    let count = payload.get(..4).ok_or_else(bad)?;
                    let declared_events = u32::from_be_bytes(count.try_into().unwrap());
                    let before = report.events.len();
                    let bits = Bits(payload);
                    for (bit, word) in bits.windows() {
                        if bit < 32 + 64 || !matches!(word >> 48, 0x2dc0 | 0x25c0) {
                            continue;
                        }
                        if let Some(event) = read_event(bits, bit, id, payload_byte) {
                            report.events.push(event);
                        }
                    }
                    report.packets.push(SummaryPacketDiagnostics {
                        chunk: id,
                        payload_byte,
                        declared_events,
                        decoded_events: report.events.len() - before,
                    });
                }
                7 if size == 0 => {}
                other => {
                    *report.unparsed_packet_types.entry(other).or_default() += 1;
                }
            }
            offset = end;
        }
    }
    report.events.sort_by_key(|e| {
        (
            e.time_us,
            e.source.map(|s| (s.chunk, s.payload_byte, s.bit)),
        )
    });
    Ok(report)
}

#[cfg(test)]
mod tests;
