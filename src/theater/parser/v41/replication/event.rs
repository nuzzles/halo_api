//! Project established event-body fields without correlating separate events.
use crate::theater::film::{
    EventPosition, ProjectileDetonation, ProjectileImpact, ProjectileVariant, ThrowSelection,
};
use crate::theater::film::{ReplicationEvent, ReplicationEventHandle, TagId, WeaponVariantId};
use crate::theater::parser::v41::context::{V41DecodeConfig, movement::PrecisionDescriptor};
use crate::theater::parser::v41::replication::source::stream::models::kill_event_chain::{
    EventFieldStage, EventRecord,
};
use crate::theater::parser::v41::replication::stream::frame::events::reference_kinds;
use crate::theater::parser::wire::bits::Cursor;

pub(super) fn project(
    record: &EventRecord,
    payload: &[u8],
    config: &V41DecodeConfig,
) -> Option<ReplicationEvent> {
    let code = record.code?;
    if !record.layout_complete || !matches!(code, 0 | 5 | 6 | 9 | 21 | 32 | 36 | 38 | 39 | 40 | 85)
    {
        return None;
    }
    let handles = handles(record, payload, code)?;
    let mut bits = Cursor::new(payload, record.body_start_bit?)?;
    let event = match code {
        38 => ReplicationEvent::WeaponReload {
            handles,
            mode: bits.read(4)? as u8,
            player: player(&mut bits)?,
        },
        39 => {
            let selection = if bits.read(1)? != 0 {
                ThrowSelection::Tagged {
                    tag: optional_tag(&mut bits)?,
                    value: bits.read(4)? as u8,
                }
            } else {
                ThrowSelection::Index(bits.read(3)? as u8)
            };
            ReplicationEvent::Throw {
                handles,
                selection,
                player: player(&mut bits)?,
            }
        }
        40 => {
            let kind = bits.read(3)? as u8;
            let extra = if kind == 1 {
                Some(bits.read(2)? as u8)
            } else {
                None
            };
            ReplicationEvent::Melee {
                handles,
                kind,
                extra,
                mode: bits.read(2)? as u8,
                weapon: optional_tag(&mut bits)?,
                variant: WeaponVariantId(bits.read(32)? as u32),
                player: player(&mut bits)?,
            }
        }
        9 => ReplicationEvent::Pickup {
            handles,
            kind: bits.read(3)? as u8,
            item: optional_tag(&mut bits)?,
        },
        21 => ReplicationEvent::Zoom {
            handles,
            level: bits.read(2)? as u8,
        },
        32 => {
            let encoding = config
                .event_quantized_vectors
                .then_some(&config.profile.movement.world_object);
            ReplicationEvent::Teleport {
                handles,
                from: point(&mut bits, encoding, 22)?,
                to: point(&mut bits, encoding, 22)?,
                tags: [optional_tag(&mut bits)?, optional_tag(&mut bits)?],
            }
        }
        5 => {
            let value6 = bits.read(6)? as u8;
            let variant = projectile_variant(&mut bits)?;
            let tag = optional_tag(&mut bits)?;
            let position = point(
                &mut bits,
                config
                    .projectile_event_positions
                    .as_ref()
                    .map(|p| &p.detonation),
                21,
            )?;
            let value19 = bits.read(19)? as u32;
            let value5 = bits.read(5)? as u8;
            let flag = bits.read(1)? != 0;
            let value9 = bits.read(9)? as u16;
            let optional18 = if bits.read(1)? != 0 {
                Some(bits.read(18)? as u32)
            } else {
                None
            };
            let direction_bits = if bits.read(1)? != 0 { 19 } else { 8 };
            let direction = bits.read(direction_bits)? as u32;
            ReplicationEvent::ProjectileDetonation {
                handles,
                data: ProjectileDetonation {
                    value6,
                    variant,
                    tag,
                    position,
                    value19,
                    value5,
                    flag,
                    value9,
                    optional18,
                    direction_bits: direction_bits as u8,
                    direction,
                    value2: bits.read(2)? as u8,
                },
            }
        }
        6 => ReplicationEvent::ProjectileImpact {
            handles,
            data: ProjectileImpact {
                variant: projectile_variant(&mut bits)?,
                value14: bits.read(14)? as u16,
                first19: bits.read(19)? as u32,
                position: point(
                    &mut bits,
                    config
                        .projectile_event_positions
                        .as_ref()
                        .map(|p| &p.impact),
                    18,
                )?,
                second19: bits.read(19)? as u32,
                value9: bits.read(9)? as u16,
                flag: bits.read(1)? != 0,
            },
        },
        0 => {
            let effect = optional_tag(&mut bits)?;
            let attacker_player = player(&mut bits)?;
            let packed_direction = bits.read(19)? as u32;
            if bits.read(1)? != 0 {
                bits.skip(31)?;
            }
            let raw_scale = bits.read(5)? as u8;
            let [victim, attacker, third_reference] = handles;
            ReplicationEvent::DamageAftermath {
                victim,
                attacker,
                third_reference,
                effect,
                attacker_player,
                packed_direction,
                raw_scale,
            }
        }
        36 => {
            bits.skip(10)?; // two mode flags, seven bits, one bit
            let player = player(&mut bits)?;
            if bits.read(1)? == 0 {
                bits.skip(2)?;
            }
            let weapon = optional_tag(&mut bits)?;
            let variant = WeaponVariantId(bits.read(32)? as u32);
            ReplicationEvent::WeaponFire {
                handles,
                player,
                weapon,
                variant,
            }
        }
        85 => {
            let victim = player(&mut bits)?;
            let killer = player(&mut bits)?;
            let killer_percent = bits.read(32)? as u32;
            let flag = bits.read(1)? != 0;
            let assistant = player(&mut bits)?;
            let assistant_percent = bits.read(32)? as u32;
            ReplicationEvent::Kill {
                handles,
                victim,
                killer,
                killer_percent,
                flag,
                assistant,
                assistant_percent,
            }
        }
        _ => return None,
    };
    // A parser or layout change must not silently reinterpret bits beyond the
    // event's established source range.
    let body_end = record
        .fields
        .iter()
        .find(|f| f.stage == EventFieldStage::CorruptionCheck)
        .map_or(record.end_bit, |f| f.bit);
    // Existing combat projections intentionally expose a prefix of their bodies.
    // New action/projectile layouts must close exactly before any guard trailer.
    (bits.position <= body_end && (matches!(code, 0 | 36 | 85) || bits.position == body_end))
        .then_some(event)
}

