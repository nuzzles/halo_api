//! Project established event fields and reference roles without correlating events.
use crate::theater::film::catalog::maps::levels::LevelCalibration;
use crate::theater::film::{
    EventPlayerSlot, ParseError, ReplicationEvent, ReplicationEventReference, TagId, WeaponTagId,
    WeaponVariantId,
};
use crate::theater::parser::v41::context::{V41DecodeConfig, movement::PrecisionDescriptor};
use crate::theater::parser::v41::reference::references::EventReference;
use crate::theater::parser::v41::replication::source::stream::models::kill_event_chain::EventRecord;
use crate::theater::parser::wire::bits::Cursor;

/// Outer absence means the layout cannot be projected. A missing required
/// value is an error, not permission to fall back to a kind-only event.
pub(super) fn project(
    record: &EventRecord,
    payload: &[u8],
    config: &V41DecodeConfig,
) -> Option<Result<ReplicationEvent, ParseError>> {
    let code = record.code?;
    if !record.layout_complete || !matches!(code, 0 | 5 | 6 | 9 | 21 | 36 | 38 | 39 | 40) {
        return None;
    }
    let body = record.body_range()?;
    let mut bits = Cursor::new(payload, body.start)?;
    let event = match code {
        38 => {
            bits.skip(4)?;
            required(player(&mut bits)?, "WeaponReload.player")
                .map(|player| ReplicationEvent::WeaponReload { player })
        }
        39 => {
            if bits.read(1)? != 0 {
                skip_tag(&mut bits)?;
                bits.skip(4)?;
            } else {
                bits.skip(3)?;
            }
            required(player(&mut bits)?, "Throw.player")
                .map(|player| ReplicationEvent::Throw { player })
        }
        40 => {
            let kind = bits.read(3)? as u8;
            if kind == 1 {
                bits.skip(2)?;
            }
            bits.skip(2)?;
            let weapon = optional_tag(&mut bits)?;
            let variant = WeaponVariantId(bits.read(32)? as u32);
            let player = player(&mut bits)?;
            (|| {
                Ok(ReplicationEvent::Melee {
                    weapon: WeaponTagId(required(weapon, "Melee.weapon")?.raw()),
                    variant,
                    player: required(player, "Melee.player")?,
                })
            })()
        }
        9 => {
            bits.skip(3)?;
            required(optional_tag(&mut bits)?, "Pickup.item")
                .map(|item| ReplicationEvent::Pickup { item })
        }
        21 => Ok(ReplicationEvent::Zoom {
            level: bits.read(2)? as u8,
        }),
        5 => {
            bits.skip(6)?;
            skip_projectile_variant(&mut bits)?;
            skip_tag(&mut bits)?;
            let position = point(
                &mut bits,
                config
                    .projectile_event_positions
                    .as_ref()
                    .map(|p| (&p.detonation, p.level)),
                15,
            )?;
            bits.skip(19 + 5 + 1 + 9)?;
            if bits.read(1)? != 0 {
                bits.skip(18)?;
            }
            let direction_bits = if bits.read(1)? != 0 { 19 } else { 8 };
            bits.skip(direction_bits + 2)?;
            Ok(ReplicationEvent::ProjectileDetonation { position })
        }
        6 => {
            skip_projectile_variant(&mut bits)?;
            bits.skip(14 + 19)?;
            let position = point(
                &mut bits,
                config
                    .projectile_event_positions
                    .as_ref()
                    .map(|p| (&p.impact, p.level)),
                12,
            )?;
            bits.skip(19 + 9 + 1)?;
            Ok(ReplicationEvent::ProjectileImpact { position })
        }
        0 => {
            let references = record.references?;
            let effect = optional_tag(&mut bits)?;
            let attacker_player = player(&mut bits)?;
            let packed_direction = bits.read(19)? as u32;
            if bits.read(1)? != 0 {
                bits.skip(31)?;
            }
            bits.skip(5)?;
            (|| {
                Ok(ReplicationEvent::DamageAftermath {
                    victim: event_reference(required(references.first, "DamageAftermath.victim")?),
                    attacker: references.second.map(event_reference),
                    effect: required(effect, "DamageAftermath.effect")?,
                    attacker_player,
                    packed_direction,
                })
            })()
        }
        36 => {
            bits.skip(10)?; // two mode flags, seven bits, one bit
            let player = player(&mut bits)?;
            if bits.read(1)? == 0 {
                bits.skip(2)?;
            }
            let weapon = optional_tag(&mut bits)?.map(|tag| WeaponTagId(tag.raw()));
            let variant = WeaponVariantId(bits.read(32)? as u32);
            let shooter = record.references?.first;
            (|| {
                Ok(ReplicationEvent::WeaponFire {
                    shooter: event_reference(required(shooter, "WeaponFire.shooter")?),
                    player: required(player, "WeaponFire.player")?,
                    weapon: required(weapon, "WeaponFire.weapon")?,
                    variant,
                })
            })()
        }
        _ => return None,
    };
    // A parser or layout change must not silently reinterpret bits beyond the
    // event's established source range.
    // Existing combat projections intentionally expose a prefix of their bodies.
    // New action/projectile layouts must close exactly before any guard trailer.
    (bits.position <= body.end && (matches!(code, 0 | 36) || bits.position == body.end))
        .then_some(event)
}

