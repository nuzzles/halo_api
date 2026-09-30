//! Player, game-engine, objective, and navpoint component primitives.
use super::reader::ComponentReader;
use crate::theater::parser::v41::reference::observations::{
    FilmComponentObservation, GameEngineField, ManagedObjectField, NavpointField, ObjectiveField,
    PlayerStateField,
};

pub(crate) fn component(r: &mut ComponentReader<'_>, name: &str) -> Option<bool> {
    let field_start = r.fields.len();
    match name {
        "player-fade-properties-component" => r.words("fade", 6, 12)?,
        "player-soft-kill-timer-component" => r.words("timers", 3, 5)?,
        "player-target-tracking-detection-component" => r.words("flags", 2, 1)?,
        "player-desired-respawn-player-component" => {
            r.r("player", 16)?;
        }
        "player-engine-loadout-component" => r.words("loadout", 8, 8)?,
        "player-lives-remaining-component" => {
            r.r("lives", 7)?;
        }
        "player-last-betrayer-component" => {
            r.r("player", 6)?;
        }
        "player-control-aiming-component" => {
            r.r("direction", 19)?;
        }
        "player-active-in-game-component"
        | "player-pending-join-in-progress-spawn-component"
        | "player-allowed-to-quit-component" => {
            r.bit("value")?;
        }
        "player-malleable-properties-simulation-component" => {
            let mut values = Vec::with_capacity(24);
            for i in 0..3 {
                values.push(r.r(&format!("head_flags[{i}]"), 1)?);
            }
            for i in 0..6 {
                let value = r.gated_value(&format!("properties[{i}]"), 12, true)?;
                values.extend([u64::from(value.is_some()), value.unwrap_or(0)]);
            }
            for i in 0..9 {
                values.push(r.r(&format!("tail_flags[{i}]"), 1)?);
            }
            r.publish_component(FilmComponentObservation::PlayerState {
                field: PlayerStateField::MalleableProperties,
                values,
                present: true,
            });
        }
        "player-respawn-timer-component" => {
            r.bit("active")?;
            r.words("timers", 2, 10)?;
        }
        "game-engine-current-state-component" => {
            r.r("state", 3)?;
        }
        "game-engine-current-round-component" => r.gate("round", 5, false)?,
        "game-engine-sudden-death-time-left-component"
        | "game-engine-grace-period-time-left-component"
        | "game-engine-round-timer-component" => {
            r.words("timers", 2, 16)?;
            r.r("state", 5)?;
        }
        "game-engine-round-condition-flags-component" => {
            r.r("flags", 10)?;
        }
        "managed-object-boundary-visibility-component" => r.words("flags", 32, 1)?,
        "managed-object-boundary-color-component" | "managed-objective-color-component" => {
            for name in ["red", "green", "blue", "alpha"] {
                r.r(name, 8)?;
            }
        }
        "managed-object-rtpc-component" => {
            if r.r("id", 32)? != 0 {
                r.r("value", 22)?;
            }
        }
        "managed-navpoint-radial-progress" => {
            r.r("progress", 8)?;
        }
        "managed-navpoint-sub-type-component" => {
            r.r("sub_type", 32)?;
        }
        "managed-navpoint-docking-order-component" => {
            r.r("order", 8)?;
        }
        "managed-navpoint-docking-group-name-component" => {
            r.r("group", 32)?;
        }
        "managed-navpoint-manual-timer-initial-duration-component"
        | "managed-navpoint-manual-timer-current-duration-component" => {
            r.r("duration", 17)?;
        }
        "managed-objective-timers-component" | "managed-navpoint-timers-component" => {
            r.words("timers", 2, 7)?
        }
        "managed-objective-object-reference-component"
        | "managed-objective-type-component"
        | "managed-objective-progress-component"
        | "managed-objective-required-progress-component"
        | "managed-objective-parent-objective-component"
        | "managed-objective-sub-objective-entities-component" => {
            r.r("value", 32)?;
        }
        "managed-objective-enabled-component"
        | "managed-objective-is-new-and-unseen-component"
        | "managed-objective-is-only-one-item-unlocked-component"
        | "managed-objective-forced-update-component" => {
            r.bit("value")?;
        }
        "managed-objective-priority-component"
        | "managed-objective-outro-phase-duration-component" => {
            r.r("value", 8)?;
        }
        "managed-objective-message-type-component" => {
            r.r("type", 4)?;
        }
        "managed-objective-state-component" => {
            r.r("state", 3)?;
        }
        "managed-objective-formatted-text-component"
        | "managed-objective-secondary-formatted-text-component" => formatted_text(r)?,
        "object-body-vitality-component" => {
            r.r("health", 8)?;
            r.words("flags", 3, 1)?;
        }
        "object-shield-vitality-component" => {
            r.r("shield", 8)?;
            if r.bit("regeneration")? {
                r.gate("regeneration_a", 12, true)?;
                r.gate("regeneration_b", 12, true)?;
            }
            r.r("block64", 16)?;
            r.words("flags", 4, 1)?;
        }
        _ => return Some(false),
    }
    // Only these reference hooks publish here. Other decoded fields intentionally
    // remain raw fields without a fabricated hook emission.
    let values = || {
        r.fields[field_start..]
            .iter()
            .map(|f| f.raw.low_u64())
            .collect::<Vec<_>>()
    };
    let publication = match name {
        "player-soft-kill-timer-component" => Some(FilmComponentObservation::PlayerState {
            field: PlayerStateField::SoftKill,
            values: values(),
            present: true,
        }),
        "player-target-tracking-detection-component" => {
            Some(FilmComponentObservation::PlayerState {
                field: PlayerStateField::TargetTracking,
                values: values(),
                present: true,
            })
        }
        "player-desired-respawn-player-component" => Some(FilmComponentObservation::PlayerState {
            field: PlayerStateField::DesiredRespawnPlayer,
            values: values(),
            present: true,
        }),
        "player-engine-loadout-component" => Some(FilmComponentObservation::PlayerState {
            field: PlayerStateField::Loadout,
            values: values(),
            present: true,
        }),
        "player-lives-remaining-component" => Some(FilmComponentObservation::PlayerState {
            field: PlayerStateField::Lives,
            values: values(),
            present: true,
        }),
        "player-last-betrayer-component" => Some(FilmComponentObservation::PlayerState {
            field: PlayerStateField::LastBetrayer,
            values: values(),
            present: true,
        }),
        "player-control-aiming-component" => Some(FilmComponentObservation::PlayerState {
            field: PlayerStateField::ControlAiming,
            values: values(),
            present: true,
        }),
        "player-active-in-game-component" => Some(FilmComponentObservation::PlayerState {
            field: PlayerStateField::ActiveInGame,
            values: values(),
            present: true,
        }),
        "player-pending-join-in-progress-spawn-component" => {
            Some(FilmComponentObservation::PlayerState {
                field: PlayerStateField::PendingJoinInProgress,
                values: values(),
                present: true,
            })
        }
        "game-engine-current-state-component" => Some(FilmComponentObservation::GameEngine {
            field: GameEngineField::State,
            values: values(),
            present: true,
        }),
        "game-engine-current-round-component" => Some(FilmComponentObservation::GameEngine {
            field: GameEngineField::Round,
            values: r.fields[field_start + 1..]
                .iter()
                .map(|f| f.raw.low_u64())
                .collect(),
            present: r.fields[field_start].raw.low_u64() == 0,
        }),
        "game-engine-sudden-death-time-left-component" => {
            Some(FilmComponentObservation::GameEngine {
                field: GameEngineField::SuddenDeath,
                values: values(),
                present: true,
            })
        }
        "game-engine-grace-period-time-left-component" => {
            Some(FilmComponentObservation::GameEngine {
                field: GameEngineField::GracePeriod,
                values: values(),
                present: true,
            })
        }
        "game-engine-round-condition-flags-component" => {
            Some(FilmComponentObservation::GameEngine {
                field: GameEngineField::RoundConditions,
                values: values(),
                present: true,
            })
        }
        "managed-object-boundary-visibility-component" => {
            Some(FilmComponentObservation::ManagedObject {
                field: ManagedObjectField::BoundaryVisibility,
                values: vec![
                    r.fields[field_start..]
                        .iter()
                        .enumerate()
                        .fold(0u64, |mask, (i, f)| mask | (f.raw.low_u64() << i)),
                ],
            })
        }
        "managed-object-boundary-color-component" => {
            Some(FilmComponentObservation::ManagedObject {
                field: ManagedObjectField::BoundaryColor,
                values: values(),
            })
        }
        "managed-object-rtpc-component" => Some(FilmComponentObservation::ManagedObject {
            field: ManagedObjectField::Rtpc,
            values: values(),
        }),
        "managed-navpoint-radial-progress" => Some(FilmComponentObservation::Navpoint {
            field: NavpointField::RadialProgress,
            values: values(),
        }),
        "managed-navpoint-manual-timer-initial-duration-component" => {
            Some(FilmComponentObservation::Navpoint {
                field: NavpointField::ManualTimerInitial,
                values: values(),
            })
        }
        "managed-navpoint-manual-timer-current-duration-component" => {
            Some(FilmComponentObservation::Navpoint {
                field: NavpointField::ManualTimerCurrent,
                values: values(),
            })
        }
        "managed-objective-timers-component" | "managed-navpoint-timers-component" => {
            Some(FilmComponentObservation::Objective {
                field: ObjectiveField::Timers,
                values: values(),
            })
        }
        "managed-objective-object-reference-component" => {
            Some(FilmComponentObservation::Objective {
                field: ObjectiveField::ObjectReference,
                values: values(),
            })
        }
        "managed-objective-type-component" => Some(FilmComponentObservation::Objective {
            field: ObjectiveField::Type,
            values: values(),
        }),
        "managed-objective-progress-component" => Some(FilmComponentObservation::Objective {
            field: ObjectiveField::Progress,
            values: values(),
        }),
        "managed-objective-required-progress-component" => {
            Some(FilmComponentObservation::Objective {
                field: ObjectiveField::RequiredProgress,
                values: values(),
            })
        }
        "managed-objective-state-component" => Some(FilmComponentObservation::Objective {
            field: ObjectiveField::State,
            values: values(),
        }),
        _ => None,
    };
    if let Some(publication) = publication {
        r.publish_component(publication);
    }
    Some(true)
}

pub(crate) fn formatted_text(r: &mut ComponentReader<'_>) -> Option<()> {
    if !r.bit("present")? {
        return Some(());
    }
    r.r("reference", 32)?;
    let count = r.r("count", 3)?;
    for i in 0..count {
        let tag = r.r(&format!("args[{i}].tag"), 3)?;
        match tag {
            0 => {}
            1 => r.gate(&format!("args[{i}].player"), 5, false)?,
            2 => {
                let short = r.bit(&format!("args[{i}].short"))?;
                r.r(&format!("args[{i}].value"), if short { 24 } else { 32 })?;
            }
            _ => {
                r.r(&format!("args[{i}].value"), 32)?;
            }
        }
    }
    Some(())
}
