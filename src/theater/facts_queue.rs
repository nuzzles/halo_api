//! Deaths and player-table cache projection, preserving raw names and nil tables.
use super::facts_events::facts_slice_count;
use super::{NativeFactsReader, NativeFactsWriter};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsDeath {
    #[serde(rename = "XUID")]
    pub xuid: u64,
    #[serde(rename = "Gamertag")]
    pub gamertag: Vec<u8>,
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsPlayerSeat {
    #[serde(rename = "FilmIndex")]
    pub film_index: i64,
    #[serde(rename = "XUID")]
    pub xuid: u64,
    #[serde(rename = "Gamertag")]
    pub gamertag: Vec<u8>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsPlayerTable {
    #[serde(rename = "Build")]
    pub build: Vec<u8>,
    #[serde(rename = "Refusal")]
    pub refusal: Vec<u8>,
    #[serde(rename = "Occupied")]
    pub occupied: i64,
    #[serde(rename = "Vacant")]
    pub vacant: i64,
    #[serde(rename = "InterleavedVacant")]
    pub interleaved_vacant: bool,
    #[serde(rename = "Seats")]
    pub seats: Option<Vec<FactsPlayerSeat>>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsPlayerIndices {
    #[serde(rename = "Readings")]
    pub readings: i64,
    #[serde(rename = "Disagreements")]
    pub disagreements: i64,
    #[serde(rename = "ByXUID")]
    pub by_xuid: BTreeMap<u64, i64>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsTeamScan {
    #[serde(rename = "ArchetypeAbsent")]
    pub archetype_absent: bool,
    #[serde(rename = "ComponentMismatch")]
    pub component_mismatch: bool,
    #[serde(rename = "Component")]
    pub component: Vec<u8>,
    #[serde(rename = "Packets")]
    pub packets: i64,
    #[serde(rename = "Records")]
    pub records: i64,
    #[serde(rename = "Read")]
    pub read: i64,
    #[serde(rename = "Unreached")]
    pub unreached: i64,
    #[serde(rename = "OutOfDomainIndex")]
    pub out_of_domain_index: i64,
    #[serde(rename = "OutOfDomainValue")]
    pub out_of_domain_value: i64,
    #[serde(rename = "Entities")]
    pub entities: i64,
    #[serde(rename = "EntityDivergences")]
    pub entity_divergences: i64,
    #[serde(rename = "IndexDivergences")]
    pub index_divergences: i64,
    #[serde(rename = "Indices")]
    pub indices: i64,
    #[serde(rename = "NoTeam")]
    pub no_team: i64,
}
pub fn encode_facts_player_table(w: &mut NativeFactsWriter, t: &FactsPlayerTable) {
    w.string_bytes(&t.build);
    w.string_bytes(&t.refusal);
    w.unsigned(t.occupied as u64);
    w.unsigned(t.vacant as u64);
    w.boolean(t.interleaved_vacant);
    let seats = t.seats.as_deref().unwrap_or(&[]);
    w.unsigned(seats.len() as u64);
    for s in seats {
        w.signed(s.film_index);
        w.unsigned(s.xuid);
        w.string_bytes(&s.gamertag);
    }
}
pub fn decode_facts_player_table(r: &mut NativeFactsReader<'_>) -> FactsPlayerTable {
    let mut t = FactsPlayerTable {
        build: r.string_bytes().to_vec(),
        refusal: r.string_bytes().to_vec(),
        occupied: r.unsigned() as i64,
        vacant: r.unsigned() as i64,
        interleaved_vacant: r.boolean(),
        seats: None,
    };
    let n = facts_slice_count(r);
    if n != 0 {
        t.seats = Some(Vec::new());
        for _ in 0..n {
            if r.error().is_some() {
                break;
            }
            t.seats.as_mut().unwrap().push(FactsPlayerSeat {
                film_index: r.signed(),
                xuid: r.unsigned(),
                gamertag: r.string_bytes().to_vec(),
            });
        }
    }
    t
}
pub fn encode_facts_player_teams(
    w: &mut NativeFactsWriter,
    teams: Option<&BTreeMap<i64, i64>>,
    s: &FactsTeamScan,
) {
    w.unsigned(teams.map_or(0, |m| m.len()) as u64);
    if let Some(m) = teams {
        for (&k, &v) in m {
            w.signed(k);
            w.signed(v);
        }
    }
    w.boolean(s.archetype_absent);
    w.boolean(s.component_mismatch);
    w.string_bytes(&s.component);
    for n in [
        s.packets,
        s.records,
        s.read,
        s.unreached,
        s.out_of_domain_index,
        s.out_of_domain_value,
        s.entities,
        s.entity_divergences,
        s.index_divergences,
        s.indices,
        s.no_team,
    ] {
        w.unsigned(n as u64);
    }
}
pub fn decode_facts_player_teams(
    r: &mut NativeFactsReader<'_>,
) -> (Option<BTreeMap<i64, i64>>, FactsTeamScan) {
    let n = r.unsigned() as i64;
    let mut teams = None;
    if n > 0 {
        let m = teams.get_or_insert_with(BTreeMap::new);
        for _ in 0..n {
            if r.error().is_some() {
                break;
            }
            let k = r.signed();
            m.insert(k, r.signed());
        }
    }
    (
        teams,
        FactsTeamScan {
            archetype_absent: r.boolean(),
            component_mismatch: r.boolean(),
            component: r.string_bytes().to_vec(),
            packets: r.unsigned() as i64,
            records: r.unsigned() as i64,
            read: r.unsigned() as i64,
            unreached: r.unsigned() as i64,
            out_of_domain_index: r.unsigned() as i64,
            out_of_domain_value: r.unsigned() as i64,
            entities: r.unsigned() as i64,
            entity_divergences: r.unsigned() as i64,
            index_divergences: r.unsigned() as i64,
            indices: r.unsigned() as i64,
            no_team: r.unsigned() as i64,
        },
    )
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsQueue {
    #[serde(rename = "Deaths")]
    pub deaths: Vec<FactsDeath>,
    #[serde(rename = "PlayerIndices")]
    pub player_indices: FactsPlayerIndices,
    #[serde(rename = "FilmTable")]
    pub film_table: FactsPlayerTable,
    #[serde(rename = "PlayerTeams")]
    pub player_teams: Option<BTreeMap<i64, i64>>,
    #[serde(rename = "TeamScan")]
    pub team_scan: FactsTeamScan,
}
pub fn encode_facts_queue(w: &mut NativeFactsWriter, g: &FactsQueue) {
    w.unsigned(g.deaths.len() as u64);
    for d in &g.deaths {
        w.unsigned(d.xuid);
        w.string_bytes(&d.gamertag);
        w.signed(d.time_ms);
    }
    w.unsigned(g.player_indices.readings as u64);
    w.unsigned(g.player_indices.disagreements as u64);
    w.unsigned(g.player_indices.by_xuid.len() as u64);
    for (&x, &index) in &g.player_indices.by_xuid {
        w.unsigned(x);
        w.signed(index);
    }
    encode_facts_player_table(w, &g.film_table);
    encode_facts_player_teams(w, g.player_teams.as_ref(), &g.team_scan);
}
pub fn decode_facts_queue(r: &mut NativeFactsReader<'_>) -> FactsQueue {
    let mut deaths = Vec::new();
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        deaths.push(FactsDeath {
            xuid: r.unsigned(),
            gamertag: r.string_bytes().to_vec(),
            time_ms: r.signed(),
        });
    }
    let mut player_indices = FactsPlayerIndices {
        readings: r.unsigned() as i64,
        disagreements: r.unsigned() as i64,
        ..Default::default()
    };
    let n = r.unsigned() as i64;
    for _ in 0..n {
        if r.error().is_some() {
            break;
        }
        let x = r.unsigned();
        player_indices.by_xuid.insert(x, r.signed());
    }
    let film_table = decode_facts_player_table(r);
    let (player_teams, team_scan) = decode_facts_player_teams(r);
    FactsQueue {
        deaths,
        player_indices,
        film_table,
        player_teams,
        team_scan,
    }
}
