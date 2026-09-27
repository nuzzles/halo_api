//! Native replay grid construction and its independently checked match-clock origin.
use super::{DecodeError, IdentityClock, ReplayPositionSample, fire_events::native_chunk_packets};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayTimeline {
    /// Stable chronological order, including equal-time positions from different lives.
    pub samples: Vec<ReplayPositionSample>,
    pub clock: IdentityClock,
    pub frame_interval_ms: i64,
    pub duration_ms: i64,
}
/// Prepare the native frame grid (100ms by default), before identity and sampling.
/// Empty positions or an overflowing interval that yields a zero step cannot form
/// a timeline. Normal film timestamps and positive intervals retain Go arithmetic.
pub fn prepare_replay_timeline(
    samples: &[ReplayPositionSample],
    interval_ms: i64,
) -> Option<ReplayTimeline> {
    if samples.is_empty() {
        return None;
    }
    let interval_ms = if interval_ms > 0 { interval_ms } else { 100 };
    let step_us = (interval_ms as u64).wrapping_mul(1000);
    if step_us == 0 {
        return None;
    }
    let mut samples = samples.to_vec();
    samples.sort_by_key(|s| s.position.timestamp_us);
    let origin_us = samples[0].position.timestamp_us;
    let last = samples.last().unwrap().position.timestamp_us;
    let frame_count = ((last.wrapping_sub(origin_us) / step_us) as i64).wrapping_add(1);
    Some(ReplayTimeline {
        samples,
        clock: IdentityClock {
            origin_us,
            step_us,
            frame_count,
        },
        frame_interval_ms: interval_ms,
        duration_ms: frame_count.wrapping_mul(interval_ms),
    })
}
/// Read the native replay clock from an already loaded source, including sources
/// without metadata (where chunk numbers are source positions). Duplicate chunk
/// numbers select the first loaded entry; an empty entry does not fall through.
pub fn scan_loaded_replay_clock_origin(source: &super::FilmSource) -> Result<u64, DecodeError> {
    let (_, packets) = source
        .chunk_by_number(1)
        .ok_or(DecodeError::Missing("clock origin chunk 1"))?;
    packets
        .first()
        .map(|p| p.timestamp_us)
        .ok_or(DecodeError::Missing("clock origin packet"))
}

/// Result of the native chunk-one clock read. A recorded zero remains a
/// successful read. Missing input is distinct from an older LegacyFilm export that
/// never retained this result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayClockOriginRead {
    Read(super::FilmPacket),
    MissingChunk,
    MissingPacket,
}
impl ReplayClockOriginRead {
    pub fn timestamp_us(&self) -> Option<u64> {
        match self {
            Self::Read(packet) => Some(packet.timestamp_us),
            _ => None,
        }
    }
    pub fn result(&self) -> Result<u64, DecodeError> {
        match self {
            Self::Read(packet) => Ok(packet.timestamp_us),
            Self::MissingChunk => Err(DecodeError::Missing("clock origin chunk 1")),
            Self::MissingPacket => Err(DecodeError::Missing("clock origin packet")),
        }
    }
}

/// Retain the first native packet of chunk one, including its original range.
/// Duplicate numbers select the first input entry, even when it is empty.
pub fn read_replay_clock_origin(chunks: &[FilmChunkData]) -> ReplayClockOriginRead {
    let Some(chunk) = chunks.iter().find(|c| c.metadata.index == 1) else {
        return ReplayClockOriginRead::MissingChunk;
    };
    native_chunk_packets(chunk)
        .into_iter()
        .next()
        .map(ReplayClockOriginRead::Read)
        .unwrap_or(ReplayClockOriginRead::MissingPacket)
}

