//! Checked packet framing inside an inflated chunk.
use super::{
    ByteRange, FilmPacketHeader, Packet, PacketBody, PacketRead, PacketSource, PacketStream,
};
use crate::theater::film::{InflatedFilmChunk, ParseError};

impl InflatedFilmChunk {
    /// Frame packet envelopes once, retaining any malformed trailing bytes.
    pub(crate) fn read_packets<T: PacketBody, E>(
        &self,
        max_packets: usize,
        mut decode: impl FnMut(usize, FilmPacketHeader, &[u8]) -> PacketRead<T, E>,
    ) -> Result<PacketStream<T, E>, ParseError> {
        let mut packets = Vec::new();
        let mut offset = 0usize;
        while offset < self.data.len() {
            if packets.len() == max_packets {
                return Err(ParseError::PacketLimit {
                    index: self.index,
                    limit: max_packets,
                });
            }
            let Some(bytes) = self.data.get(offset..offset.saturating_add(16)) else {
                break;
            };
            let size = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
            let start = offset + 16;
            let Some(end) = start
                .checked_add(size as usize)
                .filter(|end| *end <= self.data.len())
            else {
                break;
            };
            let source = PacketSource {
                header: ByteRange {
                    start: offset,
                    end: start,
                },
                payload: ByteRange { start, end },
            };
            let header = FilmPacketHeader {
                packet_type: u16::from_le_bytes(bytes[..2].try_into().unwrap()),
                unknown_2: [bytes[2], bytes[3]],
                payload_size: size,
                timestamp_us: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            };
            let body = decode(packets.len(), header, &self.data[start..end]);
            let packet = Packet {
                source,
                header,
                body,
            };
            debug_assert!(packet.payload(&self.data).is_some());
            packets.push(packet);
            offset = end;
        }
        let opaque = if offset < self.data.len() {
            tracing::warn!(kind = ?self.kind, chunk_index = self.index,
                byte_offset = offset, remaining_bytes = self.data.len() - offset,
                "stopped parsing film chunk before the end");
            vec![ByteRange {
                start: offset,
                end: self.data.len(),
            }]
        } else {
            Vec::new()
        };
        Ok(PacketStream { packets, opaque })
    }
}
