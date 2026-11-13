//! Temporary structural summary reads. None are retained by Film.
use crate::theater::parser::packet::{
    BitRange, Packet, PacketBody, PacketStream, SourceRegion, SourceRegionKind,
};
use serde::{Deserialize, Serialize};

/// Structural parse of the terminal summary chunk, consumed into typed highlights.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSummary {
    /// Numeric index assigned by the film manifest.
    pub index: usize,
    /// Manifest chunk start-time offset in milliseconds, if supplied.
    pub start_ms: i64,
    /// Inflated source bytes. Packet payloads borrow ranges from this buffer.
    pub data: Vec<u8>,
    /// Framed summary packets and any trailing bytes not framed as packets.
    pub body: PacketStream<SummaryPacketBody>,
}

/// Packet envelope used by type-3 summary chunks.
pub type SummaryPacket = Packet<SummaryPacketBody>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SummaryPacketBody {
    /// Recorded event count followed by the complete unparsed record stream.
    /// No marker search or assumed identity-to-tail distance establishes records.
    Events {
        declared_events: u32,
        records: SummaryRecordStream,
    },
}

impl PacketBody for SummaryPacketBody {
    fn source_regions(&self) -> Vec<SourceRegion> {
        match self {
            Self::Events { records, .. } => vec![
                SourceRegion {
                    source: BitRange { start: 0, end: 32 },
                    kind: SourceRegionKind::Fields,
                },
                SourceRegion {
                    source: records.source,
                    kind: SourceRegionKind::Opaque,
                },
            ],
        }
    }

    fn packet_type(&self) -> u16 {
        match self {
            Self::Events { .. } => 9,
        }
    }
}

impl ParsedSummary {
    pub fn payload(&self, packet: &SummaryPacket) -> Option<&[u8]> {
        packet.payload(&self.data)
    }
}

/// Bounded fields from a guarded summary candidate. Marker matching and
/// identity-to-tail association do not establish a proven record boundary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SummaryEventRead {
    pub xuid: u64,
    /// All 16 recorded UTF-16 code units, including terminator and padding.
    pub gamertag_utf16: [u16; 16],
    /// Recorded film-relative timestamp in milliseconds.
    pub timestamp_ms: u32,
    /// Raw metadata byte (medal code for medal events).
    pub metadata: u8,
    /// Raw medal flag.
    pub medal_flag: u8,
    pub type_code: u8,
    /// Exact 60-byte event tail; intervening identity state is not decoded.
    pub source: BitRange,
    /// Recorded XUID field, independently of roster and gamertag.
    pub identity_source: BitRange,
}

impl SummaryEventRead {
    /// Decode the guarded, NUL-terminated player name.
    pub fn gamertag(&self) -> String {
        let end = self
            .gamertag_utf16
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(self.gamertag_utf16.len());
        String::from_utf16_lossy(&self.gamertag_utf16[..end])
    }
}

/// The recorded summary stream whose sequential record grammar is unresolved.
/// Every bit remains available in the enclosing packet payload; no inferred
/// record boundaries are substituted for this opaque source range.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SummaryRecordStream {
    pub source: BitRange,
}