/// The first readable packet in manifest chunk one establishes the film clock.
/// A timestamp of zero is a successful read, but cannot establish a published origin.
pub fn scan_replay_clock_origin(chunks: &[FilmChunkData]) -> Result<u64, DecodeError> {
    read_replay_clock_origin(chunks).result()
}
/// Frame-zero time relative to the highlight clock. Five or more matched deaths
/// can contradict the direct reading; an error over one second withholds it.
pub fn resolve_replay_origin_ms(
    first_position_us: u64,
    film_clock_us: u64,
    death_offset_ms: i64,
    matched: usize,
) -> Option<i64> {
    if film_clock_us == 0 || first_position_us < film_clock_us {
        return None;
    }
    let read = (first_position_us.wrapping_sub(film_clock_us) as i64) / 1000;
    if matched >= 5 {
        let control = (first_position_us as i64 / 1000).wrapping_sub(death_offset_ms);
        let delta = control.wrapping_sub(read);
        if !(-1000..=1000).contains(&delta) {
            return None;
        }
    }
    Some(read)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        samples: Vec<ReplayPositionSample>,
        interval: i64,
        timeline: Option<ReplayTimeline>,
        first: u64,
        film_clock: u64,
        offset: i64,
        matched: usize,
        origin: Option<i64>,
    }
    #[test]
    fn film_retains_native_clock_independently_of_legacy_origin() {
        use super::super::{DecodeOptions, LegacyFilm};
        use crate::clients::hi::models::FilmChunk;
        fn chunk(index: i32, kind: i32, data: Vec<u8>) -> FilmChunkData {
            FilmChunkData {
                metadata: FilmChunk {
                    index,
                    chunk_type: kind,
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            }
        }
        fn packets(times: &[u64]) -> Vec<u8> {
            let mut bytes = Vec::new();
            for time in times {
                bytes.extend_from_slice(&[0, 0, 0, 0]);
                bytes.extend_from_slice(&1u32.to_le_bytes());
                bytes.extend_from_slice(&time.to_le_bytes());
                bytes.push(0);
            }
            bytes
        }
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut bootstrap)
            .unwrap();
        for mode in 0..4 {
            let mut chunks = vec![chunk(0, 1, bootstrap.clone())];
            if mode > 0 {
                chunks.push(chunk(
                    1,
                    2,
                    match mode {
                        1 => Vec::new(),
                        2 => packets(&[200, 100]),
                        _ => packets(&[0, 100]),
                    },
                ));
            }
            chunks.push(chunk(2, 2, packets(&[1000])));
            let film = LegacyFilm::try_from_chunks(
                &chunks,
                DecodeOptions {
                    retain_coverage: false,
                    ..DecodeOptions::v41()
                },
            )
            .unwrap();
            assert!(film.packets.is_empty());
            assert_eq!(film.origin_timestamp_us, if mode < 2 { 1000 } else { 100 });
            let identity = film
                .native_identity_inputs
                .as_ref()
                .expect("identity read retained");
            assert_eq!(
                identity,
                &super::super::scan_film_identity_inputs(&chunks, 41, &[]).unwrap()
            );
            let read = film.native_clock_origin.as_ref().unwrap();
            match mode {
                0 => assert_eq!(read, &ReplayClockOriginRead::MissingChunk),
                1 => assert_eq!(read, &ReplayClockOriginRead::MissingPacket),
                _ => {
                    let ReplayClockOriginRead::Read(packet) = read else {
                        panic!("recorded clock");
                    };
                    assert_eq!(packet.timestamp_us, if mode == 2 { 200 } else { 0 });
                    assert_eq!(
                        (
                            packet.chunk_index,
                            packet.payload_offset,
                            packet.payload_size
                        ),
                        (1, 16, 1)
                    );
                    assert_eq!(packet.packet_type, 0);
                }
            }
            let mut value = serde_json::to_value(&film).unwrap();
            assert_eq!(
                serde_json::from_value::<LegacyFilm>(value.clone()).unwrap(),
                film
            );
            let mut old_identity = value.clone();
            old_identity
                .as_object_mut()
                .unwrap()
                .remove("native_identity_inputs");
            assert!(
                serde_json::from_value::<LegacyFilm>(old_identity)
                    .unwrap()
                    .native_identity_inputs
                    .is_none()
            );
            value.as_object_mut().unwrap().remove("native_clock_origin");
            assert!(
                serde_json::from_value::<LegacyFilm>(value)
                    .unwrap()
                    .native_clock_origin
                    .is_none()
            );
        }
    }

    #[test]
    fn native_replay_grid_and_origin() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/replay-clock-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                prepare_replay_timeline(&c.samples, c.interval),
                c.timeline,
                "timeline {i}"
            );
            assert_eq!(
                resolve_replay_origin_ms(c.first, c.film_clock, c.offset, c.matched),
                c.origin,
                "origin {i}"
            );
        }
    }
}
