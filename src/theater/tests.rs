use super::bits::Bits;
use super::*;
use crate::clients::hi::models::{FilmChunk, FilmChunkData};
use serde_json::Value;

fn fixture(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}
fn bytes(row: &Value) -> Vec<u8> {
    row["hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks(2)
        .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
        .collect()
}
fn num(v: &Value) -> usize {
    v.as_u64().unwrap() as usize
}
fn flip(data: &mut [u8], bit: usize) {
    data[bit / 8] ^= 1 << (7 - bit % 8);
}
fn put_bytes(data: &mut [u8], bit: usize, value: &[u8]) {
    for (i, &byte) in value.iter().enumerate() {
        for j in 0..8 {
            let at = bit + i * 8 + j;
            let mask = 1 << (7 - at % 8);
            data[at / 8] = (data[at / 8] & !mask) | ((byte >> (7 - j) & 1) << (7 - at % 8));
        }
    }
}
fn chunk(index: i32, kind: i32, data: Vec<u8>) -> FilmChunkData {
    FilmChunkData {
        metadata: FilmChunk {
            index,
            chunk_type: kind,
            start_time_offset_ms: 0,
            duration_ms: 0,
            size: data.len() as i64,
            file_relative_path: String::new(),
        },
        data,
    }
}
fn frame(timestamp: u64, payload: &[u8]) -> Vec<u8> {
    let mut data = vec![0; 4];
    data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    data.extend_from_slice(&timestamp.to_le_bytes());
    data.extend_from_slice(payload);
    data
}
#[test]
fn bounded_reads_and_rolling_windows_agree_at_every_alignment() {
    let data = [
        0x01, 0xff, 0x35, 0xa2, 0x00, 0x71, 0xaa, 0x55, 0x84, 0x19, 0x18,
    ];
    let b = Bits(&data);
    for (offset, w) in b.windows() {
        for width in 1..=64.min(b.len() - offset) {
            let expected = (0..width).fold(0u64, |v, i| {
                (v << 1) | u64::from((data[(offset + i) / 8] >> (7 - (offset + i) % 8)) & 1)
            });
            assert_eq!(b.read(offset, width), Some(expected));
            assert_eq!(w >> (64 - width), expected);
        }
    }
    assert_eq!(b.read(usize::MAX, 2), None);
    assert_eq!(b.read(0, 65), None);
    assert_eq!(b.read(b.len(), 1), None);
    assert_eq!(b.read(b.len(), 0), Some(0));
}
#[test]
fn version_duplicates_and_packet_truncation_are_typed() {
    assert!(matches!(
        Film::try_from_chunks(
            &[],
            DecodeOptions {
                major_version: 40,
                ..DecodeOptions::v41()
            }
        ),
        Err(DecodeError::UnsupportedVersion(40))
    ));
    assert!(matches!(packets::index(&[]), Err(DecodeError::Missing(_))));
    assert!(matches!(
        packets::index(&[chunk(1, 2, vec![]), chunk(1, 1, vec![])]),
        Err(DecodeError::DuplicateChunk(1))
    ));
    let data = frame(100, &[1, 2, 3]);
    for n in 1..data.len() {
        assert!(matches!(
            packets::index(&[chunk(2, 2, data[..n].to_vec())]),
            Err(DecodeError::Truncated { .. })
        ));
    }
    let p = packets::index(&[chunk(2, 2, data.clone()), chunk(1, 2, data)]).unwrap();
    assert_eq!(p.iter().map(|p| p.chunk_index).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(
        (p[0].payload_size, p[0].payload_offset, p[0].timestamp_us),
        (3, 16, 100)
    );
}
#[test]
fn captured_spawns_derive_shift_coordinates_and_reused_identity() {
    let f = fixture(include_str!("fixtures/oddball_records.json"));
    for r in f["spawns"].as_array().unwrap() {
        let mut data = bytes(r);
        let s = motion::spawn(Bits(&data), 0).unwrap();
        assert_eq!(usize::from(s.serial), num(&r["serial"]));
        assert_eq!(usize::from(s.player), num(&r["expected"]["player"]));
        assert_eq!(serde_json::to_value(s.xyz).unwrap(), r["expected"]["xyz"]);
        assert_eq!(s.layout, CoordinateLayout::X18Y18Z15);
        flip(&mut data, 192 + num(&r["expected"]["shift"]));
        assert!(motion::spawn(Bits(&data), 0).is_none());
    }
}
#[test]
fn coordinate_layout_names_round_trip_and_read_older_exports() {
    for (old, layout, name) in [
        ("Controlled", CoordinateLayout::X15Y15Z17, "X15Y15Z17"),
        ("Ranked", CoordinateLayout::X18Y18Z15, "X18Y18Z15"),
        ("X17Y17Z16", CoordinateLayout::X17Y17Z16, "X17Y17Z16"),
    ] {
        assert_eq!(
            serde_json::from_value::<CoordinateLayout>(old.into()).unwrap(),
            layout
        );
        assert_eq!(serde_json::to_value(layout).unwrap(), name);
    }
}

#[test]
fn captured_idle_aquarius_matches_map_derived_axis_partition() {
    let f = fixture(include_str!("fixtures/aquarius_unresolved_spawn.json"));
    let data = bytes(&f);
    let b = Bits(&data);
    assert!(b.is(20, motion::SPAWN_BODY));
    assert!(b.is(192, f["coordinate_prefix"].as_str().unwrap()));
    assert!(b.is(263, f["candidate_coordinate_bits"].as_str().unwrap()));
    assert!(b.is(299, motion::COORD_SUFFIX));
    // Independently supplied Aquarius BSP bounds predict 13/12/11, rather than
    // dividing the 36-bit window evenly or using its non-unique prefix.
    let s = motion::spawn(b, 0).unwrap();
    assert_eq!(s.layout, CoordinateLayout::X13Y12Z11);
    assert_eq!(s.xyz, [1980, 2469, 727]);
}

#[test]
fn captured_full_octagon_spawns_bind_existing_coordinate_layout() {
    let f = fixture(include_str!("fixtures/octagon_gameplay_records.json"));
    for r in f["cases"].as_array().unwrap() {
        let mut data = bytes(&r["spawn"]);
        let s = motion::spawn(Bits(&data), 0).unwrap();
        assert_eq!(s.layout, CoordinateLayout::X15Y15Z17);
        assert_eq!(usize::from(s.serial), num(&r["spawn"]["wire"]));
        assert_eq!(usize::from(s.player), num(&r["spawn"]["player"]));
        assert_eq!(serde_json::to_value(s.xyz).unwrap(), r["spawn"]["expected"]);
        flip(&mut data, 310);
        assert!(motion::spawn(Bits(&data), 0).is_none());
        let update = &r["first_position"];
        let data = bytes(update);
        let d = motion::clocked_delta(
            Bits(&data),
            num(&update["offset"]),
            s.layout,
            num(&update["tick"]) as u8,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(d.position).unwrap(),
            update["expected"]
        );
        assert_eq!(d.end, num(&update["end"]));
        assert!(
            s.xyz
                .iter()
                .zip(d.position.unwrap())
                .all(|(a, b)| a.abs_diff(b) < 32)
        );
    }
}

#[test]
fn captured_weapon_hits_identify_victims_and_require_the_firing_companion() {
    let f = fixture(include_str!("fixtures/weapon_damage_records.json"));
    for r in f["records"].as_array().unwrap() {
        let data = bytes(r);
        let o = num(&r["offset"]);
        let hit = combat::weapon_damage(Bits(&data), o).unwrap();
        assert_eq!(usize::from(hit.serial), num(&r["life"]));
        assert_eq!(hit.end, o + 39);
        assert_eq!(hit.player, None); // Victim roster is established by its life.
        for bit in [o + 21, o - 108 + 80] {
            let mut bad = data.clone();
            flip(&mut bad, bit);
            assert!(combat::weapon_damage(Bits(&bad), o).is_none());
        }
        let mut different_life = data.clone();
        flip(&mut different_life, o + 19);
        assert_ne!(
            combat::weapon_damage(Bits(&different_life), o)
                .unwrap()
                .serial,
            hit.serial
        );
        assert!(combat::weapon_damage(Bits(&data[..(o + 38) / 8]), o).is_none());
    }
}

#[test]
fn captured_raid_spawns_and_events_support_all_generation_tags() {
    let f = fixture(include_str!("fixtures/raid_records.json"));
    for r in f["cases"].as_array().unwrap() {
        let record = &r["spawn"];
        let data = bytes(record);
        let s = motion::spawn(Bits(&data), 0).unwrap();
        assert_eq!(s.layout, CoordinateLayout::X15Y15Z17);
        assert_eq!(usize::from(s.serial), num(&record["serial"]));
        assert_eq!(usize::from(s.player), num(&record["player"]));
        assert_eq!(s.end, num(&record["end"]));
        assert_eq!(serde_json::to_value(s.xyz).unwrap(), record["expected"]);
        for bit in [2, 20, s.end - 30] {
            let mut corrupt = data.clone();
            flip(&mut corrupt, bit);
            assert!(motion::spawn(Bits(&corrupt), 0).is_none());
        }
        assert!(motion::spawn(Bits(&data[..data.len() - 1]), 0).is_none());
        if let Some(m) = r.get("motion") {
            let data = bytes(m);
            let o = num(&m["offset"]);
            assert_eq!(motion::pawn_header(Bits(&data), o).unwrap().0, s.serial);
            let d = motion::clocked_delta(Bits(&data), o, s.layout, num(&m["tick"]) as u8).unwrap();
            assert_eq!(serde_json::to_value(d.position).unwrap(), m["expected"]);
            assert_eq!(d.end, num(&m["end"]));
            assert!(
                s.xyz
                    .iter()
                    .zip(d.position.unwrap())
                    .all(|(a, b)| a.abs_diff(b) < 32)
            );
            assert!(
                motion::clocked_delta(
                    Bits(&data),
                    o,
                    s.layout,
                    (num(&m["tick"]) as u8).wrapping_add(1)
                )
                .is_none()
            );
        }
    }
    for r in f["firing"].as_array().unwrap() {
        let data = bytes(r);
        let o = num(&r["offset"]);
        let e = combat::firing(Bits(&data), o).unwrap();
        assert_eq!(usize::from(e.serial), num(&r["serial"]));
        assert_eq!(usize::from(e.player.unwrap()), num(&r["player"]));
        let mut corrupt = data.clone();
        flip(&mut corrupt, o + 80);
        assert!(combat::firing(Bits(&corrupt), o).is_none());
    }
    assert!(motion::spawn(Bits(&bytes(&f["unsupported"])), 0).is_none());
}

#[test]
fn captured_appearance_matches_coating_and_model_identifiers_at_every_alignment() {
    let fixture = fixture(include_str!("fixtures/appearance_records.json"));
    for r in fixture["records"].as_array().unwrap() {
        let data = bytes(r);
        let o = num(&r["offset"]);
        for shift in 0..8 {
            let mut shifted = vec![0; data.len() + 1];
            put_bytes(&mut shifted, shift, &data);
            let (xuid, name, value) = appearance::snapshot(Bits(&shifted), o + shift).unwrap();
            assert_eq!(xuid.to_string(), r["xuid"].as_str().unwrap());
            assert_eq!(name, r["name"].as_str().unwrap());
            assert_eq!(serde_json::to_value(value).unwrap(), r["expected"]);
        }
        for bit in [o, o + 16, o + 46, o + 54, o - 232] {
            let mut bad = data.clone();
            flip(&mut bad, bit);
            assert!(appearance::snapshot(Bits(&bad), o).is_none());
        }
        let mut bad = data.clone();
        put_bytes(&mut bad, o - 64, &[0; 8]);
        assert!(appearance::snapshot(Bits(&bad), o).is_none());
        // Missing final coating bits must not produce a default coating.
        assert!(appearance::snapshot(Bits(&data[..(o + 2637) / 8]), o).is_none());
    }
    let records = fixture["records"].as_array().unwrap();
    let mut blue = records[0]["expected"].clone();
    let brick = &records[1]["expected"];
    assert_eq!(blue["coating_style_id"], 0x513a2ab0);
    assert_eq!(brick["coating_style_id"], 0x727d2464);
    blue["coating_style_id"] = brick["coating_style_id"].clone();
    assert_eq!(&blue, brick); // Controlled change affects only the coating.
}

#[test]
fn captured_attachments_match_cms_and_older_exports_remain_unknown() {
    let f = fixture(include_str!("fixtures/appearance_records.json"));
    for row in f["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["film"].as_str().unwrap().starts_with("weapons/"))
    {
        let (_, _, value) = appearance::snapshot(Bits(&bytes(row)), num(&row["offset"])).unwrap();
        let a = value.attachments.as_ref().unwrap();
        // Independent official TagIds: Packrat, Myesel, Holodyne, Alpha L/R.
        assert_eq!(a.chest_tag_id as u32, 0x722f60d1);
        assert_eq!(a.utility_tag_id as u32, 0xf048a42d);
        assert_eq!(a.wrist_tag_id as u32, 0x9a201f3b);
        assert_eq!(a.left_shoulder_tag_id as u32, 0xf9d6cff1);
        assert_eq!(a.right_shoulder_tag_id as u32, 0x96f00c69);
        assert_eq!(
            value.mythic_effect_ids.unwrap().map(|id| id as u32),
            [0x5294fd4e, 0x9ad0ac33, 0xf4f430dc, 0x607b1b0f]
        );
        let mut old = serde_json::to_value(value).unwrap();
        old.as_object_mut().unwrap().remove("attachments");
        old.as_object_mut().unwrap().remove("mythic_effect_ids");
        let old: ArmorAppearance = serde_json::from_value(old).unwrap();
        assert_eq!(old.attachments, None);
        assert_eq!(old.mythic_effect_ids, None);
    }
}

#[test]
fn captured_bazaar_spawn_and_motion_use_a_third_coordinate_layout() {
    let f = fixture(include_str!("fixtures/bazaar_records.json"));
    let layout = CoordinateLayout::X17Y17Z16;
    let mut data = bytes(&f["spawn"]);
    let s = motion::spawn(Bits(&data), 0).unwrap();
    assert_eq!(s.layout, layout);
    assert_eq!((s.serial, s.player, s.end), (0, 0, 343));
    assert_eq!(serde_json::to_value(s.xyz).unwrap(), f["spawn"]["expected"]);
    flip(&mut data, 313); // The suffix is three bits later than the Forge controls.
    assert!(motion::spawn(Bits(&data), 0).is_none());
    for r in f["deltas"].as_array().unwrap() {
        let mut data = bytes(r);
        let tick = num(&r["tick"]) as u8;
        let d = motion::clocked_delta(Bits(&data), 37, layout, tick).unwrap();
        assert_eq!(serde_json::to_value(d.position).unwrap(), r["expected"]);
        assert_eq!(
            serde_json::to_value(d.aim.map(|a| [a.yaw, a.pitch])).unwrap(),
            r["aim"]
        );
        assert_eq!(d.end, num(&r["end"]));
        assert!(motion::clocked_delta(Bits(&data), 37, layout, tick.wrapping_add(1)).is_none());
        for wrong in [CoordinateLayout::X15Y15Z17, CoordinateLayout::X18Y18Z15] {
            assert!(motion::clocked_delta(Bits(&data), 37, wrong, tick).is_none());
        }
        if r["end_input"] == true {
            let c = motion::input_chain(Bits(&data), layout).unwrap();
            assert_eq!(c.position, d.position);
            assert_eq!(c.aim, d.aim);
            assert!(c.input.is_some());
            flip(&mut data, d.end + 3); // Corrupt the checked End/input continuation.
            assert!(motion::clocked_delta(Bits(&data), 37, layout, tick).is_none());
        }
    }
}

#[test]
fn input_chain_grammar_is_independent_of_coordinate_widths() {
    // Repack the captured A/B/C chains with the same XYZ in wider windows.
    // This is a synthetic encoding check, not evidence of another captured map.
    let f = fixture(include_str!("fixtures/controlled_records.json"));
    for r in f.as_array().unwrap().iter().take(3) {
        let data = bytes(r);
        let original = motion::input_chain(Bits(&data), CoordinateLayout::X15Y15Z17).unwrap();
        let (_, _, fields) = motion::pawn_header(Bits(&data), 37).unwrap();
        let xyz_start = fields + 5;
        let raw = data.iter().map(|b| format!("{b:08b}")).collect::<String>();
        for layout in [CoordinateLayout::X17Y17Z16, CoordinateLayout::X18Y18Z15] {
            // The captured Z needs 17 bits; choose a smaller Z for these layouts.
            let xyz = [
                original.position.unwrap()[0],
                original.position.unwrap()[1],
                12345,
            ];
            let mut bits = raw[..xyz_start].to_owned();
            for (value, width) in xyz.iter().zip(layout.axis_bits()) {
                bits.push_str(&format!("{value:0width$b}"));
            }
            bits.push_str(&raw[xyz_start + 47..]);
            while bits.len() % 8 != 0 {
                bits.push('0');
            }
            let data = bits
                .as_bytes()
                .chunks(8)
                .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 2).unwrap())
                .collect::<Vec<_>>();
            let c = motion::input_chain(Bits(&data), layout).unwrap();
            assert_eq!(c.position, Some(xyz));
            assert_eq!(c.input, original.input);
            assert_eq!(c.end, original.end + layout.coordinate_bits() - 47);
        }
    }
}
#[test]
fn captured_ranked_motion_and_command_ticks() {
    let f = fixture(include_str!("fixtures/oddball_records.json"));
    for r in f["deltas"].as_array().unwrap() {
        let mut data = bytes(r);
        let end = num(&r["length"]);
        if end / 8 == data.len() {
            data.push(0);
        }
        data[end / 8] |= 1 << (7 - end % 8);
        let tick = u8::from_str_radix(&r["frame_prefix"].as_str().unwrap()[29..37], 2).unwrap();
        let d = motion::clocked_delta(Bits(&data), 0, CoordinateLayout::X18Y18Z15, tick).unwrap();
        assert_eq!(serde_json::to_value(d.position).unwrap(), r["position"]);
        assert_eq!(
            serde_json::to_value(d.aim.map(|a| [a.yaw, a.pitch])).unwrap(),
            r["aim"]
        );
        assert!(
            motion::clocked_delta(
                Bits(&data),
                0,
                CoordinateLayout::X18Y18Z15,
                tick.wrapping_add(1)
            )
            .is_none()
        );
    }
}
#[test]
fn captured_firing_wrap_melee_companion_and_vitality() {
    let f = fixture(include_str!("fixtures/combat_records.json"));
    for r in f["firing"].as_array().unwrap() {
        let mut data = bytes(r);
        let e = combat::firing(Bits(&data), 0).unwrap();
        let x = &r["expected"];
        assert_eq!(usize::from(e.serial), num(&x["serial"]));
        assert_eq!(usize::from(e.player.unwrap()), num(&x["player"]));
        assert_eq!(usize::from(e.value.sequence), num(&x["sequence"]));
        assert_eq!(
            format!("{:010x}", e.value.weapon_window),
            x["weapon_window"].as_str().unwrap()
        );
        flip(&mut data, 80);
        assert!(combat::firing(Bits(&data), 0).is_none());
    }
    for r in f["melee"].as_array().unwrap() {
        let mut data = bytes(r);
        let o = num(&r["offset"]);
        let e = combat::melee(Bits(&data), o).unwrap();
        assert_eq!(usize::from(e.serial), num(&r["expected"]["serial"]));
        assert_eq!(
            format!("{:010x}", e.value),
            r["expected"]["weapon_window"].as_str().unwrap()
        );
        flip(&mut data, 10);
        assert!(combat::melee(Bits(&data), o).is_none());
    }
    for r in f["vitality"].as_array().unwrap() {
        let mut data = bytes(r);
        let end = num(&r["length"]);
        if end / 8 == data.len() {
            data.push(0);
        }
        data[end / 8] |= 1 << (7 - end % 8);
        let d = motion::clocked_delta(
            Bits(&data),
            0,
            CoordinateLayout::X18Y18Z15,
            num(&r["clock"]) as u8,
        )
        .unwrap();
        let x = &r["expected"];
        if let Some(v) = x["body_raw"].as_u64() {
            assert_eq!(u64::from(d.body.unwrap().raw), v);
        }
        if let Some(v) = x["shield_raw"].as_u64() {
            assert_eq!(u64::from(d.shield.unwrap().raw), v);
        }
        if let Some(v) = x["shield_delay_ticks"].as_u64() {
            assert_eq!(u64::from(d.shield.unwrap().delay_ticks), v);
        }
    }
}
#[test]
fn captured_grenades_do_not_confuse_weapon_selection() {
    let f = fixture(include_str!("fixtures/grenade_records.json"));
    for r in f["throws"].as_array().unwrap() {
        let data = bytes(r);
        let e = combat::grenade(Bits(&data), 0).unwrap();
        assert_eq!(usize::from(e.serial), num(&r["expected"]["serial"]));
        assert_eq!(
            usize::from(e.player.unwrap()),
            num(&r["expected"]["player"])
        );
    }
    for r in f["projectiles"].as_array().unwrap() {
        let data = bytes(r);
        let o = num(&r["source"][3]);
        let (xyz, _) = projectile::delta(Bits(&data), o).unwrap();
        assert_eq!(serde_json::to_value(xyz).unwrap(), r["expected"]["xyz"]);
    }
    let data = bytes(&serde_json::json!({"hex":f["weapon_switch_hex"]}));
    assert!((0..data.len() * 8).all(|o| combat::grenade(Bits(&data), o).is_none()));
}
#[test]
fn captured_reload_ammo_zero_and_selection() {
    let f = fixture(include_str!("fixtures/weapon_records.json"));
    for r in f["reloads"].as_array().unwrap() {
        let data = bytes(r);
        let x = &r["evidence"];
        let e = combat::reload(Bits(&data), num(&x["bit"])).unwrap();
        assert_eq!(usize::from(e.serial), num(&x["serial"]));
        assert_eq!(e.end, num(&x["clock_bit"]) + 37);
    }
    let mut zero_seen = false;
    for r in f["deltas"].as_array().unwrap() {
        let mut data = bytes(r);
        let x = &r["evidence"];
        let o = num(&x["bit"]);
        let tick = x["clock"].as_u64().map(|v| v as u8);
        let d = motion::weapon_delta(Bits(&data), o, CoordinateLayout::X15Y15Z17, tick).unwrap();
        assert_eq!(serde_json::to_value(&d.ids).unwrap(), x["components"]);
        assert_eq!(d.end, num(&x["end"]));
        for field in x["fields"].as_array().unwrap() {
            if field["kind"] == "ammo" {
                let m = Magazine {
                    slot: num(&field["slot"]) as u8,
                    rounds: num(&field["value"]) as u8,
                };
                assert!(d.magazines.contains(&m));
                zero_seen |= m.rounds == 0;
            } else {
                assert_eq!(d.selection, Some(num(&field["slot"]) as u8));
            }
        }
        flip(&mut data, d.end);
        assert!(motion::weapon_delta(Bits(&data), o, CoordinateLayout::X15Y15Z17, tick).is_none());
    }
    assert!(zero_seen);
}
#[test]
fn captured_multiplayer_magazines_require_a_checked_neighbor() {
    let f = fixture(include_str!("fixtures/ranked_ammo_records.json"));
    for row in f["records"].as_array().unwrap() {
        let data = bytes(row);
        let o = num(&row["source"]["bit"]);
        let layout: CoordinateLayout = serde_json::from_value(row["layout"].clone()).unwrap();
        let tick = row["tick"].as_u64().map(|v| v as u8);
        let read = |data: &[u8], tick| motion::weapon_delta(Bits(data), o, layout, tick);
        if !row["accepted"].as_bool().unwrap() {
            assert!(read(&data, tick).is_none(), "{}", row["label"]);
            continue;
        }
        let d = read(&data, tick).unwrap();
        let expected: Magazine = serde_json::from_value(row["expected"].clone()).unwrap();
        assert_eq!(d.magazines, [expected], "{}", row["label"]);
        assert_eq!(d.end, num(&row["source"]["end_bit"]));
        assert_eq!(serde_json::to_value(&d.ids).unwrap(), row["components"]);

        // A plausible scalar is insufficient without its exact continuation.
        let mut bad_boundary = data.clone();
        flip(&mut bad_boundary, d.end);
        assert!(read(&bad_boundary, tick).is_none(), "{}", row["label"]);
        assert!(read(&data[..d.end / 8], tick).is_none());
        if d.ids.contains(&25) {
            assert!(read(&data, tick.map(|v| v.wrapping_add(1))).is_none());
            assert!(read(&data, None).is_none());
        }
        if let Some(peer_tick_bit) = row["peer_tick_bit"].as_u64() {
            let mut bad_peer = data.clone();
            flip(&mut bad_peer, peer_tick_bit as usize + 1);
            assert!(read(&bad_peer, tick).is_none());
        }
        for field in row["fields"].as_array().unwrap() {
            if !matches!(num(&field["id"]), 30 | 33) {
                continue;
            }
            let mut bad_scalar = data.clone();
            let bit = num(&field["bit"]);
            flip(&mut bad_scalar, bit);
            assert!(read(&bad_scalar, tick).is_none());
            if expected.rounds == 36 {
                // The adjacent unsupported value is withheld too.
                let mut unverified_amount = data.clone();
                flip(&mut unverified_amount, bit + 8);
                assert!(read(&unverified_amount, tick).is_none());
            }
        }
    }
}

#[test]
fn captured_firing_separates_slot_from_weapon_fingerprint() {
    let f = fixture(include_str!("fixtures/weapon_identity_records.json"));
    for row in f["records"].as_array().unwrap() {
        let data = bytes(row);
        let event = combat::firing(Bits(&data), num(&row["source"]["bit"])).unwrap();
        let firing = event.value;
        assert_eq!(
            firing.weapon_slot(),
            Some(num(&row["expected"]["slot"]) as u8)
        );
        assert_eq!(
            format!("{:08x}", firing.weapon_fingerprint().unwrap()),
            row["expected"]["fingerprint"].as_str().unwrap()
        );
        assert_eq!(
            format!("{:010x}", firing.weapon_window),
            row["expected"]["window"].as_str().unwrap()
        );
        for mask in [1, 1 << 36, 1 << 39] {
            let bad = Firing {
                weapon_window: firing.weapon_window ^ mask,
                ..firing
            };
            assert_eq!(bad.weapon_slot(), None);
            assert_eq!(bad.weapon_fingerprint(), None);
        }
    }
}

#[test]
fn captured_weapon_switch_returns_to_primary() {
    let f = fixture(include_str!("fixtures/weapon_switch_back_records.json"));
    for row in f["records"].as_array().unwrap() {
        let mut data = bytes(row);
        let o = num(&row["source"]["bit"]);
        let tick = Some(num(&row["tick"]) as u8);
        let d = motion::weapon_delta(Bits(&data), o, CoordinateLayout::X15Y15Z17, tick).unwrap();
        assert_eq!(d.selection, Some(num(&row["expected"]) as u8));
        assert_eq!(d.end, num(&row["source"]["end_bit"]));
        let magazines: Vec<Magazine> =
            serde_json::from_value(row["expected_magazines"].clone()).unwrap();
        assert_eq!(d.magazines, magazines);
        flip(&mut data, d.end);
        assert!(motion::weapon_delta(Bits(&data), o, CoordinateLayout::X15Y15Z17, tick).is_none());
    }
}

#[test]
fn captured_stalker_cooling_field_allows_selection_without_inventing_ammo() {
    let f = fixture(include_str!("fixtures/stalker_weapon_records.json"));
    let row = &f["selection"];
    let data = bytes(row);
    let o = num(&row["source"]["bit"]);
    let tick = Some(num(&row["tick"]) as u8);
    let read = |data: &[u8]| motion::weapon_delta(Bits(data), o, CoordinateLayout::X15Y15Z17, tick);
    let d = read(&data).unwrap();
    assert_eq!(d.ids, [35, 42]);
    assert_eq!(d.selection, Some(0));
    assert_eq!(d.end, num(&row["source"]["end_bit"]));
    assert!(d.magazines.is_empty());
    let cooling = num(&row["fields"][0]["bit"]);
    // Reject unchecked state bits, an unobserved larger value, and a broken
    // following boundary. No arbitrary-width skip is allowed.
    for bit in [cooling + 8, cooling + 7, cooling + 1, d.end] {
        let mut bad = data.clone();
        flip(&mut bad, bit);
        assert!(read(&bad).is_none());
    }
    assert!(read(&data[..d.end / 8]).is_none());
    // The isolated cooling samples never masquerade as a magazine or switch.
    for r in f["standalone_states"].as_array().unwrap() {
        assert!(
            motion::weapon_delta(
                Bits(&bytes(r)),
                num(&r["source"]["bit"]),
                CoordinateLayout::X15Y15Z17,
                None,
            )
            .is_none()
        );
    }
}

#[test]
fn captured_scope_stages_and_unknown_continuation() {
    let f = fixture(include_str!("fixtures/zoom_records.json"));
    for r in f["records"].as_array().unwrap() {
        let mut data = bytes(r);
        let x = &r["evidence"];
        let o = num(&x["bit"]);
        let e = combat::zoom(Bits(&data), o).unwrap();
        assert_eq!(usize::from(e.value.raw()), num(&x["level"]));
        assert_eq!(usize::from(e.serial), num(&x["serial"]));
        flip(&mut data, num(&x["clock_bit"]) + 10);
        assert!(combat::zoom(Bits(&data), o).is_none());
    }
    for r in f["unchecked"].as_array().unwrap() {
        assert!(combat::zoom(Bits(&bytes(r)), num(&r["evidence"]["bit"])).is_none());
    }
}

#[test]
fn captured_crouch_command_tails_preserve_release_and_reject_other_inputs() {
    let f = fixture(include_str!("fixtures/crouch_input_records.json"));
    for r in f["records"].as_array().unwrap() {
        let data = bytes(r);
        let chain = motion::input_chain(Bits(&data), CoordinateLayout::X15Y15Z17).unwrap();
        assert!(chain.input.is_some());
        assert_eq!(chain.end, num(&r["input_end"]));
        let expected = r["expected"].as_bool();
        let decoded = motion::crouch_input(Bits(&data), chain.end);
        assert_eq!(decoded.map(|s| s.0), expected);
        if let Some((value, end)) = decoded {
            assert_eq!(end, chain.end + if value { 10 } else { 5 });
            // Additional command data must not be mistaken for final padding.
            let mut extended = data.clone();
            extended.push(0);
            assert!(motion::crouch_input(Bits(&extended), chain.end).is_none());
            assert!(motion::crouch_input(Bits(&data[..data.len() - 1]), chain.end).is_none());
            let mut corrupt = data.clone();
            flip(&mut corrupt, end - 1);
            assert!(motion::crouch_input(Bits(&corrupt), chain.end).is_none());
        }
    }
}

#[test]
fn captured_motion_ends_at_paired_terminal_commands() {
    let f = fixture(include_str!("fixtures/motion_input_boundary_records.json"));
    for r in f["records"].as_array().unwrap() {
        let data = bytes(r);
        let o = num(&r["offset"]);
        let tick = num(&r["tick"]) as u8;
        let read = |data: &[u8], layout, tick| motion::clocked_delta(Bits(data), o, layout, tick);
        let d = read(&data, CoordinateLayout::X15Y15Z17, tick).unwrap();
        assert_eq!(d.end, num(&r["end"]));
        assert_eq!(serde_json::to_value(d.position).unwrap(), r["position"]);
        assert_eq!(serde_json::to_value(d.aim).unwrap(), r["aim"]);
        let commands = input::terminal_at(Bits(&data), d.end).unwrap();
        assert_eq!(commands.len(), 2);
        let scanned = input::terminal(Bits(&data)).unwrap();
        assert_eq!(
            scanned.iter().map(|c| (c.start, c.end)).collect::<Vec<_>>(),
            commands
                .iter()
                .map(|c| (c.start, c.end))
                .collect::<Vec<_>>()
        );
        for (command, expected) in commands.iter().zip(r["commands"].as_array().unwrap()) {
            assert_eq!(usize::from(command.player), num(&expected["player"]));
            assert_eq!(command.start, num(&expected["start"]));
            assert_eq!(command.end, num(&expected["end"]));
            assert_eq!(
                command.crouch,
                matches!(
                    expected["tail"].as_str().unwrap(),
                    "0010000100" | "0010010100"
                )
            );
        }
        assert_eq!(commands[0].end, commands[1].start);
        // End, tag, repeated tick, command buttons and byte padding all matter.
        for bit in [
            d.end + 1,
            d.end - 1,
            commands[0].start + 9,
            commands[1].start + 9,
            commands[0].buttons,
            data.len() * 8 - 1,
        ] {
            let mut corrupt = data.clone();
            flip(&mut corrupt, bit);
            assert!(
                read(&corrupt, CoordinateLayout::X15Y15Z17, tick).is_none(),
                "{} bit {bit}",
                r["payload_byte"]
            );
        }
        let mut extended = data.clone();
        extended.push(0);
        assert!(read(&extended, CoordinateLayout::X15Y15Z17, tick).is_none());
        assert!(read(&data[..data.len() - 1], CoordinateLayout::X15Y15Z17, tick).is_none());
        assert!(read(&data, CoordinateLayout::X15Y15Z17, tick.wrapping_add(1)).is_none());
        for layout in [
            CoordinateLayout::X13Y12Z11,
            CoordinateLayout::X17Y17Z16,
            CoordinateLayout::X18Y18Z15,
        ] {
            assert!(read(&data, layout, tick).is_none());
        }
    }
}

#[test]
fn captured_terminal_inputs_decode_both_players_and_combined_jump_crouch() {
    let f = fixture(include_str!("fixtures/terminal_input_records.json"));
    for r in f["records"].as_array().unwrap() {
        let data = bytes(r);
        let rows = input::terminal(Bits(&data)).unwrap();
        let expected = r["expected"].as_array().unwrap();
        assert_eq!(rows.len(), expected.len(), "{}", r["time_us"]);
        for (row, e) in rows.iter().zip(expected) {
            assert_eq!(usize::from(row.player), num(&e["player"]));
            assert_eq!(
                [row.axes.forward, row.axes.left],
                [num(&e["axes"][0]) as u8, num(&e["axes"][1]) as u8]
            );
            assert_eq!(row.crouch, e["crouch"].as_bool().unwrap());
            assert_eq!(row.start, num(&e["start"]));
            assert_eq!(row.buttons, row.start + 25);
            assert_eq!(row.end, num(&e["end"]));
        }
        // The final command must reach only byte-alignment padding. No extra
        // byte, truncated command, or nonzero padding is silently accepted.
        let mut extended = data.clone();
        extended.push(0);
        assert!(input::terminal(Bits(&extended)).is_none());
        assert!(input::terminal(Bits(&data[..data.len() - 1])).is_none());
        let end = rows.last().unwrap().end;
        if end < data.len() * 8 {
            let mut corrupt = data.clone();
            flip(&mut corrupt, end);
            assert!(input::terminal(Bits(&corrupt)).is_none());
        }
    }
}

#[test]
fn terminal_inputs_reject_unsupported_headers_axes_and_buttons() {
    let f = fixture(include_str!("fixtures/terminal_input_records.json"));
    let data = bytes(&f["records"][0]); // One command, header tag 14, released.
    let row = input::terminal(Bits(&data)).unwrap().remove(0);
    for bit in [
        row.start,
        row.start + 7,
        row.start + 9,
        row.buttons,
        row.start - 1,
    ] {
        let mut corrupt = data.clone();
        flip(&mut corrupt, bit);
        assert!(input::terminal(Bits(&corrupt)).is_none());
    }
    let mut axes = data.clone();
    put_bytes(&mut axes, row.start + 13, &[0xff]); // Invalid axis endpoint 63.
    assert!(input::terminal(Bits(&axes)).is_none());
    let data = bytes(&f["records"][4]); // Held crouch after a respawn.
    let row = input::terminal(Bits(&data)).unwrap().remove(0);
    let mut unknown = data.clone();
    flip(&mut unknown, row.buttons + 3); // An unvalidated button combination.
    assert!(input::terminal(Bits(&unknown)).is_none());
}
#[test]
fn all_record_readers_tolerate_short_arbitrary_payloads() {
    let mut state = 0x12345678u32;
    for length in 0..128 {
        let data: Vec<u8> = (0..length)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                state as u8
            })
            .collect();
        let b = Bits(&data);
        for o in 0..=b.len() {
            let _ = (
                combat::firing(b, o),
                combat::weapon_damage(b, o),
                combat::melee(b, o),
                combat::grenade(b, o),
                combat::reload(b, o),
                combat::zoom(b, o),
            );
            let _ = (
                motion::spawn(b, o),
                motion::clocked_delta(b, o, CoordinateLayout::X18Y18Z15, 0),
                motion::weapon_delta(b, o, CoordinateLayout::X15Y15Z17, None),
                motion::isolated_vitality(b, o),
                motion::crouch_input(b, o),
            );
            let _ = (
                projectile::spawn(b, o),
                projectile::delta(b, o),
                appearance::snapshot(b, o),
            );
        }
        let _ = motion::input_chain(b, CoordinateLayout::X15Y15Z17);
        let _ = input::terminal(b);
    }
}

