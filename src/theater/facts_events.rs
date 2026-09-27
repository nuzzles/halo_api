//! Event section of the derived facts cache. This retains the cache projection,
//! not all diagnostics and identities present in native recording records.
use super::{
    FactsProjectileTrack, NativeFactsReader, NativeFactsWriter, decode_facts_tracks,
    encode_facts_tracks,
};
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsFireEvent {
    pub timestamp_us: u64,
    pub film_index: i64,
    pub weapon_id: u64,
    pub has_aim: bool,
    pub aim: [f32; 3],
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FactsLoadout {
    pub timestamp_us: u64,
    pub slot: u32,
    pub families: Option<Vec<u32>>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FactsGrenadeThrow {
    pub timestamp_us: u64,
    pub film_index: i64,
    pub type_id: u32,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsEvents {
    pub fire: Vec<FactsFireEvent>,
    pub loadouts: Vec<FactsLoadout>,
    pub grenades: Vec<FactsGrenadeThrow>,
    pub projectiles: Vec<FactsProjectileTrack>,
}
/// Native make(slice,0,n) panics for negative capacities. Refuse those safely,
/// and avoid eager allocation for untrusted positive counts.
pub(super) fn facts_slice_count(r: &mut NativeFactsReader<'_>) -> i64 {
    let n = r.unsigned() as i64;
    if n < 0 {
        r.fail(format!(
            "invalid signed facts count {n} at offset {}",
            r.offset()
        ));
    }
    n
}
pub fn encode_facts_events(w: &mut NativeFactsWriter, g: &FactsEvents) {
    w.unsigned(g.fire.len() as u64);
    let mut last = 0u64;
    for e in &g.fire {
        w.unsigned(e.timestamp_us.wrapping_sub(last));
        last = e.timestamp_us;
        w.signed(e.film_index);
        w.unsigned(e.weapon_id);
        w.boolean(e.has_aim);
        if e.has_aim {
            for a in e.aim {
                w.float32(a);
            }
        }
    }
    w.unsigned(g.loadouts.len() as u64);
    for l in &g.loadouts {
        w.unsigned(l.timestamp_us);
        w.unsigned(u64::from(l.slot));
        let families = l.families.as_deref().unwrap_or(&[]);
        w.unsigned(families.len() as u64);
        for &f in families {
            w.unsigned(u64::from(f));
        }
    }
    w.unsigned(g.grenades.len() as u64);
    for e in &g.grenades {
        w.unsigned(e.timestamp_us);
        w.signed(e.film_index);
        w.unsigned(u64::from(e.type_id));
    }
    encode_facts_tracks(w, &g.projectiles);
}
pub fn decode_facts_events(r: &mut NativeFactsReader<'_>) -> FactsEvents {
    let mut g = FactsEvents::default();
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        let mut e = FactsFireEvent {
            timestamp_us: last,
            film_index: r.signed(),
            weapon_id: r.unsigned(),
            has_aim: r.boolean(),
            aim: [0.; 3],
        };
        if e.has_aim {
            e.aim = std::array::from_fn(|_| r.float32());
        }
        g.fire.push(e);
    }
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        let mut l = FactsLoadout {
            timestamp_us: r.unsigned(),
            slot: r.unsigned() as u32,
            families: None,
        };
        // Native uses a signed loop bound here, not a slice capacity. A negative
        // family count skips the loop and continues reading the following section.
        let n = r.unsigned() as i64;
        for _ in 0..n {
            if r.error().is_some() {
                break;
            }
            l.families
                .get_or_insert_with(Vec::new)
                .push(r.unsigned() as u32);
        }
        g.loadouts.push(l);
    }
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        g.grenades.push(FactsGrenadeThrow {
            timestamp_us: r.unsigned(),
            film_index: r.signed(),
            type_id: r.unsigned() as u32,
        });
    }
    g.projectiles = decode_facts_tracks(r);
    g
}
