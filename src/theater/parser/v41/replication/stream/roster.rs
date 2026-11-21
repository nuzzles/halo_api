//! Count-driven type 8/12 readers. Shared body: reference analysis 413971b
//! player_table_record.go; wrappers: roster_type8.go and bot_metadata.go.
use crate::theater::film::{
    BotAbsoluteIndex, BotId, BotPlayer, BotSlot, InitialPlayer, InitialPlayerSlot, ParseError,
    PlayerRecord, RosterPlayer, Xuid,
};
use crate::theater::parser::v41::context::players::PlayerContext;
use crate::theater::parser::v41::packet::{BitRange, SourceRegion, SourceRegionKind};
use crate::theater::parser::v41::replication::source::{
    PlayerMetadataBody, PlayerMetadataEntry, PlayerMetadataFieldKind as K, PlayerMetadataIdentity,
    PlayerMetadataRead, PlayerMetadataStop as Stop,
};
use crate::theater::parser::wire::bits::Cursor;

struct Reader<'a> {
    cursor: Cursor<'a>,
    // Initial-player projection needs no per-field coverage allocation.
    regions: Option<Vec<SourceRegion>>,
}
impl Reader<'_> {
    fn region(&mut self, start: usize, kind: SourceRegionKind) {
        let Some(regions) = &mut self.regions else {
            return;
        };
        let end = self.cursor.position;
        if start == end {
            return;
        }
        if let Some(last) = regions.last_mut()
            && last.kind == kind
            && last.source.end == start
        {
            last.source.end = end;
        } else {
            regions.push(SourceRegion {
                source: BitRange { start, end },
                kind,
            });
        }
    }
    fn skip(&mut self, width: usize, kind: SourceRegionKind) -> Result<(), Stop> {
        let start = self.cursor.position;
        self.cursor.skip(width).ok_or(Stop::Truncated)?;
        self.region(start, kind);
        Ok(())
    }
    fn word(&mut self, width: usize) -> Result<u64, Stop> {
        let start = self.cursor.position;
        let value = self.cursor.read(width).ok_or(Stop::Truncated)?;
        self.region(start, SourceRegionKind::Fields);
        Ok(value)
    }
    fn count(&mut self, kind: K, width: usize, limit: u64) -> Result<usize, Stop> {
        let count = self.word(width)?;
        if count > limit {
            return Err(Stop::CountLimit { kind, count, limit });
        }
        Ok(count as usize)
    }
    fn body(&mut self, context: &PlayerContext) -> Result<PlayerMetadataBody, Stop> {
        use SourceRegionKind::{Fields, Opaque};
        let mask = self.word(11)? as usize + 1;
        self.skip(mask, Opaque)?;
        let bytes = self.count(K::ByteCount, 12, 2048)?;
        self.skip(bytes * 8, Opaque)?;
        let words = self.count(K::WordCount, 8, 192)?;
        self.skip(words * 32, Opaque)?;
        self.skip(832, Opaque)?;
        let mut gamertag = [0; 16];
        let mut gamertag_len = 0;
        for unit in &mut gamertag {
            *unit = self.word(16)? as u16;
            if *unit == 0 {
                break;
            }
            gamertag_len += 1;
        }
        self.skip(128, Opaque)?;
        // Keep individual read boundaries for partial-source coverage.
        for width in [32, 64, 10, 14, 6, 8, 7, 1] {
            self.skip(width, Fields)?;
        }
        let bytes = context
            .personalization_bytes
            .ok_or(Stop::UnknownPersonalization)?;
        let start = self.cursor.position;
        self.skip(bytes.checked_mul(8).ok_or(Stop::Truncated)?, Opaque)?;
        let personalization = BitRange {
            start,
            end: self.cursor.position,
        };
        // The tail has no decoded fields. Preserve the old byte-by-byte
        // truncation boundary while recording/skipping the whole prefix once.
        let tail_bytes = ((self.cursor.source_bits() - self.cursor.position) / 8).min(44);
        self.skip(tail_bytes * 8, Opaque)?;
        if tail_bytes != 44 {
            return Err(Stop::Truncated);
        }
        Ok(PlayerMetadataBody {
            gamertag,
            gamertag_len,
            personalization,
        })
    }
    fn entry(
        &mut self,
        bots: bool,
        context: &PlayerContext,
    ) -> Result<(PlayerMetadataIdentity, PlayerMetadataBody), Stop> {
        let version = context
            .format_version
            .filter(|v| matches!(v, 20 | 21 | 24 | 25 | 27))
            .ok_or(Stop::UnsupportedFormat)?;
        let identity = if bots {
            PlayerMetadataIdentity::Bot {
                absolute_index: self.word(32)? as u32,
                slot: self.word(32)? as u32,
                id: self.word(32)? as u32,
            }
        } else {
            if version >= 21 && self.word(1)? == 0 {
                self.skip(5, SourceRegionKind::Fields)?;
            }
            PlayerMetadataIdentity::Session(self.word(64)?)
        };
        Ok((identity, self.body(context)?))
    }
}

