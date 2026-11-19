//! Named item values in the game-tag catalogue.

/// Define a named item once for both its Rust variant and game display name.
macro_rules! define_named_items {
    (
        $(#[$enum_doc:meta])*
        $name:ident {
            $(
                $(#[$variant_doc:meta])*
                $variant:ident => $game_name:literal;
            )*
        }
    ) => {
        $(#[$enum_doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        #[non_exhaustive]
        pub enum $name {
            $(
                $(#[$variant_doc])*
                $variant,
            )*
        }

        impl $name {
            /// The game's English name.
            pub fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $game_name,)*
                }
            }
        }
    };
}

define_named_items! {
    /// A weapon, by the name the game gives its tags. Vehicles' and turrets' guns are weapons too.
    Weapon {
        /// Auto Turret.
        AutoTurret => "Auto Turret";
        /// Bandit Evo.
        BanditEvo => "Bandit Evo";
        /// Banshee.
        Banshee => "Banshee";
        /// BR75 Battle Rifle.
        Br75BattleRifle => "BR75 Battle Rifle";
        /// Brute Chopper.
        BruteChopper => "Brute Chopper";
        /// Cindershot.
        Cindershot => "Cindershot";
        /// CQS48 Bulldog.
        Cqs48Bulldog => "CQS48 Bulldog";
        /// Disruptor.
        Disruptor => "Disruptor";
        /// Energy Sword.
        EnergySword => "Energy Sword";
        /// Fuel Rod SPNKr.
        FuelRodSpnkr => "Fuel Rod SPNKr";
        /// Fusion Coil Shock.
        FusionCoilShock => "Fusion Coil Shock";
        /// Generic Ball.
        GenericBall => "Generic Ball";
        /// Generic Skull.
        GenericSkull => "Generic Skull";
        /// Generic Skull, Escalation.
        GenericSkullEscalation => "Generic Skull, Escalation";
        /// Ghost.
        Ghost => "Ghost";
        /// Gravity Hammer.
        GravityHammer => "Gravity Hammer";
        /// Heatwave.
        Heatwave => "Heatwave";
        /// M247 HMG.
        M247Hmg => "M247 HMG";
        /// M392 Bandit.
        M392Bandit => "M392 Bandit";
        /// M41 Rocket Launcher.
        M41RocketLauncher => "M41 Rocket Launcher";
        /// MA40 Assault Rifle.
        Ma40AssaultRifle => "MA40 Assault Rifle";
        /// MA5K Avenger.
        Ma5kAvenger => "MA5K Avenger";
        /// Mangler.
        Mangler => "Mangler";
        /// Mk50 Sidekick.
        Mk50Sidekick => "Mk50 Sidekick";
        /// MLRS-2 Hydra.
        Mlrs2Hydra => "MLRS-2 Hydra";
        /// Mongoose.
        Mongoose => "Mongoose";
        /// Mutilator.
        Mutilator => "Mutilator";
        /// Mythic Sandwich.
        MythicSandwich => "Mythic Sandwich";
        /// Needler.
        Needler => "Needler";
        /// Normal Sandwich.
        NormalSandwich => "Normal Sandwich";
        /// Plasma Cannon.
        PlasmaCannon => "Plasma Cannon";
        /// Plasma Pistol.
        PlasmaPistol => "Plasma Pistol";
        /// Power Socket UNSC.
        PowerSocketUnsc => "Power Socket UNSC";
        /// Pulse Carbine.
        PulseCarbine => "Pulse Carbine";
        /// Ravager.
        Ravager => "Ravager";
        /// S7 Sniper Rifle.
        S7SniperRifle => "S7 Sniper Rifle";
        /// Scorpion.
        Scorpion => "Scorpion";
        /// Scrap Cannon.
        ScrapCannon => "Scrap Cannon";
        /// Sentinel Beam.
        SentinelBeam => "Sentinel Beam";
        /// Shade Turret.
        ShadeTurret => "Shade Turret";
        /// Shock Rifle.
        ShockRifle => "Shock Rifle";
        /// Skewer.
        Skewer => "Skewer";
        /// Stalker Rifle.
        StalkerRifle => "Stalker Rifle";
        /// Vestige Carbine.
        VestigeCarbine => "Vestige Carbine";
        /// VK78 Commando Rifle.
        Vk78CommandoRifle => "VK78 Commando Rifle";
        /// Wasp.
        Wasp => "Wasp";
        /// Wraith.
        Wraith => "Wraith";
    }
}

define_named_items! {
    /// A vehicle or a standalone turret.
    Vehicle {
        /// Auto Turret.
        AutoTurret => "Auto Turret";
        /// Banshee.
        Banshee => "Banshee";
        /// Brute Chopper.
        BruteChopper => "Brute Chopper";
        /// Falcon.
        Falcon => "Falcon";
        /// Ghost.
        Ghost => "Ghost";
        /// M247 HMG Turret.
        M247HmgTurret => "M247 HMG Turret";
        /// Mongoose.
        Mongoose => "Mongoose";
        /// Pelican.
        Pelican => "Pelican";
        /// Phantom.
        Phantom => "Phantom";
        /// Plasma Cannon Turret.
        PlasmaCannonTurret => "Plasma Cannon Turret";
        /// Scorpion.
        Scorpion => "Scorpion";
        /// Scrap Cannon Turret.
        ScrapCannonTurret => "Scrap Cannon Turret";
        /// Shade Turret.
        ShadeTurret => "Shade Turret";
        /// Warthog.
        Warthog => "Warthog";
        /// Wasp.
        Wasp => "Wasp";
        /// Wraith.
        Wraith => "Wraith";
    }
}

define_named_items! {
    /// A piece of equipment or a power-up.
    Equipment {
        /// Active Camo.
        ActiveCamo => "Active Camo";
        /// Drop Wall.
        DropWall => "Drop Wall";
        /// Grappleshot.
        Grappleshot => "Grappleshot";
        /// Overshield.
        Overshield => "Overshield";
        /// Quantum Translocator.
        QuantumTranslocator => "Quantum Translocator";
        /// Repulsor.
        Repulsor => "Repulsor";
        /// Shroud Screen.
        ShroudScreen => "Shroud Screen";
        /// Threat Sensor.
        ThreatSensor => "Threat Sensor";
        /// Thruster.
        Thruster => "Thruster";
    }
}
