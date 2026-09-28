//! Remaining player, scene and match-state readers used by production frame traversal.
use super::{Reader, basic, position};

fn text_argument(r: &mut Reader<'_>, name: &str) -> Option<()> {
    match r.r(&format!("{name}.kind"), 3)? {
        0 => {}
        1 => r.gate(&format!("{name}.player"), 5, false)?,
        2 => {
            let short = r.bit(&format!("{name}.short"))?;
            r.r(&format!("{name}.value"), if short { 24 } else { 32 })?;
        }
        _ => {
            r.r(&format!("{name}.value"), 32)?;
        }
    }
    Some(())
}

fn quant_vector(r: &mut Reader<'_>, level: u32) -> Option<()> {
    quant_vector_values(r, level).map(|_| ())
}

fn quant_vector_values(r: &mut Reader<'_>, level: u32) -> Option<Option<[u64; 3]>> {
    if r.bit("vector_default")? {
        return Some(None);
    }
    r.gate("vector_index", 1, false)?;
    let width = (6_u64 + u64::from(level)).min(26) as usize;
    let mut values = [0; 3];
    for (i, value) in values.iter_mut().enumerate() {
        *value = r.r(&format!("vector_axis[{i}]"), width)?;
    }
    Some(Some(values))
}

pub(super) fn component(
    r: &mut Reader<'_>,
    name: &str,
    level: u32,
    archetype: u32,
) -> Option<bool> {
    match name {
        "asset-transform-component" => {
            for i in 0..5 {
                r.gate(&format!("transform[{i}].index"), 1, false)?;
                r.words(
                    &format!("transform[{i}].axis"),
                    3,
                    (6_u64 + u64::from(level)).min(26) as usize,
                )?;
            }
        }
        "crew-orders-off-flags-component" | "managed-player-campaign-progress-component" => {
            r.r("value", 8)?;
        }
        "flock-current-destination-component" => {
            r.r("destination", 4)?;
        }
        "flock-emitting-component"
        | "flock-forced-respawn-component"
        | "managed-player-show-active-mission-name-in-hud-component" => {
            r.bit("value")?;
        }
        "managed-object-property-name-component" => {
            let value = r.r("name", 32)?;
            r.publish_component(crate::theater::parser::v41::FilmComponentObservation::Probe {
                archetype,
                component: crate::theater::parser::v41::NativeProbeComponent::ManagedObjectPropertyName,
                values: vec![value],
            });
        }
        "managed-player-forge-weather-effect-overrides-component" => r.words("overrides", 2, 32)?,
        "player-power-frame-points-component" => r.words("points", 2, 16)?,
        "player-supply-lines-currency-simulation-component" => {
            r.r("currency", 16)?;
        }
        "managed-player-custom-input-prompt-widget" => {
            if r.bit("present")? {
                r.r("mode", 2)?;
                basic::formatted_text(r)?;
            }
        }
        "tacmap-poiisgoldenpath" => r.words("flags", 2, 1)?,
        "tacmap-poiiconoffset" => quant_vector(r, level)?,
        "tacmap-poiicon" => {
            r.words("ids", 2, 32)?;
            r.bit("flag")?;
            r.r("kind", 3)?;
            r.words("assets", 2, 32)?;
            r.words("values", 2, 9)?;
            quant_vector(r, level)?;
            r.r("string", 32)?;
            r.bit("tail_flag")?;
            r.words("color", 4, 8)?;
        }
        "managed-object-networked-splash-message-static-component" => {
            basic::formatted_text(r)?;
            let value = r.r("value", 24)?;
            if r.bit("body_present")? {
                let count = r.r("first_count", 3)?;
                for i in 0..count {
                    r.r(&format!("first[{i}].a"), 16)?;
                    r.words(&format!("first[{i}].bytes"), 2, 8)?;
                    r.bit(&format!("first[{i}].flag"))?;
                    r.r(&format!("first[{i}].b"), 32)?;
                }
                let count = r.r("second_count", 2)?;
                for i in 0..count {
                    r.r(&format!("second[{i}].a"), 32)?;
                    r.r(&format!("second[{i}].b"), 16)?;
                    if r.bit(&format!("second[{i}].present"))? {
                        text_argument(r, &format!("second[{i}].argument"))?;
                    }
                }
                if count > 0 {
                    r.r("second_tail", 8)?;
                }
                if r.r("tail_kind", 3)? != 0 {
                    r.r("tail_value", 32)?;
                }
                r.gate("tail_optional", 32, true)?;
                if r.bit("tail_present")? {
                    r.r("tail_a", 3)?;
                    r.r("tail_b", 6)?;
                    r.r("tail_c", 32)?;
                }
            }
            r.publish_component(
                crate::theater::parser::v41::FilmComponentObservation::Probe {
                    archetype,
                    component: crate::theater::parser::v41::NativeProbeComponent::SplashStatic,
                    values: vec![value],
                },
            );
        }
        "effect-state-data-component" => {
            r.gate("tag", 32, true)?;
            if r.bit("marker_a_present")? {
                return Some(false);
            }
            r.r("marker_name", 32)?;
            if r.bit("marker_b_present")? {
                return Some(false);
            }
        }
        "flock-destroying-component"
        | "game-engine-game-finished-component"
        | "player-early-respawn-requested-component"
        | "player-vehicle-entrance-ban-component" => {
            r.bit("value")?;
        }
        "player-engine-loadout-index-component" => {
            r.r("index", 3)?;
        }
        "player-representation-component"
        | "managed-player-back-button-scoreboard-flair-component"
        | "managed-player-current-season-component" => {
            r.r("value", 32)?;
        }
        "managed-player-active-mission-name-component" => r.words("name", 2, 32)?,
        "managed-player-color-override-component" => r.words("colors", 8, 8)?,
        "managed-player-flags-component" => {
            r.r("flags", 4)?;
        }
        "statborg-entry-index-and-type-component" => {
            r.r("index", 32)?;
            r.r("type", 8)?;
        }
        "statborg-round-outcomes-component" => r.words("outcomes", 32, 2)?,
        "game-engine-alliance-component" => {
            let mask = r.r("mask", 32)?;
            for i in 0..32 {
                if mask >> i & 1 != 0 {
                    r.words(&format!("alliance[{i}]"), 2, 32)?;
                }
            }
        }
        "game-engine-team-mapping" | "game-engine-team-mapping-component" => {
            r.r("a", 8)?;
            r.r("b", 9)?;
            r.r("c", 9)?;
            let mask = r.r("mask", 8)?;
            r.r("d", 8)?;
            r.r("e", 8)?;
            for i in 0..8 {
                if mask >> i & 1 != 0 {
                    r.r(&format!("teams[{i}]"), 4)?;
                }
            }
        }
        "crew-marked-objects-component"
        | "nav-cutscene-flag-component"
        | "player-primary-respawn-object-component"
        | "player-desired-respawn-seat-component" => {
            if r.bit("present")? {
                return Some(false);
            }
            if name == "player-desired-respawn-seat-component" {
                r.r("seat", 7)?;
            }
        }
        "player-desired-respawn-location-component" => {
            let mut values = Vec::new();
            let mut present = false;
            if r.bit("present")? {
                let vector = quant_vector_values(r, level)?;
                let id = r.r("id", 19)?;
                if let Some(vector) = vector {
                    values.extend(vector);
                    values.extend([id, u64::from(level)]);
                    present = true;
                } else {
                    values.push(id);
                }
            }
            r.publish_component(
                crate::theater::parser::v41::FilmComponentObservation::PlayerState {
                    field:
                        crate::theater::parser::v41::NativePlayerStateField::DesiredRespawnLocation,
                    values,
                    present,
                },
            );
        }

        "flock-position-component" => {
            if r.position_encoding.is_some_and(|p| p.full_precision_gate()) {
                r.words("vector_bits", 3, 32)?;
            } else {
                r.gate("index", 1, false)?;
                r.words("axis", 3, (6_u64 + u64::from(level)).min(26) as usize)?;
            }
        }
        "music-state-component" => {
            r.bit("flag")?;
            r.r("value", 32)?;
            for i in 0..2 {
                r.r(&format!("state[{i}]"), 32)?;
                r.bit(&format!("flag[{i}]"))?;
            }
            if r.bit("pair_present")? {
                r.bit("pair_flag")?;
                r.words("pair", 2, 16)?;
            }
            if r.bit("values_present")? {
                r.words("values", 6, 16)?;
            }
        }
        "unit-malleable-property-component" => {
            r.gate("head", 12, true)?;
            if level > 2 {
                r.bit("head_flag")?;
            }
            for i in 0..7 {
                r.gate(&format!("properties[{i}]"), 12, true)?;
            }
            r.words("flags", 4, 1)?;
            if level > 2 {
                r.bit("flag_level3")?;
            }
            if level > 3 {
                r.bit("flag_level4")?;
            }
            for i in 0..4 {
                r.gate(&format!("tail[{i}]"), 12, true)?;
            }
        }
        "track-frame-component" => {
            r.r("frame", 6)?;
            let flag = r.bit("flag1")?;
            r.bit("flag2")?;
            if !flag {
                let width = r.r("width", 12)? as usize;
                for offset in (0..width).step_by(64) {
                    r.r(&format!("body[{}]", offset / 64), (width - offset).min(64))?;
                }
            }
        }
        "tacmap-areaofinterest" => {
            r.r("id", 32)?;
            r.r("kind", 3)?;
            position::traversal_payload(r)?;
            r.r("value", 12)?;
        }
        "spawn-filter-type-component" => match r.r("tag", 2)? {
            0 => {}
            1 => r.gate("player", 5, false)?,
            2 => {
                r.handle("object", 1)?;
                r.r("value", 6)?;
            }
            _ => {
                r.r("id", 32)?;
                position::traversal_payload(r)?;
                r.r("kind", 3)?;
                let count = r.r("count", 4)?;
                r.words("entries", count as usize, 32)?;
                r.gate("player", 5, false)?;
            }
        },
        _ => return Some(false),
    }
    Some(true)
}
