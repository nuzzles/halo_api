//! Native filesystem research packet reconciliation. These searches intentionally
//! use CountFilmChunks and tolerate unreadable files, not loaded-film prefix rules.
use super::FilmPacket;
use std::{path::Path, sync::Arc};

pub type FilmPacketPredicate<'a> = dyn Fn(&[u8]) -> bool + 'a;
/// Exact payload equality, including empty payloads.
pub fn film_packet_equal(want: &[u8]) -> impl Fn(&[u8]) -> bool + '_ {
    move |payload| payload == want
}
/// Native prefix predicate deliberately rejects an empty requested prefix.
pub fn film_packet_prefix(want: &[u8]) -> impl Fn(&[u8]) -> bool + '_ {
    move |payload| !want.is_empty() && payload.starts_with(want)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredFilmPacket {
    pub film: String,
    pub chunk: usize,
    pub packet_index: usize,
    pub packet: FilmPacket,
}
/// A packet plus the full decompressed chunk. Multiple packets share the bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmPacketWithChunk {
    pub chunk: usize,
    pub packet_index: usize,
    pub packet: FilmPacket,
    pub data: Arc<[u8]>,
}
#[derive(Debug, thiserror::Error)]
pub enum FilmPacketDiscoveryError {
    #[error("no film chunks in {0}")]
    NoChunks(std::path::PathBuf),
    #[error("no packet of type {kind} in {directory}")]
    NoPacket {
        directory: std::path::PathBuf,
        kind: u16,
    },
}

