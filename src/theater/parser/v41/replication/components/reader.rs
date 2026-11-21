//! Source-backed component field reading and internal reference evidence.
use super::cursor::ComponentCursor as Cursor;
use super::position::PositionEncoding;
use crate::theater::parser::v41::context::movement::MovementProfile;
use crate::theater::parser::v41::context::profile::DecodeGrammar;
use crate::theater::parser::v41::context::values::FilmMppWidths;
use crate::theater::parser::v41::reference::diagnostics::{
    FilmReadDiagnostics, ReadOperation, ReadRefusal, WidthRefusal,
};
use crate::theater::parser::v41::reference::observations::{
    FilmComponentObservation, MovementComponent, UnitReference, UnitReferenceKind,
};
use crate::theater::parser::v41::replication::source::components::field::ComponentField;
use std::cell::RefCell;
use std::sync::{Arc, LazyLock};

const FLAG_NAMES: [&str; 32] = [
    "flags[0]",
    "flags[1]",
    "flags[2]",
    "flags[3]",
    "flags[4]",
    "flags[5]",
    "flags[6]",
    "flags[7]",
    "flags[8]",
    "flags[9]",
    "flags[10]",
    "flags[11]",
    "flags[12]",
    "flags[13]",
    "flags[14]",
    "flags[15]",
    "flags[16]",
    "flags[17]",
    "flags[18]",
    "flags[19]",
    "flags[20]",
    "flags[21]",
    "flags[22]",
    "flags[23]",
    "flags[24]",
    "flags[25]",
    "flags[26]",
    "flags[27]",
    "flags[28]",
    "flags[29]",
    "flags[30]",
    "flags[31]",
];

static COMMON_FIELD_NAMES: LazyLock<[Arc<str>; 39]> = LazyLock::new(|| {
    [
        "baseline.gate",
        "stationary",
        "flag",
        "direction",
        "finite",
        "position[0]",
        "position[1]",
        "position[2]",
        "extra",
        "tick",
        "tick.gate",
        "magnitude",
        "delta",
        "use_prediction",
        "precision_high",
        "default_region",
        "region_index",
        "full_precision",
        "single",
        "yaw",
        "mask",
        "value_b.selector",
        "index_a",
        "index_b",
        "value_a",
        "has_b",
        "value_b",
        "has_a",
        "value_a.selector",
        "counter",
        "pitch",
        "shield",
        "block64",
        "regeneration",
        "value",
        "tag",
        "index",
        "state",
        "count",
    ]
    .map(Arc::from)
});

thread_local! {
    static INTERNED_NAMES: RefCell<rustc_hash::FxHashMap<String, Arc<str>>> =
        RefCell::new(rustc_hash::FxHashMap::default());
}

pub(crate) fn clear_interned_names() {
    INTERNED_NAMES.with(|names| names.borrow_mut().clear());
}

pub(crate) fn intern_name(name: &str) -> Arc<str> {
    let common = match name {
        "baseline.gate" => Some(0),
        "stationary" => Some(1),
        "flag" => Some(2),
        "direction" => Some(3),
        "finite" => Some(4),
        "position[0]" => Some(5),
        "position[1]" => Some(6),
        "position[2]" => Some(7),
        "extra" => Some(8),
        "tick" => Some(9),
        "tick.gate" => Some(10),
        "magnitude" => Some(11),
        "delta" => Some(12),
        "use_prediction" => Some(13),
        "precision_high" => Some(14),
        "default_region" => Some(15),
        "region_index" => Some(16),
        "full_precision" => Some(17),
        "single" => Some(18),
        "yaw" => Some(19),
        "mask" => Some(20),
        "value_b.selector" => Some(21),
        "index_a" => Some(22),
        "index_b" => Some(23),
        "value_a" => Some(24),
        "has_b" => Some(25),
        "value_b" => Some(26),
        "has_a" => Some(27),
        "value_a.selector" => Some(28),
        "counter" => Some(29),
        "pitch" => Some(30),
        "shield" => Some(31),
        "block64" => Some(32),
        "regeneration" => Some(33),
        "value" => Some(34),
        "tag" => Some(35),
        "index" => Some(36),
        "state" => Some(37),
        "count" => Some(38),
        _ => None,
    };
    if let Some(index) = common {
        return COMMON_FIELD_NAMES[index].clone();
    }
    INTERNED_NAMES.with(|names| {
        let mut names = names.borrow_mut();
        if let Some(interned) = names.get(name) {
            return interned.clone();
        }
        let interned: Arc<str> = Arc::from(name);
        names.insert(name.to_owned(), interned.clone());
        interned
    })
}

fn gate_name<'a>(name: &str, buffer: &'a mut [u8; 128]) -> Option<&'a str> {
    let end = name.len().checked_add(5)?;
    if end > buffer.len() {
        return None;
    }
    buffer[..name.len()].copy_from_slice(name.as_bytes());
    buffer[name.len()..end].copy_from_slice(b".gate");
    std::str::from_utf8(&buffer[..end]).ok()
}

