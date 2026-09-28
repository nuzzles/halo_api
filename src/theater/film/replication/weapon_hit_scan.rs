//! Native data models.
use crate::theater::film::*;
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WeaponDamageRead {
    /// Loaded packet provenance. None for payload-only reads and older exports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<FilmPacket>,
    /// Ordinal among all packets in the chunk, before filtering by packet type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub damage: WeaponDamage,
    /// Second five-bit scalar from the damage body.
    pub secondary_mag_raw: u64,
    /// Body victim reference, separate from the packet-header victim.
    pub body_victim_idx: Option<u64>,
    pub end_bit: usize,
    pub padding_bits: usize,
}
