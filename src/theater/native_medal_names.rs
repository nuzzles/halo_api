//! Exact native medal identity pairs from LevelUp feat/v75 medalname/table.go.
//! This deliberately does not infer an identity from a medal code alone.

/// Number of measured pairs in the pinned native catalog.
pub const NATIVE_MEDAL_NAME_COUNT: usize = 124;

/// Resolve the native pair; unknown or out-of-byte-range values have no identity.
pub fn native_medal_name(type_hint: i32, medal_type: i32) -> Option<&'static str> {
    match (type_hint, medal_type) {
        (50, 26) => Some("Killjoy"),
        (50, 36) => Some("Stopped Short"),
        (50, 37) => Some("Flag Joust"),
        (50, 62) => Some("Spotter"),
        (50, 63) => Some("Treasure Hunter"),
        (50, 64) => Some("Saboteur"),
        (50, 65) => Some("Wingman"),
        (50, 66) => Some("Wheelman"),
        (50, 67) => Some("Gunner"),
        (50, 68) => Some("Driver"),
        (50, 69) => Some("Pilot"),
        (50, 70) => Some("Tanker"),
        (50, 71) => Some("Rifleman"),
        (50, 72) => Some("Bomber"),
        (50, 73) => Some("Grenadier"),
        (50, 74) => Some("Boxer"),
        (50, 75) => Some("Warrior"),
        (50, 76) => Some("Gunslinger"),
        (50, 77) => Some("Scattergunner"),
        (50, 78) => Some("Sharpshooter"),
        (50, 79) => Some("Marksman"),
        (50, 80) => Some("Heavy"),
        (50, 81) => Some("Bodyguard"),
        (50, 82) => Some("Back Smack"),
        (50, 87) => Some("Dogfight"),
        (50, 88) => Some("Harpoon"),
        (50, 91) => Some("Odin's Raven"),
        (50, 97) => Some("Skyjack"),
        (50, 98) => Some("Stick"),
        (50, 101) => Some("Kong"),
        (50, 105) => Some("Reversal"),
        (50, 108) => Some("Snipe"),
        (50, 117) => Some("Guardian Angel"),
        (50, 120) => Some("Chain Reaction"),
        (50, 126) => Some("Flyin' High"),
        (50, 127) => Some("From the Grave"),
        (50, 131) => Some("Last Shot"),
        (50, 133) => Some("Mount Up"),
        (50, 135) => Some("Quick Draw"),
        (50, 139) => Some("Reclaimer"),
        (50, 142) => Some("Special Delivery"),
        (50, 151) => Some("Always Rotating"),
        (50, 156) => Some("Splatter"),
        (50, 157) => Some("Clash of Kings"),
        (50, 160) => Some("Watch the Throne"),
        (50, 165) => Some("Breacher"),
        (51, 178) => Some("Hang Up"),
        (52, 179) => Some("Call Blocked"),
        (100, 0) => Some("Double Kill"),
        (100, 9) => Some("Killing Spree"),
        (100, 38) => Some("Goal Line Stand"),
        (100, 85) => Some("Bulltrue"),
        (100, 86) => Some("Cluster Luck"),
        (100, 89) => Some("Mind the Gap"),
        (100, 92) => Some("Pancake"),
        (100, 96) => Some("Rideshare"),
        (100, 99) => Some("Tag & Bag"),
        (100, 100) => Some("Whiplash"),
        (100, 104) => Some("Windshield Wiper"),
        (100, 106) => Some("Hail Mary"),
        (100, 107) => Some("Nade Shot"),
        (100, 109) => Some("Perfect"),
        (100, 110) => Some("Bank Shot"),
        (100, 111) => Some("Fire & Forget"),
        (100, 112) => Some("Ballista"),
        (100, 113) => Some("Pull"),
        (100, 114) => Some("No Scope"),
        (100, 119) => Some("Death Race"),
        (100, 128) => Some("From the Void"),
        (100, 129) => Some("Grapple-jack"),
        (100, 130) => Some("Hold This"),
        (100, 132) => Some("Lawnmower"),
        (100, 134) => Some("Off the Rack"),
        (100, 137) => Some("Pineapple Express"),
        (100, 138) => Some("Ramming Speed"),
        (100, 140) => Some("Shot Caller"),
        (100, 141) => Some("Yard Sale"),
        (100, 146) => Some("Fumble"),
        (100, 150) => Some("Big Deal"),
        (100, 152) => Some("Hill Guardian"),
        (100, 153) => Some("Clock Stop"),
        (100, 166) => Some("Mounted & Loaded"),
        (100, 168) => Some("Counter-snipe"),
        (100, 174) => Some("Driving Spree"),
        (101, 154) => Some("Secure Line"),
        (101, 180) => Some("Clear Reception"),
        (150, 1) => Some("Triple Kill"),
        (150, 10) => Some("Killing Frenzy"),
        (150, 31) => Some("Flawless Victory"),
        (150, 32) => Some("Steaktacular"),
        (150, 84) => Some("Boom Block"),
        (150, 95) => Some("Return to Sender"),
        (150, 102) => Some("Autopilot Engaged"),
        (150, 103) => Some("Sneak King"),
        (150, 115) => Some("Achilles Spine"),
        (150, 116) => Some("Grand Slam"),
        (150, 118) => Some("Interlinked"),
        (150, 121) => Some("360"),
        (150, 122) => Some("Combat Evolved"),
        (150, 123) => Some("Deadly Catch"),
        (150, 143) => Some("Street Sweeper"),
        (150, 148) => Some("Straight Balling"),
        (150, 158) => Some("Contract Killer"),
        (150, 162) => Some("All That Juice"),
        (150, 175) => Some("Death Cabbie"),
        (150, 177) => Some("Blind Fire"),
        (200, 13) => Some("Perfection"),
        (200, 44) => Some("Extermination"),
        (200, 90) => Some("Ninja"),
        (200, 93) => Some("Quigley"),
        (200, 94) => Some("Remote Detonation"),
        (200, 125) => Some("Fastball"),
        (205, 11) => Some("Running Riot"),
        (210, 12) => Some("Rampage"),
        (220, 2) => Some("Overkill"),
        (220, 27) => Some("Nightmare"),
        (225, 3) => Some("Killtacular"),
        (230, 4) => Some("Killtrocity"),
        (230, 28) => Some("Boogeyman"),
        (235, 5) => Some("Killamanjaro"),
        (240, 6) => Some("Killtastrophe"),
        (240, 29) => Some("Grim Reaper"),
        (245, 7) => Some("Killpocalypse"),
        (250, 30) => Some("Demon"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;

    #[test]
    fn native_medal_names_exhaustive() {
        #[derive(Deserialize)]
        struct Row {
            hint: i32,
            medal: i32,
            name: String,
            known: bool,
        }
        #[derive(Deserialize)]
        struct Oracle {
            count: usize,
            rows: Vec<Row>,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/medal-names-v41.json.zlib")[..])
            .read_to_end(&mut bytes)
            .unwrap();
        let oracle: Oracle = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(oracle.count, NATIVE_MEDAL_NAME_COUNT);
        assert_eq!(oracle.rows.len(), 258 * 258);
        let mut known = 0;
        for row in oracle.rows {
            let actual = native_medal_name(row.hint, row.medal);
            assert_eq!(
                actual,
                row.known.then_some(row.name.as_str()),
                "pair {},{}",
                row.hint,
                row.medal
            );
            known += usize::from(actual.is_some());
        }
        assert_eq!(known, 124);
        assert_eq!(native_medal_name(i32::MIN, i32::MAX), None);
    }

    #[test]
    fn native_medal_names_in_highlight_export() {
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        use crate::theater::{FilmHighlightStream, scan_film_highlights};
        let mut data = Vec::new();
        for (hint, medal, flag) in [(50, 26, 1), (51, 26, 1), (50, 26, 0), (50, 26, 1)] {
            let mut block = vec![0; 80];
            block[..8].copy_from_slice(&2_500_000_000_000_001u64.to_le_bytes());
            block[8..10].copy_from_slice(&[0x2d, 0xc0]);
            block[57] = hint;
            block[65] = flag;
            block[69] = medal;
            block[70..74].copy_from_slice(&[0, 0, 0x2e, 0xe0]);
            data.extend(block);
        }
        let stream = scan_film_highlights(
            &[FilmChunkData {
                metadata: FilmChunk {
                    index: 1,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                    chunk_type: 1,
                },
                data,
            }],
            41,
        )
        .unwrap();
        assert_eq!(stream.events.len(), 4);
        assert_eq!(
            stream.medal_names,
            vec![Some("Killjoy".into()), None, None, Some("Killjoy".into())]
        );
        let encoded = serde_json::to_value(&stream).unwrap();
        assert_eq!(
            encoded["medal_names"],
            serde_json::json!(["Killjoy", null, null, "Killjoy"])
        );
        assert_eq!(
            serde_json::from_value::<FilmHighlightStream>(encoded).unwrap(),
            stream
        );
    }
}
