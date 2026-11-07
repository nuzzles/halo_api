//! Known Halo Infinite filmshell weapon IDs and names.
//!
//! Source: LevelUp `games/weapons/filmshell/catalogue.go` and `unarmed.go`
//! at commit 413971bcc485be53a2fa8cf229e170539a5146f8. Three entries
//! explicitly unverified there (frag, plasma, dynamo grenades) are omitted.
//! The Bandit EVO label follows the CSV spelling; LevelUp writes "Bandit Evo".

use super::{WeaponDefinition, WeaponId};

macro_rules! define_known_weapons {
    ($($variant:ident => $constant:ident = $id:expr, $name:literal;)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum KnownWeapon { $($variant,)* }

        $(pub const $constant: WeaponDefinition = WeaponDefinition {
            weapon: KnownWeapon::$variant, id: WeaponId($id), name: $name,
        };)*

        pub const ALL: &[WeaponDefinition] = &[$($constant,)*];

        impl KnownWeapon {
            pub const fn definition(self) -> WeaponDefinition {
                match self { $(Self::$variant => $constant,)* }
            }
        }
    };
}

define_known_weapons! {
    BanditEvo => BANDIT_EVO = 0x6acdc44d42c9679f, "Bandit EVO";
    Br75 => BR75 = 0x2b1824d542c9679f, "BR75";
    Cindershot => CINDERSHOT = 0x230447b142c9679f, "Cindershot";
    Cqs48Bulldog => CQS48_BULLDOG = 0xb619d84a42c9679f, "CQS48 Bulldog";
    Disruptor => DISRUPTOR = 0x84bd29ed42c9679f, "Disruptor";
    FuelRodSpnkr => FUEL_ROD_SPNKR = 0x9d6aaed242c9679f, "Fuel Rod SPNKr";
    GravityHammer => GRAVITY_HAMMER = 0x841ac5e542c9679f, "Gravity Hammer";
    Heatwave => HEATWAVE = 0x2ac9c2ff42c9679f, "Heatwave";
    M41Spnkr => M41_SPNKR = 0x71ab0a2c42c9679f, "M41 SPNKr";
    M392Bandit => M392_BANDIT = 0x2fb21c8742c9679f, "M392 Bandit";
    Ma40Ar => MA40_AR = 0x48c19d2d42c9679f, "MA40 AR";
    Ma5kAvenger => MA5K_AVENGER = 0xf5c335dfe7232c0f, "MA5K Avenger";
    Mangler => MANGLER = 0x80977ba542c9679f, "Mangler";
    Mlrs2Hydra => MLRS_2_HYDRA = 0x767db96d42c9679f, "MLRS-2 Hydra";
    Mk51Sidekick => MK51_SIDEKICK = 0xf408190f42c9679f, "Mk51 Sidekick";
    Mutilator => MUTILATOR = 0xd791556542c9679f, "Mutilator";
    MythicSandwich => MYTHIC_SANDWICH = 0xb7262ca1c8fb11d0, "Mythic Sandwich";
    Needler => NEEDLER = 0xb533957e42c9679f, "Needler";
    PlasmaPistol => PLASMA_PISTOL = 0xc354294642c9679f, "Plasma Pistol";
    PulseCarbine => PULSE_CARBINE = 0x30484ea642c9679f, "Pulse Carbine";
    Ravager => RAVAGER = 0xc30d87c742c9679f, "Ravager";
    S7Sniper => S7_SNIPER = 0x0a1992bc42c9679f, "S7 Sniper";
    Sandwich => SANDWICH = 0x880fe0bc42c9679f, "Sandwich";
    SentinelBeam => SENTINEL_BEAM = 0xa0955e9e42c9679f, "Sentinel Beam";
    ShockRifle => SHOCK_RIFLE = 0x9387a8b942c9679f, "Shock Rifle";
    ShockRifleRanked => SHOCK_RIFLE_RANKED = 0x1a22fee642c9679f, "Shock Rifle (Ranked)";
    Skewer => SKEWER = 0x0d20c46942c9679f, "Skewer";
    StalkerRifle => STALKER_RIFLE = 0xdaf193c742c9679f, "Stalker Rifle";
    VestigeCarbine => VESTIGE_CARBINE = 0x3e07021742c9679f, "Vestige Carbine";
    Vk78Commando => VK78_COMMANDO = 0xfd98554c42c9679f, "VK78 Commando";
    EnergySword => ENERGY_SWORD = 0x4ff3937e42c9679f, "Energy Sword";
    DuelistEnergySword => DUELIST_ENERGY_SWORD = 0x4ff3937e8978aa7a, "Duelist Energy Sword";
    EliteBloodblade => ELITE_BLOODBLADE = 0x4ff3937e1ec48c7a, "Elite Bloodblade";
    InfectedEnergySword => INFECTED_ENERGY_SWORD = 0x0c55765f7a9376a0, "Infected Energy Sword";
    DiminisherOfHope => DIMINISHER_OF_HOPE = 0x841ac5e5a730e49f, "Diminisher of Hope";
    RushdownHammer => RUSHDOWN_HAMMER = 0x841ac5e5d8d07ca1, "Rushdown Hammer";
    Unarmed => UNARMED = 0x00007ca942c9679f, "Unarmed";
}
