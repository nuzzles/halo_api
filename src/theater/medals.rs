//! Film medal IDs are distinct from the stats API NameId namespace.
//! Mapping: SPNKr medal_codes.json; see experiments/FILM_EVENTS.md for provenance.

use crate::clients::hi::models::{Medal, MedalMetadata};
use serde::{Deserialize, Serialize};

/// A known film-code mapping. Metadata fields are absent for medals missing from the CMS snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilmMedalDefinition {
    pub film_id: u8,
    pub name: &'static str,
    pub name_id: Option<u32>,
    pub sorting_weight: Option<u16>,
}

/// All 155 published film codes, including all 151 medals in the checked CMS catalog.
pub const FILM_MEDAL_DEFINITIONS: &[FilmMedalDefinition] = &[
    FilmMedalDefinition {
        film_id: 0,
        name: "Double Kill",
        name_id: Some(622331684),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 1,
        name: "Triple Kill",
        name_id: Some(2063152177),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 2,
        name: "Overkill",
        name_id: Some(835814121),
        sorting_weight: Some(220),
    },
    FilmMedalDefinition {
        film_id: 3,
        name: "Killtacular",
        name_id: Some(2137071619),
        sorting_weight: Some(225),
    },
    FilmMedalDefinition {
        film_id: 4,
        name: "Killtrocity",
        name_id: Some(1430343434),
        sorting_weight: Some(230),
    },
    FilmMedalDefinition {
        film_id: 5,
        name: "Killamanjaro",
        name_id: Some(3835606176),
        sorting_weight: Some(235),
    },
    FilmMedalDefinition {
        film_id: 6,
        name: "Killtastrophe",
        name_id: Some(2242633421),
        sorting_weight: Some(240),
    },
    FilmMedalDefinition {
        film_id: 7,
        name: "Killpocalypse",
        name_id: Some(3352648716),
        sorting_weight: Some(245),
    },
    FilmMedalDefinition {
        film_id: 8,
        name: "Killionaire",
        name_id: Some(3233051772),
        sorting_weight: Some(250),
    },
    FilmMedalDefinition {
        film_id: 9,
        name: "Killing Spree",
        name_id: Some(2780740615),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 10,
        name: "Killing Frenzy",
        name_id: Some(4261842076),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 11,
        name: "Running Riot",
        name_id: Some(418532952),
        sorting_weight: Some(205),
    },
    FilmMedalDefinition {
        film_id: 12,
        name: "Rampage",
        name_id: Some(1486797009),
        sorting_weight: Some(210),
    },
    FilmMedalDefinition {
        film_id: 13,
        name: "Perfection",
        name_id: Some(865763896),
        sorting_weight: Some(200),
    },
    FilmMedalDefinition {
        film_id: 26,
        name: "Killjoy",
        name_id: Some(3233952928),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 27,
        name: "Nightmare",
        name_id: Some(710323196),
        sorting_weight: Some(220),
    },
    FilmMedalDefinition {
        film_id: 28,
        name: "Boogeyman",
        name_id: Some(1720896992),
        sorting_weight: Some(230),
    },
    FilmMedalDefinition {
        film_id: 29,
        name: "Grim Reaper",
        name_id: Some(2567026752),
        sorting_weight: Some(240),
    },
    FilmMedalDefinition {
        film_id: 30,
        name: "Demon",
        name_id: Some(2875941471),
        sorting_weight: Some(250),
    },
    FilmMedalDefinition {
        film_id: 31,
        name: "Flawless Victory",
        name_id: Some(1680000231),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 32,
        name: "Steaktacular",
        name_id: Some(1169390319),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 36,
        name: "Stopped Short",
        name_id: Some(3488248720),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 37,
        name: "Flag Joust",
        name_id: Some(976049027),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 38,
        name: "Goal Line Stand",
        name_id: Some(3227840152),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 39,
        name: "Necromancer",
        name_id: Some(3011158621),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 40,
        name: "Immortal",
        name_id: Some(3120600565),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 41,
        name: "Lone Wolf",
        name_id: Some(2623698509),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 42,
        name: "Duelist",
        name_id: Some(4247875860),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 43,
        name: "Ace",
        name_id: Some(521420212),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 44,
        name: "Extermination",
        name_id: Some(4100966367),
        sorting_weight: Some(200),
    },
    FilmMedalDefinition {
        film_id: 45,
        name: "Sole Survivor",
        name_id: Some(2717755703),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 46,
        name: "Untainted",
        name_id: Some(1064731598),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 47,
        name: "Blight",
        name_id: Some(88914608),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 48,
        name: "Disease",
        name_id: Some(1155542859),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 49,
        name: "Plague",
        name_id: Some(3786134933),
        sorting_weight: Some(205),
    },
    FilmMedalDefinition {
        film_id: 50,
        name: "Scourge",
        name_id: Some(3520382976),
        sorting_weight: Some(220),
    },
    FilmMedalDefinition {
        film_id: 51,
        name: "Pestilence",
        name_id: Some(1719203329),
        sorting_weight: Some(210),
    },
    FilmMedalDefinition {
        film_id: 52,
        name: "Apocalypse",
        name_id: Some(3653884673),
        sorting_weight: Some(230),
    },
    FilmMedalDefinition {
        film_id: 53,
        name: "Culling",
        name_id: Some(1025827095),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 54,
        name: "Cleansing",
        name_id: Some(1765213446),
        sorting_weight: Some(205),
    },
    FilmMedalDefinition {
        film_id: 55,
        name: "Purge",
        name_id: Some(3467301935),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 56,
        name: "Purification",
        name_id: Some(496411737),
        sorting_weight: Some(220),
    },
    FilmMedalDefinition {
        film_id: 57,
        name: "Divine Intervention",
        name_id: Some(2164872967),
        sorting_weight: Some(230),
    },
    FilmMedalDefinition {
        film_id: 58,
        name: "Zombie Slayer",
        name_id: Some(557309779),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 59,
        name: "Undead Hunter",
        name_id: Some(1447057920),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 60,
        name: "Hell's Janitor",
        name_id: Some(217730222),
        sorting_weight: Some(200),
    },
    FilmMedalDefinition {
        film_id: 61,
        name: "The Sickness",
        name_id: Some(17866865),
        sorting_weight: Some(200),
    },
    FilmMedalDefinition {
        film_id: 62,
        name: "Spotter",
        name_id: Some(2477555653),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 63,
        name: "Treasure Hunter",
        name_id: Some(1685043466),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 64,
        name: "Saboteur",
        name_id: Some(20397755),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 65,
        name: "Wingman",
        name_id: Some(1284032216),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 66,
        name: "Wheelman",
        name_id: Some(2926348688),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 67,
        name: "Gunner",
        name_id: Some(3783455472),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 68,
        name: "Driver",
        name_id: Some(3027762381),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 69,
        name: "Pilot",
        name_id: Some(2593226288),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 70,
        name: "Tanker",
        name_id: Some(2278023431),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 71,
        name: "Rifleman",
        name_id: Some(2852571933),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 72,
        name: "Bomber",
        name_id: Some(1146876011),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 73,
        name: "Grenadier",
        name_id: Some(2648272972),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 74,
        name: "Boxer",
        name_id: Some(269174970),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 75,
        name: "Warrior",
        name_id: Some(1210678802),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 76,
        name: "Gunslinger",
        name_id: Some(1172766553),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 77,
        name: "Scattergunner",
        name_id: Some(3347922939),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 78,
        name: "Sharpshooter",
        name_id: Some(4277328263),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 79,
        name: "Marksman",
        name_id: Some(2758320809),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 80,
        name: "Heavy",
        name_id: Some(4086138034),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 81,
        name: "Bodyguard",
        name_id: Some(555849395),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 82,
        name: "Back Smack",
        name_id: Some(548533137),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 83,
        name: "Nuclear Football",
        name_id: Some(2253222811),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 84,
        name: "Boom Block",
        name_id: Some(524758914),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 85,
        name: "Bulltrue",
        name_id: Some(3114137341),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 86,
        name: "Cluster Luck",
        name_id: Some(3905838030),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 87,
        name: "Dogfight",
        name_id: Some(1229018603),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 88,
        name: "Harpoon",
        name_id: Some(2418616582),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 89,
        name: "Mind the Gap",
        name_id: Some(1880789493),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 90,
        name: "Ninja",
        name_id: Some(3085856613),
        sorting_weight: Some(200),
    },
    FilmMedalDefinition {
        film_id: 91,
        name: "Odin's Raven",
        name_id: Some(87172902),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 92,
        name: "Pancake",
        name_id: Some(3876426273),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 93,
        name: "Quigley",
        name_id: Some(1312042926),
        sorting_weight: Some(200),
    },
    FilmMedalDefinition {
        film_id: 94,
        name: "Remote Detonation",
        name_id: Some(3160646854),
        sorting_weight: Some(200),
    },
    FilmMedalDefinition {
        film_id: 95,
        name: "Return to Sender",
        name_id: Some(3059799290),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 96,
        name: "Rideshare",
        name_id: Some(656245292),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 97,
        name: "Skyjack",
        name_id: Some(731054446),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 98,
        name: "Stick",
        name_id: Some(3655682764),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 99,
        name: "Tag & Bag",
        name_id: Some(1841872491),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 100,
        name: "Whiplash",
        name_id: Some(1734214473),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 101,
        name: "Kong",
        name_id: Some(3546244406),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 102,
        name: "Autopilot Engaged",
        name_id: Some(1623236079),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 103,
        name: "Sneak King",
        name_id: Some(670606868),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 104,
        name: "Windshield Wiper",
        name_id: Some(2827657131),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 105,
        name: "Reversal",
        name_id: Some(2123530881),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 106,
        name: "Hail Mary",
        name_id: Some(3934547153),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 107,
        name: "Nade Shot",
        name_id: Some(265478668),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 108,
        name: "Snipe",
        name_id: Some(4229934157),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 109,
        name: "Perfect",
        name_id: Some(1512363953),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 110,
        name: "Bank Shot",
        name_id: Some(2414983178),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 111,
        name: "Fire & Forget",
        name_id: Some(988255960),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 112,
        name: "Ballista",
        name_id: Some(4215552487),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 113,
        name: "Pull",
        name_id: Some(4132863117),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 114,
        name: "No Scope",
        name_id: Some(2602963073),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 115,
        name: "Achilles Spine",
        name_id: Some(3217141618),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 116,
        name: "Grand Slam",
        name_id: Some(1646928910),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 117,
        name: "Guardian Angel",
        name_id: Some(3334154676),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 118,
        name: "Interlinked",
        name_id: Some(651256911),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 119,
        name: "Death Race",
        name_id: Some(677323068),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 120,
        name: "Chain Reaction",
        name_id: Some(1969067783),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 121,
        name: "360",
        name_id: Some(1427176344),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 122,
        name: "Combat Evolved",
        name_id: Some(641726424),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 123,
        name: "Deadly Catch",
        name_id: Some(2396845048),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 124,
        name: "Driveby",
        name_id: Some(197913196),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 125,
        name: "Fastball",
        name_id: Some(1211820913),
        sorting_weight: Some(200),
    },
    FilmMedalDefinition {
        film_id: 126,
        name: "Flyin' High",
        name_id: Some(3739610597),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 127,
        name: "From the Grave",
        name_id: Some(2625820422),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 128,
        name: "From the Void",
        name_id: Some(3588869844),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 129,
        name: "Grapple-jack",
        name_id: Some(690125105),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 130,
        name: "Hold This",
        name_id: Some(175594566),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 131,
        name: "Last Shot",
        name_id: Some(3091261182),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 132,
        name: "Lawnmower",
        name_id: Some(3475540930),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 133,
        name: "Mount Up",
        name_id: Some(1065136443),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 134,
        name: "Off the Rack",
        name_id: Some(1283796619),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 135,
        name: "Quick Draw",
        name_id: Some(2861418269),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 136,
        name: "Party's Over",
        name_id: Some(3583966655),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 137,
        name: "Pineapple Express",
        name_id: Some(2019283350),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 138,
        name: "Ramming Speed",
        name_id: Some(1298835518),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 139,
        name: "Reclaimer",
        name_id: Some(1445036152),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 140,
        name: "Shot Caller",
        name_id: Some(1169571763),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 141,
        name: "Yard Sale",
        name_id: Some(1176569867),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 142,
        name: "Special Delivery",
        name_id: Some(275666139),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 143,
        name: "Street Sweeper",
        name_id: Some(2967011722),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 146,
        name: "Fumble",
        name_id: Some(3732790338),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 148,
        name: "Straight Balling",
        name_id: Some(781229683),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 150,
        name: "Big Deal",
        name_id: None,
        sorting_weight: None,
    },
    FilmMedalDefinition {
        film_id: 151,
        name: "Always Rotating",
        name_id: Some(1472686630),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 152,
        name: "Hill Guardian",
        name_id: Some(580478179),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 153,
        name: "Clock Stop",
        name_id: Some(3630529364),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 154,
        name: "Secure Line",
        name_id: Some(2426456555),
        sorting_weight: Some(101),
    },
    FilmMedalDefinition {
        film_id: 155,
        name: "Signal Block",
        name_id: Some(3931425309),
        sorting_weight: Some(101),
    },
    FilmMedalDefinition {
        film_id: 156,
        name: "Splatter",
        name_id: Some(221693153),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 157,
        name: "Clash of Kings",
        name_id: None,
        sorting_weight: None,
    },
    FilmMedalDefinition {
        film_id: 158,
        name: "Contract Killer",
        name_id: None,
        sorting_weight: None,
    },
    FilmMedalDefinition {
        film_id: 160,
        name: "Watch the Throne",
        name_id: None,
        sorting_weight: None,
    },
    FilmMedalDefinition {
        film_id: 162,
        name: "All That Juice",
        name_id: Some(3528500956),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 163,
        name: "Great Journey",
        name_id: Some(1376646881),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 164,
        name: "Power Outage",
        name_id: Some(629165579),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 165,
        name: "Breacher",
        name_id: Some(2750622016),
        sorting_weight: Some(50),
    },
    FilmMedalDefinition {
        film_id: 166,
        name: "Mounted & Loaded",
        name_id: Some(1331361851),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 167,
        name: "Monopoly",
        name_id: Some(1090931685),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 168,
        name: "Counter-snipe",
        name_id: Some(1477806194),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 174,
        name: "Driving Spree",
        name_id: Some(3169118333),
        sorting_weight: Some(100),
    },
    FilmMedalDefinition {
        film_id: 175,
        name: "Death Cabbie",
        name_id: Some(2848470465),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 176,
        name: "Immortal Chauffeur",
        name_id: Some(1739996188),
        sorting_weight: Some(200),
    },
    FilmMedalDefinition {
        film_id: 177,
        name: "Blind Fire",
        name_id: Some(4007438389),
        sorting_weight: Some(150),
    },
    FilmMedalDefinition {
        film_id: 178,
        name: "Hang Up",
        name_id: Some(4285712605),
        sorting_weight: Some(51),
    },
    FilmMedalDefinition {
        film_id: 179,
        name: "Call Blocked",
        name_id: Some(2964157454),
        sorting_weight: Some(52),
    },
    FilmMedalDefinition {
        film_id: 180,
        name: "Clear Reception",
        name_id: Some(394349536),
        sorting_weight: Some(101),
    },
];

