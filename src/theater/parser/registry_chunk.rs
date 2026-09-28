//! Registry bootstrap decoding before version-specific stream dispatch.
use super::{transport::inflate_film_chunk, v41::registry::parse_registry_chunk};
use crate::theater::film::{ChunkKind, FilmChunk, ParseError, ParsedChunk, Registry};

pub(crate) struct RegistryChunkParser;

impl RegistryChunkParser {
    pub(crate) fn parse(source: FilmChunk) -> Result<Registry, ParseError> {
        if source.kind != ChunkKind::Registry {
            return Err(ParseError::RegistryNotFirst);
        }
        let data = inflate_film_chunk(&source.data).into_owned();
        let header = data.get(..4).ok_or(ParseError::TruncatedRegistryHeader)?;
        let version = u32::from_le_bytes(header.try_into().unwrap());
        // Reject unknown versions before interpreting their registry layout.
        let definition = match version {
            41 => parse_registry_chunk(&data)?,
            other => return Err(ParseError::UnsupportedVersion(other)),
        };
        Ok(Registry {
            definition,
            chunk: ParsedChunk {
                source,
                source_position: 0,
                data,
                packets: Vec::new(),
                packet_walk_end_byte: 0,
            },
        })
    }
}
