//! Count-driven type 8/12 readers. Shared body: reference analysis 413971b
//! player_table_record.go; wrappers: roster_type8.go and bot_metadata.go.
use crate::theater::film::{
    BotAbsoluteIndex, BotId, BotPlayer, BotSlot, InitialPlayer, InitialPlayerSlot, ParseError,
    PlayerRecord, RosterPlayer, Xuid,
};
use crate::theater::parser::v41::context::players::PlayerContext;
use crate::theater::parser::v41::packet::BitRange;
use crate::theater::parser::v41::replication::source::{
    PlayerMetadataEntry, PlayerMetadataField, PlayerMetadataFieldKind as K, PlayerMetadataRead,
    PlayerMetadataStop as Stop,
};
use crate::theater::parser::wire::fields::RawBits;

struct Reader<'a> {
    data: &'a [u8],
    bit: usize,
    fields: Vec<PlayerMetadataField>,
}
impl Reader<'_> {
    fn field(&mut self, kind: K, width: usize) -> Result<(), Stop> {
        let raw = RawBits::from_source(self.data, self.bit, width);
        let available = raw.is_some();
        self.fields.push(PlayerMetadataField {
            kind,
            bit: self.bit,
            width,
            raw,
        });
        if !available {
            return Err(Stop::Truncated);
        }
        self.bit += width;
        Ok(())
    }
    fn word(&mut self, kind: K, width: usize) -> Result<u64, Stop> {
        self.field(kind, width)?;
        Ok(self.fields.last().unwrap().raw.as_ref().unwrap().low_u64())
    }
    fn count(&mut self, kind: K, width: usize, limit: u64) -> Result<usize, Stop> {
        let count = self.word(kind, width)?;
        if count > limit {
            return Err(Stop::CountLimit { kind, count, limit });
        }
        Ok(count as usize)
    }
    fn body(&mut self, context: &PlayerContext) -> Result<(), Stop> {
        let mask = self.word(K::MaskLengthMinusOne, 11)? as usize + 1;
        self.field(K::Mask, mask)?;
        let bytes = self.count(K::ByteCount, 12, 2048)?;
        self.field(K::ByteList, bytes * 8)?;
        let words = self.count(K::WordCount, 8, 192)?;
        self.field(K::WordList, words * 32)?;
        self.field(K::Block104, 832)?;
        for i in 0..16 {
            if self.word(K::GamertagUnit(i), 16)? == 0 {
                break;
            }
        }
        self.field(K::Block16, 128)?;
        self.word(K::Representation, 32)?;
        self.word(K::JoinKey, 64)?;
        for (i, width) in [10, 14, 6, 8, 7, 1].into_iter().enumerate() {
            self.word(K::Short(i as u8), width)?;
        }
        let bytes = context
            .personalization_bytes
            .ok_or(Stop::UnknownPersonalization)?;
        self.field(K::Personalization, bytes * 8)?;
        for i in 0..44 {
            self.field(K::Block44Byte(i), 8)?;
        }
        Ok(())
    }
    fn entry(&mut self, bots: bool, context: &PlayerContext) -> Result<(), Stop> {
        // Admission is explicit. A newer, unknown format never inherits 27.
        let version = context
            .format_version
            .filter(|v| matches!(v, 20 | 21 | 24 | 25 | 27))
            .ok_or(Stop::UnsupportedFormat)?;
        if bots {
            self.word(K::AbsoluteIndex, 32)?;
            self.word(K::Slot, 32)?;
            self.word(K::BotId, 32)?;
        } else {
            if version >= 21 && self.word(K::OptionAbsent, 1)? == 0 {
                self.word(K::Option, 5)?;
            }
            self.word(K::Identity, 64)?;
        }
        self.body(context)
    }
}

