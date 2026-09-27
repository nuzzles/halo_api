use super::*;
use serde_json::Value;
use std::io::Read;
#[test]
fn native_signed_component_widths() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/signed-width-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 24);
    for row in rows {
        let mode = row["mode"].as_u64().unwrap();
        let width = row["width"].as_i64().unwrap();
        let name = row["name"].as_str().unwrap();
        let hex = row["hex"].as_str().unwrap();
        let bytes: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![name.into()],
                levels: vec![0],
            }],
        };
        let mut context = NativeReaderContext::default();
        let widths = NativeSharedWidths::from_map([(name.into(), width)].into());
        if mode == 0 {
            context.profile.grammar.calibrated_widths = Some(widths.clone())
        } else {
            context.profile.grammar.stub_widths = Some(widths.clone())
        };
        let mut bindings = EntityBindings::default();
        bindings.bind(50, 0);
        let record = components::decode_entity_record_with_capture_slots(
            &bytes,
            0,
            &registry,
            &FrameEncoding {
                ids: RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                mpp_widths: [9, 5],
                position: None,
                extra_fields: false,
                corruption_check: false,
            },
            &bindings,
            true,
            components::RecordCaptureSlots {
                context: Some(context),
                movement: None,
                position: None,
            },
        )
        .unwrap();
        let expected = row["end"].as_i64().unwrap();
        for adjustment in &record.diagnostics.width_adjustments {
            assert_eq!(
                adjustment.native_end_bit(),
                Some(expected),
                "native endpoint mode={mode} width={width}"
            );
        }
        assert_eq!(record.end_bit, expected, "mode {mode} width {width}");
        assert_eq!(record.stop, EntityViewStop::Complete);
        if width < 0 && mode != 2 {
            let adjustment = &record.diagnostics.width_adjustments[0];
            assert_eq!(adjustment.width, width);
            assert_eq!(adjustment.end_bit, usize::try_from(expected).ok());
            assert_eq!(adjustment.bit, 25);
        } else {
            assert!(record.diagnostics.width_adjustments.is_empty());
        }
        assert_eq!(widths.get(name), Some(width));
        let restored: EntityRecord =
            serde_json::from_value(serde_json::to_value(&record).unwrap()).unwrap();
        assert_eq!(restored, record);
        for adjustment in &restored.diagnostics.width_adjustments {
            assert_eq!(adjustment.native_end_bit(), Some(expected));
        }
        let mut merged = FilmReadDiagnostics::default();
        merged.merge(&record.diagnostics);
        assert_eq!(
            merged.width_adjustments,
            record.diagnostics.width_adjustments
        );
    }
}

/// Native signed intermediate/end offsets, including recovery and panic outcomes.
#[test]
fn native_signed_component_continuation() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-continuation-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 96);
    let mut mismatches = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let bytes: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
            .collect();
        let mut names = vec!["signed-first".to_string(), "signed-second".to_string()];
        if row["tail"] == true {
            names.push("biped-emp-timer-component".into());
        }
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                levels: vec![0; names.len()],
                components: names,
            }],
        };
        let widths = NativeSharedWidths::from_map(
            [
                ("signed-first".into(), row["first"].as_i64().unwrap()),
                ("signed-second".into(), row["second"].as_i64().unwrap()),
            ]
            .into(),
        );
        let mut context = NativeReaderContext::default();
        if row["mode"] == 0 {
            context.profile.grammar.calibrated_widths = Some(widths);
        } else {
            context.profile.grammar.stub_widths = Some(widths);
        }
        let encoding = FrameEncoding {
            ids: RecordIdLayout {
                low_bits: 11,
                base: 0,
            },
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            mpp_widths: [9, 5],
            position: None,
            extra_fields: false,
            corruption_check: false,
        };
        let mut bindings = EntityBindings::default();
        bindings.bind(50, 0);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            components::decode_entity_record_with_capture_slots(
                &bytes,
                0,
                &registry,
                &encoding,
                &bindings,
                true,
                components::RecordCaptureSlots {
                    context: Some(context),
                    movement: None,
                    position: None,
                },
            )
        }));
        if result.is_err() != row["panic"].as_bool().unwrap() {
            mismatches.push(format!(
                "case {i}: panic expected {}, got {}",
                row["panic"],
                result.is_err()
            ));
            continue;
        }
        if let Ok(Some(record)) = result {
            let actual = serde_json::json!({
                "end": record.end_bit,
                "id": record.header.id,
                "archetype": record.archetype,
                "mask": record.mask,
                "ok": record.stop == EntityViewStop::Complete,
                "components": record.attempts.iter().map(|a| serde_json::json!({
                    "index": a.span.index, "name": a.span.name,
                    "start": a.span.start_bit, "ported": a.status,
                })).collect::<Vec<_>>(),
            });
            let expected = serde_json::json!({
                "end": row["end"], "id": row["record"]["ID"],
                "archetype": row["record"]["TypeIndex"],
                "mask": row["record"]["Trace"]["Mask"], "ok": row["ok"],
                "components": row["record"]["Trace"]["Comps"].as_array().unwrap().iter().map(|c| serde_json::json!({
                    "index": c["Index"], "name": c["Name"],
                    "start": c["StartBit"], "ported": c["Ported"],
                })).collect::<Vec<_>>(),
            });
            if actual != expected {
                mismatches.push(format!("case {i}: expected {expected}, got {actual}"));
            }
        } else if result.is_ok() {
            mismatches.push(format!("case {i}: missing record"));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} signed continuation mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

#[test]
fn native_signed_component_reader_cursor() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-component-cursor-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 80);
    for (i, row) in rows.iter().enumerate() {
        // wasm32 aborts on panic; host checks recovery and all post-panic cursors.
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
            .collect();
        let start = row["start"].as_i64().unwrap();
        let mut reader = NativeFilmReader::new(&data);
        reader.set_native_bit_position(start);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            reader.read_component(row["name"].as_str().unwrap(), 0, 35)
        }));
        assert_eq!(
            result.is_err(),
            row["panic"].as_bool().unwrap(),
            "panic {i}"
        );
        assert_eq!(
            serde_json::json!(reader.native_bit_position()),
            row["end"],
            "cursor {i}"
        );
        if let Ok(result) = result {
            let (status, read) = result.unwrap();
            assert_eq!(serde_json::json!(status), row["ported"], "status {i}");
            assert_eq!(read.start_bit, start);
            assert_eq!(serde_json::json!(read.end_bit), row["end"]);
            let restored: DecodedComponent =
                serde_json::from_value(serde_json::to_value(&read).unwrap()).unwrap();
            assert_eq!(restored, read);
        }
        reader.set_native_bit_position(0);
        assert_eq!(serde_json::json!(reader.read_bits_wide(8)), row["recovery"]);
        assert_eq!(
            serde_json::json!(reader.native_bit_position()),
            row["recoveryEnd"]
        );
    }
}

#[test]
fn native_signed_component_public_frame() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-continuation-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    let mut count = 0;
    for row in rows {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
            .collect();
        let end = row["end"].as_i64().unwrap();
        if row["panic"] == true
            || row["tail"] == true
            || end <= (data.len() * 8) as i64
            || row["first"].as_i64().unwrap() >= 0
        {
            continue;
        }
        count += 1;
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["signed-first".into(), "signed-second".into()],
                levels: vec![0; 2],
            }],
        };
        let widths = NativeSharedWidths::from_map(
            [
                ("signed-first".into(), row["first"].as_i64().unwrap()),
                ("signed-second".into(), row["second"].as_i64().unwrap()),
            ]
            .into(),
        );
        let mut cfg = NativeFrameConfig {
            id_low_bits: 11,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        if row["mode"] == 0 {
            cfg.context.profile.grammar.calibrated_widths = Some(widths);
        } else {
            cfg.context.profile.grammar.stub_widths = Some(widths);
        }
        let mut world = FilmWorld::default();
        world.bind_full(50, 0);
        let view = cfg
            .decode_inference_view(&data, 0, &registry, &mut world)
            .unwrap();
        assert_eq!(view.end_bit, end);
        assert_eq!(view.records.len(), 1);
        let record = view.records[0].decoded.as_ref().unwrap();
        assert_eq!(record.stop, EntityViewStop::Complete);
        assert_eq!(record.end_bit, end);
        assert_eq!(record.attempts.len(), 2);
        for (actual, expected) in record
            .attempts
            .iter()
            .zip(row["record"]["Trace"]["Comps"].as_array().unwrap())
        {
            assert_eq!(
                serde_json::json!(actual.span.start_bit),
                expected["StartBit"]
            );
            assert_eq!(serde_json::json!(actual.span.index), expected["Index"]);
            assert_eq!(serde_json::json!(actual.status), expected["Ported"]);
        }
        let restored: InferenceFrame =
            serde_json::from_value(serde_json::to_value(&view).unwrap()).unwrap();
        assert_eq!(restored, view);
    }
    assert_eq!(count, 4);
}

#[test]
fn native_frame_panic_cursor_and_world() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/frame-panic-cursor-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 80);
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        let names = vec![
            "biped-emp-timer-component".into(),
            "signed-frame-skip".into(),
            "biped-emp-timer-component".into(),
        ];
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: names,
                levels: vec![0; 3],
            }],
        };
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut cfg = NativeFrameConfig {
            id_low_bits: 11,
            packet_preamble_bits: 0,
            extra_fields: row["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        let widths = NativeSharedWidths::from_map(
            [("signed-frame-skip".into(), row["width"].as_i64().unwrap())].into(),
        );
        if row["mode"] == 0 {
            cfg.context.profile.grammar.calibrated_widths = Some(widths);
        } else {
            cfg.context.profile.grammar.stub_widths = Some(widths);
        }
        let observer = NativeFilmObserver::default();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured = events.clone();
        let hook_panic = row["hookPanic"].as_bool().unwrap();
        observer.set_hook(
            NativeHookKind::EmpTimer,
            Some(std::sync::Arc::new(move |p| {
                let NativeHookPublication::Component(FilmComponentObservation::EmpTimer {
                    quantum,
                }) = p
                else {
                    panic!("wrong hook")
                };
                captured.lock().unwrap().push(*quantum);
                assert!(!hook_panic, "oracle hook panic");
            })),
        );
        cfg.context.observer = Some(observer);
        let mut world = FilmWorld::default();
        world.bind_full(8, 0);
        world.bind_full(50, 0);
        let mut reader = NativeFilmReader::new(&data);
        reader.set_capture_slot(42);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            reader.read_frame_records(&registry, &mut world, &cfg)
        }));
        assert_eq!(
            result.is_err(),
            row["panic"].as_bool().unwrap(),
            "panic {i}"
        );
        assert_eq!(
            serde_json::json!(reader.native_bit_position()),
            row["end"],
            "cursor {i}"
        );
        assert_eq!(
            serde_json::json!(reader.capture_slot()),
            row["slot"],
            "slot {i}"
        );
        assert_eq!(
            serde_json::json!(*events.lock().unwrap()),
            row["events"],
            "events {i}"
        );
        assert_eq!(
            world.slots.len(),
            row["slots"].as_object().unwrap().len(),
            "world {i}"
        );
        for (slot, state) in &world.slots {
            let expected = &row["slots"][slot.to_string()];
            assert_eq!(
                serde_json::json!(state.full_id),
                expected["FullID"],
                "identity {i}"
            );
            assert_eq!(
                serde_json::json!(state.archetype),
                expected["TypeIndex"],
                "archetype {i}"
            );
        }
        if let Ok(result) = result {
            assert_eq!(
                result.unwrap().stop == EntityViewStop::Complete,
                row["complete"].as_bool().unwrap(),
                "completion {i}"
            );
        }
        assert_eq!(reader.profile(), cfg.context.profile, "profile {i}");
        reader.set_native_bit_position(0);
        assert_eq!(
            serde_json::json!(reader.read_bits_wide(8)),
            row["recovery"],
            "recovery {i}"
        );
        assert_eq!(
            serde_json::json!(reader.native_bit_position()),
            row["recoveryEnd"],
            "recovery cursor {i}"
        );
    }
}