fn required<T>(value: Option<T>, field: &'static str) -> Result<T, ParseError> {
    value.ok_or(ParseError::MissingRequiredEventField { field })
}

fn event_reference(reference: EventReference) -> ReplicationEventReference {
    ReplicationEventReference {
        kind: reference.kind,
        index: reference.index,
        generation: reference.generation,
    }
}

/// Preserve the occurrence without retaining an uninterpreted payload.
pub(super) fn other(record: &EventRecord) -> Option<ReplicationEvent> {
    let kind = record.code?.into();
    record.body_range()?;
    Some(ReplicationEvent::Other { kind })
}

fn skip_projectile_variant(bits: &mut Cursor<'_>) -> Option<()> {
    if bits.read(1)? == 0 {
        skip_tag(bits)?;
        bits.skip(32)?;
    }
    Some(())
}

fn skip_tag(bits: &mut Cursor<'_>) -> Option<()> {
    if bits.read(1)? != 0 {
        bits.skip(32)?;
    }
    Some(())
}

fn point(
    bits: &mut Cursor<'_>,
    encoding: Option<(&PrecisionDescriptor, &LevelCalibration)>,
    precision: u8,
) -> Option<[f64; 3]> {
    let default_width = match precision {
        15 => 21,
        12 => 18,
        _ => return None,
    };
    if bits.read(1)? != 0 {
        let mut codes = [0; 3];
        for code in &mut codes {
            *code = bits.read(default_width as usize)? as u32;
        }
        // FUN_14076e524 selects DAT_143b8c6b8 (+/-20000), not map
        // bounds, when this gate is set. FUN_140c1e978 uses bin centers.
        let step = 40_000.0 / (1_u32 << default_width) as f64;
        return Some(codes.map(|code| -20_000.0 + (f64::from(code) + 0.5) * step));
    }
    let (encoding, level) = encoding?;
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
    level.region(region)?.bounds.dequantize_at_precision(
        codes,
        widths.map(|w| w as usize),
        precision,
    )
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
    use crate::theater::parser::v41::reference::references::read_triplet;
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
        decode_result(code, refs, body).unwrap()
    }

    fn assert_missing(code: u8, refs: &[(u64, usize)], body: &[(u64, usize)], expected: &str) {
        assert!(matches!(decode_result(code, refs, body),
            Err(ParseError::MissingRequiredEventField { field }) if field == expected));
    }

    fn decode_result(
        code: u8,
        refs: &[(u64, usize)],
        body: &[(u64, usize)],
    ) -> Result<ReplicationEvent, ParseError> {
        let prefix = [(1, 1), (u64::from(code), 7)];
        let fields: Vec<_> = prefix.iter().chain(refs).chain(body).copied().collect();
        let (payload, end_bit) = pack(&fields);
        let body_start_bit = 8 + refs.iter().map(|(_, width)| *width).sum::<usize>();
        let references = if matches!(code, 0 | 36) {
            let (bytes, length) = pack(refs);
            let mut bits = Cursor::new(&bytes, 0).unwrap();
            let kinds = if code == 0 { [1, 1, 7] } else { [1, 8, 7] };
            let references = read_triplet(kinds, |width| bits.read(width)).unwrap();
            assert_eq!(bits.position, length);
            Some(references)
        } else {
            None
        };
        let record = EventRecord {
            start_bit: 0,
            end_bit,
            code: Some(code),
            body_start_bit: Some(body_start_bit),
            references,
            fields: Vec::new(),
            layout_complete: true,
        };
        let event = if matches!(code, 32 | 85) {
            assert!(project(&record, &payload, &V41DecodeConfig::default()).is_none());
            other(&record).unwrap()
        } else {
            project(&record, &payload, &V41DecodeConfig::default()).unwrap()?
        };
        if matches!(code, 0 | 36) {
            let mut missing_header = record.clone();
            missing_header.references = None;
            assert!(project(&missing_header, &payload, &V41DecodeConfig::default()).is_none());
            // Projection uses the decoded envelope, not another read of its bits.
            let mut overwritten_header = payload.clone();
            for bit in 8..body_start_bit {
                overwritten_header[bit / 8] ^= 1 << (7 - bit % 8);
            }
            assert_eq!(
                project(&record, &overwritten_header, &V41DecodeConfig::default())
                    .transpose()
                    .unwrap(),
                Some(event.clone())
            );
        }
        let json = serde_json::to_value(&event).unwrap();
        for path in [
            "/WeaponReload/player",
            "/Throw/player",
            "/Melee/weapon",
            "/Melee/player",
            "/Pickup/item",
            "/DamageAftermath/victim",
            "/DamageAftermath/effect",
            "/WeaponFire/shooter",
            "/WeaponFire/player",
            "/WeaponFire/weapon",
            "/ProjectileDetonation/position",
            "/ProjectileImpact/position",
        ] {
            if json.pointer(path).is_none() {
                continue;
            }
            let mut invalid = json.clone();
            *invalid.pointer_mut(path).unwrap() = serde_json::Value::Null;
            assert!(
                serde_json::from_value::<ReplicationEvent>(invalid.clone()).is_err(),
                "{path} accepted null"
            );
            let (parent, key) = path.rsplit_once('/').unwrap();
            invalid
                .pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert!(
                serde_json::from_value::<ReplicationEvent>(invalid).is_err(),
                "{path} accepted missing field"
            );
        }
        for (kind, body) in json.as_object().unwrap() {
            let body = body.as_object().unwrap();
            assert!(!body.contains_key("handles"));
            assert!(!body.contains_key("third_reference"));
            for field in ["flag", "mode", "extra", "raw_scale", "kind", "selection"] {
                if kind == "Other" && field == "kind" {
                    continue;
                }
                assert!(!body.contains_key(field), "unexpected {field} in {kind}");
            }
            if kind == "Throw" {
                assert!(!body.contains_key("item"));
            }
            if matches!(kind.as_str(), "ProjectileDetonation" | "ProjectileImpact") {
                assert_eq!(
                    body.keys().map(String::as_str).collect::<Vec<_>>(),
                    ["position"]
                );
            }
        }
        assert_eq!(
            serde_json::from_value::<ReplicationEvent>(json).unwrap(),
            event
        );
        Ok(event)
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
                project(record, &payload, &V41DecodeConfig::default())
                    .transpose()
                    .unwrap(),
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
    fn empirically_required_fields_reject_absent_wire_gates() {
        let none = [(0, 3)];
        let unit = [(1, 1), (1, 1), (0, 9), (0, 2), (0, 2)];
        assert_missing(38, &none, &[(0, 4), (1, 1)], "WeaponReload.player");
        assert_missing(39, &none, &[(0, 1), (0, 3), (1, 1)], "Throw.player");
        assert_missing(
            40,
            &none,
            &[(0, 5), (0, 1), (0, 32), (0, 6)],
            "Melee.weapon",
        );
        assert_missing(
            40,
            &none,
            &[(0, 5), (1, 1), (0, 32), (0, 32), (1, 1)],
            "Melee.player",
        );
        assert_missing(9, &none, &[(0, 3), (0, 1)], "Pickup.item");
        assert_missing(
            0,
            &none,
            &[(1, 1), (0, 32), (1, 1), (0, 19), (0, 6)],
            "DamageAftermath.victim",
        );
        assert_missing(
            0,
            &unit,
            &[(0, 1), (1, 1), (0, 19), (0, 6)],
            "DamageAftermath.effect",
        );
        assert_missing(
            36,
            &none,
            &[(0, 10), (0, 6), (1, 1), (1, 1), (0, 32), (0, 32)],
            "WeaponFire.shooter",
        );
        assert_missing(
            36,
            &unit,
            &[(0, 10), (1, 1), (1, 1), (1, 1), (0, 32), (0, 32)],
            "WeaponFire.player",
        );
        assert_missing(
            36,
            &unit,
            &[(0, 10), (0, 6), (1, 1), (0, 1), (0, 32)],
            "WeaponFire.weapon",
        );
        assert_eq!(
            decode(
                5,
                &none,
                &[
                    (0, 6),
                    (0, 1),
                    (0, 1),
                    (0, 32), // variant present, tag absent
                    (0, 1),
                    (1, 1),
                    (0, 63), // detonation tag absent, default position
                    (0, 34),
                    (0, 1),
                    (0, 1),
                    (0, 10),
                ],
            ),
            ReplicationEvent::ProjectileDetonation {
                position: [-19999.990463256836; 3],
            }
        );
        assert_eq!(
            decode(
                6,
                &none,
                &[
                    (0, 1),
                    (0, 1),
                    (0, 32), // variant present, tag absent
                    (0, 33),
                    (1, 1),
                    (0, 54),
                    (0, 29),
                ],
            ),
            ReplicationEvent::ProjectileImpact {
                position: [-19999.923706054688; 3],
            }
        );
    }

    #[test]
    fn combat_body_slots_and_weapon_tags_preserve_zero_and_absence() {
        assert_eq!(
            decode(
                36,
                &[(1, 1), (1, 1), (0, 9), (0, 2), (0, 2)],
                &[(0, 10), (0, 1), (0, 5), (1, 1), (1, 1), (0, 32), (0, 32)]
            ),
            ReplicationEvent::WeaponFire {
                shooter: ReplicationEventReference {
                    kind: 4,
                    index: 0,
                    generation: 0
                },
                player: EventPlayerSlot(0),
                weapon: WeaponTagId(0),
                variant: WeaponVariantId(0),
            }
        );
        assert_eq!(
            decode(
                85,
                &[],
                &[(1, 1), (1, 1), (0, 32), (0, 1), (0, 1), (31, 5), (100, 32)]
            ),
            ReplicationEvent::Other { kind: 85.into() }
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
                victim: ReplicationEventReference {
                    kind: 4,
                    index: 5,
                    generation: 2
                },
                attacker: None,
                effect: TagId(0x1234_5678),
                attacker_player: Some(EventPlayerSlot(7)),
                packed_direction: 123,
            }
        );
        assert_eq!(
            decode(
                36,
                &[(1, 1), (1, 1), (3, 9), (1, 2), (0, 2)],
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
                shooter: ReplicationEventReference {
                    kind: 4,
                    index: 3,
                    generation: 1
                },
                player: EventPlayerSlot(3),
                weapon: WeaponTagId(0x0000_8594),
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
            ReplicationEvent::Other { kind: 85.into() }
        );
    }

    #[test]
    fn damage_projection_keeps_attacker_but_omits_unexplained_third_reference() {
        assert_eq!(
            decode(
                0,
                &[
                    (1, 1),
                    (1, 1),
                    (0, 9),
                    (0, 2),
                    (1, 1),
                    (0, 1),
                    (123, 13),
                    (1, 2),
                    (1, 1),
                    (456, 13),
                    (2, 2),
                ],
                &[(1, 1), (0, 32), (1, 1), (0, 19), (0, 1), (2, 5)],
            ),
            ReplicationEvent::DamageAftermath {
                victim: ReplicationEventReference {
                    kind: 4,
                    index: 0,
                    generation: 0
                },
                attacker: Some(ReplicationEventReference {
                    kind: 1,
                    index: 123,
                    generation: 1
                }),
                effect: TagId(0),
                attacker_player: None,
                packed_direction: 0,
            }
        );
    }

    #[test]
    fn meaningful_references_preserve_presence_domains_and_generations() {
        for mask in 0..8 {
            for modes in 0..4 {
                let mut refs = Vec::new();
                let mut expected = [None; 3];
                for (slot, reference) in expected.iter_mut().enumerate() {
                    let present = mask & (1 << slot) != 0;
                    refs.push((u64::from(present), 1));
                    if present {
                        let narrow = slot < 2 && modes & (1 << slot) != 0;
                        if slot < 2 {
                            refs.push((u64::from(narrow), 1));
                        }
                        // Zero is a valid present index, not a sentinel for absence.
                        let index = if slot == 0 {
                            0
                        } else if narrow {
                            511
                        } else {
                            8191
                        };
                        refs.extend([(index, if narrow { 9 } else { 13 }), (slot as u64 + 1, 2)]);
                        *reference = Some(ReplicationEventReference {
                            kind: if slot == 2 {
                                7
                            } else if narrow {
                                4
                            } else {
                                1
                            },
                            index: index as u16,
                            generation: slot as u8 + 1,
                        });
                    }
                }
                let body = [(1, 1), (0, 32), (1, 1), (0, 19), (0, 1), (0, 5)];
                let Some(victim) = expected[0] else {
                    assert_missing(0, &refs, &body, "DamageAftermath.victim");
                    continue;
                };
                assert_eq!(
                    decode(0, &refs, &body),
                    ReplicationEvent::DamageAftermath {
                        victim,
                        attacker: expected[1],
                        effect: TagId(0),
                        attacker_player: None,
                        packed_direction: 0,
                    }
                );
            }
        }
        for narrow in [false, true] {
            let refs = [
                (1, 1),
                (u64::from(narrow), 1),
                (0, if narrow { 9 } else { 13 }),
                (3, 2),
                (0, 1),
                (0, 1),
            ];
            assert_eq!(
                decode(
                    36,
                    &refs,
                    &[(0, 10), (0, 1), (0, 5), (1, 1), (1, 1), (0, 32), (0, 32)]
                ),
                ReplicationEvent::WeaponFire {
                    shooter: ReplicationEventReference {
                        kind: if narrow { 4 } else { 1 },
                        index: 0,
                        generation: 3
                    },
                    player: EventPlayerSlot(0),
                    weapon: WeaponTagId(0),
                    variant: WeaponVariantId(0),
                }
            );
        }
    }

    #[test]
    fn action_payloads_keep_known_fields_while_consuming_unknown_codes() {
        let refs = [(0, 1); 3];
        assert_eq!(
            decode(38, &refs, &[(9, 4), (0, 1), (17, 5)]),
            ReplicationEvent::WeaponReload {
                player: EventPlayerSlot(17)
            }
        );
        assert_missing(38, &refs, &[(0, 4), (1, 1)], "WeaponReload.player");
        assert_eq!(
            decode(39, &refs, &[(0, 1), (7, 3), (0, 1), (0, 5)]),
            ReplicationEvent::Throw {
                player: EventPlayerSlot(0)
            }
        );
        assert_eq!(
            decode(
                39,
                &refs,
                &[(1, 1), (1, 1), (42, 32), (4, 4), (0, 1), (3, 5)]
            ),
            ReplicationEvent::Throw {
                player: EventPlayerSlot(3)
            }
        );
        assert_eq!(
            decode(39, &refs, &[(1, 1), (0, 1), (2, 4), (0, 1), (0, 5)]),
            ReplicationEvent::Throw {
                player: EventPlayerSlot(0)
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
                weapon: WeaponTagId(42),
                variant: WeaponVariantId(123),
                player: EventPlayerSlot(5)
            }
        );
        assert_missing(
            40,
            &refs,
            &[(0, 3), (1, 2), (0, 1), (0, 32), (1, 1)],
            "Melee.weapon",
        );
        for tag in [None, Some(42)] {
            let mut body = vec![(5, 3), (u64::from(tag.is_some()), 1)];
            if let Some(tag) = tag {
                body.push((tag, 32));
            } else {
                assert_missing(9, &refs, &body, "Pickup.item");
                continue;
            }
            assert_eq!(
                decode(9, &refs, &body),
                ReplicationEvent::Pickup {
                    item: TagId(tag.unwrap() as u32)
                }
            );
        }
        assert_eq!(
            decode(21, &refs, &[(2, 2)]),
            ReplicationEvent::Zoom { level: 2 }
        );
    }

    #[test]
    fn projectile_positions_remain_and_teleports_are_kind_only() {
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
            ReplicationEvent::Other { kind: 32.into() }
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
                project(&list.records[0], &payload, &V41DecodeConfig::default())
                    .unwrap()
                    .unwrap(),
                decode(5, &refs, &body)
            );
            let truncated = read_event_list(&payload[..(body_end - 1) / 8], 0, 3, layout());
            assert!(!truncated.records[0].layout_complete);
            assert!(other(&truncated.records[0]).is_none());
            assert_eq!(
                decode(5, &refs, &body),
                ReplicationEvent::ProjectileDetonation {
                    position: [-19999.971389770508, -19999.95231628418, -19999.93324279785],
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
                position: [
                    -19999.771118164062,
                    -19999.618530273438,
                    -19999.465942382812
                ],
            }
        );
    }

    #[test]
    fn retired_event_payloads_preserve_the_following_event() {
        for (code, body) in [
            (
                32,
                vec![
                    (1, 1),
                    (0, 22),
                    (0, 22),
                    (0, 22),
                    (1, 1),
                    (0, 22),
                    (0, 22),
                    (0, 22),
                    (1, 1),
                    (42, 32),
                    (0, 1),
                ],
            ),
            (
                85,
                vec![
                    (0, 1),
                    (4, 5),
                    (0, 1),
                    (9, 5),
                    (65, 32),
                    (0, 1),
                    (1, 1),
                    (0, 32),
                ],
            ),
        ] {
            let mut fields = vec![(1, 1), (code, 7), (0, 3)];
            fields.extend(body);
            let next_start: usize = fields.iter().map(|(_, width)| width).sum();
            fields.extend([(1, 1), (21, 7), (0, 3), (2, 2), (0, 1)]);
            let (payload, end) = pack(&fields);
            let list = read_event_list(
                &payload,
                0,
                3,
                EventLayout {
                    corruption_check: Some(false),
                    kill_tail: Some(false),
                    ..EventLayout::default()
                },
            );
            assert_eq!(list.stop, EventListStop::Terminator, "code {code}");
            assert_eq!(list.end_bit, end);
            assert_eq!(list.records.len(), 2);
            assert!(project(&list.records[0], &payload, &V41DecodeConfig::default()).is_none());
            assert_eq!(
                other(&list.records[0]),
                Some(ReplicationEvent::Other {
                    kind: (code as u8).into()
                })
            );
            assert_eq!(list.records[1].start_bit, next_start);
            assert_eq!(
                project(&list.records[1], &payload, &V41DecodeConfig::default())
                    .unwrap()
                    .unwrap(),
                ReplicationEvent::Zoom { level: 2 }
            );
            assert!(
                serde_json::from_value::<ReplicationEvent>(serde_json::json!({
                    if code == 32 { "Teleport" } else { "Kill" }: {}
                }))
                .is_err()
            );
        }
    }

    #[test]
    fn discarded_projectile_identifiers_still_obey_presence_and_bounds() {
        for mut fields in [
            vec![(1, 1)],
            vec![(0, 1), (0, 1), (123, 32)],
            vec![(0, 1), (1, 1), (42, 32), (123, 32)],
        ] {
            let next_start: usize = fields.iter().map(|(_, width)| width).sum();
            fields.push((21, 5));
            let (payload, _) = pack(&fields);
            let mut bits = Cursor::new(&payload, 0).unwrap();
            assert_eq!(skip_projectile_variant(&mut bits), Some(()));
            assert_eq!(bits.position, next_start);
            assert_eq!(bits.read(5), Some(21));
        }
        assert!(skip_tag(&mut Cursor::new(&[0x80], 0).unwrap()).is_none());
        assert!(skip_projectile_variant(&mut Cursor::new(&[0], 0).unwrap()).is_none());
        assert!(skip_projectile_variant(&mut Cursor::new(&[0x40], 0).unwrap()).is_none());
    }

    #[test]
    fn region_positions_require_the_recorded_region_and_axis_widths() {
        use crate::theater::film::catalog::maps::{PositionBounds, levels};
        let encoding = PrecisionDescriptor {
            index_bits: 2,
            axis_bits: [3, 4, 5],
            region: 2,
            regions: Default::default(),
        };
        let level = LevelCalibration {
            play: levels::LevelRegion {
                bounds: PositionBounds {
                    region: 2,
                    min: [0.0; 3],
                    max: [0.25, 0.5, 1.0],
                },
                axis_bits: [4, 5, 6],
            },
            others: &[],
            ..levels::all()[0]
        };
        let (data, _) = pack(&[(0, 1), (2, 2), (7, 3), (15, 4), (31, 5)]);
        assert_eq!(
            point(
                &mut Cursor::new(&data, 0).unwrap(),
                Some((&encoding, &level)),
                15
            ),
            Some([0.234375, 0.484375, 0.984375])
        );
        assert!(point(&mut Cursor::new(&data, 0).unwrap(), None, 15).is_none());
        let mut other = encoding.clone();
        other.region = 1;
        assert!(
            point(
                &mut Cursor::new(&data, 0).unwrap(),
                Some((&other, &level)),
                15
            )
            .is_none()
        );
        // Same codes and widths must not be decoded using another precision's bounds.
        assert!(
            point(
                &mut Cursor::new(&data, 0).unwrap(),
                Some((&encoding, &level)),
                12
            )
            .is_none()
        );
        let mut missing_region = level;
        missing_region.play.bounds.region = 1;
        assert!(
            point(
                &mut Cursor::new(&data, 0).unwrap(),
                Some((&encoding, &missing_region)),
                15
            )
            .is_none()
        );
        assert!(
            point(
                &mut Cursor::new(&data[..1], 0).unwrap(),
                Some((&encoding, &level)),
                15
            )
            .is_none()
        );
    }

    #[test]
    fn default_positions_decode_bin_centers_at_both_event_precisions() {
        for (precision, width, half_step) in
            [(15, 21, 0.0095367431640625), (12, 18, 0.0762939453125)]
        {
            let (data, length) = pack(&[
                (1, 1),
                (0, width),
                (1 << (width - 1), width),
                ((1 << width) - 1, width),
            ]);
            let mut bits = Cursor::new(&data, 0).unwrap();
            assert_eq!(
                point(&mut bits, None, precision),
                Some([-20000.0 + half_step, half_step, 20000.0 - half_step])
            );
            assert_eq!(bits.position, length);
            assert!(
                point(
                    &mut Cursor::new(&data[..data.len() - 1], 0).unwrap(),
                    None,
                    precision
                )
                .is_none()
            );
        }
    }

    #[test]
    fn all_catalogued_regions_decode_at_both_event_precisions() {
        use crate::theater::film::catalog::maps::levels;
        use crate::theater::parser::v41::context::map::level_precision;
        for level in levels::all() {
            for precision in [12, 15] {
                let encoding = level_precision(level, precision).unwrap();
                for region in level.regions() {
                    let widths = encoding.axes_for_region(region.bounds.region).unwrap();
                    let codes = [0, (1 << widths[1]) / 2, (1 << widths[2]) - 1];
                    let mut fields = vec![(0, 1), (region.bounds.region as u64, level.index_bits)];
                    fields.extend(codes.into_iter().zip(widths.map(|w| w as usize)));
                    let (data, length) = pack(&fields);
                    let mut bits = Cursor::new(&data, 0).unwrap();
                    let xyz = point(&mut bits, Some((&encoding, level)), precision).unwrap();
                    assert_eq!(bits.position, length);
                    for axis in 0..3 {
                        let min = f64::from(region.bounds.min[axis]);
                        let max = f64::from(region.bounds.max[axis]);
                        let step = (max - min) / (1_u64 << widths[axis]) as f64;
                        assert!(
                            (xyz[axis] - (min + (codes[axis] as f64 + 0.5) * step)).abs() < 1e-9
                        );
                        assert!(xyz[axis] > min && xyz[axis] < max);
                    }
                }
            }
        }
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
