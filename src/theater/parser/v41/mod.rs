//! Structural v41 decoding. Recorded models live under `theater::film`.
pub(crate) mod chunks;
pub(crate) mod context;
pub(crate) mod reference;
pub(crate) mod reference_bits;
pub(crate) use chunks::V41ChunkReader;
#[cfg(test)]
mod tests;
