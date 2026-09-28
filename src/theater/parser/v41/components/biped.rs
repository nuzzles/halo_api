//! Biped action and simple world-state wire fields.
use super::Reader;
use crate::theater::parser::v41::FilmComponentObservation;

pub(super) fn component(r: &mut Reader<'_>, name: &str) -> Option<bool> {
    match name {
        "biped-spartan-ability-energy-component" | "biped-spartan-ability-energy" => {
            let mask = r.r("mask", 3)?;
            let mut charges = [-1; 3];
            for (i, charge) in charges.iter_mut().enumerate() {
                if mask & (1 << i) != 0 {
                    *charge = r.r(&format!("charge[{i}]"), 7)? as i32;
                }
            }
            r.publish_component(FilmComponentObservation::AbilityEnergy {
                mask: mask as u32,
                charges,
            });
        }
        "biped-desired-grenade-set" | "biped-desired-grenade-set-component" => {
            let mask = r.r("mask", 6)? as u32;
            let selection = r.r("selection", 3)? as i32;
            r.publish_component(FilmComponentObservation::GrenadeSet { mask, selection });
        }
        "biped-desired-ability-set" | "biped-desired-ability-set-component" => {
            let start = r.cursor.position;
            let counter = r.r("counter", 3)?;
            let rank = r.gated_value("rank", 6, false)?.map_or(-1, |v| v as i32);
            let width = usize::try_from(r.cursor.position.wrapping_sub(start)).ok()?;
            r.publish_component(FilmComponentObservation::AbilitySet {
                counter,
                rank,
                width,
            });
        }
        "biped-map-editor-flag" | "biped-map-editor-flag-component" => {
            r.r("flags", 8)?;
        }
        "biped-low-frequency-data" | "biped-low-frequency-data-component" => {
            r.words("flags", 3, 1)?;
            if r.bit("voice_present")? {
                r.r("voice_id", 32)?;
                r.r("voice_designator", 32)?;
            }
        }
        "simulation-state-playback" | "simulation-state-playback-component" => {
            if r.bit("present")? {
                r.r("kind", 4)?;
                r.words("state", 2, 32)?;
            }
        }
        "biped-action" | "biped-action-component" => action(r)?,
        "spawn-filter-weight-component" => {
            r.r("weight", 16)?;
        }
        "managed-object-participant-respawn-block-component" => {
            r.r("reference", 32)?;
            r.gate("player", 5, false)?;
        }
        "item-ignore-player-component" => r.gate("player", 5, false)?,
        "game-engine-shared-team-lives-component" => r.words("lives", 8, 8)?,
        "change-scene-component" => {
            r.r("kind", 6)?;
            let mut bits = r.r("length", 12)? as usize;
            let mut i = 0;
            while bits > 0 {
                let width = bits.min(32);
                r.r(&format!("payload[{i}]"), width)?;
                bits -= width;
                i += 1;
            }
        }
        "statborg-current-round-value-stat-component" => {
            r.r("index_a", 5)?;
            r.r("index_b", 5)?;
            signed(r, "value_a")?;
            signed(r, "value_b")?;
            let a = r.bit("has_a")?;
            let b = r.bit("has_b")?;
            if a {
                signed(r, "extra_a")?;
            }
            if b {
                signed(r, "extra_b")?;
            }
        }
        "statborg-finalized-rounds-values-stat-component" => {
            let mask = r.r("mask", 32)?;
            for i in 0..32 {
                if mask & (1 << i) == 0 {
                    continue;
                }
                for side in ["a", "b"] {
                    let name = format!("round[{i}].{side}");
                    if !r.bit(&format!("{name}.absent"))? {
                        signed(r, &name)?;
                    }
                }
            }
        }
        _ => return Some(false),
    }
    Some(true)
}

fn signed(r: &mut Reader<'_>, name: &str) -> Option<()> {
    let selector = r.r(&format!("{name}.selector"), 2)?;
    r.r(name, 8 << selector)?;
    Some(())
}

fn action(r: &mut Reader<'_>) -> Option<()> {
    let a = r.r("mask[0]", 32)? as u32;
    let b = r.r("mask[1]", 32)? as u32;
    let c = r.r("mask[2]", 32)? as u32;
    let count = r.r("count", 4)?;
    for i in 0..count {
        r.r(&format!("action[{i}].index"), 7)?;
        let tag = r.r(&format!("action[{i}].tag"), 5)?;
        match tag {
            0 => {
                r.optional_handle("reference", 0)?;
                r.gate("value", 8, true)?;
            }
            1 => {
                r.gate("value", 8, true)?;
                r.words("bytes", 2, 8)?;
                r.r("word", 32)?;
                r.r("value15", 15)?;
            }
            2 => {
                r.r("byte", 8)?;
                r.gate("value", 8, true)?;
                r.r("word", 16)?;
            }
            3 => {
                r.r("enum", 2)?;
                r.r("value15", 15)?;
            }
            4 => {
                r.r("variant", 32)?;
                r.r("byte", 8)?;
                r.r("value15", 15)?;
                r.direction(10)?;
                r.gate("value", 8, true)?;
                r.r("word", 16)?;
            }
            5 => {
                r.r("variant", 32)?;
                r.gate("value", 8, true)?;
                r.r("word", 16)?;
                r.r("byte", 8)?;
            }
            _ => {}
        }
    }
    let count = a.count_ones() + b.count_ones() + (c & 0x1ff).count_ones();
    for i in 0..count {
        r.gate(&format!("states[{i}]"), 2, true)?;
    }
    r.words("tail_mask", 3, 32)?;
    Some(())
}
