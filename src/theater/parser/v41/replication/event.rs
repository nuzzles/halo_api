//! Project established event-body fields without correlating separate events.
use crate::theater::film::{
    EventPlayerSlot, ReplicationEvent, TagId, WeaponTagId, WeaponVariantId,
};
use crate::theater::film::{
    EventPosition, ProjectileDetonation, ProjectileImpact, ProjectileVariant,
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
        38 => {
            bits.skip(4)?;
            ReplicationEvent::WeaponReload {
                player: player(&mut bits)?,
            }
        }
        39 => {
            let item = if bits.read(1)? != 0 {
                let tag = optional_tag(&mut bits)?;
                bits.skip(4)?;
                tag
            } else {
                bits.skip(3)?;
                None
            };
            ReplicationEvent::Throw {
                item,
                player: player(&mut bits)?,
            }
        }
        40 => {
            let kind = bits.read(3)? as u8;
            if kind == 1 {
                bits.skip(2)?;
            }
            bits.skip(2)?;
            ReplicationEvent::Melee {
                weapon: optional_tag(&mut bits)?.map(|tag| WeaponTagId(tag.raw())),
                variant: WeaponVariantId(bits.read(32)? as u32),
                player: player(&mut bits)?,
            }
        }
        9 => {
            bits.skip(3)?;
            ReplicationEvent::Pickup {
                item: optional_tag(&mut bits)?,
            }
        }
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
            bits.skip(6)?;
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
            bits.skip(19 + 5 + 1 + 9)?;
            if bits.read(1)? != 0 {
                bits.skip(18)?;
            }
            let direction_bits = if bits.read(1)? != 0 { 19 } else { 8 };
            bits.skip(direction_bits + 2)?;
            ReplicationEvent::ProjectileDetonation {
                data: ProjectileDetonation {
                    variant,
                    tag,
                    position,
                },
            }
        }
        6 => {
            let variant = projectile_variant(&mut bits)?;
            bits.skip(14 + 19)?;
            let position = point(
                &mut bits,
                config
                    .projectile_event_positions
                    .as_ref()
                    .map(|p| &p.impact),
                18,
            )?;
            bits.skip(19 + 9 + 1)?;
            ReplicationEvent::ProjectileImpact {
                data: ProjectileImpact { variant, position },
            }
        }
        0 => {
            let effect = optional_tag(&mut bits)?;
            let attacker_player = player(&mut bits)?;
            let packed_direction = bits.read(19)? as u32;
            if bits.read(1)? != 0 {
                bits.skip(31)?;
            }
            bits.skip(5)?;
            ReplicationEvent::DamageAftermath {
                effect,
                attacker_player,
                packed_direction,
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
            bits.skip(1)?;
            let assistant = player(&mut bits)?;
            let assistant_percent = bits.read(32)? as u32;
            ReplicationEvent::Kill {
                victim,
                killer,
                killer_percent,
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

/// Preserve the occurrence without retaining an uninterpreted payload.
pub(super) fn other(record: &EventRecord) -> Option<ReplicationEvent> {
    let kind = record.code?.into();
    record.body_range()?;
    Some(ReplicationEvent::Other { kind })
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
            for field in ["flag", "mode", "extra", "raw_scale", "kind", "selection"] {
                assert!(!body.contains_key(field), "unexpected {field} in {kind}");
            }
            if matches!(kind.as_str(), "ProjectileDetonation" | "ProjectileImpact") {
                let data = body["data"].as_object().unwrap();
                assert!(
                    data.keys()
                        .all(|key| matches!(key.as_str(), "variant" | "tag" | "position"))
                );
            }
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
    fn framed_references_preserve_alignment_without_public_raw_payloads() {
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
            assert_eq!(
                serde_json::to_value(other(record).unwrap()).unwrap(),
                serde_json::json!({"Other": {"kind": 21}})
            );
            let empty = &list.records[1];
            assert_eq!(empty.start_bit, next_start);
            assert_eq!(
                serde_json::to_value(other(empty).unwrap()).unwrap(),
                serde_json::json!({"Other": {"kind": 3}})
            );
            let mut invalid = record.clone();
            invalid.body_start_bit = Some(invalid.end_bit + 1);
            assert!(other(&invalid).is_none());
            invalid.body_start_bit = Some(7);
            assert!(other(&invalid).is_none());
            invalid.body_start_bit = record.body_start_bit;
            invalid.layout_complete = false;
            assert!(other(&invalid).is_none());
            assert!(
                project(
                    record,
                    &payload[..body_start / 8],
                    &V41DecodeConfig::default()
                )
                .is_none()
            );
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
            }
        );
    }

    #[test]
    fn action_payloads_keep_known_fields_while_consuming_unknown_codes() {
        let refs = [(0, 1); 3];
        assert_eq!(
            decode(38, &refs, &[(9, 4), (0, 1), (17, 5)]),
            ReplicationEvent::WeaponReload {
                player: Some(EventPlayerSlot(17))
            }
        );
        assert_eq!(
            decode(38, &refs, &[(0, 4), (1, 1)]),
            ReplicationEvent::WeaponReload { player: None }
        );
        assert_eq!(
            decode(39, &refs, &[(0, 1), (7, 3), (1, 1)]),
            ReplicationEvent::Throw {
                item: None,
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
                item: Some(TagId(42)),
                player: Some(EventPlayerSlot(3))
            }
        );
        assert_eq!(
            decode(39, &refs, &[(1, 1), (0, 1), (2, 4), (1, 1)]),
            ReplicationEvent::Throw {
                item: None,
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
                weapon: Some(WeaponTagId(42)),
                variant: WeaponVariantId(123),
                player: Some(EventPlayerSlot(5))
            }
        );
        assert_eq!(
            decode(40, &refs, &[(0, 3), (1, 2), (0, 1), (0, 32), (1, 1)]),
            ReplicationEvent::Melee {
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
        for (optional, wide) in [(false, false), (false, true), (true, false), (true, true)] {
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
                (u64::from(optional), 1),
            ];
            if optional {
                body.push((18, 18));
            }
            body.extend([(u64::from(wide), 1), (7, if wide { 19 } else { 8 }), (2, 2)]);
            // Dropping unknown fields from Film must not change event boundaries.
            let mut framed = vec![(1, 1), (5, 7), (0, 3)];
            framed.extend_from_slice(&body);
            let body_end: usize = framed.iter().map(|(_, width)| width).sum();
            framed.extend([(1, 1), (0xdeadbeef, 32), (1, 1), (3, 7), (0, 3), (0, 1)]);
            let (payload, end_bit) = pack(&framed);
            let layout = || EventLayout {
                corruption_check: Some(true),
                ..EventLayout::default()
            };
            let list = read_event_list(&payload, 0, 3, layout());
            assert_eq!(list.stop, EventListStop::Terminator);
            assert_eq!(list.end_bit, end_bit);
            assert_eq!(list.records.len(), 2);
            assert_eq!(list.records[0].body_range(), Some(11..body_end));
            assert_eq!(list.records[1].start_bit, body_end + 33);
            assert_eq!(
                project(&list.records[0], &payload, &V41DecodeConfig::default()).unwrap(),
                decode(5, &refs, &body)
            );
            let truncated = read_event_list(&payload[..(body_end - 1) / 8], 0, 3, layout());
            assert!(!truncated.records[0].layout_complete);
            assert!(other(&truncated.records[0]).is_none());
            assert_eq!(
                decode(5, &refs, &body),
                ReplicationEvent::ProjectileDetonation {
                    data: ProjectileDetonation {
                        variant: Some(ProjectileVariant {
                            tag: Some(TagId(42)),
                            variant: WeaponVariantId(123)
                        }),
                        tag: None,
                        position: EventPosition::DefaultBox {
                            codes: [1, 2, 3],
                            axis_bits: 21
                        },
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
                    position: EventPosition::DefaultBox {
                        codes: [1, 2, 3],
                        axis_bits: 18
                    },
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
