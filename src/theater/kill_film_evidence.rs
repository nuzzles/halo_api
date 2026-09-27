//! LegacyFilm-derived inputs for the kill-source walk/scan/hybrid pipeline.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
fn read_table(chunks: &[FilmChunkData]) -> (KillFilmTable, Vec<PlayerTableSlot>) {
    let refusal = |s: &str| {
        (
            KillFilmTable {
                refusal: s.into(),
                ..Default::default()
            },
            Vec::new(),
        )
    };
    let Some(chunk) = chunks.first().filter(|c| !c.data.is_empty()) else {
        return refusal("sans_registre");
    };
    let Some(registry) = parse_registry(&chunk.data) else {
        return refusal("tronque");
    };
    let identity = match decode_film_identity(&chunk.data, &registry) {
        Ok(Some(i)) => i,
        Ok(None) => return refusal("sans_section"),
        Err(_) => return refusal("tronque"),
    };
    let table = decode_player_table(&chunk.data, &identity);
    if let Some(error) = table.error {
        let mut out = refusal(match error {
            PlayerTableError::UnknownBuild => "build_inconnu",
            PlayerTableError::NotFound => "table_introuvable",
            PlayerTableError::Truncated => "tronque",
        });
        out.0.build = identity.build;
        return out;
    }
    (
        KillFilmTable {
            seats: table
                .slots
                .iter()
                .filter(|s| !s.gamertag.is_empty())
                .map(|s| (s.film_index as i32, s.gamertag.clone()))
                .collect(),
            build: identity.build,
            occupied: table.report.occupied,
            vacant: table.report.vacant,
            interleaved_vacant: table.report.interleaved_vacant,
            refusal: String::new(),
        },
        table.slots,
    )
}
/// Unlike the replay index table, any disagreement refuses the entire kill-source
/// motif pin set. This preserves the native independence of read and inferred seats.
pub fn read_kill_index_motif(
    chunks: &[FilmChunkData],
    slots: &[PlayerTableSlot],
    feed: &KillFeed,
) -> KillIndexMotif {
    let mut names = BTreeMap::new();
    for s in slots {
        if s.xuid != 0 && !s.gamertag.is_empty() {
            names.insert(s.xuid, s.gamertag.clone());
        }
    }
    for (name, &xuid) in &feed.xuid_by_name {
        if xuid != 0 && !name.is_empty() {
            names.insert(xuid, name.clone());
        }
    }
    let mut out = KillIndexMotif::default();
    if names.is_empty() {
        return out;
    }
    let Ok(selected) = fire_events::native_chunk_prefix(chunks) else {
        return out;
    };
    if selected.len() < 2 {
        return out;
    }
    let xuids: Vec<_> = names.keys().copied().collect();
    let mut seen = BTreeMap::<u64, BTreeSet<i64>>::new();
    for c in &selected[..selected.len() - 1] {
        let reads = resolve_player_indices(&xuids, &c.data);
        if reads.is_empty() {
            continue;
        }
        out.readings += 1;
        for (x, index) in reads {
            seen.entry(x).or_default().insert(index);
        }
    }
    for (&xuid, name) in &names {
        let Some(indices) = seen.get(&xuid) else {
            out.absent += 1;
            continue;
        };
        if indices.len() > 1 {
            out.disagreements += 1;
            continue;
        }
        let index = *indices.first().unwrap() as i32;
        if name.is_empty() || !(0..32).contains(&index) {
            continue;
        }
        if let Some(existing) = out.names.get(&index) {
            if existing != name {
                out.names.remove(&index);
                out.disagreements += 1;
            }
        } else {
            out.names.insert(index, name.clone());
        }
    }
    if out.disagreements > 0 {
        out.names.clear();
    }
    out
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillFilmEvidence {
    pub feed: KillFeed,
    pub film_table: KillFilmTable,
    pub motif: KillIndexMotif,
    pub roster: KillRoster,
    pub kill_events: FilmKillEventScan,
    pub pairs: KillFeedPairs,
    pub candidates: Vec<KillSourceCandidate>,
    pub bijection_score: i64,
    pub bijection_margin: i64,
}
/// Build native film-derived kill evidence through roster inference. This is the
/// preparation stage, not the final kill-source result: calibrated record walks,
/// hybrid attribution, health checks and publication gates still follow it.
pub fn prepare_kill_film_evidence(
    chunks: &[FilmChunkData],
    major_version: i32,
    use_bots: bool,
    restarts: usize,
) -> Result<KillFilmEvidence, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    let feed = load_kill_feed(chunks, major_version)?;
    let (film_table, slots) = read_table(chunks);
    let motif = read_kill_index_motif(chunks, &slots, &feed);
    let metadata = scan_film_bot_metadata(chunks);
    let bots: Vec<_> = metadata.bots.iter().map(KillRosterBot::from).collect();
    let mut roster = build_kill_roster(&feed.names, &bots, use_bots, &film_table, &motif);
    let kill_events = scan_film_kill_events(chunks, major_version)?;
    let pairs = resolve_kill_feed_pairs(&feed, &kill_events.events, &roster.evidence);
    let candidates = scan_film_kill_sources(chunks, major_version, roster.player_count)?;
    let bijection_score = solve_kill_bijection(
        &mut roster,
        &pairs.pairs,
        &candidates,
        if restarts == 0 { 40 } else { restarts },
    )?;
    let bijection_margin =
        kill_bijection_margin(&roster, &pairs.pairs, &candidates, bijection_score);
    Ok(KillFilmEvidence {
        feed,
        film_table,
        motif,
        roster,
        kill_events,
        pairs,
        candidates,
        bijection_score,
        bijection_margin,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use std::{fs, io::Read, path::Path};
    #[test]
    fn native_kill_table_source() {
        fn normalized(v: serde_json::Value) -> serde_json::Value {
            match v {
                serde_json::Value::Object(m) => serde_json::Value::Object(
                    m.into_iter()
                        .map(|(k, v)| (k.replace('_', "").to_lowercase(), normalized(v)))
                        .collect(),
                ),
                serde_json::Value::Array(a) => {
                    serde_json::Value::Array(a.into_iter().map(normalized).collect())
                }
                v => v,
            }
        }
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(include_bytes!("fixtures/bootstrap-v41.zlib").as_slice())
            .read_to_end(&mut bootstrap)
            .unwrap();
        let registry = parse_registry(&bootstrap).unwrap();
        let identity = decode_film_identity(&bootstrap, &registry)
            .unwrap()
            .unwrap();
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/kill-table-source-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        let mut captured = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/kill-table-source-bootstrap-v41.zlib").as_slice(),
        )
        .read_to_end(&mut captured)
        .unwrap();
        assert_eq!(rows.len(), 56);
        for (case, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["order"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(j, number)| {
                    let mut data = bootstrap.clone();
                    if j == 0 {
                        match row["variant"].as_u64().unwrap() {
                            1 => data.clear(),
                            2 => data.truncate(8),
                            3 => data[identity.build_offset..identity.build_offset + 3].fill(0),
                            4 => {
                                data[identity.build_offset..identity.build_offset + 32].fill(0);
                                data[identity.build_offset..identity.build_offset + 10]
                                    .copy_from_slice(b"HI_unknown");
                            }
                            5 => data.truncate(identity.body_bit / 8),
                            6 => data.truncate(identity.body_bit / 8 + 1),
                            7 => data = captured.clone(),
                            _ => {}
                        }
                    }
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: number.as_i64().unwrap() as i32,
                            chunk_type: 1,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let (table, slots) = read_table(&chunks);
            assert_eq!(
                serde_json::to_value(&table).unwrap(),
                row["table"],
                "table {case}"
            );
            assert_eq!(table.read(), row["read"], "read {case}");
            assert_eq!(
                normalized(serde_json::to_value(&slots).unwrap()),
                normalized(row["slots"].clone()),
                "slots {case}"
            );
            let restored: (KillFilmTable, Vec<PlayerTableSlot>) =
                serde_json::from_value(serde_json::to_value((&table, &slots)).unwrap()).unwrap();
            assert_eq!(restored, (table, slots));
        }
    }

    #[test]
    fn native_kill_index_motif_conflicts() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/kill-index-motif-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 512);
        let mut modes = [0; 8];
        for (case, row) in rows.iter().enumerate() {
            let mode = row["mode"].as_u64().unwrap() as usize;
            modes[mode] += 1;
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let h = c["hex"].as_str().unwrap();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 2,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: 0,
                            file_relative_path: String::new(),
                        },
                        data: (0..h.len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                            .collect(),
                    }
                })
                .collect();
            let slots: Vec<_> = row["slots"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, s)| PlayerTableSlot {
                    film_index: i,
                    xuid: s["xuid"].as_u64().unwrap(),
                    gamertag: s["name"].as_str().unwrap().into(),
                    session_token: 0,
                    bit: 0,
                    total_bits: 0,
                    shorts: PlayerTableShorts {
                        tete: 0,
                        deux: 0,
                        repr: 0,
                        q64: 0,
                        f10: 0,
                        f14: 0,
                        f6: 0,
                        f8: 0,
                        f7: 0,
                        f1: 0,
                    },
                })
                .collect();
            let feed = KillFeed {
                xuid_by_name: serde_json::from_value(row["feed"].clone()).unwrap(),
                ..Default::default()
            };
            let got = read_kill_index_motif(&chunks, &slots, &feed);
            assert_eq!(
                serde_json::to_value(&got).unwrap(),
                row["output"],
                "case {case}"
            );
            match mode {
                0 | 7 => {
                    assert_eq!(got.names.len(), 3);
                    assert_eq!(got.disagreements, 0);
                    assert_eq!(got.readings, 2);
                }
                1..=3 => {
                    assert!(got.names.is_empty());
                    assert!(got.disagreements > 0);
                }
                4 => {
                    assert_eq!(got.absent, 1);
                    assert_eq!(got.names.len(), 2);
                }
                5 | 6 => assert_eq!(got, KillIndexMotif::default()),
                _ => unreachable!(),
            }
            if mode == 7 {
                assert_eq!(got.names.get(&3).map(String::as_str), Some("Replacement"));
            }
        }
        assert_eq!(modes, [64; 8]);
    }
    #[test]
    #[ignore = "requires the downloaded v41 corpus"]
    fn local_kill_film_evidence() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/kill-film-evidence-v41.json.zlib")[..],
        )
        .read_to_end(&mut json)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&json).unwrap();
        for row in rows {
            let folder = row["folder"].as_str().unwrap();
            let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
            let meta: serde_json::Value =
                serde_json::from_slice(&fs::read(dir.join("film.json")).unwrap()).unwrap();
            let chunks: Vec<_> = meta["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| FilmChunkData {
                    metadata: FilmChunk {
                        index: c["index"].as_i64().unwrap() as i32,
                        chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                        start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                        duration_ms: c["duration_ms"].as_i64().unwrap(),
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: fs::read(dir.join(c["file"].as_str().unwrap())).unwrap(),
                })
                .collect();
            let result = prepare_kill_film_evidence(&chunks, 41, true, 40);
            if row["error"] == true {
                assert!(result.is_err(), "{folder}");
                continue;
            }
            let e = result.unwrap();
            let actual = serde_json::json!({"folder":folder,"error":false,"table":e.film_table,"motif":e.motif,
                "roster_names":e.roster.evidence.names,"pins":e.roster.evidence.pins,"permutation":e.roster.permutation,"score":e.bijection_score,"margin":e.bijection_margin,
                "candidates":e.candidates,"events":e.kill_events.events,"gate":e.kill_events.gate15,"packets_with_events":e.kill_events.packets_with_events,
                "packets_with_kill":e.kill_events.packets_with_kill,"feed_events":e.pairs.events,"pairs":e.pairs.pairs,"stats":e.pairs.stats});
            if actual != row {
                fs::write(
                    format!(
                        "/private/tmp/halo-kill-evidence-{}-actual.json",
                        folder.replace('/', "-")
                    ),
                    serde_json::to_vec_pretty(&actual).unwrap(),
                )
                .unwrap();
            }
            for (key, value) in row.as_object().unwrap() {
                assert_eq!(&actual[key], value, "{folder}/{key}");
            }
            println!("kill evidence matches {folder}");
        }
    }
}
