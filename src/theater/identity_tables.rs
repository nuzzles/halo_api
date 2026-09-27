//! Native composition of bootstrap player seats and replication player indices.
use super::{PlayerTable, PlayerTableError, ReplayByteString};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerIndexTable {
    #[serde(rename = "ByXUID")]
    pub by_xuid: BTreeMap<u64, i64>,
    pub readings: i64,
    pub disagreements: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayFilmPlayerSeat {
    pub film_index: i64,
    #[serde(rename = "XUID")]
    pub xuid: u64,
    pub gamertag: super::ReplayByteString,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayFilmPlayerTable {
    pub seats: Vec<ReplayFilmPlayerSeat>,
    pub build: ReplayByteString,
    pub occupied: i64,
    pub vacant: i64,
    pub interleaved_vacant: bool,
    pub refusal: ReplayByteString,
}
impl ReplayFilmPlayerTable {
    pub fn from_decoded(table: &PlayerTable) -> Self {
        if let Some(error) = &table.error {
            return Self {
                build: table.report.build.clone().into(),
                refusal: match error {
                    PlayerTableError::UnknownBuild => "build_inconnu",
                    PlayerTableError::Truncated => "tronque",
                    PlayerTableError::NotFound => "table_introuvable",
                }
                .into(),
                ..Default::default()
            };
        }
        Self {
            seats: table
                .slots
                .iter()
                .map(|s| ReplayFilmPlayerSeat {
                    film_index: s.film_index as i64,
                    xuid: s.xuid,
                    gamertag: s.gamertag.clone().into(),
                })
                .collect(),
            build: table.report.build.clone().into(),
            occupied: table.report.occupied as i64,
            vacant: table.report.vacant as i64,
            interleaved_vacant: table.report.interleaved_vacant,
            refusal: ReplayByteString::default(),
        }
    }
    pub fn is_read(&self) -> bool {
        self.refusal.is_empty() && !self.seats.is_empty() && !self.interleaved_vacant
    }
    /// Publish the native source-table observations. A successful but interleaved
    /// table still logs its measured counts before warning about identity ambiguity.
    /// Native nil errors are omitted by tracing; the diagnostic retains None.
    pub fn log(&self, match_id: &str, diagnostics: &ReplayFilmPlayerTableDiagnostics) {
        if !self.refusal.is_empty() {
            let error = diagnostics.native_error.as_ref().map(ToString::to_string);
            tracing::warn!(
                match_id,
                build = self.build.json_text().as_str(),
                cause = self.refusal.json_text().as_str(),
                err = error.as_deref(),
                "rejeu : table des joueurs du film NON LUE — le registre d'identite retombe sur la lecture des chunks de replication"
            );
            return;
        }
        tracing::info!(
            match_id,
            build = self.build.json_text().as_str(),
            sieges = self.occupied,
            vacants = self.vacant,
            vacantIntercale = self.interleaved_vacant,
            "rejeu : table des joueurs du film lue"
        );
        if self.interleaved_vacant {
            tracing::warn!(
                match_id,
                build = self.build.json_text().as_str(),
                "rejeu : siege VACANT INTERCALE dans la table du film — le rang absolu et l'index parmi les occupes divergent, le lien direct n'est PAS affirme"
            );
        }
    }
}

/// Native failure published by the source player-table adapter. Missing registry
/// has no underlying decoder error and is represented by None on the read result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum ReplayFilmPlayerTableError {
    #[error(
        "filmdec: chunk_00 (registre) encore compresse — decompresser avant ParseRegistryChunk"
    )]
    StillCompressed,
    #[error(
        "filmdec: chunk_00 sans section d'identification (aucune chaine de build apres le registre)"
    )]
    NoIdentity,
    #[error("filmdec: chunk_00 tronque avant la fin de la section d'identification")]
    Truncated,
    #[error("filmdec: build absent de la table de profil : {0:?}")]
    UnknownBuild(String),
    #[error("filmdec: aucune table de 32 slots dans le corps de chunk_00")]
    TableNotFound,
}

