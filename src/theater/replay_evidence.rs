//! LegacyFilm-only identity inputs assembled in native scan order.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Parse external roster strings with native RosterXUIDsOf semantics. Preserve
/// order and duplicates; reject signs, whitespace, nondecimal text, overflow
/// and zero. This is distinct from the weapon-index adapter's permissive parser.
pub fn replay_roster_xuids(strings: &[String]) -> Vec<u64> {
    strings
        .iter()
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .filter_map(|s| s.parse::<u64>().ok())
        .filter(|&x| x != 0)
        .collect()
}

/// Sorted nonzero union of death identities and the caller's additional roster.
/// This includes players with zero deaths when the caller supplies their XUIDs.
pub fn replay_identity_roster(deaths: &[IdentityDeath], extra: &[u64]) -> Vec<u64> {
    deaths
        .iter()
        .map(|d| d.xuid)
        .chain(extra.iter().copied())
        .filter(|&x| x != 0)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
/// Native identity scan inputs before collision rejection or bootstrap composition.
/// A missing table means the read was skipped (no deaths) or failed; index_error
/// distinguishes those cases. The raw successful table survives collisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmIdentityInputs {
    pub deaths: Vec<IdentityDeath>,
    pub death_error: Option<String>,
    /// Sorted, deduplicated roster used by the bit-pattern index scan.
    pub roster: Vec<u64>,
    pub raw_player_indices: Option<PlayerIndexTable>,
    pub index_error: Option<String>,
}
impl FilmIdentityInputs {
    /// Native publication guard, applied without modifying retained observations.
    pub fn player_indices(&self) -> (PlayerIndexTable, usize) {
        injective_player_indices(self.raw_player_indices.clone().unwrap_or_default())
    }
}

pub fn scan_film_identity_inputs(
    chunks: &[FilmChunkData],
    major_version: i32,
    extra_roster: &[u64],
) -> Result<FilmIdentityInputs, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    Ok(retain_film_identity_inputs(
        chunks,
        scan_film_deaths(chunks, major_version),
        extra_roster,
    ))
}

