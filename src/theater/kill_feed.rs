//! Kill-source highlight selection and chronological feed construction.
use super::{DecodeError, KillPacketIdentity, NativeHighlightEvent, parse_highlight_events};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Native `XUIDNamePrefix`: a fallback identity, not a recorded gamertag.
pub const XUID_NAME_PREFIX: &str = "xuid:";

/// Native `ErrNoKillFeed`, represented by the existing typed decoder error.
pub const NO_KILL_FEED: DecodeError =
    DecodeError::KillSource(super::KillSourceFilmError::NoKillFeed);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillFeedEvent {
    pub time_ms: i64,
    pub killer: String,
    pub victim: String,
    pub victim_xuid: u64,
    pub packet: Option<KillPacketIdentity>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillFeed {
    pub events: Vec<KillFeedEvent>,
    pub names: Vec<String>,
    pub xuid_by_name: BTreeMap<String, u64>,
    pub kill_count: usize,
    pub death_count: usize,
}
/// Join highlight credit/death fields at equal timestamps. The native reader
/// retains the last field at a timestamp; raw counts still include duplicates.
/// This stage does not reconstruct missing couples from nearby events.
pub fn build_kill_feed(highlights: &[NativeHighlightEvent]) -> KillFeed {
    let mut gamertags = BTreeMap::new();
    for e in highlights {
        if !e.gamertag.is_empty() {
            gamertags.insert(e.xuid, e.gamertag.clone());
        }
    }
    let mut out = KillFeed::default();
    let mut by_time = BTreeMap::<i64, KillFeedEvent>::new();
    for e in highlights {
        if !matches!(e.event_type.as_str(), "kill" | "death") {
            continue;
        }
        let name = gamertags
            .get(&e.xuid)
            .cloned()
            .unwrap_or_else(|| format!("{XUID_NAME_PREFIX}{}", e.xuid));
        let time_ms = i64::from(e.time_ms);
        let event = by_time.entry(time_ms).or_insert_with(|| KillFeedEvent {
            time_ms,
            ..Default::default()
        });
        if e.event_type == "kill" {
            out.kill_count += 1;
            event.killer = name;
        } else {
            out.death_count += 1;
            event.victim = name.clone();
            event.victim_xuid = e.xuid;
            out.xuid_by_name.insert(name, e.xuid);
        }
    }
    out.events = by_time.into_values().collect();
    let mut names = BTreeSet::new();
    for e in &out.events {
        if !e.killer.is_empty() {
            names.insert(e.killer.clone());
        }
        if !e.victim.is_empty() {
            names.insert(e.victim.clone());
        }
    }
    out.names = names.into_iter().collect();
    out
}
/// Choose the supplied chunk with the most kill highlights, retaining the
/// first chunk on ties. Native kill-source loading skips highlight parse errors.
/// This differs from the replay helper that selects the final readable chunk.
pub fn load_kill_feed(
    chunks: &[FilmChunkData],
    major_version: i32,
) -> Result<KillFeed, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    let mut best = Vec::new();
    let mut count = 0;
    for chunk in chunks {
        let Ok(events) = parse_highlight_events(&chunk.data, major_version) else {
            continue;
        };
        let kills = events.iter().filter(|e| e.event_type == "kill").count();
        if kills > count {
            count = kills;
            best = events;
        }
    }
    if count == 0 {
        return Err(NO_KILL_FEED);
    }
    Ok(build_kill_feed(&best))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Row {
        highlights: Vec<NativeHighlightEvent>,
        expected: KillFeed,
    }
    #[test]
    fn native_kill_feed_oracle() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kill-feed-v41.json.zlib")[..])
            .read_to_end(&mut json)
            .unwrap();
        let rows: Vec<Row> = serde_json::from_slice(&json).unwrap();
        assert_eq!(rows.len(), 512);
        for (i, row) in rows.into_iter().enumerate() {
            assert_eq!(build_kill_feed(&row.highlights), row.expected, "feed {i}");
        }
    }
}
