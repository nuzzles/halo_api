//! Join recorded weapon pickups to bounded pad occupations without removing bounds.
use super::{GroundPadPickup, GroundWeaponPad};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
/// Normalize exactly eight hex digits, with an optional 0x/0X prefix.
/// Canonical power-up names do not enter this join.
pub fn pad_weapon_family_key(value: &str) -> Option<String> {
    let s = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);
    (s.len() == 8 && s.bytes().all(|c| c.is_ascii_hexdigit())).then(|| s.to_ascii_lowercase())
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayPickup {
    pub t: i64,
    #[serde(default)]
    pub slot: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub family: String,
    #[serde(default)]
    pub class: u8,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub origin: String,
    pub w: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub xuid: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PadDatingStats {
    pub occupations: usize,
    pub dated: usize,
    pub named: usize,
    pub ambiguous: usize,
    pub uncovered: usize,
    pub powerup_occupations: usize,
}
/// Only one matching native event inside the inclusive frame interval dates an
/// occupation. Ambiguous/uncovered occupations keep all preexisting fields.
pub fn date_pad_pickups(
    pads: &[GroundWeaponPad],
    picks: &mut [GroundPadPickup],
    pickups: &[ReplayPickup],
) -> PadDatingStats {
    let mut stats = PadDatingStats {
        occupations: picks.len(),
        ..Default::default()
    };
    let mut by_family = BTreeMap::<String, Vec<&ReplayPickup>>::new();
    for p in pickups {
        if p.kind == "weapon"
            && let Some(key) = pad_weapon_family_key(&p.w)
        {
            by_family.entry(key).or_default().push(p);
        }
    }
    for p in picks {
        let Some(pad) = usize::try_from(p.pad).ok().and_then(|i| pads.get(i)) else {
            stats.uncovered += 1;
            continue;
        };
        let Some(key) = pad_weapon_family_key(&pad.weapon) else {
            stats.powerup_occupations += 1;
            continue;
        };
        let hits: Vec<_> = by_family
            .get(&key)
            .into_iter()
            .flatten()
            .filter(|h| h.t >= p.t_low && h.t <= p.t_high)
            .collect();
        match hits.as_slice() {
            [] => stats.uncovered += 1,
            [h] => {
                p.t = Some(h.t);
                stats.dated += 1;
                if !h.xuid.is_empty() {
                    p.xuid = Some(h.xuid.clone());
                    stats.named += 1;
                }
            }
            _ => stats.ambiguous += 1,
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_pad_pickup_dating() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/pad-dating-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for row in rows["keys"].as_array().unwrap() {
            let expected = if row["valid"] == true {
                Some(row["key"].as_str().unwrap().into())
            } else {
                None
            };
            assert_eq!(
                pad_weapon_family_key(row["input"].as_str().unwrap()),
                expected
            );
        }
        for (i, row) in rows["cases"].as_array().unwrap().iter().enumerate() {
            let mut value = row["pads"].clone();
            for p in value.as_array_mut().unwrap() {
                p["spawns"] = serde_json::json!([]);
                p["presence"] = serde_json::json!([]);
            }
            let pads: Vec<GroundWeaponPad> = serde_json::from_value(value).unwrap();
            let mut picks: Vec<GroundPadPickup> =
                serde_json::from_value(row["input"].clone()).unwrap();
            let events: Vec<ReplayPickup> = serde_json::from_value(row["events"].clone()).unwrap();
            let got = date_pad_pickups(&pads, &mut picks, &events);
            let expected: PadDatingStats = serde_json::from_value(row["stats"].clone()).unwrap();
            let expected_picks: Vec<GroundPadPickup> =
                serde_json::from_value(row["picks"].clone()).unwrap();
            assert_eq!(got, expected, "stats {i}");
            assert_eq!(picks, expected_picks, "picks {i}");
        }
    }
}
