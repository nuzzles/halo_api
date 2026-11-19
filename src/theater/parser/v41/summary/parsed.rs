//! Convert guarded summary candidates into finished film highlights.
use super::candidates::{candidate_payload, read_summary_candidates};
use super::models::ParsedSummary;
use crate::theater::film::summary::{Summary, SummaryHighlight};

impl From<ParsedSummary> for Summary {
    fn from(parsed: ParsedSummary) -> Self {
        let ParsedSummary { data, body, .. } = parsed;
        let mut highlights = Vec::new();
        for packet in body.packets {
            if packet.body.decoded().is_none() {
                continue;
            }
            let Some(payload) = packet.payload(&data) else {
                continue;
            };
            for read in read_summary_candidates(payload) {
                highlights.push(SummaryHighlight {
                    gamertag: read.gamertag(),
                    xuid: crate::theater::film::Xuid(read.xuid),
                    timestamp_us: u64::from(read.timestamp_ms) * 1_000,
                    medal_code: (read.medal_flag == 1).then_some(read.metadata),
                    payload: candidate_payload(&read),
                });
            }
        }
        Self { highlights }
    }
}
