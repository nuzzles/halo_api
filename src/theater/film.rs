//! Native film input and the three source-preserving recording sections.
use super::parser::source::inflate_film_chunk;
use super::parser::*;
use super::parser::{
    FilmRegistryRead, FilmRegistryReadError, FilmSource, FilmSourceMetadata, parse_registry_chunk,
};
use serde::{Deserialize, Serialize};

/// Transport chunk category supplied by the film manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkKind {
    Registry,
    Replication,
    Summary,
}
impl TryFrom<i32> for ChunkKind {
    type Error = ParseError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Registry),
            2 => Ok(Self::Replication),
            3 => Ok(Self::Summary),
            other => Err(ParseError::ChunkKind(other)),
        }
    }
}
impl ChunkKind {
    pub(crate) fn number(self) -> i64 {
        match self {
            Self::Registry => 1,
            Self::Replication => 2,
            Self::Summary => 3,
        }
    }
}

/// One ordered input chunk. Bytes may be zlib-compressed or already decompressed.
/// Manifest identity and timing are optional; supplied values are retained exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmChunk {
    pub kind: ChunkKind,
    pub index: Option<i64>,
    pub start_ms: Option<i64>,
    pub data: Vec<u8>,
}
impl FilmChunk {
    pub fn new(kind: ChunkKind, data: impl Into<Vec<u8>>) -> Self {
        Self {
            kind,
            index: None,
            start_ms: None,
            data: data.into(),
        }
    }
}

