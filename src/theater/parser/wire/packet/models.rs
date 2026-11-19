//! Source-backed packet envelopes independent of body grammar.

/// One framed packet whose body type is selected by its enclosing chunk kind.
///
/// The header locates the original payload in the enclosing decompressed chunk.
/// A failed body decode therefore preserves the packet envelope and source bytes.
/// Parsed envelopes agree with their retained header bytes. Public construction
/// or deserialization must preserve that contract; use [`Self::payload`] to
/// check the envelope against an enclosing chunk before accessing its payload.
#[derive(Debug, Clone, PartialEq)]
pub struct Packet<T, E> {
    /// Byte ranges of the header and payload in the enclosing inflated chunk.
    pub source: PacketSource,
    /// Values read from the packet's 16-byte wire header.
    pub header: FilmPacketHeader,
    /// Decoded body or an explicit reason it remains opaque; payload bytes
    /// stay in the enclosing chunk rather than being copied here.
    pub body: PacketRead<T, E>,
}

/// Associates a decoded body variant with its recorded packet type code.
pub trait PacketBody {
    fn packet_type(&self) -> u16;

    /// Source-backed structural regions. Unlisted bits remain unparsed.
    fn source_regions(&self) -> Vec<SourceRegion>;
}

impl<T: PacketBody, E> Packet<T, E> {
    /// Partition the retained payload into source-backed fields, opaque regions,
    /// known padding, and unparsed bits. This derives coverage from the model;
    /// it does not decode bytes again or claim a gap is padding.
    /// Costs O(n log n) for n region boundaries and allocates the result.
    /// Invalid envelopes or conflicting/out-of-bounds regions return None.
    pub fn source_coverage(&self, data: &[u8]) -> Option<Vec<SourceRegion>> {
        let bits = self.payload(data)?.len().checked_mul(8)?;
        let regions = match &self.body {
            PacketRead::Decoded(body) => body.source_regions(),
            PacketRead::Opaque { .. } => vec![SourceRegion {
                source: BitRange {
                    start: 0,
                    end: bits,
                },
                kind: SourceRegionKind::Opaque,
            }],
        };
        super::coverage::partition(bits, regions)
    }

    /// Borrow the payload only when source ranges, the recorded header, and a
    /// decoded body type agree. This validates the envelope, not nested fields.
    /// The bytes must be from the packet's enclosing decompressed chunk.
    pub fn payload<'a>(&self, data: &'a [u8]) -> Option<&'a [u8]> {
        let source = self.source;
        if source.header.end.checked_sub(source.header.start)? != 16
            || source.header.end != source.payload.start
            || source.payload.end.checked_sub(source.payload.start)?
                != usize::try_from(self.header.payload_size).ok()?
            || !self.body_type_matches_header()
        {
            return None;
        }
        let header = data.get(source.header.start..source.header.end)?;
        if header[..2] != self.header.packet_type.to_le_bytes()
            || header[2..4] != self.header.unknown_2
            || header[4..8] != self.header.payload_size.to_le_bytes()
            || header[8..16] != self.header.timestamp_us.to_le_bytes()
        {
            return None;
        }
        data.get(source.payload.start..source.payload.end)
    }

    /// Checks that a decoded body agrees with the wire header. Opaque bodies
    /// carry no typed variant and therefore cannot conflict.
    pub fn body_type_matches_header(&self) -> bool {
        match &self.body {
            PacketRead::Decoded(body) => body.packet_type() == self.header.packet_type,
            PacketRead::Opaque { .. } => true,
        }
    }
}

/// Packet framing result for one inflated replication or summary chunk.
#[derive(Debug, Clone, PartialEq)]
pub struct PacketStream<T, E> {
    /// Envelopes in recorded byte order, including packets with opaque bodies.
    pub packets: Vec<Packet<T, E>>,
    /// Half-open byte ranges in the inflated chunk that framing did not
    /// classify as packets.
    pub opaque: Vec<ByteRange>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PacketRead<T, E> {
    /// A supported body decoder ran. Nested records may still be partial,
    /// unsupported, or refused; inspect their individual decode outcomes.
    Decoded(T),
    Opaque {
        reason: E,
    },
}

impl<T, E> PacketRead<T, E> {
    pub fn decoded(&self) -> Option<&T> {
        match self {
            Self::Decoded(value) => Some(value),
            Self::Opaque { .. } => None,
        }
    }
}

/// Half-open byte range in an enclosing inflated chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    /// Inclusive starting byte offset.
    pub start: usize,
    /// Exclusive ending byte offset.
    pub end: usize,
}

/// Half-open bit range relative to its documented enclosing payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitRange {
    /// Inclusive starting bit offset.
    pub start: usize,
    /// Exclusive ending bit offset.
    pub end: usize,
}

/// Packet envelope locations in the enclosing inflated chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketSource {
    /// Byte range of the 16-byte packet header.
    pub header: ByteRange,
    /// Byte range of the packet payload declared by that header.
    pub payload: ByteRange,
}

/// One checked byte-aligned film packet header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilmPacketHeader {
    /// Recorded packet-type code; interpretation depends on the chunk kind.
    pub packet_type: u16,
    /// Two header bytes whose meaning has not been established.
    pub unknown_2: [u8; 2],
    /// Declared payload length in bytes, excluding the 16-byte header.
    pub payload_size: u32,
    /// Recorded packet timestamp in microseconds, separate from the chunk's
    /// manifest start-time offset.
    pub timestamp_us: u64,
}

/// Classification of retained payload bits; never an inferred gameplay value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRegionKind {
    /// Source-backed fields or structural headers, including uninterpreted words.
    Fields,
    /// A known body/region whose internal grammar is not decoded.
    Opaque,
    /// Alignment bits identified by an established layout.
    Padding,
    /// Bits not consumed by an established field or opaque layout.
    Unparsed,
}

/// One half-open payload-relative region of a coverage partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRegion {
    pub source: BitRange,
    pub kind: SourceRegionKind,
}
