use super::*;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
trait FromOracle: Sized {
    fn from_oracle(v: &Value) -> Self;
}
fn parse<T: FromOracle>(v: &Value) -> T {
    T::from_oracle(v)
}
macro_rules! scalar {($($t:ty),*)=>{$(impl FromOracle for $t{fn from_oracle(v:&Value)->Self{serde_json::from_value(v.clone()).unwrap()}})*};}
scalar!(bool, u64, u32, u8, i64, i32, i8);
impl FromOracle for f32 {
    fn from_oracle(v: &Value) -> Self {
        Self::from_bits(v.as_u64().unwrap() as u32)
    }
}
impl<T: FromOracle> FromOracle for Option<T> {
    fn from_oracle(v: &Value) -> Self {
        if v.is_null() { None } else { Some(parse(v)) }
    }
}
impl<T: FromOracle> FromOracle for Vec<T> {
    fn from_oracle(v: &Value) -> Self {
        v.as_array().unwrap().iter().map(parse).collect()
    }
}
impl<T: FromOracle, const N: usize> FromOracle for [T; N] {
    fn from_oracle(v: &Value) -> Self {
        std::array::from_fn(|i| parse(&v[i]))
    }
}
impl FromOracle for BTreeMap<u32, i64> {
    fn from_oracle(v: &Value) -> Self {
        v.as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.parse().unwrap(), parse(v)))
            .collect()
    }
}
impl FromOracle for BTreeMap<Vec<u8>, i64> {
    fn from_oracle(v: &Value) -> Self {
        v.as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (unhex(k), parse(v)))
            .collect()
    }
}
macro_rules! convert {($name:ty{$($field:ident=>$native:literal),*$(,)?})=>{impl FromOracle for $name{fn from_oracle(v:&Value)->Self{Self{$($field:parse(&v[$native]),)*}}}};}
convert!(FactsPrecisionDescriptor{index_width=>"IndexW",axis_widths=>"AxisW",region=>"Region"});
convert!(FactsAxisRange{min=>"Min",max=>"Max"});
convert!(FactsMovementProfile{traversal=>"Traversal",world_object=>"WorldObject",delta_quantum=>"DeltaQuantum",delta_axis_width=>"DeltaAxisWidth",range=>"Range",full_precision=>"FullPrecision",delta_has_handle_tail=>"DeltaHasHandleTail",calibrated_skip=>"CalibratedSkip",mobility_action_extra_bits=>"MobilityActionExtraBits"});
convert!(FactsKeyframeProfile{header_bits=>"EnTeteBits",size_word_bits=>"MotDeTailleBits"});
convert!(FactsMppWidths{lead=>"Lead",index=>"Index"});
convert!(FactsScanGrammar{corruption_check=>"ControleDeCorruption",new_record_tail_bits=>"BitsDeQueueRecordNew",default_state_by_archetype=>"DeserEtatParArchetype",simulation_complete=>"SimStateComplet",baseline_scope=>"PorteeBaseline",writer_absolute=>"GrammaireEcrivainI0",mobility_action_body=>"CorpsActionMobilite",ability_anchor_body=>"CorpsAncrageCapacite",chain_inference=>"InferenceChaine",calibrated_widths=>"LargeursCalibrees",generation_strict=>"GenerationStricte",view_tables=>"TablesParVue",view_classes=>"ClassesDeVue",stub_widths=>"LargeursBouchon"});
convert!(FactsScanProfile{movement=>"Mouvement",keyframe=>"Cadre",mpp=>"MPP",grammar=>"Grammaire"});
convert!(FactsDeathFrame{has_extra_fields=>"HasExtraFields",id_low_bits=>"IDLowBits",id_base=>"IDBase",new_default_state_bits=>"NewDefaultStateBits",packet_preamble_bits=>"PacketPreambleBits",profile=>"Profil"});
convert!(FactsObjectDeathStats{default_frame=>"CadreParDefaut",frame_located=>"CadreLocalises",frame_runner=>"CadreDauphin",frame_events=>"CadreEvenements",keyframes=>"Keyframes",deltas=>"Deltas",packets=>"Packets",event_packets=>"EventPackets",located_packets=>"LocatedPackets",records=>"Records",clean_records=>"CleanRecords",mask_declared=>"MaskDeclared",mask_declared_desync=>"MaskDeclaredDesync"});
convert!(FactsObjectDeaths{deaths=>"Morts",stats=>"Stats",frame=>"Cadre"});
convert!(ObjectDeath{ timestamp_us=>"TimestampUS",slot=>"Slot",r#gen=>"Gen",type_index=>"TypeIndex",dead=>"Dead",tail_desync=>"TailDesync" });
convert!(ObjectDeadState{ mort=>"Mort",enum_a=>"EnumA",enum_b=>"EnumB",val_0c=>"Val0c",val_0e=>"Val0e",has_ref=>"HasRef",gid_present=>"GIDPresent",global_id=>"GlobalID",val14=>"Val14",val18=>"Val18",src_tag0=>"SrcTag0",src_tag4c=>"SrcTag4c" });
#[test]
fn native_facts_object_deaths_writer() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-object-deaths-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 2304);
    for (i, row) in rows.iter().enumerate() {
        let source: FactsObjectDeaths = parse(&row["source"]);
        let mut w = NativeFactsWriter::default();
        w.byte(0x55);
        w.byte(0x11);
        if row["prior"].as_bool().unwrap() {
            w.fail("prior");
        }
        encode_facts_object_deaths(&mut w, &source);
        assert_eq!(
            w.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
        assert_eq!(
            w.bytes(),
            unhex(row["encoded_hex"].as_str().unwrap()),
            "bytes {i}: {}",
            String::from_utf8_lossy(w.bytes())
        );
    }
}

#[test]
fn native_facts_json_float32_wire() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-json-floats-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 68608);
    for (i, row) in rows.iter().enumerate() {
        let mut source = FactsObjectDeaths::default();
        source.frame.profile.movement.delta_quantum =
            f32::from_bits(row["bits"].as_u64().unwrap() as u32);
        let mut w = NativeFactsWriter::default();
        encode_facts_object_deaths(&mut w, &source);
        let error = row["error"].as_str().unwrap();
        if error.is_empty() {
            assert!(w.error().is_none(), "error {i}");
            let text = String::from_utf8_lossy(w.bytes());
            let value = text
                .split("\"DeltaQuantum\":")
                .nth(1)
                .unwrap()
                .split(',')
                .next()
                .unwrap();
            assert_eq!(
                value,
                row["json"].as_str().unwrap(),
                "float case {i}, bits {}",
                row["bits"]
            );
        } else {
            assert_eq!(
                w.error(),
                Some(format!("morts d objet non serialisables : {error}").as_str())
            );
            assert_eq!(w.bytes(), [0]);
        }
    }
}

