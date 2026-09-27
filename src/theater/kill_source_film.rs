//! Native kill-source loaded-film adapter. This is a borrowed packet view,
//! not a resolved replay world or an architectural replacement for LegacyFilm.
use super::{FilmSource, KillPacketIdentity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KillSourcePacket<'a> {
    /// Source position and original per-chunk ordinal, not manifest file number.
    pub identity: KillPacketIdentity,
    pub packet_type: u16,
    pub timestamp_us: u64,
    pub payload: &'a [u8],
}
impl KillSourcePacket<'_> {
    /// Native event-list marker; an absent second bit is zero.
    pub fn has_events(&self) -> bool {
        self.payload.first().is_some_and(|b| b & 0x40 != 0)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum KillSourceFilmError {
    #[error("killsource: kill-feed introuvable (aucun chunk HIGHLIGHT)")]
    NoKillFeed,
    #[error("killsource: aucun chunk")]
    NoChunk,
    #[error("killsource: aucun paquet de replication (type 0)")]
    NoReplicationPacket,
    #[error(
        "killsource: registre ECS illisible: filmdec: chunk_00 (registre) encore compresse — decompresser avant ParseRegistryChunk"
    )]
    Registry(#[source] super::FilmRegistryReadError),
}

/// Validate the same initial source conditions before the full chunk-based
/// decoder attempts its kill-feed scan. Metadata numbering does not select data.
pub(crate) fn validate_kill_source_chunks(
    chunks: &[crate::clients::hi::models::FilmChunkData],
) -> Result<(), KillSourceFilmError> {
    if chunks.is_empty() {
        return Err(KillSourceFilmError::NoChunk);
    }
    if !chunks.iter().any(|c| {
        super::fire_events::native_chunk_packets(c)
            .iter()
            .any(|p| p.packet_type == 0)
    }) {
        return Err(KillSourceFilmError::NoReplicationPacket);
    }
    Ok(())
}

