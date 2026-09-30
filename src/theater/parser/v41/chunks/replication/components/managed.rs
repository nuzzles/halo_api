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
