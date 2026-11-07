use super::*;
use crate::theater::parser::v41::chunks::replication::components::{
    cursor::ComponentCursor, dispatch, reader::ComponentReader,
};

fn source() -> PoseSource {
    PoseSource {
        chunk_index: Some(1),
        payload_byte: 0,
        start_bit: 0,
        end_bit: 100,
    }
}
fn field(name: &str, width: usize, raw: u64) -> ComponentField {
    ComponentField {
        name: name.into(),
        bit: 0,
        width,
        raw: RawBits::from_low(raw, width),
    }
}
fn record(components: Vec<EntityComponentRead>) -> EntityRecord {
    EntityRecord {
        header: RecordHeader {
            prefix: None,
            kind: RecordKind::Delta,
            id: Some(1),
            start_bit: 0,
            end_bit: 16,
        },
        archetype: None,
        mask: None,
        fields: vec![],
        default_state: None,
        components,
        end_bit: 100,
        stop: EntityViewStop::Complete,
    }
}
fn component(index: usize, name: &str, fields: Vec<ComponentField>) -> EntityComponentRead {
    let weapon_id = if name == "weapon-state-type-info"
        && fields
            .first()
            .is_some_and(|f| f.name == "present" && f.raw.low_u64() == 1)
    {
        let high = fields
            .iter()
            .find(|f| f.name == "id_high")
            .unwrap()
            .raw
            .low_u64() as u32;
        let low = fields
            .iter()
            .find(|f| f.name == "variant")
            .unwrap()
            .raw
            .low_u64() as u32;
        Some(Weapon::from(WeaponId::from_parts(high, low)))
    } else {
        None
    };
    EntityComponentRead {
        index,
        name: name.into(),
        start_bit: 16,
        end_bit: 100,
        status: ComponentReadStatus::Complete,
        fields,
        weapon_id,
    }
}
fn observation(time: u64, change: WeaponChange) -> WeaponObservation {
    WeaponObservation {
        timestamp_us: time,
        entity_id: 1,
        source: source(),
        change,
    }
}
fn selection(slot: usize) -> WeaponChange {
    WeaponChange::Selection {
        principal: Some(slot),
        offhand: None,
    }
}
fn inventory() -> Vec<WeaponObservation> {
    let mut obs: Vec<_> = [
        Some(Weapon::from(WeaponId(0xfedcba9876543210))),
        Some(Weapon::from(WeaponId(0x123456789abcdef0))),
        None,
        None,
    ]
    .into_iter()
    .enumerate()
    .map(|(slot, asset_id)| observation(100, WeaponChange::Slot { slot, asset_id }))
    .collect();
    obs.push(observation(100, selection(0)));
    obs
}

#[test]
fn captured_switch_fields_select_slot_not_request_counter() {
    for (bytes, bit, slot) in [
        (
            &include_bytes!("../../../fixtures/weapon-selection-v41/br-to-shock.bin")[..],
            64,
            1,
        ),
        (
            &include_bytes!("../../../fixtures/weapon-selection-v41/shock-to-br.bin")[..],
            64,
            0,
        ),
        (
            &include_bytes!("../../../fixtures/weapon-selection-v41/stalker-to-ar.bin")[..],
            79,
            0,
        ),
    ] {
        let mut reader = ComponentReader {
            reference_widths: None,
            width_error: None,
            live_grammar: None,
            movement_slot: None,
            references: vec![],
            diagnostics: Default::default(),
            cursor: ComponentCursor::new(bytes, bit).unwrap(),
            fields: vec![],
            position_encoding: None,
        };
        assert_eq!(
            dispatch::component(&mut reader, "biped-desired-weapon-set", 0, 35),
            Some(true)
        );
        assert_eq!(reader.cursor.position, (bit + 7) as i64);
        assert_eq!(value(&reader.fields, "selection", 3), Some(1));
        let r = record(vec![component(
            42,
            "biped-desired-weapon-set",
            reader.fields,
        )]);
        let obs = observe(&r, &[43, 44, 45, 46], &[], 100, source());
        assert!(
            matches!(obs[0].change, WeaponChange::Selection { principal: Some(p), offhand: None } if p == slot)
        );
    }
}

