//! Registry-name dispatch; family modules own the actual component grammars.
use super::reader::ComponentReader;
use super::{
    basic, biped, equipment, flock, m4b, managed, navpoint, object, physics, projectile, scene,
    tacmap, unit, weapon,
};
pub(crate) fn component(
    r: &mut ComponentReader<'_>,
    name: &str,
    level: u32,
    archetype: u32,
) -> Option<bool> {
    match name {
        "game-engine-screen-sequence-component"
        | "simulation-state"
        | "simulation-state-component"
        | "crew-order-component"
        | "music-variables-component"
        | "branch-script-results-component"
        | "high-frequency"
        | "animated-mesh-dynamic-state-component"
        | "player-waypoint-component"
        | "player-unsafe-respawn-timer-component"
        | "player-respawn-safety-component"
        | "device-position-component"
        | "game-engine-campaign-timer-component" => scene::read_primary(r, name, level, archetype),
        "object-position-component"
        | "object-forward-and-up-dynamic-precision-component"
        | "object-angular-velocity-dynamic-precision-component"
        | "object-position-dynamic-precision-component"
        | "object-translational-velocity-dynamic-precision-component"
        | "object-translational-velocity-component"
        | "object-angular-velocity-component"
        | "object-region-state-component"
        | "object-damage-sections-component"
        | "object-constraint-component"
        | "object-parent-state-component"
        | "object-scale-component"
        | "object-maximum-vitalities-component"
        | "object-dissolver-component"
        | "object-physics-flags-component"
        | "object-frame-configuration-component"
        | "object-multiplayer-properties-component" => {
            object::read_primary(r, name, level, archetype)
        }
        "tacmap-queuedreplaymission"
        | "tacmap-backmenu-openoverride"
        | "tacmap-cooptetherarea"
        | "tacmap-displayasset"
        | "tacmap-waypointstate"
        | "tacmap-iconlodthresholds"
        | "tacmap-mapscale"
        | "tacmap-settingstag"
        | "tacmap-cameraheading"
        | "tacmap-missioncount"
        | "tacmap-missionmarkerstate"
        | "tacmap-lockedlights"
        | "tacmap-dungeonstate" => tacmap::read_primary(r, name, level, archetype),
        "equipment-has-infinite-uses-component"
        | "equipment-deployed-component"
        | "equipment-energy-component"
        | "equipment-energy-delay-ticks-left-component"
        | "equipment-charges-remaining-component"
        | "equipment-creator-component"
        | "equipment-activated-component"
        | "equipment-tracked-object-handles-stack-component"
        | "equipment-command-tick-component"
        | "equipment-being-hacked-component"
        | "equipment-control-signal-component" => {
            equipment::read_primary(r, name, level, archetype)
        }
        "biped-control-context"
        | "biped-control-context-component"
        | "biped-emp-timer-component"
        | "biped-malleable-property"
        | "biped-malleable-property-component"
        | "biped-spartan-ability-malleable-property-component"
        | "biped-slide-component"
        | "biped-slide"
        | "biped-posture-physics-component"
        | "biped-mobility-action-component"
        | "biped-mobility-action"
        | "biped-spartan-ability-component"
        | "biped-spartan-ability-non-predicted-state-component"
        | "biped-spartan-ability-non-predicted-state"
        | "biped-desired-weapon-set" => biped::read_primary(r, name, level, archetype),
        "unit-control-component"
        | "unit-actor-control-component"
        | "unit-actor-state-component"
        | "unit-grenade-counts-component"
        | "unit-equipment-component"
        | "unit-low-frequency-component"
        | "unit-command-tick-component"
        | "unit-stun-component"
        | "unit-crouch-component"
        | "unit-desired-aiming-vector-component"
        | "unit-active-camo-state-component" => unit::read_primary(r, name, level, archetype),
        "flock-relevancy-component"
        | "flock-fleeing-component"
        | "flock-remembered-danger-component"
        | "flock-destination-component" => flock::read_primary(r, name, level, archetype),
        "projectile-at-rest-state"
        | "projectile-command_tick"
        | "projectile-tether-state"
        | "projectile-deceleration-disabled-state"
        | "item-at-rest-component"
        | "tacmap-fasttravelstate" => projectile::read_primary(r, name, level, archetype),
        "managed-player-team-designator-component"
        | "managed-object-networked-splash-message-dynamic-component"
        | "managed-object-property-component"
        | "managed-object-player-masked-property-component" => {
            managed::read_primary(r, name, level, archetype)
        }
        "generic-rigid-body-transforms"
        | "generic-rigid-body-transforms-component"
        | "physics-state-component" => physics::read_primary(r, name, level, archetype),
        "weapon-ammo-component"
        | "weapon-state-ammo"
        | "weapon-state-rounds-inventory"
        | "weapon-state-overheated" => weapon::read_primary(r, name, level, archetype),
        _ => {
            let mut result = basic::component(r, name);
            if result == Some(false) {
                result = object::component(r, name, archetype);
            }
            if result == Some(false) {
                result = scene::component(r, name, level, archetype);
            }
            if result == Some(false) {
                result = navpoint::component(r, name, level);
            }
            if result == Some(false) {
                result = m4b::component(r, name);
            }
            result
        }
    }
}
