//! Structural v41 decoding. Recorded models live under `theater::film`.
pub(crate) mod chunks;
pub(crate) mod context;
pub(crate) mod packet;
pub(crate) mod reference;
pub(crate) mod registry;
pub(crate) mod replication;
pub(crate) mod summary;
pub(crate) use chunks::V41ChunkReader;
