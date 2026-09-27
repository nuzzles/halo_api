//! Native derived-cache ammunition section. Cache bytes carry no gauge quantum.
use super::{KeyframeSlotAmmo, NativeFactsReader, NativeFactsWriter};
pub fn encode_facts_ammo(writer: &mut NativeFactsWriter, ammo: &KeyframeSlotAmmo) {
    writer.boolean(ammo.mag.is_some());
    if let Some(mag) = ammo.mag {
        writer.unsigned(u64::from(mag));
    }
    writer.boolean(ammo.res.is_some());
    if let Some(res) = ammo.res {
        writer.unsigned(u64::from(res));
    }
    writer.boolean(ammo.gauge.is_some());
    if let Some(gauge) = ammo.gauge {
        writer.float64(gauge);
    }
    writer.unsigned(u64::from(ammo.overheat));
    writer.unsigned(u64::from(ammo.flags));
}
/// Preserve fields decoded before an error, including present zero-valued fields.
/// Inspect reader.error() before treating the partial value as a complete record.
pub fn decode_facts_ammo(reader: &mut NativeFactsReader<'_>) -> KeyframeSlotAmmo {
    let mut ammo = KeyframeSlotAmmo::default();
    if reader.boolean() {
        ammo.mag = Some(reader.unsigned() as u32);
    }
    if reader.boolean() {
        ammo.res = Some(reader.unsigned() as u32);
    }
    if reader.boolean() {
        if reader.remaining() < 8 {
            reader.fail(format!("jauge tronquee a l offset {}", reader.offset()));
            return ammo;
        }
        let bytes = reader.section(8).expect("validated gauge section");
        ammo.gauge = Some(f64::from_bits(u64::from_le_bytes(
            bytes.try_into().unwrap(),
        )));
    }
    ammo.overheat = reader.unsigned() as u32;
    ammo.flags = reader.unsigned() as u32;
    ammo
}
