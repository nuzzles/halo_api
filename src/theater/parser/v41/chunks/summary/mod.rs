//! Structural v41 summary packet header. Record boundaries are not established.
/// Read summary envelopes and their recorded counts without searching for events.
pub struct V41SummaryChunkReader;

impl V41SummaryChunkReader {
    pub(crate) fn read(
        input: crate::theater::parser::transport::PreparedChunk,
    ) -> crate::theater::film::SummaryChunk {
        use crate::theater::film::{PacketDecodeError, PacketRead, SummaryPacketBody};
        let body = input.read_packets(|header, payload| {
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
        input.with_body(body)
    }
}
use crate::theater::film::BitRange;
use crate::theater::film::chunks::summary::SummaryRecordStream;

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
    use crate::theater::film::BitRange;

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
