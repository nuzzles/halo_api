//! Internal decoding implementation behind `Film::parse`.
//! Native models are public under `theater::film`; resolved models under `theater::resolved`.
pub(crate) mod bits;
pub(crate) mod registry_chunk;
pub(crate) mod transport;
pub(crate) mod v41;
