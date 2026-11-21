//! Project established event-body fields without correlating separate events.
use crate::theater::film::{
    EventPlayerSlot, ReplicationEvent, TagId, WeaponTagId, WeaponVariantId,
};
use crate::theater::film::{
    EventPosition, ProjectileDetonation, ProjectileImpact, ProjectileVariant, ThrowSelection,
};
use crate::theater::parser::v41::context::{V41DecodeConfig, movement::PrecisionDescriptor};
use crate::theater::parser::v41::replication::source::stream::models::kill_event_chain::EventRecord;
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
    let body = record.body_range()?;
    let mut bits = Cursor::new(payload, body.start)?;
    let event = match code {
        38 => ReplicationEvent::WeaponReload {
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
                kind,
                extra,
                mode: bits.read(2)? as u8,
                weapon: optional_tag(&mut bits)?.map(|tag| WeaponTagId(tag.raw())),
                variant: WeaponVariantId(bits.read(32)? as u32),
                player: player(&mut bits)?,
            }
        }
        9 => ReplicationEvent::Pickup {
            kind: bits.read(3)? as u8,
            item: optional_tag(&mut bits)?,
        },
        21 => ReplicationEvent::Zoom {
            level: bits.read(2)? as u8,
        },
        32 => {
            let encoding = config
                .event_quantized_vectors
                .then_some(&config.profile.movement.world_object);
            ReplicationEvent::Teleport {
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
            ReplicationEvent::DamageAftermath {
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
            let weapon = optional_tag(&mut bits)?.map(|tag| WeaponTagId(tag.raw()));
            let variant = WeaponVariantId(bits.read(32)? as u32);
            ReplicationEvent::WeaponFire {
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
    // Existing combat projections intentionally expose a prefix of their bodies.
    // New action/projectile layouts must close exactly before any guard trailer.
    (bits.position <= body.end && (matches!(code, 0 | 36 | 85) || bits.position == body.end))
        .then_some(event)
}

/// Preserve undecoded body data without copying envelope bookkeeping into Film.
pub(super) fn raw(record: &EventRecord, payload: &[u8]) -> Option<ReplicationEvent> {
    let kind = record.code?.into();
    let body = record.body_range()?;
    let payload = crate::theater::parser::wire::fields::RawBits::from_source(
        payload,
        body.start,
        body.len(),
    )?;
    Some(ReplicationEvent::Raw {
        kind,
        payload: payload.into(),
    })
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

fn player(bits: &mut Cursor<'_>) -> Option<Option<EventPlayerSlot>> {
    if bits.read(1)? != 0 {
        Some(None)
    } else {
        Some(Some(EventPlayerSlot(bits.read(5)? as u8)))
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
    use crate::theater::parser::v41::replication::source::stream::models::kill_event_chain::{
        EventFieldStage, EventListStop,
    };
    use crate::theater::parser::v41::replication::stream::frame::events::{
        EventLayout, read_event_list,
    };

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
            references: None,
            fields: Vec::new(),
            layout_complete: true,
        };
        let event = project(&record, &payload, &V41DecodeConfig::default()).unwrap();
        let json = serde_json::to_value(&event).unwrap();
        for (kind, body) in json.as_object().unwrap() {
            let body = body.as_object().unwrap();
            assert!(!body.contains_key("handles"));
            assert!(!body.contains_key("third_reference"));
            assert!(!body.contains_key("attacker"));
            if kind == "DamageAftermath" {
                assert!(!body.contains_key("victim"));
            }
        }
        assert_eq!(
            serde_json::from_value::<ReplicationEvent>(json).unwrap(),
            event
        );
        event
    }

    #[test]
    fn projection_uses_the_established_body_start_without_decoding_references() {
        assert_eq!(
            decode(21, &[], &[(2, 2)]),
            ReplicationEvent::Zoom { level: 2 }
        );
        assert_eq!(
            decode(21, &[(u64::MAX, 64)], &[(2, 2)]),
            ReplicationEvent::Zoom { level: 2 }
        );
    }

    #[test]
    fn framed_references_preserve_alignment_and_stay_out_of_raw_bodies() {
        for mask in 0..8 {
            let mut fields = vec![(1, 1), (21, 7)];
            for (slot, width) in [9, 13, 13].into_iter().enumerate() {
                let present = mask & (1 << slot) != 0;
                fields.push((u64::from(present), 1));
                if present {
                    fields.extend([(slot as u64 + 7, width), (slot as u64, 2)]);
                }
            }
            let body_start: usize = fields.iter().map(|(_, width)| width).sum();
            fields.extend([(2, 2), (1, 1), (0xdeadbeef, 32)]);
            let next_start: usize = fields.iter().map(|(_, width)| width).sum();
            // Code 3 has an empty body and no guard, even when guards are enabled.
            fields.extend([(1, 1), (3, 7), (0, 3), (0, 1)]);
            let (payload, end) = pack(&fields);
            let list = read_event_list(
                &payload,
                0,
                3,
                EventLayout {
                    corruption_check: Some(true),
                    ..EventLayout::default()
                },
            );
            assert_eq!(list.stop, EventListStop::Terminator);
            assert_eq!(list.end_bit, end);
            assert_eq!(list.records.len(), 2);
            let record = &list.records[0];
            assert_eq!(record.body_range(), Some(body_start..body_start + 2));
            let refs = record.references.unwrap();
            for (slot, reference) in [refs.first, refs.second, refs.third]
                .into_iter()
                .enumerate()
            {
                assert_eq!(reference.is_some(), mask & (1 << slot) != 0);
                if let Some(reference) = reference {
                    assert_eq!(reference.kind, [4, 8, 7][slot]);
                    assert_eq!(reference.index, slot as u16 + 7);
                    assert_eq!(reference.generation, slot as u8);
                }
            }
            assert_eq!(
                project(record, &payload, &V41DecodeConfig::default()),
                Some(ReplicationEvent::Zoom { level: 2 })
            );
            let Some(ReplicationEvent::Raw {
                payload: raw_body, ..
            }) = raw(record, &payload)
            else {
                panic!("missing raw body")
            };
            assert_eq!(raw_body.bit_len, 2);
            assert_eq!(raw_body.as_u64(), Some(2));
            let empty = &list.records[1];
            assert_eq!(empty.start_bit, next_start);
            let Some(ReplicationEvent::Raw {
                payload: empty_body,
                ..
            }) = raw(empty, &payload)
            else {
                panic!("missing empty body")
            };
            assert_eq!(empty_body.bit_len, 0);
            assert!(empty_body.bytes.is_empty());
            let mut invalid = record.clone();
            invalid.body_start_bit = Some(invalid.end_bit + 1);
            assert!(raw(&invalid, &payload).is_none());
            invalid.body_start_bit = Some(7);
            assert!(raw(&invalid, &payload).is_none());
            invalid.body_start_bit = record.body_start_bit;
            invalid.layout_complete = false;
            assert!(raw(&invalid, &payload).is_none());
            assert!(raw(record, &payload[..body_start / 8]).is_none());
        }
    }

    #[test]
    fn combat_body_slots_and_weapon_tags_preserve_zero_and_absence() {
        assert_eq!(
            decode(36, &[], &[(0, 10), (1, 1), (1, 1), (0, 1), (0, 32)]),
            ReplicationEvent::WeaponFire {
                player: None,
                weapon: None,
                variant: WeaponVariantId(0),
            }
        );
        assert_eq!(
            decode(
                36,
                &[],
                &[(0, 10), (0, 1), (0, 5), (1, 1), (1, 1), (0, 32), (0, 32)]
            ),
            ReplicationEvent::WeaponFire {
                player: Some(EventPlayerSlot(0)),
                weapon: Some(WeaponTagId(0)),
                variant: WeaponVariantId(0),
            }
        );
        assert_eq!(
            decode(
                85,
                &[],
                &[(1, 1), (1, 1), (0, 32), (0, 1), (0, 1), (31, 5), (100, 32)]
            ),
            ReplicationEvent::Kill {
                victim: None,
                killer: None,
                killer_percent: 0,
                flag: false,
                assistant: Some(EventPlayerSlot(31)),
                assistant_percent: 100,
            }
        );
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
                effect: Some(TagId(0x1234_5678)),
                attacker_player: Some(EventPlayerSlot(7)),
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
                player: Some(EventPlayerSlot(3)),
                weapon: Some(WeaponTagId(0x0000_8594)),
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
                victim: Some(EventPlayerSlot(4)),
                killer: Some(EventPlayerSlot(9)),
                killer_percent: 65,
                flag: false,
                assistant: None,
                assistant_percent: 0,
            }
        );
    }

    #[test]
    fn damage_projection_omits_references_and_preserves_absent_body_fields() {
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
                mode: 9,
                player: Some(EventPlayerSlot(17))
            }
        );
        assert_eq!(
            decode(38, &refs, &[(0, 4), (1, 1)]),
            ReplicationEvent::WeaponReload {
                mode: 0,
                player: None
            }
        );
        assert_eq!(
            decode(39, &refs, &[(0, 1), (7, 3), (1, 1)]),
            ReplicationEvent::Throw {
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
                selection: ThrowSelection::Tagged {
                    tag: Some(TagId(42)),
                    value: 4
                },
                player: Some(EventPlayerSlot(3))
            }
        );
        assert_eq!(
            decode(39, &refs, &[(1, 1), (0, 1), (2, 4), (1, 1)]),
            ReplicationEvent::Throw {
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
                kind: 1,
                extra: Some(2),
                mode: 3,
                weapon: Some(WeaponTagId(42)),
                variant: WeaponVariantId(123),
                player: Some(EventPlayerSlot(5))
            }
        );
        assert_eq!(
            decode(40, &refs, &[(0, 3), (1, 2), (0, 1), (0, 32), (1, 1)]),
            ReplicationEvent::Melee {
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
                    kind: 5,
                    item: tag.map(|n| TagId(n as u32))
                }
            );
        }
        assert_eq!(
            decode(21, &refs, &[(2, 2)]),
            ReplicationEvent::Zoom { level: 2 }
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
            references: None,
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
            references: None,
            fields: Vec::new(),
            layout_complete: true,
        };
        assert!(project(&record, &payload, &V41DecodeConfig::default()).is_none());
    }
}