pub(super) fn retain_film_identity_inputs(
    chunks: &[FilmChunkData],
    deaths: Result<Vec<IdentityDeath>, DecodeError>,
    extra_roster: &[u64],
) -> FilmIdentityInputs {
    let (deaths, death_error) = match deaths {
        Ok(deaths) => (deaths, None),
        Err(error) => (Vec::new(), Some(error.to_string())),
    };
    let roster = replay_identity_roster(&deaths, extra_roster);
    let mut out = FilmIdentityInputs {
        deaths,
        death_error,
        roster,
        raw_player_indices: None,
        index_error: None,
    };
    if !out.deaths.is_empty() {
        match scan_player_indices(chunks, &out.roster) {
            Ok(table) => out.raw_player_indices = Some(table),
            Err(error) => out.index_error = Some(error.to_string()),
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayIdentityEvidence {
    pub deaths: Vec<IdentityDeath>,
    pub replication_indices: PlayerIndexTable,
    pub index_collisions: usize,
    pub film_clock_us: u64,
    /// Named scan failures retain native nonfatal degradation rather than hiding it.
    pub death_error: Option<String>,
    pub index_error: Option<String>,
    pub clock_error: Option<String>,
}
/// Read deaths, then replication indices from their supplemented roster, then
/// the clock origin. Without deaths, native production does not scan indices.
/// Bootstrap seats are composed later by the shared identity-registry builder.
pub fn scan_replay_identity_evidence(
    chunks: &[FilmChunkData],
    major_version: i32,
    extra_roster: &[u64],
) -> Result<ReplayIdentityEvidence, DecodeError> {
    scan_replay_identity_evidence_with_clock(chunks, major_version, extra_roster, None)
}

pub(super) fn scan_replay_identity_evidence_with_clock(
    chunks: &[FilmChunkData],
    major_version: i32,
    extra_roster: &[u64],
    retained_clock: Option<&ReplayClockOriginRead>,
) -> Result<ReplayIdentityEvidence, DecodeError> {
    scan_replay_identity_evidence_retained(
        chunks,
        major_version,
        extra_roster,
        retained_clock,
        None,
    )
}

pub(super) fn scan_replay_identity_evidence_retained(
    chunks: &[FilmChunkData],
    major_version: i32,
    extra_roster: &[u64],
    retained_clock: Option<&ReplayClockOriginRead>,
    retained_inputs: Option<&FilmIdentityInputs>,
) -> Result<ReplayIdentityEvidence, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    let scanned;
    let inputs = if let Some(inputs) = retained_inputs.filter(|inputs| {
        inputs.deaths.is_empty()
            || inputs.roster == replay_identity_roster(&inputs.deaths, extra_roster)
    }) {
        inputs
    } else {
        scanned = scan_film_identity_inputs(chunks, major_version, extra_roster)?;
        &scanned
    };
    let (replication_indices, index_collisions) = inputs.player_indices();
    let mut out = ReplayIdentityEvidence {
        deaths: inputs.deaths.clone(),
        replication_indices,
        index_collisions,
        film_clock_us: 0,
        death_error: inputs.death_error.clone(),
        index_error: inputs.index_error.clone(),
        clock_error: None,
    };
    match retained_clock
        .map(ReplayClockOriginRead::result)
        .unwrap_or_else(|| scan_replay_clock_origin(chunks))
    {
        Ok(clock) => out.film_clock_us = clock,
        Err(error) => out.clock_error = Some(error.to_string()),
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_roster_string_adapter() {
        #[derive(Deserialize)]
        struct Row {
            strings: Vec<String>,
            expected: Vec<u64>,
        }
        let rows: Vec<Row> =
            serde_json::from_str(include_str!("fixtures/roster-strings-v41.json")).unwrap();
        assert_eq!(rows.len(), 256);
        for (i, row) in rows.into_iter().enumerate() {
            assert_eq!(
                replay_roster_xuids(&row.strings),
                row.expected,
                "roster strings {i}"
            );
        }
    }
    #[derive(Deserialize)]
    struct Chunk {
        index: i32,
        hex: String,
    }
    #[derive(Deserialize)]
    struct Case {
        chunks: Vec<Chunk>,
        extra: Option<Vec<u64>>,
        deaths: Option<Vec<IdentityDeath>>,
        indices: PlayerIndexTable,
        collisions: usize,
        clock: u64,
        death_error: bool,
        index_error: bool,
        clock_error: bool,
        roster: Vec<u64>,
    }
    #[test]
    fn native_automatic_identity_evidence() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-evidence-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let chunks: Vec<_> = c
                .chunks
                .into_iter()
                .map(|ch| {
                    let data: Vec<_> = ch
                        .hex
                        .as_bytes()
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                        .collect();
                    FilmChunkData {
                        data,
                        metadata: crate::clients::hi::models::FilmChunk {
                            index: ch.index,
                            chunk_type: 0,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: 0,
                            file_relative_path: String::new(),
                        },
                    }
                })
                .collect();
            let out =
                scan_replay_identity_evidence(&chunks, 41, c.extra.as_deref().unwrap_or_default())
                    .unwrap();
            let inputs =
                scan_film_identity_inputs(&chunks, 41, c.extra.as_deref().unwrap_or_default())
                    .unwrap();
            let inputs: FilmIdentityInputs =
                serde_json::from_value(serde_json::to_value(inputs).unwrap()).unwrap();
            assert_eq!(inputs.deaths, out.deaths);
            if !inputs.deaths.is_empty() && !c.index_error {
                assert_eq!(
                    inputs.raw_player_indices,
                    Some(scan_player_indices(&chunks, &inputs.roster).unwrap())
                );
            }
            let clock_read = read_replay_clock_origin(&chunks);
            assert_eq!(
                clock_read.timestamp_us(),
                (!c.clock_error).then_some(c.clock)
            );
            assert_eq!(
                serde_json::from_value::<ReplayClockOriginRead>(
                    serde_json::to_value(&clock_read).unwrap()
                )
                .unwrap(),
                clock_read
            );
            let retained = scan_replay_identity_evidence_with_clock(
                &chunks,
                41,
                c.extra.as_deref().unwrap_or_default(),
                Some(&clock_read),
            )
            .unwrap();
            assert_eq!(retained, out, "retained clock {i}");
            assert_eq!(
                scan_replay_identity_evidence_retained(
                    &[],
                    41,
                    c.extra.as_deref().unwrap_or_default(),
                    Some(&clock_read),
                    Some(&inputs),
                )
                .unwrap(),
                out,
                "retained inputs without chunks {i}"
            );
            // A new requested identity must be scanned from source, not silently
            // dropped by reusing a cache built for a different roster.
            let mut extra = c.extra.clone().unwrap_or_default();
            extra.push(0x1234_5678_9abc_def0);
            assert_eq!(
                scan_replay_identity_evidence_retained(
                    &chunks,
                    41,
                    &extra,
                    Some(&clock_read),
                    Some(&inputs),
                )
                .unwrap(),
                scan_replay_identity_evidence(&chunks, 41, &extra).unwrap(),
                "changed roster {i}"
            );
            let exported = serde_json::to_vec(&out).unwrap();
            let restored: ReplayIdentityEvidence = serde_json::from_slice(&exported).unwrap();
            assert_eq!(restored, out, "portable identity evidence {i}");
            assert_eq!(out.deaths, c.deaths.unwrap_or_default(), "deaths {i}");
            assert_eq!(out.replication_indices, c.indices, "indices {i}");
            assert_eq!(out.index_collisions, c.collisions, "collisions {i}");
            assert_eq!(out.film_clock_us, c.clock, "clock {i}");
            assert_eq!(out.death_error.is_some(), c.death_error, "death error {i}");
            assert_eq!(out.index_error.is_some(), c.index_error, "index error {i}");
            assert_eq!(out.clock_error.is_some(), c.clock_error, "clock error {i}");
            assert_eq!(
                replay_identity_roster(&out.deaths, c.extra.as_deref().unwrap_or_default()),
                c.roster,
                "roster {i}"
            );
        }
    }
}