#[test]
fn native_facts_object_death_decoder() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-object-death-decode-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 6450);
    for (i, row) in rows.iter().enumerate() {
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut r = NativeFactsReader::new(&input);
        let actual = decode_facts_object_deaths(&mut r);
        let expected: FactsObjectDeaths = parse(&row["decoded"]);
        assert_eq!(
            r.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}, input {}",
            String::from_utf8_lossy(&input)
        );
        assert_eq!(
            r.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "cursor {i}"
        );
        assert_eq!(actual, expected, "values {i}");
        let floats = |v: &FactsObjectDeaths| {
            let m = &v.frame.profile.movement;
            [
                m.delta_quantum.to_bits(),
                m.range[0].min.to_bits(),
                m.range[0].max.to_bits(),
                m.range[1].min.to_bits(),
                m.range[1].max.to_bits(),
                m.range[2].min.to_bits(),
                m.range[2].max.to_bits(),
            ]
        };
        assert_eq!(floats(&actual), floats(&expected), "float bits {i}");
    }
}

#[test]
fn native_facts_json_syntax() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-json-syntax-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 3783);
    for (i, row) in rows.iter().enumerate() {
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut r = NativeFactsReader::new(&input);
        let _ = decode_facts_object_deaths(&mut r);
        assert_eq!(
            r.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}: {}",
            String::from_utf8_lossy(&input[..input.len().min(120)])
        );
        assert_eq!(
            r.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "offset {i}"
        );
    }
}

fn vehicle_creation(v: &Value) -> FactsEquipmentCreation {
    FactsEquipmentCreation {
        timestamp_us: v["TimestampUS"].as_u64().unwrap(),
        slot: v["Slot"].as_u64().unwrap() as u32,
        generation: v["Gen"].as_u64().unwrap() as u32,
        chunk: v["Chunk"].as_i64().unwrap(),
        packet_index: v["PacketIndex"].as_i64().unwrap(),
        bit_pos: v["BitPos"].as_i64().unwrap(),
        has_ref: v["HasRef"].as_bool().unwrap(),
        reference: v["Ref"].as_u64().unwrap() as u32,
        has_id: v["HasID"].as_bool().unwrap(),
        ability_id: v["AbilityID"].as_u64().unwrap() as u32,
        mpp_present: serde_json::from_value(v["MPPPresent"].clone()).unwrap(),
        mpp_val: serde_json::from_value(v["MPPVal"].clone()).unwrap(),
        position: ["X", "Y", "Z"].map(|key| f32::from_bits(v[key].as_u64().unwrap() as u32)),
        mask: serde_json::from_value(v["Mask"].clone()).unwrap(),
        mask_full: v["MaskFull"].as_bool().unwrap(),
        mask_has_i0: v["MaskHasI0"].as_bool().unwrap(),
        default_state_bits: v["DefaultStateBits"].as_i64().unwrap(),
        has_ammo: v["HasAmmo"].as_bool().unwrap(),
        ammo: serde_json::from_value(v["Ammo"].clone()).unwrap(),
        after_bit: v["AfterBit"].as_i64().unwrap(),
    }
}

