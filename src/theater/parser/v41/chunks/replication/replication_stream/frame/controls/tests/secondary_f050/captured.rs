//! Export source-backed control slices for an external, two-revision oracle.
use super::*;
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::parser::packet::PacketRead;
use crate::theater::parser::v41::chunks::replication::source::ReplicationStreamPacketBody;

#[test]
fn captured_secondary_controls_match_corrected_oracle_and_preserve_partial_stops() {
    #[derive(Deserialize)]
    struct Capture {
        name: String,
        hex: String,
        bit_len: usize,
        configuration: bool,
        canonical_end: i64,
        canonical_entries: usize,
        canonical_stop: FrameViewStop,
        widths: [u64; 3],
        index_bits: u64,
        boxes: [[u64; 3]; 4],
    }
    #[derive(Deserialize)]
    struct Oracle {
        commit: String,
        cases: Vec<Trace>,
    }
    #[derive(Deserialize)]
    struct Trace {
        name: String,
        end: Option<i64>,
        trace: Vec<(usize, usize)>,
    }
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/theater/fixtures/controls-f050371-v41");
    let cases: Vec<Capture> =
        serde_json::from_slice(&std::fs::read(fixture.join("captured-cases.json")).unwrap())
            .unwrap();
    let oracle = |file: &str| -> Oracle {
        serde_json::from_slice(&std::fs::read(fixture.join(file)).unwrap()).unwrap()
    };
    let current = oracle("captured-reference.json");
    let original = oracle("captured-original-reference.json");
    assert_eq!(current.commit, "f050371a6cda83b9887eeb83a1884336fce28e53");
    assert_eq!(original.commit, "18067987ebc3628ad499c819c6aee814692df43b");
    assert_eq!(cases.len(), 72);
    assert_eq!(current.cases.len(), cases.len());
    assert_eq!(original.cases.len(), cases.len());
    let (mut complete, mut changed, mut unsupported_count, mut truncated) = (0, 0, 0, 0);
    let (mut full, mut local, mut target) = (0, 0, 0);
    for ((case, expected), old) in cases.iter().zip(&current.cases).zip(&original.cases) {
        assert_eq!(case.name, expected.name);
        assert_eq!(case.name, old.name);
        assert!(case.configuration);
        assert_eq!(case.boxes, [[15, 15, 17], [18, 18, 18], [0; 3], [0; 3]]);
        let (hex_bytes, remainder) = case.hex.as_bytes().as_chunks::<2>();
        assert!(remainder.is_empty());
        let raw: Vec<_> = hex_bytes
            .iter()
            .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect();
        // Put the original logical end on the physical byte boundary. Padding
        // from slicing an unaligned packet must not hide source truncation.
        let shift = (8 - case.bit_len % 8) % 8;
        let data = shifted(&raw, case.bit_len, shift);
        assert_eq!(data.len() * 8, case.bit_len + shift);
        let mut profile = DecodeProfile::default();
        let world = &mut profile.movement.world_object;
        world.axis_bits = case.widths;
        world.index_bits = case.index_bits;
        world.regions = [(0, case.boxes[0]), (1, case.boxes[1])].into();
        let encoding = profile.component_encoding().unwrap();
        let view = decode_control_view_contextual(
            &data,
            shift as i64,
            Some(&encoding),
            Some(&profile),
            case.configuration,
        );
        assert_eq!(
            view.end_bit - shift as i64,
            case.canonical_end,
            "{}",
            case.name
        );
        assert_eq!(view.stop, case.canonical_stop, "{}", case.name);
        assert_eq!(
            view.control_entries.len(),
            case.canonical_entries,
            "{}",
            case.name
        );
        let mut boundaries = BTreeSet::from([shift]);
        for field in &view.fields {
            assert_eq!(
                field.raw,
                RawBits::from_source(&data, field.bit, field.width).unwrap()
            );
            boundaries.insert(field.bit);
            boundaries.insert(field.bit + field.width);
        }
        for &(at, width) in &expected.trace {
            if at + width <= case.canonical_end as usize {
                assert!(boundaries.contains(&(at + shift)), "{}: {at}", case.name);
                assert!(
                    boundaries.contains(&(at + width + shift)),
                    "{}: {}",
                    case.name,
                    at + width
                );
            }
        }
        match &view.stop {
            FrameViewStop::Complete => {
                complete += 1;
                assert_eq!(expected.end, Some(case.canonical_end), "{}", case.name);
                assert!(case.canonical_end as usize <= case.bit_len);
                changed += usize::from(old.end != expected.end);
            }
            FrameViewStop::Unsupported { reason } => {
                assert_eq!(reason, "control handler kind 3");
                unsupported_count += 1;
            }
            FrameViewStop::Truncated => truncated += 1,
            other => panic!("unexpected captured stop: {other:?}"),
        }
        for entry in &view.control_entries {
            let ControlPayload::Input(input) = &entry.payload else {
                continue;
            };
            let Some(range) = input.secondary else {
                continue;
            };
            let fields: Vec<_> = view
                .fields
                .iter()
                .filter(|f| range.start <= f.bit && f.bit + f.width <= range.end)
                .collect();
            let set = |name| {
                fields
                    .iter()
                    .any(|f| f.name == name && f.raw.low_u64() != 0)
            };
            full += usize::from(
                fields
                    .iter()
                    .any(|f| f.name == "control.secondary.state" && f.raw.low_u64() & 2 != 0),
            );
            local += usize::from(set("control.secondary.local_position"));
            target += usize::from(set("control.secondary.target.present"));
        }
    }
    assert_eq!(
        (complete, changed, unsupported_count, truncated),
        (47, 46, 19, 6)
    );
    assert_eq!((full, local, target), (70, 28, 27));
}