#[test]
fn film_pipeline_binds_lives_excludes_death_deduplicates_and_round_trips() {
    // Synthetic envelope around captured record forms isolates orchestration.
    let mut registry = vec![0; 36 * 16640];
    for i in 0..43 {
        let name = match i {
            0 => "object-position-dynamic-precision-component",
            1 => "object-translational-velocity-dynamic-precision-component",
            4 => "object-body-vitality-component",
            5 => "object-shield-vitality-component",
            21 => "unit-desired-aiming-vector-component",
            25 => "unit-command-tick-component",
            30 | 33 => "weapon-state-ammo",
            31 | 34 => "weapon-state-rounds-inventory",
            35 => "weapon-state-overheated",
            42 => "biped-desired-weapon-set",
            _ => "test-opaque-component",
        };
        let start = 35 * 16640 + i * 260 + 8;
        registry[start..start + name.len()].copy_from_slice(name.as_bytes());
    }
    let mut tag = vec![0; 32];
    tag[..8].copy_from_slice(&[b'T', 0, b'e', 0, b's', 0, b't', 0]);
    let mut roster = tag.clone();
    roster.extend_from_slice(&[0; 21]);
    roster.extend_from_slice(&1u64.to_le_bytes());
    roster.extend_from_slice(&[0x2d, 0xc0]);
    let spawns = fixture(include_str!("fixtures/oddball_records.json"));
    let spawn = bytes(&spawns["spawns"][0]);
    let combat = fixture(include_str!("fixtures/combat_records.json"));
    let mut shot = bytes(&combat["firing"][0]);
    // Set the captured event's wire/roster to the synthetic life zero.
    for bit in (12..20).chain(35..40) {
        shot[bit / 8] &= !(1 << (7 - bit % 8));
    }
    let mut second = shot.clone();
    flip(&mut second, 26);
    let mut data = frame(1_000_000, &roster);
    let appearances = fixture(include_str!("fixtures/appearance_records.json"));
    let row = &appearances["records"][0];
    let mut appearance = bytes(row);
    let marker = num(&row["offset"]);
    put_bytes(&mut appearance, marker - 488, &tag);
    put_bytes(&mut appearance, marker - 64, &1u64.to_le_bytes());
    for stamp in [1_100_000, 5_500_000] {
        let mut snapshot = frame(stamp, &appearance);
        snapshot[0] = 8;
        data.extend(snapshot); // Appearance is valid before spawn and after death.
    }
    data.extend(frame(1_200_000, &appearance)); // Same bytes in a FRAME aren't a snapshot.
    let mut wrong_name = frame(1_300_000, &bytes(row));
    wrong_name[0] = 8;
    put_bytes(&mut wrong_name[16..], marker - 64, &1u64.to_le_bytes());
    data.extend(wrong_name); // Matching XUID with a different name must not bind.
    data.extend(frame(1_500_000, &shot)); // Before spawn: exclude.
    data.extend(frame(2_000_000, &spawn));
    // Captured component-1/25-only packet from the natural-end idle control.
    let stopped = bytes(&serde_json::json!({"hex":"a07b42058440088165d804035f7c00"}));
    for stamp in [1_500_000, 2_200_000, 2_200_000, 5_000_000] {
        data.extend(frame(stamp, &stopped)); // Before spawn, duplicate, at death.
    }
    let crouches = fixture(include_str!("fixtures/crouch_input_records.json"));
    for (t, value) in [(1_700_000, true), (3_500_000, true), (3_600_000, false)] {
        let r = crouches["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["expected"].as_bool() == Some(value))
            .unwrap();
        data.extend(frame(t, &bytes(r)));
    }
    for payload in [&shot, &second, &shot] {
        data.extend(frame(3_000_000, payload));
    }
    data.extend(frame(5_000_000, &shot)); // At death: exclude.
    let mut summary = vec![0; 15];
    let mut event = vec![0; 60];
    event[..32].copy_from_slice(&tag);
    event[47] = 20;
    event[48..52].copy_from_slice(&4000u32.to_be_bytes());
    summary.extend(event);
    let mut chunks = vec![
        chunk(3, 3, summary),
        chunk(2, 2, data),
        chunk(0, 1, registry),
    ];
    let options = DecodeOptions {
        duration_us: Some(6_000_000),
        ..DecodeOptions::v41()
    };
    let film = Film::try_from_chunks(&chunks, options.clone()).unwrap();
    assert_eq!(film.origin_timestamp_us, 1_000_000);
    assert_eq!(film.players.len(), 1);
    let p = &film.players[0];
    assert_eq!(p.name, "Test");
    assert_eq!(p.xuid.as_deref(), Some("1"));
    assert_eq!(p.appearance.len(), 2);
    assert_eq!(p.appearance[0].time_us, 100_000);
    assert_eq!(p.appearance[1].time_us, 4_500_000);
    assert_eq!(p.appearance[0].value.coating_style_id, 0x513a2ab0);
    assert_eq!(
        p.crouch_input
            .iter()
            .map(|s| (s.time_us, s.value))
            .collect::<Vec<_>>(),
        vec![(1_200_000, false), (2_500_000, true), (2_600_000, false)]
    );
    assert_eq!(p.lives[0].death_us, Some(4_000_000));
    assert_eq!(p.velocities.len(), 1);
    assert_eq!(
        (p.velocities[0].time_us, p.velocities[0].life),
        (1_200_000, 0)
    );
    assert_eq!(p.velocities[0].value, Velocity::Stationary);
    assert_eq!(
        (p.velocities[0].source.bit, p.velocities[0].source.end_bit),
        (70, 72)
    );
    assert_eq!(p.positions.len(), 1);
    assert!(p.positions[0].value.spawn);
    assert_eq!(p.firing.len(), 2);
    // Two shots at the same time carry the same weapon observation.
    assert_eq!(p.weapons.len(), 1);
    assert!(
        p.weapons
            .iter()
            .all(|w| w.value.slot == Some(0) && w.value.weapon_window.is_some())
    );
    assert_eq!(film.diagnostics.rejected["duplicate firing"], 1);
    assert!(
        p.aim.is_empty()
            && p.magazines.is_empty()
            && p.body.is_empty()
            && p.shields.is_empty()
            && p.zoom.is_empty()
    );
    assert!(!film.diagnostics.checked_regions.is_empty());
    let json = serde_json::to_vec(&film).unwrap();
    assert_eq!(serde_json::from_slice::<Film>(&json).unwrap(), film);

    // Terminal commands name roster slots even after the pawn wire changes.
    // Use a synthetic envelope around captured commands to isolate life binding.
    let inputs = fixture(include_str!("fixtures/terminal_input_records.json"));
    let command_payload = |record: &Value| {
        let original = bytes(record);
        let raw = original
            .iter()
            .map(|b| format!("{b:08b}"))
            .collect::<String>();
        let expected = record["expected"].as_array().unwrap();
        let mut bits = format!(
            "{}000{}",
            &raw[..37],
            &raw[num(&expected[0]["start"])..num(&expected.last().unwrap()["end"])]
        );
        while bits.len() % 8 != 0 {
            bits.push('0');
        }
        bits.as_bytes()
            .chunks(8)
            .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 2).unwrap())
            .collect::<Vec<_>>()
    };
    let held = command_payload(&inputs["records"][4]);
    let paired = command_payload(&inputs["records"][3]);
    let mut respawn_chunks = chunks.clone();
    let data = &mut respawn_chunks
        .iter_mut()
        .find(|c| c.metadata.chunk_type == 2)
        .unwrap()
        .data;
    let mut respawn = spawn.clone();
    flip(&mut respawn, 16); // Wire 2, still roster 0.
    data.extend(frame(5_200_000, &respawn));
    let mut other = spawn.clone();
    flip(&mut other, 17); // Wire 1.
    flip(&mut other, 73); // Roster 1.
    data.extend(frame(2_100_000, &other));
    for stamp in [1_900_000, 5_000_000, 5_300_000] {
        data.extend(frame(stamp, &held)); // Before spawn, at death, then after respawn.
    }
    data.extend(frame(5_400_000, &paired));
    let mut no_clock = held.clone();
    flip(&mut no_clock, 8);
    data.extend(frame(5_500_000, &no_clock));
    let rebound = Film::try_from_chunks(&respawn_chunks, options.clone()).unwrap();
    assert_eq!(
        rebound.players[0]
            .crouch_input
            .iter()
            .map(|s| (s.time_us, s.life, s.value))
            .collect::<Vec<_>>(),
        vec![
            (1_200_000, 0, false),
            (2_500_000, 0, true),
            (2_600_000, 0, false),
            (4_300_000, 2, true),
            (4_400_000, 2, false)
        ]
    );
    assert_eq!(
        rebound.players[1]
            .crouch_input
            .iter()
            .map(|s| (s.time_us, s.life, s.value))
            .collect::<Vec<_>>(),
        vec![(4_400_000, 1, true)]
    );

    chunks.reverse();
    let compact = Film::try_from_chunks(
        &chunks,
        DecodeOptions {
            retain_coverage: false,
            ..options
        },
    )
    .unwrap();
    assert_eq!(compact.players, film.players);
    assert!(compact.packets.is_empty() && compact.diagnostics.checked_regions.is_empty());
}

