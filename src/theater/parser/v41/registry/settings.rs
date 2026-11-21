//! Named projection over the already bounded Bond tree; all lookup is offline.
//! Field paths: installation-08 d731fb7, film-decoder's `film_infos_de_match.go`,
//! independently projected through this parser's Compact Binary tree.
use super::source::*;
use crate::theater::film::{
    MatchOption, MatchOptionValue, MatchSettings, StartingLoadout, StartingWeapon, TagId,
};

fn field(v: &BondValue, id: u16) -> Option<&BondValue> {
    let BondValueBody::Struct { fields, .. } = &v.body else {
        return None;
    };
    let start = fields
        .iter()
        .rposition(|f| f.wire_type == Some(1))
        .map_or(0, |i| i + 1);
    let mut hits = fields[start..]
        .iter()
        .filter(|f| f.id == Some(id) && f.wire_type.is_some_and(|t| t > 1));
    let value = hits.next()?.value.as_ref()?;
    if hits.next().is_some() {
        return None;
    }
    Some(value)
}

fn sequence(v: &BondValue) -> Option<&[BondValue]> {
    let BondValueBody::Sequence {
        count: Some(count),
        values,
        packed: None,
        ..
    } = &v.body
    else {
        return None;
    };
    (*count as usize == values.len()).then_some(values.as_slice())
}

fn path<'a>(mut v: &'a BondValue, p: &[u16]) -> Option<&'a BondValue> {
    for &i in p {
        v = match &v.body {
            BondValueBody::Struct { .. } => field(v, i)?,
            BondValueBody::Sequence { .. } => sequence(v)?.get(i as usize)?,
            _ => return None,
        };
    }
    Some(v)
}

fn integer(v: &BondValue) -> Option<i64> {
    match &v.body {
        BondValueBody::Integer(n) => {
            let n = n.value?;
            match v.wire_type {
                15..=17 => Some((n >> 1) as i64 ^ -((n & 1) as i64)),
                4..=6 => i64::try_from(n).ok(),
                _ => None,
            }
        }
        BondValueBody::Fixed(b) if matches!(v.wire_type, 2 | 3 | 14) => {
            let n = b.raw.as_ref()?.low_u64();
            Some(if v.wire_type == 14 {
                i64::from(n as u8 as i8)
            } else {
                n as i64
            })
        }
        _ => None,
    }
}

fn number(v: &BondValue, p: &[u16]) -> Option<i64> {
    integer(path(v, p)?)
}
fn boolean(v: &BondValue, p: &[u16]) -> Option<bool> {
    let v = path(v, p)?;
    (v.wire_type == 2)
        .then(|| integer(v).map(|n| n != 0))
        .flatten()
}
fn tag(v: &BondValue, p: &[u16]) -> Option<TagId> {
    let n = number(v, p)?;
    (n >= i32::MIN as i64 && n <= u32::MAX as i64).then_some(TagId(n as u32))
}

fn bytes(v: &BondValue) -> Option<Vec<u8>> {
    let BondValueBody::Sequence {
        header,
        count: Some(count),
        packed,
        values,
        ..
    } = &v.body
    else {
        return None;
    };
    if !matches!(header.raw.as_ref()?.low_u64() & 31, 3 | 14) {
        return None;
    }
    if let Some(packed) = packed {
        let raw = packed.raw.as_ref()?;
        return (raw.bit_len == *count as usize * 8).then(|| raw.bytes.to_vec());
    }
    if values.len() != *count as usize {
        return None;
    }
    values.iter().map(|v| integer(v).map(|n| n as u8)).collect()
}

fn text(block: &[u8], offset: Option<i64>) -> Option<String> {
    let tail = block.get(usize::try_from(offset?).ok()?..)?;
    let end = tail.iter().position(|b| *b == 0)?;
    std::str::from_utf8(&tail[..end]).ok().map(str::to_owned)
}

fn variant(root: &BondValue) -> Option<&BondValue> {
    let mut selected = None;
    for id in 0..4 {
        let Some(list) = field(root, id) else {
            continue;
        };
        let entries = sequence(list)?;
        match entries {
            [] => {}
            [v] if selected.is_none() && v.wire_type == 10 => selected = Some(v),
            _ => return None,
        }
    }
    selected
}

