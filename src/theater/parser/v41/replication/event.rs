//! Project established event-body fields without correlating separate events.
use crate::theater::film::{ReplicationEvent, ReplicationEventHandle, TagId, WeaponVariantId};
use crate::theater::parser::v41::replication::source::stream::models::kill_event_chain::EventRecord;
use crate::theater::parser::v41::replication::stream::frame::events::reference_kinds;
use crate::theater::parser::wire::bits::Cursor;

pub(super) fn project(record: &EventRecord, payload: &[u8]) -> Option<ReplicationEvent> {
    let code = record.code?;
    if !matches!(code, 0 | 36 | 85) {
        return None;
    }
    let handles = handles(record, payload, code)?;
    let mut bits = Cursor::new(payload, record.body_start_bit?)?;
    let event = match code {
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
    (bits.position <= record.end_bit).then_some(event)
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
        project(&record, &payload).unwrap()
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
        assert!(project(&record, &payload).is_none());
    }
}
