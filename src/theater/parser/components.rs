//! Component value readers ported from LevelUp. See `docs/CREDIT.md`.
//! Every primitive read is retained, including unnamed flags and raw quantized values.

mod cursor;
use super::{NativeUnitReference, NativeUnitReferenceKind};
use cursor::ComponentCursor as Cursor;
use serde::{Deserialize, Serialize};
mod ability;
mod basic;
mod biped;
mod control;
pub use control::NativeActionBlock;
mod defaults;
mod frames;
mod keyframe_chain;
mod keyframes;
pub use keyframe_chain::*;
mod m4b;
mod movement;
mod navpoint;
mod object;
mod orientation;
mod position;

mod scene;
mod tlv;
mod views;
mod widths;
pub use frames::*;
pub use keyframes::*;
pub(crate) use position::{ComponentBodyPolicy, PositionEncoding};
pub use views::*;
pub(crate) use widths::ComponentWidthOverrides;

/// One scalar from a component, with exact positions in the supplied payload.
/// For native widths above 64, raw is the low 64 bits, matching ReadBits.
/// Additional `<name>.discarded[N]` fields retain all source bits from the
/// discarded prefix. Synthetic zero padding is not expanded into prefix fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentField {
    pub name: String,
    pub bit: i64,
    pub width: u64,
    pub raw: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct NativeComponentWidths<'a> {
    pub movement: &'a super::NativeMovementProfile,
    pub mpp: super::FilmMppWidths,
    pub maximum: u64,
}
struct Reader<'a> {
    native_widths: Option<NativeComponentWidths<'a>>,
    width_error: Option<&'static str>,

    live_grammar: Option<super::NativeScanGrammar>,
    position_capture: Option<super::NativePositionCapture<'a>>,
    position_start: i64,
    position_slot: u32,
    position_fallback: bool,
    movement_slot: Option<u32>,
    references: Vec<NativeUnitReference>,
    diagnostics: super::FilmReadDiagnostics,
    cursor: Cursor<'a>,
    fields: Vec<ComponentField>,
    position_encoding: Option<&'a PositionEncoding>,
}