#[test]
fn captured_controlled_position_aim_and_analog_chains() {
    let cases = fixture(include_str!("fixtures/controlled_records.json"));
    for r in cases.as_array().unwrap() {
        let mut data = bytes(r);
        let c = motion::input_chain(Bits(&data), CoordinateLayout::X15Y15Z17).unwrap();
        let expected = &r["expected"];
        if let Some(p) = expected.get("position") {
            assert_eq!(serde_json::to_value(c.position).unwrap(), *p);
        }
        if let Some(a) = expected.get("aim") {
            assert_eq!(
                serde_json::to_value(c.aim.map(|a| [a.yaw, a.pitch])).unwrap(),
                *a
            );
        }
        if let Some(input) = expected.get("input") {
            assert_eq!(
                serde_json::to_value(c.input.map(|i| [i.forward, i.left])).unwrap(),
                *input
            );
        }
        assert!(c.end <= data.len() * 8);
        // Both the frame and the pawn carry the tick; their agreement matters.
        flip(&mut data, 29);
        assert!(motion::input_chain(Bits(&data), CoordinateLayout::X15Y15Z17).is_none());
    }
}

#[test]
fn captured_velocity_directions_and_magnitudes_have_exact_source_ranges() {
    let f = fixture(include_str!("fixtures/velocity_records.json"));
    for r in f["records"].as_array().unwrap() {
        let data = bytes(r);
        let start = num(&r["velocity_bit"]);
        let end = num(&r["velocity_end"]);
        let (value, observed_end) = velocity::read(Bits(&data), start).unwrap();
        assert_eq!(observed_end, end);
        let value = value.unwrap();
        if r["direction_code"].is_null() {
            assert_eq!(value, Velocity::Stationary);
        } else {
            let Velocity::Directed {
                direction_code,
                direction,
                magnitude_code,
            } = value
            else {
                panic!("lost long velocity form")
            };
            assert_eq!(direction_code as usize, num(&r["direction_code"]));
            assert_eq!(magnitude_code as usize, num(&r["magnitude_code"]));
            for (actual, expected) in direction.iter().zip(r["direction"].as_array().unwrap()) {
                assert!((f64::from(*actual) - expected.as_f64().unwrap()).abs() < 1e-7);
            }
        }
        let layout = serde_json::from_value(r["layout"].clone()).unwrap();
        let accepted = if let Some(c) = motion::input_chain(Bits(&data), layout) {
            c.velocity
        } else {
            motion::clocked_delta(
                Bits(&data),
                num(&r["source"]["bit"]),
                layout,
                num(&r["tick"]) as u8,
            )
            .unwrap_or_else(|| panic!("{}", r["film"]))
            .velocity
        };
        assert_eq!(accepted, Some((start, end, value)));
        // Repack original component bits at all eight byte alignments.
        for alignment in 0..8 {
            let mut packed = vec![0; (alignment + end - start).div_ceil(8)];
            for i in 0..end - start {
                if Bits(&data).read(start + i, 1) == Some(1) {
                    flip(&mut packed, alignment + i);
                }
            }
            assert_eq!(
                velocity::read(Bits(&packed), alignment),
                Some((Some(value), alignment + end - start))
            );
            assert!(
                velocity::read(
                    Bits(&packed[..(alignment + end - start - 1) / 8]),
                    alignment
                )
                .is_none()
            );
        }
    }
}

