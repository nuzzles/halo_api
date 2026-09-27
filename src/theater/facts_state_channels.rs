//! Delta, ability and movement projections in the native FilmFacts cache.
//! Native machine words remain 64-bit on WASM. Presence flags do not clear data.
use super::facts_events::facts_slice_count;
use super::{NativeFactsReader, NativeFactsWriter};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsAbilityRank {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(rename = "Rank")]
    pub rank: i64,
}
fn encode_ability_rank(w: &mut NativeFactsWriter, values: &[FactsAbilityRank]) {
    w.unsigned(values.len() as u64);
    let mut last = 0u64;
    for v in values {
        w.unsigned(v.timestamp_us.wrapping_sub(last));
        last = v.timestamp_us;
        w.unsigned(v.slot as u64);
        w.signed(v.rank);
    }
}
fn decode_ability_rank(r: &mut NativeFactsReader<'_>) -> Vec<FactsAbilityRank> {
    let mut out = Vec::new();
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        out.push(FactsAbilityRank {
            timestamp_us: last,
            slot: r.unsigned() as u32,
            rank: r.signed(),
        });
    }
    out
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsCamoState {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(rename = "Q")]
    pub quantum: u16,
}
fn encode_camo_state(w: &mut NativeFactsWriter, values: &[FactsCamoState]) {
    w.unsigned(values.len() as u64);
    let mut last = 0u64;
    for v in values {
        w.unsigned(v.timestamp_us.wrapping_sub(last));
        last = v.timestamp_us;
        w.unsigned(v.slot as u64);
        w.unsigned(v.quantum as u64);
    }
}
fn decode_camo_state(r: &mut NativeFactsReader<'_>) -> Vec<FactsCamoState> {
    let mut out = Vec::new();
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        out.push(FactsCamoState {
            timestamp_us: last,
            slot: r.unsigned() as u32,
            quantum: r.unsigned() as u16,
        });
    }
    out
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsGrappleRead {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(rename = "Heavy")]
    pub heavy: bool,
    #[serde(rename = "PosQ")]
    pub position_quanta: [u32; 3],
}
fn encode_grapple_read(w: &mut NativeFactsWriter, values: &[FactsGrappleRead]) {
    w.unsigned(values.len() as u64);
    let mut last = 0u64;
    for v in values {
        w.unsigned(v.timestamp_us.wrapping_sub(last));
        last = v.timestamp_us;
        w.unsigned(v.slot as u64);
        w.boolean(v.heavy);
        for a in v.position_quanta {
            w.unsigned(a as u64);
        }
    }
}
fn decode_grapple_read(r: &mut NativeFactsReader<'_>) -> Vec<FactsGrappleRead> {
    let mut out = Vec::new();
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        out.push(FactsGrappleRead {
            timestamp_us: last,
            slot: r.unsigned() as u32,
            heavy: r.boolean(),
            position_quanta: std::array::from_fn(|_| r.unsigned() as u32),
        });
    }
    out
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsTranslocation {
    pub timestamp_us: u64,
    pub slot: u32,
    pub has_positions: bool,
    pub from: [f32; 3],
    pub to: [f32; 3],
}
fn encode_translocation(w: &mut NativeFactsWriter, values: &[FactsTranslocation]) {
    w.unsigned(values.len() as u64);
    let mut last = 0u64;
    for v in values {
        w.unsigned(v.timestamp_us.wrapping_sub(last));
        last = v.timestamp_us;
        w.unsigned(v.slot as u64);
        w.boolean(v.has_positions);
        for a in v.from {
            w.float32(a);
        }
        for a in v.to {
            w.float32(a);
        }
    }
}
fn decode_translocation(r: &mut NativeFactsReader<'_>) -> Vec<FactsTranslocation> {
    let mut out = Vec::new();
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        out.push(FactsTranslocation {
            timestamp_us: last,
            slot: r.unsigned() as u32,
            has_positions: r.boolean(),
            from: std::array::from_fn(|_| r.float32()),
            to: std::array::from_fn(|_| r.float32()),
        });
    }
    out
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsAbilityImpulse {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(rename = "Predicted")]
    pub predicted: bool,
}
fn encode_ability_impulse(w: &mut NativeFactsWriter, values: &[FactsAbilityImpulse]) {
    w.unsigned(values.len() as u64);
    let mut last = 0u64;
    for v in values {
        w.unsigned(v.timestamp_us.wrapping_sub(last));
        last = v.timestamp_us;
        w.unsigned(v.slot as u64);
        w.boolean(v.predicted);
    }
}
fn decode_ability_impulse(r: &mut NativeFactsReader<'_>) -> Vec<FactsAbilityImpulse> {
    let mut out = Vec::new();
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        out.push(FactsAbilityImpulse {
            timestamp_us: last,
            slot: r.unsigned() as u32,
            predicted: r.boolean(),
        });
    }
    out
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsAbilityCharge {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(rename = "Emplacement")]
    pub emplacement: i64,
    #[serde(rename = "Charges")]
    pub charges: i64,
    #[serde(rename = "Low")]
    pub low: i64,
}
fn encode_ability_charge(w: &mut NativeFactsWriter, values: &[FactsAbilityCharge]) {
    w.unsigned(values.len() as u64);
    let mut last = 0u64;
    for v in values {
        w.unsigned(v.timestamp_us.wrapping_sub(last));
        last = v.timestamp_us;
        w.unsigned(v.slot as u64);
        w.unsigned(v.emplacement as u64);
        w.unsigned(v.charges as u64);
        w.unsigned(v.low as u64);
    }
}
fn decode_ability_charge(r: &mut NativeFactsReader<'_>) -> Vec<FactsAbilityCharge> {
    let mut out = Vec::new();
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        out.push(FactsAbilityCharge {
            timestamp_us: last,
            slot: r.unsigned() as u32,
            emplacement: r.unsigned() as i64,
            charges: r.unsigned() as i64,
            low: r.unsigned() as i64,
        });
    }
    out
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsMovementState {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(rename = "Kind")]
    pub kind: Vec<u8>,
    #[serde(rename = "On")]
    pub on: bool,
    #[serde(rename = "Progress")]
    pub progress: u32,
    #[serde(rename = "Chunk")]
    pub chunk: i64,
    #[serde(rename = "PacketIndex")]
    pub packet_index: i64,
}
fn encode_movement_state(w: &mut NativeFactsWriter, values: &[FactsMovementState]) {
    w.unsigned(values.len() as u64);
    let mut last = 0u64;
    for v in values {
        w.unsigned(v.timestamp_us.wrapping_sub(last));
        last = v.timestamp_us;
        w.unsigned(v.slot as u64);
        w.string_bytes(&v.kind);
        w.boolean(v.on);
        w.unsigned(v.progress as u64);
        w.unsigned(v.chunk as u64);
        w.unsigned(v.packet_index as u64);
    }
}
fn decode_movement_state(r: &mut NativeFactsReader<'_>) -> Vec<FactsMovementState> {
    let mut out = Vec::new();
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        out.push(FactsMovementState {
            timestamp_us: last,
            slot: r.unsigned() as u32,
            kind: r.string_bytes().to_vec(),
            on: r.boolean(),
            progress: r.unsigned() as u32,
            chunk: r.unsigned() as i64,
            packet_index: r.unsigned() as i64,
        });
    }
    out
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsAbilityImpulseStats {
    #[serde(rename = "Records")]
    pub records: i64,
    #[serde(rename = "WithI57")]
    pub with_i57: i64,
    #[serde(rename = "WithI59")]
    pub with_i59: i64,
    #[serde(rename = "Read")]
    pub read: i64,
    #[serde(rename = "Unread")]
    pub unread: i64,
    #[serde(rename = "Tag1")]
    pub tag1: i64,
    #[serde(rename = "Absent")]
    pub absent: bool,
    #[serde(rename = "Scanned")]
    pub scanned: bool,
}
fn encode_ability_impulse_stats(w: &mut NativeFactsWriter, v: &FactsAbilityImpulseStats) {
    w.unsigned(v.records as u64);
    w.unsigned(v.with_i57 as u64);
    w.unsigned(v.with_i59 as u64);
    w.unsigned(v.read as u64);
    w.unsigned(v.unread as u64);
    w.unsigned(v.tag1 as u64);
    w.boolean(v.absent);
    w.boolean(v.scanned);
}
fn decode_ability_impulse_stats(r: &mut NativeFactsReader<'_>) -> FactsAbilityImpulseStats {
    FactsAbilityImpulseStats {
        records: r.unsigned() as i64,
        with_i57: r.unsigned() as i64,
        with_i59: r.unsigned() as i64,
        read: r.unsigned() as i64,
        unread: r.unsigned() as i64,
        tag1: r.unsigned() as i64,
        absent: r.boolean(),
        scanned: r.boolean(),
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsAbilityChargeStats {
    #[serde(rename = "Records")]
    pub records: i64,
    #[serde(rename = "WithI56")]
    pub with_i56: i64,
    #[serde(rename = "Read")]
    pub read: i64,
    #[serde(rename = "Unread")]
    pub unread: i64,
    #[serde(rename = "Armed")]
    pub armed: i64,
    #[serde(rename = "Absent")]
    pub absent: bool,
    #[serde(rename = "Scanned")]
    pub scanned: bool,
}
fn encode_ability_charge_stats(w: &mut NativeFactsWriter, v: &FactsAbilityChargeStats) {
    w.unsigned(v.records as u64);
    w.unsigned(v.with_i56 as u64);
    w.unsigned(v.read as u64);
    w.unsigned(v.unread as u64);
    w.unsigned(v.armed as u64);
    w.boolean(v.absent);
    w.boolean(v.scanned);
}
fn decode_ability_charge_stats(r: &mut NativeFactsReader<'_>) -> FactsAbilityChargeStats {
    FactsAbilityChargeStats {
        records: r.unsigned() as i64,
        with_i56: r.unsigned() as i64,
        read: r.unsigned() as i64,
        unread: r.unsigned() as i64,
        armed: r.unsigned() as i64,
        absent: r.boolean(),
        scanned: r.boolean(),
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsMovementStats {
    #[serde(rename = "Records")]
    pub records: i64,
    #[serde(rename = "Read")]
    pub read: i64,
    #[serde(rename = "Absent")]
    pub absent: bool,
    #[serde(rename = "Scanned")]
    pub scanned: bool,
    #[serde(rename = "Packets")]
    pub packets: i64,
    #[serde(rename = "EventPackets")]
    pub event_packets: i64,
    #[serde(rename = "EventPacketsLocated")]
    pub event_packets_located: i64,
    #[serde(rename = "EventPacketsUnlocated")]
    pub event_packets_unlocated: i64,
    #[serde(rename = "Desyncs")]
    pub desyncs: i64,
    #[serde(rename = "SlotUnbound")]
    pub slot_unbound: i64,
    #[serde(rename = "Duplicates")]
    pub duplicates: i64,
    #[serde(rename = "MapWidths")]
    pub map_widths: [u64; 3],
}
fn encode_movement_stats(w: &mut NativeFactsWriter, v: &FactsMovementStats) {
    w.unsigned(v.records as u64);
    w.unsigned(v.read as u64);
    w.boolean(v.absent);
    w.boolean(v.scanned);
    w.unsigned(v.packets as u64);
    w.unsigned(v.event_packets as u64);
    w.unsigned(v.event_packets_located as u64);
    w.unsigned(v.event_packets_unlocated as u64);
    w.unsigned(v.desyncs as u64);
    w.unsigned(v.slot_unbound as u64);
    w.unsigned(v.duplicates as u64);
    for a in v.map_widths {
        w.unsigned(a);
    }
}
fn decode_movement_stats(r: &mut NativeFactsReader<'_>) -> FactsMovementStats {
    FactsMovementStats {
        records: r.unsigned() as i64,
        read: r.unsigned() as i64,
        absent: r.boolean(),
        scanned: r.boolean(),
        packets: r.unsigned() as i64,
        event_packets: r.unsigned() as i64,
        event_packets_located: r.unsigned() as i64,
        event_packets_unlocated: r.unsigned() as i64,
        desyncs: r.unsigned() as i64,
        slot_unbound: r.unsigned() as i64,
        duplicates: r.unsigned() as i64,
        map_widths: std::array::from_fn(|_| r.unsigned()),
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsDeltaChannels {
    pub ability_ranks: Vec<FactsAbilityRank>,
    pub camo_states: Vec<FactsCamoState>,
    pub grapple_reads: Vec<FactsGrappleRead>,
    pub translocations: Vec<FactsTranslocation>,
}
pub fn encode_facts_delta_channels(w: &mut NativeFactsWriter, g: &FactsDeltaChannels) {
    encode_ability_rank(w, &g.ability_ranks);
    encode_camo_state(w, &g.camo_states);
    encode_grapple_read(w, &g.grapple_reads);
    encode_translocation(w, &g.translocations);
}
pub fn decode_facts_delta_channels(r: &mut NativeFactsReader<'_>) -> FactsDeltaChannels {
    FactsDeltaChannels {
        ability_ranks: decode_ability_rank(r),
        camo_states: decode_camo_state(r),
        grapple_reads: decode_grapple_read(r),
        translocations: decode_translocation(r),
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsAbilities {
    #[serde(rename = "AbilityImpulses")]
    pub impulses: Vec<FactsAbilityImpulse>,
    #[serde(rename = "AbilityImpulseStats")]
    pub impulse_stats: FactsAbilityImpulseStats,
    #[serde(rename = "AbilityCharges")]
    pub charges: Vec<FactsAbilityCharge>,
    #[serde(rename = "AbilityChargeStats")]
    pub charge_stats: FactsAbilityChargeStats,
}
pub fn encode_facts_abilities(w: &mut NativeFactsWriter, g: &FactsAbilities) {
    encode_ability_impulse(w, &g.impulses);
    encode_ability_impulse_stats(w, &g.impulse_stats);
    encode_ability_charge(w, &g.charges);
    encode_ability_charge_stats(w, &g.charge_stats);
}
pub fn decode_facts_abilities(r: &mut NativeFactsReader<'_>) -> FactsAbilities {
    FactsAbilities {
        impulses: decode_ability_impulse(r),
        impulse_stats: decode_ability_impulse_stats(r),
        charges: decode_ability_charge(r),
        charge_stats: decode_ability_charge_stats(r),
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactsMovement {
    #[serde(rename = "MovementStates")]
    pub states: Vec<FactsMovementState>,
    #[serde(rename = "MovementStateStats")]
    pub stats: FactsMovementStats,
}
pub fn encode_facts_movement(w: &mut NativeFactsWriter, g: &FactsMovement) {
    encode_movement_state(w, &g.states);
    encode_movement_stats(w, &g.stats);
}
pub fn decode_facts_movement(r: &mut NativeFactsReader<'_>) -> FactsMovement {
    FactsMovement {
        states: decode_movement_state(r),
        stats: decode_movement_stats(r),
    }
}