impl Reader<'_> {
    fn native_position_width(&mut self, raw: u64, field: &'static str) -> Option<u64> {
        self.native_width_limited(raw, field, u64::MAX)
    }

    fn native_width_limited(&mut self, raw: u64, field: &'static str, limit: u64) -> Option<u64> {
        let maximum = self
            .native_widths
            .map_or(u64::MAX, |w| w.maximum)
            .min(limit);
        if raw > maximum {
            self.width_error = Some(field);
            self.diagnostics
                .width_refusals
                .push(super::NativeWidthRefusal {
                    field: field.into(),
                    bit: self.cursor.position,
                    raw_width: raw,
                    maximum,
                });
            return None;
        }
        Some(raw)
    }

    fn publish_component(&mut self, value: super::FilmComponentObservation) {
        self.diagnostics.component_observations.push(value);
    }
    fn publish_mobility(&mut self, flags: [bool; 2]) {
        self.diagnostics.publish_mobility(flags);
    }
    fn record_absolute(&mut self, index: i32) {
        self.diagnostics.absolute(index);
    }

    fn publish_movement(
        &mut self,
        component: crate::theater::parser::NativeMovementComponent,
        values: Vec<u64>,
    ) {
        if let Some(slot) = self.movement_slot {
            self.publish_component(
                crate::theater::parser::FilmComponentObservation::MovementState {
                    component,
                    slot,
                    values,
                },
            );
        }
    }

    fn mask(&mut self) -> Option<u64> {
        if self.bit("mask.dense")? {
            return self.r("mask.bits", 64);
        }
        let count = self.r("mask.count", 3)?;
        let mut mask = 0;
        for i in 0..count {
            mask |= 1 << self.r(&format!("mask.index[{i}]"), 6)?;
        }
        Some(mask)
    }
    fn refuse_read(&mut self, name: &str, width: u64, operation: super::NativeReadOperation) {
        self.diagnostics
            .read_refusals
            .push(super::NativeReadRefusal {
                field: name.into(),
                bit: self.cursor.position,
                width,
                source_bits: self.cursor.source_bits(),
                operation,
            });
    }
    fn guard_source(&mut self, name: &str, width: i64) -> Option<()> {
        if self.cursor.fits_source(width) {
            return Some(());
        }
        self.refuse_read(name, width as u64, super::NativeReadOperation::GroupGuard);
        None
    }
    fn r(&mut self, name: &str, width: usize) -> Option<u64> {
        self.r_wide(name, width as u64)
    }
    fn r_wide(&mut self, name: &str, width: u64) -> Option<u64> {
        let bit = self.cursor.position;
        let mut prefix = self.cursor.detached();
        let Some(raw) = self.cursor.read_wide(width) else {
            self.refuse_read(name, width, super::NativeReadOperation::Scalar);
            return None;
        };
        if width > 64 {
            // Retain information the native numeric accumulator discards.
            // Bound work by actual source size, even for enormous padded reads.
            let mut remaining = (width - 64).min(prefix.remaining_source_bits() as u64);
            let mut index = 0;
            while remaining > 0 {
                let part_width = remaining.min(64);
                let part_bit = prefix.position;
                let part_raw = prefix.read_wide(part_width)?;
                self.fields.push(ComponentField {
                    name: format!("{name}.discarded[{index}]"),
                    bit: part_bit,
                    width: part_width,
                    raw: part_raw,
                });
                remaining -= part_width;
                index += 1;
            }
        }
        self.fields.push(ComponentField {
            name: name.into(),
            bit,
            width,
            raw,
        });
        Some(raw)
    }
    fn bit(&mut self, name: &str) -> Option<bool> {
        Some(self.r(name, 1)? != 0)
    }
    fn gate(&mut self, name: &str, width: usize, polarity: bool) -> Option<()> {
        self.gated_value(name, width, polarity).map(|_| ())
    }
    fn gated_value(&mut self, name: &str, width: usize, polarity: bool) -> Option<Option<u64>> {
        if self.bit(&format!("{name}.gate"))? == polarity {
            Some(Some(self.r(name, width)?))
        } else {
            Some(None)
        }
    }
    fn words(&mut self, name: &str, count: usize, width: usize) -> Option<()> {
        for i in 0..count {
            self.r(&format!("{name}[{i}]"), width)?;
        }
        Some(())
    }
    fn handle(&mut self, name: &str, mut category: u8) -> Option<()> {
        if category == 1 && self.bit(&format!("{name}.category4"))? {
            category = 4;
        }
        let width = match category {
            2 | 3 | 5 => 8,
            4 | 6 => 9,
            _ => 13,
        };
        self.r(&format!("{name}.value"), width)?;
        self.r(&format!("{name}.generation"), 2)?;
        Some(())
    }
    // Same wire shape, used by native inline readers which do not publish UnitRefRead.
    fn inline_optional_handle(&mut self, name: &str, category: u8) -> Option<()> {
        if self.bit(&format!("{name}.present"))? {
            self.handle(name, category)?;
        }
        Some(())
    }
    fn publish_reference(&mut self, reference: NativeUnitReference) {
        self.publish_component(super::FilmComponentObservation::UnitReference {
            reference: reference.clone(),
        });
        self.references.push(reference);
    }
    fn optional_handle(&mut self, name: &str, category: u8) -> Option<()> {
        let start_bit = self.cursor.position;
        let present = self.bit(&format!("{name}.present"))?;
        let (value, tail) = if present {
            self.handle(name, category)?;
            (
                self.fields[self.fields.len() - 2].raw as u32,
                self.fields[self.fields.len() - 1].raw as u32,
            )
        } else {
            (0, 0)
        };
        self.publish_reference(NativeUnitReference {
            kind: NativeUnitReferenceKind::VariableWidth,
            start_bit,
            end_bit: self.cursor.position,
            present,
            value,
            tail,
            probe: category == 1,
        });
        Some(())
    }
    fn optional_word_reference(&mut self, name: &str, emit_absent: bool) -> Option<()> {
        let start_bit = self.cursor.position;
        let present = self.bit(&format!("{name}.gate"))?;
        let value = if present { self.r(name, 32)? as u32 } else { 0 };
        if present || emit_absent {
            self.publish_reference(NativeUnitReference {
                kind: NativeUnitReferenceKind::GatedWord32,
                start_bit,
                end_bit: self.cursor.position,
                present,
                value,
                tail: 0,
                probe: false,
            });
        }
        Some(())
    }
    fn word_reference(&mut self, name: &str) -> Option<()> {
        let start_bit = self.cursor.position;
        let value = self.r(name, 32)? as u32;
        self.publish_reference(NativeUnitReference {
            kind: NativeUnitReferenceKind::Word32,
            start_bit,
            end_bit: self.cursor.position,
            present: true,
            value,
            tail: 0,
            probe: false,
        });
        Some(())
    }
    fn direction(&mut self, scale_width: usize) -> Option<()> {
        if !self.bit("stationary")? {
            self.r("direction", 19)?;
            self.r("magnitude", scale_width)?;
        }
        Some(())
    }
    fn frame_configuration(&mut self) -> Option<()> {
        if self.bit("present")? {
            self.r("reference", 32)?;
            let count = self.r("count_minus_one", 6)? as usize + 1;
            self.words("elements", count, 1)?;
        }
        for i in 0..3 {
            self.gate(&format!("axis[{i}].index"), 6, false)?;
            self.gate(&format!("axis[{i}].near"), 12, true)?;
            self.gate(&format!("axis[{i}].far"), 12, true)?;
        }
        Some(())
    }
}

