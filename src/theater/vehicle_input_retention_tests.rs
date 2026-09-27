//! Exact native vehicle-input projection before replay assembly.
use super::film_input_retention_tests::{
    first_difference, native_census, native_position, normalize_empty, normalize_xyz,
};
use super::*;
use serde_json::{Value, json};

pub(super) fn native_vehicle_input(film: &LegacyFilm) -> Value {
    let empty = FilmVehicleFacts::default();
    let facts = film
        .native_vehicles
        .as_ref()
        .filter(|f| f.scanned)
        .unwrap_or(&empty);
    let march = film.native_march_facts.as_ref().filter(|_| facts.scanned);
    let creations = facts.creations.as_ref();
    json!({
        "Scanned":facts.scanned,"Keyframes":native_census(&facts.keyframes),
        "Creations":creations.map(|c| c.records.clone()).unwrap_or_default(),
        "Stats":creations.map(|c| c.stats.clone()).unwrap_or_default(),
        "Positions":facts.positions.as_ref().map(|p| p.accepted().map(native_position).collect::<Vec<_>>()).unwrap_or_default(),
        "Events":facts.events,"Aims":facts.aims,
        "Deaths":march.map(|m| m.facts.deaths.iter().filter(|d|d.type_index==40).collect::<Vec<_>>()).unwrap_or_default(),
        "Occupancy":march.map(|m| m.facts.occupancy.clone()).unwrap_or_default(),
        "DeathStats":native_march_stats(march),
    })
}
fn native_march_stats(march: Option<&FilmMarchFacts>) -> Value {
    let empty = FilmMarchFacts::default();
    let m = march.unwrap_or(&empty);
    let c = m.calibration.as_ref();
    json!({
        "Config":native_config(m.native_config.as_ref()),
        "CadreParDefaut":c.is_some_and(|c|c.retained_default),
        "CadreLocalises":c.map_or(0, |c|c.best.located),
        "CadreDauphin":c.map_or(0, |c|c.runner_up.located),
        "CadreEvenements":c.map_or(0, |c|c.best.events),
        "Keyframes":m.keyframes,"Deltas":m.deltas,"Packets":m.packets,
        "EventPackets":m.event_packets,"LocatedPackets":m.located_packets,
        "Records":m.facts.coverage.records,"CleanRecords":m.facts.coverage.clean_records,
        "MaskDeclared":m.facts.coverage.mask_declared,"MaskDeclaredDesync":m.facts.coverage.mask_declared_desync,
    })
}
fn native_config(config: Option<&NativeFrameMetadata>) -> Value {
    let default = NativeFrameConfig::default().snapshot();
    let c = config.unwrap_or(&default);
    let m = &c.profile.movement;
    let g = &c.profile.grammar;
    let descriptor = |d: &NativePrecisionDescriptor| json!({"IndexW":d.index_bits,"AxisW":d.axis_bits,"Region":d.region});
    let movement = json!({"Traversal":descriptor(&m.traversal),"WorldObject":descriptor(&m.world_object),
        "DeltaQuantum":m.delta_quantum,"DeltaAxisWidth":m.delta_axis_width,
        "Range":m.range.map(|r|json!({"Min":r[0],"Max":r[1]})),
        "FullPrecision":m.full_precision,"DeltaHasHandleTail":m.delta_has_handle_tail,
        "CalibratedSkip":m.calibrated_skip,"MobilityActionExtraBits":m.mobility_action_extra_bits});
    let grammar = json!({"ControleDeCorruption":g.corruption_check,"BitsDeQueueRecordNew":g.new_record_tail_bits,
        "DeserEtatParArchetype":g.default_state_by_archetype,"SimStateComplet":g.simulation_complete,
        "PorteeBaseline":g.baseline_scope,"GrammaireEcrivainI0":g.writer_absolute,
        "CorpsActionMobilite":g.mobility_action_body,"CorpsAncrageCapacite":g.ability_anchor_body,
        "InferenceChaine":g.chain_inference,"LargeursCalibrees":g.calibrated_widths,
        "GenerationStricte":g.generation_strict,"TablesParVue":g.view_tables,
        "ClassesDeVue":g.view_classes,"LargeursBouchon":g.stub_widths});
    assert!(
        !c.observer_present,
        "native vehicle scan excludes observers"
    );
    let mut value = json!({"HasExtraFields":c.extra_fields,"IDLowBits":c.id_low_bits,"IDBase":c.id_base,
        "NewDefaultStateBits":c.new_default_state_bits,"PacketPreambleBits":c.packet_preamble_bits,"Obs":null,
        "Profil":{"Mouvement":movement,"Grammaire":grammar,"MPP":c.profile.mpp,
            "Cadre":{"EnTeteBits":c.profile.keyframe.header_bits,"MotDeTailleBits":c.profile.keyframe.size_word_bits}}});
    if config.is_none() {
        // Go returns the zero-value Config on the pre-scan path. Rust retains
        // absence instead. Preserve the complete native schema while projecting
        // that explicit absence; never substitute the configured Rust defaults.
        fn zero(v: &mut Value) {
            match v {
                Value::Number(_) => *v = json!(0),
                Value::Bool(_) => *v = json!(false),
                Value::Array(a) => a.iter_mut().for_each(zero),
                Value::Object(o) => o.values_mut().for_each(zero),
                Value::Null => {}
                _ => panic!("unexpected config field"),
            }
        }
        zero(&mut value);
    }
    value
}
pub(super) fn normalize_vehicle(native: &mut Value, actual: &Value) {
    normalize_xyz(&mut native["Creations"]);
    normalize_xyz(&mut native["Positions"]);
    if let Some(rows) = native["Positions"].as_array_mut() {
        for row in rows {
            for (outer, inner) in [("Body", "Health"), ("Shield", "Shield")] {
                row[outer][inner] = json!(row[outer][inner].as_f64().unwrap() as f32);
            }
        }
    }
    // Absent config projects integer zero above; retained configs use float32.
    if actual["DeathStats"]["Config"]["Profil"]["Mouvement"]["DeltaQuantum"].is_f64() {
        let m = &mut native["DeathStats"]["Config"]["Profil"]["Mouvement"];
        m["DeltaQuantum"] = json!(m["DeltaQuantum"].as_f64().unwrap() as f32);
        for axis in m["Range"].as_array_mut().unwrap() {
            for key in ["Min", "Max"] {
                axis[key] = json!(axis[key].as_f64().unwrap() as f32);
            }
        }
    }
}
pub(super) fn assert_native_vehicle_inputs(film: &LegacyFilm, mut expected: Value) {
    let actual = native_vehicle_input(film);
    normalize_empty(&mut expected, &actual);
    normalize_vehicle(&mut expected, &actual);
    assert!(
        actual == expected,
        "{}",
        first_difference(&actual, &expected, "$Vehicles")
    );
}