/// Scan immediate film subdirectories once for all predicates, in lexical film,
/// numeric chunk and packet order. Includes chunk zero even with no data chunks.
/// Root listing errors propagate; unreadable chunks are skipped.
pub fn find_film_packets(
    root: impl AsRef<Path>,
    predicates: &[&FilmPacketPredicate<'_>],
) -> Result<Vec<Vec<DiscoveredFilmPacket>>, std::io::Error> {
    let mut entries = std::fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    let mut out = vec![Vec::new(); predicates.len()];
    for entry in entries {
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let dir = entry.path();
        for chunk in 0..=super::count_film_chunks(&dir) {
            let Ok(data) = super::read_film_chunk(&dir, chunk as i64) else {
                continue;
            };
            for (packet_index, packet) in super::walk_film_packets(&data).into_iter().enumerate() {
                let payload =
                    &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
                for (i, predicate) in predicates.iter().enumerate() {
                    if predicate(payload) {
                        out[i].push(DiscoveredFilmPacket {
                            film: entry.file_name().to_string_lossy().into_owned(),
                            chunk,
                            packet_index,
                            packet,
                        });
                    }
                }
            }
        }
    }
    Ok(out)
}

fn packets_of_type(directory: &Path, kind: u16, first: bool) -> Vec<FilmPacketWithChunk> {
    let mut out = Vec::new();
    for chunk in 1..=super::count_film_chunks(directory) {
        let Ok(data) = super::read_film_chunk(directory, chunk as i64) else {
            continue;
        };
        let data: Arc<[u8]> = data.into();
        for (packet_index, packet) in super::walk_film_packets(&data).into_iter().enumerate() {
            if packet.packet_type == kind {
                out.push(FilmPacketWithChunk {
                    chunk,
                    packet_index,
                    packet,
                    data: data.clone(),
                });
                if first {
                    return out;
                }
            }
        }
    }
    out
}
/// Native FirstPacketOfType starts at chunk one (despite its chunk-zero comment).
pub fn first_film_packet_of_type(
    directory: impl AsRef<Path>,
    kind: u16,
) -> Result<FilmPacketWithChunk, FilmPacketDiscoveryError> {
    let directory = directory.as_ref();
    if super::count_film_chunks(directory) == 0 {
        return Err(FilmPacketDiscoveryError::NoChunks(directory.into()));
    }
    packets_of_type(directory, kind, true)
        .into_iter()
        .next()
        .ok_or_else(|| FilmPacketDiscoveryError::NoPacket {
            directory: directory.into(),
            kind,
        })
}
/// All matching packets from chunks one through the native contiguous path count.
/// A missing directory or unreadable chunks yield no matches.
pub fn all_film_packets_of_type(
    directory: impl AsRef<Path>,
    kind: u16,
) -> Vec<FilmPacketWithChunk> {
    packets_of_type(directory.as_ref(), kind, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    fn packet_json(p: &FilmPacket, index: usize) -> serde_json::Value {
        serde_json::json!({"Index":index,"Type":p.packet_type,"Start":p.payload_offset,"Size":p.payload_size,"TimestampUS":p.timestamp_us})
    }
    fn unhex(h: &str) -> Vec<u8> {
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect()
    }
    #[test]
    fn native_packet_discovery() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/packet-discovery-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        struct Directory(std::path::PathBuf);
        impl Drop for Directory {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let root = Directory(std::env::temp_dir().join(format!(
                "halo-packet-discovery-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )));
        std::fs::create_dir(&root.0).unwrap();
        for dir in fixture["dirs"].as_array().unwrap() {
            std::fs::create_dir_all(root.0.join(dir.as_str().unwrap())).unwrap();
        }
        for file in fixture["files"].as_array().unwrap() {
            std::fs::write(
                root.0.join(file["path"].as_str().unwrap()),
                unhex(file["hex"].as_str().unwrap()),
            )
            .unwrap();
        }
        let empty = film_packet_equal(&[]);
        let no_prefix = film_packet_prefix(&[]);
        let zero = film_packet_prefix(&[0]);
        let exact = film_packet_equal(&[1, 1, 1]);
        let length = |p: &[u8]| p.len() == 3;
        let found =
            find_film_packets(&root.0, &[&empty, &no_prefix, &zero, &exact, &length]).unwrap();
        assert_eq!(found.len(), 5);
        let actual:Vec<Vec<_>>=found.iter().map(|g|g.iter().map(|r|serde_json::json!({"film":r.film,"chunk":r.chunk,"packet":packet_json(&r.packet,r.packet_index)})).collect()).collect();
        assert_eq!(serde_json::to_value(actual).unwrap(), fixture["matches"]);
        assert!(found[1].is_empty());
        assert!(found.iter().flatten().any(|r| r.chunk == 0));
        assert!(find_film_packets(root.0.join("missing"), &[]).is_err());
        assert!(find_film_packets(&root.0, &[]).unwrap().is_empty());
        let queries = fixture["queries"].as_array().unwrap();
        assert_eq!(queries.len(), 52);
        for (i, q) in queries.iter().enumerate() {
            let dir = root.0.join(q["film"].as_str().unwrap());
            let kind = q["kind"].as_u64().unwrap() as u16;
            match first_film_packet_of_type(&dir, kind) {
                Ok(r) => {
                    assert_eq!(q["error"], "");
                    assert_eq!(r.chunk, q["chunk"].as_u64().unwrap() as usize);
                    assert_eq!(packet_json(&r.packet, r.packet_index), q["packet"]);
                    assert_eq!(
                        r.data.as_ref(),
                        unhex(q["hex"].as_str().unwrap()),
                        "first bytes {i}"
                    );
                }
                Err(FilmPacketDiscoveryError::NoChunks(_)) => assert_eq!(q["error"], "no_chunks"),
                Err(FilmPacketDiscoveryError::NoPacket { .. }) => {
                    assert_eq!(q["error"], "no_packet")
                }
            }
            let all = all_film_packets_of_type(&dir, kind);
            let expected = q["all"].as_array().unwrap();
            assert_eq!(all.len(), expected.len());
            for (r, e) in all.iter().zip(expected) {
                assert_eq!(r.chunk, e["chunk"].as_u64().unwrap() as usize);
                assert_eq!(packet_json(&r.packet, r.packet_index), e["packet"]);
                assert_eq!(r.data.as_ref(), unhex(e["hex"].as_str().unwrap()));
            }
        }
    }
}
