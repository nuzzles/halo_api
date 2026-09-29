//! Native position publication categories.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativePositionKind {
    /// Native PosKindRaw: re-emits the saved baseline, never the 96 copied wire bits.
    Baseline,
    Absolute,
    AbsoluteFallback,
    Delta8,
    DeltaAxis,
}
