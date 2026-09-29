//! v41 archetype defaults read before the component loop.
use super::Reader;

/// FUN_1408efb58 at d61443e. NEW calls with param5=true. The full-state
/// reference form is retained here, but is not wired into keyframe traversal.
pub(super) fn projectile(r: &mut Reader<'_>, widths: [usize; 2], param5: bool) -> Option<bool> {
    let version = if r.bit("version.present")? {
        r.r("version", 8)?
    } else {
        1
    };
    multiplayer(r, widths)?;
    r.gate("projectile.value_70", 5, true)?;
    if r.bit("projectile.flag2")? {
        r.optional_handle("projectile.reference_64", 1)?;
        r.gate("projectile.id2", 2, false)?;
    }
    if r.bit("projectile.target.present")? {
        let kind = r.r("projectile.target.kind", 2)?;
        if kind == 1 || kind == 2 {
            if param5 {
                r.handle("projectile.target.reference", kind as u8)?;
            } else {
                r.r("projectile.target.word", 32)?;
            }
            if kind == 1 {
                r.gate("projectile.target.index", 6, true)?;
            }
        }
    }
    r.bit("projectile.flag4")?;
    if r.bit("projectile.scales.present")? {
        r.words("projectile.scales", 2, 5)?;
    }
    r.r("projectile.scale_90", 5)?;
    if r.bit("projectile.flag8")? {
        if !projectile_position(r)? {
            return Some(false);
        }
        r.r("projectile.direction", 19)?;
        r.r("projectile.speed", 12)?;
    }
    if r.bit("projectile.flag10")? {
        r.optional_handle("projectile.reference_b0", 0)?;
    }
    if r.bit("projectile.flag20")? {
        if r.bit("projectile.relative")? {
            r.r("projectile.relative.kind", 2)?;
            r.words("projectile.relative.axes", 3, 13)?;
            r.gate("projectile.relative.tail", 16, true)?;
        } else if !projectile_position(r)? {
            return Some(false);
        }
    }
    if version > 2 {
        r.bit("projectile.flag40")?;
    }
    if r.bit("projectile.tail.present")? {
        r.gate("projectile.tail.index", 5, false)?;
        r.r("projectile.tail.value7", 7)?;
        r.bit("projectile.tail.flag")?;
        r.r("projectile.tail.value4", 4)?;
    }
    Some(true)
}

fn projectile_position(r: &mut Reader<'_>) -> Option<bool> {
    let Some(encoding) = r.position_encoding else {
        return Some(false);
    };
    if encoding.full_precision_gate() {
        r.words("projectile.position_bits", 3, 32)?;
    } else {
        super::position::absolute_payload(r, encoding)?;
    }
    Some(true)
}

/// Known v41 default-state entries, including the special biped path.
/// Verified zero-bit stubs are intentionally absent: TraverseEntity applies its
/// configured fallback width to them even though their ordinary default is empty.
pub(super) fn has_reference_deserializer(ti: u32) -> bool {
    matches!(
        ti,
        3 | 5
            | 6
            | 8
            | 9
            | 10
            | 11
            | 12
            | 13
            | 14
            | 17
            | 20
            | 21
            | 24
            | 28
            | 29
            | 35
            | 36
            | 37
            | 38
            | 39
            | 40
            | 42
            | 43
            | 47
            | 48
            | 49
    )
}