#[test]
fn native_source_frame_panic_cursor_and_bindings() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/frame-panic-cursor-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    let mut count = 0;
    for (i, row) in rows.iter().enumerate() {
        // This source-only API has static unsigned width configuration and no
        // live observer. Compare all native cases in that represented domain.
        if row["hookPanic"] == true || row["width"].as_i64().unwrap() < 0 {
            continue;
        }
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        let width = usize::try_from(row["width"].as_i64().unwrap()).unwrap();
        count += 1;
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![
                    "biped-emp-timer-component".into(),
                    "signed-frame-skip".into(),
                    "biped-emp-timer-component".into(),
                ],
                levels: vec![0; 3],
            }],
        };
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut widths = ComponentWidthOverrides::default();
        if row["mode"] == 0 {
            widths.calibrated.insert("signed-frame-skip".into(), width);
        } else {
            widths.stubs.insert("signed-frame-skip".into(), width);
        }
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            component_widths: widths,
            new_record: Default::default(),
            position_capture: None,
            native_id_low_bits: Some(11),
            ids: RecordIdLayout {
                low_bits: 11,
                base: 0,
            },
            mpp_widths: [9, 5],
            position: None,
            extra_fields: row["extra"].as_bool().unwrap(),
            corruption_check: false,
        };
        let mut reader = NativeFilmBits::new(&data);
        let mut bindings = EntityBindings::default();
        bindings.bind(8, 0);
        bindings.bind(50, 0);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            decode_native_frame_records(&mut reader, &registry, &encoding, &mut bindings, true, 0)
        }));
        assert_eq!(
            result.is_err(),
            row["panic"].as_bool().unwrap(),
            "panic {i}"
        );
        assert_eq!(
            serde_json::json!(reader.position()),
            row["end"],
            "cursor {i}"
        );
        assert_eq!(
            bindings.slots.len(),
            row["slots"].as_object().unwrap().len(),
            "bindings {i}"
        );
        for (slot, binding) in &bindings.slots {
            let expected = &row["slots"][slot.to_string()];
            assert_eq!(serde_json::json!(binding.id), expected["FullID"], "id {i}");
            assert_eq!(
                serde_json::json!(binding.archetype),
                expected["TypeIndex"],
                "type {i}"
            );
        }
        if let Ok(result) = result {
            assert_eq!(
                result.unwrap().stop == EntityViewStop::Complete,
                row["complete"].as_bool().unwrap(),
                "completion {i}"
            );
        }
        reader.set_position(0);
        assert_eq!(
            serde_json::json!(reader.read_wide(8)),
            row["recovery"],
            "recovery {i}"
        );
        assert_eq!(
            serde_json::json!(reader.position()),
            row["recoveryEnd"],
            "recovery cursor {i}"
        );
    }
    assert_eq!(count, if cfg!(target_arch = "wasm32") { 8 } else { 24 });
}

#[test]
fn native_signed_record_headers() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-header-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 411);
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut reader = NativeFilmBits::new(&data);
        reader.set_position(row["start"].as_i64().unwrap());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            decode_native_record_header(
                &mut reader,
                row["width"].as_i64().unwrap(),
                row["base"].as_u64().unwrap() as u32,
            )
        }));
        assert_eq!(
            result.is_err(),
            row["panic"].as_bool().unwrap(),
            "panic {i}"
        );
        assert_eq!(serde_json::json!(reader.position()), row["end"], "end {i}");
        if let Ok(header) = result {
            let kind = match header.kind {
                RecordKind::End => 0,
                RecordKind::New => 1,
                RecordKind::Delete => 2,
                RecordKind::Delta => 3,
            };
            assert_eq!(serde_json::json!(kind), row["kind"], "kind {i}");
            assert_eq!(
                serde_json::json!(header.id.unwrap_or(0)),
                row["id"],
                "id {i}"
            );
            assert_eq!(serde_json::json!(header.start_bit), row["start"]);
            assert_eq!(serde_json::json!(header.end_bit), row["end"]);
            let restored: RecordHeader =
                serde_json::from_value(serde_json::to_value(&header).unwrap()).unwrap();
            assert_eq!(restored, header);
        }
        reader.set_position(0);
        assert_eq!(serde_json::json!(reader.read_wide(8)), row["recovery"]);
        assert_eq!(serde_json::json!(reader.position()), row["recoveryEnd"]);
    }
}

#[test]
fn native_signed_header_frame_entries() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-header-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    let mut count = 0;
    for (i, row) in rows.iter().enumerate() {
        // A successful non-End header would enter a body not covered by this
        // header-only oracle. Panics and End records are whole-frame outcomes.
        if row["panic"] != true && row["kind"] != 0 {
            continue;
        }
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        count += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let start = row["start"].as_i64().unwrap();
        let width = row["width"].as_i64().unwrap();
        let base = row["base"].as_u64().unwrap() as u32;
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![],
        };
        let cfg = NativeFrameConfig {
            id_low_bits: width,
            id_base: base,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        let mut reader = NativeFilmReader::new(&data);
        reader.set_native_bit_position(start);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            reader.read_frame_records(&registry, &mut FilmWorld::default(), &cfg)
        }));
        assert_eq!(
            result.is_err(),
            row["panic"].as_bool().unwrap(),
            "live panic {i}"
        );
        assert_eq!(
            serde_json::json!(reader.native_bit_position()),
            row["end"],
            "live end {i}"
        );
        if let Ok(view) = result {
            let view = view.unwrap();
            assert_eq!(view.stop, EntityViewStop::Complete);
            assert_eq!(view.records.len(), 1);
            assert_eq!(view.records[0].header.start_bit, start);
            if reader.native_bit_position() < 0 {
                assert_eq!(reader.padded_bits(), 0);
            }
        }
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            native_id_low_bits: Some(width),
            ids: RecordIdLayout { low_bits: 0, base },
            mpp_widths: [9, 5],
            position: None,
            extra_fields: false,
            corruption_check: false,
        };
        let mut source = NativeFilmBits::new(&data);
        source.set_position(start);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            decode_native_frame_records(
                &mut source,
                &registry,
                &encoding,
                &mut EntityBindings::default(),
                true,
                0,
            )
        }));
        assert_eq!(
            result.is_err(),
            row["panic"].as_bool().unwrap(),
            "source panic {i}"
        );
        assert_eq!(
            serde_json::json!(source.position()),
            row["end"],
            "source end {i}"
        );
        if let Ok(view) = result {
            let view = view.unwrap();
            assert_eq!(view.stop, EntityViewStop::Complete);
            assert_eq!(view.records.len(), 1);
            assert_eq!(view.records[0].header.start_bit, start);
        }
    }
    assert_eq!(
        count,
        if cfg!(target_arch = "wasm32") {
            225
        } else {
            361
        }
    );
}

#[test]
fn native_mpp_wide_frames() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(include_bytes!("fixtures/mpp-domain-v41.json.zlib").as_slice())
        .read_to_end(&mut raw)
        .unwrap();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    let rows = fixture["frames"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    for (i, row) in rows.iter().enumerate() {
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: (0..37)
                .map(|index| FilmArchetype {
                    index,
                    components: vec![],
                    levels: vec![],
                })
                .collect(),
        };
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        let width = row["width"].as_i64().unwrap();
        let field = if row["axis"] == 0 {
            cfg.context.profile.mpp.lead = width;
            "mpp.lead"
        } else {
            cfg.context.profile.mpp.index = width;
            "mpp.index"
        };
        let observer = NativeFilmObserver::default();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured = events.clone();
        observer.set_hook(
            NativeHookKind::Mpp,
            Some(std::sync::Arc::new(move |p| {
                let NativeHookPublication::Component(FilmComponentObservation::Mpp {
                    field,
                    value,
                    present,
                }) = p
                else {
                    panic!("wrong hook")
                };
                captured
                    .lock()
                    .unwrap()
                    .push(serde_json::json!({"field":field,"value":value,"present":present}));
            })),
        );
        cfg.context.observer = Some(observer);
        let mut reader = NativeFilmReader::new(&data);
        let mut world = FilmWorld::default();
        let view = reader
            .read_frame_records(&registry, &mut world, &cfg)
            .unwrap();
        assert_eq!(
            serde_json::json!(reader.native_bit_position()),
            row["end"],
            "end {i}"
        );
        assert_eq!(
            serde_json::json!(view.stop == EntityViewStop::Complete),
            row["complete"]
        );
        assert_eq!(
            serde_json::json!(*events.lock().unwrap()),
            row["events"],
            "events {i}"
        );
        let records: Vec<_> = view
            .records
            .iter()
            .filter(|r| r.header.kind != RecordKind::End)
            .collect();
        let expected = row["records"].as_array().unwrap();
        assert_eq!(records.len(), expected.len());
        for (r, e) in records.iter().zip(expected) {
            assert_eq!(serde_json::json!(r.header.id), e["ID"]);
            assert_eq!(serde_json::json!(r.end_bit), e["Trace"]["EndBit"]);
            assert_eq!(serde_json::json!(r.archetype), e["TypeIndex"]);
        }
        assert_eq!(world.slots.len(), row["slots"].as_object().unwrap().len());
        assert_eq!(world.slots[&1].archetype, 36);
        let wide = records[0].fields.iter().find(|f| f.name == field).unwrap();
        assert_eq!(wide.width, width as u64);
        assert_eq!(wide.raw, 0);
        let mut next = wide.bit;
        for f in records[0]
            .fields
            .iter()
            .filter(|f| f.name.starts_with(&format!("{field}.discarded[")))
        {
            assert_eq!(f.bit, next);
            assert!(f.width <= 64);
            let expected = (0..f.width).fold(0u64, |v, j| {
                let bit = usize::try_from(f.bit + j as i64).unwrap();
                (v << 1) | u64::from((data[bit / 8] >> (7 - bit % 8)) & 1)
            });
            assert_eq!(f.raw, expected);
            next += f.width as i64;
        }
        assert_eq!(next, (data.len() * 8) as i64, "source prefix coverage {i}");
        assert!(records[0].fields.len() < 64, "bounded retention {i}");
        let restored: DecodedEntityView =
            serde_json::from_value(serde_json::to_value(&view).unwrap()).unwrap();
        assert_eq!(restored, view);
    }
}