pub(crate) fn decode(data: &[u8], bots: bool, context: &PlayerContext) -> PlayerMetadataRead {
    let mut r = Reader {
        cursor: Cursor::new(data, 0).unwrap(),
        regions: Some(Vec::new()),
    };
    let count = r.word(32);
    let mut stop = count.as_ref().err().copied().unwrap_or(Stop::Complete);
    let mut entries = Vec::new();
    if let Ok(count) = count {
        if bots && count > 64 {
            stop = Stop::CountLimit {
                kind: K::Count,
                count,
                limit: 64,
            };
        } else {
            // No allocation from an untrusted count. The first refusal ends traversal.
            for _ in 0..count {
                let start_bit = r.cursor.position;
                let values = r.entry(bots, context);
                stop = values.as_ref().err().copied().unwrap_or(Stop::Complete);
                entries.push(PlayerMetadataEntry {
                    start_bit,
                    end_bit: r.cursor.position,
                    values: values.ok(),
                    stop,
                });
                if stop != Stop::Complete {
                    break;
                }
            }
            if bots && stop == Stop::Complete && data.len() * 8 - r.cursor.position >= 8 {
                stop = Stop::TrailingData;
            }
        }
    }
    PlayerMetadataRead {
        build: context.build.clone(),
        entries,
        regions: r.regions.unwrap(),
        end_bit: r.cursor.position,
        stop,
        tail: BitRange {
            start: r.cursor.position,
            end: data.len() * 8,
        },
    }
}

fn player(
    body: &PlayerMetadataBody,
    data: &[u8],
    build: Option<&str>,
) -> Result<PlayerRecord, ParseError> {
    let customization = crate::theater::parser::v41::customization::decode_source(
        data,
        body.personalization,
        build,
    )?;
    Ok(PlayerRecord {
        gamertag: String::from_utf16_lossy(&body.gamertag[..body.gamertag_len]),
        customization,
    })
}

/// Only complete packets become snapshots: a truncated roster must not remove players.
pub(crate) fn roster(
    read: &PlayerMetadataRead,
    data: &[u8],
) -> Result<Option<Vec<RosterPlayer>>, ParseError> {
    if read.stop == Stop::UnknownPersonalization {
        return Err(ParseError::InvalidCustomization);
    }
    if read.stop != Stop::Complete {
        return Ok(None);
    }
    let mut players = Vec::with_capacity(read.entries.len());
    for entry in &read.entries {
        let Some((PlayerMetadataIdentity::Session(identity), body)) = &entry.values else {
            return Ok(None);
        };
        players.push(RosterPlayer {
            xuid: Xuid(identity.swap_bytes()),
            player: player(body, data, read.build.as_deref())?,
        });
    }
    Ok(Some(players))
}

