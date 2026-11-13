//! Writer widths from reference analysis 413971b film_identity.go, FUN_14299b198/278;
//! timestamp and map IDs independently located by pinned reference reader header.rs.
use crate::theater::parser::v41::chunks::registry::source::{
    BootstrapIdentificationField, BootstrapIdentificationFieldKind as Kind,
    BootstrapIdentificationRead, BootstrapIdentificationStop,
};
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;

pub(super) fn read(data: &[u8], start_bit: usize) -> BootstrapIdentificationRead {
    let mut out = BootstrapIdentificationRead {
        start_bit,
        end_bit: start_bit,
        fields: Vec::new(),
        stop: BootstrapIdentificationStop::Complete,
        game_options: None,
    };
    let fields = [
        (Kind::NameBuffer(0), 256 * 8),
        (Kind::NameBuffer(1), 256 * 8),
        (Kind::StartTime, 32),
        (Kind::UnknownWord(0), 32),
        (Kind::UnknownWord(1), 32),
        (Kind::UnknownWord(2), 32),
        (Kind::UnknownBlock(0), 4096 * 8),
        (Kind::UnknownBlock(1), 4096 * 8),
        (Kind::UnknownBlock(2), 4096 * 8),
        (Kind::MapAssetId, 128),
        (Kind::MapVersionId, 128),
    ];
    for (kind, width) in fields {
        let bit = out.end_bit;
        let raw = RawBits::from_source(data, bit, width);
        let complete = raw.is_some();
        out.fields.push(BootstrapIdentificationField {
            kind,
            bit,
            width,
            raw,
        });
        if !complete {
            out.stop = BootstrapIdentificationStop::Truncated;
            break;
        }
        out.end_bit += width;
    }
    if out.stop == BootstrapIdentificationStop::Complete {
        out.game_options = Some(super::game_options::read(data, out.end_bit));
    }
    out
}

#[cfg(test)]
mod tests;
