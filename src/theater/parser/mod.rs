//! Decoding implementation behind `Film::parse`.
//! Native models are public under `theater::film`; resolved models under `theater::resolved`.
pub(crate) mod bits;
pub(crate) mod transport;
pub mod v41;

use crate::theater::film::{ChunkKind, Film, FilmChunk, ParseError};
use transport::inflate_film_chunk;

/// Version-selected reader used internally by [`Film::parse`].
///
/// The enum makes the version dispatch explicit while preserving one public
/// parsing entry point. Add future film versions as variants here.
pub enum ChunkReader {
    V41(v41::V41ChunkReader),
}

impl ChunkReader {
    pub(crate) fn read(chunks: impl IntoIterator<Item = FilmChunk>) -> Result<Film, ParseError> {
        let mut chunks = chunks.into_iter();
        let first = chunks.next().ok_or(ParseError::MissingRegistry)?;
        if first.kind != ChunkKind::Registry {
            return Err(ParseError::RegistryNotFirst);
        }

        // The registry version header selects the reader before any
        // version-dependent registry fields are interpreted.
        let registry_data = inflate_film_chunk(&first.data).into_owned();
        let header = registry_data
            .get(..4)
            .ok_or(ParseError::TruncatedRegistryHeader)?;
        let version = u32::from_le_bytes(header.try_into().unwrap());
        let reader = match version {
            41 => Self::V41(v41::V41ChunkReader),
            other => return Err(ParseError::UnsupportedVersion(other)),
        };

        match reader {
            Self::V41(reader) => reader.read(first, registry_data, chunks),
        }
    }
}
