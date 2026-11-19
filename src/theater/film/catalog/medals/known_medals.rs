//! Known native medal indices and their reference metadata.

use super::{MedalDifficulty, MedalKind};

/// Generate all medal lookups from one recorded index, name, and metadata entry.
macro_rules! define_known_medals {
    (
        $(
            $(#[$variant_doc:meta])*
            $variant:ident = $index:literal, $name:literal,
                $kind:ident, $difficulty:ident, $score:literal, $weight:literal;
        )*
    ) => {
        /// A medal. Its discriminant is the reference medal index, not a service NameId.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        #[repr(u8)]
        pub enum Medal {
            $(
                $(#[$variant_doc])*
                $variant = $index,
            )*
        }

        impl Medal {
            /// The medal with the game's index `index`.
            pub(super) fn lookup(index: u32) -> Option<Self> {
                match index {
                    $($index => Some(Self::$variant),)*
                    _ => None,
                }
            }

            /// The game's English name.
            pub fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)*
                }
            }

            pub(super) fn row(self) -> (MedalKind, MedalDifficulty, u32, u32) {
                match self {
                    $(Self::$variant => (MedalKind::$kind, MedalDifficulty::$difficulty, $score, $weight),)*
                }
            }
        }
    };
}

define_known_medals! {
    /// Double Kill.
    DoubleKill = 0, "Double Kill", Multikill, Heroic, 50, 100;
    /// Triple Kill.
    TripleKill = 1, "Triple Kill", Multikill, Legendary, 100, 150;
    /// Overkill.
    Overkill = 2, "Overkill", Multikill, Mythic, 150, 220;
    /// Killtacular.
    Killtacular = 3, "Killtacular", Multikill, Mythic, 200, 225;
    /// Killtrocity.
    Killtrocity = 4, "Killtrocity", Multikill, Mythic, 250, 230;
    /// Killamanjaro.
    Killamanjaro = 5, "Killamanjaro", Multikill, Mythic, 300, 235;
    /// Killtastrophe.
    Killtastrophe = 6, "Killtastrophe", Multikill, Mythic, 350, 240;
    /// Killpocalypse.
    Killpocalypse = 7, "Killpocalypse", Multikill, Mythic, 400, 245;
    /// Killionaire.
    Killionaire = 8, "Killionaire", Multikill, Mythic, 350, 250;
    /// Killing Spree.
    KillingSpree = 9, "Killing Spree", Spree, Heroic, 100, 100;
    /// Killing Frenzy.
    KillingFrenzy = 10, "Killing Frenzy", Spree, Legendary, 200, 150;
    /// Running Riot.
    RunningRiot = 11, "Running Riot", Spree, Mythic, 300, 205;
    /// Rampage.
    Rampage = 12, "Rampage", Spree, Mythic, 400, 210;
    /// Perfection.
    Perfection = 13, "Perfection", Mode, Mythic, 0, 200;
    /// Killjoy.
    Killjoy = 26, "Killjoy", Spree, Normal, 5, 50;
    /// Nightmare.
    Nightmare = 27, "Nightmare", Spree, Mythic, 500, 220;
    /// Boogeyman.
    Boogeyman = 28, "Boogeyman", Spree, Mythic, 600, 230;
    /// Grim Reaper.
    GrimReaper = 29, "Grim Reaper", Spree, Mythic, 700, 240;
    /// Demon.
    Demon = 30, "Demon", Spree, Mythic, 800, 250;
    /// Flawless Victory.
    FlawlessVictory = 31, "Flawless Victory", Mode, Legendary, 0, 150;
    /// Steaktacular.
    Steaktacular = 32, "Steaktacular", Mode, Legendary, 0, 150;
    /// Stopped Short.
    StoppedShort = 36, "Stopped Short", Mode, Normal, 0, 50;
    /// Flag Joust.
    FlagJoust = 37, "Flag Joust", Mode, Normal, 0, 50;
    /// Goal Line Stand.
    GoalLineStand = 38, "Goal Line Stand", Mode, Heroic, 0, 100;
    /// Necromancer.
    Necromancer = 39, "Necromancer", Mode, Legendary, 0, 150;
    /// Immortal.
    Immortal = 40, "Immortal", Mode, Legendary, 0, 150;
    /// Lone Wolf.
    LoneWolf = 41, "Lone Wolf", Mode, Heroic, 0, 100;
    /// Duelist.
    Duelist = 42, "Duelist", Mode, Normal, 0, 50;
    /// Ace.
    Ace = 43, "Ace", Mode, Legendary, 0, 150;
    /// Extermination.
    Extermination = 44, "Extermination", Mode, Mythic, 0, 200;
    /// Sole Survivor.
    SoleSurvivor = 45, "Sole Survivor", Mode, Normal, 0, 50;
    /// Untainted.
    Untainted = 46, "Untainted", Mode, Legendary, 0, 150;
    /// Blight.
    Blight = 47, "Blight", Mode, Heroic, 0, 100;
    /// Disease.
    Disease = 48, "Disease", Mode, Legendary, 0, 150;
    /// Plague.
    Plague = 49, "Plague", Mode, Mythic, 0, 205;
    /// Scourge.
    Scourge = 50, "Scourge", Mode, Mythic, 0, 220;
    /// Pestilence.
    Pestilence = 51, "Pestilence", Mode, Mythic, 0, 210;
    /// Apocalypse.
    Apocalypse = 52, "Apocalypse", Mode, Mythic, 0, 230;
    /// Culling.
    Culling = 53, "Culling", Mode, Heroic, 0, 100;
    /// Cleansing.
    Cleansing = 54, "Cleansing", Mode, Mythic, 0, 205;
    /// Purge.
    Purge = 55, "Purge", Mode, Legendary, 0, 150;
    /// Purification.
    Purification = 56, "Purification", Mode, Mythic, 0, 220;
    /// Divine Intervention.
    DivineIntervention = 57, "Divine Intervention", Mode, Mythic, 0, 230;
    /// Zombie Slayer.
    ZombieSlayer = 58, "Zombie Slayer", Mode, Heroic, 0, 100;
    /// Undead Hunter.
    UndeadHunter = 59, "Undead Hunter", Mode, Legendary, 0, 150;
    /// Hell's Janitor.
    HellsJanitor = 60, "Hell's Janitor", Mode, Mythic, 0, 200;
    /// The Sickness.
    TheSickness = 61, "The Sickness", Mode, Mythic, 0, 200;
    /// Spotter.
    Spotter = 62, "Spotter", Proficiency, Normal, 0, 50;
    /// Treasure Hunter.
    TreasureHunter = 63, "Treasure Hunter", Proficiency, Normal, 0, 50;
    /// Saboteur.
    Saboteur = 64, "Saboteur", Proficiency, Normal, 0, 50;
    /// Wingman.
    Wingman = 65, "Wingman", Proficiency, Normal, 0, 50;
    /// Wheelman.
    Wheelman = 66, "Wheelman", Proficiency, Normal, 0, 50;
    /// Gunner.
    Gunner = 67, "Gunner", Proficiency, Normal, 0, 50;
    /// Driver.
    Driver = 68, "Driver", Proficiency, Normal, 0, 50;
    /// Pilot.
    Pilot = 69, "Pilot", Proficiency, Normal, 0, 50;
    /// Tanker.
    Tanker = 70, "Tanker", Proficiency, Normal, 0, 50;
    /// Rifleman.
    Rifleman = 71, "Rifleman", Proficiency, Normal, 0, 50;
    /// Bomber.
    Bomber = 72, "Bomber", Proficiency, Normal, 0, 50;
    /// Grenadier.
    Grenadier = 73, "Grenadier", Proficiency, Normal, 0, 50;
    /// Boxer.
    Boxer = 74, "Boxer", Proficiency, Normal, 0, 50;
    /// Warrior.
    Warrior = 75, "Warrior", Proficiency, Normal, 0, 50;
    /// Gunslinger.
    Gunslinger = 76, "Gunslinger", Proficiency, Normal, 0, 50;
    /// Scattergunner.
    Scattergunner = 77, "Scattergunner", Proficiency, Normal, 0, 50;
    /// Sharpshooter.
    Sharpshooter = 78, "Sharpshooter", Proficiency, Normal, 0, 50;
    /// Marksman.
    Marksman = 79, "Marksman", Proficiency, Normal, 0, 50;
    /// Heavy.
    Heavy = 80, "Heavy", Proficiency, Normal, 0, 50;
    /// Bodyguard.
    Bodyguard = 81, "Bodyguard", Proficiency, Normal, 0, 50;
    /// Back Smack.
    BackSmack = 82, "Back Smack", Skill, Normal, 5, 50;
    /// Nuclear Football.
    NuclearFootball = 83, "Nuclear Football", Skill, Legendary, 0, 150;
    /// Boom Block.
    BoomBlock = 84, "Boom Block", Skill, Legendary, 0, 150;
    /// Bulltrue.
    Bulltrue = 85, "Bulltrue", Skill, Heroic, 5, 100;
    /// Cluster Luck.
    ClusterLuck = 86, "Cluster Luck", Skill, Heroic, 10, 100;
    /// Dogfight.
    Dogfight = 87, "Dogfight", Skill, Normal, 0, 50;
    /// Harpoon.
    Harpoon = 88, "Harpoon", Skill, Normal, 0, 50;
    /// Mind the Gap.
    MindTheGap = 89, "Mind the Gap", Skill, Heroic, 0, 100;
    /// Ninja.
    Ninja = 90, "Ninja", Skill, Mythic, 0, 200;
    /// Odin's Raven.
    OdinsRaven = 91, "Odin's Raven", Skill, Normal, 5, 50;
    /// Pancake.
    Pancake = 92, "Pancake", Skill, Heroic, 0, 100;
    /// Quigley.
    Quigley = 93, "Quigley", Skill, Mythic, 10, 200;
    /// Remote Detonation.
    RemoteDetonation = 94, "Remote Detonation", Skill, Mythic, 0, 200;
    /// Return to Sender.
    ReturnToSender = 95, "Return to Sender", Skill, Legendary, 0, 150;
    /// Rideshare.
    Rideshare = 96, "Rideshare", Skill, Heroic, 0, 100;
    /// Skyjack.
    Skyjack = 97, "Skyjack", Skill, Normal, 0, 50;
    /// Stick.
    Stick = 98, "Stick", Skill, Normal, 10, 50;
    /// Tag & Bag.
    TagAndBag = 99, "Tag & Bag", Skill, Heroic, 0, 100;
    /// Whiplash.
    Whiplash = 100, "Whiplash", Skill, Heroic, 0, 100;
    /// Kong.
    Kong = 101, "Kong", Skill, Normal, 0, 50;
    /// Autopilot Engaged.
    AutopilotEngaged = 102, "Autopilot Engaged", Skill, Legendary, 0, 150;
    /// Sneak King.
    SneakKing = 103, "Sneak King", Skill, Legendary, 0, 150;
    /// Windshield Wiper.
    WindshieldWiper = 104, "Windshield Wiper", Skill, Heroic, 5, 100;
    /// Reversal.
    Reversal = 105, "Reversal", Skill, Normal, 10, 50;
    /// Hail Mary.
    HailMary = 106, "Hail Mary", Skill, Heroic, 5, 100;
    /// Nade Shot.
    NadeShot = 107, "Nade Shot", Skill, Heroic, 0, 100;
    /// Snipe.
    Snipe = 108, "Snipe", Skill, Normal, 10, 50;
    /// Perfect.
    Perfect = 109, "Perfect", Skill, Heroic, 5, 100;
    /// Bank Shot.
    BankShot = 110, "Bank Shot", Skill, Heroic, 0, 100;
    /// Fire & Forget.
    FireAndForget = 111, "Fire & Forget", Skill, Heroic, 5, 100;
    /// Ballista.
    Ballista = 112, "Ballista", Skill, Heroic, 0, 100;
    /// Pull.
    Pull = 113, "Pull", Skill, Heroic, 0, 100;
    /// No Scope.
    NoScope = 114, "No Scope", Skill, Heroic, 10, 100;
    /// Achilles Spine.
    AchillesSpine = 115, "Achilles Spine", Skill, Legendary, 5, 150;
    /// Grand Slam.
    GrandSlam = 116, "Grand Slam", Skill, Legendary, 0, 150;
    /// Guardian Angel.
    GuardianAngel = 117, "Guardian Angel", Skill, Normal, 10, 50;
    /// Interlinked.
    Interlinked = 118, "Interlinked", Skill, Legendary, 0, 150;
    /// Death Race.
    DeathRace = 119, "Death Race", Skill, Heroic, 0, 100;
    /// Chain Reaction.
    ChainReaction = 120, "Chain Reaction", Skill, Normal, 0, 50;
    /// 360.
    ThreeSixty = 121, "360", Style, Legendary, 0, 150;
    /// Combat Evolved.
    CombatEvolved = 122, "Combat Evolved", Style, Legendary, 0, 150;
    /// Deadly Catch.
    DeadlyCatch = 123, "Deadly Catch", Style, Legendary, 0, 150;
    /// Driveby.
    Driveby = 124, "Driveby", Style, Legendary, 0, 150;
    /// Fastball.
    Fastball = 125, "Fastball", Style, Mythic, 5, 200;
    /// Flyin' High.
    FlyinHigh = 126, "Flyin' High", Style, Normal, 5, 50;
    /// From the Grave.
    FromTheGrave = 127, "From the Grave", Style, Normal, 10, 50;
    /// From the Void.
    FromTheVoid = 128, "From the Void", Style, Heroic, 0, 100;
    /// Grapple-jack.
    GrappleJack = 129, "Grapple-jack", Style, Heroic, 0, 100;
    /// Hold This.
    HoldThis = 130, "Hold This", Style, Heroic, 0, 100;
    /// Last Shot.
    LastShot = 131, "Last Shot", Style, Normal, 5, 50;
    /// Lawnmower.
    Lawnmower = 132, "Lawnmower", Style, Heroic, 0, 100;
    /// Mount Up.
    MountUp = 133, "Mount Up", Style, Normal, 0, 50;
    /// Off the Rack.
    OffTheRack = 134, "Off the Rack", Style, Heroic, 0, 100;
    /// Quick Draw.
    QuickDraw = 135, "Quick Draw", Style, Normal, 0, 50;
    /// Party's Over.
    PartysOver = 136, "Party's Over", Style, Heroic, 0, 100;
    /// Pineapple Express.
    PineappleExpress = 137, "Pineapple Express", Style, Heroic, 0, 100;
    /// Ramming Speed.
    RammingSpeed = 138, "Ramming Speed", Style, Heroic, 0, 100;
    /// Reclaimer.
    Reclaimer = 139, "Reclaimer", Style, Normal, 0, 50;
    /// Shot Caller.
    ShotCaller = 140, "Shot Caller", Style, Heroic, 0, 100;
    /// Yard Sale.
    YardSale = 141, "Yard Sale", Style, Heroic, 0, 100;
    /// Special Delivery.
    SpecialDelivery = 142, "Special Delivery", Style, Normal, 0, 50;
    /// Street Sweeper.
    StreetSweeper = 143, "Street Sweeper", Style, Legendary, 0, 150;
    /// Fumble.
    Fumble = 146, "Fumble", Mode, Heroic, 0, 100;
    /// Straight Balling.
    StraightBalling = 148, "Straight Balling", Mode, Legendary, 0, 150;
    /// Big Deal.
    BigDeal = 150, "Big Deal", Mode, Heroic, 0, 100;
    /// Always Rotating.
    AlwaysRotating = 151, "Always Rotating", Mode, Normal, 0, 50;
    /// Hill Guardian.
    HillGuardian = 152, "Hill Guardian", Mode, Heroic, 0, 100;
    /// Clock Stop.
    ClockStop = 153, "Clock Stop", Mode, Heroic, 0, 100;
    /// Secure Line.
    SecureLine = 154, "Secure Line", Mode, Heroic, 0, 101;
    /// Signal Block.
    SignalBlock = 155, "Signal Block", Mode, Heroic, 0, 101;
    /// Splatter.
    Splatter = 156, "Splatter", Skill, Normal, 5, 50;
    /// Clash of Kings.
    ClashOfKings = 157, "Clash of Kings", Mode, Normal, 0, 50;
    /// Contract Killer.
    ContractKiller = 158, "Contract Killer", Mode, Legendary, 0, 150;
    /// Watch the Throne.
    WatchTheThrone = 160, "Watch the Throne", Mode, Normal, 0, 50;
    /// All That Juice.
    AllThatJuice = 162, "All That Juice", Mode, Legendary, 0, 150;
    /// Great Journey.
    GreatJourney = 163, "Great Journey", Mode, Heroic, 0, 100;
    /// Power Outage.
    PowerOutage = 164, "Power Outage", Mode, Legendary, 0, 150;
    /// Breacher.
    Breacher = 165, "Breacher", Proficiency, Normal, 0, 50;
    /// Mounted & Loaded.
    MountedAndLoaded = 166, "Mounted & Loaded", Style, Heroic, 0, 100;
    /// Monopoly.
    Monopoly = 167, "Monopoly", Mode, Heroic, 0, 100;
    /// Counter-snipe.
    CounterSnipe = 168, "Counter-snipe", Skill, Heroic, 0, 100;
    /// Remember This.
    RememberThis = 172, "Remember This", Style, Normal, 0, 50;
    /// Hang Time.
    HangTime = 173, "Hang Time", Skill, Legendary, 0, 150;
    /// Driving Spree.
    DrivingSpree = 174, "Driving Spree", Spree, Heroic, 0, 100;
    /// Death Cabbie.
    DeathCabbie = 175, "Death Cabbie", Spree, Legendary, 0, 150;
    /// Immortal Chauffeur.
    ImmortalChauffeur = 176, "Immortal Chauffeur", Spree, Mythic, 0, 200;
    /// Blind Fire.
    BlindFire = 177, "Blind Fire", Style, Legendary, 0, 150;
    /// Hang Up.
    HangUp = 178, "Hang Up", Mode, Normal, 0, 51;
    /// Call Blocked.
    CallBlocked = 179, "Call Blocked", Mode, Normal, 0, 52;
    /// Clear Reception.
    ClearReception = 180, "Clear Reception", Mode, Heroic, 0, 101;
}
