//! Native scoreboard resolution of participant indices refused by direct creation naming.
use super::{IdentityCreationReport, IdentityDeathClock, PlayerIndexTable, ReplayIdentityState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentityBot {
    pub film_index: i64,
    pub name: super::ReplayByteString,
    #[serde(rename = "BotID")]
    pub bot_id: i64,
}
impl IdentityBot {
    pub fn bid(&self) -> String {
        if self.bot_id > 0 {
            format!("bid({}.0)", self.bot_id)
        } else {
            String::new()
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentityParticipant {
    #[serde(rename = "ID")]
    pub id: String,
    pub joined_in_progress: bool,
    #[serde(rename = "JoinMatchMS")]
    pub join_match_ms: Option<i64>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityScoreboardReport {
    #[serde(rename = "Lignes")]
    pub rows: usize,
    #[serde(rename = "Bots")]
    pub bots: usize,
    #[serde(rename = "Humains")]
    pub humans: usize,
    #[serde(rename = "Conflits")]
    pub conflicts: usize,
    #[serde(rename = "SansCandidat")]
    pub no_candidate: usize,
}
impl IdentityScoreboardReport {
    pub fn named(&self) -> usize {
        self.bots + self.humans
    }
}
pub struct IdentityScoreboardInput<'a> {
    pub participants: &'a [IdentityParticipant],
    pub bots: &'a [IdentityBot],
    /// Must already be injective, as produced by compose_identity_tables.
    pub indices: &'a PlayerIndexTable,
    pub clock: &'a IdentityDeathClock,
}
pub fn identity_bot_ids_by_index(bots: &[IdentityBot]) -> BTreeMap<i64, String> {
    let mut seen = BTreeMap::<i64, BTreeSet<String>>::new();
    for bot in bots {
        let bid = bot.bid();
        if !bid.is_empty() {
            seen.entry(bot.film_index).or_default().insert(bid);
        }
    }
    seen.into_iter()
        .filter_map(|(index, bids)| {
            if bids.len() == 1 {
                Some((index, bids.into_iter().next().unwrap()))
            } else {
                None
            }
        })
        .collect()
}
/// Requires a declared bot with a scoreboard row. Shared human/bot indices need
/// a known join time and a measured clock; lives crossing that instant abstain.
pub fn resolve_identity_scoreboard(
    state: &mut ReplayIdentityState,
    creation: &IdentityCreationReport,
    input: IdentityScoreboardInput<'_>,
) -> IdentityScoreboardReport {
    let mut out = IdentityScoreboardReport {
        rows: input.participants.len(),
        ..Default::default()
    };
    if input.participants.is_empty() || creation.read_indices.is_empty() {
        return out;
    }
    let mut present = BTreeSet::new();
    let mut arrivals = BTreeMap::new();
    for p in input.participants {
        if p.id.is_empty() {
            continue;
        }
        present.insert(p.id.as_str());
        if p.joined_in_progress
            && let Some(ms) = p.join_match_ms
        {
            arrivals.insert(p.id.as_str(), ms);
        }
    }
    let bots = identity_bot_ids_by_index(input.bots);
    let humans: BTreeMap<_, _> = input
        .indices
        .by_xuid
        .iter()
        .map(|(&x, &i)| (i, x))
        .collect();
    for i in 0..state.lives().len() {
        let l = &state.lives()[i];
        if l.xuid != 0
            || !l.bid.is_empty()
            || creation.causes.get(&i).map(String::as_str) != Some("index_hors_table")
        {
            continue;
        }
        let Some(&index) = creation.read_indices.get(&i) else {
            continue;
        };
        let index = i64::from(index);
        let Some(bid) = bots
            .get(&index)
            .filter(|bid| present.contains(bid.as_str()))
        else {
            out.no_candidate += 1;
            continue;
        };
        let Some(&xuid) = humans.get(&index) else {
            if state.assign_bot(i as i64, bid) {
                out.bots += 1;
            }
            continue;
        };
        let arrival = arrivals
            .get(xuid.to_string().as_str())
            .filter(|_| input.clock.matched > 0)
            .map(|ms| ms.wrapping_add(input.clock.offset_ms).wrapping_mul(1000));
        let Some(arrival) = arrival else {
            out.conflicts += 1;
            continue;
        };
        if l.to < arrival {
            if state.assign_bot(i as i64, bid) {
                out.bots += 1;
            }
        } else if l.from >= arrival {
            if state.assign_life(i as i64, xuid, Some(index), "tableau_api") {
                out.humans += 1;
            }
        } else {
            out.conflicts += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::IdentityLife;
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        lives: Vec<IdentityLife>,
        indices: PlayerIndexTable,
        bots: Vec<IdentityBot>,
        participants: Vec<IdentityParticipant>,
        clock: IdentityDeathClock,
        creation: IdentityCreationReport,
        report: IdentityScoreboardReport,
        state: ReplayIdentityState,
        bot_ids: BTreeMap<i64, String>,
    }
    #[test]
    fn native_scoreboard_identity_resolution() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-scoreboard-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in rows.into_iter().enumerate() {
            assert_eq!(identity_bot_ids_by_index(&c.bots), c.bot_ids, "bots {i}");
            let mut state = ReplayIdentityState::from_lives(c.lives, &c.indices.by_xuid);
            let report = resolve_identity_scoreboard(
                &mut state,
                &c.creation,
                IdentityScoreboardInput {
                    participants: &c.participants,
                    bots: &c.bots,
                    indices: &c.indices,
                    clock: &c.clock,
                },
            );
            assert_eq!(report, c.report, "report {i}");
            assert_eq!(state, c.state, "state {i}");
        }
    }
}
