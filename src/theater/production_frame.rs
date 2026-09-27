//! Native default production frame policy: message/entity/control classes and world admission.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionEntityEnd {
    Marker(RecordHeader),
    Rejected {
        header: RecordHeader,
        reason: FilmViewAdmission,
    },
    Failure(EntityViewStop),
    Truncated,
    PayloadBoundary,
    RecordLimit,
}
/// Native entity-view admission counters for one production frame.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionAdmissionDiagnostics {
    pub rejected_unbound: u64,
    pub rejected_other_view: u64,
    pub anticipated_bindings: std::collections::BTreeMap<u32, u64>,
}
impl ProductionAdmissionDiagnostics {
    pub fn is_empty(&self) -> bool {
        self.rejected_unbound == 0
            && self.rejected_other_view == 0
            && self.anticipated_bindings.is_empty()
    }
    fn observe(&mut self, admission: &FilmViewAdmission) {
        match admission {
            FilmViewAdmission::Unbound => self.rejected_unbound += 1,
            FilmViewAdmission::OtherView => self.rejected_other_view += 1,
            FilmViewAdmission::Anticipated(d) => {
                *self.anticipated_bindings.entry(d.archetype).or_default() += 1
            }
            FilmViewAdmission::Allowed => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionFrame {
    /// Failed native header reads and refused NEW binding mutations.
    #[serde(default, skip_serializing_if = "FilmReadDiagnostics::is_empty")]
    pub header_diagnostics: FilmReadDiagnostics,
    /// None in older exports that did not retain these native counters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admission_diagnostics: Option<ProductionAdmissionDiagnostics>,
    /// Optional 32-bit per-record words, in read order, including End/rejected
    /// headers. Signed ranges are payload-relative; negative positions identify
    /// absent head bits, and padded_bits identifies the absent tail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub record_prefixes: Vec<ComponentField>,
    pub messages: Option<DecodedFrameView>,
    pub records: Vec<EntityRecord>,
    pub controls: Option<DecodedFrameView>,
    pub entity_end: Option<ProductionEntityEnd>,
    pub views_completed: usize,
    pub end_bit: i64,
    /// Number of synthetic zero tail bits consumed by the native reader.
    #[serde(default)]
    pub padded_bits: usize,
}
/// Decode the reference's default `ClassesDeVue`/`TablesParVue` policy.
/// `start_bit=2` begins at the packet preamble; a localized event packet starts directly
/// at its entity view. Rejected entity headers terminate that view without a body.
/// Bindings persist across calls. This does not enable optional chain-inference repair.
/// With extra_fields, the native inference loop reads a 32-bit word before each
/// entity header, including End. Unlike the generic loop it has no NEW/DEL guard.
pub fn decode_production_frame(
    data: &[u8],
    start_bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &mut FilmWorld,
) -> Option<ProductionFrame> {
    decode_production_frame_observed(data, start_bit, registry, encoding, world, |_, _, _| {})
}

pub(crate) fn decode_production_frame_observed(
    data: &[u8],
    start_bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &mut FilmWorld,
    mut observer: impl FnMut(&EntityRecord, u32, &FilmWorld),
) -> Option<ProductionFrame> {
    decode_production_frame_contextual(
        data,
        start_bit,
        registry,
        encoding,
        world,
        ProductionReaderContext {
            reader: None,
            preamble_bits: 2,
        },
        &mut observer,
    )
}

pub(crate) struct ProductionReaderContext<'a> {
    pub reader: Option<&'a NativeReaderContext>,
    pub preamble_bits: i64,
}

pub(crate) fn decode_production_frame_contextual(
    data: &[u8],
    start_bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &mut FilmWorld,
    context: ProductionReaderContext<'_>,
    mut observer: impl FnMut(&EntityRecord, u32, &FilmWorld),
) -> Option<ProductionFrame> {
    let start_bit = start_bit.try_into().ok()?;
    if !encoding.valid() {
        return None;
    }
    let mut body_encoding = std::borrow::Cow::Borrowed(encoding);
    if encoding.extra_fields {
        body_encoding.to_mut().extra_fields = false;
    }
    let mut out = ProductionFrame {
        header_diagnostics: Default::default(),
        admission_diagnostics: Some(ProductionAdmissionDiagnostics::default()),
        record_prefixes: Vec::new(),
        messages: None,
        records: vec![],
        controls: None,
        entity_end: None,
        views_completed: 0,
        end_bit: start_bit,
        padded_bits: 0,
    };
    if start_bit == context.preamble_bits && context.preamble_bits >= 1 {
        let messages = decode_message_view_signed(data, context.preamble_bits - 1);
        out.end_bit = messages.end_bit;
        let complete = messages.stop == FrameViewStop::Complete;
        out.messages = Some(messages);
        if !complete {
            out.padded_bits =
                super::bits::padded_from_native(out.end_bit, data.len().saturating_mul(8));
            if let Some(observer) = context.reader.and_then(|c| c.observer.as_ref()) {
                observer.publish_control_verdict(data, None);
            }
            return Some(out);
        }
        out.views_completed += 1;
    }
    world.current_view = 0;
    let mut hit_end = false;
    let mut capture_slot = 0;
    for _ in 0..8192 {
        if out.end_bit >= data.len().saturating_mul(8) as i64 {
            break;
        }
        if encoding.extra_fields {
            out.record_prefixes.push(ComponentField {
                name: "record.prefix".into(),
                bit: out.end_bit,
                width: 32,
                // Native skips this prefix. Retaining its available bits must
                // not introduce a read/panic before a prefix-repaired header.
                raw: native_bits_tolerant(data, out.end_bit, 32),
            });
        }
        let header_bit = out
            .end_bit
            .wrapping_add(if encoding.extra_fields { 32 } else { 0 });
        let header = super::records::decode_frame_header_signed(data, header_bit, encoding)?;
        if header.kind == RecordKind::End {
            out.end_bit = header.end_bit;
            out.entity_end = Some(ProductionEntityEnd::Marker(header));
            hit_end = true;
            break;
        }
        let id = header.id?;
        if header.kind == RecordKind::Delta {
            let reason = world.admit_delta(id, true);
            out.admission_diagnostics.as_mut().unwrap().observe(&reason);
            if let Some(observer) = context.reader.and_then(|c| c.observer.as_ref()) {
                observer.record_admission(&reason);
            }
            if matches!(
                reason,
                FilmViewAdmission::Unbound | FilmViewAdmission::OtherView
            ) {
                out.end_bit = header.end_bit;
                out.entity_end = Some(ProductionEntityEnd::Rejected { header, reason });
                hit_end = true;
                break;
            }
        }
        if header.kind == RecordKind::Delta {
            capture_slot = id & 0x3fff_ffff;
        }
        let mut bindings = EntityBindings::default();
        if let Some(ti) = world.archetype(id & 0x3fff_ffff) {
            bindings.bind(id, ti);
        }
        let Some(record) = super::components::decode_entity_record_with_capture_slots(
            data,
            header_bit,
            registry,
            &body_encoding,
            &bindings,
            context
                .reader
                .is_none_or(|c| c.profile.grammar.simulation_complete),
            super::components::RecordCaptureSlots {
                context: context.reader.cloned(),
                movement: Some(capture_slot),
                position: Some(capture_slot),
            },
        ) else {
            out.entity_end = Some(ProductionEntityEnd::Truncated);
            break;
        };
        out.end_bit = record.end_bit;
        observer(&record, capture_slot, world);
        let complete = record.stop == EntityViewStop::Complete;
        if complete {
            match record.header.kind {
                RecordKind::New => {
                    if let Some(refusal) =
                        world.bind_native_new(id, record.archetype?, record.header.start_bit)
                    {
                        if let Some(observer) = context.reader.and_then(|c| c.observer.as_ref()) {
                            observer.record_new_binding_refusal(&refusal);
                        }
                        out.header_diagnostics.new_binding_refusals.push(refusal);
                    }
                }
                RecordKind::Delete => world.unbind(id & 0x3fff_ffff),
                _ => {}
            }
        } else {
            out.entity_end = Some(ProductionEntityEnd::Failure(record.stop.clone()));
        }
        out.records.push(record);
        if !complete {
            break;
        }
    }
    if hit_end {
        out.views_completed += 1;
        let controls = components::decode_control_view_contextual(
            data,
            out.end_bit,
            encoding.position.as_ref(),
            context.reader,
        );
        out.end_bit = controls.end_bit;
        out.views_completed += usize::from(controls.stop == FrameViewStop::Complete);
        out.controls = Some(controls);
    } else if out.entity_end.is_none() {
        out.entity_end = Some(if out.records.len() == 8192 {
            ProductionEntityEnd::RecordLimit
        } else {
            ProductionEntityEnd::PayloadBoundary
        });
    }
    if let Some(observer) = context.reader.and_then(|c| c.observer.as_ref()) {
        observer.publish_control_verdict(data, out.controls.as_ref());
    }
    out.padded_bits = super::bits::padded_from_native(out.end_bit, data.len().saturating_mul(8));
    Some(out)
}

/// Locate the reference's strict event-list boundary: the first slot-123 delta
/// preceded by zero, spanning exactly 35 bits and carrying one component.
/// Candidate reads are side-effect free: no world mutations or published observations.
/// Returns `None` for no match or unsupported encoding; there is no width fallback.
pub fn locate_strict_entity_view(
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
) -> Option<usize> {
    if !encoding.valid() || encoding.extra_fields {
        return None;
    }
    let ti = world.archetype(123)?;
    for start in 2..data.len().saturating_mul(8).saturating_sub(35) {
        if data[(start - 1) / 8] & (1 << (7 - (start - 1) % 8)) != 0 {
            continue;
        }
        let Some(header) = decode_record_header(data, start, encoding.ids) else {
            continue;
        };
        let Some(id) = header.id else {
            continue;
        };
        if header.kind != RecordKind::Delta || id & 0x3fff_ffff != 123 {
            continue;
        }
        let mut bindings = EntityBindings::default();
        bindings.bind(id, ti);
        let Some(record) =
            super::components::decode_entity_record_at(data, start, registry, encoding, &bindings)
        else {
            continue;
        };
        if record.stop == EntityViewStop::Complete
            && record.end_bit == (start + 35) as i64
            && record.components.len() == 1
        {
            return Some(start);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn registry() -> FilmRegistry {
        FilmRegistry {
            archetypes: (0..4)
                .map(|index| FilmArchetype {
                    index,
                    components: if index == 3 {
                        vec!["high-frequency".into()]
                    } else {
                        vec![]
                    },
                    levels: if index == 3 { vec![1] } else { vec![] },
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        }
    }

    #[test]
    fn native_movement_hook_production() {
        check_production_observations(include_bytes!(
            "fixtures/movement-hook-production-v41.json.zlib"
        ));
    }
    #[test]
    fn native_position_hook_production() {
        check_production_observations(include_bytes!(
            "fixtures/position-hook-production-v41.json.zlib"
        ));
    }
    #[test]
    fn native_position_hook_resolved_profiles() {
        check_production_observations(include_bytes!(
            "fixtures/position-hook-resolved-v41.json.zlib"
        ));
    }
    fn check_production_observations(fixture: &[u8]) {
        #[derive(Deserialize)]
        struct Case {
            #[serde(default)]
            extra: bool,
            slots: Option<serde_json::Value>,
            map: Option<String>,
            capture: Option<PositionCaptureEncoding>,
            name: String,
            level: u32,
            unbound: bool,
            hex: String,
            encoding: PositionEncoding,
            observations: Vec<FilmComponentObservation>,
            records: serde_json::Value,
            views: usize,
            end: usize,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(
            cases.len(),
            if cases.iter().any(|c| c.extra) {
                1024
            } else {
                512
            }
        );
        for (i, c) in cases.into_iter().enumerate() {
            let mut registry = FilmRegistry {
                archetypes: (0..36)
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
            registry.archetypes[35].components = vec![c.name];
            registry.archetypes[35].levels = vec![c.level];
            let data: Vec<_> = c
                .hex
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                .collect();
            let mut encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: c.capture,
                ids: RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: Some(c.encoding),
                extra_fields: c.extra,
                corruption_check: false,
            };
            if let Some(map) = c.map.as_deref() {
                // Exercise the Rust factory, not the oracle's copied settings.
                encoding = resolve_v41_profile(&registry, None, Some(map))
                    .unwrap()
                    .frame_encoding(encoding.ids)
                    .unwrap();
            }
            let mut world = FilmWorld::default();
            world.bind_full(50, 35);
            if !c.unbound {
                world.bind_full(51, 35);
            }
            let frame =
                decode_production_frame(&data, 13, &registry, &encoding, &mut world).unwrap();
            let observations: Vec<_> = frame
                .records
                .iter()
                .flat_map(|r| r.diagnostics.component_observations.iter().cloned())
                .collect();
            assert_eq!(observations, c.observations, "production observations {i}");
            assert_eq!(
                serde_json::json!(frame.end_bit),
                serde_json::json!(c.end),
                "end {i}"
            );
            assert_eq!(frame.views_completed, c.views, "views {i}");
            assert_eq!(
                frame.record_prefixes.is_empty(),
                !c.extra,
                "prefix presence {i}"
            );
            if c.extra {
                assert_eq!(frame.record_prefixes[0].bit, 13);
                let terminal_prefix = matches!(
                    frame.entity_end,
                    Some(ProductionEntityEnd::Marker(_) | ProductionEntityEnd::Rejected { .. })
                );
                assert_eq!(
                    frame.record_prefixes.len(),
                    frame.records.len() + usize::from(terminal_prefix)
                );
                for prefix in &frame.record_prefixes {
                    assert_eq!(prefix.width, 32);
                    let raw = (prefix.bit..prefix.bit + 32).fold(0u64, |raw, bit| {
                        (raw << 1)
                            | u64::from(
                                data.get(crate::theater::bits::native_address(bit / 8))
                                    .map_or(0, |byte| (byte >> (7 - bit % 8)) & 1),
                            )
                    });
                    assert_eq!(prefix.raw, raw, "prefix source bits {i}");
                }
                for (prefix, record) in frame.record_prefixes.iter().zip(&frame.records) {
                    assert_eq!(
                        serde_json::json!(prefix.bit + 32),
                        serde_json::json!(record.header.start_bit)
                    );
                }
                if let Some(
                    ProductionEntityEnd::Marker(header)
                    | ProductionEntityEnd::Rejected { header, .. },
                ) = &frame.entity_end
                {
                    assert_eq!(
                        serde_json::json!(frame.record_prefixes.last().unwrap().bit + 32),
                        serde_json::json!(header.start_bit)
                    );
                }
            }

            let records: Vec<_> = frame
                .records
                .iter()
                .map(|r| serde_json::json!({"id":r.header.id.unwrap(),"end":if r.header.kind == RecordKind::Delete { 0 } else { r.end_bit }}))
                .collect();
            assert_eq!(
                serde_json::to_value(records).unwrap(),
                c.records,
                "records {i}"
            );
            if let Some(expected) = c.slots {
                let slots: std::collections::BTreeMap<_, _> = world.slots.iter().map(|(slot, state)| {
                    (*slot, serde_json::json!({"ti":state.archetype,"id":state.full_id,"soft":state.soft,"view":state.view.unwrap_or(-1)}))
                }).collect();
                assert_eq!(serde_json::to_value(slots).unwrap(), expected, "world {i}");
            }
            let restored: ProductionFrame =
                serde_json::from_slice(&serde_json::to_vec(&frame).unwrap()).unwrap();
            assert_eq!(restored, frame);
        }
    }
    #[test]
    fn view_rejections_and_anticipated_bindings_match_native_cursor() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/production-admission-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 16);
        let registry = registry();
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            ids: RecordIdLayout {
                low_bits: 5,
                base: 0,
            },
            mpp_widths: [9, 5],
            position: None,
            extra_fields: false,
            corruption_check: false,
        };
        for case in cases {
            let hex = case["hex"].as_str().unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let mode = case["mode"].as_u64().unwrap();
            let mut world = FilmWorld::default();
            match mode {
                0 => {}
                1 => world.bind_full(0x40000001, 3),
                2 => world.bind_full(0x80000001, 3),
                3 => {
                    world.current_view = 1;
                    world.bind_full(0x40000001, 3);
                }
                4 => world.bind_datum(1, 3),
                5..=7 => {
                    world.anticipated = Some(AnticipatedBindings::from_declarations([(
                        if mode == 7 { 0x80000001 } else { 0x40000001 },
                        if mode == 6 { 0 } else { 5 },
                        3,
                    )]))
                }
                _ => unreachable!(),
            }
            let frame = decode_production_frame(
                &data,
                case["start"].as_u64().unwrap() as usize,
                &registry,
                &encoding,
                &mut world,
            )
            .unwrap();
            assert_eq!(
                serde_json::to_value(frame.admission_diagnostics.as_ref().unwrap()).unwrap(),
                case["diagnostics"]
            );
            let restored: ProductionFrame =
                serde_json::from_value(serde_json::to_value(&frame).unwrap()).unwrap();
            assert_eq!(restored, frame);
            let mut old = serde_json::to_value(&frame).unwrap();
            old.as_object_mut().unwrap().remove("admission_diagnostics");
            assert!(
                serde_json::from_value::<ProductionFrame>(old)
                    .unwrap()
                    .admission_diagnostics
                    .is_none()
            );
            assert_eq!(frame.end_bit, case["end"], "{case}");
            assert_eq!(frame.views_completed, case["views"], "{case}");
            let expected = case["records"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default();
            assert_eq!(frame.records.len(), expected.len(), "{case}");
            for (record, expected) in frame.records.iter().zip(expected) {
                assert_eq!(
                    record.header.id,
                    Some(expected["ID"].as_u64().unwrap() as u32)
                );
                assert_eq!(record.end_bit, expected["Trace"]["EndBit"]);
                assert_eq!(record.stop, EntityViewStop::Complete);
            }
            assert_eq!(
                world.anticipations.get(&3).copied().unwrap_or(0),
                usize::from(mode == 5)
            );
            if matches!(mode, 0 | 3 | 6 | 7) {
                assert!(matches!(
                    frame.entity_end,
                    Some(ProductionEntityEnd::Rejected { .. })
                ));
            } else {
                assert!(matches!(
                    frame.entity_end,
                    Some(ProductionEntityEnd::Marker(_))
                ));
            }
        }
    }
    #[test]
    fn strict_locator_matches_native_without_mutating_world() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/strict-locator-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 256);
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            ids: RecordIdLayout {
                low_bits: 13,
                base: 0,
            },
            mpp_widths: [9, 5],
            position: None,
            extra_fields: false,
            corruption_check: false,
        };
        let registry = registry();
        let mut matches = 0;
        for case in cases {
            let hex = case["hex"].as_str().unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let mut world = FilmWorld::default();
            if case["bound"] == true {
                world.bind_full(0x8000007b, 3);
            }
            let snapshot = world.clone();
            let actual = locate_strict_entity_view(&data, &registry, &encoding, &world);
            assert_eq!(
                actual.map(|x| x as i64).unwrap_or(-1),
                case["offset"],
                "{case}"
            );
            assert_eq!(world, snapshot);
            matches += usize::from(actual.is_some());
        }
        assert!(matches > 0 && matches < 256);
    }
    #[test]
    fn default_fallbacks_are_explicit_and_match_native() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/native-default-fallbacks-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(rows.len(), 48);
        let reg = FilmRegistry {
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
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            ids: RecordIdLayout {
                low_bits: 5,
                base: 0,
            },
            mpp_widths: [9, 5],
            position: None,
            extra_fields: false,
            corruption_check: false,
        };
        for row in rows {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let mut world = FilmWorld::default();
            let frame = decode_production_frame(&data, 2, &reg, &encoding, &mut world).unwrap();
            assert_eq!(frame.end_bit, row["end"]);
            assert_eq!(frame.views_completed, row["views"]);
            assert_eq!(frame.records.len(), 1);
            let record = &frame.records[0];
            assert_eq!(record.stop, EntityViewStop::Complete);
            assert_eq!(record.end_bit, row["records"][0]["Trace"]["EndBit"]);
            assert!(record.default_state_fallback);
            assert_eq!(record.padded_bits, 0);
            assert_eq!(world.archetype(1), Some(row["ti"].as_u64().unwrap() as u32));
            assert_eq!(
                decode_default_state(&[], 0, row["ti"].as_u64().unwrap() as u32, [9, 5], None),
                ComponentDecode::Unsupported
            );
        }
    }
    #[test]
    fn production_padding_retains_real_prefix_and_reports_provenance() {
        // Preamble, delta slot 1/gen 1, baseline, sparse mask i0, then an 8-bit field.
        let bits = "00100001010000100000010101010";
        let mut data = vec![0u8; bits.len().div_ceil(8)];
        for (i, b) in bits.bytes().enumerate() {
            data[i / 8] |= (b - b'0') << (7 - i % 8);
        }
        data.truncate(3);
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            ids: RecordIdLayout {
                low_bits: 5,
                base: 0,
            },
            mpp_widths: [9, 5],
            position: None,
            extra_fields: false,
            corruption_check: false,
        };
        let mut world = FilmWorld::default();
        world.bind_full(0x40000001, 3);
        let frame = decode_production_frame(&data, 2, &registry(), &encoding, &mut world).unwrap();
        assert_eq!(frame.end_bit, 29);
        assert_eq!(frame.padded_bits, 5);
        assert_eq!(frame.records[0].padded_bits, 5);
        assert_eq!(frame.records[0].fields.last().unwrap().raw, 0b10100000);
        assert_eq!(frame.entity_end, Some(ProductionEntityEnd::PayloadBoundary));
        let mut bindings = EntityBindings::default();
        bindings.bind(0x40000001, 3);
        let bounded =
            decode_replication_frame(&data, &registry(), &encoding, &mut bindings).unwrap();
        assert_eq!(bounded.entities.unwrap().stop, EntityViewStop::Truncated);
    }
}