fn indexed_name<'a>(prefix: &str, index: usize, buffer: &'a mut [u8; 128]) -> Option<&'a str> {
    let mut digits = [0u8; 20];
    let mut number = index;
    let mut count = 0;
    loop {
        digits[count] = b'0' + (number % 10) as u8;
        count += 1;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    let end = prefix.len().checked_add(count + 2)?;
    if end > buffer.len() {
        return None;
    }
    buffer[..prefix.len()].copy_from_slice(prefix.as_bytes());
    buffer[prefix.len()] = b'[';
    for position in 0..count {
        buffer[prefix.len() + 1 + position] = digits[count - 1 - position];
    }
    buffer[end - 1] = b']';
    std::str::from_utf8(&buffer[..end]).ok()
}
#[derive(Clone, Copy)]
pub(crate) struct ComponentWidths<'a> {
    pub movement: &'a MovementProfile,
    pub mpp: FilmMppWidths,
    pub maximum: u64,
}
pub(crate) struct ComponentReader<'a> {
    pub(crate) category_references: bool,
    /// Encoding mode established by the enclosing packet, never inferred from values.
    pub(crate) snapshot: bool,
    pub(crate) reference_widths: Option<ComponentWidths<'a>>,
    /// Missing layout context is unsupported, not truncated source.
    pub(crate) context_error: Option<&'static str>,
    pub(crate) width_error: Option<&'static str>,

    pub(crate) live_grammar: Option<&'a DecodeGrammar>,
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
        const MASK_INDEX_NAMES: [&str; 7] = [
            "mask.index[0]",
            "mask.index[1]",
            "mask.index[2]",
            "mask.index[3]",
            "mask.index[4]",
            "mask.index[5]",
            "mask.index[6]",
        ];
        for i in 0..count {
            mask |= 1 << self.r(MASK_INDEX_NAMES[i as usize], 6)?;
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
            name: intern_name(name),
            bit: usize::try_from(bit).ok()?,
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
        let mut buffer = [0u8; 128];
        let present = if let Some(gate) = gate_name(name, &mut buffer) {
            self.bit(gate)?
        } else {
            self.bit(&format!("{name}.gate"))?
        };
        if present == polarity {
            Some(Some(self.r(name, width)?))
        } else {
            Some(None)
        }
    }
    pub(crate) fn words(&mut self, name: &str, count: usize, width: usize) -> Option<()> {
        if name == "flags" && count <= FLAG_NAMES.len() {
            for flag in FLAG_NAMES.iter().take(count) {
                self.r(flag, width)?;
            }
            return Some(());
        }
        let mut buffer = [0u8; 128];
        for i in 0..count {
            if let Some(indexed) = indexed_name(name, i, &mut buffer) {
                self.r(indexed, width)?;
            } else {
                self.r(&format!("{name}[{i}]"), width)?;
            }
        }
        Some(())
    }
    pub(crate) fn handle(&mut self, name: &str, mut category: u8) -> Option<()> {
        // FUN_1406d3140 consumes category 1's probe even with the table
        // disabled. Its value cannot establish the unknown global range.
        if category == 1 && self.bit(&format!("{name}.category4"))? {
            category = 4;
        }
        self.require_category_references()?;
        let width = match category {
            2 | 3 | 5 => 8,
            4 | 6 => 9,
            _ => 13,
        };
        self.r(&format!("{name}.value"), width)?;
        self.r(&format!("{name}.generation"), 2)?;
        Some(())
    }
    pub(crate) fn reference_configuration_unavailable(&self) -> bool {
        self.context_error == Some("global reference range")
    }
    pub(crate) fn runtime_context_unavailable(&self) -> Option<&'static str> {
        (self.context_error == Some("engine-one handle width")).then_some("engine-one handle width")
    }
    pub(crate) fn require_category_references(&mut self) -> Option<()> {
        if !self.category_references {
            self.context_error = Some("global reference range");
            return None;
        }
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
    pub(crate) fn optional_word_reference(
        &mut self,
        name: &str,
        emit_absent: bool,
    ) -> Option<bool> {
        let start_bit = self.cursor.position;
        let mut buffer = [0u8; 128];
        let present = if let Some(gate) = gate_name(name, &mut buffer) {
            self.bit(gate)?
        } else {
            self.bit(&format!("{name}.gate"))?
        };
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
        Some(present)
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

#[cfg(test)]
mod tests {
    use super::{COMMON_FIELD_NAMES, gate_name, indexed_name, intern_name};
    use std::sync::Arc;

    #[test]
    fn stack_gate_names_preserve_utf8_and_fall_back_when_long() {
        let mut buffer = [0u8; 128];
        assert_eq!(gate_name("λ.flag", &mut buffer), Some("λ.flag.gate"));
        assert_eq!(
            gate_name(&"x".repeat(123), &mut buffer).map(str::len),
            Some(128)
        );
        assert_eq!(gate_name(&"x".repeat(124), &mut buffer), None);
    }

    #[test]
    fn indexed_names_preserve_prefix_and_decimal_index() {
        let mut buffer = [0u8; 128];
        assert_eq!(indexed_name("λ.flags", 0, &mut buffer), Some("λ.flags[0]"));
        let expected = format!("flags[{}]", usize::MAX);
        assert_eq!(
            indexed_name("flags", usize::MAX, &mut buffer),
            Some(expected.as_str())
        );
        assert_eq!(indexed_name(&"x".repeat(128), 1, &mut buffer), None);
    }

    #[test]
    fn common_name_slots_match_their_lookup_keys() {
        for name in COMMON_FIELD_NAMES.iter() {
            assert!(Arc::ptr_eq(&intern_name(name), name));
        }
    }
}