#[test]
fn native_position_width_domain() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/position-domain-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 18);
    for (i, row) in rows.iter().enumerate() {
        let width = row["width"].as_u64().unwrap();
        let mode = row["mode"].as_u64().unwrap();
        let name = row["name"].as_str().unwrap();
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        let movement = &mut cfg.context.profile.movement;
        movement.world_object.index_bits = 1;
        movement.world_object.axis_bits = [6, 7, 8];
        movement.world_object.region = 0;
        movement.traversal.index_bits = 1;
        movement.traversal.axis_bits = [6, 7, 8];
        movement.delta_axis_width = 0;
        movement.delta_quantum = 0.25;
        movement.full_precision = false;
        movement.delta_has_handle_tail = false;
        movement.range = [[0., 1.]; 3];
        match mode {
            0 => movement.world_object.index_bits = width,
            1 | 6 | 7 => movement.world_object.axis_bits[1] = width,
            2 | 8 => movement.traversal.index_bits = width,
            3 | 5 => movement.traversal.axis_bits[1] = width,
            4 => movement.delta_axis_width = width,
            _ => unreachable!(),
        }
        cfg.context.profile.grammar.baseline_scope = false;
        let observer = NativeFilmObserver::default();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
        for kind in [
            NativeHookKind::Position,
            NativeHookKind::AbilityNonPredicted,
        ] {
            let captured = events.clone();
            observer.set_hook(
                kind,
                Some(std::sync::Arc::new(move |p| {
                    let NativeHookPublication::Component(value) = p else {
                        panic!("unexpected hook")
                    };
                    captured
                        .lock()
                        .unwrap()
                        .push(serde_json::to_value(value).unwrap());
                })),
            );
        }
        cfg.context.observer = Some(observer);
        for frame in [false, true] {
            events.lock().unwrap().clear();
            let hex = row[if frame { "frameHex" } else { "hex" }]
                .as_str()
                .unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                .collect();
            let mut reader = NativeFilmReader::with_context(&data, cfg.context.clone());
            if frame {
                let registry = FilmRegistry {
                    major_version: 41,
                    format_version: 27,
                    end_byte: 0,
                    truncated: false,
                    archetypes: (0..36)
                        .map(|index| FilmArchetype {
                            index,
                            components: if index == 35 {
                                vec![name.into()]
                            } else {
                                vec![]
                            },
                            levels: if index == 35 { vec![0] } else { vec![] },
                        })
                        .collect(),
                };
                let mut world = FilmWorld::default();
                world.bind_full(1, 35);
                let view = reader
                    .read_frame_records(&registry, &mut world, &cfg)
                    .unwrap();
                assert_eq!(
                    serde_json::json!(reader.native_bit_position()),
                    row["frameEnd"],
                    "frame cursor {i}"
                );
                assert_eq!(
                    serde_json::json!(view.stop == EntityViewStop::Complete),
                    row["complete"],
                    "completion {i}"
                );
                let records: Vec<_> = view
                    .records
                    .iter()
                    .filter(|r| r.header.kind != RecordKind::End)
                    .collect();
                let expected = row["records"].as_array().unwrap();
                assert_eq!(records.len(), expected.len(), "records {i}");
                for (record, expected) in records.iter().zip(expected) {
                    assert_eq!(serde_json::json!(record.header.id), expected["ID"]);
                    assert_eq!(
                        serde_json::json!(record.end_bit),
                        expected["Trace"]["EndBit"]
                    );
                    assert_eq!(serde_json::json!(record.archetype), expected["TypeIndex"]);
                }
                assert_eq!(
                    serde_json::json!(*events.lock().unwrap()),
                    row["events"],
                    "frame events {i}"
                );
            } else {
                reader.set_capture_slot(42);
                let (status, read) = reader.read_component(name, 0, 35).unwrap();
                let mut covered = vec![false; data.len() * 8];
                for field in &read.fields {
                    let count = field.width.min(64);
                    let start = field.bit + (field.width - count) as i64;
                    for j in 0..count {
                        let bit = start + j as i64;
                        if bit >= 0 && (bit as u64) < covered.len() as u64 {
                            let bit = bit as usize;
                            assert_eq!(
                                (field.raw >> (count - j - 1)) & 1,
                                u64::from((data[bit / 8] >> (7 - bit % 8)) & 1),
                                "source field {i}/{}",
                                field.name
                            );
                            covered[bit] = true;
                        }
                    }
                }
                let end = (read.end_bit as u64).min(covered.len() as u64) as usize;
                assert!(covered[..end].iter().all(|v| *v), "source coverage {i}");
                assert!(
                    read.fields.iter().any(|f| f.width == width),
                    "raw width {i}"
                );
                assert!(read.fields.len() < 64, "bounded retention {i}");
                assert_eq!(
                    serde_json::json!(status == Some(true)),
                    row["direct"]["ported"],
                    "status {i}"
                );
                assert_eq!(
                    serde_json::json!(read.end_bit),
                    row["direct"]["end"],
                    "cursor {i}"
                );
                assert_eq!(
                    serde_json::json!(*events.lock().unwrap()),
                    row["direct"]["events"],
                    "events {i}"
                );
                assert_eq!(
                    serde_json::json!(reader.read_bits(7)),
                    row["direct"]["tail"]
                );
                assert_eq!(
                    serde_json::json!(reader.native_bit_position()),
                    row["direct"]["tailEnd"]
                );
            }
        }
    }
}

#[test]
fn native_signed_inference_entries() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-inference-entry-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 90);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![],
                levels: vec![],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: row["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        cfg.context.profile.grammar.view_tables = false;
        cfg.context.profile.grammar.chain_inference = false;
        let mut world = FilmWorld::default();
        world.bind_full(0, 0);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cfg.decode_inference_view(&data, row["start"].as_i64().unwrap(), &registry, &mut world)
        }));
        assert_eq!(
            result.is_err(),
            row["panic"].as_bool().unwrap(),
            "panic {i}"
        );
        if let Ok(result) = result {
            let view = result.unwrap();
            assert_eq!(serde_json::json!(view.end_bit), row["end"], "end {i}");
            assert_eq!(serde_json::json!(view.hit_end()), row["hit"], "hit {i}");
            assert_eq!(
                serde_json::json!(view.inferred_count),
                row["inferred"],
                "inferred {i}"
            );
            let expected = row["records"].as_array().unwrap();
            assert_eq!(view.records.len(), expected.len(), "records {i}");
            for (record, expected) in view.records.iter().zip(expected) {
                assert_eq!(record.header.kind, RecordKind::Delta);
                assert_eq!(serde_json::json!(record.header.id), expected["ID"]);
                assert_eq!(serde_json::json!(record.archetype), expected["TypeIndex"]);
                assert_eq!(
                    serde_json::json!(record.decoded.as_ref().unwrap().end_bit),
                    expected["Trace"]["EndBit"]
                );
            }
            let expected_padding = if view.end_bit < 0 {
                0
            } else {
                usize::try_from((view.end_bit as u64).saturating_sub((data.len() * 8) as u64))
                    .unwrap_or(usize::MAX)
            };
            assert_eq!(view.padded_bits, expected_padding);
        }
        assert_eq!(world.slots.len(), 1);
        assert_eq!(world.slots[&0].archetype, 0);
    }
    assert_eq!(checked, if cfg!(target_arch = "wasm32") { 69 } else { 90 });
}

#[test]
fn native_signed_inference_trials() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-inference-trial-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 72);
    let mut confirmed_negative = [0; 2];
    let mut checked = [0; 2];
    let mut frames_checked = 0;
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["signed-trial".into()],
                levels: vec![0],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: row["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        let widths = NativeSharedWidths::default();
        widths.insert("signed-trial".into(), row["width"].as_i64().unwrap());
        if row["mode"] == 0 {
            cfg.context.profile.grammar.calibrated_widths = Some(widths);
        } else {
            cfg.context.profile.grammar.stub_widths = Some(widths);
        }
        let body = row["bodyStart"].as_i64().unwrap();
        let mut world = FilmWorld::default();
        world.bind_full(0, 0);
        let before = world.clone();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let encoding = cfg.contextual_frame_encoding().unwrap();
            let (trial, _) = super::components::chain_delta_body_trial_contextual(
                &data,
                body,
                &registry,
                0,
                &encoding,
                cfg.context.profile.grammar.simulation_complete,
                Some(&cfg.context),
            );
            assert_eq!(
                trial,
                row["ok"].as_bool().unwrap().then(|| (
                    row["end"].as_i64().unwrap(),
                    row["comps"].as_u64().unwrap() as usize
                )),
                "trial {i}"
            );
        }
        if !(cfg!(target_arch = "wasm32") && row["inferPanic"] == true) {
            checked[0] += 1;
            let actual = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cfg.infer_unbound(&data, body, &registry, &world)
            }));
            assert_eq!(
                actual.is_err(),
                row["inferPanic"].as_bool().unwrap(),
                "single panic {i}"
            );
            if let Ok(actual) = actual {
                let actual = actual.unwrap();
                let expected = row["inferOK"].as_bool().unwrap().then(|| {
                    (
                        row["inferTi"].as_u64().unwrap() as u32,
                        row["inferEnd"].as_i64().unwrap(),
                    )
                });
                assert_eq!(actual.result, expected, "single result {i}");
                if expected.is_some_and(|(_, end)| end < 0) {
                    confirmed_negative[0] += 1;
                }
            }
        }
        if !(cfg!(target_arch = "wasm32") && row["chainPanic"] == true) {
            checked[1] += 1;
            let actual = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cfg.infer_chain(&data, body, &registry, &world)
            }));
            assert_eq!(
                actual.is_err(),
                row["chainPanic"].as_bool().unwrap(),
                "chain panic {i}"
            );
            if let Ok(actual) = actual {
                let actual = actual.unwrap();
                assert_eq!(
                    actual.archetype,
                    row["chainOK"]
                        .as_bool()
                        .unwrap()
                        .then(|| row["chainTi"].as_u64().unwrap() as u32),
                    "chain type {i}"
                );
                assert_eq!(
                    serde_json::json!(actual.end_bit),
                    row["chainEnd"],
                    "chain end {i}"
                );
                assert_eq!(
                    serde_json::json!(actual.unique_archetype),
                    row["chainUnique"],
                    "unique {i}"
                );
                let counts: serde_json::Map<String, Value> = row["chainCounts"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter(|(_, v)| **v != 0)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                assert_eq!(
                    serde_json::to_value(&actual.diagnostics.chain_outcomes).unwrap(),
                    Value::Object(counts),
                    "chain counts {i}"
                );
                let restored: ChainInference =
                    serde_json::from_value(serde_json::to_value(&actual).unwrap()).unwrap();
                assert_eq!(actual, restored);
                if actual.archetype.is_some() && actual.end_bit < 0 {
                    confirmed_negative[1] += 1;
                }
            }
        }
        assert_eq!(world, before);
        for frame in row["frames"].as_array().unwrap() {
            frames_checked += 1;
            let hex = frame["hex"].as_str().unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                .collect();
            let mut cfg = cfg.clone();
            cfg.context.profile.grammar.view_tables = false;
            cfg.context.profile.grammar.chain_inference = frame["chain"].as_bool().unwrap();
            let mut world = before.clone();
            let view = cfg
                .decode_inference_view(
                    &data,
                    frame["start"].as_i64().unwrap(),
                    &registry,
                    &mut world,
                )
                .unwrap();
            assert_eq!(serde_json::json!(view.end_bit), frame["end"]);
            assert_eq!(serde_json::json!(view.hit_end()), frame["hit"]);
            assert_eq!(serde_json::json!(view.inferred_count), frame["inferred"]);
            let expected = frame["records"].as_array().unwrap();
            assert_eq!(view.records.len(), expected.len());
            for (record, expected) in view.records.iter().zip(expected) {
                assert_eq!(serde_json::json!(record.header.id), expected["ID"]);
                assert_eq!(serde_json::json!(record.archetype), expected["TypeIndex"]);
                if let Some(decoded) = &record.decoded {
                    assert_eq!(
                        serde_json::json!(decoded.end_bit),
                        expected["Trace"]["EndBit"]
                    );
                }
            }
            let inferred_end = match view.records[0].inference.as_ref().unwrap() {
                InferenceEvidence::SingleStep { end_bit, .. } => *end_bit,
                InferenceEvidence::Chain(inference) => inference.end_bit,
            };
            assert_eq!(inferred_end, row["inferEnd"].as_i64().unwrap());
            assert!(inferred_end < 0);
            let slots = frame["slots"].as_object().unwrap();
            assert_eq!(world.slots.len(), slots.len());
            for (slot, state) in &world.slots {
                let expected = &slots[&slot.to_string()];
                assert_eq!(serde_json::json!(state.archetype), expected["TypeIndex"]);
                assert_eq!(serde_json::json!(state.soft), expected["Soft"]);
            }
            let restored: InferenceFrame =
                serde_json::from_value(serde_json::to_value(&view).unwrap()).unwrap();
            assert_eq!(view, restored);
        }
    }
    assert_eq!(frames_checked, 4);
    assert_eq!(confirmed_negative, [2, 2]);
    assert_eq!(
        checked,
        if cfg!(target_arch = "wasm32") {
            [56, 40]
        } else {
            [72, 72]
        }
    );
}