#[test]
fn unsupported_velocity_codes_do_not_discard_checked_position() {
    // Unused face-grid slots, and the two codes beyond the last face.
    for code in [293, 293 * 294, 87_380, 524_286, 524_287, u32::MAX] {
        assert!(velocity::direction(code).is_none());
    }
    let f = fixture(include_str!("fixtures/velocity_records.json"));
    let r = &f["records"][0];
    let data = bytes(r);
    let o = num(&r["source"]["bit"]);
    let start = num(&r["velocity_bit"]);
    let layout = serde_json::from_value(r["layout"].clone()).unwrap();
    let tick = num(&r["tick"]) as u8;
    let original = motion::clocked_delta(Bits(&data), o, layout, tick).unwrap();
    let mut bad = data.clone();
    // Set the 19-bit direction to all ones, retaining the checked field length.
    for bit in start + 2..start + 21 {
        if Bits(&bad).read(bit, 1) == Some(0) {
            flip(&mut bad, bit);
        }
    }
    let d = motion::clocked_delta(Bits(&bad), o, layout, tick).unwrap();
    assert_eq!(d.position, original.position);
    assert_eq!(d.end, original.end);
    assert!(d.velocity.is_none());
    flip(&mut bad, start); // Unsupported form 10 must not be skipped.
    assert!(motion::clocked_delta(Bits(&bad), o, layout, tick).is_none());
    assert!(velocity::read(Bits(&data), usize::MAX).is_none());
    // Older portable exports remain readable without inventing stationary data.
    let mut old = serde_json::to_value(PlayerTrack::default()).unwrap();
    old.as_object_mut().unwrap().remove("velocities");
    assert!(
        serde_json::from_value::<PlayerTrack>(old)
            .unwrap()
            .velocities
            .is_empty()
    );
}