#[test]
#[ignore = "requires the retained 32-film corpus and HALO_CONTROL_CAPTURE_EXPORT destination"]
fn captured_full_secondary_controls_for_upstream_comparison() {
    fn manifests(path: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                manifests(&p, out);
            } else if p.file_name().unwrap() == "film.json" {
                out.push(p);
            }
        }
    }
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films")
        });
    let destination =
        std::env::var_os("HALO_CONTROL_CAPTURE_EXPORT").expect("explicit export destination");
    let mut paths = vec![];
    manifests(&root, &mut paths);
    paths.sort();
    assert_eq!(paths.len(), 32);
    let mut rows = vec![];
    for path in paths {
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let chunks = manifest["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| RawFilmChunk {
                kind: crate::clients::hi::models::RawChunkKind(
                    i32::try_from(c["chunk_type"].as_i64().unwrap()).unwrap(),
                ),
                index: i32::try_from(c["index"].as_i64().expect("film chunk index"))
                    .expect("film chunk index fits i32"),
                start_ms: c["start_time_offset_ms"]
                    .as_i64()
                    .expect("film chunk start time"),
                data: std::fs::read(path.parent().unwrap().join(c["file"].as_str().unwrap()))
                    .unwrap(),
            });
        let film = crate::theater::parser::test_support::parse_fixture_structural(chunks).unwrap();
        let mut profile = DecodeProfile::default();
        use crate::theater::parser::v41::context::map::{level_precision, recorded_level};
        if let Some(level) = recorded_level(&film.registry.body) {
            profile.movement.world_object = level_precision(level, 16).unwrap();
        }
        let world = &profile.movement.world_object;
        let mut boxes = [[0u32; 3]; 4];
        for (&index, widths) in &world.regions {
            boxes[usize::try_from(index).unwrap()] = widths.map(|v| u32::try_from(v).unwrap());
        }
        for chunk in &film.replication_stream.0 {
            for (packet_index, packet) in chunk.body.packets.iter().enumerate() {
                let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) =
                    &packet.body
                else {
                    continue;
                };
                let Some(view) = frame.frame.decoded().and_then(|f| f.controls.as_ref()) else {
                    continue;
                };
                if !view
                    .fields
                    .iter()
                    .any(|f| f.name == "control.secondary.state" && f.raw.low_u64() & 2 != 0)
                {
                    continue;
                }
                let payload = packet.payload(&chunk.data).unwrap();
                let start = usize::try_from(view.start_bit).unwrap();
                let bit_len = payload.len() * 8 - start;
                let mut data = vec![0; bit_len.div_ceil(8)];
                for i in 0..bit_len {
                    data[i / 8] |=
                        ((payload[(start + i) / 8] >> (7 - (start + i) % 8)) & 1) << (7 - i % 8);
                }
                rows.push(serde_json::json!({
                    "name": format!("{}:{}:{packet_index}:{start}", path.strip_prefix(&root).unwrap().display(), (chunk.index)),
                    "hex": data.iter().map(|b| format!("{b:02x}")).collect::<String>(),
                    "bit_len": bit_len, "configuration": frame.configuration,
                    "canonical_end": view.end_bit - view.start_bit, "canonical_stop": view.stop,
                    "canonical_entries": view.control_entries.len(),
                    "widths": world.axis_bits, "index_bits": world.index_bits, "boxes": boxes,
                }));
            }
        }
        eprintln!("extracted {}", path.strip_prefix(&root).unwrap().display());
    }
    assert!(!rows.is_empty());
    std::fs::write(destination, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    eprintln!("exported {} full-secondary candidate views", rows.len());
}
