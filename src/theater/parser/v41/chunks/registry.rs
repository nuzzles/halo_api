use super::*;

impl V41RegistryChunkReader {
    pub(super) fn read(
        source: FilmChunk,
        data: Vec<u8>,
        transport: ChunkTransport,
    ) -> Result<RegistryChunk, ParseError> {
        let body = crate::theater::parser::v41::registry::parse_registry_chunk(&data)?;
        Ok(RegistryChunk {
            source,
            source_position: 0,
            data,
            transport,
            body,
        })
    }
}
