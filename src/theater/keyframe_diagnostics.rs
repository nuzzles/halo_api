//! Native keyframe research measurements. Candidate layout scores and recovered
//! anchors are diagnostics, not proof of sequential decoding or native structure.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyframeQueueVariant {
    #[serde(rename = "IDLowBits")]
    pub id_low_bits: i64,
    pub preamble: i64,
    pub extra_fields: bool,
}
/// Native enumeration order: extra-fields flag, preamble 0..2, then ID width.
/// A reversed inclusive width interval yields no candidates.
pub fn keyframe_queue_variants(from: i64, to: i64) -> Vec<KeyframeQueueVariant> {
    let mut out = Vec::new();
    for extra_fields in [false, true] {
        for preamble in 0..=2 {
            for id_low_bits in from..=to {
                out.push(KeyframeQueueVariant {
                    id_low_bits,
                    preamble,
                    extra_fields,
                });
            }
        }
    }
    out
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyframeQueueWalk {
    pub variant: KeyframeQueueVariant,
    pub records: usize,
    pub new: usize,
    pub del: usize,
    pub delta: usize,
    pub clean_new: usize,
    pub end_bit: i64,
    pub total_bits: i64,
    pub overrun: bool,
    pub stop: String,
    #[serde(rename = "ByTI")]
    pub by_ti: BTreeMap<u32, usize>,
}
impl KeyframeQueueWalk {
    /// Native coverage formula: zero for empty payloads or flagged overruns.
    /// Otherwise the ratio is returned without clamping or reinterpreting flags.
    pub fn coverage(&self) -> f64 {
        if self.total_bits == 0 || self.overrun {
            0.
        } else {
            self.end_bit as f64 / self.total_bits as f64
        }
    }
    pub fn sorted_type_indices(&self) -> Vec<u32> {
        self.by_ti.keys().copied().collect()
    }
}
/// Native BestVariant selection rule, applied to already measured walks. Chooses
/// strictly greater EndBit without overrun; retains the first tie. No positive
/// candidate yields the native zero-value result. Desync is not an extra filter.
/// This only ranks supplied measurements; it does not run the record walker.
pub fn best_keyframe_queue_measurement(walks: &[KeyframeQueueWalk]) -> KeyframeQueueWalk {
    let mut best = KeyframeQueueWalk::default();
    for w in walks {
        if !w.overrun && w.end_bit > best.end_bit {
            best = w.clone();
        }
    }
    best
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyframeAnchorShape {
    pub records: usize,
    pub gap_mod8: BTreeMap<usize, usize>,
    pub gap_values: BTreeMap<usize, usize>,
    pub bit_aligned: usize,
}
/// Measure native recovered anchors, reusing the existing scanner. Spacing and
/// alignment do not prove intervening records or payload boundaries.
pub fn measure_keyframe_anchors(payload: &[u8]) -> KeyframeAnchorShape {
    let mut records = super::recover_keyframe_anchors(payload);
    super::native_sort::sort_by(&mut records, |a, b| a.bit.cmp(&b.bit));
    let mut out = KeyframeAnchorShape {
        records: records.len(),
        ..Default::default()
    };
    out.bit_aligned = records.iter().filter(|r| r.bit % 8 == 0).count();
    for pair in records.windows(2) {
        let gap = pair[1].bit - pair[0].bit;
        *out.gap_mod8.entry(gap % 8).or_default() += 1;
        *out.gap_values.entry(gap).or_default() += 1;
    }
    out
}

/// Walk the native generic record loop with an explicitly supplied reader profile.
/// The world is mutated only by clean NEWs and DELs. Unlike production view
/// admission, this loop reads a DELTA baseline before rejecting an unbound slot
/// when strict generation checking is disabled. Native zero-tail reads are kept.
pub fn walk_keyframe_queue_records(
    registry: &super::FilmRegistry,
    payload: &[u8],
    variant: KeyframeQueueVariant,
    encoding: &super::FrameEncoding,
    world: &mut super::FilmWorld,
    generation_strict: bool,
) -> Result<KeyframeQueueWalk, super::DecodeError> {
    use super::{EntityBindings, EntityViewStop, NativeFilmBits, RecordKind};
    let mut encoding = encoding.clone();
    encoding.native_id_low_bits = Some(variant.id_low_bits);
    encoding.extra_fields = variant.extra_fields;
    if !encoding.valid() {
        return Err(super::DecodeError::Missing("valid frame encoding"));
    }
    let mut bit = variant.preamble.max(0);
    let mut out = KeyframeQueueWalk {
        variant,
        total_bits: payload.len().saturating_mul(8) as i64,
        stop: "fin propre".into(),
        ..Default::default()
    };
    loop {
        let header_at = bit.wrapping_add(if encoding.extra_fields { 32 } else { 0 });
        let header = super::records::decode_frame_header_signed(payload, header_at, &encoding)
            .ok_or(super::DecodeError::Missing("record header"))?;
        if header.kind == RecordKind::End {
            bit = header.end_bit;
            break;
        }
        let id = header.id.expect("non-end record ID");
        let slot = id & 0x3fff_ffff;
        out.records += 1;
        let mut ti = 0;
        let desync;
        if header.kind == RecordKind::Delta && !world.generation_matches(id, generation_strict) {
            bit = header.end_bit;
            desync = Some(0);
        } else if header.kind == RecordKind::Delta && world.archetype(slot).is_none() {
            let mut cursor = NativeFilmBits::new(payload);
            cursor.set_position(header.end_bit);
            if cursor.read_bit() {
                cursor.skip(7);
            }
            bit = cursor.position();
            desync = Some(0);
        } else {
            let mut bindings = EntityBindings::default();
            if let Some(archetype) = world.archetype(slot) {
                bindings.bind(id, archetype);
            }
            let record = super::components::decode_entity_record_with_movement(
                payload,
                bit,
                registry,
                &encoding,
                &bindings,
                encoding.keyframe_simulation_complete.unwrap_or(true),
                Some(slot),
            )
            .ok_or(super::DecodeError::Missing("native record body"))?;
            bit = record.end_bit;
            ti = record.archetype.unwrap_or(0);
            desync = match record.stop {
                EntityViewStop::Complete => None,
                EntityViewStop::InvalidComponent { index }
                | EntityViewStop::UnsupportedComponent { index, .. } => Some(index),
                _ => Some(0),
            };
            if desync.is_none() {
                match header.kind {
                    RecordKind::New => world.bind_full(id, ti),
                    RecordKind::Delete => world.unbind(slot),
                    _ => {}
                }
            }
        }
        match header.kind {
            RecordKind::New => {
                out.new += 1;
                if desync.is_none() {
                    out.clean_new += 1;
                    *out.by_ti.entry(ti).or_default() += 1;
                }
            }
            RecordKind::Delete => out.del += 1,
            RecordKind::Delta => out.delta += 1,
            RecordKind::End => unreachable!(),
        }
        if let Some(index) = desync {
            out.stop =
                format!("desync record slot={slot} typeIdx={ti} at component i{index} (bit {bit})");
            break;
        }
    }
    out.end_bit = bit;
    out.overrun = out.end_bit > out.total_bits;
    if out.overrun {
        out.stop = format!("depassement du tampon : {}", out.stop);
    }
    Ok(out)
}

/// Traverse every candidate against a fresh world from `seed`, returning all
/// measurements and the native winning measurement. Seed state never leaks from
/// one candidate into another; selection preserves the native overrun/tie rules.
pub fn best_keyframe_queue_variant(
    registry: &super::FilmRegistry,
    payload: &[u8],
    variants: &[KeyframeQueueVariant],
    encoding: &super::FrameEncoding,
    generation_strict: bool,
    mut seed: impl FnMut() -> super::FilmWorld,
) -> Result<(KeyframeQueueWalk, Vec<KeyframeQueueWalk>), super::DecodeError> {
    let mut all = Vec::with_capacity(variants.len());
    for variant in variants {
        all.push(walk_keyframe_queue_records(
            registry,
            payload,
            variant.clone(),
            encoding,
            &mut seed(),
            generation_strict,
        )?);
    }
    Ok((best_keyframe_queue_measurement(&all), all))
}

/// The pinned parser's uncalibrated KFQ research profile. These are its reader
/// defaults, not a claim that every v41 recording uses this world precision.
pub fn native_keyframe_queue_encoding() -> super::FrameEncoding {
    super::FrameEncoding {
        keyframe_layout: Default::default(),
        keyframe_simulation_complete: Some(false),
        native_id_low_bits: None,
        component_widths: Default::default(),
        new_record: Default::default(),
        position_capture: None,
        ids: super::RecordIdLayout {
            low_bits: 13,
            base: 0,
        },
        mpp_widths: [9, 5],
        position: Some(super::PositionEncoding {
            bodies: Default::default(),
            index_bits: 1,
            world_axis_bits: Some([13, 13, 14]),
            handle_bits: 1,
            traversal_axis_bits: [6; 3],
            default_axis_bits: [22; 3],
            region_axis_bits: BTreeMap::new(),
            delta_axis_bits: [14; 3],
            calibrated_skip: false,
            baseline_scope: false,
            full_precision: false,
            writer_absolute: false,
            delta_handle_tail: false,
        }),
        extra_fields: false,
        corruption_check: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    #[ignore = "requires four local v41 captured films"]
    fn local_keyframe_queue_corpus() {
        use crate::theater::*;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kfq-corpus-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let rows = rows.as_array().unwrap();
        assert_eq!(rows.len(), 64);
        let mut total = 0;
        for (i, row) in rows.iter().enumerate() {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(row["folder"].as_str().unwrap());
            let meta: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join("film.json")).unwrap()).unwrap();
            assert_eq!(meta["film_major_version"], 41);
            let header = meta["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["index"] == 0)
                .unwrap();
            let header = std::fs::read(root.join(header["file"].as_str().unwrap())).unwrap();
            let registry = parse_registry(&inflate_film_chunk(&header)).unwrap();
            let bytes = std::fs::read(root.join(row["file"].as_str().unwrap())).unwrap();
            let data = inflate_film_chunk(&bytes);
            let packets = walk_film_packets(&data);
            let packet = &packets[row["packet"].as_u64().unwrap() as usize];
            assert_eq!(
                packet.packet_type,
                u16::try_from(row["kind"].as_u64().unwrap()).unwrap()
            );
            let mut anchors = Vec::new();
            if row["seeded"].as_bool().unwrap() && !row["seed_file"].as_str().unwrap().is_empty() {
                let raw = std::fs::read(root.join(row["seed_file"].as_str().unwrap())).unwrap();
                let seed_data = inflate_film_chunk(&raw);
                let seed_packets = walk_film_packets(&seed_data);
                let seed_packet = &seed_packets[row["seed_packet"].as_u64().unwrap() as usize];
                anchors = recover_keyframe_anchors(
                    &seed_data[seed_packet.payload_offset
                        ..seed_packet.payload_offset + seed_packet.payload_size],
                );
            }
            let seed = || {
                let mut w = FilmWorld::default();
                for a in &anchors {
                    w.bind_full(a.id, a.archetype);
                }
                w
            };
            let variants: Vec<KeyframeQueueVariant> =
                serde_json::from_value(row["variants"].clone()).unwrap();
            let (best, all) = best_keyframe_queue_variant(
                &registry,
                &data[packet.payload_offset..packet.payload_offset + packet.payload_size],
                &variants,
                &native_keyframe_queue_encoding(),
                false,
                seed,
            )
            .unwrap();
            let expected: Vec<KeyframeQueueWalk> =
                serde_json::from_value(row["all"].clone()).unwrap();
            assert_eq!(all, expected, "captured {i} {}", row["folder"]);
            let mut expected = row["best"].clone();
            if expected["ByTI"].is_null() {
                expected["ByTI"] = serde_json::json!({});
            }
            assert_eq!(best, serde_json::from_value(expected).unwrap(), "best {i}");
            total += all.len();
        }
        assert_eq!(total, 1536);
    }

    #[test]
    fn native_keyframe_queue_components() {
        use crate::theater::*;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/kfq-components-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let rows = rows.as_array().unwrap();
        assert_eq!(rows.len(), 3201);
        let encoding = native_keyframe_queue_encoding();
        let mut mismatches = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            let ti = row["ti"].as_u64().unwrap() as usize;
            let mut registry = FilmRegistry {
                archetypes: (0..50)
                    .map(|index| FilmArchetype {
                        index,
                        components: vec![],
                        levels: vec![],
                    })
                    .collect(),
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            registry.archetypes[ti].components = vec![row["name"].as_str().unwrap().to_owned()];
            registry.archetypes[ti].levels = vec![row["level"].as_u64().unwrap() as u32];
            let h = row["hex"].as_str().unwrap();
            let pay: Vec<_> = (0..h.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&h[j..j + 2], 16).unwrap())
                .collect();
            let expected: KeyframeQueueWalk = serde_json::from_value(row["walk"].clone()).unwrap();
            let mut world = FilmWorld::default();
            world.bind_full(0x40000001, ti as u32);
            let actual = walk_keyframe_queue_records(
                &registry,
                &pay,
                expected.variant.clone(),
                &encoding,
                &mut world,
                false,
            )
            .unwrap();
            if actual != expected {
                mismatches.push(format!(
                    "case {i} ti={ti} {}: got {actual:?}, expected {expected:?}",
                    row["name"]
                ));
            }
        }
        assert!(
            mismatches.is_empty(),
            "{} mismatches: {}",
            mismatches.len(),
            mismatches
                .iter()
                .take(8)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn native_keyframe_queue_traversal() {
        use crate::theater::*;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kfq-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let oracle: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: Some(oracle["simulation"].as_bool().unwrap()),
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            ids: RecordIdLayout {
                low_bits: 13,
                base: 0,
            },
            mpp_widths: serde_json::from_value(oracle["mpp"].clone()).unwrap(),
            position: Some(serde_json::from_value(oracle["position"].clone()).unwrap()),
            extra_fields: false,
            corruption_check: oracle["corruption"].as_bool().unwrap(),
        };
        assert_eq!(encoding, native_keyframe_queue_encoding());
        let registry = FilmRegistry {
            archetypes: (0..4)
                .map(|index| FilmArchetype {
                    index,
                    components: vec![],
                    levels: vec![],
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let mut count = 0;
        for (i, row) in oracle["walks"].as_array().unwrap().iter().enumerate() {
            let h = row["hex"].as_str().unwrap();
            let pay: Vec<_> = (0..h.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&h[j..j + 2], 16).unwrap())
                .collect();
            for measurement in row["measurements"].as_array().unwrap() {
                let expected: KeyframeQueueWalk =
                    serde_json::from_value(measurement["walk"].clone()).unwrap();
                let actual = walk_keyframe_queue_records(
                    &registry,
                    &pay,
                    expected.variant.clone(),
                    &encoding,
                    &mut FilmWorld::default(),
                    oracle["strict"].as_bool().unwrap(),
                )
                .unwrap();
                assert_eq!(actual, expected, "traversal {i}");
                count += 1;
            }
        }
        assert_eq!(count, 2832);
        let mut registry = registry;
        registry.archetypes[3].components = vec!["high-frequency".into()];
        registry.archetypes[3].levels = vec![1];
        let cases = oracle["seeded"].as_array().unwrap();
        assert_eq!(cases.len(), 128);
        for (i, row) in cases.iter().enumerate() {
            let h = row["hex"].as_str().unwrap();
            let pay: Vec<_> = (0..h.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&h[j..j + 2], 16).unwrap())
                .collect();
            let seed = || {
                let mut w = FilmWorld::default();
                match row["mode"].as_u64().unwrap() {
                    0 => w.bind_full(0x40000001, 3),
                    1 => w.bind_full(0x80000001, 3),
                    2 => w.bind_soft(1, 3),
                    _ => {}
                }
                w
            };
            let variant = serde_json::from_value(row["variant"].clone()).unwrap();
            let mut world = seed();
            let actual =
                walk_keyframe_queue_records(&registry, &pay, variant, &encoding, &mut world, false)
                    .unwrap();
            let expected: KeyframeQueueWalk = serde_json::from_value(row["walk"].clone()).unwrap();
            assert_eq!(actual, expected, "seeded {i}");
            let slots: BTreeMap<String, _> = world
                .slots
                .iter()
                .map(|(slot, s)| {
                    (
                        slot.to_string(),
                        serde_json::json!({"id":s.full_id,"ti":s.archetype,"soft":s.soft}),
                    )
                })
                .collect();
            assert_eq!(
                serde_json::to_value(slots).unwrap(),
                row["slots"],
                "world {i}"
            );
            let variants: Vec<KeyframeQueueVariant> =
                serde_json::from_value(row["variants"].clone()).unwrap();
            let (best, all) =
                best_keyframe_queue_variant(&registry, &pay, &variants, &encoding, false, seed)
                    .unwrap();
            let expected: Vec<KeyframeQueueWalk> =
                serde_json::from_value(row["all"].clone()).unwrap();
            assert_eq!(all, expected, "seeded search {i}");
            let mut expected = row["best"].clone();
            if expected["ByTI"].is_null() {
                expected["ByTI"] = serde_json::json!({});
            }
            assert_eq!(
                best,
                serde_json::from_value(expected).unwrap(),
                "seeded best {i}"
            );
        }
    }

    #[test]
    fn native_keyframe_diagnostics() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kfq-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let oracle: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let hex = |h: &str| {
            (0..h.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>()
        };
        let anchors = oracle["anchors"].as_array().unwrap();
        assert_eq!(anchors.len(), 256);
        let mut count = 0;
        for (i, row) in anchors.iter().enumerate() {
            let expected: KeyframeAnchorShape =
                serde_json::from_value(row["shape"].clone()).unwrap();
            let actual = measure_keyframe_anchors(&hex(row["hex"].as_str().unwrap()));
            assert_eq!(actual, expected, "anchor {i}");
            count += actual.records;
        }
        assert!(count > 0);
        let cases = oracle["walks"].as_array().unwrap();
        assert_eq!(cases.len(), 256);
        let mut overruns = 0;
        for (i, row) in cases.iter().enumerate() {
            let expected: Vec<KeyframeQueueVariant> =
                serde_json::from_value(row["variants"].clone()).unwrap();
            assert_eq!(
                keyframe_queue_variants(row["from"].as_i64().unwrap(), row["to"].as_i64().unwrap()),
                expected,
                "variants {i}"
            );
            let walks: Vec<KeyframeQueueWalk> = row["measurements"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| {
                    let w: KeyframeQueueWalk = serde_json::from_value(m["walk"].clone()).unwrap();
                    assert_eq!(
                        w.coverage(),
                        m["coverage"].as_f64().unwrap(),
                        "coverage {i}"
                    );
                    let tis: Vec<u32> = serde_json::from_value(m["tis"].clone()).unwrap();
                    assert_eq!(w.sorted_type_indices(), tis);
                    overruns += usize::from(w.overrun);
                    w
                })
                .collect();
            let mut best = row["best"].clone();
            if best["ByTI"].is_null() {
                best["ByTI"] = serde_json::json!({});
            }
            let expected: KeyframeQueueWalk = serde_json::from_value(best).unwrap();
            assert_eq!(
                best_keyframe_queue_measurement(&walks),
                expected,
                "best {i}"
            );
        }
        assert!(overruns > 0);
        eprintln!("KFQ: {count} recovered anchors, {overruns} overrun walks");
    }
}
