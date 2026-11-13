use super::*;
use crate::theater::parser::packet::{PacketBody, SourceRegionKind};
use crate::theater::parser::v41::chunks::replication::source::ReplicationStreamPacketBody;
use crate::theater::parser::v41::context::players::personalization_bytes;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../../../../../../fixtures/roster-v41/reference.json"
    ))
    .unwrap()
}
fn data(case: &Value) -> Vec<u8> {
    case["Data"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect()
}
fn context(c: &Value) -> PlayerContext {
    let build = c["Build"].as_str().unwrap();
    PlayerContext {
        format_version: Some(c["Version"].as_u64().unwrap() as u32),
        build: Some(build.to_owned()),
        personalization_bytes: personalization_bytes(build),
    }
}
fn word(e: &PlayerMetadataEntry, kind: K) -> u64 {
    e.fields
        .iter()
        .find(|f| f.kind == kind)
        .unwrap()
        .raw
        .as_ref()
        .unwrap()
        .low_u64()
}
fn name(e: &PlayerMetadataEntry) -> String {
    let units: Vec<_> = e
        .fields
        .iter()
        .filter(|f| matches!(f.kind, K::GamertagUnit(_)))
        .map(|f| f.raw.as_ref().unwrap().low_u64() as u16)
        .take_while(|u| *u != 0)
        .collect();
    // This comparison alone uses upstream's lossy display semantics. The Film
    // retains every original unit, including an unpaired surrogate and NUL.
    String::from_utf16_lossy(&units)
}
fn exact(data: &[u8], read: &PlayerMetadataRead) {
    let mut end = 0;
    for f in std::iter::once(&read.count).chain(read.entries.iter().flat_map(|e| &e.fields)) {
        assert_eq!(f.bit, end);
        assert_eq!(f.raw, RawBits::from_source(data, f.bit, f.width));
        if f.raw.is_some() {
            end += f.width;
        }
    }
    assert_eq!(read.end_bit, end);
    assert_eq!(
        read.tail,
        BitRange {
            start: end,
            end: data.len() * 8
        }
    );
    assert!(
        read.entries
            .windows(2)
            .all(|e| e[0].end_bit == e[1].start_bit && e[0].stop == Stop::Complete)
    );
}

#[test]
fn original_player_readers_match_all_builds_gates_values_and_closure() {
    let cases = cases();
    assert_eq!(cases.len(), 30);
    for c in cases {
        let bytes = data(&c);
        let ctx = context(&c);
        assert_eq!(
            ctx.personalization_bytes,
            Some(c["Perso"].as_u64().unwrap() as usize)
        );
        let bots = c["Bots"].as_bool().unwrap();
        let read = decode(&bytes, bots, &ctx);
        exact(&bytes, &read);
        let complete: Vec<_> = read
            .entries
            .iter()
            .filter(|e| e.stop == Stop::Complete)
            .collect();
        if bots {
            assert_eq!(
                read.stop == Stop::Complete,
                c["Closed"].as_bool().unwrap(),
                "{}",
                c["Name"]
            );
            if let Some(rows) = c["BotsRead"].as_array() {
                assert_eq!(complete.len(), rows.len());
                for (e, row) in complete.iter().zip(rows) {
                    assert_eq!(word(e, K::Slot), row["Slot"].as_u64().unwrap());
                    assert_eq!(word(e, K::BotId), row["BotID"].as_u64().unwrap());
                    assert_eq!(name(e), row["Nom"].as_str().unwrap());
                    // Raw team bytes remain opaque here; source meanings vary by build.
                    if row["EquipeLue"] == true {
                        let (a, b) = if ctx.build.as_deref() == Some("HI_1_13_0") {
                            (37, 38)
                        } else {
                            (4, 6)
                        };
                        assert_eq!(
                            word(e, K::Block44Byte(a)) as u8 as i8 as i64,
                            row["Equipe"].as_i64().unwrap()
                        );
                        assert_eq!(
                            word(e, K::Block44Byte(b)) as u8 as i8 as i64,
                            row["Jumeau"].as_i64().unwrap()
                        );
                    }
                }
            }
        } else {
            let report = &c["Report"];
            assert_eq!(complete.len(), report["Entrees"].as_u64().unwrap() as usize);
            assert_eq!(
                read.stop == Stop::Complete,
                report["Refusees"] == 0 && report["Debordement"] == false
            );
            if read.stop == Stop::Complete {
                assert_eq!(read.end_bit, report["BitsLus"].as_u64().unwrap() as usize);
            }
            for (e, row) in complete
                .iter()
                .zip(c["Roster"].as_array().into_iter().flatten())
            {
                assert_eq!(
                    word(e, K::Identity).swap_bytes(),
                    row["Identite"].as_u64().unwrap()
                );
                let option = e
                    .fields
                    .iter()
                    .find(|f| f.kind == K::Option)
                    .map(|f| f.raw.as_ref().unwrap().low_u64() as i64)
                    .unwrap_or(-1);
                assert_eq!(option, row["Option"].as_i64().unwrap());
                let slot = &row["Joueur"];
                assert_eq!(e.start_bit, slot["Bit"].as_u64().unwrap() as usize);
                assert_eq!(
                    e.end_bit - e.start_bit,
                    slot["TotalBits"].as_u64().unwrap() as usize
                );
                assert_eq!(name(e), slot["Gamertag"].as_str().unwrap());
                let shorts = &slot["Shorts"];
                assert_eq!(word(e, K::Representation), shorts["Repr"].as_u64().unwrap());
                assert_eq!(word(e, K::JoinKey), shorts["Q64"].as_u64().unwrap());
                for (i, key) in ["F10", "F14", "F6", "F8", "F7", "F1"]
                    .into_iter()
                    .enumerate()
                {
                    assert_eq!(
                        word(e, K::Short(i as u8)) as i64 - i64::from(i == 2),
                        shorts[key].as_i64().unwrap()
                    );
                }
            }
        }
    }
}

#[test]
fn metadata_truncation_preserves_prefix_without_successor_scans() {
    for c in cases().into_iter().take(2) {
        let bytes = data(&c);
        let ctx = context(&c);
        let bots = c["Bots"].as_bool().unwrap();
        let complete = decode(&bytes, bots, &ctx);
        for len in 0..complete.end_bit.div_ceil(8) {
            let read = decode(&bytes[..len], bots, &ctx);
            assert_eq!(read.stop, Stop::Truncated, "{} length {len}", c["Name"]);
            exact(&bytes[..len], &read);
            assert!(read.entries.len() <= 2);
            assert!(
                read.entries
                    .iter()
                    .filter(|e| e.stop != Stop::Complete)
                    .count()
                    <= 1
            );
            for (entry, full) in read.entries.iter().zip(&complete.entries) {
                for (field, expected) in entry.fields.iter().zip(&full.fields) {
                    assert_eq!(
                        (field.kind, field.bit, field.width),
                        (expected.kind, expected.bit, expected.width)
                    );
                    if field.raw.is_some() {
                        assert_eq!(field, expected);
                    }
                }
            }
        }
    }
}

#[test]
fn unknown_context_limits_and_tail_coverage_are_explicit() {
    let c = &cases()[0];
    let bytes = data(c);
    let mut ctx = context(c);
    ctx.format_version = Some(28);
    let read = decode(&bytes, false, &ctx);
    assert_eq!(read.stop, Stop::UnsupportedFormat);
    assert_eq!(read.end_bit, 32);
    ctx.format_version = Some(27);
    ctx.personalization_bytes = None;
    let read = decode(&bytes, false, &ctx);
    assert_eq!(read.stop, Stop::UnknownPersonalization);
    assert_eq!(read.entries[0].fields.last().unwrap().kind, K::Short(5));
    assert!(personalization_bytes("HI_1_14_0").is_none());
    for c in cases() {
        let read = decode(&data(&c), c["Bots"].as_bool().unwrap(), &context(&c));
        let name = c["Name"].as_str().unwrap();
        if name.starts_with("byte-limit") {
            assert!(matches!(
                read.stop,
                Stop::CountLimit {
                    kind: K::ByteCount,
                    count: 2049,
                    limit: 2048
                }
            ));
        }
        if name.starts_with("word-limit") {
            assert!(matches!(
                read.stop,
                Stop::CountLimit {
                    kind: K::WordCount,
                    count: 193,
                    limit: 192
                }
            ));
        }
        if name == "count-limit-True" {
            assert!(matches!(
                read.stop,
                Stop::CountLimit {
                    kind: K::Count,
                    count: 65,
                    limit: 64
                }
            ));
        }
        let tail = read.tail;
        let bots = c["Bots"].as_bool().unwrap();
        let stop = read.stop;
        let body = if bots {
            ReplicationStreamPacketBody::BotsPacketBody(Box::new(read))
        } else {
            ReplicationStreamPacketBody::RosterPacketBody(Box::new(read))
        };
        let regions = body.source_regions();
        assert_eq!(
            regions.iter().any(|r| r.kind == SourceRegionKind::Padding),
            bots && stop == Stop::Complete
        );
        assert!(
            regions
                .iter()
                .filter(|r| r.kind != SourceRegionKind::Padding)
                .all(|r| r.source.end <= tail.start)
        );
    }
}

#[test]
#[ignore = "requires retained 32-film corpus; writes metadata oracle inputs when HALO_ROSTER_EXPORT is set"]
fn captured_player_metadata_preserves_fields_and_packet_boundaries() {
    use crate::clients::hi::models::RawFilmChunk;
    use crate::theater::parser::packet::PacketRead;
    use std::path::{Path, PathBuf};
    fn visit(p: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(p).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                visit(&p, out);
            } else if p.file_name().unwrap() == "film.json" {
                out.push(p);
            }
        }
    }
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("experiments/films"));
    let mut paths = Vec::new();
    visit(&root, &mut paths);
    paths.sort();
    assert_eq!(paths.len(), 32);
    let mut outputs = Vec::new();
    let mut counts = [0usize; 4];
    for path in paths {
        let manifest: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let film = crate::theater::parser::test_support::parse_fixture_structural(
            manifest["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| RawFilmChunk {
                    kind: crate::clients::hi::models::RawChunkKind(
                        i32::try_from(c["chunk_type"].as_i64().unwrap()).unwrap(),
                    ),
                    data: std::fs::read(path.parent().unwrap().join(c["file"].as_str().unwrap()))
                        .unwrap(),
                    index: i32::try_from(c["index"].as_i64().expect("film chunk index"))
                        .expect("film chunk index fits i32"),
                    start_ms: c["start_time_offset_ms"]
                        .as_i64()
                        .expect("film chunk start time"),
                }),
        )
        .unwrap();
        for chunk in &film.replication_stream.0 {
            for (i, packet) in chunk.body.packets.iter().enumerate() {
                let (read, bots) = match &packet.body {
                    PacketRead::Decoded(ReplicationStreamPacketBody::RosterPacketBody(r)) => {
                        (r, false)
                    }
                    PacketRead::Decoded(ReplicationStreamPacketBody::BotsPacketBody(r)) => {
                        (r, true)
                    }
                    _ => continue,
                };
                let bytes = packet.payload(&chunk.data).unwrap();
                exact(bytes, read.as_ref());
                assert!(packet.source_coverage(&chunk.data).is_some());
                counts[usize::from(bots) * 2 + usize::from(read.stop != Stop::Complete)] += 1;
                let name = format!(
                    "{}:{}:{i}",
                    path.strip_prefix(&root).unwrap().display(),
                    (chunk.index)
                );
                // The independent exporter is run separately so the normal
                // corpus check needs neither Go nor network access.
                if std::env::var_os("HALO_ROSTER_EXPORT").is_some() {
                    outputs.push(serde_json::json!({"Name":name,"Build":read.build,"Version":read.format_version,
                        "Bots":bots,"Data":bytes.iter().map(|b|format!("{b:02x}")).collect::<String>(),"Actual":read}));
                }
            }
        }
    }
    assert!(counts[0] > 0 && counts[2] > 0, "{counts:?}");
    if let Some(path) = std::env::var_os("HALO_ROSTER_EXPORT") {
        std::fs::write(path, serde_json::to_vec(&outputs).unwrap()).unwrap();
    }
    println!(
        "32 films: roster complete/partial {},{}; bots complete/partial {},{}",
        counts[0], counts[1], counts[2], counts[3]
    );
}
