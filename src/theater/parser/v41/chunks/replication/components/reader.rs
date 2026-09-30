//! Source-backed component field reading and internal reference evidence.
use super::cursor::ComponentCursor as Cursor;
use super::position::PositionEncoding;
use crate::theater::film::ComponentField;
use crate::theater::parser::v41::context::movement::MovementProfile;
use crate::theater::parser::v41::context::profile::DecodeGrammar;
use crate::theater::parser::v41::context::values::FilmMppWidths;
use crate::theater::parser::v41::reference::diagnostics::{
    FilmReadDiagnostics, ReadOperation, ReadRefusal, WidthRefusal,
};
use crate::theater::parser::v41::reference::observations::{
    FilmComponentObservation, MovementComponent, UnitReference, UnitReferenceKind,
};
#[derive(Clone, Copy)]
pub(crate) struct ComponentWidths<'a> {
    pub movement: &'a MovementProfile,
    pub mpp: FilmMppWidths,
    pub maximum: u64,
}
pub(crate) struct ComponentReader<'a> {
    pub(crate) reference_widths: Option<ComponentWidths<'a>>,
    pub(crate) width_error: Option<&'static str>,

    pub(crate) live_grammar: Option<DecodeGrammar>,
    pub(crate) movement_slot: Option<u32>,
    pub(crate) references: Vec<UnitReference>,
    pub(crate) diagnostics: FilmReadDiagnostics,
    pub(crate) cursor: Cursor<'a>,
    pub(crate) fields: Vec<ComponentField>,
    pub(crate) position_encoding: Option<&'a PositionEncoding>,
}

