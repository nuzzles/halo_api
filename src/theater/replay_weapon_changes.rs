//! Publish held-weapon identity changes on the replay clock.
use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayWeaponChange {
    pub t: i64,
    pub slot: u32,
    pub kind: HeldWeaponChangeKind,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub w: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub from: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayWeaponChangeCoverage {
    pub decoded: usize,
    pub published: usize,
    pub restated: usize,
    pub before_origin: usize,
    pub taken: usize,
    pub dropped: usize,
    pub swapped: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayWeaponChanges {
    pub changes: Vec<ReplayWeaponChange>,
    pub coverage: ReplayWeaponChangeCoverage,
}
/// Keep scan order, without a published-slot filter or an end-frame clamp.
/// Restated weapons are excluded before the origin check. All-ones is the absent
/// family sentinel; zero is a real family and retains its eight-digit identifier.
pub fn build_replay_weapon_changes(
    raw: &[HeldWeaponChange],
    origin: u64,
    step: u64,
) -> ReplayWeaponChanges {
    build_weapon_change_values(
        raw.iter().map(|r| WeaponChangeValue {
            timestamp_us: r.timestamp_us,
            slot: r.slot,
            kind: r.kind,
            family: r.family,
            previous: r.previous,
        }),
        origin,
        step,
    )
}
/// Native replay normalizes unknown cache kinds to taken. Original kind bytes
/// and the unpublished slot index remain in FactsWeaponChange.
pub fn build_facts_replay_weapon_changes(
    raw: &[FactsWeaponChange],
    origin: u64,
    step: u64,
) -> ReplayWeaponChanges {
    build_weapon_change_values(
        raw.iter().map(|r| WeaponChangeValue {
            timestamp_us: r.timestamp_us,
            slot: r.slot,
            family: r.family,
            previous: r.previous,
            kind: match r.kind.as_slice() {
                b"restated" => HeldWeaponChangeKind::Restated,
                b"dropped" => HeldWeaponChangeKind::Dropped,
                b"swapped" => HeldWeaponChangeKind::Swapped,
                _ => HeldWeaponChangeKind::Taken,
            },
        }),
        origin,
        step,
    )
}
struct WeaponChangeValue {
    timestamp_us: u64,
    slot: u32,
    kind: HeldWeaponChangeKind,
    family: u32,
    previous: u32,
}
fn build_weapon_change_values(
    raw: impl ExactSizeIterator<Item = WeaponChangeValue>,
    origin: u64,
    step: u64,
) -> ReplayWeaponChanges {
    let mut out = ReplayWeaponChanges {
        coverage: ReplayWeaponChangeCoverage {
            decoded: raw.len(),
            ..Default::default()
        },
        ..Default::default()
    };
    if step == 0 {
        return out;
    }
    let family = |f| {
        if f == u32::MAX {
            String::new()
        } else {
            format!("{f:08x}")
        }
    };
    for r in raw {
        if r.kind == HeldWeaponChangeKind::Restated {
            out.coverage.restated += 1;
            continue;
        }
        if r.timestamp_us < origin {
            out.coverage.before_origin += 1;
            continue;
        }
        out.changes.push(ReplayWeaponChange {
            t: ((r.timestamp_us - origin) / step) as i64,
            slot: r.slot,
            kind: r.kind,
            w: family(r.family),
            from: family(r.previous),
        });
        out.coverage.published += 1;
        match r.kind {
            HeldWeaponChangeKind::Taken => out.coverage.taken += 1,
            HeldWeaponChangeKind::Dropped => out.coverage.dropped += 1,
            HeldWeaponChangeKind::Swapped => out.coverage.swapped += 1,
            HeldWeaponChangeKind::Restated => unreachable!(),
        }
    }
    out
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_weapon_changes(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> ReplayWeaponChanges {
    build_replay_weapon_changes(
        film.weapon_changes
            .as_ref()
            .map_or(&[][..], |s| s.records.as_slice()),
        players.clock.origin_us,
        players.clock.step_us,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        raw: Vec<HeldWeaponChange>,
        origin: u64,
        step: u64,
        output: ReplayWeaponChanges,
    }
    #[test]
    fn native_weapon_change_publication() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-weapon-changes-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                build_replay_weapon_changes(&c.raw, c.origin, c.step),
                c.output,
                "case {i}"
            );
        }
    }
}