fn vehicle_positions(v: &Value) -> Vec<FactsBipedPosition> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| FactsBipedPosition {
            timestamp_us: p["timestamp_us"].as_u64().unwrap(),
            slot: p["slot"].as_u64().unwrap() as u32,
            has_world: p["has_world"].as_bool().unwrap(),
            quantized: serde_json::from_value(p["quantized"].clone()).unwrap(),
            world: std::array::from_fn(|i| {
                f32::from_bits(p["world_bits"][i].as_u64().unwrap() as u32)
            }),
            has_yaw: p["has_yaw"].as_bool().unwrap(),
            yaw_raw: p["yaw_raw"].as_u64().unwrap() as u32,
            pitch_raw: p["pitch_raw"].as_u64().unwrap() as u32,
            directions: serde_json::from_value(p["directions"].clone()).unwrap(),
            has_body: p["has_body"].as_bool().unwrap(),
            health: f32::from_bits(p["health_bits"].as_u64().unwrap() as u32),
            has_shield: p["has_shield"].as_bool().unwrap(),
            shield: f32::from_bits(p["shield_bits"].as_u64().unwrap() as u32),
            shield_quantum: p["shield_quantum"].as_u64().unwrap() as u8,
        })
        .collect()
}

fn vehicle_scan(v: &Value) -> FactsVehicleScan {
    let k = &v["keyframes"];
    FactsVehicleScan {
        scanned: v["scanned"].as_bool().unwrap(),
        keyframes: FactsWorldKeyframes {
            times_us: serde_json::from_value(k["times"].clone()).unwrap(),
            seen_us: k["seen"]
                .as_array()
                .unwrap()
                .iter()
                .map(|life| {
                    (
                        (
                            life["slot"].as_u64().unwrap() as u32,
                            life["generation"].as_u64().unwrap() as u32,
                        ),
                        serde_json::from_value(life["times"].clone()).unwrap(),
                    )
                })
                .collect(),
        },
        creations: v["creations"]
            .as_array()
            .unwrap()
            .iter()
            .map(vehicle_creation)
            .collect(),
        stats: serde_json::from_value(v["stats"].clone()).unwrap(),
        positions: vehicle_positions(&v["positions"]),
        events: serde_json::from_value(v["events"].clone()).unwrap(),
        aims: serde_json::from_value(v["aims"].clone()).unwrap(),
        deaths: parse(&v["deaths"]),
        occupancy: serde_json::from_value(v["occupancy"].clone()).unwrap(),
    }
}
#[test]
fn native_facts_vehicle_scan() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-vehicles-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 8342);
    let layout = I0Layout {
        gate_bits: 0,
        axis_widths: [13, 14, 15],
        region: 0,
    };
    let bounds = [[-100., -200., -300.], [200., 300., 400.]];
    for (i, row) in rows.iter().enumerate() {
        let source = vehicle_scan(&row["source"]);
        let mut w = NativeFactsWriter::default();
        encode_facts_vehicle_scan(&mut w, &source);
        assert!(w.error().is_none());
        assert_eq!(
            w.bytes(),
            unhex(row["encoded_hex"].as_str().unwrap()),
            "encode {i}"
        );
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut r = NativeFactsReader::new(&input);
        let actual = decode_facts_vehicle_scan(&mut r, &layout, bounds);
        let expected = vehicle_scan(&row["decoded"]);
        assert_eq!(
            r.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
        assert_eq!(
            r.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "offset {i}"
        );
        assert_eq!(actual, expected, "scan {i}");
        let floats = |v: &FactsVehicleScan| {
            v.positions
                .iter()
                .flat_map(|p| p.world.into_iter().chain([p.health, p.shield]))
                .chain(v.creations.iter().flat_map(|c| c.position))
                .map(f32::to_bits)
                .collect::<Vec<_>>()
        };
        assert_eq!(floats(&actual), floats(&expected), "float bits {i}");
    }
}