#[test]
fn native_signed_resync_starts() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-resync-start-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 108);
    let mut checked = 0;
    let mut negative = [0; 2];
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["signed-scan".into()],
                levels: vec![0],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: row["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        let widths = NativeSharedWidths::default();
        widths.insert("signed-scan".into(), 0);
        cfg.context.profile.grammar.calibrated_widths = Some(widths);
        let mut world = FilmWorld::default();
        world.bind_full(0, 0);
        let before = world.clone();
        let targets = [0].into();
        let from = row["from"].as_i64().unwrap();
        let accepts = row["accept"].as_bool().unwrap();
        let mut calls = Vec::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cfg.scan_for_target_delta(
                &data,
                from,
                &registry,
                &world,
                &targets,
                |slot, position, has| {
                    calls.push((slot, position.map(f32::to_bits), has));
                    accepts
                },
            )
        }));
        assert_eq!(
            result.is_err(),
            row["panic"].as_bool().unwrap(),
            "context panic {i}"
        );
        let expected: Vec<_> = row["calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|call| {
                let position: [f32; 3] = serde_json::from_value(call["position"].clone()).unwrap();
                (
                    call["slot"].as_u64().unwrap() as u32,
                    position.map(f32::to_bits),
                    call["has"].as_bool().unwrap(),
                )
            })
            .collect();
        assert_eq!(calls, expected, "context calls {i}");
        if let Ok(result) = result {
            let result = result.unwrap().result;
            if result.as_ref().is_some_and(|(bit, _)| *bit < 0) {
                negative[0] += 1;
            }
            assert_eq!(
                serde_json::json!(result.map_or(-1, |(bit, _)| bit)),
                row["landing"],
                "context landing {i}"
            );
        }
        let profile = KillWalkProfile {
            simulation_complete: true,
            encoding: FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: Some(5),
                component_widths: ComponentWidthOverrides {
                    calibrated: [("signed-scan".into(), 0)].into(),
                    ..Default::default()
                },
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 5,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: cfg.extra_fields,
                corruption_check: false,
            },
        };
        let mut calls = Vec::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            scan_for_target_delta_observed(
                &data,
                from,
                &registry,
                &profile,
                &world,
                &targets,
                |record| {
                    calls.push(record.header.id.unwrap() & 0x3fff_ffff);
                    accepts
                },
            )
        }));
        assert_eq!(
            result.is_err(),
            row["panic"].as_bool().unwrap(),
            "plain panic {i}"
        );
        let expected: Vec<_> = row["calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["slot"].as_u64().unwrap() as u32)
            .collect();
        assert_eq!(calls, expected, "plain calls {i}");
        if let Ok(result) = result {
            if result.result.as_ref().is_some_and(|(bit, _)| *bit < 0) {
                negative[1] += 1;
            }
            assert_eq!(
                serde_json::json!(result.result.map_or(-1, |(bit, _)| bit)),
                row["landing"],
                "plain landing {i}"
            );
        }
        assert_eq!(world, before);
    }
    assert_eq!(negative, [1, 1]);
    assert_eq!(
        checked,
        if cfg!(target_arch = "wasm32") {
            70
        } else {
            108
        }
    );
}

#[test]
fn native_signed_resync_frames() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-resync-frame-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 8);
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: ["rewind", "signed-scan"]
                .into_iter()
                .enumerate()
                .map(|(index, name)| FilmArchetype {
                    index,
                    components: vec![name.into()],
                    levels: vec![0],
                })
                .collect(),
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: row["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        let widths = NativeSharedWidths::default();
        widths.insert("rewind".into(), -83);
        widths.insert("signed-scan".into(), 0);
        if row["mode"] == 0 {
            cfg.context.profile.grammar.calibrated_widths = Some(widths);
        } else {
            cfg.context.profile.grammar.stub_widths = Some(widths);
        }
        let mut world = FilmWorld::default();
        world.bind_full(1, 0);
        world.bind_full(0, 1);
        let before = world.clone();
        let mut calls = Vec::new();
        let frame = cfg
            .decode_resync_frame(&data, &registry, &mut world, &[0].into(), |slot, _, _| {
                calls.push(slot);
                row["accept"].as_bool().unwrap()
            })
            .unwrap();
        assert_eq!(serde_json::json!(calls), row["calls"], "calls {i}");
        assert_eq!(serde_json::json!(frame.end_bit), row["end"], "cursor {i}");
        let expected = row["records"].as_array().unwrap();
        assert_eq!(frame.records.len(), expected.len(), "records {i}");
        for (record, expected) in frame.records.iter().zip(expected) {
            assert_eq!(serde_json::json!(record.header.id), expected["ID"]);
            assert_eq!(serde_json::json!(record.archetype), expected["TypeIndex"]);
            assert_eq!(
                serde_json::json!(record.end_bit),
                expected["Trace"]["EndBit"]
            );
        }
        if cfg.extra_fields {
            assert_eq!(frame.records[0].end_bit, -32);
            assert!(
                frame.resync_bits.is_empty(),
                "negative landing rejected {i}"
            );
        }
        assert_eq!(world, before);
        let restored: NativeResyncFrame =
            serde_json::from_value(serde_json::to_value(&frame).unwrap()).unwrap();
        assert_eq!(frame, restored);
    }
}

#[test]
fn native_signed_harvest_successors() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-harvest-successor-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 224);
    let mut checked = 0;
    let mut negative = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["signed-harvest".into()],
                levels: vec![0],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: row["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        let widths = NativeSharedWidths::default();
        widths.insert("signed-harvest".into(), row["width"].as_i64().unwrap());
        cfg.context.profile.grammar.calibrated_widths = Some(widths);
        let mut world = FilmWorld::default();
        if row["hard"] == true {
            world.bind_full(0, 0);
        } else {
            world.bind_soft(0, 0);
        }
        let before = world.clone();
        let position = row["position"].as_i64().unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cfg.confirm_harvest_successor(&data, position, &registry, &world)
        }));
        assert_eq!(
            result.is_err(),
            row["panic"].as_bool().unwrap(),
            "panic {i}"
        );
        if let Ok(result) = result {
            let confirmed = result.unwrap().result.unwrap();
            assert_eq!(
                confirmed,
                row["confirmed"].as_bool().unwrap(),
                "confirmation {i}"
            );
            if confirmed && position < 0 {
                negative += 1;
            }
        }
        assert_eq!(world, before);
    }
    assert_eq!(negative, 2);
    assert_eq!(
        checked,
        if cfg!(target_arch = "wasm32") {
            164
        } else {
            224
        }
    );
}

#[test]
fn native_context_harvest() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/context-harvest-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 32);
    let mut partial = 0;
    let mut publications = 0;
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![
                    "object-position-dynamic-precision-component".into(),
                    "harvest-tail".into(),
                ],
                levels: vec![0, 0],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: row["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        cfg.context.profile.movement.delta_axis_width = 1;
        cfg.context.profile.movement.delta_quantum = 0.25;
        cfg.context.profile.movement.full_precision = false;
        cfg.context.profile.grammar.baseline_scope = false;
        cfg.context.profile.movement.delta_has_handle_tail = false;
        let widths = NativeSharedWidths::default();
        widths.insert("harvest-tail".into(), 8);
        if row["mode"] == 0 {
            cfg.context.profile.grammar.calibrated_widths = Some(widths.clone());
        } else {
            cfg.context.profile.grammar.stub_widths = Some(widths.clone());
        }
        let observer = NativeFilmObserver::default();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let emitted = events.clone();
        let mutable_widths = widths.clone();
        let mutation = row["mutation"].as_u64().unwrap();
        observer.set_hook(
            NativeHookKind::Position,
            Some(std::sync::Arc::new(move |p| {
                let NativeHookPublication::Component(value) = p else {
                    panic!("wrong hook")
                };
                emitted
                    .lock()
                    .unwrap()
                    .push(serde_json::to_value(value).unwrap());
                match mutation {
                    1 => {
                        mutable_widths.remove("harvest-tail");
                    }
                    2 => {
                        mutable_widths.insert("harvest-tail".into(), 0);
                    }
                    3 => {
                        mutable_widths.insert("harvest-tail".into(), 16);
                    }
                    _ => {}
                }
            })),
        );
        cfg.context.observer = Some(observer);
        let mut world = FilmWorld::default();
        world.bind_full(0, 0);
        world.bind_full(1, 0);
        let before = world.clone();
        let mode = if row["chain"] == true {
            HarvestConfirmation::ChainWalk
        } else {
            HarvestConfirmation::NextBound
        };
        let result = cfg
            .harvest_targets(&data, &registry, &world, &[0, 1].into(), mode)
            .unwrap();
        let records = result.result.unwrap();
        let expected = row["records"].as_array().unwrap();
        assert_eq!(records.len(), expected.len(), "records {i}");
        for (record, expected) in records.iter().zip(expected) {
            assert_eq!(serde_json::json!(record.header.id), expected["ID"]);
            assert_eq!(serde_json::json!(record.archetype), expected["TypeIndex"]);
            assert_eq!(
                serde_json::json!(record.end_bit),
                expected["Trace"]["EndBit"],
                "end {i}"
            );
            assert_eq!(serde_json::json!(record.mask), expected["Trace"]["Mask"]);
            let desync = match record.stop {
                EntityViewStop::Complete => -1,
                EntityViewStop::UnsupportedComponent { index, .. }
                | EntityViewStop::InvalidComponent { index } => index as i64,
                _ => panic!("unexpected stop"),
            };
            assert_eq!(serde_json::json!(desync), expected["DesyncAt"], "stop {i}");
            if desync >= 0 {
                partial += 1;
            }
            let restored: EntityRecord =
                serde_json::from_value(serde_json::to_value(record).unwrap()).unwrap();
            assert_eq!(*record, restored);
        }
        assert_eq!(
            serde_json::json!(*events.lock().unwrap()),
            row["events"],
            "events {i}"
        );
        publications += events.lock().unwrap().len();
        assert_eq!(
            widths.get("harvest-tail"),
            row["widths"]["harvest-tail"].as_i64(),
            "widths {i}"
        );
        assert_eq!(world, before);
    }
    assert!(partial > 0);
    assert!(publications > 0);
}

