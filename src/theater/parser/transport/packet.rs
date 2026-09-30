//! Version-independent packet envelopes and source extents.
pub(crate) fn read_packet_headers(
    data: &[u8],
) -> Vec<(
    crate::theater::film::PacketSource,
    crate::theater::film::FilmPacketHeader,
)> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    while let Some(header) = data.get(offset..offset.saturating_add(16)) {
        let kind = u16::from_le_bytes(header[..2].try_into().unwrap());
        let size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        let stamp = u64::from_le_bytes(header[8..16].try_into().unwrap());
        let start = offset + 16;
        let Some(end) = start.checked_add(size).filter(|end| *end <= data.len()) else {
            break;
        };
        if size == 0 && kind != 7 {
            break;
        }
        out.push((
            crate::theater::film::PacketSource {
                header: crate::theater::film::ByteRange {
                    start: offset,
                    end: start,
                },
                payload: crate::theater::film::ByteRange { start, end },
            },
            crate::theater::film::FilmPacketHeader {
                packet_type: kind,
                unknown_2: [header[2], header[3]],
                payload_size: size as u32,
                timestamp_us: stamp,
            },
        ));
        offset = end;
        if kind == 7 {
            break;
        }
    }
    out
}