/// Native source view: all packets except CHUNK_END, plus timestamp-ordered
/// type-zero indexes. Packet bytes borrow the loaded source without copying.
#[derive(Debug)]
pub struct KillSourceFilm<'a> {
    pub source: &'a FilmSource,
    pub packets: Vec<KillSourcePacket<'a>>,
    pub delta_packet_indices: Vec<usize>,
    pub timestamp_base_us: u64,
    /// Recorded metadata only. None means no registry header could be read.
    /// Retaining an earlier major here does not enable earlier-major decoding.
    pub major_version: Option<u32>,
}
impl<'a> KillSourceFilm<'a> {
    pub fn from_source(source: Option<&'a FilmSource>) -> Result<Self, KillSourceFilmError> {
        let source = source
            .filter(|s| s.num_chunks() != 0)
            .ok_or(KillSourceFilmError::NoChunk)?;
        let mut packets = Vec::new();
        for chunk in 0..source.num_chunks() {
            for (index, p) in source.packets(chunk).iter().enumerate() {
                if p.packet_type == 7 {
                    continue;
                }
                packets.push(KillSourcePacket {
                    identity: KillPacketIdentity {
                        chunk: chunk as i64,
                        packet: index as i64,
                    },
                    packet_type: p.packet_type,
                    timestamp_us: p.timestamp_us,
                    payload: source
                        .payload(p)
                        .expect("packet range belongs to its loaded source"),
                });
            }
        }
        let mut deltas: Vec<_> = packets
            .iter()
            .enumerate()
            .filter_map(|(i, p)| (p.packet_type == 0).then_some((i, p.timestamp_us)))
            .collect();
        if deltas.is_empty() {
            return Err(KillSourceFilmError::NoReplicationPacket);
        }
        super::native_sort::sort_by(&mut deltas, |a, b| a.1.cmp(&b.1));
        let delta_packet_indices: Vec<_> = deltas.into_iter().map(|(i, _)| i).collect();
        let timestamp_base_us = packets[delta_packet_indices[0]].timestamp_us;
        let major_version = source
            .registry_chunk()
            .and_then(|b| b.get(..4))
            .map(|h| u32::from_le_bytes(h.try_into().unwrap()));
        Ok(Self {
            source,
            packets,
            delta_packet_indices,
            timestamp_base_us,
            major_version,
        })
    }
    /// Native unsigned subtraction before conversion to milliseconds. Packets
    /// predating the replication origin intentionally retain wrapping semantics.
    pub fn milliseconds(&self, packet: &KillSourcePacket<'_>) -> i64 {
        (packet.timestamp_us.wrapping_sub(self.timestamp_base_us) / 1000) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_kill_decode_input_refusals() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/kill-chunks-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 128);
        let mut categories = [0; 3];
        for (i, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let hex = c["hex"].as_str().unwrap();
                    crate::clients::hi::models::FilmChunkData {
                        metadata: crate::clients::hi::models::FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 0,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: 0,
                            file_relative_path: String::new(),
                        },
                        data: (0..hex.len())
                            .step_by(2)
                            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                            .collect(),
                    }
                })
                .collect();
            let error = crate::theater::decode_film_kill_sources(
                "error-oracle",
                &chunks,
                41,
                None,
                &Default::default(),
            )
            .unwrap_err();
            assert_eq!(
                error.to_string(),
                row["decode_error"].as_str().unwrap(),
                "input error {i}"
            );
            let crate::theater::DecodeError::KillSource(kind) = error else {
                panic!("untyped native error {i}")
            };
            categories[match kind {
                KillSourceFilmError::NoChunk => 0,
                KillSourceFilmError::NoReplicationPacket => 1,
                KillSourceFilmError::NoKillFeed => 2,
                KillSourceFilmError::Registry(_) => panic!("unexpected registry error {i}"),
            }] += 1;
        }
        assert!(categories.iter().all(|&n| n > 0));
    }
    #[test]
    fn native_loaded_kill_source_packets() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/kill-chunks-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 128);
        for (i, row) in rows.into_iter().enumerate() {
            let mut chunks = Vec::new();
            let mut metadata = Vec::new();
            for c in row["chunks"].as_array().unwrap() {
                let h = c["hex"].as_str().unwrap();
                chunks.push(
                    (0..h.len())
                        .step_by(2)
                        .map(|p| u8::from_str_radix(&h[p..p + 2], 16).unwrap())
                        .collect::<Vec<_>>(),
                );
                metadata.push(crate::theater::FilmSourceMetadata {
                    index: c["index"].as_i64().unwrap(),
                    chunk_type: 0,
                    start_ms: 0,
                });
            }
            if !row["metadata"].as_bool().unwrap() {
                metadata.clear();
            }
            let source =
                (!chunks.is_empty()).then(|| FilmSource::load(&chunks, &metadata).unwrap());
            let result = KillSourceFilm::from_source(source.as_ref());
            if let Err(e) = result {
                assert_eq!(e.to_string(), row["error"].as_str().unwrap(), "case {i}");
                continue;
            }
            assert_eq!(row["error"], "");
            let film = result.unwrap();
            assert_eq!(film.timestamp_base_us, row["base"].as_u64().unwrap());
            assert_eq!(
                film.major_version,
                row["major_read"]
                    .as_bool()
                    .unwrap()
                    .then(|| row["major"].as_u64().unwrap() as u32)
            );
            let encode = |p: &KillSourcePacket<'_>| {
                let original =
                    &film.source.packets(p.identity.chunk as usize)[p.identity.packet as usize];
                assert_eq!(
                    p.payload.as_ptr(),
                    film.source.payload(original).unwrap().as_ptr()
                );
                let hex: String = p.payload.iter().map(|b| format!("{b:02x}")).collect();
                serde_json::json!({"chunk":p.identity.chunk,"index":p.identity.packet,"kind":p.packet_type,"ts":p.timestamp_us,"hex":hex,"events":p.has_events(),"ms":film.milliseconds(p)})
            };
            let packets: Vec<_> = film.packets.iter().map(encode).collect();
            let deltas: Vec<_> = film
                .delta_packet_indices
                .iter()
                .map(|&p| encode(&film.packets[p]))
                .collect();
            assert_eq!(serde_json::json!(packets), row["packets"], "packets {i}");
            assert_eq!(serde_json::json!(deltas), row["deltas"], "deltas {i}");
        }
    }
}