#[test]
fn sparse_identity_uses_registry_rank_and_first_presence_gate() {
    let mut registry = FilmRegistry {
        archetypes: vec![
            FilmArchetype {
                index: 0,
                components: vec![],
                source: ByteRange { start: 0, end: 0 },
                padding: ByteRange { start: 0, end: 0 },
            };
            36
        ],
    };
    for name in [
        "other",
        "weapon-state-type-info",
        "other",
        "weapon-state-type-info",
        "weapon-state-type-info",
        "weapon-state-type-info",
    ] {
        registry.archetypes[35].components.push(RegistryComponent {
            name_bytes: name.as_bytes().to_vec(),
            precision_level: 0,
            source: ByteRange { start: 0, end: 0 },
        });
    }
    let indices = slot_indices(&registry);
    assert_eq!(indices, [1, 3, 4, 5]);
    let mut r = record(vec![component(
        3,
        "weapon-state-type-info",
        vec![
            field("present", 1, 1),
            field("id_high", 32, 0xfedcba98),
            field("variant", 32, 0x76543210),
            field("present", 1, 0), // Nested frame configuration, not identity presence.
        ],
    )]);
    let obs = observe(&r, &indices, &[], 100, source());
    assert!(matches!(
        obs[0].change,
        WeaponChange::Slot {
            slot: 1,
            asset_id: Some(Weapon::Unknown(WeaponId(0xfedcba9876543210)))
        }
    ));
    r.stop = EntityViewStop::Truncated;
    assert!(observe(&r, &indices, &[], 100, source()).is_empty());
}

#[test]
fn switches_pickups_drops_and_ambiguous_inventory() {
    let mut state = Inventory::default();
    for o in inventory() {
        state.apply(o.change);
    }
    let initial = state.sample(100, &[false; 4]);
    assert_eq!(initial.secondary_weapon_empty, Some(false));
    state.apply(selection(1));
    assert_eq!(
        state.sample(200, &[false; 4]).active_weapon_id,
        initial.secondary_weapon_id
    );
    assert_eq!(
        state.sample(200, &[false; 4]).secondary_weapon_id,
        initial.active_weapon_id
    );
    state.apply(selection(0));
    assert_eq!(
        state.sample(300, &[false; 4]).active_weapon_id,
        initial.active_weapon_id
    );
    state.apply(WeaponChange::Slot {
        slot: 1,
        asset_id: Some(Weapon::from(WeaponId(99))),
    });
    assert_eq!(
        state.sample(400, &[false; 4]).secondary_weapon_id,
        Some(Weapon::from(WeaponId(99)))
    );
    state.apply(WeaponChange::Slot {
        slot: 2,
        asset_id: Some(Weapon::from(WeaponId(100))),
    });
    assert_eq!(state.sample(500, &[false; 4]).secondary_weapon_id, None); // Three carried assets.
    assert_eq!(state.sample(500, &[false; 4]).secondary_weapon_empty, None);
    state.apply(WeaponChange::Slot {
        slot: 2,
        asset_id: None,
    });
    state.apply(WeaponChange::Slot {
        slot: 1,
        asset_id: None,
    });
    assert_eq!(state.sample(600, &[false; 4]).secondary_weapon_id, None); // Explicit drop.
    assert_eq!(
        state.sample(600, &[false; 4]).secondary_weapon_empty,
        Some(true)
    );
    state.apply(WeaponChange::Selection {
        principal: Some(0),
        offhand: Some(1),
    });
    assert_eq!(state.sample(700, &[false; 4]).active_weapon_id, None); // Dual wield unsupported.
    state.apply(selection(0));
    state.apply(WeaponChange::UnknownSelection);
    assert_eq!(state.sample(800, &[false; 4]).active_weapon_id, None);
    assert_eq!(state.sample(800, &[false; 4]).secondary_weapon_empty, None);
    assert_eq!(
        Inventory::default()
            .sample(900, &[false; 4])
            .secondary_weapon_empty,
        None
    );
}

