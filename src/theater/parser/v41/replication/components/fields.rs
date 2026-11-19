//! Transfer decoded fields from temporary records into components.
use crate::theater::parser::v41::replication::source::components::field::ComponentField;

/// Retain fields the source reader also reports at record scope, including
/// reads outside a component's addressable range.
pub(crate) fn take_component_fields(
    record_fields: &mut Vec<ComponentField>,
    from: usize,
    start_bit: i64,
    end_bit: i64,
    source_bits: usize,
) -> Vec<ComponentField> {
    let fields: Vec<_> = record_fields.drain(from..).collect();
    let source_end = i64::try_from(source_bits).unwrap_or(i64::MAX);
    let range = (start_bit < source_end)
        .then(|| {
            Some((
                usize::try_from(start_bit).ok()?,
                usize::try_from(end_bit.min(source_end)).ok()?,
            ))
        })
        .flatten();
    for field in &fields {
        if range.is_none_or(|(start, end)| field.bit < start || field.bit >= end) {
            record_fields.push(field.clone());
        }
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::{ComponentField, take_component_fields};
    use crate::theater::parser::wire::fields::RawBits;

    fn field(bit: usize) -> ComponentField {
        ComponentField {
            name: format!("field.{bit}"),
            bit,
            width: 1,
            raw: RawBits::from_low(1, 1),
        }
    }

    #[test]
    fn transfers_component_fields_and_keeps_out_of_range_record_fields() {
        let mut record = vec![field(1), field(12), field(32)];
        let component = take_component_fields(&mut record, 1, 8, 24, 64);
        assert_eq!(component, vec![field(12), field(32)]);
        assert_eq!(record, vec![field(1), field(32)]);

        let mut record = vec![field(61), field(70)];
        let component = take_component_fields(&mut record, 0, 60, 100, 64);
        assert_eq!(component, vec![field(61), field(70)]);
        assert_eq!(record, vec![field(70)]);
    }
}