fn component(r: &mut Reader<'_>, name: &str, level: u32, archetype: u32) -> Option<bool> {
    match name {
        "biped-control-context" | "biped-control-context-component" => {
            r.r(
                "context",
                if r.position_encoding.is_some_and(|e| e.full_precision) {
                    4
                } else {
                    2
                },
            )?;
            r.bit("flag")?;
        }
        "biped-emp-timer-component" => {
            let quantum = r.r("timer", 8)? as u32;
            r.publish_component(super::FilmComponentObservation::EmpTimer { quantum });
        }
        "biped-malleable-property" | "biped-malleable-property-component" => {
            r.gate("value8", 8, true)?;
            for i in 0..11 {
                r.gate(&format!("property[{i}]"), 12, true)?;
            }
            r.words("flags", 7, 1)?;
            if level > 1 {
                r.bit("level_flag")?;
            }
            r.bit("tail_flag")?;
            let width = r.r("width", 5)? as usize;
            if width > 0 {
                r.r("value", width)?;
            }
        }
        "biped-spartan-ability-malleable-property-component" => {
            r.gate("property[0]", 12, true)?;
            r.gate("value8", 8, true)?;
            for i in 1..4 {
                r.gate(&format!("property[{i}]"), 12, true)?;
            }
            r.words("flags", 2, 1)?;
        }

        "managed-player-team-designator-component" => {
            r.r("team", 4)?;
        }
        "managed-object-networked-splash-message-dynamic-component" => {
            let value = r.r("message", 24)?;
            r.publish_component(crate::theater::parser::FilmComponentObservation::Probe {
                archetype,
                component: crate::theater::parser::NativeProbeComponent::SplashDynamic,
                values: vec![value],
            });
        }
        "managed-object-property-component" | "managed-object-player-masked-property-component" => {
            let tag = r.r("tag", 4)?;
            let mode_a = name == "managed-object-property-component";
            let width = match (mode_a, tag) {
                (true, 1) | (false, 11..=15) => 4,
                (true, 2) | (false, 10) => 1,
                (true, 3) | (false, 7) => 24,
                (true, 4..=6) | (false, 8..=9) => 32,
                _ => 0,
            };
            let mut values = vec![tag];
            if width != 0 {
                values.push(r.r("value", width)?);
            }
            let field = if mode_a {
                super::NativeManagedPropertyField::Scalar
            } else {
                super::NativeManagedPropertyField::PerPlayer
            };
            r.publish_component(super::FilmComponentObservation::ManagedProperty { field, values });
        }
        "flock-relevancy-component" => {
            r.r("relevancy", 8)?;
        }
        "flock-fleeing-component" => {
            r.bit("fleeing")?;
        }
        "flock-remembered-danger-component" => {
            let kind = r.r("kind", 3)?;
            r.r("value", 8)?;
            if kind != 0 {
                r.r("direction", 19)?;
            }
        }
        "flock-destination-component" => {
            r.bit("flag")?;
            if !r.bit("default_vector")? {
                r.gate("region", 1, false)?;
                r.words("position", 3, (u64::from(level) + 6).min(26) as usize)?;
            }
            if level > 1 {
                r.r("tail", 2)?;
            }
        }
        "game-engine-screen-sequence-component" => {
            r.r("sequence", 4)?;
            r.r("value", 8)?;
        }
        "object-position-component" => {
            if r.bit("high_precision")? {
                r.r("high_precision_body", 59)?;
            } else {
                let Some(encoding) = r.position_encoding else {
                    return Some(false);
                };
                let widths = if let Some(raw) = r.native_widths {
                    raw.movement.world_object.axis_bits
                } else if let Some(widths) = encoding.world_axis_bits {
                    widths.map(|w| w as u64)
                } else {
                    return Some(false);
                };
                if !r.bit("region.gate")? {
                    let raw = r.native_widths.map_or(encoding.index_bits as u64, |w| {
                        w.movement.world_object.index_bits
                    });
                    let width = r.native_position_width(raw, "world index")?;
                    r.r_wide("region", width)?;
                }
                for (i, raw) in widths.into_iter().enumerate() {
                    let width = r.native_position_width(raw, "world axes")?;
                    r.r_wide(&format!("position[{i}]"), width)?;
                }
                r.r("finite", 2)?;
            }
        }

        "biped-slide-component" | "biped-slide" => return movement::slide(r, level),
        "biped-posture-physics-component" => return movement::posture(r),
        "biped-mobility-action-component" | "biped-mobility-action" => return ability::mobility(r),
        "biped-spartan-ability-component" => {
            return ability::predicted(r);
        }
        "biped-spartan-ability-non-predicted-state-component"
        | "biped-spartan-ability-non-predicted-state" => return ability::non_predicted(r, level),
        "tacmap-queuedreplaymission" | "tacmap-backmenu-openoverride" => {
            for i in 0..32 {
                r.gate(&format!("entry[{i}].handle"), 5, false)?;
                if name == "tacmap-queuedreplaymission" {
                    r.r(&format!("entry[{i}].id"), 32)?;
                } else {
                    r.bit(&format!("entry[{i}].flag"))?;
                }
                r.bit(&format!("entry[{i}].tail"))?;
            }
        }
        "tacmap-cooptetherarea" => {
            if r.position_encoding.is_none() {
                return Some(false);
            }
            position::traversal_payload(r)?;
            r.words("extent", 2, 12)?;
        }
        "tacmap-displayasset" => {
            if r.position_encoding.is_none() {
                return Some(false);
            }
            r.words("references", 2, 32)?;
            r.r("kind", 2)?;
            position::traversal_payload(r)?;
            for i in 0..2 {
                r.r(&format!("asset[{i}].id"), 64)?;
                r.r(&format!("asset[{i}].word"), 32)?;
            }
            r.bit("flag")?;
        }
        "tacmap-waypointstate" => {
            if r.position_encoding.is_none() {
                return Some(false);
            }
            r.bit("flag")?;
            r.r("reference", 32)?;
            position::traversal_payload(r)?;
        }
        "generic-rigid-body-transforms" | "generic-rigid-body-transforms-component" => {
            if r.position_encoding.is_none() {
                return Some(false);
            }
            let mask = r.r("mask", 8)?;
            for i in 0..8 {
                if mask & (1 << i) != 0 {
                    r.gate(&format!("body[{i}].direction"), 19, false)?;
                    r.r(&format!("body[{i}].magnitude"), 8)?;
                    position::traversal_payload(r)?;
                }
            }
        }
        "physics-state-component" => {
            r.r("state", 32)?;
            r.gate("extra", 32, true)?;
        }
        "object-forward-and-up-dynamic-precision-component" => orientation::dynamic(r, level)?,
        "object-angular-velocity-dynamic-precision-component" => {
            if r.bit("full_precision")? {
                r.words("vector_bits", 3, 32)?;
            } else {
                r.direction(8)?;
            }
        }
        "simulation-state" | "simulation-state-component" => return orientation::simulation(r),
        "unit-control-component" => {
            let present = r.bit("control.present")?;
            let mut values = vec![u64::from(present), 0, 0, 0];
            if present {
                values[1] = r.r("control.index", 5)?;
                let second = r.gated_value("control.second_index", 6, true)?;
                values[2] = u64::from(second.is_some());
                values[3] = second.unwrap_or(0);
            }
            r.publish_movement(
                crate::theater::parser::NativeMovementComponent::UnitControl,
                values,
            );
            r.optional_word_reference("control.reference", true)?;
        }
        "unit-actor-control-component" => return control::actor_control(r, level),
        "unit-actor-state-component" => control::actor_state(r, level)?,
        "object-position-dynamic-precision-component" => return position::component(r),
        "object-translational-velocity-dynamic-precision-component" => {
            let full = r.bit("full_precision")?;
            let mut values = vec![u64::from(full), 0, 0, 0];
            if full {
                r.words("vector_bits", 3, 32)?;
            } else {
                let absent = r.bit("stationary")?;
                values[1] = u64::from(absent);
                if !absent {
                    values[2] = r.r("direction", 19)?;
                    values[3] = r.r("magnitude", 10)?;
                }
            }
            r.publish_movement(
                crate::theater::parser::NativeMovementComponent::Velocity,
                values,
            );
        }
        "object-translational-velocity-component" => r.direction(10)?,
        "object-angular-velocity-component" => r.direction(8)?,
        "object-region-state-component" => {
            let present = r.bit("has_values")?;
            let count = r.r("count", 6)? as usize;
            r.words("states", count, 3)?;
            if present {
                r.words("values", count, 10)?;
            }
        }
        "object-damage-sections-component" => {
            let count = r.r("count", 6)?;
            for i in 0..count {
                if r.bit(&format!("section[{i}].present"))? {
                    r.r(&format!("section[{i}].amount"), 7)?;
                    r.r(&format!("section[{i}].state"), 16)?;
                }
            }
        }
        "object-constraint-component" => {
            let count = r.r("count", 5)? as usize;
            if count != 0 {
                r.r("flags_a", count)?;
                r.r("flags_b", count)?;
            }
        }
        "object-parent-state-component" => parent(r, level, archetype)?,
        "object-scale-component" => {
            if !r.bit("default")? {
                r.r("scale", 15)?;
                if r.bit("transition")? {
                    r.r("target_scale", 15)?;
                    r.r("duration", 12)?;
                    r.r("flags", 5)?;
                }
            }
        }
        "object-maximum-vitalities-component" => {
            let flags = r.r("flags", 5)?;
            for (mask, name) in [(4, "body"), (8, "shield"), (16, "extra")] {
                if flags & mask != 0 {
                    for i in 0..3 {
                        r.gate(&format!("{name}[{i}]"), 12, true)?;
                    }
                    if mask != 16 {
                        r.bit(&format!("{name}.flag"))?;
                    }
                }
            }
            r.words("tail_flags", 3, 1)?;
        }
        "object-dissolver-component" => {
            if r.r("state", 4)? != 13 {
                r.words("body_words", 3, 32)?;
                r.r("duration", 12)?;
                r.bit("flag")?;
            }
        }
        "object-physics-flags-component" => r.words("flags", 5, 1)?,
        "object-frame-configuration-component" => r.frame_configuration()?,
        "unit-grenade-counts-component" => {
            let count = r.r("count", 3)?;
            let mut values = Vec::new();
            for i in 0..count {
                values.push(r.r(&format!("grenades[{i}]"), 8)?);
            }
            r.publish_component(super::FilmComponentObservation::GrenadeCounts { count, values });
        }
        "unit-equipment-component" => {
            let head = r.r("head", 3)? as u32;
            let count = r.r("count", 3)?;
            let mut entries = Vec::with_capacity(count as usize);
            for i in 0..count {
                r.optional_handle(&format!("equipment[{i}]"), 0)?;
                let reference = r.references.last()?;
                entries.push(super::UnitEquipmentEntry {
                    value: reference.value,
                    tail: reference.tail,
                    present: reference.present,
                });
            }
            r.publish_component(super::FilmComponentObservation::UnitEquipment {
                state: Box::new(super::UnitEquipmentRead { head, entries }),
            });
        }
        "unit-low-frequency-component" => {
            r.bit("head")?;
            r.optional_handle("reference", 0)?;
            r.r("state", 3)?;
            r.bit("flag")?;
            let count = r.r("count", 4)?;
            for i in 0..count {
                r.optional_handle(&format!("references[{i}]"), 0)?;
            }
            r.r("tail", 2)?;
        }
        "unit-command-tick-component" => {
            r.gate("tick", 8, true)?;
            if r.bit("extra")? {
                let single = r.bit("single")?;
                r.gate("tick_a", 8, true)?;
                if !single {
                    r.gate("tick_b", 8, true)?;
                }
                if !r.bit("short_tail")? {
                    r.words("tail", 2, 1)?;
                }
            }
        }
        "unit-stun-component" => {
            r.r("state", 16)?;
            r.r("amount", 12)?;
            r.r("duration", 12)?;
        }
        "unit-crouch-component" => {
            let flag = r.bit("flag")?;
            let progress = r.r("progress", 10)?;
            r.publish_movement(
                crate::theater::parser::NativeMovementComponent::Crouch,
                vec![u64::from(flag), progress],
            );
        }
        "unit-desired-aiming-vector-component" => {
            let single = r.bit("single")?;
            r.r("yaw", 12)?;
            r.r("pitch", 11)?;
            r.bit("flag")?;
            if !single {
                r.r("second_yaw", 12)?;
                r.r("second_pitch", 11)?;
                r.bit("second_flag")?;
            }
        }
        "unit-active-camo-state-component" => {
            let mut state = super::NativeCamoState {
                state: r.r("state", 3)? as u8,
                flag0: r.bit("flag0")?,
                ..Default::default()
            };
            if !state.flag0 {
                let flag1 = r.bit("flag1")?;
                state.flag1 = Some(flag1);
                if !flag1 {
                    state.fraction = Some(r.r("fraction", 12)? as u16);
                }
            }
            for i in 0..6 {
                state.sub[i] = r
                    .gated_value(&format!("sub[{i}]"), 12, true)?
                    .map(|v| v as u16);
            }
            r.publish_component(super::FilmComponentObservation::CamoState {
                state: Box::new(state),
            });
        }
        "crew-order-component" => {
            r.r("order", 3)?;
            if r.bit("vector_present")? && !r.bit("vector_default")? {
                r.gate("index", 1, false)?;
                r.words("axis", 3, (6_u64 + u64::from(level)).min(26) as usize)?;
            }
        }
        "music-variables-component" => {
            for i in 0..32 {
                if !r.bit(&format!("variables[{i}].absent"))? {
                    r.r(&format!("variables[{i}].kind"), 2)?;
                    r.words(&format!("variables[{i}].words"), 2, 32)?;
                }
            }
        }
        "weapon-ammo-component" => {
            let a = r.r("a", 8)? as u32;
            let b = r.r("b", 11)? as u32;
            let c = r.r("c", 12)? as u32;
            r.publish_component(super::FilmComponentObservation::GroundWeaponAmmo { a, b, c });
        }
        "weapon-state-ammo" => {
            let magazine = r.gated_value("magazine", 8, false)?.map(|v| v as u32);
            let fraction = r.gated_value("fraction", 12, false)?.map(|v| v as u32);
            r.publish_component(super::FilmComponentObservation::WeaponAmmo { magazine, fraction });
        }
        "weapon-state-rounds-inventory" => {
            let rounds = r.r("rounds", 11)? as u32;
            r.publish_component(super::FilmComponentObservation::WeaponRounds { rounds });
        }
        "weapon-state-overheated" => {
            r.r("heat", 7)?;
            r.words("flags", 2, 1)?;
        }
        "biped-desired-weapon-set" => {
            let selection = r.r("selection", 3)? as u32;
            r.gate("slot_a", 2, false)?;
            r.gate("slot_b", 2, false)?;
            r.publish_component(super::FilmComponentObservation::DesiredWeaponSet { selection });
        }
        "projectile-at-rest-state" => {
            r.bit("rest")?;
            r.gate("direction", 19, true)?;
        }
        "projectile-command_tick" => r.gate("tick", 8, true)?,
        "projectile-tether-state"
        | "projectile-deceleration-disabled-state"
        | "item-at-rest-component"
        | "tacmap-fasttravelstate" => {
            r.bit("value")?;
        }
        "tacmap-iconlodthresholds" => {
            let count = r.r("count", 5)? as usize;
            r.words("thresholds", count, 12)?;
        }
        "tacmap-mapscale" | "tacmap-settingstag" => {
            r.r("value", 32)?;
        }
        "tacmap-cameraheading" => {
            r.r("heading", 12)?;
        }
        "tacmap-missioncount" => {
            r.r("count", 9)?;
        }
        "tacmap-missionmarkerstate" => r.words("words", 2, 32)?,
        "tacmap-lockedlights" => {
            for i in 0..4 {
                r.r(&format!("light[{i}].id"), 32)?;
                r.words(&format!("light[{i}].vector_bits"), 3, 32)?;
            }
        }
        "tacmap-dungeonstate" => {
            r.bit("flag")?;
            r.words("vector_bits", 3, 32)?;
        }
        "branch-script-results-component" => {
            r.r("index", 6)?;
            r.r("value", 32)?;
        }
        "high-frequency" => {
            let value = r.r("counter", 8)?;
            r.publish_component(crate::theater::parser::FilmComponentObservation::Probe {
                archetype,
                component: crate::theater::parser::NativeProbeComponent::HighFrequency,
                values: vec![value],
            });
        }
        "animated-mesh-dynamic-state-component" => {
            r.r("state", 8)?;
            r.bit("flag")?;
            r.r("value", 16)?;
        }
        "object-multiplayer-properties-component" => tlv::multiplayer_properties(r)?,
        "equipment-has-infinite-uses-component" => {
            r.bit("value")?;
        }
        "equipment-deployed-component" => {
            let value = r.r("value", 1)?;
            r.publish_component(
                crate::theater::parser::FilmComponentObservation::EquipmentState {
                    field: crate::theater::parser::NativeEquipmentField::Deployed,
                    value,
                    present: true,
                },
            );
        }
        "equipment-energy-component" => {
            let value = r.r("energy", 14)?;
            r.publish_component(
                crate::theater::parser::FilmComponentObservation::EquipmentState {
                    field: crate::theater::parser::NativeEquipmentField::Energy,
                    value,
                    present: true,
                },
            );
        }
        "equipment-energy-delay-ticks-left-component" => {
            let value = r.r("ticks", 10)?;
            r.publish_component(
                crate::theater::parser::FilmComponentObservation::EquipmentState {
                    field: crate::theater::parser::NativeEquipmentField::EnergyDelay,
                    value,
                    present: true,
                },
            );
        }
        "equipment-charges-remaining-component" => {
            let value = r.r("charges", 8)?;
            r.publish_component(
                crate::theater::parser::FilmComponentObservation::EquipmentState {
                    field: crate::theater::parser::NativeEquipmentField::Charges,
                    value,
                    present: true,
                },
            );
        }
        "equipment-creator-component" => {
            let value = r.gated_value("creator", 5, false)?;
            r.publish_component(
                crate::theater::parser::FilmComponentObservation::EquipmentState {
                    field: crate::theater::parser::NativeEquipmentField::Creator,
                    value: value.unwrap_or(0),
                    present: value.is_some(),
                },
            );
        }
        "equipment-activated-component" => {
            let value = if r.bit("reference_present")? {
                r.optional_handle("reference", 4)?;
                None
            } else {
                Some(r.r("value", 3)?)
            };
            r.publish_component(
                crate::theater::parser::FilmComponentObservation::EquipmentState {
                    field: crate::theater::parser::NativeEquipmentField::Activated,
                    value: value.unwrap_or(0),
                    present: value.is_some(),
                },
            );
        }
        "equipment-tracked-object-handles-stack-component" => {
            let count = r.r("count", 4)?;
            for i in 0..=count {
                r.inline_optional_handle(&format!("objects[{i}]"), 0)?;
            }
        }
        "equipment-command-tick-component" => {
            let single = r.bit("single")?;
            for i in 0..if single { 1 } else { 2 } {
                r.gate(&format!("ticks[{i}]"), 8, true)?;
            }
        }
        "equipment-being-hacked-component" => {
            r.r("value", 8)?;
        }
        "equipment-control-signal-component" => {
            r.r("signal", 4)?;
            r.optional_handle("reference", 4)?;
        }
        "player-waypoint-component" => {
            r.r("waypoint", 3)?;
        }
        "player-unsafe-respawn-timer-component" => {
            r.r("timer", 10)?;
        }
        "player-respawn-safety-component" => {
            r.r("safety", 5)?;
        }
        "device-position-component" => {
            r.r("position", 14)?;
            r.bit("flag")?;
        }
        "game-engine-campaign-timer-component" => {
            r.words("timers", 2, 16)?;
            r.r("state", 5)?;
        }
        _ => match basic::component(r, name) {
            Some(false) => match object::component(r, name, archetype) {
                Some(false) => match scene::component(r, name, level, archetype) {
                    Some(false) => match navpoint::component(r, name, level) {
                        Some(false) => return m4b::component(r, name),
                        result => return result,
                    },
                    result => return result,
                },
                result => return result,
            },
            result => return result,
        },
    }
    Some(true)
}

