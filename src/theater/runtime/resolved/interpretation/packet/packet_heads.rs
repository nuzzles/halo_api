//! Reference data models.
/// The scanner selected a reader for this packet. `None` inside a selected
/// variant is a reference refusal, distinct from no applicable reader. Padding is
/// retained on successful reader results and must not become recorded data.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PacketHeadRead {
    Zoom(Option<ZoomRead>),
    Pickup(PickupRead),
    Translocator(Option<TranslocatorRead>),
}

use super::super::head_observations::BipedPickupStats;
use super::{PickupRead, TranslocatorRead, ZoomRead};
use super::{
    pickups::decode_biped_pickup, translocator::decode_translocator_head, zoom::decode_zoom_head,
};
use crate::theater::parser::v41::FilmMapBounds;

/// Apply the pinned reference type, minimum-length and first-byte predicates.
/// Pickup results are direct decoder attempts, without slot-band publication
/// filters, player resolution or timestamp sorting. The optional translocator
/// map is external quantization configuration, not a value read from the film.
pub(crate) fn read_packet_head(
    packet_type: u16,
    payload: &[u8],
    translocator_map: Option<&FilmMapBounds>,
) -> Option<PacketHeadRead> {
    if packet_type != 0 || payload.len() < 2 {
        return None;
    }
    match payload[0] {
        0xca => Some(PacketHeadRead::Zoom(decode_zoom_head(payload))),
        0xc4 => Some(PacketHeadRead::Pickup(decode_biped_pickup(
            payload,
            &mut BipedPickupStats::default(),
        ))),
        0xfa => Some(PacketHeadRead::Translocator(decode_translocator_head(
            payload,
            translocator_map,
        ))),
        _ => None,
    }
}

#[cfg(test)]
#[path = "packet_heads_tests.rs"]
pub(crate) mod tests;
