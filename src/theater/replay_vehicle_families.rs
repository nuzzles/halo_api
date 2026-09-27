//! Pinned LevelUp vehicle chassis catalog (vehicle_families.go).
pub fn replay_vehicle_family(chassis: u32) -> &'static str {
    match chassis {
        0x00002705 => "warthog",
        0x000025aa => "mongoose",
        0x0000d3db => "scorpion",
        0xb65b3b4a => "wasp",
        0x0000d3dc => "ghost",
        0x000026ed => "banshee",
        0x00002706 => "wraith",
        0x002ba902 => "chopper",
        0x000026f2 => "phantom",
        0x000026f0 => "pelican",
        0x86799cb6 => "skiff",
        0x000df0c4 => "shade",
        0x0000254b => "falcon",
        0x5b80c406 => "ghost",
        0xc6e79dcc => "banshee",
        0xaf31ab1a => "mongoose",
        0xde26e3d7 => "mongoose",
        0x3d4a8a5a => "chopper",
        0xfe32c0f4 => "warthog",
        0xcb96ca07 => "warthog",
        0x038df01a => "tourelle_auto_bannie",
        0xae845375 => "wraith",
        0xf6f54e56 => "scorpion",
        0x9af9e693 => "ghost",
        0x0001530a => "banshee",
        0x5159c8ef => "warthog",
        0x75312e51 => "warthog",
        0x7617ff6e => "warthog",
        0x10754375 => "wraith",
        _ => "",
    }
}
/// Unknown families remain eligible; only explicitly non-pilotable families refuse rides.
pub fn replay_vehicle_family_is_rideable(family: &str) -> bool {
    !matches!(
        family,
        "falcon" | "pelican" | "phantom" | "skiff" | "tourelle_auto_bannie"
    )
}
