//! Structural v41 summary packet header. Record boundaries are not established.
mod candidates;
pub(crate) mod models;
mod parsed;
use models::{ParsedSummary, SummaryPacketBody, SummaryRecordStream};

/// Read summary envelopes and their recorded counts without searching for events.
pub struct V41SummaryChunkReader;

impl V41SummaryChunkReader {
    pub(crate) fn read(
        input: crate::theater::film::InflatedFilmChunk,
        max_packets: usize,
    ) -> Result<ParsedSummary, crate::theater::film::ParseError> {
        use crate::theater::parser::v41::packet::{PacketDecodeError, PacketRead};
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
use crate::theater::parser::v41::packet::BitRange;

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
