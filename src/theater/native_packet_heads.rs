//! Source-selected native head readers. These do not establish list boundaries.
use super::*;
use serde::{Deserialize, Serialize};

/// The scanner selected a reader for this packet. `None` inside a selected
/// variant is a native refusal, distinct from no applicable reader. Padding is
/// retained on successful reader results and must not become recorded data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NativePacketHeadRead {
    Zoom(Option<NativeZoomRead>),
    Pickup(NativePickupRead),
    Translocator(Option<NativeTranslocatorEvent>),
}

/// Apply the pinned native type, minimum-length and first-byte predicates.
/// Pickup results are direct decoder attempts, without slot-band publication
/// filters, player resolution or timestamp sorting. The optional translocator
/// map is external quantization configuration, not a value read from the film.
pub fn read_native_packet_head(
    packet_type: u16,
    payload: &[u8],
    translocator_map: Option<&FilmMapBounds>,
) -> Option<NativePacketHeadRead> {
    if packet_type != 0 || payload.len() < 2 {
        return None;
    }
    match payload[0] {
        0xca => Some(NativePacketHeadRead::Zoom(decode_native_zoom_head(payload))),
        0xc4 => Some(NativePacketHeadRead::Pickup(decode_native_biped_pickup(
            payload,
            &mut BipedPickupStats::default(),
        ))),
        0xfa => Some(NativePacketHeadRead::Translocator(
            decode_native_translocator_head(payload, translocator_map),
        )),
        _ => None,
    }
}

#[cfg(test)]
#[path = "native_packet_heads_tests.rs"]
pub(crate) mod tests;
