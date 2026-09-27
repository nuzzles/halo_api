use super::*;
use crate::theater::{FilmArchetype, GroundWeaponAmmo};
use std::io::Read;

#[test]
fn native_ground_ammo_profile() {
    #[derive(Deserialize)]
    struct Case {
        hex: String,
        start: usize,
        mask: Vec<usize>,
        names: Vec<String>,
        widths: ComponentWidthOverrides,
        simulation: bool,
        corruption: bool,
        ok: bool,
        ammo: GroundWeaponAmmo,
    }
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("../fixtures/ground-ammo-profile-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 512);
    let mut position = crate::theater::film_map_catalog().maps["bazaar"].position_encoding();
    // Native ContexteParDefaut's world descriptor, not Bazaar's map precision.
    position.index_bits = 1;
    position.world_axis_bits = Some([13, 13, 14]);
    position.region_axis_bits = [(0, [13, 13, 14])].into();
    for (i, c) in cases.into_iter().enumerate() {
        let data: Vec<u8> = c
            .hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
            .collect();
        let arch = FilmArchetype {
            index: 42,
            levels: vec![0; c.names.len()],
            components: c.names,
        };
        let policy = (&c.widths, c.simulation);
        let actual = read_ground_weapon_ammo(
            &data,
            c.start,
            &c.mask,
            &arch,
            &position,
            c.corruption,
            Some(policy),
        );
        assert_eq!(actual, c.ok.then_some(c.ammo), "case {i}");
    }
}

#[test]
fn native_ground_creation_profile() {
    use crate::theater::*;
    #[derive(Deserialize)]
    struct Case {
        map: FilmMapBounds,
        index: usize,
        region: u32,
        axes: [usize; 3],
        widths: [usize; 2],
        hex: String,
        names: Vec<String>,
        component_widths: ComponentWidthOverrides,
        simulation: bool,
        corruption: bool,
        records: Vec<EquipmentCreation>,
        stats: EquipmentCreationStats,
    }
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("../fixtures/ground-creation-profile-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 512);
    let mut changed = 0;
    let mut ammo = 0;
    let mut mpp_ammo = 0;
    for (i, c) in cases.into_iter().enumerate() {
        let data: Vec<u8> = c
            .hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
            .collect();
        let arch = FilmArchetype {
            index: 42,
            levels: vec![0; c.names.len()],
            components: c.names,
        };
        let mut position = c.map.position_encoding();
        position.index_bits = c.index;
        position.world_axis_bits = Some(c.axes);
        position.region_axis_bits = [(c.region, c.axes)].into();
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: Some(c.simulation),
            native_id_low_bits: None,
            component_widths: c.component_widths,
            new_record: Default::default(),
            position_capture: None,
            ids: RecordIdLayout {
                low_bits: 13,
                base: 0,
            },
            mpp_widths: c.widths,
            position: Some(position.clone()),
            extra_fields: false,
            corruption_check: c.corruption,
        };
        let actual = decode_ground_weapon_creations_with_encoding(
            &data,
            &[512].into(),
            &arch,
            &c.map,
            &encoding,
        );
        assert_eq!(actual.records, c.records, "records {i}");
        assert_eq!(actual.stats, c.stats, "stats {i}");
        assert_eq!(
            serde_json::from_value::<EquipmentCreationStream>(
                serde_json::to_value(&actual).unwrap()
            )
            .unwrap(),
            actual
        );
        let inherited_missing = decode_ground_weapon_creations_with_position(
            &data,
            &[512].into(),
            &arch,
            c.widths,
            &c.map,
            c.corruption,
            &position,
        );
        changed += usize::from(inherited_missing != actual);
        ammo += actual.records.iter().filter(|r| r.has_ammo).count();
        mpp_ammo += actual
            .records
            .iter()
            .filter(|r| r.has_ammo && r.mask.contains(&9))
            .count();
    }
    assert!(changed > 0 && ammo > 0);
    assert!(mpp_ammo > 0, "must traverse MPP before reaching ammo");
    println!(
        "ground creation profile: {changed} changed cases, {ammo} ammo records, {mpp_ammo} with MPP"
    );
}
