//! Structural v41 summary packet header. Record boundaries are not established.
mod candidates;
use crate::theater::film::summary::{Summary, SummaryHighlight};
pub(crate) mod models;
use candidates::{candidate_payload, read_summary_candidates};
use models::{ParsedSummary, SummaryPacketBody, SummaryRecordStream};

/// Derive an event-focused summary once from the structural source parse.
pub(crate) fn summarize(parsed: &ParsedSummary) -> Summary {
    let mut events = Vec::new();
    for packet in &parsed.body.packets {
        if packet.body.decoded().is_none() {
            continue;
        }
        let Some(payload) = parsed.payload(packet) else {
            continue;
        };
        for read in read_summary_candidates(payload) {
            events.push(SummaryHighlight {
                gamertag: read.gamertag(),
                xuid: crate::theater::film::Xuid(read.xuid),
                timestamp_us: u64::from(read.timestamp_ms) * 1_000,
                payload: candidate_payload(&read),
            });
        }
    }
    Summary { highlights: events }
}
/// Read summary envelopes and their recorded counts without searching for events.
pub struct V41SummaryChunkReader;

impl V41SummaryChunkReader {
    pub(crate) fn read(
        input: crate::theater::film::InflatedFilmChunk,
        max_packets: usize,
    ) -> Result<ParsedSummary, crate::theater::film::ParseError> {
        use crate::theater::parser::packet::{PacketDecodeError, PacketRead};
        let body = input.read_packets(max_packets, |_, header, payload| {
            if header.packet_type != 9 {
                return PacketRead::Opaque {
                    reason: PacketDecodeError::UnsupportedLayout {
                        packet_type: header.packet_type,
                    },
                };
            }
            read_summary_packet(payload)
                .map(|(declared_events, records)| {
                    PacketRead::Decoded(SummaryPacketBody::Events {
                        declared_events,
                        records,
                    })
                })
                .unwrap_or(PacketRead::Opaque {
                    reason: PacketDecodeError::TruncatedSummaryCount {
                        packet_type: header.packet_type,
                    },
                })
        });
        Ok(ParsedSummary {
            index: input.index,
            start_ms: input.start_ms,
            data: input.data,
            body: body?,
        })
    }
}
use crate::theater::parser::packet::BitRange;

/// Read only the recorded count. Candidate marker searches belong to resolution.
pub(crate) fn read_summary_packet(payload: &[u8]) -> Option<(u32, SummaryRecordStream)> {
    let declared = u32::from_be_bytes(payload.get(..4)?.try_into().ok()?);
    Some((
        declared,
        SummaryRecordStream {
            source: BitRange {
                start: 32,
                end: payload.len().checked_mul(8)?,
            },
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::read_summary_packet;
    use crate::theater::parser::packet::BitRange;

    #[test]
    fn count_is_recorded_and_the_entire_record_stream_remains_opaque() {
        let payload = [0, 0, 0, 2, 0x2d, 0xc0, 0x25, 0xc0, 0, 0, 0x2e, 0xe0];
        let (count, records) = read_summary_packet(&payload).unwrap();
        assert_eq!(count, 2);
        assert_eq!(records.source, BitRange { start: 32, end: 96 });
        for length in 0..4 {
            assert!(read_summary_packet(&payload[..length]).is_none());
        }
        let (count, empty) = read_summary_packet(&payload[..4]).unwrap();
        assert_eq!(count, 2);
        assert_eq!(empty.source, BitRange { start: 32, end: 32 });
    }
}
