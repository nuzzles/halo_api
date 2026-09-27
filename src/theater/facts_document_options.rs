//! Caller-owned cached assembly inputs. Recorded channels come only from facts.
use super::*;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct FactsReplayFlagOptions<'a> {
    pub scanned: bool,
    pub records: &'a [StatborgRecord],
    pub bursts: &'a [i64],
    pub identity: Option<&'a StatborgRoundIdentity>,
    pub spawns: &'a [ReplayFlagSpawn],
}
#[derive(Default)]
pub struct FactsReplayVipOptions<'a> {
    pub scanned: bool,
    pub records: &'a [StatborgRecord],
}
#[derive(Default)]
pub struct FactsReplayCarryOptions<'a> {
    pub scanned: bool,
    pub records: &'a [StatborgRecord],
    pub identity: Option<&'a StatborgRoundIdentity>,
}
#[derive(Default)]
pub struct FactsReplayZoneOptions<'a> {
    pub zones: &'a [ObjectiveZone],
    pub roles: &'a str,
    pub teams: BTreeMap<String, i64>,
    pub hill: bool,
}
/// Options which survive native FilmInputs.applyTo. Cached channels and scan
/// gates are read directly from the file rather than duplicated in caller state.
/// The cache's Statborg and Kills extra sections are deliberately not applied.
#[derive(Default)]
pub struct FactsReplayDocumentOptions<'a> {
    pub frame_interval_ms: i64,
    pub min_points: i64,
    pub geometry: Vec<ReplayMapObject>,
    pub structure: Vec<ReplaySurface>,
    pub map: Option<&'a FilmMapBounds>,
    pub labels: Option<&'a ReplayLabelCatalog>,
    pub spawn_points: &'a [MapSpawnPoint],
    pub spawn_points_state: &'a str,
    pub roster_xuids: &'a [u64],
    pub participants: &'a [IdentityParticipant],
    pub bots: &'a [IdentityBot],
    pub successions: &'a [IdentitySuccession],
    pub scoreboard_teams: BTreeMap<String, i64>,
    pub statborg_identity: Option<&'a StatborgRoundIdentity>,
    pub objectives: &'a [StatborgIdentifiedEvent],
    pub objectives_unnamed: i64,
    pub objectives_refused: i64,
    pub score: Option<ReplayScoreInput<'a>>,
    pub flag: FactsReplayFlagOptions<'a>,
    pub vip: FactsReplayVipOptions<'a>,
    pub skull: FactsReplayCarryOptions<'a>,
    pub bomb_carry_scanned: bool,
    pub bomb_arming_scanned: bool,
    pub zone: FactsReplayZoneOptions<'a>,
    pub neutral_deaths: &'a [ReplayNeutralDeath],
    pub equipment_kills: &'a [ReplayEquipmentKill],
    pub equipment_kills_read: bool,
    pub deaths_paths: Option<ReplayDeathsPathsCoverage>,
    pub match_kills: &'a [ReplayKillReference],
    pub match_kills_read: bool,
    pub match_kills_dropped: i64,
    pub fallbacks: Option<&'a FallbackCounter>,
}
