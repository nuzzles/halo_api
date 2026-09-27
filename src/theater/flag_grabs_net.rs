//! Native net flag-grab counts: collapse repeated carries of the same flag/player.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FlagGrabSpan {
    pub state: String,
    #[serde(rename = "StartMS")]
    pub start_ms: i64,
    #[serde(rename = "EndMS")]
    pub end_ms: i64,
    #[serde(rename = "XUID")]
    pub xuid: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FlagGrabTrack {
    pub team: i64,
    pub spans: Vec<FlagGrabSpan>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FlagGrabCounts {
    #[serde(rename = "XUID")]
    pub xuid: String,
    pub raw: usize,
    pub net: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetFlagGrabs {
    pub measured: bool,
    #[serde(rename = "WindowMS")]
    pub window_ms: i64,
    pub players: Vec<FlagGrabCounts>,
}
/// Read the published flag layer only when its coverage establishes a flag film.
/// `None` is an unavailable channel; an empty track list can still be measured.
pub fn replay_flag_grab_tracks(doc: &super::ReplayDocument) -> Option<(Vec<FlagGrabTrack>, usize)> {
    let coverage = doc.coverage.as_ref()?.flag_carries.as_ref()?;
    let interval = doc.content.frame_interval_ms;
    if !coverage.flag_film || interval <= 0 {
        return None;
    }
    Some((
        doc.content
            .flag_carries
            .iter()
            .map(|track| FlagGrabTrack {
                team: track.team,
                spans: track
                    .spans
                    .iter()
                    .map(|span| FlagGrabSpan {
                        state: span.state.clone(),
                        start_ms: span.t0.wrapping_mul(interval),
                        end_ms: span.t1.wrapping_mul(interval),
                        xuid: span.xuid.clone().unwrap_or_default(),
                    })
                    .collect(),
            })
            .collect(),
        coverage.openings,
    ))
}

/// The signed nanosecond window follows native time.Duration. A positive window
/// shorter than one millisecond is measured with a zero-millisecond threshold.
pub fn count_net_flag_grabs(tracks: &[FlagGrabTrack], window_ns: i64) -> NetFlagGrabs {
    if window_ns <= 0 {
        return NetFlagGrabs::default();
    }
    let window_ms = window_ns / 1_000_000;
    let mut counts = BTreeMap::<String, FlagGrabCounts>::new();
    for track in tracks {
        let mut spans: Vec<_> = track.spans.iter().collect();
        spans.sort_by_key(|s| s.start_ms);
        let mut previous: Option<&FlagGrabSpan> = None;
        for span in &spans {
            if !matches!(span.state.as_str(), "carried" | "carried_open") {
                continue;
            }
            if !span.xuid.is_empty() {
                let c = counts
                    .entry(span.xuid.clone())
                    .or_insert_with(|| FlagGrabCounts {
                        xuid: span.xuid.clone(),
                        ..Default::default()
                    });
                c.raw += 1;
                let net = previous.is_none_or(|p| {
                    p.xuid != span.xuid
                        || p.state == "carried_open"
                        || span.start_ms.wrapping_sub(p.end_ms) > window_ms
                        || spans.iter().any(|s| {
                            s.state == "home" && s.end_ms >= p.end_ms && s.start_ms <= span.start_ms
                        })
                });
                c.net += usize::from(net);
            }
            previous = Some(span);
        }
    }
    NetFlagGrabs {
        measured: true,
        window_ms,
        players: counts.into_values().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Row {
        tracks: Vec<FlagGrabTrack>,
        window_ns: i64,
        expected: NetFlagGrabs,
    }
    #[test]
    fn native_net_flag_grabs_oracle() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/flag-grabs-net-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<Row> = serde_json::from_slice(&bytes).unwrap();
        for (i, row) in rows.into_iter().enumerate() {
            assert_eq!(
                count_net_flag_grabs(&row.tracks, row.window_ns),
                row.expected,
                "row {i}"
            );
        }
    }
    #[test]
    fn native_flag_grab_document_bridge() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/flag-grab-bridge-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let doc: crate::theater::ReplayDocument =
                serde_json::from_value(row["doc"].clone()).unwrap();
            let result = replay_flag_grab_tracks(&doc);
            assert_eq!(
                result.is_some(),
                row["ok"].as_bool().unwrap(),
                "availability {i}"
            );
            let (tracks, openings) = result.unwrap_or_default();
            assert_eq!(
                serde_json::to_value(tracks).unwrap(),
                row["tracks"],
                "tracks {i}"
            );
            assert_eq!(
                openings as u64,
                row["openings"].as_u64().unwrap(),
                "openings {i}"
            );
        }
    }
}