/// Source-table projection and native refusal diagnostic. Metric increments are
/// per-call publication data; this API does not mutate global counters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFilmPlayerTableRead {
    pub table: ReplayFilmPlayerTable,
    pub diagnostics: ReplayFilmPlayerTableDiagnostics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFilmPlayerTableDiagnostics {
    pub native_error: Option<ReplayFilmPlayerTableError>,
    pub metric_increments: Vec<(String, i64)>,
}

impl ReplayFilmPlayerTableDiagnostics {
    /// No identity means no player-table attempt. Successful attempts retain a
    /// measured absence of errors; missing diagnostics in old LegacyFilm JSON remain None.
    pub fn from_decoded(table: Option<&PlayerTable>) -> Self {
        let Some(table) = table else {
            return Self {
                native_error: Some(ReplayFilmPlayerTableError::NoIdentity),
                metric_increments: Vec::new(),
            };
        };
        let native_error = table.error.as_ref().map(|error| match error {
            PlayerTableError::UnknownBuild => {
                ReplayFilmPlayerTableError::UnknownBuild(table.report.build.clone())
            }
            PlayerTableError::Truncated => ReplayFilmPlayerTableError::Truncated,
            PlayerTableError::NotFound => ReplayFilmPlayerTableError::TableNotFound,
        });
        let metric_increments = if table.error == Some(PlayerTableError::UnknownBuild) {
            super::unknown_build_metric_pairs(&table.report.build).into()
        } else {
            Vec::new()
        };
        Self {
            native_error,
            metric_increments,
        }
    }
}

/// Native source-level player table projection. Use the diagnostic variant when
/// the underlying refusal error and metric publication are needed as well.
pub fn scan_replay_film_player_table(
    source: Option<&super::FilmSource>,
) -> Result<ReplayFilmPlayerTable, super::DecodeError> {
    Ok(scan_replay_film_player_table_with_diagnostics(source)?.table)
}

