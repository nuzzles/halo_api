//! Native data models.
/// Original location in a decompressed chunk; bits are MSB-first and half-open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceSpan {
    /// Chunk index.
    pub chunk: i32,
    /// Packet payload byte offset in that chunk (zero for non-packet data).
    pub payload_byte: usize,
    /// First bit relative to the payload.
    pub bit: usize,
    /// First bit after the checked window; does not imply a complete record.
    pub end_bit: usize,
}

/// One checked byte-aligned replication packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmPacketHeader {
    pub chunk_index: i32,
    pub packet_type: u16,
    pub byte_2: u8,
    pub byte_3: u8,
    pub payload_offset: usize,
    pub payload_size: usize,
    pub timestamp_us: u64,
}