pub(super) fn project(body: &FilmRegistryRead) -> MatchSettings {
    let mut out = MatchSettings::default();
    let Some(header) = &body.bootstrap_header else {
        return out;
    };
    out.build = super::super::context::players::PlayerContext::recorded(body).build;
    let Some(identity) = &header.identification else {
        return out;
    };
    out.start_time_unix_seconds = identity
        .fields
        .iter()
        .find(|f| f.kind == BootstrapIdentificationFieldKind::StartTime)
        .and_then(|f| f.raw.as_ref())
        .and_then(|r| r.bytes.as_slice().try_into().ok())
        .map(u32::from_le_bytes);
    let Some(options) = &identity.game_options else {
        return out;
    };
    if options.stop != GameOptionsStop::Complete {
        return out;
    }
    let mut variants = options.entries.iter().filter_map(|e| match e {
        GameOptionsEntry::Bond {
            kind: GameOptionsBondKind::Variant,
            read,
        } if read.stop == BondStop::Complete => Some(read),
        _ => None,
    });
    let Some(read) = variants.next() else {
        return out;
    };
    if variants.next().is_some() {
        return out;
    }
    let Some(v) = variant(&read.root) else {
        return out;
    };
    out.variant_decoded = true;
    if let Some(block) = path(v, &[3, 0, 0, 0]).and_then(bytes) {
        out.name = text(&block, number(v, &[3, 0, 1, 0, 0, 1, 0]));
        out.description = text(&block, number(v, &[3, 0, 1, 0, 1, 1, 0]));
    }
    out.engine_type = number(v, &[0, 0]);
    out.killcam_enabled = boolean(v, &[1, 0, 5, 0]);
    out.play_of_game_enabled = boolean(v, &[1, 0, 5, 2]);
    out.minimum_respawn_seconds = number(v, &[1, 2, 3]);
    out.score_to_win = number(v, &[1, 7, 0]);
    out.time_limit_seconds = number(v, &[1, 7, 3]);
    out.overtime_seconds = number(v, &[1, 7, 9]);
    out.last_chance_enabled = boolean(v, &[1, 7, 10]);
    out.last_chance_seconds = number(v, &[1, 7, 11]);
    out.starting_loadouts = path(v, &[1, 6, 1]).and_then(sequence).map(|loadouts| {
        loadouts
            .iter()
            .map(|l| StartingLoadout {
                weapons: path(l, &[1]).and_then(sequence).map(|weapons| {
                    weapons
                        .iter()
                        .map(|w| StartingWeapon {
                            tag: tag(w, &[0, 0]),
                            random: boolean(w, &[2]),
                        })
                        .collect()
                }),
                equipment: tag(l, &[2, 0, 0]),
            })
            .collect()
    });
    out.options = path(v, &[3, 9, 0])
        .and_then(sequence)
        .map(|options| options.iter().map(option).collect());
    out
}

fn option(o: &BondValue) -> MatchOption {
    let raw_value = path(o, &[1, 0]).and_then(bytes);
    let value = raw_value.as_deref().and_then(|b| {
        if b.len() != 8 {
            return None;
        }
        match b[4] {
            2 if b[0] <= 1 => Some(MatchOptionValue::Boolean(b[0] != 0)),
            3 => Some(MatchOptionValue::NumberBits(u32::from_le_bytes(
                b[..4].try_into().unwrap(),
            ))),
            _ => None,
        }
    });
    MatchOption {
        name_hash: tag(o, &[0, 0]).map(|t| t.0),
        raw_value,
        value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::parser::v41::packet::BitRange;
    use crate::theater::parser::v41::registry::bootstrap::bond::read_bounded;

    fn read(data: &[u8]) -> BondValue {
        let read = read_bounded(
            data,
            BitRange {
                start: 0,
                end: data.len() * 8,
            },
        );
        assert_eq!(read.stop, BondStop::Complete);
        read.root
    }

    #[test]
    fn absent_fields_are_not_defaults_and_inherited_ids_are_not_derived_fields() {
        // Base field 0=9, base stop, derived field 0=0, derived field 1=false.
        let root = read(&[8, 3, 9, 1, 3, 0, 34, 0, 0]);
        assert_eq!(number(&root, &[0]), Some(0));
        assert_eq!(boolean(&root, &[1]), Some(false));
        assert_eq!(number(&root, &[2]), None);
        let root = read(&[4, 3, 9, 1, 0]);
        assert_eq!(number(&root, &[0]), None);
        let duplicate = read(&[5, 3, 9, 3, 8, 0]);
        assert_eq!(number(&duplicate, &[0]), None);
    }

    #[test]
    fn options_preserve_unknown_types_and_nonfinite_number_bits() {
        for (payload, expected) in [
            (
                [1, 0, 0, 0, 2, 0, 0, 0],
                Some(MatchOptionValue::Boolean(true)),
            ),
            (
                [0, 0, 0, 0, 2, 0, 0, 0],
                Some(MatchOptionValue::Boolean(false)),
            ),
            ([2, 0, 0, 0, 2, 0, 0, 0], None),
            (
                [1, 0, 192, 127, 3, 0, 0, 0],
                Some(MatchOptionValue::NumberBits(0x7fc00001)),
            ),
            ([0, 0, 0, 0, 99, 0, 0, 0], None),
        ] {
            // Two nested structs: name/hash and raw value as a packed byte array.
            let mut wire = vec![20, 10, 3, 3, 42, 0, 42, 12, 11, 3, 8];
            wire.extend(payload);
            wire.extend([0, 0]);
            let option = option(&read(&wire));
            assert_eq!(option.name_hash, Some(42));
            assert_eq!(option.raw_value.as_deref(), Some(payload.as_slice()));
            assert_eq!(option.value, expected);
            assert!(serde_json::to_string(&option).is_ok());
        }
    }

    #[test]
    fn variant_requires_one_entry_and_text_requires_valid_offsets_and_utf8() {
        let empty = read(&[3, 11, 42, 0]); // empty list of structs
        assert!(variant(&empty).is_none());
        let single = read(&[5, 11, 74, 1, 0, 0]);
        assert!(variant(&single).is_some());
        let multiple = read(&[7, 11, 106, 1, 0, 1, 0, 0]);
        assert!(variant(&multiple).is_none());
        assert_eq!(text(b"Slayer\0desc\0", Some(7)), Some("desc".into()));
        assert_eq!(text(b"\0", Some(0)), Some(String::new()));
        assert_eq!(text(b"abc", Some(0)), None);
        assert_eq!(text(b"\xff\0", Some(0)), None);
        assert_eq!(text(b"abc\0", Some(-1)), None);
        assert_eq!(text(b"abc\0", Some(99)), None);
    }
}