const fn legacy_names() -> [(u8, &'static str); FILM_MEDAL_DEFINITIONS.len()] {
    let mut names = [(0, ""); FILM_MEDAL_DEFINITIONS.len()];
    let mut i = 0;
    while i < names.len() {
        names[i] = (
            FILM_MEDAL_DEFINITIONS[i].film_id,
            FILM_MEDAL_DEFINITIONS[i].name,
        );
        i += 1;
    }
    names
}

/// Compatibility view of known film medal IDs and English names.
pub const KNOWN_FILM_MEDALS: &[(u8, &str)] = &legacy_names();

/// Resolve an on-film code without assuming it is a stats API NameId.
pub fn film_medal_definition(id: u8) -> Option<&'static FilmMedalDefinition> {
    FILM_MEDAL_DEFINITIONS
        .binary_search_by_key(&id, |m| m.film_id)
        .ok()
        .map(|i| &FILM_MEDAL_DEFINITIONS[i])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilmMedal {
    Known { id: u8, name: &'static str },
    Unknown(u8),
}

impl FilmMedal {
    pub fn from_id(id: u8) -> Self {
        film_medal_definition(id).map_or(Self::Unknown(id), |m| Self::Known { id, name: m.name })
    }
    pub const fn id(self) -> u8 {
        match self {
            Self::Known { id, .. } | Self::Unknown(id) => id,
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::Known { name, .. } => name,
            Self::Unknown(_) => "Unknown medal",
        }
    }
    pub fn name_id(self) -> Option<u32> {
        film_medal_definition(self.id())?.name_id
    }
    /// Resolve live/localized CMS metadata using the checked NameId mapping.
    pub fn metadata(self, catalog: &MedalMetadata) -> Option<&Medal> {
        catalog.medal(i64::from(self.name_id()?))
    }
}

/// Portable medal identity. Unknown codes remain present with no invented name or NameId.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MedalAward {
    pub film_id: u8,
    pub name: Option<String>,
    pub name_id: Option<u32>,
}

impl MedalAward {
    pub fn from_film_id(id: u8) -> Self {
        let definition = film_medal_definition(id);
        Self {
            film_id: id,
            name: definition.map(|m| m.name.to_owned()),
            name_id: definition.and_then(|m| m.name_id),
        }
    }
}
