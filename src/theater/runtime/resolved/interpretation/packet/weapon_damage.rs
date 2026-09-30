//! Reference data models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WeaponDamageField {
    pub field: ProjectedField,
    /// The reference skips these bits; retaining their raw value does not
    /// assign semantics to the field. Offset names follow the reference reader.
    pub opaque: bool,
    /// Synthetic zero tail bits included in this field's raw value.
    pub padded_bits: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WeaponDamageTrace {
    pub source_bits: usize,
    pub end_bit: usize,
    pub padding_bits: usize,
    /// None retains a refused fixed-prefix/type read, not an absent action.
    pub read: Option<WeaponDamageRead>,
    pub fields: Vec<WeaponDamageField>,
}

use super::{WeaponDamage, WeaponDamageRead};
use crate::theater::parser::bits;
use crate::theater::runtime::interpretation::ProjectedField;

/// Reference weapon-statistics damage reader. This is independent of the generic
/// event-list layout reader: its endpoint must not substitute for a terminated
/// list. Source/body references, generations, unknown fields and quantized
/// values remain available even when the reference projection omits them.
pub(crate) fn read_weapon_damage(payload: &[u8], timestamp_us: u64) -> WeaponDamageTrace {
    decode_with_fields(payload, timestamp_us, true)
}

struct DamageReader<'a> {
    cursor: bits::Cursor<'a>,
    source_bits: usize,
    fields: Option<Vec<WeaponDamageField>>,
}
impl DamageReader<'_> {
    fn read(&mut self, name: &str, width: usize, opaque: bool) -> Option<u64> {
        let start = self.cursor.position;
        let raw = self.cursor.read(width)?;
        if let Some(fields) = &mut self.fields {
            fields.push(WeaponDamageField {
                field: ProjectedField {
                    name: name.into(),
                    bit: start,
                    width,
                    value: raw,
                    source_bits: self
                        .source_bits
                        .min(self.cursor.position)
                        .saturating_sub(start.min(self.source_bits)),
                },
                opaque,
                padded_bits: self
                    .cursor
                    .position
                    .saturating_sub(start.max(self.source_bits)),
            });
        }
        Some(raw)
    }
    fn bit(&mut self, name: &str) -> Option<bool> {
        self.read(name, 1, false).map(|v| v != 0)
    }
    fn skip(&mut self, name: &str, width: usize) -> Option<()> {
        self.read(name, width, true).map(|_| ())
    }
    fn reference(&mut self, names: [&str; 4], probe: bool) -> Option<u64> {
        if !self.bit(names[0])? {
            return None;
        }
        let width = if probe && self.bit(names[1])? { 9 } else { 13 };
        let index = self.read(names[2], width, false)?;
        self.read(names[3], 2, false)?;
        Some(index)
    }
}

pub(super) fn decode_with_fields(
    payload: &[u8],
    timestamp_us: u64,
    retain: bool,
) -> WeaponDamageTrace {
    let mut r = DamageReader {
        cursor: bits::Cursor::new_padded(payload, 0),
        source_bits: payload.len() * 8,
        fields: retain.then(Vec::new),
    };
    let read = (|| {
        if payload.len() < 2 || payload[0] != 0xc0 {
            return None;
        }
        r.bit("header.configuration")?;
        r.bit("header.continuation")?;
        if r.read("header.code", 7, false)? != 0 {
            return None;
        }
        let mut damage = WeaponDamage {
            timestamp_us,
            victim_idx: r
                .reference(
                    [
                        "header.victim.present",
                        "header.victim.narrow",
                        "header.victim.index",
                        "header.victim.generation",
                    ],
                    true,
                )
                .map_or(-1, |v| v as i64),
            responsible_idx: r
                .reference(
                    [
                        "header.responsible.present",
                        "header.responsible.narrow",
                        "header.responsible.index",
                        "header.responsible.generation",
                    ],
                    true,
                )
                .map_or(-1, |v| v as i64),
            ..Default::default()
        };
        r.reference(
            [
                "header.third.present",
                "header.third.narrow",
                "header.third.index",
                "header.third.generation",
            ],
            false,
        );
        if r.bit("body.source.present")? {
            damage.source = r.read("body.source.id", 32, false)?;
            damage.has_source = true;
        }
        if r.bit("body.field_0x10.present")? {
            r.skip("body.field_0x10", 5)?;
        }
        r.skip("body.field_0x14", 19)?;
        if r.bit("body.field_0x20.present")? {
            r.skip("body.field_0x20.part0", 19)?;
            r.skip("body.field_0x20.part1", 12)?;
        }
        for (name, width) in [
            ("body.vector[0]", 5),
            ("body.vector[1]", 5),
            ("body.vector[2]", 6),
        ] {
            r.skip(name, width)?;
        }
        if r.bit("body.field_0x40.present")? {
            r.skip("body.field_0x40", 5)?;
        }
        for name in [
            "body.flags[0]",
            "body.flags[1]",
            "body.flags[2]",
            "body.flags[3]",
            "body.flags[4]",
            "body.flags[5]",
            "body.flags[6]",
            "body.flags[7]",
            "body.flags[8]",
            "body.flags[9]",
            "body.flags[10]",
            "body.flags[11]",
            "body.flags[12]",
            "body.flags[13]",
        ] {
            r.skip(name, 1)?;
        }
        if r.bit("body.flags[14]")? {
            r.skip("body.flags_payload", 32)?;
        }
        r.skip("body.bit19", 1)?;
        r.skip("body.field_0x5c", 3)?;
        damage.mag_raw = r.read("body.magnitude", 5, false)?;
        let secondary_mag_raw = r.read("body.secondary_magnitude", 5, false)?;
        damage.mag_clear = match damage.mag_raw {
            0 => 0.,
            31 => 16.,
            v => (v as f64 - 0.5) * 16. / 30.,
        };
        damage.negative = r.bit("body.negative")?;
        if damage.negative {
            damage.mag_clear = -damage.mag_clear;
        }
        r.skip("body.field_0x52", 4)?;
        if !r.bit("body.field_0x54.omitted")? {
            r.skip("body.field_0x54", 10)?;
        }
        let f58 = r.read("body.field_0x58", 4, true)?;
        if r.bit("body.field_0x60.present")? {
            r.skip("body.field_0x60", 32)?;
        }
        if f58 == 1 {
            r.skip("body.field_0x58_payload", 8)?;
        }
        r.skip("body.field_0x70", 4)?;
        let body_victim_idx = r.reference(
            [
                "body.victim.present",
                "body.victim.narrow",
                "body.victim.index",
                "body.victim.generation",
            ],
            false,
        );
        Some(WeaponDamageRead {
            source: None,
            packet_index: None,
            damage,
            secondary_mag_raw,
            body_victim_idx,
            end_bit: r.cursor.position,
            padding_bits: r.cursor.position.saturating_sub(r.source_bits),
        })
    })();
    WeaponDamageTrace {
        source_bits: r.source_bits,
        end_bit: r.cursor.position,
        padding_bits: r.cursor.position.saturating_sub(r.source_bits),
        read,
        fields: r.fields.unwrap_or_default(),
    }
}

#[cfg(test)]
#[path = "weapon_damage_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
pub(crate) fn decode_weapon_damage(payload: &[u8], timestamp_us: u64) -> Option<WeaponDamageRead> {
    decode_with_fields(payload, timestamp_us, false).read
}
