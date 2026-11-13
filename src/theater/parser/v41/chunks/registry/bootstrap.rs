//! v41/27 fixed event-version/build prefix (FUN_1428e1c64: film+0xcb208+4*genre).
//! No build-string scan selects a boundary or supplies a missing version.
use crate::theater::parser::v41::chunks::registry::source::{
    BootstrapHeaderField, BootstrapHeaderFieldKind as Kind, BootstrapHeaderRead,
    BootstrapHeaderStop, FilmRegistry,
};
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;
mod bond;
mod game_options;
mod identification;

pub(super) fn read(
    data: &[u8],
    header: Option<[u32; 2]>,
    registry: &FilmRegistry,
    end: usize,
) -> Option<BootstrapHeaderRead> {
    // This is the currently established layout, not a promise that every
    // future v41 writer has 123 event kinds or the same registry shape.
    if header != Some([41, 27])
        || registry.archetypes.len() != 50
        || end != 0xcb208
        || registry.fingerprint() != Some(super::KNOWN_REGISTRY_FINGERPRINT)
    {
        return None;
    }
    Some(read_fields(data, end * 8))
}

fn read_fields(data: &[u8], start_bit: usize) -> BootstrapHeaderRead {
    let mut out = BootstrapHeaderRead {
        start_bit,
        end_bit: start_bit,
        fields: Vec::new(),
        stop: BootstrapHeaderStop::Complete,
        identification: None,
    };
    let fields = (0..123).map(|code| (Kind::EventVersion(code), 32)).chain([
        (Kind::BuildVersion, 256),
        (Kind::BuildBranch, 256),
        (Kind::BuildFlavor, 256),
        (Kind::BuildId, 32),
        (Kind::Changelist, 32),
        (Kind::CorruptionChecks, 1),
    ]);
    for (kind, width) in fields {
        let bit = out.end_bit;
        let raw = RawBits::from_source(data, bit, width);
        let complete = raw.is_some();
        out.fields.push(BootstrapHeaderField {
            kind,
            bit,
            width,
            raw,
        });
        if !complete {
            out.stop = BootstrapHeaderStop::Truncated;
            break;
        }
        out.end_bit += width;
    }
    if out.stop == BootstrapHeaderStop::Complete {
        out.identification = Some(identification::read(data, out.end_bit));
    }
    out
}

#[cfg(test)]
mod tests;
