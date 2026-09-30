//! Structural v41 decoding. Recorded models live under `theater::film`.
pub(crate) mod chunks;
pub(crate) mod context;
pub(crate) mod reference;
pub(crate) mod reference_bits;
pub use chunks::{
    V41ChunkReader, registry::V41RegistryChunkReader, replication::V41ReplicationStreamChunkReader,
    summary::V41SummaryChunkReader,
};
#[cfg(test)]
mod tests;
