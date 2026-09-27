//! LegacyFilm-wide kill-event scan and measured runtime gate selection.
use super::{
    DecodeError, KillEventFields, KillPacketIdentity, bits::Bits, fire_events, native_sort,
    scan_kill_event_chains,
};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmKillEvent {
    pub time_ms: i64,
    pub packet: KillPacketIdentity,
    pub bit: usize,
    pub fields: KillEventFields,
    pub chain: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmKillEventScan {
    pub events: Vec<FilmKillEvent>,
    pub gate15: bool,
    pub packets_with_events: usize,
    pub packets_with_kill: usize,
}
struct Packet<'a> {
    identity: KillPacketIdentity,
    timestamp_us: u64,
    data: &'a [u8],
}
fn scan(mut packets: Vec<Packet<'_>>) -> Result<FilmKillEventScan, DecodeError> {
    native_sort::sort_by(&mut packets, |a, b| a.timestamp_us.cmp(&b.timestamp_us));
    let base = packets
        .first()
        .ok_or(DecodeError::Missing("replication packets"))?
        .timestamp_us;
    let gate = super::select_native_event_gate15(
        packets.iter().map(|p| p.data),
        super::NativeEventGate15Policy::ReferenceInference,
    );
    let mut out = FilmKillEventScan {
        gate15: gate
            .selected
            .expect("reference inference always selects a setting"),
        ..Default::default()
    };
    for p in packets {
        if Bits(p.data).read(1, 1) != Some(1) {
            continue;
        }
        out.packets_with_events += 1;
        let events = scan_kill_event_chains(p.data, out.gate15);
        if !events.is_empty() {
            out.packets_with_kill += 1;
        }
        for e in events {
            out.events.push(FilmKillEvent {
                time_ms: ((p.timestamp_us - base) / 1000) as i64,
                packet: p.identity,
                bit: e.bit,
                fields: e.fields,
                chain: e.chain,
            });
        }
    }
    native_sort::sort_by(&mut out.events, |a, b| a.time_ms.cmp(&b.time_ms));
    Ok(out)
}
/// Scan v41 replication packets from all supplied chunks. Time zero is
/// the earliest replication timestamp. Chunk IDs are source positions, and packet
/// indices include other types.
/// This produces localized evidence, not credited kills or published assists.
pub fn scan_film_kill_events(
    chunks: &[FilmChunkData],
    major_version: i32,
) -> Result<FilmKillEventScan, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    let mut packets = Vec::new();
    for (chunk_position, c) in chunks.iter().enumerate() {
        for (index, p) in fire_events::native_chunk_packets(c).into_iter().enumerate() {
            if p.packet_type != 0 {
                continue;
            }
            packets.push(Packet {
                identity: KillPacketIdentity {
                    chunk: chunk_position as i64,
                    packet: index as i64,
                },
                timestamp_us: p.timestamp_us,
                data: &c.data[p.payload_offset..p.payload_offset + p.payload_size],
            });
        }
    }
    scan(packets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Input {
        chunk: i64,
        packet: i64,
        timestamp_us: u64,
        data: String,
    }
    #[derive(Deserialize)]
    struct Row {
        packets: Vec<Input>,
        expected: FilmKillEventScan,
    }
    #[test]
    fn native_kill_event_scan_oracle() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/kill-event-scan-v41.json.zlib")[..],
        )
        .read_to_end(&mut json)
        .unwrap();
        let rows: Vec<Row> = serde_json::from_slice(&json).unwrap();
        assert_eq!(rows.len(), 128);
        let mut gates = [0; 2];
        for (i, row) in rows.into_iter().enumerate() {
            let bytes: Vec<Vec<u8>> = row
                .packets
                .iter()
                .map(|r| {
                    r.data
                        .as_bytes()
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
                        .collect()
                })
                .collect();
            let packets = row
                .packets
                .iter()
                .zip(&bytes)
                .map(|(r, data)| Packet {
                    identity: KillPacketIdentity {
                        chunk: r.chunk,
                        packet: r.packet,
                    },
                    timestamp_us: r.timestamp_us,
                    data,
                })
                .collect();
            let actual = scan(packets).unwrap();
            assert_eq!(actual, row.expected, "scan {i}");
            let selection = super::super::select_native_event_gate15(
                bytes.iter().map(Vec::as_slice),
                super::super::NativeEventGate15Policy::ReferenceInference,
            );
            assert_eq!(
                selection.selected,
                Some(row.expected.gate15),
                "runtime selection {i}"
            );
            let scores = selection.localized_candidate_counts.unwrap();
            assert_eq!(selection.selected, Some(scores[1] > scores[0]));
            assert_eq!(
                serde_json::from_value::<super::super::NativeEventGate15Selection>(
                    serde_json::to_value(&selection).unwrap()
                )
                .unwrap(),
                selection
            );
            gates[usize::from(actual.gate15)] += 1;
        }
        assert!(gates[0] > 0 && gates[1] > 0);
    }
}