fn projectile_variant(bits: &mut Cursor<'_>) -> Option<Option<ProjectileVariant>> {
    if bits.read(1)? != 0 {
        return Some(None);
    }
    Some(Some(ProjectileVariant {
        tag: optional_tag(bits)?,
        variant: WeaponVariantId(bits.read(32)? as u32),
    }))
}

fn point(
    bits: &mut Cursor<'_>,
    encoding: Option<&PrecisionDescriptor>,
    default_width: u8,
) -> Option<EventPosition> {
    if bits.read(1)? != 0 {
        let mut codes = [0; 3];
        for code in &mut codes {
            *code = bits.read(default_width as usize)? as u32;
        }
        return Some(EventPosition::DefaultBox {
            codes,
            axis_bits: default_width,
        });
    }
    let encoding = encoding?;
    if encoding.index_bits > 30 {
        return None;
    }
    let region = bits.read(encoding.index_bits as usize)? as u32;
    let widths = encoding.axes_for_region(region)?;
    if widths.iter().any(|w| *w > 32) {
        return None;
    }
    let mut codes = [0; 3];
    for (code, width) in codes.iter_mut().zip(widths) {
        *code = bits.read(width as usize)? as u32;
    }
    Some(EventPosition::MapRegion {
        region,
        codes,
        axis_bits: widths.map(|w| w as u8),
    })
}

fn handles(
    record: &EventRecord,
    payload: &[u8],
    code: u8,
) -> Option<[Option<ReplicationEventHandle>; 3]> {
    let mut bits = Cursor::new(payload, record.start_bit.checked_add(8)?)?;
    let mut out = [None; 3];
    // The reference grammar's configured kind controls the index width. Kind
    // one carries an extra mode bit that switches it to kind four.
    const WIDTHS: [usize; 9] = [13, 13, 8, 8, 9, 8, 9, 13, 13];
    for (slot, configured) in out.iter_mut().zip(reference_kinds(code)?) {
        if bits.read(1)? == 0 {
            continue;
        }
        let mut kind = u8::try_from(configured).ok()?;
        if kind == 1 && bits.read(1)? != 0 {
            kind = 4;
        }
        let index = bits.read(*WIDTHS.get(usize::from(kind))?)? as u16;
        let generation = bits.read(2)? as u8;
        *slot = Some(ReplicationEventHandle {
            kind,
            index,
            generation,
        });
    }
    (bits.position == record.body_start_bit?).then_some(out)
}