fn parent(r: &mut Reader<'_>, level: u32, archetype: u32) -> Option<()> {
    let mut state = super::NativeObjectParentState {
        archetype,
        parameter: level,
        start_bit: r.cursor.position,
        attached: r.bit("attached")?,
        ..Default::default()
    };
    if state.attached {
        r.handle("parent", 1)?;
        state.quantized_word = (r.fields[r.fields.len() - 1].raw as u32) << 30
            | r.fields[r.fields.len() - 2].raw as u32;
        state.word = r.r("word", 16)? as u32;
        state.optional_word = r.gated_value("optional_word", 16, true)?.map(|v| v as u32);
        for i in 0..2 {
            state.flags[i] = r.bit(&format!("flags[{i}]"))?;
        }
        for i in 0..3 {
            state.matrix[i] = r.r(&format!("matrix[{i}]"), 16)? as u32;
        }
        state.velocity = r.gated_value("velocity", 19, false)?.map(|v| v as u32);
        state.byte = r.r("magnitude", 8)? as u32;
        state.flag_c = r.bit("flag_c")?;
    } else if level < 2 {
        state.free_read = true;
        let at = r.cursor.position;
        r.optional_handle("free_reference", 0)?;
        state.free_bits = usize::try_from(r.cursor.position.wrapping_sub(at)).ok()?;
        let reference = r.references.last()?;
        state.free_id = reference.present.then_some(u64::from(reference.value));
        state.alternate = r.gated_value("alternate", 11, true)?.map(|v| v as u32);
    }
    state.tail_sign = r.bit("tail_sign")?;
    if state.tail_sign {
        state.tail6 = Some(r.r("tail_value", 6)? as u32);
    }
    state.tail_bit = r.bit("tail_flag")?;
    if level <= 2 || archetype == 35 {
        state.tail3 = Some(r.r("tail_enum", 3)? as u32);
    }
    state.end_bit = r.cursor.position;
    r.publish_component(super::FilmComponentObservation::ObjectParent {
        state: Box::new(state),
    });
    Some(())
}