#[test]
fn native_signed_views() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-views-d61443e-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 792);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![],
                levels: vec![],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: row["extra"].as_bool().unwrap(),
            packet_preamble_bits: row["preamble"].as_i64().unwrap(),
            ..Default::default()
        };
        cfg.context.profile.grammar.view_classes = row["classes"].as_bool().unwrap();
        cfg.context.profile.grammar.chain_inference = false;
        let start = row["start"].as_i64().unwrap();
        let count = row["count"].as_i64().unwrap();
        let initial_world = || {
            let mut w = FilmWorld::default();
            w.bind_full(0, 0);
            w.current_view = 7;
            w
        };
        for contextual in [false, true] {
            let mut world = initial_world();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if contextual {
                    cfg.decode_inference_views(&data, start, count, &registry, &mut world)
                        .unwrap()
                } else {
                    decode_inference_views(
                        &data,
                        &registry,
                        &KillWalkProfile {
                            encoding: FrameEncoding {
                                ids: RecordIdLayout {
                                    low_bits: 5,
                                    base: 0,
                                },
                                extra_fields: cfg.extra_fields,
                                keyframe_layout: Default::default(),
                                keyframe_simulation_complete: None,
                                component_widths: Default::default(),
                                new_record: Default::default(),
                                position_capture: None,
                                native_id_low_bits: None,
                                mpp_widths: [9, 5],
                                position: None,
                                corruption_check: false,
                            },
                            simulation_complete: true,
                        },
                        &mut world,
                        InferenceViewsOptions {
                            inference: InferenceFrameOptions {
                                chain_inference: false,
                                view_tables: true,
                            },
                            view_classes: cfg.context.profile.grammar.view_classes,
                            view_count: count,
                            skip_lead_bits: start,
                            packet_preamble_bits: cfg.packet_preamble_bits,
                        },
                    )
                    .unwrap()
                }
            }));
            assert_eq!(
                result.is_err(),
                row["panic"] == true,
                "panic {i} contextual={contextual}"
            );
            assert_eq!(
                serde_json::json!(world.current_view),
                row["world_view"],
                "world {i}"
            );
            if let Ok(frame) = result {
                assert_eq!(
                    serde_json::json!(frame.end_bit),
                    row["end"],
                    "end {i} contextual={contextual}"
                );
                assert_eq!(
                    serde_json::json!(frame.views_completed),
                    row["views"],
                    "views {i}"
                );
                let records: Vec<_> = frame.entities.iter().flat_map(|v| &v.records).collect();
                let expected = row["records"].as_array().unwrap();
                assert_eq!(records.len(), expected.len(), "records {i}");
                for (record, expected) in records.iter().zip(expected) {
                    assert_eq!(record.header.kind, RecordKind::Delta);
                    assert_eq!(serde_json::json!(record.header.id), expected["ID"]);
                    assert_eq!(serde_json::json!(record.archetype), expected["TypeIndex"]);
                    let decoded = record.decoded.as_ref().unwrap();
                    assert_eq!(
                        serde_json::json!(decoded.end_bit),
                        expected["Trace"]["EndBit"]
                    );
                    assert_eq!(decoded.stop, EntityViewStop::Complete);
                }
                let restored: InferenceViews =
                    serde_json::from_value(serde_json::to_value(&frame).unwrap()).unwrap();
                assert_eq!(frame, restored);
            }
        }
        if cfg.context.profile.grammar.view_classes {
            let mut world = initial_world();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cfg.decode_production_views(&data, start, &registry, &mut world)
                    .unwrap()
            }));
            assert_eq!(
                result.is_err(),
                row["panic"] == true,
                "production panic {i}"
            );
            assert_eq!(
                serde_json::json!(world.current_view),
                row["world_view"],
                "production world {i}"
            );
            if let Ok(frame) = result {
                assert_eq!(
                    serde_json::json!(frame.end_bit),
                    row["end"],
                    "production end {i}"
                );
                assert_eq!(
                    serde_json::json!(frame.views_completed),
                    row["views"],
                    "production views {i}"
                );
                let expected = row["records"].as_array().unwrap();
                assert_eq!(
                    frame.records.len(),
                    expected.len(),
                    "production records {i}"
                );
                for (record, expected) in frame.records.iter().zip(expected) {
                    assert_eq!(serde_json::json!(record.header.id), expected["ID"]);
                    assert_eq!(
                        serde_json::json!(record.end_bit),
                        expected["Trace"]["EndBit"]
                    );
                    assert_eq!(record.stop, EntityViewStop::Complete);
                }
                let restored: ProductionFrame =
                    serde_json::from_value(serde_json::to_value(&frame).unwrap()).unwrap();
                assert_eq!(frame, restored);
            }
        }
    }
    assert_eq!(
        checked,
        if cfg!(target_arch = "wasm32") {
            698
        } else {
            792
        }
    );
}

#[test]
fn native_signed_view_readers() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-view-readers-d61443e-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 120);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let start = row["start"].as_i64().unwrap();
        let result = std::panic::catch_unwind(|| {
            if row["control"] == true {
                decode_control_view_signed(&data, start)
            } else {
                decode_message_view_signed(&data, start)
            }
        });
        assert_eq!(result.is_err(), row["panic"] == true, "panic {i}");
        if let Ok(view) = result {
            assert_eq!(serde_json::json!(view.end_bit), row["end"], "end {i}");
            assert_eq!(serde_json::json!(view.kinds), row["kinds"], "kinds {i}");
            assert_eq!(
                view.stop == FrameViewStop::Complete,
                row["complete"] == true,
                "complete {i}"
            );
            let restored: DecodedFrameView =
                serde_json::from_value(serde_json::to_value(&view).unwrap()).unwrap();
            assert_eq!(view, restored);
        }
    }
    assert_eq!(
        checked,
        if cfg!(target_arch = "wasm32") {
            72
        } else {
            120
        }
    );
}

#[test]
fn native_signed_view_continuation() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-view-continuation-d61443e-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 16);
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["view-tail".into()],
                levels: vec![0],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: true,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        cfg.context.profile.grammar.view_classes = row["classes"].as_bool().unwrap();
        cfg.context.profile.grammar.chain_inference = false;
        let widths = NativeSharedWidths::default();
        widths.insert("view-tail".into(), row["width"].as_i64().unwrap());
        if row["mode"] == 0 {
            cfg.context.profile.grammar.calibrated_widths = Some(widths);
        } else {
            cfg.context.profile.grammar.stub_widths = Some(widths);
        }
        let mut initial = FilmWorld::default();
        initial.bind_full(0, 0);
        initial.current_view = 7;
        let mut world = initial.clone();
        let frame = cfg
            .decode_inference_views(&data, 0, 3, &registry, &mut world)
            .unwrap();
        assert_eq!(serde_json::json!(frame.end_bit), row["end"], "end {i}");
        assert_eq!(
            serde_json::json!(frame.views_completed),
            row["views"],
            "views {i}"
        );
        assert_eq!(
            serde_json::json!(world.current_view),
            row["world_view"],
            "world {i}"
        );
        let records: Vec<_> = frame.entities.iter().flat_map(|v| &v.records).collect();
        let expected = row["records"].as_array().unwrap();
        assert_eq!(records.len(), expected.len());
        for (record, expected) in records.iter().zip(expected) {
            let decoded = record.decoded.as_ref().unwrap();
            assert_eq!(
                serde_json::json!(decoded.end_bit),
                expected["Trace"]["EndBit"]
            );
            assert_eq!(serde_json::json!(decoded.mask), expected["Trace"]["Mask"]);
            assert_eq!(decoded.stop, EntityViewStop::Complete);
            assert_eq!(decoded.header.start_bit, 32);
            assert_eq!(decoded.components.len(), 1);
        }
        if cfg.context.profile.grammar.view_classes {
            let mut world = initial.clone();
            let production = cfg
                .decode_production_views(&data, 0, &registry, &mut world)
                .unwrap();
            assert_eq!(production.end_bit, frame.end_bit, "production end {i}");
            assert_eq!(
                production.views_completed, frame.views_completed,
                "production views {i}"
            );
            assert_eq!(serde_json::json!(world.current_view), row["world_view"]);
            assert_eq!(production.records.len(), records.len());
            for (actual, expected) in production.records.iter().zip(&records) {
                assert_eq!(
                    actual,
                    expected.decoded.as_ref().unwrap(),
                    "production record {i}"
                );
            }
            if row["width"] == -83 {
                assert_eq!(production.record_prefixes[1].bit, -32);
                assert_eq!(production.record_prefixes[1].raw, 0);
                assert_eq!(production.controls.as_ref().unwrap().start_bit, 3);
            }
            let restored: ProductionFrame =
                serde_json::from_value(serde_json::to_value(&production).unwrap()).unwrap();
            assert_eq!(production, restored);
        }
        let restored: InferenceViews =
            serde_json::from_value(serde_json::to_value(&frame).unwrap()).unwrap();
        assert_eq!(frame, restored);
    }
}

#[test]
fn native_signed_march_entries() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-march-entry-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 84);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![],
                levels: vec![],
            }],
        };
        let mut encoding = native_keyframe_queue_encoding();
        encoding.ids.low_bits = 5;
        encoding.extra_fields = row["extra"].as_bool().unwrap();
        let mut world = FilmWorld::default();
        world.bind_full(0, 0);
        let initial = world.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            walk_march_records(
                &data,
                row["start"].as_i64().unwrap(),
                &registry,
                &encoding,
                &world,
                true,
            )
        }));
        assert_eq!(result.is_err(), row["panic"] == true, "panic {i}");
        assert_eq!(world, initial);
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: encoding.extra_fields,
            ..Default::default()
        };
        cfg.context.profile.grammar.generation_strict = true;
        let mut native_world = initial.clone();
        let native = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cfg.march_records(
                &data,
                row["start"].as_i64().unwrap(),
                &registry,
                &mut native_world,
            )
            .unwrap()
        }));
        assert_eq!(native.is_err(), row["panic"] == true, "context panic {i}");
        assert_eq!(native_world, initial);
        if let Ok(records) = &result {
            let actual = native.unwrap();
            assert_eq!(actual.len(), records.len(), "context records {i}");
            for (a, b) in actual.iter().zip(records) {
                assert_eq!(a.header, b.header);
                assert_eq!(a.end_bit, b.end_bit);
                assert_eq!(a.stop, b.stop);
            }
        }

        if let Ok(records) = result {
            let expected = row["records"].as_array().unwrap();
            assert_eq!(records.len(), expected.len(), "records {i}");
            for (record, expected) in records.iter().zip(expected) {
                assert_eq!(record.header.kind, RecordKind::Delta);
                assert_eq!(serde_json::json!(record.header.id), expected["ID"]);
                assert_eq!(serde_json::json!(record.archetype), expected["TypeIndex"]);
                assert_eq!(
                    serde_json::json!(record.end_bit),
                    expected["Trace"]["EndBit"]
                );
                assert_eq!(serde_json::json!(record.mask), expected["Trace"]["Mask"]);
                assert_eq!(record.stop, EntityViewStop::Complete);
            }
        }
    }
    assert_eq!(checked, if cfg!(target_arch = "wasm32") { 70 } else { 84 });
}

#[test]
fn native_queue_unused_widths() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/queue-unused-widths-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 210);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![],
                levels: vec![],
            }],
        };
        let variant: KeyframeQueueVariant = serde_json::from_value(row["variant"].clone()).unwrap();
        let mut world = FilmWorld::default();
        let initial = world.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            walk_keyframe_queue_records(
                &registry,
                &data,
                variant,
                &native_keyframe_queue_encoding(),
                &mut world,
                true,
            )
            .unwrap()
        }));
        assert_eq!(result.is_err(), row["panic"] == true, "panic {i}");
        assert_eq!(world, initial);
        if let Ok(walk) = result {
            assert_eq!(
                serde_json::to_value(&walk).unwrap(),
                row["result"],
                "result {i}"
            );
            let restored: KeyframeQueueWalk =
                serde_json::from_value(serde_json::to_value(&walk).unwrap()).unwrap();
            assert_eq!(walk, restored);
        }
    }
    assert_eq!(
        checked,
        if cfg!(target_arch = "wasm32") {
            168
        } else {
            210
        }
    );
}

