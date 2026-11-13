//! Internal projections retained for comparison with pinned reference readers.
//! These callbacks do not replace canonical raw fields or runtime world state.
#[cfg(test)]
pub(crate) mod controls;
pub(crate) mod diagnostics;
pub(crate) mod observations;
