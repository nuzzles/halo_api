//! Event grammar selected from recorded bootstrap fields, never playback state.
use crate::theater::parser::v41::chunks::registry::source::{
    BootstrapHeaderStop, BootstrapIdentificationStop, FilmRegistryRead, GameOptionsEntry,
    GameOptionsFieldKind, GameOptionsStop,
};
mod variant;
mod versions;
pub(crate) use variant::{VariantContext, recorded_variant};
pub(crate) use versions::EventVersionContext;

/// Script v1's writer emits the 15-bit prefix unless the recorder is a
/// distributed client (simulation 2). The playback reader's own simulation
/// predicate is different; it must not substitute for the recorded writer.
pub(crate) fn recorded_script_prefix(registry: &FilmRegistryRead) -> Option<bool> {
    let header = registry.bootstrap_header.as_ref()?;
    if header.stop != BootstrapHeaderStop::Complete || header.event_version(15) != Some(1) {
        return None;
    }
    let identity = header.identification.as_ref()?;
    if identity.stop != BootstrapIdentificationStop::Complete {
        return None;
    }
    let options = identity.game_options.as_ref()?;
    if options.stop != GameOptionsStop::Complete {
        return None;
    }
    options.entries.iter().find_map(|entry| match entry {
        GameOptionsEntry::Field {
            kind: GameOptionsFieldKind::RecorderSimulation,
            bits,
        } if bits.width == 3 => match bits.raw.as_ref()?.low_u64() {
            0 | 1 | 3 => Some(true),
            2 => Some(false),
            _ => None,
        },
        _ => None,
    })
}

#[cfg(test)]
pub(super) mod tests;