pub(super) fn state(r: &mut Reader<'_>, ti: u32, widths: [usize; 2]) -> Option<bool> {
    match ti {
        // Verified zero-bit vtable implementations; excluded from the fallback.
        0 | 1 | 2 | 4 | 7 | 15 | 16 | 18 | 19 | 22 | 25 | 26 | 27 | 30 | 31 | 32 | 33 | 34 | 45
        | 46 => {}
        3 => {
            version(r)?;
            r.optional_handle("reference", 0)?;
            r.gate("value", 8, true)?;
            r.bit("flag")?;
            r.r("tail", 8)?;
        }
        5 | 14 | 17 | 47 => {
            version(r)?;
            r.r(
                "index",
                match ti {
                    5 => 6,
                    17 => 7,
                    _ => 5,
                },
            )?;
        }
        6 => {
            r.r("index", 6)?;
        }
        8 => {
            let version = if r.bit("version.present")? {
                r.r("version", 8)?
            } else {
                0
            };
            r.r("value", if version > 1 { 16 } else { 8 })?;
        }
        9 => {
            version(r)?;
            r.r("player_index", 6)?;
            r.r("secondary_index", 6)?;
            r.bit("flag")?;
        }
        10 => {
            version(r)?;
            r.optional_handle("reference", 0)?;
        }
        11 | 12 | 20 | 29 | 49 => version(r)?,
        13 => {
            version(r)?;
            r.r("property_name", 32)?;
            let count = if r.bit("array")? { 32 } else { 1 };
            r.words("properties", count, 4)?;
        }
        21 => {
            r.r("value", 18)?;
        }
        24 => {
            r.gate("index", 2, false)?;
            r.r("state", 3)?;
        }
        28 => {
            r.r("value", 32)?;
        }
        35 => {
            let v = if r.bit("version.present")? {
                r.r("version", 8)?
            } else {
                13
            };
            r.gate("representation_name", 32, true)?;
            if v > 10 {
                r.gate("player_index", 5, false)?;
            }
            multiplayer(r, widths)?;
            r.gate("index", 6, true)?;
            r.bit("flag")?;
            r.optional_word_reference("reference", true)?;
            r.r("direction", 19)?;
            if v > 5 {
                r.bit("version_flag")?;
            }
            if v >= 12 {
                // d61443e: this word belongs to the film reader, before the
                // NEW component gate/mask (also present in full defaults).
                let start_bit = r.cursor.position;
                let present = r.bit("external_reference_gate")?;
                let value = if present {
                    r.r("external_reference_word", 32)? as u32
                } else {
                    0
                };
                r.publish_reference(crate::theater::parser::v41::UnitReference {
                    kind: crate::theater::parser::v41::UnitReferenceKind::GatedWord32,
                    start_bit,
                    end_bit: r.cursor.position,
                    present,
                    value,
                    tail: 0,
                    probe: false,
                });
            }
        }
        36..=40 | 42 | 43 => {
            if ti == 40 && r.position_encoding.is_none() {
                return Some(false);
            }
            version(r)?;
            if ti == 37 || ti == 42 {
                version(r)?;
            }
            multiplayer(r, widths)?;
            match ti {
                37 => {
                    let value = r.gated_value("player_index", 5, false)?;
                    publish_creation(
                        r,
                        crate::theater::parser::v41::EquipmentCreationField::Reference,
                        value,
                    );
                    let value = r.gated_value("ability_enabled_id", 32, true)?;
                    publish_creation(
                        r,
                        crate::theater::parser::v41::EquipmentCreationField::AbilityId,
                        value,
                    );
                }
                38 | 39 => r.optional_handle("reference", 0)?,
                40 => {
                    if r.bit("media_frame.present")? {
                        super::orientation::media_frame(r)?;
                        r.gate("media_frame.direction", 19, false)?;
                        r.r("media_frame.magnitude", 8)?;
                    }
                    r.r("direction", 19)?;
                    let count = if r.bit("reference_list")? {
                        r.r("reference_count", 2)?
                    } else {
                        1
                    };
                    for i in 0..count {
                        r.optional_word_reference(&format!("references[{i}]"), true)?;
                    }
                }
                42 => {
                    r.r("value12", 12)?;
                    r.r("value7", 7)?;
                    super::object::magazines(r)?;
                    let value = r.gated_value("player_index", 5, false)?;
                    publish_creation(
                        r,
                        crate::theater::parser::v41::EquipmentCreationField::Reference,
                        value,
                    );
                }
                _ => {}
            }
        }
        48 => r.gate("player_index", 5, false)?,
        _ => return Some(false),
    }
    Some(true)
}

fn publish_mpp(
    r: &mut Reader<'_>,
    field: crate::theater::parser::v41::MppField,
    value: Option<u64>,
) {
    r.publish_component(crate::theater::parser::v41::FilmComponentObservation::Mpp {
        field,
        value: value.unwrap_or(0),
        present: value.is_some(),
    });
}

fn publish_creation(
    r: &mut Reader<'_>,
    field: crate::theater::parser::v41::EquipmentCreationField,
    value: Option<u64>,
) {
    r.publish_component(
        crate::theater::parser::v41::FilmComponentObservation::EquipmentCreation {
            field,
            value: value.unwrap_or(0),
            present: value.is_some(),
        },
    );
}

/// Reference MPP widths are signed profile values cast to uint at the read. Admit
/// each only when reached, retaining that unsigned value on a domain refusal.
fn mpp_field(r: &mut Reader<'_>, lead: bool, fallback: usize) -> Option<u64> {
    let (name, field) = if lead {
        ("mpp.lead", "MPP lead")
    } else {
        ("mpp.index", "MPP index")
    };
    if let Some(reference) = r.reference_widths {
        let raw = if lead {
            reference.mpp.lead
        } else {
            reference.mpp.index
        } as u64;
        let width = r.reference_width_limited(raw, field, u64::MAX)?;
        r.r_wide(name, width)
    } else {
        r.r(name, fallback)
    }
}

fn multiplayer(r: &mut Reader<'_>, widths: [usize; 2]) -> Option<()> {
    let value = mpp_field(r, true, widths[0])?;
    publish_mpp(r, crate::theater::parser::v41::MppField::Word9, Some(value));
    let value = r.r("mpp.object_tag", 32)?;
    publish_mpp(
        r,
        crate::theater::parser::v41::MppField::Word32,
        Some(value),
    );
    let value = r.gated_value("mpp.variant_name", 32, false)?;
    publish_mpp(r, crate::theater::parser::v41::MppField::VariantName, value);
    r.gate("mpp.value18", 18, true)?;
    r.gate("mpp.value13", 13, true)?;
    r.r("mpp.kind", 2)?;
    mpp_field(r, false, widths[1])?;
    let count = r.r("mpp.count", 3)?;
    if count <= 4 {
        for i in 0..count {
            r.r(&format!("mpp.entry[{i}].index"), 5)?;
            r.optional_word_reference(&format!("mpp.entry[{i}].reference"), true)?;
        }
    }
    if r.bit("mpp.extra.present")? {
        r.gate("mpp.extra.index", 5, false)?;
        r.gate("mpp.extra.value", 8, true)?;
        r.words("mpp.extra.bytes", 2, 8)?;
    }
    if r.bit("mpp.tail.present")? {
        let value = r.r("mpp.tail.name", 32)?;
        publish_mpp(
            r,
            crate::theater::parser::v41::MppField::TailName,
            Some(value),
        );
        r.optional_word_reference("mpp.tail.reference", true)?;
        r.r("mpp.tail.fraction", 14)?;
    } else {
        publish_mpp(r, crate::theater::parser::v41::MppField::TailName, None);
    }
    Some(())
}

fn version(r: &mut Reader<'_>) -> Option<()> {
    r.gate("version", 8, true)
}
