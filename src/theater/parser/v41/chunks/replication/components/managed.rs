//! Managed component wire grammars.

use super::reader::ComponentReader;
use crate::theater::parser::v41::reference::observations::{
    FilmComponentObservation, ManagedPropertyField, ProbeComponent,
};

pub(crate) fn read_primary(
    r: &mut ComponentReader<'_>,
    name: &str,
    _level: u32,
    archetype: u32,
) -> Option<bool> {
    match name {
        "managed-radialmenu-visibility-component" => {
            let count = r.r("count", 32)?;
            let mask = r.r("mask", 32)?;
            r.r("value", 32)?;
            if count > 32 {
                return Some(false);
            }
            for i in 0..count {
                if mask & (1 << i) != 0 {
                    r.gate(&format!("items[{i}].player"), 5, false)?;
                    r.bit(&format!("items[{i}].flag"))?;
                }
            }
        }
        "managed-player-team-designator-component" => {
            r.r("team", 4)?;
        }
        "managed-object-networked-splash-message-dynamic-component" => {
            let value = r.r("message", 24)?;
            r.publish_component(FilmComponentObservation::Probe {
                archetype,
                component: ProbeComponent::SplashDynamic,
                values: vec![value],
            });
        }
        "managed-object-property-component" | "managed-object-player-masked-property-component" => {
            let tag = r.r("tag", 4)?;
            let mode_a = name == "managed-object-property-component";
            let width = property_width(tag, !mode_a);
            let mut values = vec![tag];
            if width != 0 {
                values.push(r.r("value", width)?);
            }
            let field = if mode_a {
                ManagedPropertyField::Scalar
            } else {
                ManagedPropertyField::PerPlayer
            };
            r.publish_component(FilmComponentObservation::ManagedProperty { field, values });
        }
        _ => return Some(false),
    }
    Some(true)
}

/// Shared creation/component property wire width. Opposite-domain tags carry no
/// value. The native switch's default (tags11..15) shares the four-bit enum
/// reader, FUN_141fce2f0 -> FUN_1407ef804; the reference reader's tags12..15 width differs.
pub(crate) fn property_width(tag: u64, masked: bool) -> usize {
    if tag == 0 || (tag < 7) == masked {
        return 0;
    }
    match tag {
        1 | 11..=15 => 4,
        2 | 10 => 1,
        3 | 7 => 24,
        _ => 32,
    }
}
