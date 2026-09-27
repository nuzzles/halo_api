//! Native footer objective interactions and capture-burst event extraction.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectiveFooterEvent {
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
    pub slot: u8,
    pub team: u8,
    #[serde(rename = "XUID")]
    pub xuid: u64,
}
/// Native v41 mode-footer fields from one complete 60-byte block. Slot and
/// team are raw byte values; no roster, team mapping or objective inference.
pub fn decode_objective_footer_block(block: &[u8; 60], xuid: u64) -> Option<ObjectiveFooterEvent> {
    (block[47] == 10).then(|| ObjectiveFooterEvent {
        time_ms: u32::from_be_bytes(block[48..52].try_into().unwrap()).into(),
        slot: block[36],
        team: block[37],
        xuid,
    })
}
/// Scan unaligned XUID markers and the first closing marker within 20,000 bits.
/// An earlier invalid block stops that XUID's search; later blocks are not guessed.
pub fn scan_objective_footer(data: &[u8]) -> Vec<ObjectiveFooterEvent> {
    let bits = bits::Bits(data);
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    if bits.len() < 80 {
        return out;
    }
    for marker in 72..=bits.len() - 8 {
        if bits.read(marker, 8) != Some(0xc0)
            || !matches!(bits.read(marker - 8, 8), Some(0x2d | 0x25))
        {
            continue;
        }
        let start = marker - 72;
        if seen.contains(&start) {
            continue;
        }
        let xuid = bits.read(start, 64).unwrap().swap_bytes();
        if xuid <= 2_000_000_000_000_000 || xuid >= 3_000_000_000_000_000 {
            continue;
        }
        seen.insert(start);
        let end = (start + 20000).min(bits.len());
        if end < 32 {
            continue;
        }
        for at in start..=end - 32 {
            if bits.read(at, 32) != Some(0x00002ee0) {
                continue;
            }
            if let Some(start) = at.checked_sub(480).filter(|&p| p >= start) {
                let block = std::array::from_fn(|i| bits.read(start + i * 8, 8).unwrap() as u8);
                if let Some(event) = decode_objective_footer_block(&block, xuid) {
                    out.push(event);
                }
            }
            break;
        }
    }
    native_sort::sort_by(&mut out, |a, b| a.time_ms.cmp(&b.time_ms));
    out
}
/// Select the first highest-index type-3 chunk described by the manifest.
pub fn scan_film_objective_footer(chunks: &[FilmChunkData]) -> Vec<ObjectiveFooterEvent> {
    let mut selected = None;
    let mut index = -1;
    for c in chunks {
        if c.metadata.chunk_type == 3 && c.metadata.index > index {
            index = c.metadata.index;
            selected = Some(c);
        }
    }
    selected.map_or_else(Vec::new, |c| scan_objective_footer(&c.data))
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectiveEventPlayer {
    #[serde(rename = "XUID")]
    pub xuid: String,
    pub role: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExtractedObjectiveEvent {
    #[serde(rename = "MatchID")]
    pub match_id: String,
    pub seq: usize,
    #[serde(rename = "TimeMS")]
    pub time_ms: Option<i64>,
    pub objective_type: String,
    pub event_type: String,
    #[serde(rename = "TeamID")]
    pub team_id: Option<i64>,
    #[serde(rename = "ObjectiveID")]
    pub objective_id: Option<i64>,
    pub value: Option<i64>,
    pub source: String,
    pub confidence: String,
    pub details: String,
    pub players: Vec<ObjectiveEventPlayer>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectiveTeamControl {
    pub film: usize,
    pub accord: usize,
    pub contradiction: usize,
    pub silence: usize,
}
impl ObjectiveTeamControl {
    fn note(&mut self, roster: &mut impl FnMut(&str) -> Option<i64>, xuid: &str, team: i64) {
        self.film += 1;
        match roster(xuid) {
            None => self.silence += 1,
            Some(t) if t == team => self.accord += 1,
            Some(_) => self.contradiction += 1,
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedObjectives {
    pub events: Vec<ExtractedObjectiveEvent>,
    pub teams: ObjectiveTeamControl,
}
/// Preserve native event order on timestamp ties and assign a dense sequence.
pub fn finalize_objective_events(match_id: &str, events: &mut [ExtractedObjectiveEvent]) {
    events.sort_by_key(|e| e.time_ms.unwrap_or(-1));
    for (seq, e) in events.iter_mut().enumerate() {
        e.seq = seq;
        e.match_id = match_id.into();
    }
}
/// Feed teams are authoritative; the supplied roster only measures disagreement.
/// Bomb and VIP have no footer event extractor in the pinned native implementation.
pub fn extract_objective_events(
    match_id: &str,
    variant: &str,
    bursts: &[i64],
    footer: &[ObjectiveFooterEvent],
    roster: &BTreeMap<String, i64>,
) -> ExtractedObjectives {
    extract_objective_events_with_roster(match_id, variant, bursts, footer, |xuid| {
        roster.get(xuid).copied()
    })
}

/// Extract with a caller-owned roster lookup, equivalent to native `Roster::TeamOf`.
/// Called once per attributed event, in extraction order before final sorting.
/// `None` records silence; returned teams only affect diagnostic counters.
/// Unattributed events and unsupported modes never invoke the callback.
pub fn extract_objective_events_with_roster(
    match_id: &str,
    variant: &str,
    bursts: &[i64],
    footer: &[ObjectiveFooterEvent],
    mut roster: impl FnMut(&str) -> Option<i64>,
) -> ExtractedObjectives {
    let kind = replay_objective_type(variant);
    let mut out = ExtractedObjectives::default();
    let mut make = |time_ms: i64, scorer: Option<&ObjectiveFooterEvent>, capture: bool| {
        let mut event = ExtractedObjectiveEvent {
            time_ms: Some(time_ms),
            objective_type: kind.into(),
            event_type: if capture {
                "capture"
            } else {
                match kind {
                    "zone" => "zone_capture",
                    "hill" => "hill_capture",
                    _ => "skull_carry",
                }
            }
            .into(),
            value: capture.then_some(1),
            source: if capture { "burst" } else { "th10" }.into(),
            confidence: if capture { "exact" } else { "approx" }.into(),
            details: "{}".into(),
            ..Default::default()
        };
        if let Some(scorer) = scorer {
            let xuid = scorer.xuid.to_string();
            event.team_id = Some(i64::from(scorer.team));
            out.teams.note(&mut roster, &xuid, i64::from(scorer.team));
            event.players.push(ObjectiveEventPlayer {
                xuid,
                role: "scorer".into(),
            });
        }
        event
    };
    match kind {
        "flag" => {
            for &time in bursts {
                let mut scorer: Option<&ObjectiveFooterEvent> = None;
                for event in footer {
                    if event.time_ms.abs_diff(time) <= 2000
                        && scorer.is_none_or(|s| event.time_ms > s.time_ms)
                    {
                        scorer = Some(event);
                    }
                }
                out.events.push(make(time, scorer, true));
            }
        }
        "zone" | "hill" | "skull" => {
            for event in footer {
                out.events.push(make(event.time_ms, Some(event), false));
            }
        }
        _ => {}
    }
    finalize_objective_events(match_id, &mut out.events);
    out
}
pub fn extract_film_objective_events(
    match_id: &str,
    variant: &str,
    chunks: &[FilmChunkData],
    major_version: i32,
    roster: &BTreeMap<String, i64>,
) -> Result<ExtractedObjectives, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    let kind = replay_objective_type(variant);
    if !matches!(kind, "flag" | "zone" | "hill" | "skull") {
        return Ok(ExtractedObjectives::default());
    }
    let bursts = if kind == "flag" {
        scan_capture_bursts(chunks)?
    } else {
        Vec::new()
    };
    Ok(extract_objective_events(
        match_id,
        variant,
        &bursts,
        &scan_film_objective_footer(chunks),
        roster,
    ))
}

#[allow(dead_code)]
impl LegacyFilm {
    /// Extract mode-specific objective events from the evidence retained at decode time.
    pub fn objective_events(
        &self,
        variant: &str,
        roster: &BTreeMap<String, i64>,
    ) -> ExtractedObjectives {
        extract_objective_events(
            self.match_id.as_deref().unwrap_or_default(),
            variant,
            &self.capture_bursts_ms,
            &self.objective_footer,
            roster,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use std::io::Read;

    #[test]
    fn native_objective_footer_and_extraction() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/objective-extract-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        let hex = |s: &str| {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>()
        };
        for (i, r) in rows.iter().enumerate() {
            let footer = scan_objective_footer(&hex(r["data"].as_str().unwrap()));
            assert_eq!(
                serde_json::to_value(&footer).unwrap(),
                r["footer"],
                "footer {i}"
            );
            let chunks: Vec<_> = r["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| FilmChunkData {
                    metadata: FilmChunk {
                        index: c["index"].as_i64().unwrap() as i32,
                        chunk_type: c["kind"].as_i64().unwrap() as i32,
                        start_time_offset_ms: c["start"].as_i64().unwrap(),
                        duration_ms: 0,
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: hex(c["data"].as_str().unwrap()),
                })
                .collect();
            assert_eq!(
                serde_json::to_value(scan_film_objective_footer(&chunks)).unwrap(),
                r["selected"],
                "selection {i}"
            );
            assert_eq!(
                serde_json::to_value(scan_capture_bursts(&chunks).unwrap()).unwrap(),
                r["capture_times"],
                "bursts {i}"
            );
            let roster = serde_json::from_value(r["roster"].clone()).unwrap();
            let source_meta: Vec<_> = chunks
                .iter()
                .map(|c| FilmSourceMetadata {
                    index: i64::from(c.metadata.index),
                    chunk_type: i64::from(c.metadata.chunk_type),
                    start_ms: c.metadata.start_time_offset_ms,
                })
                .collect();
            let source_bytes: Vec<_> = chunks.iter().map(|c| c.data.as_slice()).collect();
            let source = FilmSource::load(&source_bytes, &source_meta).unwrap();
            assert_eq!(
                serde_json::to_value(scan_source_objective_footer(Some(&source))).unwrap(),
                r["selected"],
                "source footer {i}"
            );
            assert_eq!(
                serde_json::to_value(scan_source_capture_bursts(Some(&source))).unwrap(),
                r["capture_times"],
                "source bursts {i}"
            );
            let source_events = extract_source_objective_events(
                "fixture",
                r["variant"].as_str().unwrap(),
                Some(&source),
                41,
                &roster,
            )
            .unwrap();
            assert_eq!(
                serde_json::to_value(source_events.events).unwrap(),
                r["events"],
                "source events {i}"
            );
            assert_eq!(
                serde_json::to_value(source_events.teams).unwrap(),
                r["teams"],
                "source teams {i}"
            );
            let actual = extract_film_objective_events(
                "fixture",
                r["variant"].as_str().unwrap(),
                &chunks,
                41,
                &roster,
            )
            .unwrap();
            assert_eq!(
                serde_json::to_value(actual.events).unwrap(),
                r["events"],
                "events {i}"
            );
            assert_eq!(
                serde_json::to_value(actual.teams).unwrap(),
                r["teams"],
                "teams {i}"
            );
            for (burst, scorer) in r["bursts"]
                .as_array()
                .unwrap()
                .iter()
                .zip(r["scorers"].as_array().unwrap())
            {
                let actual = extract_objective_events(
                    "fixture",
                    "CTF",
                    &[burst.as_i64().unwrap()],
                    &footer,
                    &roster,
                );
                let event = &actual.events[0];
                if scorer.is_null() {
                    assert!(event.players.is_empty(), "scorer {i}");
                    assert!(event.team_id.is_none());
                } else {
                    assert_eq!(
                        event.players[0].xuid,
                        scorer["XUID"].as_u64().unwrap().to_string(),
                        "scorer {i}"
                    );
                    assert_eq!(event.team_id, scorer["Team"].as_i64());
                }
            }
        }
    }
    #[test]
    fn objective_finalization_preserves_ties_and_absent_time() {
        let mut events = [Some(4), None, Some(-2), Some(4)]
            .into_iter()
            .enumerate()
            .map(|(i, time_ms)| ExtractedObjectiveEvent {
                time_ms,
                details: i.to_string(),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        finalize_objective_events("match", &mut events);
        assert_eq!(
            events
                .iter()
                .map(|e| e.details.as_str())
                .collect::<Vec<_>>(),
            ["2", "1", "0", "3"]
        );
        for (i, e) in events.iter().enumerate() {
            assert_eq!(e.seq, i);
            assert_eq!(e.match_id, "match");
        }
        assert!(matches!(
            extract_film_objective_events("match", "CTF", &[], 75, &BTreeMap::new()),
            Err(DecodeError::UnsupportedVersion(75))
        ));
    }

    #[test]
    #[ignore = "requires the local decompressed film corpus"]
    fn local_objective_corpus() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/objective-corpus-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 32);
        for row in rows {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(row["folder"].as_str().unwrap());
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join("film.json")).unwrap()).unwrap();
            let chunks: Vec<_> = manifest["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| FilmChunkData {
                    metadata: FilmChunk {
                        index: c["index"].as_i64().unwrap() as i32,
                        chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                        start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                        duration_ms: 0,
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: std::fs::read(root.join(c["file"].as_str().unwrap())).unwrap(),
                })
                .collect();
            let source_meta: Vec<_> = chunks
                .iter()
                .map(|c| FilmSourceMetadata {
                    index: i64::from(c.metadata.index),
                    chunk_type: i64::from(c.metadata.chunk_type),
                    start_ms: c.metadata.start_time_offset_ms,
                })
                .collect();
            let source_bytes: Vec<_> = chunks.iter().map(|c| c.data.as_slice()).collect();
            let source = FilmSource::load(&source_bytes, &source_meta).unwrap();
            assert_eq!(
                serde_json::to_value(scan_source_objective_footer(Some(&source))).unwrap(),
                row["footer"],
                "source footer {}",
                root.display()
            );
            assert_eq!(
                serde_json::to_value(scan_source_capture_bursts(Some(&source))).unwrap(),
                row["bursts"],
                "source bursts {}",
                root.display()
            );
            let source_events = extract_source_objective_events(
                "fixture",
                "CTF",
                Some(&source),
                41,
                &BTreeMap::new(),
            )
            .unwrap();
            assert_eq!(
                serde_json::to_value(source_events.events).unwrap(),
                row["events"],
                "source events {}",
                root.display()
            );
            assert_eq!(
                serde_json::to_value(source_events.teams).unwrap(),
                row["teams"],
                "source teams {}",
                root.display()
            );
            let footer = scan_film_objective_footer(&chunks);
            let bursts = scan_capture_bursts(&chunks).unwrap();
            assert_eq!(
                serde_json::to_value(&footer).unwrap(),
                row["footer"],
                "footer {}",
                root.display()
            );
            assert_eq!(
                serde_json::to_value(&bursts).unwrap(),
                row["bursts"],
                "bursts {}",
                root.display()
            );
            let actual =
                extract_objective_events("fixture", "CTF", &bursts, &footer, &BTreeMap::new());
            assert_eq!(
                serde_json::to_value(actual.events).unwrap(),
                row["events"],
                "events {}",
                root.display()
            );
            assert_eq!(
                serde_json::to_value(actual.teams).unwrap(),
                row["teams"],
                "teams {}",
                root.display()
            );
        }
    }
}
