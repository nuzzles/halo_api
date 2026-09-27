//! Object-death cache projection. This is derived scan data, not a LegacyFilm replacement.
use super::facts_json_read::{Cell, FactsJsonDecode, Shape};
use super::facts_json_write::{FactsJsonValue, FactsJsonWriter};
use super::{NativeFactsReader, NativeFactsWriter, ObjectDeadState, ObjectDeath};
use std::collections::BTreeMap;
macro_rules! json_shape {
    (FactsMovementProfile,range,$ty:ty) => {
        &Shape::NamedArray(&FactsAxisRange::SHAPE, 3, "profile.Vec3Range")
    };
    (FactsPrecisionDescriptor,index_width,$ty:ty) => {
        &Shape::Unsigned(64, "uint")
    };
    (FactsPrecisionDescriptor,axis_widths,$ty:ty) => {
        &Shape::Array(&Shape::Unsigned(64, "uint"), 3)
    };
    (FactsMovementProfile,delta_axis_width,$ty:ty) => {
        &Shape::Unsigned(64, "uint")
    };
    ($name:ident,$field:ident,$ty:ty) => {
        &<$ty as FactsJsonDecode>::SHAPE
    };
}
macro_rules! data {
    ($name:ident, $go:literal { $($field:ident:$ty:ty  =>  $native:literal),* $(,)? })  =>  {
        #[derive(Debug,Clone,Default,PartialEq)]
        pub struct $name { $(pub $field:$ty,)* }
        impl FactsJsonDecode for $name {
            const SHAPE: Shape = Shape::Struct { name: $go, fields: &[$(($native, json_shape!($name,$field,$ty)),)*] };
            fn from_cell(cell: &Cell) -> Self { Self { $($field: FactsJsonDecode::from_cell(cell.field($native)),)* } }
        }

        impl FactsJsonValue for $name { fn write_json(&self,w:&mut FactsJsonWriter){w.raw(b"{");let mut first=true;$(w.field($native,&self.$field,&mut first);)*w.raw(b"}");} }
    }
}
data!(FactsPrecisionDescriptor, "profile.PrecisionDescriptor" { index_width:u64 => "IndexW",axis_widths:[u64;3] => "AxisW",region:u32 => "Region" });
data!(FactsAxisRange, "profile.AxisRange" { min:f32 => "Min",max:f32 => "Max" });
data!(FactsMovementProfile, "profile.MovementProfile" {
    traversal:FactsPrecisionDescriptor => "Traversal",world_object:FactsPrecisionDescriptor => "WorldObject",
    delta_quantum:f32 => "DeltaQuantum",delta_axis_width:u64 => "DeltaAxisWidth",range:[FactsAxisRange;3] => "Range",
    full_precision:bool => "FullPrecision",delta_has_handle_tail:bool => "DeltaHasHandleTail",calibrated_skip:bool => "CalibratedSkip",mobility_action_extra_bits:i64 => "MobilityActionExtraBits",
});
data!(FactsKeyframeProfile, "profile.KeyframeProfile" { header_bits:i64 => "EnTeteBits",size_word_bits:i64 => "MotDeTailleBits" });
data!(FactsMppWidths, "profile.MPPWidths" { lead:i64 => "Lead",index:i64 => "Index" });
data!(FactsScanGrammar, "grammar.GrammaireBalayage" {
    corruption_check:bool => "ControleDeCorruption",new_record_tail_bits:i64 => "BitsDeQueueRecordNew",default_state_by_archetype:bool => "DeserEtatParArchetype",
    simulation_complete:bool => "SimStateComplet",baseline_scope:bool => "PorteeBaseline",writer_absolute:bool => "GrammaireEcrivainI0",mobility_action_body:bool => "CorpsActionMobilite",
    ability_anchor_body:bool => "CorpsAncrageCapacite",chain_inference:bool => "InferenceChaine",calibrated_widths:Option<BTreeMap<Vec<u8>,i64>> => "LargeursCalibrees",
    generation_strict:bool => "GenerationStricte",view_tables:bool => "TablesParVue",view_classes:bool => "ClassesDeVue",stub_widths:Option<BTreeMap<Vec<u8>,i64>> => "LargeursBouchon",
});
data!(FactsScanProfile, "grammar.ProfilDeBalayage" { movement:FactsMovementProfile => "Mouvement",keyframe:FactsKeyframeProfile => "Cadre",mpp:FactsMppWidths => "MPP",grammar:FactsScanGrammar => "Grammaire" });
data!(FactsDeathFrame, "replay.cadreSerialisable" {
    has_extra_fields:bool => "HasExtraFields",id_low_bits:i64 => "IDLowBits",id_base:u32 => "IDBase",new_default_state_bits:i64 => "NewDefaultStateBits",packet_preamble_bits:i64 => "PacketPreambleBits",profile:FactsScanProfile => "Profil",
});
data!(FactsObjectDeathStats, "replay.statsSansCadre" {
    default_frame:bool => "CadreParDefaut",frame_located:i64 => "CadreLocalises",frame_runner:i64 => "CadreDauphin",frame_events:i64 => "CadreEvenements",keyframes:i64 => "Keyframes",deltas:i64 => "Deltas",
    packets:i64 => "Packets",event_packets:i64 => "EventPackets",located_packets:i64 => "LocatedPackets",records:Option<BTreeMap<u32,i64>> => "Records",clean_records:Option<BTreeMap<u32,i64>> => "CleanRecords",
    mask_declared:Option<BTreeMap<u32,i64>> => "MaskDeclared",mask_declared_desync:Option<BTreeMap<u32,i64>> => "MaskDeclaredDesync",
});
data!(FactsObjectDeaths, "replay.chargeDesMortsDObjet" { deaths:Option<Vec<ObjectDeath>> => "Morts",stats:FactsObjectDeathStats => "Stats",frame:FactsDeathFrame => "Cadre" });
macro_rules! project { ($name:ty { $($field:ident => $native:literal),* $(,)? }) => {
    impl FactsJsonValue for $name { fn write_json(&self,w:&mut FactsJsonWriter){w.raw(b"{");let mut first=true;$(w.field($native,&self.$field,&mut first);)*w.raw(b"}");} }
}; }
project!(ObjectDeath { timestamp_us => "TimestampUS",slot => "Slot",r#gen => "Gen",type_index => "TypeIndex",dead => "Dead",tail_desync => "TailDesync" });
project!(ObjectDeadState { mort => "Mort",enum_a => "EnumA",enum_b => "EnumB",val_0c => "Val0c",val_0e => "Val0e",has_ref => "HasRef",gid_present => "GIDPresent",global_id => "GlobalID",val14 => "Val14",val18 => "Val18",src_tag0 => "SrcTag0",src_tag4c => "SrcTag4c" });
/// Native JSON charge with exact declaration order and Go map/string/float rules.
/// Observation callbacks are deliberately absent from this persisted projection.
pub fn encode_facts_object_deaths(w: &mut NativeFactsWriter, value: &FactsObjectDeaths) {
    let mut json = FactsJsonWriter::default();
    value.write_json(&mut json);
    if let Some(error) = json.error {
        w.replace_error(format!("morts d objet non serialisables : {error}"));
        w.unsigned(0);
        return;
    }
    w.string_bytes(&json.bytes);
}

macro_rules! existing_schema {
    ($name:ident,$go:literal{$($field:ident:$ty:ty => $native:literal),*$(,)?})=>{
        impl FactsJsonDecode for $name {
            const SHAPE:Shape=Shape::Struct{name:$go,fields:&[$(($native,&<$ty as FactsJsonDecode>::SHAPE),)*]};
            fn from_cell(cell:&Cell)->Self{Self{$($field:FactsJsonDecode::from_cell(cell.field($native)),)*}}
        }
    };
}
existing_schema!(ObjectDeath,"types.ObjectDeath"{timestamp_us:u64 => "TimestampUS",slot:u32 => "Slot",r#gen:u32 => "Gen",type_index:u32 => "TypeIndex",dead:ObjectDeadState => "Dead",tail_desync:bool => "TailDesync"});
existing_schema!(ObjectDeadState,"types.DeadState"{mort:bool => "Mort",enum_a:i32 => "EnumA",enum_b:i32 => "EnumB",val_0c:u8 => "Val0c",val_0e:u8 => "Val0e",has_ref:bool => "HasRef",gid_present:bool => "GIDPresent",global_id:u32 => "GlobalID",val14:u8 => "Val14",val18:i8 => "Val18",src_tag0:u32 => "SrcTag0",src_tag4c:u32 => "SrcTag4c"});
/// Read the native length-prefixed JSON charge. An invalid charge returns the
/// native zero projection and retains its framing or JSON diagnostic.
pub fn decode_facts_object_deaths(r: &mut NativeFactsReader<'_>) -> FactsObjectDeaths {
    let length = r.unsigned() as i64;
    let Some(charge) = r.section(length) else {
        return FactsObjectDeaths::default();
    };
    if charge.is_empty() {
        return FactsObjectDeaths::default();
    }
    match super::facts_json_read::decode(charge) {
        Ok(value) => value,
        Err(error) => {
            r.replace_error(format!("morts d objet : {error}"));
            FactsObjectDeaths::default()
        }
    }
}
