//! Kill and assist attribution during measured equipment activation periods.
use super::ReplayEquipmentEpisode;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayEquipmentKill {
    #[serde(rename = "XUID")]
    pub xuid: u64,
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
    #[serde(rename = "AssistXUID")]
    pub assist_xuid: u64,
    pub assist_known: bool,
}
/// Return whether the native measurement gate is open. A known origin and read
/// source open that gate even when no credit is possible (empty input or invalid
/// interval). Existing counters are incremented, not reset. All active families
/// receive credit, with identity resolved at the kill frame on recycled seats.
pub fn attach_replay_equipment_kills(
    episodes: &mut [ReplayEquipmentEpisode],
    kills: &[ReplayEquipmentKill],
    read: bool,
    occupant: Option<&dyn Fn(u32, i64) -> u64>,
    origin_ms: Option<i64>,
    interval_ms: i64,
) -> bool {
    let Some(origin) = origin_ms.filter(|_| read) else {
        return false;
    };
    if episodes.is_empty() || kills.is_empty() || interval_ms <= 0 {
        return true;
    }
    let Some(occupant) = occupant else {
        return true;
    };
    for k in kills {
        let frame = k.time_ms.wrapping_sub(origin) / interval_ms;
        for (xuid, assist) in [
            (k.xuid, false),
            (if k.assist_known { k.assist_xuid } else { 0 }, true),
        ] {
            if xuid == 0 {
                continue;
            }
            for e in episodes.iter_mut() {
                if frame < e.t0 || frame > e.t1 || occupant(e.slot, frame) != xuid {
                    continue;
                }
                if assist {
                    e.a += 1;
                } else {
                    e.k += 1;
                }
            }
        }
    }
    true
}

/// Apply caller-resolved kill evidence using the player layer's calibrated origin
/// and temporal identity. Source decoding and its publication gate stay explicit.
pub fn attach_film_replay_equipment_kills(
    episodes: &mut super::ReplayEquipmentEpisodes,
    players: &super::FilmReplayPlayers,
    kills: &[ReplayEquipmentKill],
    read: bool,
) {
    let occupant = |slot, frame| {
        super::replay_occupant_at_frame(&players.registry.owners.state, players.clock, slot, frame)
    };
    episodes.coverage.kills_read = attach_replay_equipment_kills(
        &mut episodes.episodes,
        kills,
        read,
        Some(&occupant),
        players.origin_ms,
        players.frame_interval_ms,
    );
}