impl ComponentReader<'_> {
    pub(crate) fn reference_position_width(
        &mut self,
        raw: u64,
        field: &'static str,
    ) -> Option<u64> {
        self.reference_width_limited(raw, field, u64::MAX)
    }

    pub(crate) fn reference_width_limited(
        &mut self,
        raw: u64,
        field: &'static str,
        limit: u64,
    ) -> Option<u64> {
        let maximum = self
            .reference_widths
            .map_or(u64::MAX, |w| w.maximum)
            .min(limit);
        if raw > maximum {
            self.width_error = Some(field);
            self.diagnostics.width_refusals.push(WidthRefusal {
                field: field.into(),
                bit: self.cursor.position,
                raw_width: raw,
                maximum,
            });
            return None;
        }
        Some(raw)
    }

    pub(crate) fn publish_component(&mut self, value: FilmComponentObservation) {
        self.diagnostics.component_observations.push(value);
    }
    pub(crate) fn publish_mobility(&mut self, flags: [bool; 2]) {
        self.diagnostics.publish_mobility(flags);
    }
    pub(crate) fn record_absolute(&mut self, index: i32) {
        self.diagnostics.absolute(index);
    }

    pub(crate) fn publish_movement(&mut self, component: MovementComponent, values: Vec<u64>) {
        if let Some(slot) = self.movement_slot {
            self.publish_component(FilmComponentObservation::MovementState {
                component,
                slot,
                values,
            });
        }
    }

    pub(crate) fn mask(&mut self) -> Option<u64> {
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
    pub(crate) fn refuse_read(&mut self, name: &str, width: u64, operation: ReadOperation) {
        self.diagnostics.read_refusals.push(ReadRefusal {
            field: name.into(),
            bit: self.cursor.position,
            width,
            source_bits: self.cursor.source_bits(),
            operation,
        });
    }
    pub(crate) fn guard_source(&mut self, name: &str, width: i64) -> Option<()> {
        if self.cursor.fits_source(width) {
            return Some(());
        }
        self.refuse_read(name, width as u64, ReadOperation::GroupGuard);
        None
    }
    pub(crate) fn r(&mut self, name: &str, width: usize) -> Option<u64> {
        self.r_wide(name, width as u64)
    }
    pub(crate) fn r_wide(&mut self, name: &str, width: u64) -> Option<u64> {
        let bit = self.cursor.position;
        let Some(bits) = self.cursor.raw_bits(bit, width) else {
            self.refuse_read(name, width, ReadOperation::Scalar);
            return None;
        };
        self.cursor.skip_signed(i64::try_from(width).ok()?);
        let raw = bits.low_u64();
        self.fields.push(ComponentField {
            name: name.into(),
            bit: usize::try_from(bit).ok()?,
            width: usize::try_from(width).ok()?,
            raw: bits,
        });
        Some(raw)
    }
    pub(crate) fn bit(&mut self, name: &str) -> Option<bool> {
        Some(self.r(name, 1)? != 0)
    }
    pub(crate) fn gate(&mut self, name: &str, width: usize, polarity: bool) -> Option<()> {
        self.gated_value(name, width, polarity).map(|_| ())
    }
    pub(crate) fn gated_value(
        &mut self,
        name: &str,
        width: usize,
        polarity: bool,
    ) -> Option<Option<u64>> {
        if self.bit(&format!("{name}.gate"))? == polarity {
            Some(Some(self.r(name, width)?))
        } else {
            Some(None)
        }
    }
    pub(crate) fn words(&mut self, name: &str, count: usize, width: usize) -> Option<()> {
        for i in 0..count {
            self.r(&format!("{name}[{i}]"), width)?;
        }
        Some(())
    }
    pub(crate) fn handle(&mut self, name: &str, mut category: u8) -> Option<()> {
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
    // Same wire shape, used by reference inline readers which do not publish UnitRefRead.
    pub(crate) fn inline_optional_handle(&mut self, name: &str, category: u8) -> Option<()> {
        if self.bit(&format!("{name}.present"))? {
            self.handle(name, category)?;
        }
        Some(())
    }
    pub(crate) fn publish_reference(&mut self, reference: UnitReference) {
        self.publish_component(FilmComponentObservation::UnitReference {
            reference: reference.clone(),
        });
        self.references.push(reference);
    }
    pub(crate) fn optional_handle(&mut self, name: &str, category: u8) -> Option<()> {
        let start_bit = self.cursor.position;
        let present = self.bit(&format!("{name}.present"))?;
        let (value, tail) = if present {
            self.handle(name, category)?;
            (
                self.fields[self.fields.len() - 2].raw.low_u64() as u32,
                self.fields[self.fields.len() - 1].raw.low_u64() as u32,
            )
        } else {
            (0, 0)
        };
        self.publish_reference(UnitReference {
            kind: UnitReferenceKind::VariableWidth,
            start_bit,
            end_bit: self.cursor.position,
            present,
            value,
            tail,
            probe: category == 1,
        });
        Some(())
    }
    pub(crate) fn optional_word_reference(&mut self, name: &str, emit_absent: bool) -> Option<()> {
        let start_bit = self.cursor.position;
        let present = self.bit(&format!("{name}.gate"))?;
        let value = if present { self.r(name, 32)? as u32 } else { 0 };
        if present || emit_absent {
            self.publish_reference(UnitReference {
                kind: UnitReferenceKind::GatedWord32,
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
    pub(crate) fn word_reference(&mut self, name: &str) -> Option<()> {
        let start_bit = self.cursor.position;
        let value = self.r(name, 32)? as u32;
        self.publish_reference(UnitReference {
            kind: UnitReferenceKind::Word32,
            start_bit,
            end_bit: self.cursor.position,
            present: true,
            value,
            tail: 0,
            probe: false,
        });
        Some(())
    }
    pub(crate) fn direction(&mut self, scale_width: usize) -> Option<()> {
        if !self.bit("stationary")? {
            self.r("direction", 19)?;
            self.r("magnitude", scale_width)?;
        }
        Some(())
    }
    pub(crate) fn frame_configuration(&mut self) -> Option<()> {
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