#[test]
fn captured_projectile_spawns_and_velocity_records_use_recorded_owners() {
    let f = fixture(include_str!("fixtures/projectile_motion_records.json"));
    for r in f["records"].as_array().unwrap() {
        let data = bytes(r);
        let o = num(&r["bit"]);
        let layout: CoordinateLayout = serde_json::from_value(r["layout"].clone()).unwrap();
        if r["kind"] == "spawn" {
            let (s, _) = projectile::birth(Bits(&data), o, layout).unwrap();
            assert_eq!(usize::from(s.id), num(&r["id"]));
            assert_eq!(usize::from(s.generation), num(&r["gen"]));
            assert_eq!(usize::from(s.player), num(&r["player"]));
            assert_eq!(usize::from(s.life), num(&r["life"]));
            let mut corrupt = data.clone();
            flip(&mut corrupt, o + 20); // A different archetype is not a projectile.
            assert!(projectile::birth(Bits(&corrupt), o, layout).is_none());
        } else {
            let d = projectile::record(Bits(&data), o, layout).unwrap();
            assert_eq!(d.end, num(&r["end"]));
            assert_eq!(usize::from(d.id), num(&r["id"]));
            assert_eq!(usize::from(d.generation), num(&r["gen"]));
            assert_eq!(
                serde_json::to_value(d.position.map(|(_, _, v)| v)).unwrap(),
                r["xyz"]
            );
            let (a, end, v) = d.velocity;
            if r["vel"] == "stationary" {
                assert_eq!(v, Velocity::Stationary);
                assert_eq!(end - a, 1);
            } else {
                let Velocity::Directed {
                    direction_code,
                    magnitude_code,
                    ..
                } = v
                else {
                    panic!("lost projectile velocity")
                };
                assert_eq!(end - a, 30);
                assert_eq!(direction_code as usize, num(&r["vel"][0]));
                assert_eq!(usize::from(magnitude_code), num(&r["vel"][1]));
            }
            assert!(projectile::record(Bits(&data[..end / 8]), o, layout).is_none());
            let mut corrupt = data.clone();
            flip(&mut corrupt, o + 16); // Unsupported header/baseline.
            assert!(projectile::record(Bits(&corrupt), o, layout).is_none());
        }
    }
}

