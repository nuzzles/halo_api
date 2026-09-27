//! Pinned title objective roles and raw film-variant matching.
use super::*;
use std::{collections::BTreeSet, sync::OnceLock};
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayObjectiveMode {
    pub r#match: Vec<String>,
    pub roles: Vec<String>,
    pub neutral: bool,
    pub points_only: bool,
}
pub fn replay_objective_modes() -> &'static [ReplayObjectiveMode] {
    static MODES: OnceLock<Vec<ReplayObjectiveMode>> = OnceLock::new();
    MODES.get_or_init(|| {
        serde_json::from_str(include_str!("reference/objective-modes-v75.json"))
            .expect("embedded objective mode table")
    })
}
fn lowercase(s: &str) -> String {
    s.chars()
        .map(|c| c.to_lowercase().next().unwrap_or(c))
        .collect()
}
/// Native Go matcher uses ASCII byte boundaries, including underscore, and
/// prefers the longest token by UTF-8 byte length, preserving first-match ties.
pub fn replay_extract_known_mode(label: &str, tokens: &[String]) -> String {
    let label = lowercase(label.trim());
    let mut best = "";
    for token in tokens {
        let token = token.trim();
        if token.is_empty() || token.len() <= best.len() {
            continue;
        }
        let needle = lowercase(token);
        let bytes = label.as_bytes();
        let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        if bytes
            .windows(needle.len())
            .enumerate()
            .filter(|(_, w)| *w == needle.as_bytes())
            .any(|(i, _)| {
                (i == 0 || !word(bytes[i - 1]))
                    && (i + needle.len() == bytes.len() || !word(bytes[i + needle.len()]))
            })
        {
            best = token;
        }
    }
    best.to_owned()
}
pub fn replay_objective_role_specs(variant: &str) -> Vec<ObjectiveRoleSpec> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for mode in replay_objective_modes() {
        if replay_extract_known_mode(variant, &mode.r#match).is_empty() {
            continue;
        }
        for role in &mode.roles {
            if seen.insert(role.clone()) {
                out.push(ObjectiveRoleSpec {
                    role: role.clone(),
                    neutral: mode.neutral,
                    points_only: mode.points_only,
                });
            }
        }
    }
    out
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReplayMatchObjectives {
    pub map: Option<ReplayMapObjectives>,
    pub held_zones: Vec<ObjectiveZone>,
    pub held_roles: String,
    pub flag_spawns: Vec<ReplayFlagSpawn>,
    pub hill: bool,
}
/// Resolve by map asset ID and the raw film variant. HTTP pair-name normalization
/// is a separate reference-service concern and is not applied here.
pub fn replay_match_objectives(map_id: &str, variant: &str) -> ReplayMatchObjectives {
    let mut out = ReplayMatchObjectives {
        hill: replay_objective_type(variant) == "hill",
        ..Default::default()
    };
    let Some(entry) = replay_map_objectives_catalog()
        .lookup(map_id)
        .filter(|_| !map_id.is_empty())
    else {
        return out;
    };
    let specs = replay_objective_role_specs(variant);
    out.map = build_replay_map_objectives(entry, &specs);
    out.flag_spawns = entry.flag_spawns();
    let mut roles = Vec::new();
    for spec in &specs {
        if spec.role == "strongholds_zone" || spec.role == "hill" {
            out.held_zones.extend(entry.zones_of_role(&spec.role).zones);
            roles.push(spec.role.as_str());
        }
    }
    if !out.held_zones.is_empty() {
        out.held_roles = roles.join(",");
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(serde::Deserialize)]
    struct Case {
        label: String,
        tokens: Vec<String>,
        matched: String,
        specs: Vec<ObjectiveRoleSpec>,
    }
    #[test]
    fn native_objective_mode_selection() {
        let cases: Vec<Case> =
            serde_json::from_str(include_str!("fixtures/objective-modes-v41.json")).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                replay_extract_known_mode(&c.label, &c.tokens),
                c.matched,
                "match {i}"
            );
            assert_eq!(replay_objective_role_specs(&c.label), c.specs, "specs {i}");
        }
    }
}

#[cfg(test)]
mod map_tests {
    use super::*;
    use std::io::Read;
    #[derive(serde::Deserialize)]
    struct Case {
        map_id: String,
        variant: String,
        map: Option<ReplayMapObjectives>,
        held: Vec<i64>,
        roles: String,
    }
    #[test]
    fn native_objective_map_selection() {
        let mut b = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/objective-map-selection-v41.json.zlib")[..],
        )
        .read_to_end(&mut b)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&b).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let selected = replay_match_objectives(&c.map_id, &c.variant);
            assert_eq!(selected.map, c.map, "map {i}");
            assert_eq!(selected.held_roles, c.roles, "roles {i}");
            assert_eq!(
                selected
                    .held_zones
                    .iter()
                    .map(|z| z.object_idx)
                    .collect::<Vec<_>>(),
                c.held,
                "held {i}"
            );
        }
        for id in ["", "unknown"] {
            let selected = replay_match_objectives(id, "Strongholds");
            assert!(selected.map.is_none());
            assert!(selected.held_roles.is_empty());
            assert!(selected.held_zones.is_empty());
            assert!(selected.flag_spawns.is_empty());
        }
    }
}
