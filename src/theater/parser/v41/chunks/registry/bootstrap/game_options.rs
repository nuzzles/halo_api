//! FUN_1407ec560 / FUN_1407ee138: pinned reference analysis film_variante_de_partie.go.
//! Walk both Bond messages and all intervening fields; never search for strings.
use crate::theater::parser::v41::chunks::registry::source::{bond::*, game_options::*};
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;

pub(super) fn read(data: &[u8], start_bit: usize) -> GameOptionsRead {
    let mut r = Reader {
        data,
        out: GameOptionsRead {
            start_bit,
            end_bit: start_bit,
            entries: Vec::new(),
            stop: GameOptionsStop::Complete,
        },
    };
    use GameOptionsFieldKind as K;
    for (i, width) in [3, 3, 2, 7, 64, 32, 3, 32, 32, 32].into_iter().enumerate() {
        let kind = match i {
            0 => K::GameMode,
            1 => K::RecorderSimulation,
            9 => K::LevelTag,
            _ => K::Prefix(i as u8),
        };
        r.field(kind, width);
    }
    r.bond(GameOptionsBondKind::Options);
    r.string(0, 0x80);
    r.field(K::AfterShortName(0), 32);
    r.field(K::AfterShortName(1), 32);
    if r.field(K::OptionalBlockPresent, 1) == Some(1) {
        r.field(K::OptionalBlock, 0x700);
    }
    for (index, width) in [1, 1, 2, 1, 32, 32, 1].into_iter().enumerate() {
        r.field(K::BeforeVariant(index as u8), width);
    }
    if r.field(K::VariantPresent, 1) == Some(1) {
        r.bond(GameOptionsBondKind::Variant);
    }
    r.string(1, 0x100);
    r.string(2, 0x100);
    r.field(K::TailWord, 32);
    r.field(K::TailBlock, 0x6c0);
    r.out
}

struct Reader<'a> {
    data: &'a [u8],
    out: GameOptionsRead,
}
impl Reader<'_> {
    fn bits(&mut self, width: usize) -> BondBits {
        let bit = self.out.end_bit;
        let raw = RawBits::from_source(self.data, bit, width);
        if raw.is_some() {
            self.out.end_bit += width;
        } else {
            self.out.stop = GameOptionsStop::Truncated;
        }
        BondBits { bit, width, raw }
    }
    fn field(&mut self, kind: GameOptionsFieldKind, width: usize) -> Option<u64> {
        if self.out.stop != GameOptionsStop::Complete {
            return None;
        }
        let bits = self.bits(width);
        let value = bits
            .raw
            .as_ref()
            .filter(|_| width <= 64)
            .map(RawBits::low_u64);
        self.out
            .entries
            .push(GameOptionsEntry::Field { kind, bits });
        value
    }
    fn bond(&mut self, kind: GameOptionsBondKind) {
        if self.out.stop != GameOptionsStop::Complete {
            return;
        }
        let read = super::bond::read(self.data, self.out.end_bit);
        self.out.end_bit = read.root.source.end;
        if read.stop != BondStop::Complete {
            self.out.stop = GameOptionsStop::Bond(read.stop);
        }
        self.out.entries.push(GameOptionsEntry::Bond { kind, read });
    }
    fn string(&mut self, index: u8, limit: usize) {
        if self.out.stop != GameOptionsStop::Complete {
            return;
        }
        let mut bytes = Vec::new();
        let mut terminated = false;
        for _ in 0..limit {
            let byte = self.bits(8);
            terminated = byte.raw.as_ref().is_some_and(|r| r.low_u64() == 0);
            bytes.push(byte);
            if terminated || self.out.stop != GameOptionsStop::Complete {
                break;
            }
        }
        if !terminated && self.out.stop == GameOptionsStop::Complete {
            self.out.stop = GameOptionsStop::UnterminatedString;
        }
        self.out.entries.push(GameOptionsEntry::String {
            index,
            bytes,
            terminated,
        });
    }
}

#[cfg(test)]
mod tests;
