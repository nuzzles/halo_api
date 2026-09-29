//! Native weapon-hit event readers, including their explicit zero-tail provenance.
#[cfg(test)]
use crate::theater::film::chunks::replication::weapon_hit_scan::WeaponDamageRead;

#[cfg(test)]
/// Read the first damage_aftermath exactly as the native statistics scanner.
/// Synthetic tail bits are retained separately rather than hidden as physical data.
pub fn decode_weapon_damage(payload: &[u8], timestamp_us: u64) -> Option<WeaponDamageRead> {
    super::native_weapon_damage::decode_with_fields(payload, timestamp_us, false).read
}
