//! Snapshot-only capture audit, using the production Film configuration and
//! original packet envelopes. Frame packets are excluded to avoid replay cost.
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::film::*;
use crate::theater::parser::packet::*;
use crate::theater::parser::test_support::empty_summary;
use crate::theater::parser::v41::chunks::replication::source::*;
use serde_json::json;
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

#[test]
#[ignore = "requires retained 32-film corpus; optional HALO_KEYFRAME_AUDIT JSON export"]
fn captured_native_snapshot_stops() {
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films"));
    let mut paths = Vec::new();
    manifests(&root, &mut paths);
    paths.sort();
    assert_eq!(paths.len(), 32);
    let mut reports = Vec::new();
    let mut stops = BTreeMap::<String, usize>::new();
    for path in paths {
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let mut inputs = Vec::new();
        let mut sources = Vec::new();
        for c in manifest["chunks"].as_array().unwrap() {
            let kind = crate::clients::hi::models::RawChunkKind(
                i32::try_from(c["chunk_type"].as_i64().unwrap()).unwrap(),
            );
            if kind == ChunkKind::Summary.into() {
                continue;
            }
            let file = path.parent().unwrap().join(c["file"].as_str().unwrap());
            let mut chunk = RawFilmChunk {
                kind,
                index: i32::try_from(c["index"].as_i64().expect("film chunk index"))
                    .expect("film chunk index fits i32"),
                start_ms: c["start_time_offset_ms"]
                    .as_i64()
                    .expect("film chunk start time"),
                data: std::fs::read(&file).unwrap(),
            };
            if kind == ChunkKind::Replication.into() {
                let inflated = chunk.clone().inflate().unwrap();
                let stream = inflated
                    .read_packets::<ReplicationStreamPacketBody>(65536, |_, h, _| {
                        PacketRead::Opaque {
                            reason: PacketDecodeError::UnsupportedLayout {
                                packet_type: h.packet_type,
                            },
                        }
                    })
                    .unwrap();
                assert!(stream.opaque.is_empty());
                chunk.data.clear();
                for packet in stream.packets.iter().filter(|p| p.header.packet_type == 2) {
                    chunk.data.extend_from_slice(
                        &inflated.data[packet.source.header.start..packet.source.payload.end],
                    );
                    sources.push((
                        file.strip_prefix(&root).unwrap().display().to_string(),
                        packet.source.payload.start,
                    ));
                }
            }
            inputs.push(chunk);
        }
        inputs.push(empty_summary());
        let film = crate::theater::parser::test_support::parse_fixture_structural(inputs).unwrap();
        let mut sources = sources.into_iter();
        for chunk in &film.replication_stream.0 {
            for packet in &chunk.body.packets {
                let (file, payload_byte) = sources.next().unwrap();
                let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
                    &packet.body
                else {
                    panic!()
                };
                assert!(packet.source_coverage(&chunk.data).is_some());
                let record = table.records.last().and_then(|r| r.record.as_ref());
                let reason = if table.stop == KeyframeChainStop::End {
                    "End".into()
                } else {
                    format!("{:?}/{:?}", table.stop, record.map(|r| &r.stop))
                };
                *stops.entry(reason).or_default() += 1;
                reports.push(json!({"file":file,"payload_byte":payload_byte,
                    "table_stop":table.stop,"headers":table.slots.as_ref().unwrap().headers.len(),
                    "end":table.slots.as_ref().unwrap().end_bit,"records":table.records.len(),
                    "last_record":record}));
            }
        }
        assert!(sources.next().is_none());
    }
    assert_eq!(reports.len(), 451);
    eprintln!("snapshot stop counts: {stops:#?}");
    if let Some(path) = std::env::var_os("HALO_KEYFRAME_AUDIT") {
        std::fs::write(path, serde_json::to_vec(&reports).unwrap()).unwrap();
    }
}