#[test]
fn native_signed_wrapper_continuation() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-wrapper-continuation-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 10);
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["wrapper-tail".into()],
                levels: vec![0],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: true,
            ..Default::default()
        };
        let width = row["width"].as_i64().unwrap();
        let widths = NativeSharedWidths::default();
        widths.insert("wrapper-tail".into(), width);
        if row["mode"] == 0 {
            cfg.context.profile.grammar.calibrated_widths = Some(widths);
        } else {
            cfg.context.profile.grammar.stub_widths = Some(widths);
        }
        let mut world = FilmWorld::default();
        world.bind_full(0, 0);
        let initial = world.clone();
        let records = cfg.march_records(&data, 1, &registry, &mut world).unwrap();
        assert_eq!(world, initial, "restored {i}");
        let expected = row["records"].as_array().unwrap();
        assert_eq!(records.len(), expected.len(), "records {i}");
        for (record, expected) in records.iter().zip(expected) {
            assert_eq!(serde_json::json!(record.header.id), expected["ID"]);
            assert_eq!(
                record.header.start_bit,
                expected["HeaderBit"].as_i64().unwrap() + 32
            );
            assert_eq!(record.stop, EntityViewStop::Complete);
            if expected["Type"] == 3 {
                assert_eq!(record.header.kind, RecordKind::Delta);
                assert_eq!(
                    serde_json::json!(record.end_bit),
                    expected["Trace"]["EndBit"]
                );
                assert_eq!(serde_json::json!(record.mask), expected["Trace"]["Mask"]);
            } else {
                assert_eq!(record.header.kind, RecordKind::Delete);
            }
        }
        // Legacy source-only profiles have unsigned pointer-sized static widths.
        // Compare their represented domain; the contextual method above covers all rows.
        if let Ok(width) = usize::try_from(width) {
            let mut encoding = native_keyframe_queue_encoding();
            encoding.ids.low_bits = 5;
            encoding.extra_fields = true;
            if row["mode"] == 0 {
                encoding
                    .component_widths
                    .calibrated
                    .insert("wrapper-tail".into(), width);
            } else {
                encoding
                    .component_widths
                    .stubs
                    .insert("wrapper-tail".into(), width);
            }
            let plain = walk_march_records(&data, 1, &registry, &encoding, &initial, true);
            assert_eq!(plain.len(), records.len());
            for (a, b) in plain.iter().zip(&records) {
                assert_eq!(a.end_bit, b.end_bit);
                assert_eq!(a.header, b.header);
            }
        }
        let restored: Vec<EntityRecord> =
            serde_json::from_value(serde_json::to_value(&records).unwrap()).unwrap();
        assert_eq!(records, restored);
    }
}

#[test]
fn native_march_rollback() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/march-rollback-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 4);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["rollback-tail".into()],
                levels: vec![0],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: true,
            ..Default::default()
        };
        let widths = NativeSharedWidths::default();
        widths.insert("rollback-tail".into(), row["width"].as_i64().unwrap());
        if row["mode"] == 0 {
            cfg.context.profile.grammar.calibrated_widths = Some(widths);
        } else {
            cfg.context.profile.grammar.stub_widths = Some(widths);
        }
        let mut world = FilmWorld::default();
        world.bind_full(0, 0);
        world.bind_full(1, 0);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cfg.march_records(&data, 1, &registry, &mut world).unwrap()
        }));
        assert_eq!(result.is_err(), row["panic"] == true, "panic {i}");
        assert_eq!(
            world.slots.contains_key(&0),
            row["slot0"] == true,
            "slot0 {i}"
        );
        assert_eq!(
            world.slots.contains_key(&1),
            row["slot1"] == true,
            "slot1 {i}"
        );
        if let Ok(records) = result {
            assert_eq!(records.len(), row["records"].as_array().unwrap().len());
            assert_eq!(records[0].header.kind, RecordKind::Delete);
            assert_eq!(records[1].header.kind, RecordKind::Delta);
        }
    }
    assert_eq!(checked, if cfg!(target_arch = "wasm32") { 2 } else { 4 });
}

#[test]
fn native_signed_keyframe_chain() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-keyframe-chain-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 56);
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![],
        };
        let from = row["from"].as_i64().unwrap();
        let want = row["want"].as_i64().unwrap();
        let plain = chain_keyframe_records(
            &data,
            &registry,
            from,
            want,
            -1,
            &native_keyframe_queue_encoding(),
        );
        let contextual = NativeFrameConfig::default()
            .chain_keyframes(&data, &registry, from, want, -1)
            .unwrap();
        for result in [plain, contextual] {
            assert_eq!(
                serde_json::json!(result.reached),
                row["reached"],
                "reached {i}"
            );
            assert_eq!(serde_json::json!(result.skipped), row["skipped"]);
            assert_eq!(
                serde_json::json!(result.skipped_without_archetype),
                row["without"]
            );
            assert_eq!(
                serde_json::to_value(result.stop).unwrap(),
                row["stop"],
                "stop {i}"
            );
            assert!(result.attempts.is_empty());
        }
    }
}

#[test]
fn native_signed_keyframe_entries() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-keyframe-entry-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 24);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![],
                levels: vec![],
            }],
        };
        for contextual in [false, true] {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if contextual {
                    NativeFrameConfig::default()
                        .read_keyframe_record(&data, row["start"].as_i64().unwrap(), &registry)
                        .unwrap()
                } else {
                    decode_native_keyframe_record(
                        &data,
                        row["start"].as_i64().unwrap(),
                        &registry,
                        &native_keyframe_queue_encoding(),
                    )
                    .unwrap()
                }
            }));
            assert_eq!(
                result.is_err(),
                row["panic"] == true,
                "panic {i} context={contextual}"
            );
            if let Ok(record) = result {
                assert_eq!(
                    serde_json::json!(record.end_bit),
                    row["trace"]["EndBit"],
                    "end {i}"
                );
                assert_eq!(
                    serde_json::json!(record.archetype),
                    row["trace"]["TypeIndex"]
                );
                assert_eq!(record.stop, KeyframeStop::Complete);
                let restored: KeyframeRecord =
                    serde_json::from_value(serde_json::to_value(&record).unwrap()).unwrap();
                assert_eq!(record, restored);
            }
        }
    }
    assert_eq!(checked, if cfg!(target_arch = "wasm32") { 16 } else { 24 });
}

#[test]
fn native_context_keyframes() {
    use std::sync::{Arc, Mutex};
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/context-keyframe-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 30);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["biped-emp-timer-component".into(), "kf-tail".into()],
                levels: vec![0, 0],
            }],
        };
        let mut cfg = NativeFrameConfig::default();
        let widths = NativeSharedWidths::default();
        widths.insert("kf-tail".into(), 8);
        if row["mode"] == 0 {
            cfg.context.profile.grammar.calibrated_widths = Some(widths.clone());
        } else {
            cfg.context.profile.grammar.stub_widths = Some(widths.clone());
        }
        let events = Arc::new(Mutex::new(Vec::<u32>::new()));
        let observer = NativeFilmObserver::default();
        let width = row["width"].as_i64().unwrap();
        observer.set_hook(
            NativeHookKind::EmpTimer,
            Some({
                let events = events.clone();
                let widths = widths.clone();
                Arc::new(move |p| {
                    let NativeHookPublication::Component(FilmComponentObservation::EmpTimer {
                        quantum,
                    }) = p
                    else {
                        panic!("wrong hook")
                    };
                    events.lock().unwrap().push(*quantum);
                    widths.insert("kf-tail".into(), width);
                })
            }),
        );
        cfg.context.observer = Some(observer);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            match row["api"].as_u64().unwrap() {
                0 => {
                    let r = cfg.read_keyframe_record(&data, 1, &registry).unwrap();
                    let metadata = serde_json::json!({"end":r.end_bit});
                    (vec![r], metadata)
                }
                1 => {
                    let r = cfg.chain_keyframes(&data, &registry, 1, 189, -1).unwrap();
                    let metadata =
                        serde_json::json!({"reached":r.reached,"skipped":r.skipped,"stop":r.stop});
                    (
                        r.attempts.into_iter().filter_map(|a| a.record).collect(),
                        metadata,
                    )
                }
                _ => {
                    let r = cfg.read_keyframe_table(&data, &registry).unwrap();
                    let metadata = serde_json::json!({"stop":r.stop,"ends":r.records.iter().map(|a|a.end_bit).collect::<Vec<_>>()});
                    (
                        r.records.into_iter().filter_map(|a| a.record).collect(),
                        metadata,
                    )
                }
            }
        }));
        assert_eq!(result.is_err(), row["panic"] == true, "panic {i}");
        assert_eq!(
            serde_json::json!(*events.lock().unwrap()),
            row["events"],
            "events {i}"
        );
        assert_eq!(
            serde_json::json!(widths.get("kf-tail").unwrap()),
            row["final_width"]
        );
        if let Ok((records, metadata)) = result {
            let expected = match row["api"].as_u64().unwrap() {
                0 => serde_json::json!({"end":row["trace"]["EndBit"]}),
                1 => {
                    serde_json::json!({"reached":row["reached"],"skipped":row["skipped"],"stop":row["stop"]})
                }
                _ => {
                    serde_json::json!({"stop":row["stop"],"ends":row["records"].as_array().unwrap().iter().map(|r|r["BitEnd"].clone()).collect::<Vec<_>>()})
                }
            };
            assert_eq!(metadata, expected, "result {i}");
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].stop, KeyframeStop::Complete);
            assert_eq!(records[0].components.len(), 2);
            assert_eq!(records[0].components[0].start_bit, 173);
            assert_eq!(records[0].components[1].start_bit, 181);
            let restored: Vec<KeyframeRecord> =
                serde_json::from_value(serde_json::to_value(&records).unwrap()).unwrap();
            assert_eq!(records, restored);
        }
    }
    assert_eq!(checked, if cfg!(target_arch = "wasm32") { 28 } else { 30 });
}

#[test]
fn native_keyframe_layout_domain() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/keyframe-layout-domain-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 90);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![],
        };
        let mut cfg = NativeFrameConfig::default();
        cfg.context.profile.keyframe = NativeKeyframeLayout {
            header_bits: row["header"].as_i64().unwrap(),
            size_word_bits: row["word"].as_i64().unwrap(),
        };
        let restored: NativeScanProfile =
            serde_json::from_value(serde_json::to_value(&cfg.context.profile).unwrap()).unwrap();
        assert_eq!(restored, cfg.context.profile);
        let mut reader = NativeFilmReader::new(&[0]);
        let view = reader
            .read_frame_records(&registry, &mut FilmWorld::default(), &cfg)
            .unwrap();
        assert_eq!(
            serde_json::json!(reader.native_bit_position()),
            row["generic_end"]
        );
        assert_eq!(
            serde_json::json!(
                view.records
                    .iter()
                    .filter(|r| r.header.kind != RecordKind::End)
                    .count()
            ),
            row["generic_records"]
        );
        assert_eq!(
            serde_json::json!(view.stop != EntityViewStop::Complete),
            row["generic_error"]
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if row["api"] == 0 {
                let r = cfg
                    .chain_keyframes(&data, &registry, 1, i64::MAX, -1)
                    .unwrap();
                (
                    serde_json::json!({"reached":r.reached,"skipped":r.skipped,"without":r.skipped_without_archetype,"stop":r.stop}),
                    r.attempts,
                )
            } else {
                let r = cfg.read_keyframe_table(&data, &registry).unwrap();
                (
                    serde_json::json!({"stop":r.stop,"ends":r.records.iter().map(|a|a.end_bit).collect::<Vec<_>>()}),
                    r.records,
                )
            }
        }));
        assert_eq!(result.is_err(), row["panic"] == true, "panic {i}");
        if let Ok((actual, records)) = result {
            let expected = if row["api"] == 0 {
                serde_json::json!({"reached":row["reached"],"skipped":row["skipped"],"without":row["without"],"stop":row["stop"]})
            } else {
                serde_json::json!({"stop":row["stop"],"ends":row["records"].as_array().unwrap().iter().map(|r|r["BitEnd"].clone()).collect::<Vec<_>>()})
            };
            assert_eq!(actual, expected, "result {i}");
            assert_eq!(records.len(), 1);
            assert!(records[0].record.is_none());
            assert_eq!(records[0].archetype, u32::MAX);
            let restored: Vec<KeyframeChainAttempt> =
                serde_json::from_value(serde_json::to_value(&records).unwrap()).unwrap();
            assert_eq!(records, restored);
        }
    }
    assert_eq!(checked, if cfg!(target_arch = "wasm32") { 80 } else { 90 });
}