/// Structurally decoded native recording. Interpretations belong to ResolvedFilm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Film {
    pub registry: Registry,
    pub replication: ReplicationStream,
    pub summaries: SummaryEvents,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Registry {
    pub definition: FilmRegistryRead,
    /// Includes all bootstrap bytes after the component registry, even when opaque.
    pub chunk: ParsedChunk,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplicationStream {
    pub chunks: Vec<ParsedChunk>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SummaryEvents {
    pub chunks: Vec<ParsedChunk>,
}
impl SummaryEvents {
    /// Recorded summary entries in source order, without inferred player linkage.
    pub fn events(&self) -> impl Iterator<Item = &SummaryEvent> {
        self.chunks
            .iter()
            .flat_map(|c| &c.packets)
            .flat_map(|p| match &p.body {
                NativeFilmPacketBody::Summary { events, .. } => events.as_slice(),
                _ => &[],
            })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedChunk {
    /// Original input, including exact transport bytes and supplied metadata.
    pub source: FilmChunk,
    /// Position in the input list, independent of optional manifest numbering.
    pub source_position: usize,
    pub data: Vec<u8>,
    pub packets: Vec<NativeFilmPacket>,
    /// Start of the unwalked suffix. Nested packet bit offsets address `data`.
    pub packet_walk_end_byte: usize,
}
impl ParsedChunk {
    /// Borrow a packet payload from its retained decompressed source.
    pub fn payload(&self, packet: &NativeFilmPacket) -> Option<&[u8]> {
        let header = packet.header;
        self.data
            .get(header.payload_offset..header.payload_offset.checked_add(header.payload_size)?)
    }
}
impl Film {
    /// Read the registry header, select a supported parser, and decode the chunks.
    /// Input order is retained. Exactly one registry chunk is required.
    pub fn parse(chunks: impl IntoIterator<Item = FilmChunk>) -> Result<Self, ParseError> {
        let chunks: Vec<_> = chunks.into_iter().collect();
        let registries: Vec<_> = chunks
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == ChunkKind::Registry)
            .map(|(i, _)| i)
            .collect();
        let position = match registries.as_slice() {
            [] => return Err(ParseError::MissingRegistry),
            [i] => *i,
            _ => return Err(ParseError::MultipleRegistries),
        };
        let bytes = inflate_film_chunk(&chunks[position].data);
        let version = bytes
            .get(..4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .ok_or(ParseError::TruncatedRegistryHeader)?;
        if version != 41 {
            return Err(ParseError::UnsupportedVersion(version));
        }
        let registry = parse_registry_chunk(&bytes)?;
        let metadata: Vec<_> = chunks
            .iter()
            .enumerate()
            .map(|(i, c)| FilmSourceMetadata {
                index: c.index.unwrap_or(i as i64),
                chunk_type: c.kind.number(),
                start_ms: c.start_ms.unwrap_or(0),
            })
            .collect();
        let raw: Vec<_> = chunks.iter().map(|c| c.data.as_slice()).collect();
        let source =
            FilmSource::load(&raw, &metadata).map_err(|e| ParseError::Source(e.to_string()))?;
        super::parser::v41::V41ChunkParser::parse(chunks, &source, registry, position)
    }
    pub fn resolve(&self) -> super::ResolvedFilm<'_> {
        super::ResolvedFilm::from_film(self)
    }
    /// Sections retain input positions, allowing a lookup without copying bytes.
    pub(crate) fn chunk(&self, position: usize) -> Option<&ParsedChunk> {
        if self.registry.chunk.source_position == position {
            return Some(&self.registry.chunk);
        }
        for chunks in [&self.replication.chunks, &self.summaries.chunks] {
            if let Ok(i) = chunks.binary_search_by_key(&position, |c| c.source_position) {
                return Some(&chunks[i]);
            }
        }
        None
    }
    pub(crate) fn chunks(&self) -> impl Iterator<Item = &ParsedChunk> {
        std::iter::once(&self.registry.chunk)
            .chain(&self.replication.chunks)
            .chain(&self.summaries.chunks)
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("film requires a registry chunk")]
    MissingRegistry,
    #[error("film contains multiple registry chunks")]
    MultipleRegistries,
    #[error("truncated registry version header")]
    TruncatedRegistryHeader,
    #[error("unsupported film version {0}; supported: 41")]
    UnsupportedVersion(u32),
    #[error("unsupported film chunk kind {0}")]
    ChunkKind(i32),
    #[error(transparent)]
    Registry(#[from] FilmRegistryReadError),
    #[error("unable to load film chunks: {0}")]
    Source(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeFilmPacket {
    pub header: FilmPacket,
    /// All nested bit offsets are relative to this header's payload_offset.
    /// Stops and synthetic padded reads remain in the native reader result.
    pub body: NativeFilmPacketBody,
    /// An independent direct event-head read, never an inferred gameplay event.
    pub event_head: Option<DecodedHeadEvent>,
    /// Sequential event-layout reads at payload bit one. An absent runtime
    /// code-15 gate stops this read; fixed-size opaque bodies remain marked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_list: Option<NativeEventListRead>,
    /// Entity/control reads beginning at a completed nonempty event list's
    /// terminator. The original generic message-view stop remains in `body`.
    /// Runtime layout selection is recorded by the enclosing event list/gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_continuation: Option<NativeEventContinuation>,
    /// Dedicated native padded reads, including selected-reader refusals.
    /// The bounded `event_head` and original payload remain independently kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_head: Option<NativePacketHeadRead>,
    /// Independent native damage projection and raw fields. This endpoint does
    /// not replace event-list termination or imply inferred player identities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_read: Option<NativeWeaponDamageRead>,
    /// Bounded roster trace, including the first rejected entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roster_read: Option<NativeRosterRead>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeEventContinuation {
    /// Payload-relative bit immediately after the event-list terminator.
    pub start_bit: usize,
    /// Only the following entity/control views are counted in this result.
    /// Refusals and partial reads do not erase the established list boundary.
    pub frame: Result<ProductionFrame, String>,
    /// Whether this attempt may update the grammar state used by later packets.
    #[serde(default)]
    pub state_policy: NativeContinuationStatePolicy,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeContinuationStatePolicy {
    /// Older exports did not report this policy.
    #[default]
    Unknown,
    Applied,
    /// Pinned code-0 body readers disagree on a field's gate polarity. Keep the
    /// generic continuation attempt but do not propagate its binding effects.
    IsolatedConflictingDamageGrammar,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// Preserve the native by-value API while retained diagnostics grow.
#[allow(clippy::large_enum_variant)]
pub enum NativeFilmPacketBody {
    Frame(Box<ProductionFrame>),
    Datums(DatumTable),
    Roster(RosterUpdate),
    Keyframes(NativeKeyframeTable),
    /// Guarded captured-layout reads in wire order; intervening state is opaque.
    Summary {
        declared_events: u32,
        events: Vec<SummaryEvent>,
    },
    /// A reader could not start. The complete payload remains in the chunk.
    Refused {
        message: String,
    },
    /// No body reader selected; this does not imply an empty payload.
    Opaque,
}

#[cfg(test)]
mod tests;