pub(crate) fn decode(data: &[u8], bots: bool, context: &PlayerContext) -> PlayerMetadataRead {
    let mut r = Reader {
        data,
        bit: 0,
        fields: Vec::new(),
    };
    let count = r.word(K::Count, 32);
    let mut stop = count.as_ref().err().copied().unwrap_or(Stop::Complete);
    let mut out = PlayerMetadataRead {
        format_version: context.format_version,
        build: context.build.clone(),
        personalization_bytes: context.personalization_bytes,
        count: r.fields.pop().unwrap(),
        entries: Vec::new(),
        end_bit: r.bit,
        stop,
        tail: BitRange {
            start: r.bit,
            end: data.len() * 8,
        },
    };
    if let Ok(count) = count {
        if bots && count > 64 {
            stop = Stop::CountLimit {
                kind: K::Count,
                count,
                limit: 64,
            };
        } else {
            // No allocation from an untrusted count. Each successful iteration
            // consumes a nonempty entry, and the first refusal ends traversal.
            for _ in 0..count {
                let start_bit = r.bit;
                stop = r.entry(bots, context).err().unwrap_or(Stop::Complete);
                out.entries.push(PlayerMetadataEntry {
                    start_bit,
                    end_bit: r.bit,
                    fields: std::mem::take(&mut r.fields),
                    stop,
                });
                if stop != Stop::Complete {
                    break;
                }
            }
            if bots && stop == Stop::Complete && data.len() * 8 - r.bit >= 8 {
                stop = Stop::TrailingData;
            }
        }
    }
    out.end_bit = r.bit;
    out.tail.start = r.bit;
    out.stop = stop;
    out
}

fn value(fields: &[PlayerMetadataField], kind: K) -> Option<&RawBits> {
    fields.iter().find(|f| f.kind == kind)?.raw.as_ref()
}

fn scalar(fields: &[PlayerMetadataField], kind: K) -> Option<u64> {
    Some(value(fields, kind)?.low_u64())
}

fn player(
    fields: &[PlayerMetadataField],
    build: Option<&str>,
) -> Result<Option<PlayerRecord>, ParseError> {
    let raw = value(fields, K::Personalization).ok_or(ParseError::InvalidCustomization)?;
    let customization = crate::theater::parser::v41::customization::decode(&raw.bytes, build)?;
    Ok((|| {
        Some(PlayerRecord {
            gamertag: gamertag(fields)?,
            customization,
        })
    })())
}

fn gamertag(fields: &[PlayerMetadataField]) -> Option<String> {
    let mut units = [0; 16];
    let mut len = 0;
    for field in fields {
        if let K::GamertagUnit(_) = field.kind {
            let unit = field.raw.as_ref()?.low_u64() as u16;
            if unit == 0 {
                break;
            }
            *units.get_mut(len)? = unit;
            len += 1;
        }
    }
    Some(String::from_utf16_lossy(&units[..len]))
}

/// Only complete packets become snapshots: a truncated roster must not remove players.
pub(crate) fn roster(read: &PlayerMetadataRead) -> Result<Option<Vec<RosterPlayer>>, ParseError> {
    if read.stop == Stop::UnknownPersonalization {
        return Err(ParseError::InvalidCustomization);
    }
    if read.stop != Stop::Complete {
        return Ok(None);
    }
    let mut players = Vec::with_capacity(read.entries.len());
    for e in &read.entries {
        let Some(player) = player(&e.fields, read.build.as_deref())? else {
            return Ok(None);
        };
        let Some(identity) = scalar(&e.fields, K::Identity) else {
            return Ok(None);
        };
        players.push(RosterPlayer {
            xuid: Xuid(identity.swap_bytes()),
            player,
        });
    }
    Ok(Some(players))
}

pub(crate) fn bots(read: &PlayerMetadataRead) -> Result<Option<Vec<BotPlayer>>, ParseError> {
    if read.stop == Stop::UnknownPersonalization {
        return Err(ParseError::InvalidCustomization);
    }
    if read.stop != Stop::Complete {
        return Ok(None);
    }
    let mut players = Vec::with_capacity(read.entries.len());
    for e in &read.entries {
        let Some(player) = player(&e.fields, read.build.as_deref())? else {
            return Ok(None);
        };
        let Some(absolute_index) = scalar(&e.fields, K::AbsoluteIndex) else {
            return Ok(None);
        };
        let Some(slot) = scalar(&e.fields, K::Slot) else {
            return Ok(None);
        };
        let Some(bot_id) = scalar(&e.fields, K::BotId) else {
            return Ok(None);
        };
        players.push(BotPlayer {
            absolute_index: BotAbsoluteIndex(absolute_index as u32),
            slot: BotSlot(slot as u32),
            bot_id: BotId(bot_id as u32),
            player,
        });
    }
    Ok(Some(players))
}

