//! Named projection over the already bounded Bond tree; all lookup is offline.
//! Field paths: installation-08 d731fb7, film-decoder's `film_infos_de_match.go`,
//! independently projected through this parser's Compact Binary tree.
use super::source::*;
use crate::theater::film::{
    EquipmentTagId, MatchOption, MatchOptionValue, MatchSettings, ParseError, StartingEquipment,
    StartingLoadout, StartingWeapon, TagId, WeaponTagId,
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

fn bytes(v: &BondValue) -> Option<std::borrow::Cow<'_, [u8]>> {
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
        return (raw.bit_len == *count as usize * 8)
            .then(|| std::borrow::Cow::Borrowed(raw.bytes.as_slice()));
    }
    if values.len() != *count as usize {
        return None;
    }
    values
        .iter()
        .map(|v| integer(v).map(|n| n as u8))
        .collect::<Option<Vec<_>>>()
        .map(std::borrow::Cow::Owned)
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

fn required<T>(value: Option<T>, field: &'static str) -> Result<T, ParseError> {
    value.ok_or(ParseError::InvalidMatchSettings { field })
}

/// Absence is allowed, but malformed or ambiguous fields must not become absence.
fn optional_path<'a>(
    mut v: &'a BondValue,
    path: &[u16],
    name: &'static str,
) -> Result<Option<&'a BondValue>, ParseError> {
    for &id in path {
        let next = match &v.body {
            BondValueBody::Struct { fields, .. } => {
                let start = fields
                    .iter()
                    .rposition(|f| f.wire_type == Some(1))
                    .map_or(0, |i| i + 1);
                let mut hits = fields[start..]
                    .iter()
                    .filter(|f| f.id == Some(id) && f.wire_type.is_some_and(|t| t > 1));
                let Some(hit) = hits.next() else {
                    return Ok(None);
                };
                if hits.next().is_some() {
                    return Err(ParseError::InvalidMatchSettings { field: name });
                }
                Some(required(hit.value.as_ref(), name)?)
            }
            BondValueBody::Sequence { .. } => required(sequence(v), name)?.get(id as usize),
            _ => return Err(ParseError::InvalidMatchSettings { field: name }),
        };
        let Some(next) = next else { return Ok(None) };
        v = next;
    }
    Ok(Some(v))
}

fn optional_number(
    v: &BondValue,
    path: &[u16],
    name: &'static str,
) -> Result<Option<i64>, ParseError> {
    optional_path(v, path, name)?
        .map(|v| required(integer(v), name))
        .transpose()
}

fn optional_boolean(
    v: &BondValue,
    path: &[u16],
    name: &'static str,
) -> Result<Option<bool>, ParseError> {
    optional_path(v, path, name)?
        .map(|v| required(boolean(v, &[]), name))
        .transpose()
}

fn optional_tag(
    v: &BondValue,
    path: &[u16],
    name: &'static str,
) -> Result<Option<TagId>, ParseError> {
    optional_path(v, path, name)?
        .map(|v| required(tag(v, &[]), name))
        .transpose()
}

pub(super) fn project(body: &FilmRegistryRead) -> Result<MatchSettings, ParseError> {
    let header = required(body.bootstrap_header.as_ref(), "bootstrap header")?;
    let build = required(
        super::super::context::players::PlayerContext::recorded(body).build,
        "build",
    )?;
    let identity = required(header.identification.as_ref(), "identification")?;
    let start_time_unix_seconds = required(
        identity
            .fields
            .iter()
            .find(|f| f.kind == BootstrapIdentificationFieldKind::StartTime)
            .and_then(|f| f.raw.as_ref())
            .and_then(|r| r.bytes.as_slice().try_into().ok())
            .map(u32::from_le_bytes),
        "recording_start_time",
    )?;
    let options = required(
        identity
            .game_options
            .as_ref()
            .filter(|o| o.stop == GameOptionsStop::Complete),
        "game options",
    )?;
    let mut variants = options.entries.iter().filter_map(|e| match e {
        GameOptionsEntry::Bond {
            kind: GameOptionsBondKind::Variant,
            read,
        } if read.stop == BondStop::Complete => Some(read),
        _ => None,
    });
    let read = required(variants.next(), "variant")?;
    if variants.next().is_some() {
        return Err(ParseError::InvalidMatchSettings { field: "variant" });
    }
    let v = required(variant(&read.root), "variant")?;
    let block = required(path(v, &[3, 0, 0, 0]).and_then(bytes), "variant strings")?;
    let loadouts = required(path(v, &[1, 6, 1]).and_then(sequence), "starting_loadouts")?;
    let options = required(path(v, &[3, 9, 0]).and_then(sequence), "options")?;
    Ok(MatchSettings {
        build,
        recording_start_time: required(
            chrono::DateTime::from_timestamp(i64::from(start_time_unix_seconds), 0),
            "recording_start_time",
        )?,
        name: required(text(&block, number(v, &[3, 0, 1, 0, 0, 1, 0])), "name")?,
        description: required(
            text(&block, number(v, &[3, 0, 1, 0, 1, 1, 0])),
            "description",
        )?,
        engine_type: required(number(v, &[0, 0]), "engine_type")?,
        minimum_respawn_seconds: required(number(v, &[1, 2, 3]), "minimum_respawn_seconds")?,
        score_to_win: optional_number(v, &[1, 7, 0], "score_to_win")?,
        time_limit_seconds: optional_number(v, &[1, 7, 3], "time_limit_seconds")?,
        overtime_seconds: optional_number(v, &[1, 7, 9], "overtime_seconds")?,
        last_chance_enabled: optional_boolean(v, &[1, 7, 10], "last_chance_enabled")?,
        last_chance_seconds: optional_number(v, &[1, 7, 11], "last_chance_seconds")?,
        starting_loadouts: loadouts.iter().map(loadout).collect::<Result<_, _>>()?,
        options: options.iter().map(option).collect::<Result<_, _>>()?,
    })
}

