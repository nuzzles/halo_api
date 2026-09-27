//! Native weapon-hit event readers, including their explicit zero-tail provenance.
use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
/// Read the first damage_aftermath exactly as the native statistics scanner.
/// Synthetic tail bits are retained separately rather than hidden as physical data.
pub fn decode_weapon_damage(payload: &[u8], timestamp_us: u64) -> Option<WeaponDamageRead> {
    native_weapon_damage::decode_with_fields(payload, timestamp_us, false).read
}
