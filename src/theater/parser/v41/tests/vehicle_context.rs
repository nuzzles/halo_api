//! Research audit of recorded mask evidence for the unrecorded vehicle gate.
//! This does not enable a layout, assume a false gate, or use later evidence.
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::parser::packet::*;
use crate::theater::parser::v41::chunks::replication::source::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn manifests(path: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            manifests(&path, out);
        } else if path.file_name().unwrap() == "film.json" {
            out.push(path);
        }
    }
}
fn tag(state: Option<&DefaultState>) -> Option<u32> {
    let state = state.filter(|s| s.status == DefaultStateStatus::Complete)?;
    let mut fields = state.fields.iter().filter(|f| f.name == "mpp.object_tag");
    let field = fields.next()?;
    (fields.next().is_none()
        && field.width == 32
        && field.raw.bit_len == 32
        && field.raw.bytes.len() == 4)
        .then(|| field.raw.low_u64() as u32)
}
fn vehicle_row(name: &str) -> bool {
    matches!(
        name,
        "vehicle-type-state-component" | "vehicle-type-physics-component"
    )
}

#[test]
#[ignore = "requires the retained 32-film corpus; audits rather than supplies vehicle context"]
fn captured_vehicle_masks_audit_prior_asset_gate_evidence() {
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("experiments/films"));
    let mut paths = Vec::new();
    manifests(&root, &mut paths);
    paths.sort();
    assert_eq!(paths.len(), 32);
    if let Ok(name) = std::env::var("HALO_VEHICLE_FILM") {
        paths.retain(|p| p.parent().unwrap().strip_prefix(&root).unwrap() == Path::new(&name));
        assert_eq!(paths.len(), 1);
    }
    let mut report = Vec::new();
    for path in paths {
        let manifest: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let files: Vec<_> = manifest["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| path.parent().unwrap().join(c["file"].as_str().unwrap()))
            .collect();
        let film = crate::theater::parser::test_support::parse_fixture_structural(
            manifest["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .zip(&files)
                .map(|(c, p)| RawFilmChunk {
                    kind: crate::clients::hi::models::RawChunkKind(
                        i32::try_from(c["chunk_type"].as_i64().unwrap()).unwrap(),
                    ),
                    index: i32::try_from(c["index"].as_i64().expect("film chunk index"))
                        .expect("film chunk index fits i32"),
                    start_ms: c["start_time_offset_ms"]
                        .as_i64()
                        .expect("film chunk start time"),
                    data: std::fs::read(p).unwrap(),
                }),
        )
        .unwrap();
        assert_eq!(film.registry.body.header, Some([41, 27]));
        let mut known = BTreeMap::<u32, Value>::new();
        let mut creations = Vec::new();
        let mut masked = Vec::new();
        let mut stops = Vec::new();
        let mut tables = 0;
        for chunk in &film.replication_stream.0 {
            let file = files[chunk.index]
                .strip_prefix(&root)
                .unwrap()
                .to_str()
                .unwrap();
            for (packet_index, packet) in chunk.body.packets.iter().enumerate() {
                let source = |start: i64, end: i64| json!({"file":file,"packet":packet_index,"timestamp_us":packet.header.timestamp_us,"start_bit":start,"end_bit":end});
                match &packet.body {
                    PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(body)) => {
                        let Some(frame) = body.frame.decoded() else {
                            continue;
                        };
                        for r in &frame.records {
                            let rows: Vec<_> = r
                                .components
                                .iter()
                                .filter(|c| vehicle_row(&c.name))
                                .collect();
                            let direct_tag = if r.header.kind == RecordKind::New
                                && r.archetype == Some(40)
                                && r.stop == EntityViewStop::Complete
                            {
                                tag(r.default_state.as_ref())
                            } else {
                                None
                            };
                            if r.header.kind == RecordKind::New && r.archetype == Some(40) {
                                creations.push(json!({"source":source(r.header.start_bit,r.end_bit),"id":r.header.id,"tag":direct_tag,"mask":r.mask,"complete":r.stop==EntityViewStop::Complete}));
                            }
                            if rows.is_empty() {
                                continue;
                            }
                            let registry = film.registry.body.registry.archetype(40).unwrap();
                            let proven = rows.iter().all(|c| {
                                c.status == ComponentReadStatus::Complete
                                    && c.index < 64
                                    && r.mask.is_some_and(|mask| mask & (1 << c.index) != 0)
                                    && registry
                                        .components
                                        .get(c.index)
                                        .is_some_and(|d| d.name() == Some(c.name.as_str()))
                            });
                            let evidence = json!({"source":source(r.header.start_bit,r.end_bit),"kind":r.header.kind,"id":r.header.id,"tag_from_same_new":direct_tag,"mask":r.mask,"earlier_same_id_new_candidates":creations.iter().filter(|c| c["id"] == json!(r.header.id) && c["complete"] == true && r.header.kind == RecordKind::Delta).collect::<Vec<_>>(),"rows":rows.iter().map(|c|json!({"index":c.index,"name":c.name,"parameter":registry.components[c.index].precision_level,"start_bit":c.start_bit,"end_bit":c.end_bit,"status":c.status})).collect::<Vec<_>>(),"complete_masked_rows":proven});
                            if proven && let Some(tag) = direct_tag {
                                known.entry(tag).or_insert_with(|| evidence.clone());
                            }
                            masked.push(evidence);
                        }
                    }
                    PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(
                        table,
                    )) => {
                        tables += 1;
                        for a in &table.records {
                            let Some(r) = &a.record else { continue };
                            if r.archetype != 40 {
                                continue;
                            }
                            let is_stop = match &r.stop {
                                KeyframeStop::RuntimeContextUnavailable { name, .. }
                                | KeyframeStop::UnsupportedComponent { name, .. } => {
                                    vehicle_row(name)
                                }
                                _ => false,
                            };
                            if !is_stop {
                                continue;
                            }
                            let tag = tag(r.default_state.as_ref());
                            stops.push(json!({"source":source(r.start_bit,r.end_bit),"tag":tag,"stop":r.stop,"prior_direct_tag_evidence":tag.and_then(|t|known.get(&t))}));
                        }
                    }
                    _ => {}
                }
            }
        }
        let eligible = stops
            .iter()
            .filter(|s| !s["prior_direct_tag_evidence"].is_null())
            .count();
        let name = path
            .parent()
            .unwrap()
            .strip_prefix(&root)
            .unwrap()
            .to_str()
            .unwrap();
        eprintln!(
            "Vehicle context {name}: {tables} snapshots, {} stops, {} NEWs, {} masked records, {} directly evidenced tags, {eligible} earlier-tag matches",
            stops.len(),
            creations.len(),
            masked.len(),
            known.len()
        );
        report.push(json!({"film":name,"tables":tables,"creations":creations,"masked_records":masked,"direct_tags":known,"stops":stops,"earlier_tag_matches":eligible}));
    }
    if let Some(path) = std::env::var_os("HALO_VEHICLE_CONTEXT_AUDIT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
}
