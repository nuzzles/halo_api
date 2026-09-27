//! Complete world-object creation records in the derived facts cache.
use super::{GroundWeaponAmmo, NativeFactsReader, NativeFactsWriter};
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsEquipmentCreation {
    pub timestamp_us: u64,
    pub slot: u32,
    pub generation: u32,
    pub chunk: i64,
    pub packet_index: i64,
    pub bit_pos: i64,
    pub has_ref: bool,
    pub reference: u32,
    pub has_id: bool,
    pub ability_id: u32,
    pub mpp_present: [bool; 4],
    pub mpp_val: [u64; 4],
    pub position: [f32; 3],
    pub mask: Option<Vec<i64>>,
    pub mask_full: bool,
    pub mask_has_i0: bool,
    pub default_state_bits: i64,
    pub has_ammo: bool,
    pub ammo: GroundWeaponAmmo,
    pub after_bit: i64,
}
pub fn encode_facts_creations(
    writer: &mut NativeFactsWriter,
    creations: &[FactsEquipmentCreation],
) {
    writer.unsigned(creations.len() as u64);
    let mut timestamp = 0u64;
    for c in creations {
        writer.unsigned(c.timestamp_us.wrapping_sub(timestamp));
        timestamp = c.timestamp_us;
        writer.unsigned(u64::from(c.slot));
        writer.unsigned(u64::from(c.generation));
        writer.signed(c.chunk);
        writer.signed(c.packet_index);
        writer.signed(c.bit_pos);
        writer.boolean(c.has_ref);
        writer.unsigned(u64::from(c.reference));
        writer.boolean(c.has_id);
        writer.unsigned(u64::from(c.ability_id));
        for i in 0..4 {
            writer.boolean(c.mpp_present[i]);
            writer.unsigned(c.mpp_val[i]);
        }
        for value in c.position {
            writer.float32(value);
        }
        let mask = c.mask.as_deref().unwrap_or(&[]);
        writer.unsigned(mask.len() as u64);
        for &value in mask {
            writer.signed(value);
        }
        writer.boolean(c.mask_full);
        writer.boolean(c.mask_has_i0);
        writer.signed(c.default_state_bits);
        writer.boolean(c.has_ammo);
        writer.unsigned(u64::from(c.ammo.mag));
        writer.unsigned(u64::from(c.ammo.res));
        writer.signed(c.after_bit);
    }
}
/// Returns native partial records on failure; inspect reader.error().
pub fn decode_facts_creations(reader: &mut NativeFactsReader<'_>) -> Vec<FactsEquipmentCreation> {
    let count = reader.count(20);
    let mut out = Vec::new();
    let mut timestamp = 0u64;
    for _ in 0..count {
        if reader.error().is_some() {
            break;
        }
        timestamp = timestamp.wrapping_add(reader.unsigned());
        let mut c = FactsEquipmentCreation {
            timestamp_us: timestamp,
            slot: reader.unsigned() as u32,
            generation: reader.unsigned() as u32,
            chunk: reader.signed(),
            packet_index: reader.signed(),
            bit_pos: reader.signed(),
            has_ref: reader.boolean(),
            reference: reader.unsigned() as u32,
            has_id: reader.boolean(),
            ability_id: reader.unsigned() as u32,
            ..Default::default()
        };
        for i in 0..4 {
            c.mpp_present[i] = reader.boolean();
            c.mpp_val[i] = reader.unsigned();
        }
        c.position = [reader.float32(), reader.float32(), reader.float32()];
        let count = reader.unsigned() as i64;
        if count > 0 {
            let mut mask = Vec::new();
            for _ in 0..count {
                if reader.error().is_some() {
                    break;
                }
                mask.push(reader.signed());
            }
            c.mask = Some(mask);
        }
        c.mask_full = reader.boolean();
        c.mask_has_i0 = reader.boolean();
        c.default_state_bits = reader.signed();
        c.has_ammo = reader.boolean();
        c.ammo.mag = reader.unsigned() as u32;
        c.ammo.res = reader.unsigned() as u32;
        c.after_bit = reader.signed();
        out.push(c);
    }
    out
}
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FactsCreationStats {
    pub slots: i64,
    pub anchors: i64,
    pub overflow: i64,
    pub mask_bad: i64,
    pub pos_bad: i64,
    pub accepted: i64,
    pub mask_sparse: i64,
    pub mask_full: i64,
    #[serde(rename = "NoI0")]
    pub no_i0: i64,
    pub with_ref: i64,
    #[serde(rename = "WithID")]
    pub with_id: i64,
    pub with_ammo: i64,
}
pub fn encode_facts_creation_stats(writer: &mut NativeFactsWriter, s: &FactsCreationStats) {
    for v in [
        s.slots,
        s.anchors,
        s.overflow,
        s.mask_bad,
        s.pos_bad,
        s.accepted,
        s.mask_sparse,
        s.mask_full,
        s.no_i0,
        s.with_ref,
        s.with_id,
        s.with_ammo,
    ] {
        writer.signed(v);
    }
}
pub fn decode_facts_creation_stats(reader: &mut NativeFactsReader<'_>) -> FactsCreationStats {
    FactsCreationStats {
        slots: reader.signed(),
        anchors: reader.signed(),
        overflow: reader.signed(),
        mask_bad: reader.signed(),
        pos_bad: reader.signed(),
        accepted: reader.signed(),
        mask_sparse: reader.signed(),
        mask_full: reader.signed(),
        no_i0: reader.signed(),
        with_ref: reader.signed(),
        with_id: reader.signed(),
        with_ammo: reader.signed(),
    }
}