#[test]
fn native_keyframe_words() {
    use std::sync::{Arc, Mutex};
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/keyframe-words-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 194);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["biped-emp-timer-component".into()],
                levels: vec![0],
            }],
        };
        let mut cfg = NativeFrameConfig::default();
        cfg.context.profile.keyframe = NativeKeyframeLayout {
            header_bits: row["header"].as_i64().unwrap(),
            size_word_bits: row["word"].as_i64().unwrap(),
        };
        cfg.context.profile.grammar.corruption_check = row["check"].as_bool().unwrap();
        let observer = NativeFilmObserver::default();
        let events = Arc::new(Mutex::new(Vec::<u32>::new()));
        observer.set_hook(
            NativeHookKind::EmpTimer,
            Some({
                let events = events.clone();
                Arc::new(move |p| {
                    let NativeHookPublication::Component(FilmComponentObservation::EmpTimer {
                        quantum,
                    }) = p
                    else {
                        panic!("wrong hook")
                    };
                    events.lock().unwrap().push(*quantum);
                })
            }),
        );
        cfg.context.observer = Some(observer);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cfg.read_keyframe_record(&data, 1, &registry).unwrap()
        }));
        assert_eq!(result.is_err(), row["panic"] == true, "panic {i}");
        assert_eq!(
            serde_json::json!(*events.lock().unwrap()),
            row["events"],
            "events {i}"
        );
        if let Ok(record) = result {
            assert_eq!(
                serde_json::json!(record.end_bit),
                row["trace"]["EndBit"],
                "end {i}"
            );
            assert_eq!(record.stop, KeyframeStop::Complete);
            for (name, key) in [("default_guard", "n1"), ("components_guard", "n2")] {
                let field = record.fields.iter().find(|f| f.name == name).unwrap();
                assert_eq!(serde_json::json!(field.raw), row[key], "{name} raw {i}");
                assert_eq!(field.width, row["word"].as_i64().unwrap() as u64);
            }
            for field in record
                .fields
                .iter()
                .filter(|f| f.name.contains(".discarded["))
            {
                assert_eq!(field.width, 1);
                let bit = usize::try_from(field.bit).unwrap();
                assert_eq!(field.raw, u64::from((data[bit / 8] >> (7 - bit % 8)) & 1));
            }
            let expected = row["trace"]["Comps"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            assert_eq!(record.components.len(), expected.len());
            for (actual, expected) in record.components.iter().zip(expected) {
                assert_eq!(serde_json::json!(actual.start_bit), expected["StartBit"]);
            }
            let restored: KeyframeRecord =
                serde_json::from_value(serde_json::to_value(&record).unwrap()).unwrap();
            assert_eq!(record, restored);
        }
    }
    assert_eq!(
        checked,
        if cfg!(target_arch = "wasm32") {
            162
        } else {
            194
        }
    );
}

#[test]
fn native_signed_validated_resync() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/signed-validated-resync-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 126);
    let mut checked = 0;
    for (i, row) in rows.iter().enumerate() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        checked += 1;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let constructed = row["constructed"] == true;
        let name = if constructed {
            "object-dead-state-component"
        } else {
            "signed-scan"
        };
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![name.into()],
                levels: vec![0],
            }],
        };
        let mut world = FilmWorld::default();
        world.bind_full(if constructed { 50 } else { 0 }, 0);
        let before = world.clone();
        let targets = [row["target"].as_u64().unwrap() as u32].into();
        let from = row["from"].as_i64().unwrap();
        let mut cfg = NativeFrameConfig {
            id_low_bits: if constructed { 11 } else { 5 },
            extra_fields: row["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        cfg.context.profile.grammar.calibrated_widths = Some(NativeSharedWidths::from_map(
            [("signed-scan".into(), 0)].into(),
        ));
        let observer = NativeFilmObserver::default();
        for kind in [
            NativeHookKind::Position,
            NativeHookKind::UnitReference,
            NativeHookKind::MovementState,
        ] {
            observer.set_hook(
                kind,
                Some(std::sync::Arc::new(|_| {
                    panic!("speculative capture escaped")
                })),
            );
        }
        cfg.context.observer = Some(observer.clone());
        let profile = KillWalkProfile {
            encoding: FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                new_record: Default::default(),
                component_widths: ComponentWidthOverrides {
                    calibrated: [("signed-scan".into(), 0)].into(),
                    stubs: Default::default(),
                },
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: cfg.id_low_bits as usize,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: cfg.extra_fields,
                corruption_check: false,
            },
            simulation_complete: false,
        };
        for api in 0..2 {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if api == 0 {
                    cfg.validated_resync(&data, from, &registry, &world, &targets)
                        .unwrap()
                } else {
                    validated_chain_resync_observed(
                        &data, from, &registry, &profile, &world, &targets,
                    )
                }
            }));
            assert_eq!(result.is_err(), row["panic"] == true, "panic {i}/{api}");
            if let Ok(attempt) = result {
                let restored: FilmReadAttempt<i64> =
                    serde_json::from_value(serde_json::to_value(&attempt).unwrap()).unwrap();
                assert_eq!(restored, attempt);
                assert_eq!(
                    attempt.result,
                    row["ok"]
                        .as_bool()
                        .unwrap()
                        .then(|| row["landing"].as_i64().unwrap()),
                    "landing {i}/{api}"
                );
                assert_eq!(
                    attempt.diagnostics.validated_resyncs,
                    u64::from(row["ok"] == true),
                    "counter {i}/{api}"
                );
            }
            assert_eq!(world, before, "world {i}/{api}");
        }
        assert_eq!(
            observer.counters().validated_resyncs,
            u64::from(row["ok"] == true)
        );
        for kind in [
            NativeHookKind::Position,
            NativeHookKind::UnitReference,
            NativeHookKind::MovementState,
        ] {
            assert!(observer.has_hook(kind), "restored capture {i}");
        }
    }
    assert_eq!(
        checked,
        if cfg!(target_arch = "wasm32") {
            82
        } else {
            126
        }
    );
}

#[test]
fn native_isolation_gap_options() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/isolation-gap-domain-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 64);
    for row in rows {
        let mut options = BipedScanOptions {
            native_isolation_gap_ms: Some(row["gap_ms"].as_i64().unwrap()),
            isolation_gap_us: 42,
            ..Default::default()
        };
        let expected = row["enabled"]
            .as_bool()
            .unwrap()
            .then(|| row["threshold_us"].as_u64().unwrap());
        assert_eq!(options.isolation_threshold_us(), expected);
        let restored: BipedScanOptions =
            serde_json::from_value(serde_json::to_value(&options).unwrap()).unwrap();
        assert_eq!(options, restored);
        options.native_isolation_gap_ms = None;
        assert_eq!(options.isolation_threshold_us(), Some(42));
        options.isolation_gap_us = 0;
        assert_eq!(options.isolation_threshold_us(), None);
        let legacy = serde_json::to_value(&options).unwrap();
        assert!(legacy.get("native_isolation_gap_ms").is_none());
        let restored: BipedScanOptions = serde_json::from_value(legacy).unwrap();
        assert_eq!(restored, options);
    }
}

#[test]
fn native_overlapping_payloads() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/overlapping-payloads-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 36);
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let name = if row.get("timers").is_some() {
            "game-engine-round-timer-component"
        } else {
            "object-parent-state-component"
        };
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![name.into(), "rewind".into(), name.into()],
                levels: vec![
                    row["first"].as_u64().unwrap() as u32,
                    0,
                    row["second"].as_u64().unwrap() as u32,
                ],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        cfg.context.profile.grammar.calibrated_widths = Some(NativeSharedWidths::from_map(
            [("rewind".into(), row["rewind"].as_i64().unwrap())].into(),
        ));
        let (end, payloads) = if row["api"] == 0 {
            let record = cfg.read_keyframe_record(&data, 0, &registry).unwrap();
            assert_eq!(record.components.len(), 3);
            assert!(record.captured_payload(1).is_none());
            let restored: KeyframeRecord =
                serde_json::from_value(serde_json::to_value(&record).unwrap()).unwrap();
            assert_eq!(restored, record);
            (
                record.end_bit,
                [record.captured_payload(0), record.captured_payload(2)],
            )
        } else {
            let mut world = FilmWorld::default();
            world.bind_full(0, 0);
            let mut reader = NativeFilmReader::new(&data);
            let view = reader
                .read_frame_records(&registry, &mut world, &cfg)
                .unwrap();
            let record = &view.records[0];
            assert_eq!(record.attempts.len(), 3);
            assert!(record.captured_payload(1).is_none());
            let restored: EntityRecord =
                serde_json::from_value(serde_json::to_value(record).unwrap()).unwrap();
            assert_eq!(&restored, record);
            (
                reader.native_bit_position(),
                [record.captured_payload(0), record.captured_payload(2)],
            )
        };
        assert_eq!(serde_json::json!(end), row["end"], "end {i}");
        for (j, payload) in payloads.into_iter().enumerate() {
            if let Some(timers) = row.get("timers") {
                let v = &timers[j];
                let expected = DecodedRoundTimer {
                    quanta: [
                        v["QA"].as_u64().unwrap() as u16,
                        v["QB"].as_u64().unwrap() as u16,
                    ],
                    seconds: [
                        v["A"].as_f64().unwrap() as f32,
                        v["B"].as_f64().unwrap() as f32,
                    ],
                    tail: v["Tail"].as_u64().unwrap() as u8,
                };
                assert_eq!(
                    payload,
                    Some(CapturedComponentPayload::RoundTimer(expected)),
                    "timer {i}/{j}"
                );
                continue;
            }
            let event = &row["events"][j];
            let expected = NativeObjectParentState {
                parameter: event["Param"].as_u64().unwrap() as u32,
                start_bit: event["StartBit"].as_i64().unwrap(),
                end_bit: event["EndBit"].as_i64().unwrap(),
                free_read: event["FreeRead"].as_bool().unwrap(),
                free_bits: event["FreeBits"].as_u64().unwrap() as usize,
                tail3: event["HasTail3"]
                    .as_bool()
                    .unwrap()
                    .then(|| event["Tail3"].as_u64().unwrap() as u32),
                ..Default::default()
            };
            assert_eq!(
                payload,
                Some(CapturedComponentPayload::Parent(expected)),
                "payload {i}/{j}"
            );
        }
    }
}