fn loadout(l: &BondValue) -> Result<StartingLoadout, ParseError> {
    Ok(StartingLoadout {
        weapons: required(path(l, &[1]).and_then(sequence), "loadout.weapons")?
            .iter()
            .map(weapon)
            .collect::<Result<_, _>>()?,
        equipment: optional_tag(l, &[2, 0, 0], "loadout.equipment")?
            .map_or(StartingEquipment::None, |tag| {
                StartingEquipment::Real(EquipmentTagId(tag.raw()))
            }),
    })
}

fn weapon(w: &BondValue) -> Result<StartingWeapon, ParseError> {
    let tag = optional_tag(w, &[0, 0], "weapon.tag")?;
    let random = optional_boolean(w, &[2], "weapon.random")?;
    Ok(match (tag, random == Some(true)) {
        (Some(tag), false) => StartingWeapon::Real(WeaponTagId(tag.raw())),
        (None, true) => StartingWeapon::Random,
        (Some(_), true) => {
            return Err(ParseError::InvalidMatchSettings {
                field: "weapon.tag/random conflict",
            });
        }
        (None, false) => StartingWeapon::Unknown,
    })
}

fn option(o: &BondValue) -> Result<MatchOption, ParseError> {
    let raw_value: [u8; 8] = required(
        path(o, &[1, 0])
            .and_then(bytes)
            .and_then(|b| b.as_ref().try_into().ok()),
        "option.raw_value",
    )?;
    let value = match raw_value[4] {
        2 if raw_value[0] <= 1 => MatchOptionValue::Boolean(raw_value[0] != 0),
        2 => {
            return Err(ParseError::InvalidMatchSettings {
                field: "option.boolean",
            });
        }
        3 => MatchOptionValue::NumberBits(u32::from_le_bytes(raw_value[..4].try_into().unwrap())),
        _ => MatchOptionValue::Unknown,
    };
    Ok(MatchOption {
        name_hash: required(tag(o, &[0, 0]), "option.name_hash")?.0,
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::parser::v41::packet::BitRange;
    use crate::theater::parser::v41::registry::bootstrap::bond::read_bounded;

    #[test]
    fn required_settings_and_recording_datetime_decode_without_defaults() {
        use std::io::Read;
        let compressed = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/theater/bazaar_idle/chunk-000-type-1.bin"
        ));
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(compressed.as_slice())
            .read_to_end(&mut data)
            .unwrap();
        for field in ["recording_start_time", "variant"] {
            let mut body = super::super::parse_registry_chunk(&data).unwrap();
            let identity = body
                .bootstrap_header
                .as_mut()
                .unwrap()
                .identification
                .as_mut()
                .unwrap();
            if field == "recording_start_time" {
                identity
                    .fields
                    .retain(|f| f.kind != BootstrapIdentificationFieldKind::StartTime);
            } else {
                identity.game_options.as_mut().unwrap().entries.retain(|e| {
                    !matches!(
                        e,
                        GameOptionsEntry::Bond {
                            kind: GameOptionsBondKind::Variant,
                            ..
                        }
                    )
                });
            }
            assert!(
                matches!(project(&body), Err(ParseError::InvalidMatchSettings { field: failed }) if failed == field)
            );
        }
        for (seconds, expected) in [
            (0_u32, "1970-01-01T00:00:00Z"),
            (1_700_000_000, "2023-11-14T22:13:20Z"),
            (u32::MAX, "2106-02-07T06:28:15Z"),
        ] {
            let mut body = super::super::parse_registry_chunk(&data).unwrap();
            let raw = body
                .bootstrap_header
                .as_mut()
                .unwrap()
                .identification
                .as_mut()
                .unwrap()
                .fields
                .iter_mut()
                .find(|field| field.kind == BootstrapIdentificationFieldKind::StartTime)
                .unwrap()
                .raw
                .as_mut()
                .unwrap();
            raw.bytes.copy_from_slice(&seconds.to_le_bytes());
            let settings = project(&body).unwrap();
            assert_eq!(
                settings.recording_start_time.timestamp(),
                i64::from(seconds)
            );
            assert_eq!(settings.recording_start_time.timestamp_subsec_nanos(), 0);
            let json = serde_json::to_value(&settings).unwrap();
            assert_eq!(json["recording_start_time"], expected);
            assert!(json.get("start_time_unix_seconds").is_none());
            assert_eq!(
                serde_json::from_value::<MatchSettings>(json).unwrap(),
                settings
            );
        }
    }

    #[test]
    #[ignore = "set HALO_FILM_CORPUS to a directory of local film.json fixtures"]
    fn local_corpus_settings_audit() {
        use std::{collections::BTreeMap, io::Read, path::Path};
        fn visit(path: &Path, manifests: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    visit(&entry.path(), manifests);
                } else if entry.file_name() == "film.json" || entry.file_name() == "manifest.json" {
                    manifests.push(entry.path());
                }
            }
        }
        fn count(
            value: &serde_json::Value,
            path: &str,
            counts: &mut BTreeMap<String, (usize, usize)>,
        ) {
            let entry = counts.entry(path.to_owned()).or_default();
            entry.0 += 1;
            entry.1 += usize::from(value.is_null());
            match value {
                serde_json::Value::Object(fields) => {
                    for (name, value) in fields {
                        count(value, &format!("{path}.{name}"), counts);
                    }
                }
                serde_json::Value::Array(values) => {
                    for value in values {
                        count(value, &format!("{path}[]"), counts);
                    }
                }
                _ => {}
            }
        }
        let root = std::env::var_os("HALO_FILM_CORPUS").expect("HALO_FILM_CORPUS");
        let mut manifests = Vec::new();
        visit(Path::new(&root), &mut manifests);
        visit(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/theater"),
            &mut manifests,
        );
        manifests.sort();
        assert!(!manifests.is_empty());
        let mut counts = BTreeMap::new();
        let mut profiles = BTreeMap::<String, usize>::new();
        let mut weapons = BTreeMap::<String, usize>::new();
        let mut equipment = BTreeMap::<String, usize>::new();
        for manifest in &manifests {
            let json: serde_json::Value =
                serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
            let chunk = json["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["chunk_type"] == 1 || c["kind"] == 1)
                .unwrap();
            let mut data = std::fs::read(
                manifest
                    .parent()
                    .unwrap()
                    .join(chunk["file"].as_str().unwrap()),
            )
            .unwrap();
            if data
                .get(..2)
                .is_some_and(|h| h[0] & 15 == 8 && u16::from_be_bytes([h[0], h[1]]) % 31 == 0)
            {
                let mut inflated = Vec::new();
                flate2::read::ZlibDecoder::new(data.as_slice())
                    .read_to_end(&mut inflated)
                    .unwrap();
                data = inflated;
            }
            let parsed = super::super::V41RegistryChunkReader::read(
                crate::theater::film::InflatedFilmChunk {
                    kind: crate::theater::film::ChunkKind::Registry,
                    index: 0,
                    start_ms: 0,
                    data,
                },
            )
            .unwrap_or_else(|e| panic!("{}: {e}", manifest.display()));
            *profiles
                .entry(format!(
                    "v{}/{}/{}",
                    parsed.registry.version.film,
                    parsed.registry.version.format,
                    parsed.registry.settings.build
                ))
                .or_default() += 1;
            assert_eq!(parsed.registry.players.len(), 32);
            let settings = serde_json::to_value(&parsed.registry.settings).unwrap();
            count(&settings, "settings", &mut counts);
            if let Some(loadouts) = settings["starting_loadouts"].as_array() {
                for l in loadouts {
                    *equipment.entry(l["equipment"].to_string()).or_default() += 1;
                    if let Some(entries) = l["weapons"].as_array() {
                        for w in entries {
                            *weapons.entry(w.to_string()).or_default() += 1;
                        }
                    }
                }
            }
        }
        eprintln!("{} registry films", manifests.len());
        eprintln!("profiles: {profiles:?}");
        for (path, (total, missing)) in counts {
            eprintln!("{path}: {missing}/{total} absent");
        }
        eprintln!("starting weapons: {weapons:?}");
        eprintln!("starting equipment: {equipment:?}");
    }

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
            ([0, 0, 0, 0, 99, 0, 0, 0], Some(MatchOptionValue::Unknown)),
        ] {
            // Two nested structs: name/hash and raw value as a packed byte array.
            let mut wire = vec![20, 10, 3, 3, 42, 0, 42, 12, 11, 3, 8];
            wire.extend(payload);
            wire.extend([0, 0]);
            let decoded = option(&read(&wire));
            let Some(expected) = expected else {
                assert!(matches!(
                    decoded,
                    Err(ParseError::InvalidMatchSettings {
                        field: "option.boolean"
                    })
                ));
                continue;
            };
            let option = decoded.unwrap();
            assert_eq!(option.name_hash, 42);
            assert_eq!(option.value, expected);
            assert!(serde_json::to_string(&option).is_ok());
            assert!(
                serde_json::to_value(&option)
                    .unwrap()
                    .get("raw_value")
                    .is_none()
            );
        }
    }

    #[test]
    fn starting_equipment_preserves_tags_and_rejects_malformed_records() {
        for tag in [0, 42, 255] {
            let loadout = loadout(&read(&[11, 43, 42, 74, 6, 10, 3, 3, tag, 0, 0, 0])).unwrap();
            assert_eq!(
                loadout.equipment,
                StartingEquipment::Real(EquipmentTagId(u32::from(tag)))
            );
            let json = serde_json::to_value(&loadout.equipment).unwrap();
            assert_eq!(json, serde_json::json!({"Real": tag}));
            assert_eq!(
                serde_json::from_value::<StartingEquipment>(json).unwrap(),
                loadout.equipment
            );
        }
        for wire in [&[3, 43, 42, 0][..], &[6, 43, 42, 74, 1, 0, 0][..]] {
            assert_eq!(
                loadout(&read(wire)).unwrap().equipment,
                StartingEquipment::None
            );
        }
        assert_eq!(
            serde_json::to_value(StartingEquipment::None).unwrap(),
            serde_json::json!("None")
        );
        assert_eq!(
            serde_json::from_str::<StartingEquipment>("\"None\"").unwrap(),
            StartingEquipment::None
        );
        for json in ["null", "\"Unknown\"", "42", "{\"Real\":null}"] {
            assert!(serde_json::from_str::<StartingEquipment>(json).is_err());
        }
        for wire in [
            &[5, 43, 42, 67, 42, 0][..],
            &[
                19, 43, 42, 74, 6, 10, 3, 3, 42, 0, 0, 74, 6, 10, 3, 3, 42, 0, 0, 0,
            ][..],
        ] {
            assert!(matches!(
                loadout(&read(wire)),
                Err(ParseError::InvalidMatchSettings {
                    field: "loadout.equipment"
                })
            ));
        }
    }

    #[test]
    fn loadouts_and_options_require_complete_fields_without_inventing_defaults() {
        let empty = read(&[1, 0]);
        assert!(matches!(
            loadout(&empty),
            Err(ParseError::InvalidMatchSettings {
                field: "loadout.weapons"
            })
        ));
        assert!(option(&empty).is_err());
        assert_eq!(weapon(&empty).unwrap(), StartingWeapon::Unknown);
        // An explicitly empty weapon list is valid, unlike a missing list.
        let empty_loadout = read(&[3, 43, 42, 0]);
        assert!(loadout(&empty_loadout).unwrap().weapons.is_empty());
        assert_eq!(
            loadout(&empty_loadout).unwrap().equipment,
            StartingEquipment::None
        );
        assert_eq!(
            weapon(&read(&[3, 66, 1, 0])).unwrap(),
            StartingWeapon::Random
        );
        assert_eq!(
            weapon(&read(&[3, 66, 0, 0])).unwrap(),
            StartingWeapon::Unknown
        );
        assert_eq!(
            weapon(&read(&[6, 10, 3, 3, 42, 0, 0])).unwrap(),
            StartingWeapon::Real(WeaponTagId(42))
        );
        assert!(matches!(
            weapon(&read(&[8, 10, 3, 3, 42, 0, 66, 1, 0])),
            Err(ParseError::InvalidMatchSettings {
                field: "weapon.tag/random conflict"
            })
        ));
        // Malformed and duplicated tags are errors, not an unknown weapon.
        assert!(weapon(&read(&[3, 3, 42, 0])).is_err());
        assert!(weapon(&read(&[5, 66, 1, 66, 0, 0])).is_err());
        // Required option name and fixed-size payload cannot disappear silently.
        let payload = [0, 0, 0, 0, 3, 0, 0, 0];
        let mut wire = vec![15, 42, 12, 11, 3, 8];
        wire.extend(payload);
        wire.extend([0, 0]);
        assert!(matches!(
            option(&read(&wire)),
            Err(ParseError::InvalidMatchSettings {
                field: "option.name_hash"
            })
        ));
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