fn player(bits: &mut Cursor<'_>) -> Option<Option<u8>> {
    if bits.read(1)? != 0 {
        Some(None)
    } else {
        Some(Some(bits.read(5)? as u8))
    }
}

fn optional_tag(bits: &mut Cursor<'_>) -> Option<Option<TagId>> {
    if bits.read(1)? == 0 {
        Some(None)
    } else {
        Some(Some(TagId(bits.read(32)? as u32)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(fields: &[(u64, usize)]) -> (Vec<u8>, usize) {
        let bit_len: usize = fields.iter().map(|(_, width)| *width).sum();
        let mut bytes = vec![0u8; bit_len.div_ceil(8)];
        let mut at = 0;
        for &(value, width) in fields {
            for bit in (0..width).rev() {
                bytes[at / 8] |= (((value >> bit) & 1) as u8) << (7 - at % 8);
                at += 1;
            }
        }
        (bytes, bit_len)
    }

    fn decode(code: u8, refs: &[(u64, usize)], body: &[(u64, usize)]) -> ReplicationEvent {
        let prefix = [(1, 1), (u64::from(code), 7)];
        let fields: Vec<_> = prefix.iter().chain(refs).chain(body).copied().collect();
        let (payload, end_bit) = pack(&fields);
        let body_start_bit = 8 + refs.iter().map(|(_, width)| *width).sum::<usize>();
        let record = EventRecord {
            start_bit: 0,
            end_bit,
            code: Some(code),
            body_start_bit: Some(body_start_bit),
            fields: Vec::new(),
            layout_complete: true,
        };
        project(&record, &payload, &V41DecodeConfig::default()).unwrap()
    }

    #[test]
    fn projects_recorded_combat_fields_without_inferring_a_weapon_from_a_hit() {
        assert_eq!(
            decode(
                0,
                &[(1, 1), (1, 1), (5, 9), (2, 2), (0, 1), (0, 1)],
                &[
                    (1, 1),
                    (0x1234_5678, 32),
                    (0, 1),
                    (7, 5),
                    (123, 19),
                    (1, 1),
                    (0, 31),
                    (15, 5)
                ]
            ),
            ReplicationEvent::DamageAftermath {
                victim: Some(ReplicationEventHandle {
                    kind: 4,
                    index: 5,
                    generation: 2
                }),
                attacker: None,
                third_reference: None,
                effect: Some(TagId(0x1234_5678)),
                attacker_player: Some(7),
                packed_direction: 123,
                raw_scale: 15,
            }
        );
        assert_eq!(
            decode(
                36,
                &[(0, 1), (0, 1), (0, 1)],
                &[
                    (0, 2),
                    (0, 8),
                    (0, 1),
                    (3, 5),
                    (0, 1),
                    (0, 2),
                    (1, 1),
                    (0x0000_8594, 32),
                    (0xfeed_beef, 32)
                ]
            ),
            ReplicationEvent::WeaponFire {
                handles: [None; 3],
                player: Some(3),
                weapon: Some(TagId(0x0000_8594)),
                variant: WeaponVariantId(0xfeed_beef),
            }
        );
        assert_eq!(
            decode(
                85,
                &[(0, 1), (0, 1), (0, 1)],
                &[
                    (0, 1),
                    (4, 5),
                    (0, 1),
                    (9, 5),
                    (65, 32),
                    (0, 1),
                    (1, 1),
                    (0, 32)
                ]
            ),
            ReplicationEvent::Kill {
                handles: [None; 3],
                victim: Some(4),
                killer: Some(9),
                killer_percent: 65,
                flag: false,
                assistant: None,
                assistant_percent: 0,
            }
        );
    }

    #[test]
    fn damage_references_have_independent_presence_bits() {
        assert_eq!(
            decode(
                0,
                &[
                    (0, 1),
                    (1, 1),
                    (0, 1),
                    (123, 13),
                    (1, 2),
                    (1, 1),
                    (456, 13),
                    (2, 2),
                ],
                &[(0, 1), (1, 1), (0, 19), (0, 1), (2, 5)],
            ),
            ReplicationEvent::DamageAftermath {
                victim: None,
                attacker: Some(ReplicationEventHandle {
                    kind: 1,
                    index: 123,
                    generation: 1,
                }),
                third_reference: Some(ReplicationEventHandle {
                    kind: 7,
                    index: 456,
                    generation: 2,
                }),
                effect: None,
                attacker_player: None,
                packed_direction: 0,
                raw_scale: 2,
            }
        );
    }

    #[test]
    fn action_payloads_keep_optional_gates_and_raw_codes() {
        let refs = [(0, 1); 3];
        assert_eq!(
            decode(38, &refs, &[(9, 4), (0, 1), (17, 5)]),
            ReplicationEvent::WeaponReload {
                handles: [None; 3],
                mode: 9,
                player: Some(17)
            }
        );
        assert_eq!(
            decode(38, &refs, &[(0, 4), (1, 1)]),
            ReplicationEvent::WeaponReload {
                handles: [None; 3],
                mode: 0,
                player: None
            }
        );
        assert_eq!(
            decode(39, &refs, &[(0, 1), (7, 3), (1, 1)]),
            ReplicationEvent::Throw {
                handles: [None; 3],
                selection: ThrowSelection::Index(7),
                player: None
            }
        );
        assert_eq!(
            decode(
                39,
                &refs,
                &[(1, 1), (1, 1), (42, 32), (4, 4), (0, 1), (3, 5)]
            ),
            ReplicationEvent::Throw {
                handles: [None; 3],
                selection: ThrowSelection::Tagged {
                    tag: Some(TagId(42)),
                    value: 4
                },
                player: Some(3)
            }
        );
        assert_eq!(
            decode(39, &refs, &[(1, 1), (0, 1), (2, 4), (1, 1)]),
            ReplicationEvent::Throw {
                handles: [None; 3],
                selection: ThrowSelection::Tagged {
                    tag: None,
                    value: 2
                },
                player: None
            }
        );
        assert_eq!(
            decode(
                40,
                &refs,
                &[
                    (1, 3),
                    (2, 2),
                    (3, 2),
                    (1, 1),
                    (42, 32),
                    (123, 32),
                    (0, 1),
                    (5, 5)
                ]
            ),
            ReplicationEvent::Melee {
                handles: [None; 3],
                kind: 1,
                extra: Some(2),
                mode: 3,
                weapon: Some(TagId(42)),
                variant: WeaponVariantId(123),
                player: Some(5)
            }
        );
        assert_eq!(
            decode(40, &refs, &[(0, 3), (1, 2), (0, 1), (0, 32), (1, 1)]),
            ReplicationEvent::Melee {
                handles: [None; 3],
                kind: 0,
                extra: None,
                mode: 1,
                weapon: None,
                variant: WeaponVariantId(0),
                player: None
            }
        );
        for tag in [None, Some(42)] {
            let mut body = vec![(5, 3), (u64::from(tag.is_some()), 1)];
            if let Some(tag) = tag {
                body.push((tag, 32));
            }
            assert_eq!(
                decode(9, &refs, &body),
                ReplicationEvent::Pickup {
                    handles: [None; 3],
                    kind: 5,
                    item: tag.map(|n| TagId(n as u32))
                }
            );
        }
        assert_eq!(
            decode(21, &refs, &[(2, 2)]),
            ReplicationEvent::Zoom {
                handles: [None; 3],
                level: 2
            }
        );
    }

    #[test]
    fn projectile_and_teleport_payloads_preserve_quantized_positions() {
        let refs = [(0, 1); 3];
        assert_eq!(
            decode(
                32,
                &refs,
                &[
                    (1, 1),
                    (1, 22),
                    (2, 22),
                    (3, 22),
                    (1, 1),
                    (4, 22),
                    (5, 22),
                    (6, 22),
                    (0, 1),
                    (1, 1),
                    (42, 32)
                ]
            ),
            ReplicationEvent::Teleport {
                handles: [None; 3],
                from: EventPosition::DefaultBox {
                    codes: [1, 2, 3],
                    axis_bits: 22
                },
                to: EventPosition::DefaultBox {
                    codes: [4, 5, 6],
                    axis_bits: 22
                },
                tags: [None, Some(TagId(42))]
            }
        );
        for wide in [false, true] {
            let mut body = vec![
                (3, 6),
                (0, 1),
                (1, 1),
                (42, 32),
                (123, 32),
                (0, 1),
                (1, 1),
                (1, 21),
                (2, 21),
                (3, 21),
                (17, 19),
                (5, 5),
                (1, 1),
                (9, 9),
                (u64::from(wide), 1),
            ];
            if wide {
                body.push((18, 18));
            }
            body.extend([(u64::from(wide), 1), (7, if wide { 19 } else { 8 }), (2, 2)]);
            assert_eq!(
                decode(5, &refs, &body),
                ReplicationEvent::ProjectileDetonation {
                    handles: [None; 3],
                    data: ProjectileDetonation {
                        value6: 3,
                        variant: Some(ProjectileVariant {
                            tag: Some(TagId(42)),
                            variant: WeaponVariantId(123)
                        }),
                        tag: None,
                        position: EventPosition::DefaultBox {
                            codes: [1, 2, 3],
                            axis_bits: 21
                        },
                        value19: 17,
                        value5: 5,
                        flag: true,
                        value9: 9,
                        optional18: wide.then_some(18),
                        direction_bits: if wide { 19 } else { 8 },
                        direction: 7,
                        value2: 2,
                    }
                }
            );
        }
        assert_eq!(
            decode(
                6,
                &[(1, 1), (7, 8), (2, 2), (0, 1), (0, 1)],
                &[
                    (1, 1),
                    (14, 14),
                    (19, 19),
                    (1, 1),
                    (1, 18),
                    (2, 18),
                    (3, 18),
                    (23, 19),
                    (9, 9),
                    (0, 1)
                ]
            ),
            ReplicationEvent::ProjectileImpact {
                handles: [
                    Some(ReplicationEventHandle {
                        kind: 5,
                        index: 7,
                        generation: 2
                    }),
                    None,
                    None
                ],
                data: ProjectileImpact {
                    variant: None,
                    value14: 14,
                    first19: 19,
                    position: EventPosition::DefaultBox {
                        codes: [1, 2, 3],
                        axis_bits: 18
                    },
                    second19: 23,
                    value9: 9,
                    flag: false
                }
            }
        );
    }

    #[test]
    fn region_positions_require_the_recorded_region_and_axis_widths() {
        let encoding = PrecisionDescriptor {
            index_bits: 2,
            axis_bits: [3, 4, 5],
            region: 2,
            regions: Default::default(),
        };
        let (data, _) = pack(&[(0, 1), (2, 2), (7, 3), (15, 4), (31, 5)]);
        assert_eq!(
            point(&mut Cursor::new(&data, 0).unwrap(), Some(&encoding), 22),
            Some(EventPosition::MapRegion {
                region: 2,
                codes: [7, 15, 31],
                axis_bits: [3, 4, 5]
            })
        );
        assert!(point(&mut Cursor::new(&data, 0).unwrap(), None, 22).is_none());
        let mut other = encoding.clone();
        other.region = 1;
        assert!(point(&mut Cursor::new(&data, 0).unwrap(), Some(&other), 22).is_none());
        assert!(
            point(
                &mut Cursor::new(&data[..1], 0).unwrap(),
                Some(&encoding),
                22
            )
            .is_none()
        );
    }

    #[test]
    fn new_projections_require_complete_exact_bodies_but_allow_recorded_guards() {
        use crate::theater::parser::v41::replication::source::stream::models::kill_event_chain::{
            EventField, EventFieldValue,
        };
        let (payload, end_bit) = pack(&[(1, 1), (21, 7), (0, 1), (0, 1), (0, 1), (2, 2), (0, 1)]);
        let mut record = EventRecord {
            start_bit: 0,
            end_bit: end_bit - 1,
            code: Some(21),
            body_start_bit: Some(11),
            fields: vec![],
            layout_complete: true,
        };
        let config = V41DecodeConfig::default();
        assert!(project(&record, &payload, &config).is_some());
        record.layout_complete = false;
        assert!(project(&record, &payload, &config).is_none());
        record.layout_complete = true;
        record.end_bit = end_bit;
        assert!(project(&record, &payload, &config).is_none());
        record.fields.push(EventField {
            bit: end_bit - 1,
            width: 1,
            stage: EventFieldStage::CorruptionCheck,
            value: EventFieldValue::Scalar(0),
        });
        assert!(project(&record, &payload, &config).is_some());
        assert!(project(&record, &payload[..1], &config).is_none());
    }

    #[test]
    fn refuses_a_body_past_the_record_boundary() {
        let (payload, _) = pack(&[
            (1, 1),
            (0, 7),
            (0, 1),
            (0, 1),
            (0, 1),
            (1, 1),
            (42, 32),
            (1, 1),
            (0, 19),
            (0, 1),
            (15, 5),
        ]);
        let record = EventRecord {
            start_bit: 0,
            end_bit: 32,
            code: Some(0),
            body_start_bit: Some(11),
            fields: Vec::new(),
            layout_complete: true,
        };
        assert!(project(&record, &payload, &V41DecodeConfig::default()).is_none());
    }
}