/// Select bootstrap number zero through source metadata, falling back to position
/// zero without metadata. Named refusals and native decoder errors remain data;
/// unsupported major versions still fail. No logs or global counters are emitted.
pub fn scan_replay_film_player_table_with_diagnostics(
    source: Option<&super::FilmSource>,
) -> Result<ReplayFilmPlayerTableRead, super::DecodeError> {
    let refused = |reason: &str, native_error| ReplayFilmPlayerTableRead {
        table: ReplayFilmPlayerTable {
            refusal: reason.into(),
            ..Default::default()
        },
        diagnostics: ReplayFilmPlayerTableDiagnostics {
            native_error,
            metric_increments: Vec::new(),
        },
    };
    let Some(data) = source.and_then(super::FilmSource::registry_chunk) else {
        return Ok(refused("sans_registre", None));
    };
    let Ok(parsed) = super::parse_registry_chunk(data) else {
        return Ok(refused(
            "tronque",
            Some(ReplayFilmPlayerTableError::StillCompressed),
        ));
    };
    if parsed.registry.truncated {
        return Ok(refused(
            "tronque",
            Some(ReplayFilmPlayerTableError::Truncated),
        ));
    }
    match super::decode_film_identity(data, &parsed.registry) {
        Ok(Some(identity)) => {
            let table = super::decode_player_table(data, &identity);
            Ok(ReplayFilmPlayerTableRead {
                table: ReplayFilmPlayerTable::from_decoded(&table),
                diagnostics: ReplayFilmPlayerTableDiagnostics::from_decoded(Some(&table)),
            })
        }
        Ok(None) => Ok(refused(
            "sans_section",
            Some(ReplayFilmPlayerTableError::NoIdentity),
        )),
        Err(error @ super::DecodeError::UnsupportedVersion(_)) => Err(error),
        Err(_) => Ok(refused(
            "tronque",
            Some(ReplayFilmPlayerTableError::Truncated),
        )),
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityFilmTableCoverage {
    #[serde(rename = "lu")]
    pub read: bool,
    #[serde(
        rename = "refus",
        default,
        skip_serializing_if = "ReplayByteString::is_empty"
    )]
    pub refusal: ReplayByteString,
    #[serde(rename = "sieges")]
    pub seats: i64,
    pub direct: i64,
    #[serde(rename = "repli")]
    pub fallback: i64,
    pub accord: usize,
    pub contradiction: usize,
    pub silence: usize,
    #[serde(rename = "collisionsIndex", default, skip_serializing_if = "is_zero")]
    pub index_collisions: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityTableLinks {
    pub table: PlayerIndexTable,
    pub methods: BTreeMap<u64, String>,
    pub names: BTreeMap<u64, super::ReplayByteString>,
    pub coverage: IdentityFilmTableCoverage,
}
impl IdentityTableLinks {
    pub fn method(&self, xuid: u64) -> &str {
        self.methods
            .get(&xuid)
            .map(String::as_str)
            .unwrap_or("PlayerIndexTable")
    }
}
/// Bootstrap seats take precedence on disagreement. Replication adds only missing
/// identities. Shared indices are then removed for every claimant; names survive.
pub fn compose_identity_tables(
    film: &ReplayFilmPlayerTable,
    replication: &PlayerIndexTable,
) -> IdentityTableLinks {
    let mut out = IdentityTableLinks {
        table: PlayerIndexTable {
            readings: replication.readings,
            disagreements: replication.disagreements,
            ..Default::default()
        },
        methods: BTreeMap::new(),
        names: BTreeMap::new(),
        coverage: IdentityFilmTableCoverage {
            read: film.is_read(),
            refusal: if film.interleaved_vacant && film.refusal.is_empty() {
                "vacant_intercale".into()
            } else {
                film.refusal.clone()
            },
            seats: film.occupied,
            ..Default::default()
        },
    };
    if out.coverage.read {
        for seat in &film.seats {
            if seat.xuid == 0 {
                continue;
            }
            out.table.by_xuid.insert(seat.xuid, seat.film_index);
            out.methods.insert(seat.xuid, "film_table".into());
            out.coverage.direct += 1;
            if !seat.gamertag.is_empty() {
                out.names.insert(seat.xuid, seat.gamertag.clone());
            }
            match replication.by_xuid.get(&seat.xuid) {
                None => out.coverage.silence += 1,
                Some(&i) if i == seat.film_index => out.coverage.accord += 1,
                _ => out.coverage.contradiction += 1,
            }
        }
    }
    for (&xuid, &index) in &replication.by_xuid {
        if let std::collections::btree_map::Entry::Vacant(e) = out.table.by_xuid.entry(xuid) {
            e.insert(index);
            out.methods.insert(xuid, "PlayerIndexTable".into());
            out.coverage.fallback += 1;
        }
    }
    let mut by_index = BTreeMap::<i64, Vec<u64>>::new();
    for (&x, &i) in &out.table.by_xuid {
        by_index.entry(i).or_default().push(x);
    }
    for group in by_index.values().filter(|v| v.len() > 1) {
        out.coverage.index_collisions += 1;
        for x in group {
            if out.methods[x] == "film_table" {
                out.coverage.direct -= 1;
            } else {
                out.coverage.fallback -= 1;
            }
            out.table.by_xuid.remove(x);
            out.methods.remove(x);
        }
    }
    out
}
fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// Native pre-composition guard rejects the entire replication table on collision.
pub fn injective_player_indices(mut table: PlayerIndexTable) -> (PlayerIndexTable, usize) {
    let mut counts = BTreeMap::<i64, usize>::new();
    for &index in table.by_xuid.values() {
        *counts.entry(index).or_default() += 1;
    }
    let collisions = counts.values().filter(|&&n| n > 1).count();
    if collisions > 0 {
        table.by_xuid.clear();
    }
    (table, collisions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_film_player_table_source() {
        use crate::theater::{FilmSource, FilmSourceMetadata};
        #[derive(Deserialize)]
        struct Row {
            run: usize,
            cut: i64,
            build_offset: usize,
            erase: bool,
            unknown: bool,
            table: ReplayFilmPlayerTable,
            read: bool,
            observations: Vec<serde_json::Value>,
            metric_increments: BTreeMap<String, i64>,
            synthetic_row: usize,
            body_bit: usize,
        }
        let mut base = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/film-player-table-source-input-v41.zlib")[..],
        )
        .read_to_end(&mut base)
        .unwrap();
        let rows: Vec<Row> =
            serde_json::from_slice(include_bytes!("fixtures/film-player-table-source-v41.json"))
                .unwrap();
        for row in rows {
            let mut data = base.clone();
            if row.run == 20 {
                let mut slot_json = Vec::new();
                flate2::read::ZlibDecoder::new(
                    &include_bytes!("fixtures/player-table-v41.json.zlib")[..],
                )
                .read_to_end(&mut slot_json)
                .unwrap();
                let slots: Vec<serde_json::Value> = serde_json::from_slice(&slot_json).unwrap();
                let hex = slots[row.synthetic_row]["hex"].as_str().unwrap();
                let body: Vec<u8> = (0..hex.len())
                    .step_by(2)
                    .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
                    .collect();
                let mut joined = vec![0; (row.body_bit + body.len() * 8 - 8).div_ceil(8)];
                for bit in 0..row.body_bit {
                    joined[bit / 8] |= ((data[crate::theater::bits::native_address(bit / 8)]
                        >> (7 - bit % 8))
                        & 1)
                        << (7 - bit % 8);
                }
                for bit in 8..body.len() * 8 {
                    let at = row.body_bit + bit - 8;
                    joined[at / 8] |= ((body[bit / 8] >> (7 - bit % 8)) & 1) << (7 - at % 8);
                }
                data = joined;
            }
            if row.run == 19 {
                use std::io::Write;
                for _ in 0..2 {
                    let mut encoder =
                        flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
                    encoder.write_all(&data).unwrap();
                    data = encoder.finish().unwrap();
                }
            }
            if row.run == 1 {
                data.clear();
            }
            if row.cut >= 0 {
                data.truncate(row.cut as usize);
            }
            if row.erase {
                data[row.build_offset..row.build_offset + 3].copy_from_slice(b"---");
            }
            if row.unknown {
                data[row.build_offset..row.build_offset + 32].fill(0);
                data[row.build_offset..row.build_offset + 10].copy_from_slice(b"HI_UNKNOWN");
            }
            let mut chunks = vec![data];
            let mut metadata = Vec::new();
            if row.run == 16 {
                metadata.push(FilmSourceMetadata {
                    index: 7,
                    ..Default::default()
                });
            }
            if matches!(row.run, 17 | 18) {
                chunks.insert(0, vec![]);
                metadata = vec![
                    FilmSourceMetadata {
                        index: if row.run == 17 { 9 } else { 0 },
                        ..Default::default()
                    },
                    FilmSourceMetadata::default(),
                ];
            }
            let source = (row.run != 0).then(|| FilmSource::load(&chunks, &metadata).unwrap());
            let actual = scan_replay_film_player_table(source.as_ref()).unwrap();
            assert_eq!(actual, row.table, "case {}", row.run);
            assert_eq!(actual.is_read(), row.read, "case {}", row.run);
            let detailed = scan_replay_film_player_table_with_diagnostics(source.as_ref()).unwrap();
            assert_eq!(detailed.table, actual);
            let normalize = |mut logs: Vec<serde_json::Value>| {
                for event in &mut logs {
                    if event.get("cause").is_some() && event.get("err").is_none() {
                        event
                            .as_object_mut()
                            .unwrap()
                            .insert("err".into(), serde_json::Value::Null);
                    }
                }
                logs
            };
            let expected_logs: Vec<_> = row
                .observations
                .iter()
                .filter(|event| event["match_id"] == "rust-source-oracle")
                .cloned()
                .collect();
            let logs = crate::theater::log_test_support::capture_logs(|| {
                detailed
                    .table
                    .log("rust-source-oracle", &detailed.diagnostics);
            });
            assert_eq!(
                normalize(logs),
                expected_logs,
                "source publication {}",
                row.run
            );
            let published = row
                .observations
                .iter()
                .find(|event| event["match_id"] == "rust-source-oracle")
                .unwrap();
            assert_eq!(
                serde_json::to_value(
                    detailed
                        .diagnostics
                        .native_error
                        .as_ref()
                        .map(ToString::to_string)
                )
                .unwrap(),
                published
                    .get("err")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
                "native error {}",
                row.run,
            );
            assert_eq!(
                detailed
                    .diagnostics
                    .metric_increments
                    .iter()
                    .cloned()
                    .collect::<BTreeMap<_, _>>(),
                row.metric_increments
            );
            let detailed_json = serde_json::to_vec(&detailed).unwrap();
            assert_eq!(
                serde_json::from_slice::<ReplayFilmPlayerTableRead>(&detailed_json).unwrap(),
                detailed
            );
            if matches!(row.run, 9 | 10 | 15 | 20) {
                use crate::{
                    clients::hi::models::{FilmChunk, FilmChunkData},
                    theater::{DecodeOptions, LegacyFilm},
                };
                let mut packet = vec![0; 16];
                packet[4..8].copy_from_slice(&1u32.to_le_bytes());
                packet[8..16].copy_from_slice(&1000u64.to_le_bytes());
                packet.push(0);
                let film_chunks: Vec<_> = [chunks[0].clone(), packet]
                    .into_iter()
                    .enumerate()
                    .map(|(index, data)| FilmChunkData {
                        metadata: FilmChunk {
                            index: index as i32,
                            chunk_type: if index == 0 { 1 } else { 2 },
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    })
                    .collect();
                let mut options = DecodeOptions::v41();
                options.match_id = Some("rust-source-oracle".into());
                let mut decoded = None;
                let logs = crate::theater::log_test_support::capture_logs(|| {
                    decoded = Some(LegacyFilm::try_from_chunks(&film_chunks, options).unwrap());
                });
                let film = decoded.unwrap();
                let logs: Vec<_> = logs
                    .into_iter()
                    .filter(|event| {
                        event["msg"].as_str().is_some_and(|message| {
                            message.starts_with("rejeu : table des joueurs du film")
                                || message.starts_with("rejeu : siege VACANT INTERCALE")
                        })
                    })
                    .collect();
                assert_eq!(
                    normalize(logs),
                    expected_logs,
                    "Film publication {}",
                    row.run
                );
                assert_eq!(
                    film.player_table_diagnostics.as_ref(),
                    Some(&detailed.diagnostics)
                );
                let mut json = serde_json::to_value(&film).unwrap();
                assert_eq!(
                    serde_json::from_value::<LegacyFilm>(json.clone()).unwrap(),
                    film
                );
                json.as_object_mut()
                    .unwrap()
                    .remove("player_table_diagnostics");
                assert!(
                    serde_json::from_value::<LegacyFilm>(json)
                        .unwrap()
                        .player_table_diagnostics
                        .is_none()
                );
            }
            let json = serde_json::to_vec(&actual).unwrap();
            assert_eq!(
                serde_json::from_slice::<ReplayFilmPlayerTable>(&json).unwrap(),
                actual
            );
        }
    }
    #[derive(Deserialize)]
    struct Case {
        film: ReplayFilmPlayerTable,
        replication: PlayerIndexTable,
        output: IdentityTableLinks,
        guard: PlayerIndexTable,
        collisions: usize,
        methods: BTreeMap<u64, String>,
    }
    #[test]
    fn native_identity_table_composition() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-tables-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let out = compose_identity_tables(&c.film, &c.replication);
            assert_eq!(out, c.output, "composition {i}");
            assert_eq!(
                injective_player_indices(c.replication),
                (c.guard, c.collisions),
                "guard {i}"
            );
            for (x, m) in c.methods {
                assert_eq!(out.method(x), m, "method {i}/{x}");
            }
        }
    }
}
