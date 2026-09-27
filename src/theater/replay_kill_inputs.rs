//! Native offline projection of decoded kills into replay identity and coverage inputs.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayDecodedKillInputs {
    pub read: bool,
    pub equipment: Vec<ReplayEquipmentKill>,
    pub pairs: Vec<ReplayKillReference>,
    pub dropped_pairs: usize,
    pub killer_unresolved: usize,
    pub victim_unresolved: usize,
    pub paths: Option<ReplayDeathsPathsCoverage>,
    pub neutral_deaths: Vec<ReplayNeutralDeath>,
}
/// First nonempty gamertag wins even when its recorded XUID is zero.
pub fn replay_kill_identity_index(deaths: &[IdentityDeath]) -> BTreeMap<ReplayByteString, u64> {
    let mut out = BTreeMap::new();
    for d in deaths {
        if !d.gamertag.is_empty() {
            out.entry(d.gamertag.clone()).or_insert(d.xuid);
        }
    }
    out
}
/// A malformed xuid prefix never falls back to a same-named gamertag entry.
pub fn resolve_replay_kill_identity(
    name: impl AsRef<[u8]>,
    names: &BTreeMap<ReplayByteString, u64>,
) -> Option<u64> {
    let name = name.as_ref();
    if let Some(text) = name.strip_prefix(b"xuid:") {
        if text.is_empty() || !text.iter().all(u8::is_ascii_digit) {
            return None;
        }
        text.iter().try_fold(0u64, |v, b| {
            v.checked_mul(10)?.checked_add(u64::from(b - b'0'))
        })
    } else {
        names.get(name).copied()
    }
}
pub fn resolve_replay_kill_references(
    kills: &[AttributedFilmKill],
    deaths: &[IdentityDeath],
) -> ReplayDecodedKillInputs {
    let names = replay_kill_identity_index(deaths);
    let mut out = ReplayDecodedKillInputs {
        read: true,
        ..Default::default()
    };
    for k in kills {
        let Some(killer) = resolve_replay_kill_identity(&k.killer, &names) else {
            out.killer_unresolved += 1;
            continue;
        };
        let assist = if k.assist.known && !k.assist.name.is_empty() {
            resolve_replay_kill_identity(&k.assist.name, &names)
        } else {
            None
        };
        out.equipment.push(ReplayEquipmentKill {
            xuid: killer,
            time_ms: k.time_ms,
            assist_xuid: assist.unwrap_or(0),
            assist_known: assist.is_some(),
        });
        let Some(victim) = resolve_replay_kill_identity(&k.victim, &names) else {
            out.victim_unresolved += 1;
            continue;
        };
        out.pairs.push(ReplayKillReference {
            killer_xuid: killer,
            victim_xuid: victim,
            time_ms: k.time_ms,
        });
    }
    out.dropped_pairs = kills.len() - out.pairs.len();
    out
}
/// Use only for unclaimed deaths: ordinary kills cannot be classified as suicides.
pub fn replay_neutral_death_icon(tag: u32) -> Option<(&'static str, &'static str)> {
    let (kind, sprite) = super::kill_icons::neutral_death_sprite(tag)?;
    let image = match sprite {
        "killfeed-55" => "/static/weapons-assets/halo_infinite/jeu/killfeed-55.png",
        "killfeed-61" => "/static/weapons-assets/halo_infinite/jeu/killfeed-61.png",
        _ => unreachable!("native neutral sprite table"),
    };
    Some((kind, image))
}
pub fn project_replay_neutral_deaths(
    deaths: &[UnclaimedFilmDeath],
    publishable: bool,
) -> Vec<ReplayNeutralDeath> {
    if !publishable {
        return Vec::new();
    }
    deaths
        .iter()
        .filter(|d| d.victim_xuid != 0)
        .filter_map(|d| {
            let (kind, img) = replay_neutral_death_icon(d.source.tag)?;
            Some(ReplayNeutralDeath {
                xuid: d.victim_xuid.to_string(),
                feed_ms: d.time_ms,
                kind: kind.into(),
                img: img.into(),
                tinted: true,
            })
        })
        .collect()
}
/// Missing/failed death reads close both kill-reference inputs. Neutral deaths
/// retain their independent XUID source and only share the line-publication gate.
pub fn build_replay_decoded_kill_inputs(
    result: Option<&FilmKillSourceResult>,
    deaths: Option<&[IdentityDeath]>,
) -> ReplayDecodedKillInputs {
    let Some(result) = result else {
        return ReplayDecodedKillInputs::default();
    };
    let publishable = result.line_by_line_publishable();
    let mut out = if publishable {
        deaths
            .map(|d| resolve_replay_kill_references(&result.attribution.kills, d))
            .unwrap_or_default()
    } else {
        ReplayDecodedKillInputs::default()
    };
    if out.read {
        let tally = |s: &KillPathStats| ReplayDeathsPathTally {
            population: s.population,
            matched: s.matched,
            published: s.published,
        };
        out.paths = Some(ReplayDeathsPathsCoverage {
            walk: tally(&result.attribution.walk),
            direct_scan: tally(&result.attribution.scan),
        });
    }
    out.neutral_deaths = project_replay_neutral_deaths(&result.attribution.unclaimed, publishable);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    #[test]
    fn native_replay_kill_inputs_oracle() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-kill-inputs-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let fixture: Value = serde_json::from_slice(&bytes).unwrap();
        for (i, row) in fixture["rows"].as_array().unwrap().iter().enumerate() {
            let deaths: Vec<IdentityDeath> = serde_json::from_value(row["deaths"].clone()).unwrap();
            let index = replay_kill_identity_index(&deaths);
            assert_eq!(json!(index), row["index"]);
            for entry in row["identities"].as_array().unwrap() {
                assert_eq!(
                    json!(resolve_replay_kill_identity(
                        entry["name"].as_str().unwrap(),
                        &index
                    )),
                    entry["value"],
                    "identity {i}"
                );
            }
            let kills: Vec<_> = row["kills"]
                .as_array()
                .unwrap()
                .iter()
                .map(|k| AttributedFilmKill {
                    time_ms: k["time_ms"].as_i64().unwrap(),
                    victim: k["victim"].as_str().unwrap().into(),
                    killer: k["killer"].as_str().unwrap().into(),
                    feed_present: false,
                    source: kill_source_truth(0, 0),
                    diverges: false,
                    read: KillSourceProvenance {
                        path: KillReadPath::Walk,
                        origin: String::new(),
                        multiplicity: 0,
                    },
                    packet: None,
                    assist: KillAssist {
                        name: k["assist_name"].as_str().unwrap().into(),
                        known: k["assist_known"].as_bool().unwrap(),
                        ..Default::default()
                    },
                    killer_damage: KillDamageShare::default(),
                    assist_damage: KillDamageShare::default(),
                })
                .collect();
            assert_eq!(
                json!(resolve_replay_kill_references(&kills, &deaths)),
                row["expected"],
                "row {i}"
            );
        }
        for row in fixture["icons"].as_array().unwrap() {
            assert_eq!(
                json!(replay_neutral_death_icon(
                    row["tag"].as_u64().unwrap() as u32
                )),
                row["icon"]
            );
        }
    }
}
