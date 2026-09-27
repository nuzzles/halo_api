//! Publication filter for externally classified deaths without a claimant.
use super::{ReplayNeutralDeath, ReplayTrack, replay_published_track_xuid};
use std::collections::{BTreeMap, BTreeSet};

pub fn retain_replay_neutral_deaths(
    deaths: &mut Vec<ReplayNeutralDeath>,
    tracks: &[ReplayTrack],
    bridge: &BTreeMap<u32, u64>,
) {
    let before = deaths.len();
    let published: BTreeSet<_> = tracks
        .iter()
        .filter_map(|t| replay_published_track_xuid(t, bridge))
        .collect();
    deaths.retain(|d| !d.kind.is_empty() && published.contains(&d.xuid));
    let dropped = before - deaths.len();
    if dropped > 0 {
        tracing::info!(
            ecartees = dropped,
            publiees = deaths.len(),
            "rejeu 2D : morts sans revendication écartées (joueur sans trajectoire publiée, ou type non établi)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[test]
    fn native_neutral_death_publication() {
        #[derive(Deserialize)]
        struct Case {
            logs: Vec<serde_json::Value>,
            tracks: Vec<ReplayTrack>,
            bridge: BTreeMap<u32, u64>,
            deaths: Vec<ReplayNeutralDeath>,
            output: Option<Vec<ReplayNeutralDeath>>,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/neutral-deaths-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for (i, mut row) in rows.into_iter().enumerate() {
            let logs = crate::theater::log_test_support::capture_logs(|| {
                retain_replay_neutral_deaths(&mut row.deaths, &row.tracks, &row.bridge);
            });
            assert_eq!(logs, row.logs, "logs {i}");
            assert_eq!(row.deaths, row.output.unwrap_or_default(), "case {i}");
        }
    }
}
