//! Always-scanned channel projections in the native derived facts cache.
//! Omitted scan diagnostics belong to the richer decoder output, not this cache.
use super::{NativeFactsReader, NativeFactsWriter, VehicleOccupancy};
use serde::{Deserialize, Serialize};
macro_rules! write_field {
    ($w:ident,$v:expr,unsigned) => {
        $w.unsigned($v as u64)
    };
    ($w:ident,$v:expr,signed) => {
        $w.signed($v as i64)
    };
    ($w:ident,$v:expr,byte) => {
        $w.byte($v)
    };
    ($w:ident,$v:expr,boolean) => {
        $w.boolean($v)
    };
    ($w:ident,$v:expr,bytes) => {
        $w.string_bytes(&$v)
    };
}
macro_rules! read_field {
    ($r:ident,unsigned,$t:ty) => {
        $r.unsigned() as $t
    };
    ($r:ident,signed,$t:ty) => {
        $r.signed() as $t
    };
    ($r:ident,byte,$t:ty) => {
        $r.byte()
    };
    ($r:ident,boolean,$t:ty) => {
        $r.boolean()
    };
    ($r:ident,bytes,$t:ty) => {
        $r.string_bytes().to_vec()
    };
}
// Avoid native eager allocations from untrusted counts. Negative native slice
// capacities are refused at the same cursor instead of reproducing a panic.
fn count(r: &mut NativeFactsReader<'_>) -> i64 {
    let n = r.unsigned() as i64;
    if n < 0 {
        r.fail(format!(
            "invalid signed channel count {n} at offset {}",
            r.offset()
        ));
    }
    n
}
macro_rules! codec {
 ($name:ident,$encode:ident,$decode:ident{$($field:ident:$t:ty => $wire:ident),*$(,)?})=>{
  pub fn $encode(w:&mut NativeFactsWriter,values:&[$name]){w.unsigned(values.len() as u64);let mut last=0u64;for v in values{w.unsigned(v.timestamp_us.wrapping_sub(last));last=v.timestamp_us;$(write_field!(w,v.$field,$wire);)*}}
  pub fn $decode(r:&mut NativeFactsReader<'_>)->Vec<$name>{let mut out=Vec::new();let mut last=0u64;for _ in 0..count(r){if r.error().is_some(){break;}last=last.wrapping_add(r.unsigned());out.push($name{timestamp_us:last,$($field:read_field!(r,$wire,$t),)*});}out}
 };
}
macro_rules! channel {
 ($name:ident,$encode:ident,$decode:ident{$($field:ident:$t:ty => $wire:ident $json:literal),*$(,)?})=>{
  #[derive(Debug,Clone,Default,PartialEq,Eq,Serialize,Deserialize)]
  pub struct $name{#[serde(rename="TimestampUS")]pub timestamp_us:u64,$(#[serde(rename=$json)]pub $field:$t,)*}
  codec!($name,$encode,$decode{$($field:$t => $wire,)*});
 };
}
channel!(FactsBipedCreation,encode_facts_biped_creations,decode_facts_biped_creations{
 slot:u32 => unsigned "Slot",generation:u32 => unsigned "Generation",has_index:bool => boolean "HasIndex",participant_index:u32 => unsigned "ParticipantIndex",
});
channel!(FactsWeaponChange,encode_facts_weapon_changes,decode_facts_weapon_changes{
 slot:u32 => unsigned "Slot",slot_index:i64 => signed "SlotIndex",family:u32 => unsigned "Family",previous:u32 => unsigned "Previous",kind:Vec<u8> => bytes "Kind",
});
channel!(FactsPickup,encode_facts_pickup_records,decode_facts_pickup_records{
 slot:u32 => unsigned "Slot",catalog_id:u32 => unsigned "CatalogID",class:u8 => byte "Class",
});
channel!(FactsEquipmentChange,encode_facts_equipment_change_records,decode_facts_equipment_change_records{
 slot:u32 => unsigned "Slot",counter:u32 => unsigned "Counter",rank:i64 => signed "Rank",previous:i64 => signed "Previous",kind:Vec<u8> => bytes "Kind",recovered:bool => boolean "Recovered",gap:i64 => signed "Gap",
});
channel!(FactsZoomEvent,encode_facts_zoom_events,decode_facts_zoom_events{
 slot:u32 => unsigned "Slot",level:i64 => signed "Level",
});
channel!(FactsVehicleEvent,encode_facts_vehicle_events,decode_facts_vehicle_events{
 kind:i64 => signed "Kind",occupant_present:bool => boolean "OccupantPresent",occupant_sonde:i64 => signed "OccupantSonde",occupant_slot:u32 => unsigned "OccupantSlot",occupant_in_band:bool => boolean "OccupantInBand",vehicle_slot:u32 => unsigned "VehicleSlot",vehicle_slot_valid:bool => boolean "VehicleSlotValid",vehicle_gen:u32 => unsigned "VehicleGen",seat:u32 => unsigned "Seat",seat_valid:bool => boolean "SeatValid",
});
channel!(FactsVehicleAim,encode_facts_vehicle_aims,decode_facts_vehicle_aims{
 slot:u32 => unsigned "Slot",yaw_raw:u32 => unsigned "YawRaw",pitch_raw:u32 => unsigned "PitchRaw",
});
codec!(VehicleOccupancy,encode_facts_vehicle_occupancy,decode_facts_vehicle_occupancy{
 slot:u32 => unsigned,r#gen:u32 => unsigned,attached:bool => boolean,parent_slot:u32 => unsigned,parent_gen:u32 => unsigned,has_seat:bool => boolean,seat:u32 => unsigned,
});
macro_rules! counters {
 ($name:ident,$encode:ident,$decode:ident{$($field:ident => $json:literal),*$(,)?})=>{
  #[derive(Debug,Clone,Default,PartialEq,Eq,Serialize,Deserialize)]
  pub struct $name{$(#[serde(rename=$json)]pub $field:i64,)*}
  fn $encode(w:&mut NativeFactsWriter,s:&$name){$(w.unsigned(s.$field as u64);)*}
  fn $decode(r:&mut NativeFactsReader<'_>)->$name{$name{$($field:r.unsigned() as i64,)*}}
 };
}
counters!(FactsPickupStats,encode_pickup_stats,decode_pickup_stats{
 packets => "Packets",type9 => "Type9",type8 => "Type8",other_type => "OtherType",published => "Published",multi_event => "MultiEvent",refused_no_ref => "RefusedNoRef",refused_no_catalog => "RefusedNoCatalog",refused_off_band => "RefusedOffBand",unexpected_wide_ref => "UnexpectedWideRef",
});
counters!(FactsAbilityRankStats,encode_rank_stats,decode_rank_stats{
 records => "Records",with_i48 => "WithI48",read => "Read",unread => "Unread",gated => "Gated",
});
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FactsEquipmentChangeStats {
    pub walk: FactsAbilityRankStats,
    pub lives: i64,
    pub repeats: i64,
    pub counter_jumps: i64,
    pub missed_estimate: i64,
    pub lives_first_off_spec: i64,
    pub spawned: i64,
    pub taken: i64,
    pub spent: i64,
    pub recovered: i64,
}
pub fn encode_facts_pickups(
    w: &mut NativeFactsWriter,
    values: &[FactsPickup],
    stats: &FactsPickupStats,
) {
    encode_facts_pickup_records(w, values);
    encode_pickup_stats(w, stats);
}
pub fn decode_facts_pickups(r: &mut NativeFactsReader<'_>) -> (Vec<FactsPickup>, FactsPickupStats) {
    (decode_facts_pickup_records(r), decode_pickup_stats(r))
}
pub fn encode_facts_equipment_changes(
    w: &mut NativeFactsWriter,
    values: &[FactsEquipmentChange],
    s: &FactsEquipmentChangeStats,
) {
    encode_facts_equipment_change_records(w, values);
    encode_rank_stats(w, &s.walk);
    for v in [
        s.lives,
        s.repeats,
        s.counter_jumps,
        s.missed_estimate,
        s.lives_first_off_spec,
        s.spawned,
        s.taken,
        s.spent,
        s.recovered,
    ] {
        w.unsigned(v as u64);
    }
}
pub fn decode_facts_equipment_changes(
    r: &mut NativeFactsReader<'_>,
) -> (Vec<FactsEquipmentChange>, FactsEquipmentChangeStats) {
    (
        decode_facts_equipment_change_records(r),
        FactsEquipmentChangeStats {
            walk: decode_rank_stats(r),
            lives: r.unsigned() as i64,
            repeats: r.unsigned() as i64,
            counter_jumps: r.unsigned() as i64,
            missed_estimate: r.unsigned() as i64,
            lives_first_off_spec: r.unsigned() as i64,
            spawned: r.unsigned() as i64,
            taken: r.unsigned() as i64,
            spent: r.unsigned() as i64,
            recovered: r.unsigned() as i64,
        },
    )
}