#[test]
fn velocity_speed_quantizer_has_distinct_zero_and_exact_endpoints() {
    let directed = |q| Velocity::Directed {
        direction_code: 392594,
        direction: [0., -1., 0.],
        magnitude_code: q,
    };
    assert_eq!(Velocity::Stationary.speed(), 0.);
    assert_eq!(Velocity::Stationary.vector(), [0.; 3]);
    assert_eq!(directed(0).speed(), 0.03);
    assert_eq!(directed(1023).speed(), 350.);
    // Captured grenade launch q418; independently checked against position motion.
    assert!((directed(418).speed() - 10.00059).abs() < 0.0001);
    assert_eq!(directed(418).vector(), [0., -directed(418).speed(), 0.]);
    for q in 1..=1023 {
        assert!(directed(q).speed() > directed(q - 1).speed());
    }
}

#[test]
fn map_bounds_predict_bazaar_and_aquarius_coordinate_widths() {
    let aquarius = CoordinateBounds {
        min: [-39.014282, -27.861597, -2.8262208],
        max: [38.79729, 18.353075, 15.290748],
    };
    let bazaar = CoordinateBounds {
        min: [-973.866, -361.43918, -86.55155],
        max: [179.37695, 1047.0077, 489.09164],
    };
    assert_eq!(aquarius.axis_bits(), Some([13, 12, 11]));
    assert_eq!(bazaar.axis_bits(), Some([17, 17, 16]));
    let spawn = aquarius
        .world_position([1980, 2469, 727], CoordinateLayout::X13Y12Z11)
        .unwrap();
    assert!((spawn[0] + 20.2022).abs() < 0.01);
    assert!(
        aquarius
            .world_position([1980, 2469, 727], CoordinateLayout::X15Y15Z17)
            .is_none()
    );
    assert!(
        aquarius
            .world_position([8192, 0, 0], CoordinateLayout::X13Y12Z11)
            .is_none()
    );
    assert!(
        CoordinateBounds {
            min: [0.; 3],
            max: [f32::NAN; 3]
        }
        .axis_bits()
        .is_none()
    );
    assert!(
        CoordinateBounds {
            min: [1.; 3],
            max: [0.; 3]
        }
        .axis_bits()
        .is_none()
    );
}