#[test]
fn gaps_death_and_generation_changes_never_recover_stale_inventory() {
    let lives = vec![
        PlayerLife {
            start_us: 100,
            end_us: 400,
            entity_id: 1,
            source: source(),
            end_reason: LifeEnd::SummaryDeath,
        },
        PlayerLife {
            start_us: 400,
            end_us: 800,
            entity_id: 0x40000001,
            source: source(),
            end_reason: LifeEnd::FilmEnd,
        },
    ];
    let mut obs = inventory();
    obs.push(observation(250, selection(1))); // Selection alone cannot recover identities after gap.
    obs.push(observation(
        400,
        WeaponChange::Slot {
            slot: 0,
            asset_id: Some(Weapon::from(WeaponId(123))),
        },
    )); // Old generation.
    let mut new = observation(400, selection(0));
    new.entity_id = 0x40000001;
    obs.push(new);
    let samples = timeline(&mut obs, vec![(200, 5)], &lives);
    assert!(samples[0].active_weapon_id.is_some());
    assert!(
        samples
            .iter()
            .filter(|s| s.timestamp_us >= 200)
            .all(|s| s.active_weapon_id.is_none())
    );
    assert!(
        !obs.iter()
            .any(|o| o.timestamp_us >= 400 && o.entity_id == 1)
    );
    let mut obs = inventory();
    let samples = timeline(&mut obs, vec![], &lives);
    assert!(
        samples
            .iter()
            .find(|s| s.timestamp_us == 400)
            .unwrap()
            .active_weapon_id
            .is_none()
    );
}

#[test]
fn coincident_packet_gap_follows_prior_observation() {
    let mut obs = inventory();
    let lives = vec![PlayerLife {
        start_us: 100,
        end_us: 200,
        entity_id: 1,
        source: source(),
        end_reason: LifeEnd::FilmEnd,
    }];
    let samples = timeline(&mut obs, vec![(100, 5)], &lives);
    assert!(samples[0].active_weapon_id.is_none());
    let mut obs = inventory();
    let samples = timeline(&mut obs, vec![(100, 0)], &lives);
    assert!(samples[0].active_weapon_id.is_some());
}

#[test]
fn sparse_ammo_gate_and_slot_replacement() {
    let mut r = record(vec![
        component(
            30,
            "weapon-state-ammo",
            vec![field("magazine.gate", 1, 0), field("magazine", 8, 2)],
        ),
        component(
            43,
            "weapon-state-type-info",
            vec![
                field("present", 1, 1),
                field("id_high", 32, 1),
                field("variant", 32, 2),
            ],
        ),
        component(
            42,
            "biped-desired-weapon-set",
            vec![
                field("slot_a.gate", 1, 0),
                field("slot_a", 2, 0),
                field("slot_b.gate", 1, 1),
            ],
        ),
    ]);
    let changes = observe(&r, &[43, 44, 45, 46], &[30, 33, 36, 39], 100, source());
    let mut state = Inventory::default();
    let mut touched = [false; 4];
    for observation in changes {
        if let WeaponChange::Ammo { slot, .. } = observation.change {
            touched[slot] = true;
        }
        state.apply(observation.change);
    }
    assert_eq!(state.sample(100, &touched).active_weapon_ammo, Some(2));
    r.components[0].fields = vec![field("magazine.gate", 1, 1)];
    assert!(
        observe(&r, &[43, 44, 45, 46], &[30, 33, 36, 39], 200, source())
            .iter()
            .all(|o| !matches!(o.change, WeaponChange::Ammo { .. }))
    );
    state.apply(WeaponChange::Slot {
        slot: 0,
        asset_id: Some(Weapon::from(WeaponId(3))),
    });
    assert_eq!(state.magazines[0], None);
}