/// The game-options reader establishes the boundary. No roster search or XUID heuristic.
pub(crate) fn initial(
    data: &[u8],
    start: usize,
    context: &PlayerContext,
) -> Result<Vec<InitialPlayer>, ParseError> {
    context
        .format_version
        .filter(|v| matches!(v, 20 | 21 | 24 | 25 | 27))
        .ok_or(ParseError::IncompleteRegistryPlayers)?;
    let mut r = Reader {
        data,
        bit: start,
        fields: Vec::new(),
    };
    let take = |r: &mut Reader<'_>, width| -> Result<u64, ParseError> {
        let mut cursor = crate::theater::parser::wire::bits::Cursor::new(r.data, r.bit)
            .ok_or(ParseError::IncompleteRegistryPlayers)?;
        let raw = cursor
            .read(width)
            .ok_or(ParseError::IncompleteRegistryPlayers)?;
        r.bit = cursor.position;
        Ok(raw)
    };
    let skip = |r: &mut Reader<'_>, width| -> Result<(), ParseError> {
        let mut cursor = crate::theater::parser::wire::bits::Cursor::new(r.data, r.bit)
            .ok_or(ParseError::IncompleteRegistryPlayers)?;
        cursor
            .skip(width)
            .ok_or(ParseError::IncompleteRegistryPlayers)?;
        r.bit = cursor.position;
        Ok(())
    };
    let mut players = Vec::with_capacity(32);
    for slot in 0..32 {
        skip(&mut r, 3 + 32 + 2 + 48)?;
        let xuid = Xuid(take(&mut r, 64)?);
        r.body(context).map_err(|stop| match stop {
            Stop::UnknownPersonalization => ParseError::InvalidCustomization,
            _ => ParseError::IncompleteRegistryPlayers,
        })?;
        let player = player(&r.fields, context.build.as_deref())?
            .ok_or(ParseError::IncompleteRegistryPlayers)?;
        r.fields.clear();
        skip(&mut r, 32)?;
        players.push(InitialPlayer {
            slot: InitialPlayerSlot(slot),
            xuid,
            player,
        });
    }
    Ok(players)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> PlayerContext {
        PlayerContext {
            format_version: Some(27),
            build: Some("HI_1_13_0".into()),
            personalization_bytes: Some(1852),
        }
    }

    fn pack(fields: &[(u64, usize)]) -> Vec<u8> {
        let mut data = vec![0; fields.iter().map(|(_, n)| n).sum::<usize>().div_ceil(8)];
        let mut at = 0;
        for &(value, width) in fields {
            for bit in (0..width).rev() {
                if bit < 64 {
                    data[at / 8] |= (((value >> bit) & 1) as u8) << (7 - at % 8);
                }
                at += 1;
            }
        }
        data
    }

    fn body() -> Vec<(u64, usize)> {
        vec![
            (0, 11),
            (1, 1),
            (1, 12),
            (0xab, 8),
            (1, 8),
            (0x12345678, 32),
            (0, 832),
            ('A' as u64, 16),
            (0, 16),
            (0, 128),
            (12, 32),
            (0x0102030405060708, 64),
            (1, 10),
            (2, 14),
            (3, 6),
            (4, 8),
            (5, 7),
            (1, 1),
            (0, 1852 * 8),
            (0, 44 * 8),
        ]
    }

    fn invalid_appearance_body() -> Vec<(u64, usize)> {
        let mut fields = body();
        fields.splice(18..19, [(0x22000000, 32), (0, 1852 * 8 - 32)]);
        fields
    }

    #[test]
    fn invalid_customization_fails_roster_bot_and_initial_projection() {
        // A valid first record must not hide an invalid appearance in a later record.
        for is_bot in [false, true] {
            let mut fields = vec![(2, 32)];
            for body in [body(), invalid_appearance_body()] {
                if is_bot {
                    fields.extend([(17, 32), (3, 32), (42, 32)]);
                } else {
                    fields.extend([(1, 1), (100, 64)]);
                }
                fields.extend(body);
            }
            let read = decode(&pack(&fields), is_bot, &context());
            assert_eq!(read.stop, Stop::Complete);
            if is_bot {
                assert!(matches!(bots(&read), Err(ParseError::InvalidCustomization)));
            } else {
                assert!(matches!(
                    roster(&read),
                    Err(ParseError::InvalidCustomization)
                ));
            }
        }
        let mut fields = Vec::new();
        for slot in 0..32 {
            fields.extend([(5, 3), (12, 32), (2, 2), (123, 48), (slot + 100, 64)]);
            fields.extend(if slot == 31 {
                invalid_appearance_body()
            } else {
                body()
            });
            fields.push((99, 32));
        }
        assert!(matches!(
            initial(&pack(&fields), 0, &context()),
            Err(ParseError::InvalidCustomization)
        ));
    }

    #[test]
    fn complete_roster_preserves_identity_without_opaque_player_values() {
        let xuid = 2535472547643888u64;
        for (format, option) in [(27, Some(7)), (27, None), (20, None)] {
            let mut fields = vec![(1, 32)];
            if format >= 21 {
                fields.push((u64::from(option.is_none()), 1));
                if let Some(n) = option {
                    fields.push((n, 5));
                }
            }
            fields.push((xuid.swap_bytes(), 64));
            fields.extend(body());
            let data = pack(&fields);
            let mut context = context();
            context.format_version = Some(format);
            let read = decode(&data, false, &context);
            let players = roster(&read).unwrap().unwrap();
            assert_eq!(players.len(), 1);
            assert_eq!(players[0].xuid, Xuid(xuid));
            assert_eq!(players[0].player.gamertag, "A");
            let json = serde_json::to_value(&players[0]).unwrap();
            assert_eq!(json.as_object().unwrap().len(), 2);
            assert_eq!(json["player"].as_object().unwrap().len(), 2);
            assert!(json["player"].get("customization").is_some());
            assert!(
                roster(&decode(&data[..data.len() - 1], false, &context))
                    .unwrap()
                    .is_none()
            );
            context.build = Some("future".into());
            context.personalization_bytes = None;
            assert!(matches!(
                roster(&decode(&data, false, &context)),
                Err(ParseError::InvalidCustomization)
            ));
        }
    }

    #[test]
    fn empty_and_truncated_snapshots_are_distinct_and_bots_keep_their_ids() {
        assert_eq!(
            roster(&decode(&[0; 4], false, &context())).unwrap(),
            Some(vec![])
        );
        assert_eq!(
            bots(&decode(&[0; 4], true, &context())).unwrap(),
            Some(vec![])
        );
        assert!(
            roster(&decode(&[0; 3], false, &context()))
                .unwrap()
                .is_none()
        );
        assert!(
            roster(&decode(&[0, 0, 0, 1], false, &context()))
                .unwrap()
                .is_none()
        );
        let mut fields = vec![(1, 32), (17, 32), (3, 32), (42, 32)];
        fields.extend(body());
        let data = pack(&fields);
        let players = bots(&decode(&data, true, &context())).unwrap().unwrap();
        assert_eq!(
            (
                players[0].absolute_index,
                players[0].slot,
                players[0].bot_id
            ),
            (BotAbsoluteIndex(17), BotSlot(3), BotId(42))
        );
        assert_eq!(players[0].player.gamertag, "A");
        assert!(
            bots(&decode(&data[..data.len() - 1], true, &context()))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn gamertags_decode_without_a_duplicate_utf16_buffer() {
        for (units, expected) in [
            (vec![0], String::new()),
            (vec![65, 0], "A".into()),
            (vec![0xd83d, 0xde00, 0], "😀".into()),
            (vec![0xd800, 65, 0], "�A".into()),
            (vec![65; 16], "A".repeat(16)),
        ] {
            let mut fields = vec![(1, 32), (1, 1), (100u64.swap_bytes(), 64)];
            let mut body = body();
            body.splice(7..9, units.into_iter().map(|unit| (unit, 16)));
            fields.extend(body);
            let read = decode(&pack(&fields), false, &context());
            let players = roster(&read).unwrap().unwrap();
            assert_eq!(players[0].player.gamertag, expected);
        }
    }

    #[test]
    fn initial_table_requires_all_slots_and_a_supported_layout() {
        let mut fields = vec![(5, 3)]; // deliberately unaligned table
        for slot in 0..32 {
            fields.extend([(5, 3), (12, 32), (2, 2), (123, 48), (slot + 100, 64)]);
            fields.extend(body());
            fields.push((99, 32));
        }
        let data = pack(&fields);
        let players = initial(&data, 3, &context()).unwrap();
        assert_eq!(players.len(), 32);
        assert_eq!(players[31].xuid, Xuid(131));
        assert_eq!(players[31].slot, InitialPlayerSlot(31));
        assert_eq!(players[31].player.gamertag, "A");
        assert_eq!(
            serde_json::to_value(&players[31])
                .unwrap()
                .as_object()
                .unwrap()
                .len(),
            3
        );
        assert!(initial(&data[..data.len() - 1], 3, &context()).is_err());
        assert!(
            initial(
                &data,
                3,
                &PlayerContext {
                    format_version: Some(999),
                    ..context()
                }
            )
            .is_err()
        );
        assert!(
            initial(
                &data,
                3,
                &PlayerContext {
                    personalization_bytes: None,
                    ..context()
                }
            )
            .is_err()
        );
    }
}
