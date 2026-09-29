//! Native data models.
use super::*;
/// The scanner selected a reader for this packet. `None` inside a selected
/// variant is a native refusal, distinct from no applicable reader. Padding is
/// retained on successful reader results and must not become recorded data.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NativePacketHeadRead {
    Zoom(Option<NativeZoomRead>),
    Pickup(NativePickupRead),
    Translocator(Option<NativeTranslocatorEvent>),
}
