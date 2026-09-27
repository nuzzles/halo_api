//! Compare native mutable-reader transitions with Rust's per-attempt context.
use super::*;
use crate::theater::{
    FilmComponentObservation, NativeFilmObserver, NativeFilmReader, NativeHookKind,
    NativeHookPublication, NativeReaderContext, NativeScanProfile,
};
use std::io::Read;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
struct Case {
    hex: String,
    start: usize,
    name: String,
    steps: Vec<Step>,
    detached: Detached,
}
#[derive(Deserialize)]
struct Detached {
    end: usize,
    ported: bool,
    ordered: [Vec<ExpectedPublication>; 2],
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum ExpectedPublication {
    MobilityAction([bool; 2]),
    Component(FilmComponentObservation),
}

#[derive(Deserialize)]
struct Step {
    ordered: [Vec<ExpectedPublication>; 2],
    encoding: PositionEncoding,
    active: usize,
    suppressed: bool,
    end: usize,
    ported: bool,
    observations: [Vec<FilmComponentObservation>; 2],
    actions: [Vec<[bool; 2]>; 2],
}

#[test]
fn native_profile_observer_replacement_and_restoration() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("../fixtures/reader-sequence-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1064);
    let mut merged = crate::theater::FilmReadDiagnostics::default();
    let mut expected_merged = Vec::new();
    let mut rejected = 0;
    let mut rejected_callbacks = 0;
    let mut profile_width_changes = 0;
    for (i, case) in cases.iter().enumerate() {
        let bytes: Vec<u8> = (0..case.hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&case.hex[i..i + 2], 16).unwrap())
            .collect();
        assert_eq!(case.steps.len(), 6);
        let streams: [Arc<Mutex<Vec<ExpectedPublication>>>; 2] =
            std::array::from_fn(|_| Arc::new(Mutex::new(Vec::new())));
        let observers: [NativeFilmObserver; 2] = std::array::from_fn(|index| {
            let observer = NativeFilmObserver::default();
            // Exactly the named hooks installed by haloRustComponentHooks,
            // plus the sequence harness's movement and mobility hooks.
            for kind in [
                NativeHookKind::UnitReference,
                NativeHookKind::Mpp,
                NativeHookKind::EquipmentCreation,
                NativeHookKind::EquipmentState,
                NativeHookKind::Probe,
                NativeHookKind::PlayerState,
                NativeHookKind::GameEngine,
                NativeHookKind::ManagedObject,
                NativeHookKind::Navpoint,
                NativeHookKind::Objective,
                NativeHookKind::ManagedProperty,
                NativeHookKind::HeldWeapon,
                NativeHookKind::UnitEquipment,
                NativeHookKind::ObjectParent,
                NativeHookKind::CamoState,
                NativeHookKind::SpartanAbility,
                NativeHookKind::AbilityNonPredicted,
                NativeHookKind::AbilityEnergy,
                NativeHookKind::GrenadeSet,
                NativeHookKind::AbilitySet,
                NativeHookKind::EmpTimer,
                NativeHookKind::GrenadeCounts,
                NativeHookKind::WeaponAmmo,
                NativeHookKind::WeaponRounds,
                NativeHookKind::DesiredWeaponSet,
                NativeHookKind::GroundWeaponAmmo,
                NativeHookKind::MovementState,
                NativeHookKind::MobilityAction,
            ] {
                let stream = streams[index].clone();
                observer.set_hook(
                    kind,
                    Some(Arc::new(move |publication| {
                        let value = match publication {
                            NativeHookPublication::Component(value) => {
                                ExpectedPublication::Component(value.clone())
                            }
                            NativeHookPublication::MobilityAction(value) => {
                                ExpectedPublication::MobilityAction(value)
                            }
                            NativeHookPublication::RecordMask { .. }
                            | NativeHookPublication::ControlView(_) => {
                                panic!("unexpected record hook")
                            }
                        };
                        stream.lock().unwrap().push(value);
                    })),
                );
            }
            observer
        });
        let mut profile_a = NativeScanProfile::default();
        profile_a.grammar.ability_anchor_body = false;
        profile_a.grammar.mobility_action_body = false;
        let mut profile_b = profile_a.clone();
        profile_b.grammar.ability_anchor_body = true;
        profile_b.grammar.mobility_action_body = true;
        profile_b.movement.mobility_action_extra_bits = 17;
        profile_b.movement.full_precision = i.is_multiple_of(3);
        let mut live = NativeFilmReader::with_context(
            &bytes,
            NativeReaderContext {
                profile: profile_a.clone(),
                observer: Some(observers[0].clone()),
            },
        );
        let mut restoration = None;
        let mut previous_profile = None;
        let mut previous_observer = None;
        let mut results = Vec::new();
        for (j, step) in case.steps.iter().enumerate() {
            let old_bit = live.bit_position();
            match j {
                1 => restoration = Some(observers[0].neutralize_captures()),
                2 => {
                    previous_profile = Some(live.replace_profile(profile_b.clone()));
                    previous_observer = live.replace_observer(Some(observers[1].clone()));
                    assert_eq!(previous_profile.as_ref().unwrap(), &profile_a);
                    assert!(
                        previous_observer
                            .as_ref()
                            .unwrap()
                            .same_instance(&observers[0])
                    );
                }
                3 => {
                    assert_eq!(
                        live.replace_profile(previous_profile.take().unwrap()),
                        profile_b
                    );
                    assert!(
                        live.replace_observer(previous_observer.take())
                            .unwrap()
                            .same_instance(&observers[1])
                    );
                }
                4 => restoration.take().unwrap().restore(),
                5 => {
                    let old = live.replace_context(NativeReaderContext {
                        profile: profile_b.clone(),
                        observer: Some(observers[1].clone()),
                    });
                    assert_eq!(old.profile, profile_a);
                    assert!(old.observer.unwrap().same_instance(&observers[0]));
                }
                _ => (),
            }
            assert_eq!(
                live.bit_position(),
                old_bit,
                "replacement must preserve cursor"
            );
            assert_eq!(live.capture_slot(), 0);
            assert_eq!(
                live.profile(),
                if step.active == 0 {
                    profile_a.clone()
                } else {
                    profile_b.clone()
                }
            );
            for stream in &streams {
                stream.lock().unwrap().clear();
            }
            live.set_bit_position(case.start);
            let (live_status, live_component) = live.read_component(&case.name, 0, 35).unwrap();
            assert_eq!(live_status, Some(step.ported), "live status {i}/{j}");
            assert_eq!(live.bit_position(), step.end, "live cursor {i}/{j}");
            for (receiver, stream) in streams.iter().enumerate() {
                assert_eq!(
                    *stream.lock().unwrap(),
                    step.ordered[receiver],
                    "live callbacks {i}/{j}/{receiver}"
                );
            }
            let (status, component) = decode_native_component_with_capture(
                &bytes,
                case.start,
                &case.name,
                0,
                35,
                Some(&step.encoding),
                NativeComponentCapture {
                    position: None,
                    movement_slot: (!step.suppressed).then_some(0),
                    unit_references: !step.suppressed,
                },
            );
            assert_eq!(
                live_component.fields, component.fields,
                "live fields {i}/{j}"
            );
            assert_eq!(status, Some(step.ported), "status {i}/{j}");
            assert_eq!(
                serde_json::json!(component.end_bit),
                serde_json::json!(step.end),
                "end {i}/{j}"
            );
            assert_eq!(
                component.diagnostics.component_observations, step.observations[step.active],
                "callbacks {i}/{j}"
            );
            assert_eq!(
                component.diagnostics.mobility_actions, step.actions[step.active],
                "actions {i}/{j}"
            );
            merged.merge(&component.diagnostics);
            expected_merged.extend(step.ordered[step.active].iter().cloned());
            assert_eq!(
                serde_json::to_value(
                    component
                        .diagnostics
                        .ordered_publications()
                        .unwrap()
                        .collect::<Vec<_>>()
                )
                .unwrap(),
                serde_json::json!(step.ordered[step.active])
            );
            let publications: Vec<_> = component.ordered_publications().unwrap().collect();
            assert_eq!(
                serde_json::to_value(publications).unwrap(),
                serde_json::json!(step.ordered[step.active]),
                "publication order {i}/{j}"
            );
            assert!(step.ordered[1 - step.active].is_empty());
            assert!(step.observations[1 - step.active].is_empty());
            assert!(step.actions[1 - step.active].is_empty());
            if !step.ported {
                rejected += 1;
                rejected_callbacks += component.diagnostics.component_observations.len();
            }
            let restored: DecodedComponent =
                serde_json::from_value(serde_json::to_value(&component).unwrap()).unwrap();
            assert_eq!(restored, component);
            results.push(component);
        }
        for stream in &streams {
            stream.lock().unwrap().clear();
        }
        live.set_bit_position(case.start);
        let old_profile = live.profile();
        let detached = live.replace_observer(None).unwrap();
        assert!(detached.same_instance(&observers[1]));
        assert_eq!(live.profile(), old_profile);
        assert_eq!(live.bit_position(), case.start);
        let (status, _) = live.read_component(&case.name, 0, 35).unwrap();
        assert_eq!(status, Some(case.detached.ported), "detached status {i}");
        assert_eq!(live.bit_position(), case.detached.end, "detached end {i}");
        for (receiver, stream) in streams.iter().enumerate() {
            assert_eq!(*stream.lock().unwrap(), case.detached.ordered[receiver]);
            assert!(stream.lock().unwrap().is_empty());
        }
        assert!(live.replace_observer(Some(detached)).is_none());
        assert_eq!(live.profile(), old_profile);
        assert_eq!(live.bit_position(), case.detached.end);
        // Profile/observer restoration must reproduce the original result.
        assert_eq!(results[0], results[4]);
        assert_eq!(results[1], results[3]);
        assert_eq!(results[2], results[5]);
        // Capture suppression cannot affect consumed bits or decoded raw fields.
        assert_eq!(results[0].fields, results[1].fields);
        assert_eq!(results[0].end_bit, results[1].end_bit);
        profile_width_changes += usize::from(results[0].end_bit != results[2].end_bit);
    }
    assert_eq!(
        serde_json::to_value(merged.ordered_publications().unwrap().collect::<Vec<_>>()).unwrap(),
        serde_json::json!(expected_merged)
    );
    let json = serde_json::to_value(&merged).unwrap();
    let restored: crate::theater::FilmReadDiagnostics =
        serde_json::from_value(json.clone()).unwrap();
    assert_eq!(restored, merged);
    let mut old = json;
    old.as_object_mut().unwrap().remove("mobility_offsets");
    let old: crate::theater::FilmReadDiagnostics = serde_json::from_value(old).unwrap();
    assert!(old.ordered_publications().is_none());
    merged.merge(&old);
    assert!(merged.ordered_publications().is_none());
    assert_eq!(rejected, 38);
    assert_eq!(rejected_callbacks, 38);
    assert!(profile_width_changes > 0);
}

