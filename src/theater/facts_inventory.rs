//! Inventory cache fields; presence flags stay distinct from recorded counters.
use super::facts_events::facts_slice_count;
use super::{
    KeyframeSlotAmmo, NativeFactsReader, NativeFactsWriter, decode_facts_ammo, encode_facts_ammo,
};
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKeyframeInventory {
    pub timestamp_us: u64,
    pub slot: u32,
    pub grenades_read: bool,
    pub grenades: [u32; 4],
    pub selected_grenade_rank: i64,
    pub ability_rank: i64,
    pub drawn_slot: i64,
    pub ammo_candidates: i64,
    pub ammo_read: bool,
    pub ammo: [KeyframeSlotAmmo; 4],
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FactsInventoryDelta {
    pub timestamp_us: u64,
    pub slot: u32,
    pub grenades: Option<Vec<u32>>,
    pub selection_read: bool,
    pub selection: i64,
    pub mask: u32,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsInventory {
    pub inventory: Vec<FactsKeyframeInventory>,
    pub deltas: Vec<FactsInventoryDelta>,
}
pub fn encode_facts_inventory(w: &mut NativeFactsWriter, g: &FactsInventory) {
    w.unsigned(g.inventory.len() as u64);
    for inv in &g.inventory {
        w.unsigned(inv.timestamp_us);
        w.unsigned(u64::from(inv.slot));
        w.boolean(inv.grenades_read);
        for c in inv.grenades {
            w.unsigned(u64::from(c));
        }
        w.signed(inv.selected_grenade_rank);
        w.signed(inv.ability_rank);
        w.signed(inv.drawn_slot);
        w.unsigned(inv.ammo_candidates as u64);
        w.boolean(inv.ammo_read);
        for a in &inv.ammo {
            encode_facts_ammo(w, a);
        }
    }
    w.unsigned(g.deltas.len() as u64);
    let mut last = 0u64;
    for d in &g.deltas {
        w.unsigned(d.timestamp_us.wrapping_sub(last));
        last = d.timestamp_us;
        w.unsigned(u64::from(d.slot));
        let grenades = d.grenades.as_deref().unwrap_or(&[]);
        w.unsigned(grenades.len() as u64);
        for &c in grenades {
            w.unsigned(u64::from(c));
        }
        w.boolean(d.selection_read);
        w.signed(d.selection);
        w.unsigned(u64::from(d.mask));
    }
}
pub fn decode_facts_inventory(r: &mut NativeFactsReader<'_>) -> FactsInventory {
    let mut g = FactsInventory::default();
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        g.inventory.push(FactsKeyframeInventory {
            timestamp_us: r.unsigned(),
            slot: r.unsigned() as u32,
            grenades_read: r.boolean(),
            grenades: std::array::from_fn(|_| r.unsigned() as u32),
            selected_grenade_rank: r.signed(),
            ability_rank: r.signed(),
            drawn_slot: r.signed(),
            ammo_candidates: r.unsigned() as i64,
            ammo_read: r.boolean(),
            ammo: std::array::from_fn(|_| decode_facts_ammo(r)),
        });
    }
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        let mut d = FactsInventoryDelta {
            timestamp_us: last,
            slot: r.unsigned() as u32,
            ..Default::default()
        };
        let n = r.unsigned() as i64;
        if n > 0 {
            // Native allocates an empty (non-nil) slice even if reading the first
            // element fails. Negative counts do not enter this conditional.
            d.grenades = Some(Vec::new());
            for _ in 0..n {
                if r.error().is_some() {
                    break;
                }
                d.grenades.as_mut().unwrap().push(r.unsigned() as u32);
            }
        }
        d.selection_read = r.boolean();
        d.selection = r.signed();
        d.mask = r.unsigned() as u32;
        g.deltas.push(d);
    }
    g
}