pub(crate) fn bots(
    read: &PlayerMetadataRead,
    data: &[u8],
) -> Result<Option<Vec<BotPlayer>>, ParseError> {
    if read.stop == Stop::UnknownPersonalization {
        return Err(ParseError::InvalidCustomization);
    }
    if read.stop != Stop::Complete {
        return Ok(None);
    }
    let mut players = Vec::with_capacity(read.entries.len());
    for entry in &read.entries {
        let Some((
            PlayerMetadataIdentity::Bot {
                absolute_index,
                slot,
                id,
            },
            body,
        )) = &entry.values
        else {
            return Ok(None);
        };
        players.push(BotPlayer {
            absolute_index: BotAbsoluteIndex(*absolute_index),
            slot: BotSlot(*slot),
            bot_id: BotId(*id),
            player: player(body, data, read.build.as_deref())?,
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
        cursor: Cursor::new(data, start).ok_or(ParseError::IncompleteRegistryPlayers)?,
        regions: None,
    };
    let mut players = Vec::with_capacity(32);
    for slot in 0..32 {
        r.cursor
            .skip(3 + 32 + 2 + 48)
            .ok_or(ParseError::IncompleteRegistryPlayers)?;
        let xuid = Xuid(
            r.cursor
                .read(64)
                .ok_or(ParseError::IncompleteRegistryPlayers)?,
        );
        let body = r.body(context).map_err(|stop| match stop {
            Stop::UnknownPersonalization => ParseError::InvalidCustomization,
            _ => ParseError::IncompleteRegistryPlayers,
        })?;
        let player = player(&body, data, context.build.as_deref())?;
        r.cursor
            .skip(32)
            .ok_or(ParseError::IncompleteRegistryPlayers)?;
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
    fn truncated_rosters_preserve_exact_source_coverage_without_publishing_players() {
        for bot in [false, true] {
            let mut fields = vec![(1, 32)];
            if bot {
                fields.extend([(17, 32), (3, 32), (42, 32)]);
            } else {
                fields.extend([(0, 1), (7, 5), (100, 64)]);
            }
            let mut body = body();
            body.pop();
            body.extend([(0, 8); 44]);
            fields.extend(body);
            let data = pack(&fields);
            for len in 0..data.len() {
                let data = &data[..len];
                let read = decode(data, bot, &context());
                assert_eq!(read.stop, Stop::Truncated);
                let mut expected_end = 0;
                let mut expected_regions: Vec<SourceRegion> = Vec::new();
                for (index, &(_, width)) in fields.iter().enumerate() {
                    if expected_end + width > len * 8 {
                        break;
                    }
                    let kind = if index >= 4
                        && ([1, 3, 5, 6, 9, 18].contains(&(index - 4)) || index - 4 >= 19)
                    {
                        SourceRegionKind::Opaque
                    } else {
                        SourceRegionKind::Fields
                    };
                    if let Some(last) = expected_regions.last_mut()
                        && last.kind == kind
                    {
                        last.source.end += width;
                    } else {
                        expected_regions.push(SourceRegion {
                            source: BitRange {
                                start: expected_end,
                                end: expected_end + width,
                            },
                            kind,
                        });
                    }
                    expected_end += width;
                }
                assert_eq!(read.end_bit, expected_end, "bot={bot}, len={len}");
                assert_eq!(read.regions, expected_regions, "bot={bot}, len={len}");
                let mut end = 0;
                for region in &read.regions {
                    assert_eq!(region.source.start, end);
                    assert!(region.source.end > end);
                    end = region.source.end;
                }
                assert_eq!(end, expected_end);
                let players_absent = if bot {
                    bots(&read, data).unwrap().is_none()
                } else {
                    roster(&read, data).unwrap().is_none()
                };
                assert!(players_absent);
            }
            let read = decode(&data, bot, &context());
            assert_eq!(read.stop, Stop::Complete);
            assert!(read.regions.len() < 16, "coverage should be coalesced");
        }
    }

    #[test]
    fn incomplete_packets_do_not_validate_or_publish_earlier_customization() {
        for bot in [false, true] {
            let mut fields = vec![(2, 32)];
            for body in [invalid_appearance_body(), body()] {
                if bot {
                    fields.extend([(17, 32), (3, 32), (42, 32)]);
                } else {
                    fields.extend([(1, 1), (100, 64)]);
                }
                fields.extend(body);
            }
            let data = pack(&fields);
            let data = &data[..data.len() - 1];
            let read = decode(data, bot, &context());
            assert_eq!(read.stop, Stop::Truncated);
            if bot {
                assert!(bots(&read, data).unwrap().is_none());
            } else {
                assert!(roster(&read, data).unwrap().is_none());
            }
        }
    }

    #[test]
    fn untrusted_counts_stop_before_allocation_or_successor_search() {
        let data = u32::MAX.to_be_bytes();
        let read = decode(&data, true, &context());
        assert!(matches!(read.stop, Stop::CountLimit { kind: K::Count, .. }));
        assert!(read.entries.is_empty());
        let read = decode(&data, false, &context());
        assert_eq!(read.stop, Stop::Truncated);
        assert_eq!(read.entries.len(), 1);
        for (counts, kind) in [
            (vec![(0, 11), (0, 1), (2049, 12)], K::ByteCount),
            (vec![(0, 11), (0, 1), (0, 12), (193, 8)], K::WordCount),
        ] {
            let mut fields = vec![(1, 32), (1, 1), (100, 64)];
            fields.extend(counts);
            let data = pack(&fields);
            let read = decode(&data, false, &context());
            assert!(matches!(read.stop, Stop::CountLimit { kind: found, .. } if found == kind));
            assert_eq!(read.entries.len(), 1);
            assert!(roster(&read, &data).unwrap().is_none());
        }
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
            let data = pack(&fields);
            let read = decode(&data, is_bot, &context());
            assert_eq!(read.stop, Stop::Complete);
            if is_bot {
                assert!(matches!(
                    bots(&read, &data),
                    Err(ParseError::InvalidCustomization)
                ));
            } else {
                assert!(matches!(
                    roster(&read, &data),
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
            let players = roster(&read, &data).unwrap().unwrap();
            assert_eq!(players.len(), 1);
            assert_eq!(players[0].xuid, Xuid(xuid));
            assert_eq!(players[0].player.gamertag, "A");
            let json = serde_json::to_value(&players[0]).unwrap();
            assert_eq!(json.as_object().unwrap().len(), 2);
            assert_eq!(json["player"].as_object().unwrap().len(), 2);
            assert!(json["player"].get("customization").is_some());
            assert!(
                roster(
                    &decode(&data[..data.len() - 1], false, &context),
                    &data[..data.len() - 1]
                )
                .unwrap()
                .is_none()
            );
            context.build = Some("future".into());
            context.personalization_bytes = None;
            assert!(matches!(
                roster(&decode(&data, false, &context), &data),
                Err(ParseError::InvalidCustomization)
            ));
        }
    }

    #[test]
    fn empty_and_truncated_snapshots_are_distinct_and_bots_keep_their_ids() {
        assert_eq!(
            roster(&decode(&[0; 4], false, &context()), &[0; 4]).unwrap(),
            Some(vec![])
        );
        assert_eq!(
            bots(&decode(&[0; 4], true, &context()), &[0; 4]).unwrap(),
            Some(vec![])
        );
        assert!(
            roster(&decode(&[0; 3], false, &context()), &[0; 3])
                .unwrap()
                .is_none()
        );
        assert!(
            roster(&decode(&[0, 0, 0, 1], false, &context()), &[0, 0, 0, 1])
                .unwrap()
                .is_none()
        );
        let mut fields = vec![(1, 32), (17, 32), (3, 32), (42, 32)];
        fields.extend(body());
        let data = pack(&fields);
        let players = bots(&decode(&data, true, &context()), &data)
            .unwrap()
            .unwrap();
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
            bots(
                &decode(&data[..data.len() - 1], true, &context()),
                &data[..data.len() - 1]
            )
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
            let data = pack(&fields);
            let read = decode(&data, false, &context());
            let players = roster(&read, &data).unwrap().unwrap();
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
