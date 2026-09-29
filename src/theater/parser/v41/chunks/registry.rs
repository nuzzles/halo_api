use super::*;

impl V41RegistryChunkReader {
    pub(super) fn read(source: FilmChunk, data: Vec<u8>) -> Result<RegistryChunk, ParseError> {
        let definition = crate::theater::parser::v41::registry::parse_registry_chunk(&data)?;
        Ok(RegistryChunk {
            definition,
            source,
            source_position: 0,
            data,
        })
    }
}