#[test]
fn projectile_tracks_reject_wrong_owner_generation_and_discontinuous_updates() {
    let f = fixture(include_str!("fixtures/projectile_motion_records.json"));
    let rows = f["records"].as_array().unwrap();
    for invalid in 0..4 {
        let player = PlayerTrack {
            id: 0,
            lives: vec![serde_json::from_value(f["controlled_life"].clone()).unwrap()],
            ..PlayerTrack::default()
        };
        let birth_row = &rows[0];
        let birth_bytes = bytes(birth_row);
        let layout = CoordinateLayout::X15Y15Z17;
        let (mut birth, end) =
            projectile::birth(Bits(&birth_bytes), num(&birth_row["bit"]), layout).unwrap();
        if invalid == 1 {
            birth.player = 1;
        }
        let source = SourceSpan {
            chunk: 2,
            payload_byte: 495715,
            bit: num(&birth_row["bit"]),
            end_bit: end,
        };
        let start = 26_709_491;
        let mut candidates = projectile::Tracks {
            births: vec![Sample {
                time_us: start,
                life: 0,
                source,
                value: birth,
            }],
            ..Default::default()
        };
        for row in rows
            .iter()
            .filter(|r| r["layout"] == "X15Y15Z17" && r["kind"] == "delta")
            .take(2)
        {
            let data = bytes(row);
            let mut record = projectile::record(Bits(&data), num(&row["bit"]), layout).unwrap();
            if invalid == 2 {
                record.generation = 2;
            }
            let time_us = (row["time"].as_f64().unwrap() * 1e6).round() as u64
                + if invalid == 3 { 1_000_000 } else { 0 };
            candidates.records.push(Sample {
                time_us,
                life: 0,
                source,
                value: record,
            });
        }
        let tracks = candidates.finish(&[player]);
        assert_eq!(tracks.len(), usize::from(invalid == 0));
        if invalid == 0 {
            assert_eq!(tracks[0].positions.len(), 3);
            assert!(tracks[0].terminal.is_none());
        }
    }
}