#[test]
fn aggregate_mobility_diagnostics_do_not_claim_component_order() {
    let (_, mut component) = decode_native_component_with_capture(
        &[0],
        0,
        "biped-mobility-action-component",
        0,
        35,
        None,
        NativeComponentCapture {
            position: None,
            movement_slot: None,
            unit_references: false,
        },
    );
    assert!(component.ordered_publications().is_some());
    component.diagnostics.mobility_actions.push([false, false]);
    assert!(component.ordered_publications().is_none());
    component.diagnostics.mobility_actions.pop();
    component.name = "unit-crouch-component".into();
    assert!(component.ordered_publications().is_none());
}

#[test]
fn merged_order_survives_suppression_and_detached_resync() {
    use crate::theater::*;
    let position = FilmComponentObservation::Position {
        position_kind: NativePositionKind::Absolute,
        vector_bits: [0; 3],
        bit: 3,
        slot: 7,
    };
    let timer = FilmComponentObservation::EmpTimer { quantum: 5 };
    let mut a = FilmReadDiagnostics::default();
    a.component_observations.push(position);
    a.component_observations
        .push(FilmComponentObservation::UnitReference {
            reference: NativeUnitReference {
                kind: NativeUnitReferenceKind::Word32,
                start_bit: 3,
                end_bit: 35,
                present: true,
                value: 7,
                tail: 0,
                probe: false,
            },
        });
    a.publish_mobility([true, false]);
    a.component_observations.push(timer.clone());
    let mut b = FilmReadDiagnostics::default();
    b.publish_mobility([false, true]);
    b.component_observations.push(timer);
    a.merge(&b);
    assert_eq!(a.mobility_offsets, Some(vec![2, 3]));
    a.suppress_positions();
    assert_eq!(a.mobility_offsets, Some(vec![1, 2]));
    a.suppress_unit_references();
    assert_eq!(a.mobility_offsets, Some(vec![0, 1]));
    let actual =
        serde_json::to_value(a.ordered_publications().unwrap().collect::<Vec<_>>()).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([
            {"kind":"mobility_action","value":[true,false]},
            {"kind":"component","value":{"kind":"emp_timer","quantum":5}},
            {"kind":"mobility_action","value":[false,true]},
            {"kind":"component","value":{"kind":"emp_timer","quantum":5}}
        ])
    );
    let mut resync = RawResyncDiagnostics::default();
    resync.absorb_scan(&a);
    assert_eq!(resync.caller_visible.mobility_offsets, a.mobility_offsets);
    assert_eq!(
        serde_json::to_value(
            resync
                .caller_visible
                .ordered_publications()
                .unwrap()
                .collect::<Vec<_>>()
        )
        .unwrap(),
        actual
    );
    assert!(resync.detached_scans.mobility_offsets.is_none());
}

#[test]
fn malformed_merge_order_cannot_become_known() {
    use crate::theater::*;
    for offsets in [vec![10], vec![1, 0], vec![0, 0, 0], vec![usize::MAX, 0]] {
        let mut a = FilmReadDiagnostics::default();
        a.component_observations
            .push(FilmComponentObservation::EmpTimer { quantum: 1 });
        a.mobility_actions = vec![[false, false]; 2];
        a.mobility_offsets = Some(offsets);
        assert!(a.ordered_publications().is_none());
        let b = FilmReadDiagnostics {
            component_observations: vec![FilmComponentObservation::EmpTimer { quantum: 2 }; 20],
            ..Default::default()
        };
        let mut incoming = b.clone();
        incoming.merge(&a);
        assert!(incoming.ordered_publications().is_none());
        a.merge(&b);
        assert!(a.ordered_publications().is_none());
    }
}