#[test]
fn native_component_mask_wrap() {
    use std::sync::{Arc, Mutex};
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/component-mask-wrap-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 36);
    for (i, row) in rows.iter().enumerate() {
        let count = row["count"].as_u64().unwrap() as usize;
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["biped-emp-timer-component".into(); count],
                levels: vec![0; count],
            }],
        };
        let events = Arc::new(Mutex::new(Vec::<u32>::new()));
        let observer = NativeFilmObserver::default();
        observer.set_hook(
            NativeHookKind::EmpTimer,
            Some({
                let events = events.clone();
                Arc::new(move |p| {
                    let NativeHookPublication::Component(FilmComponentObservation::EmpTimer {
                        quantum,
                    }) = p
                    else {
                        panic!("wrong callback")
                    };
                    events.lock().unwrap().push(*quantum);
                })
            }),
        );
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        cfg.context.observer = Some(observer);
        let mut world = FilmWorld::default();
        world.bind_full(0, 0);
        let (end, spans, values) = if row["mode"] == 0 {
            let record = cfg.read_keyframe_record(&data, 0, &registry).unwrap();
            assert_eq!(record.stop, KeyframeStop::Complete);
            let spans: Vec<_> = record
                .components
                .iter()
                .map(|c| (c.index, c.start_bit))
                .collect();
            let values: Vec<_> = record
                .components
                .iter()
                .map(|c| record.fields[c.field_range.unwrap()[0]].raw)
                .collect();
            (record.end_bit, spans, values)
        } else {
            let mut reader = NativeFilmReader::new(&data);
            let view = reader
                .read_frame_records(&registry, &mut world, &cfg)
                .unwrap();
            assert_eq!(view.stop, EntityViewStop::Complete);
            let record = &view.records[0];
            let spans: Vec<_> = record
                .attempts
                .iter()
                .map(|a| (a.span.index, a.span.start_bit))
                .collect();
            let values: Vec<_> = record
                .attempts
                .iter()
                .map(|a| record.fields[a.field_start].raw)
                .collect();
            (reader.native_bit_position(), spans, values)
        };
        assert_eq!(serde_json::json!(end), row["end"], "end {i}");
        let expected: Vec<_> = row["components"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|c| {
                (
                    c["Index"].as_u64().unwrap() as usize,
                    c["StartBit"].as_i64().unwrap(),
                )
            })
            .collect();
        assert_eq!(spans, expected, "spans {i}");
        assert_eq!(serde_json::json!(values), row["events"], "fields {i}");
        assert_eq!(
            serde_json::json!(*events.lock().unwrap()),
            row["events"],
            "callbacks {i}"
        );
        if row["mode"] != 0 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let (trial, _) = super::components::chain_delta_body_trial_contextual(
                    &data,
                    9,
                    &registry,
                    0,
                    &cfg.contextual_frame_encoding().unwrap(),
                    false,
                    Some(&cfg.context),
                );
                let expected = row["trial_ok"].as_bool().unwrap().then(|| {
                    (
                        row["trial_end"].as_i64().unwrap(),
                        row["trial_count"].as_u64().unwrap() as usize,
                    )
                });
                assert_eq!(trial, expected, "trial {i}");
            }
            let chain = cfg.infer_chain(&data, 9, &registry, &world).unwrap();
            assert_eq!(
                chain.archetype,
                row["chain_ok"]
                    .as_bool()
                    .unwrap()
                    .then(|| row["chain_ti"].as_u64().unwrap() as u32),
                "chain type {i}"
            );
            assert_eq!(
                serde_json::json!(chain.end_bit),
                row["chain_end"],
                "chain end {i}"
            );
            assert_eq!(
                serde_json::json!(chain.unique_archetype),
                row["chain_unique"],
                "chain unique {i}"
            );
        }
    }
}

#[test]
fn native_component_result_contract() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/component-result-contract-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 10);
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![row["name"].as_str().unwrap().into()],
                levels: vec![0],
            }],
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        cfg.context.profile = native_scan_profile::tests::profile(&row["profile"]);
        let (end, results) = if row["full"] == true {
            let record = cfg.read_keyframe_record(&data, 0, &registry).unwrap();
            let restored: KeyframeRecord =
                serde_json::from_value(serde_json::to_value(&record).unwrap()).unwrap();
            assert_eq!(restored, record);
            assert_eq!(record.components.len(), usize::from(row["mode"] != 2));
            let results: Vec<_> = record.attempts.iter().map(|c| serde_json::json!({"Index": c.index, "Name":c.name, "StartBit":c.start_bit,"Variant":c.variant,"Ported":c.ported,"Payload":null})).collect();
            let mut old = serde_json::to_value(&record).unwrap();
            old.as_object_mut().unwrap().remove("attempts");
            for c in old["components"].as_array_mut().unwrap() {
                c.as_object_mut().unwrap().remove("variant");
                c.as_object_mut().unwrap().remove("ported");
            }
            let old: KeyframeRecord = serde_json::from_value(old).unwrap();
            assert!(old.attempts.is_empty());
            assert!(
                old.components
                    .iter()
                    .all(|c| c.variant.is_none() && c.ported.is_none())
            );
            (record.end_bit, results)
        } else {
            let mut reader = NativeFilmReader::new(&data);
            let mut world = FilmWorld::default();
            world.bind_full(0, 0);
            let view = reader
                .read_frame_records(&registry, &mut world, &cfg)
                .unwrap();
            let record = &view.records[0];
            let restored: EntityRecord =
                serde_json::from_value(serde_json::to_value(record).unwrap()).unwrap();
            assert_eq!(&restored, record);
            let results: Vec<_> = record.attempts.iter().map(|a| serde_json::json!({"Index":a.span.index,"Name":a.span.name,"StartBit":a.span.start_bit,"Variant":a.variant,"Ported":a.status,"Payload":null})).collect();
            (reader.native_bit_position(), results)
        };
        assert_eq!(
            serde_json::json!(results),
            row["trace"]["Comps"],
            "results {i}"
        );
        assert_eq!(serde_json::json!(end), row["end"], "end {i}");
    }
}

#[test]
fn native_dead_result_retention() {
    use std::sync::Arc;
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/dead-result-retention-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 32);
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let name = "object-dead-state-component";
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![name.into(), "biped-emp-timer-component".into(), name.into()],
                levels: vec![0; 3],
            }],
        };
        let widths = NativeSharedWidths::default();
        let observer = NativeFilmObserver::default();
        let skip = row["skip"].as_bool().unwrap();
        observer.set_hook(
            NativeHookKind::EmpTimer,
            Some({
                let widths = widths.clone();
                Arc::new(move |_| {
                    if skip {
                        widths.insert("object-dead-state-component".into(), 0);
                    }
                })
            }),
        );
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        cfg.context.profile.grammar.calibrated_widths = Some(widths);
        cfg.context.observer = Some(observer);
        if row["full"] == true {
            let record = cfg.read_keyframe_record(&data, 0, &registry).unwrap();
            assert_eq!(record.stop, KeyframeStop::Complete);
            assert_eq!(
                serde_json::to_value(record.captured_dead_state()).unwrap(),
                row["dead"],
                "keyframe dead {i}"
            );
            assert_eq!(
                serde_json::json!(record.end_bit),
                row["end"],
                "keyframe end {i}"
            );
            let restored: KeyframeRecord =
                serde_json::from_value(serde_json::to_value(&record).unwrap()).unwrap();
            assert_eq!(restored.captured_dead_state(), record.captured_dead_state());
            continue;
        }
        let mut world = FilmWorld::default();
        world.bind_full(0, 0);
        let mut reader = NativeFilmReader::new(&data);
        let view = reader
            .read_frame_records(&registry, &mut world, &cfg)
            .unwrap();
        assert_eq!(view.stop, EntityViewStop::Complete);
        assert_eq!(
            serde_json::json!(reader.native_bit_position()),
            row["end"],
            "end {i}"
        );
        let record = &view.records[0];
        assert_eq!(
            serde_json::to_value(record.captured_dead_state()).unwrap(),
            row["dead"],
            "generic dead {i}"
        );
        let mut facts = MarchRecordFacts::default();
        facts.harvest(std::slice::from_ref(record), 1000, &registry);
        assert_eq!(
            serde_json::to_value(&facts.deaths).unwrap(),
            row["deaths"],
            "deaths {i}"
        );
        let restored: EntityRecord =
            serde_json::from_value(serde_json::to_value(record).unwrap()).unwrap();
        let mut restored_facts = MarchRecordFacts::default();
        restored_facts.harvest(&[restored], 1000, &registry);
        assert_eq!(restored_facts, facts, "portable harvest {i}");
    }
}

#[test]
fn native_parent_result_selection() {
    use std::sync::Arc;
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/parent-result-selection-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 16);
    for (i, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let name = "object-parent-state-component";
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: (0..36)
                .map(|index| FilmArchetype {
                    index,
                    components: if index == 35 {
                        vec![name.into(), "biped-emp-timer-component".into(), name.into()]
                    } else {
                        vec![]
                    },
                    levels: vec![0; 3],
                })
                .collect(),
        };
        let widths = NativeSharedWidths::default();
        if row["skip_first"] == true {
            widths.insert(name.into(), 0);
        }
        let observer = NativeFilmObserver::default();
        let skip = row["skip_second"].as_bool().unwrap();
        observer.set_hook(
            NativeHookKind::EmpTimer,
            Some({
                let widths = widths.clone();
                Arc::new(move |_| {
                    if skip {
                        widths.insert("object-parent-state-component".into(), 0);
                    } else {
                        widths.remove("object-parent-state-component");
                    }
                })
            }),
        );
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        cfg.context.profile.grammar.calibrated_widths = Some(widths);
        cfg.context.observer = Some(observer);
        let mut world = FilmWorld::default();
        world.bind_full(0, 35);
        let mut reader = NativeFilmReader::new(&data);
        let view = reader
            .read_frame_records(&registry, &mut world, &cfg)
            .unwrap();
        assert_eq!(view.stop, EntityViewStop::Complete);
        assert_eq!(
            serde_json::json!(reader.native_bit_position()),
            row["end"],
            "end {i}"
        );
        let record = &view.records[0];
        let mut facts = MarchRecordFacts::default();
        facts.harvest(std::slice::from_ref(record), 1000, &registry);
        assert_eq!(
            serde_json::to_value(&facts.occupancy).unwrap(),
            row["occupancy"],
            "occupancy {i}"
        );
        let restored: EntityRecord =
            serde_json::from_value(serde_json::to_value(record).unwrap()).unwrap();
        assert_eq!(
            vehicle_occupancy_from_record(&restored, 1000),
            facts.occupancy.first().cloned(),
            "portable occupancy {i}"
        );
    }
}

#[test]
fn native_keyframe_frame_bits() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/keyframe-frame-bits-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 81);
    for row in rows {
        let layout = NativeKeyframeLayout {
            header_bits: row["header"].as_i64().unwrap(),
            size_word_bits: row["word"].as_i64().unwrap(),
        };
        let expected = row["frame_bits"].as_i64().unwrap();
        assert_eq!(layout.frame_bits(), expected, "{row}");
        let restored: NativeKeyframeLayout =
            serde_json::from_value(serde_json::to_value(layout).unwrap()).unwrap();
        assert_eq!(restored.frame_bits(), expected, "{row}");
    }
    // Only fixed framing overhead: component and default-state bodies are excluded.
    assert_eq!(
        NativeKeyframeLayout {
            header_bits: 108,
            size_word_bits: 32
        }
        .frame_bits(),
        172
    );
}
