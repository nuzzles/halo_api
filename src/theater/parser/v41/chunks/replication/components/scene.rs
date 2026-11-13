//! Remaining player, scene and match-state readers used by production frame traversal.
use super::reader::ComponentReader;
use super::{basic, orientation, position};
use crate::theater::parser::v41::reference::observations::{
    FilmComponentObservation, PlayerStateField, ProbeComponent,
};

fn text_argument(r: &mut ComponentReader<'_>, name: &str) -> Option<()> {
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

fn quant_vector(r: &mut ComponentReader<'_>, level: u32) -> Option<()> {
    quant_vector_values(r, level).map(|_| ())
}

fn quant_vector_values(r: &mut ComponentReader<'_>, level: u32) -> Option<Option<[u64; 3]>> {
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

pub(crate) fn component(
    r: &mut ComponentReader<'_>,
    name: &str,
    level: u32,
    archetype: u32,
) -> Option<bool> {
    match name {
        "low-frequency" => {
            if !position::point(r, "position")? {
                return Some(false);
            }
            low_frequency_orientation(r, "orientation")?;
            r.r("value16", 16)?;
            r.r("value8", 8)?;
            r.r("value2", 2)?;
            let count = r.r("entries.count", 6)?;
            for i in 0..count {
                let name = format!("entries[{i}]");
                let flags = r.r(&format!("{name}.flags"), 3)?;
                if flags & 1 != 0 && !position::point(r, &format!("{name}.position"))? {
                    return Some(false);
                }
                if flags & 2 != 0 {
                    low_frequency_orientation(r, &format!("{name}.orientation"))?;
                }
                r.r(&format!("{name}.value16"), 16)?;
                r.r(&format!("{name}.value5"), 5)?;
            }
        }
        "game-engine-disabled-kill-volume-flags-component" => {
            let count = r.r("count", 13)?;
            r.r_wide("flags", count)?;
        }
        "GameEngineComposerLetterboxComponent" => {
            r.bit("flag")?;
            r.r("value", 16)?;
            for i in 0..4 {
                r.gate(&format!("indices[{i}]"), 7, false)?;
            }
            for i in 0..4 {
                r.gate(&format!("values[{i}]"), 16, true)?;
            }
        }
        "scenario-intro-component" => {
            r.r("value", 7)?;
            r.bit("flag")?;
        }
        "sound-placement-state-data-component" => {
            r.gate("value", 20, false)?;
            r.bit("flag")?;
        }
        "powerframe-player-selection-data-component" => {
            let count = r.r("count", 6)?;
            for i in 0..count {
                r.gate(&format!("entries[{i}].player"), 5, false)?;
                r.r(&format!("entries[{i}].bits"), 256)?;
            }
        }
        "supply-lines-blocked-status-component" => {
            r.r("state", 2)?;
        }
        "supply-lines-busy-state-component" => {
            let count = r.r("count_minus_one", 5)? + 1;
            r.r_wide("flags", count)?;
        }
        "matchflow-sequence-data-component" => {
            r.r("state", 4)?;
            r.words("values", 4, 32)?;
        }
        "matchflow-focus-data-component" => {
            r.r("value6", 6)?;
            r.r("value4", 4)?;
        }
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
            r.publish_component(FilmComponentObservation::Probe {
                archetype,
                component: ProbeComponent::ManagedObjectPropertyName,
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
            if !position::point(r, "position")? {
                return Some(false);
            }
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
            r.publish_component(FilmComponentObservation::Probe {
                archetype,
                component: ProbeComponent::SplashStatic,
                values: vec![value],
            });
        }
        "effect-state-data-component" => {
            r.gate("tag", 32, true)?;
            if r.bit("marker_a_present")? {
                r.handle("marker_a", 0)?;
            }
            r.r("marker_name", 32)?;
            if r.bit("marker_b_present")? {
                r.inline_optional_handle("marker_b", 0)?;
                r.r("marker_b.name", 32)?;
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
        "crew-marked-objects-component" => {
            r.inline_optional_handle("reference", 0)?;
        }
        "player-primary-respawn-object-component" => {
            r.inline_optional_handle("reference", 8)?;
        }
        "player-desired-respawn-seat-component" => {
            r.inline_optional_handle("reference", 3)?;
            r.r("seat", 7)?;
        }
        "nav-cutscene-flag-component" => {
            if r.bit("present")? {
                r.gate("index", 16, false)?;
                if !position::point(r, "position")? {
                    return Some(false);
                }
                r.words("values32", 2, 32)?;
                r.r("value16", 16)?;
                r.r("value8", 8)?;
                r.r("value2", 2)?;
                r.r("tail", 32)?;
            }
        }
        "player-desired-respawn-location-component" => {
            let field_start = r.fields.len();
            let present = r.bit("present")?;
            if present {
                if !position::point(r, "position")? {
                    return Some(false);
                }
                r.r("id", 19)?;
            }
            // Retain recorded gates/codes/words, not a fabricated precision level
            // or the former generic-vector interpretation. Runtime has no consumer
            // of the old quantized-vector diagnostic shape.
            let values = r.fields[field_start..]
                .iter()
                .map(|f| f.raw.low_u64())
                .collect();
            r.publish_component(FilmComponentObservation::PlayerState {
                field: PlayerStateField::DesiredRespawnLocation,
                values,
                present,
            });
        }

        "flock-position-component" => return position::point(r, "position"),
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
            if !position::point(r, "position")? {
                return Some(false);
            }
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

pub(crate) fn read_primary(
    r: &mut ComponentReader<'_>,
    name: &str,
    _level: u32,
    archetype: u32,
) -> Option<bool> {
    match name {
        "game-engine-screen-sequence-component" => {
            r.r("sequence", 4)?;
            r.r("value", 8)?;
        }
        "simulation-state" | "simulation-state-component" => return orientation::simulation(r),
        "crew-order-component" => {
            r.r("order", 3)?;
            if r.bit("vector_present")? && !position::point(r, "position")? {
                return Some(false);
            }
        }
        "music-variables-component" => {
            for i in 0..32 {
                if !r.bit(&format!("variables[{i}].absent"))? {
                    r.r(&format!("variables[{i}].kind"), 2)?;
                    r.words(&format!("variables[{i}].words"), 2, 32)?;
                }
            }
        }
        "branch-script-results-component" => {
            r.r("index", 6)?;
            r.r("value", 32)?;
        }
        "high-frequency" => match archetype {
            3 => {
                // FUN_142ed4880, shared fields of archetype 3's frequency state.
                r.r("value16", 16)?;
                r.r("value8", 8)?;
                r.r("value2", 2)?;
            }
            4 => {
                // FUN_14076d034 is a different descriptor with the same name.
                let value = r.r("counter", 8)?;
                r.publish_component(FilmComponentObservation::Probe {
                    archetype,
                    component: ProbeComponent::HighFrequency,
                    values: vec![value],
                });
            }
            _ => return Some(false),
        },
        "animated-mesh-dynamic-state-component" => {
            r.r("state", 8)?;
            r.bit("flag")?;
            r.r("value", 16)?;
        }
        "player-waypoint-component" => {
            r.r("waypoint", 3)?;
        }
        "player-unsafe-respawn-timer-component" => {
            r.r("timer", 10)?;
        }
        "player-respawn-safety-component" => {
            r.r("safety", 5)?;
        }
        "device-position-component" => {
            r.r("position", 14)?;
            r.bit("flag")?;
        }
        "game-engine-campaign-timer-component" => {
            r.words("timers", 2, 16)?;
            r.r("state", 5)?;
        }
        _ => return Some(false),
    }
    Some(true)
}

fn low_frequency_orientation(r: &mut ComponentReader<'_>, name: &str) -> Option<()> {
    r.gate(&format!("{name}.forward"), 19, false)?;
    r.r(&format!("{name}.roll"), 8)?;
    Some(())
}
