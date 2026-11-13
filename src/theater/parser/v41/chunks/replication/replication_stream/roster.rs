//! Count-driven type 8/12 readers. Shared body: reference analysis 413971b
//! player_table_record.go; wrappers: roster_type8.go and bot_metadata.go.
use crate::theater::parser::packet::BitRange;
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;
use crate::theater::parser::v41::chunks::replication::source::{
    PlayerMetadataEntry, PlayerMetadataField, PlayerMetadataFieldKind as K, PlayerMetadataRead,
    PlayerMetadataStop as Stop,
};
use crate::theater::parser::v41::context::players::PlayerContext;

struct Reader<'a> {
    data: &'a [u8],
    bit: usize,
    fields: Vec<PlayerMetadataField>,
}
impl Reader<'_> {
    fn field(&mut self, kind: K, width: usize) -> Result<(), Stop> {
        let raw = RawBits::from_source(self.data, self.bit, width);
        let available = raw.is_some();
        self.fields.push(PlayerMetadataField {
            kind,
            bit: self.bit,
            width,
            raw,
        });
        if !available {
            return Err(Stop::Truncated);
        }
        self.bit += width;
        Ok(())
    }
    fn word(&mut self, kind: K, width: usize) -> Result<u64, Stop> {
        self.field(kind, width)?;
        Ok(self.fields.last().unwrap().raw.as_ref().unwrap().low_u64())
    }
    fn count(&mut self, kind: K, width: usize, limit: u64) -> Result<usize, Stop> {
        let count = self.word(kind, width)?;
        if count > limit {
            return Err(Stop::CountLimit { kind, count, limit });
        }
        Ok(count as usize)
    }
    fn body(&mut self, context: &PlayerContext) -> Result<(), Stop> {
        let mask = self.word(K::MaskLengthMinusOne, 11)? as usize + 1;
        self.field(K::Mask, mask)?;
        let bytes = self.count(K::ByteCount, 12, 2048)?;
        self.field(K::ByteList, bytes * 8)?;
        let words = self.count(K::WordCount, 8, 192)?;
        self.field(K::WordList, words * 32)?;
        self.field(K::Block104, 832)?;
        for i in 0..16 {
            if self.word(K::GamertagUnit(i), 16)? == 0 {
                break;
            }
        }
        self.field(K::Block16, 128)?;
        self.word(K::Representation, 32)?;
        self.word(K::JoinKey, 64)?;
        for (i, width) in [10, 14, 6, 8, 7, 1].into_iter().enumerate() {
            self.word(K::Short(i as u8), width)?;
        }
        let bytes = context
            .personalization_bytes
            .ok_or(Stop::UnknownPersonalization)?;
        self.field(K::Personalization, bytes * 8)?;
        for i in 0..44 {
            self.field(K::Block44Byte(i), 8)?;
        }
        Ok(())
    }
    fn entry(&mut self, bots: bool, context: &PlayerContext) -> Result<(), Stop> {
        // Admission is explicit. A newer, unknown format never inherits 27.
        let version = context
            .format_version
            .filter(|v| matches!(v, 20 | 21 | 24 | 25 | 27))
            .ok_or(Stop::UnsupportedFormat)?;
        if bots {
            self.word(K::AbsoluteIndex, 32)?;
            self.word(K::Slot, 32)?;
            self.word(K::BotId, 32)?;
        } else {
            if version >= 21 && self.word(K::OptionAbsent, 1)? == 0 {
                self.word(K::Option, 5)?;
            }
            self.word(K::Identity, 64)?;
        }
        self.body(context)
    }
}

pub(crate) fn decode(data: &[u8], bots: bool, context: &PlayerContext) -> PlayerMetadataRead {
    let mut r = Reader {
        data,
        bit: 0,
        fields: Vec::new(),
    };
    let count = r.word(K::Count, 32);
    let mut stop = count.as_ref().err().copied().unwrap_or(Stop::Complete);
    let mut out = PlayerMetadataRead {
        format_version: context.format_version,
        build: context.build.clone(),
        personalization_bytes: context.personalization_bytes,
        count: r.fields.pop().unwrap(),
        entries: Vec::new(),
        end_bit: r.bit,
        stop,
        tail: BitRange {
            start: r.bit,
            end: data.len() * 8,
        },
    };
    if let Ok(count) = count {
        if bots && count > 64 {
            stop = Stop::CountLimit {
                kind: K::Count,
                count,
                limit: 64,
            };
        } else {
            // No allocation from an untrusted count. Each successful iteration
            // consumes a nonempty entry, and the first refusal ends traversal.
            for _ in 0..count {
                let start_bit = r.bit;
                stop = r.entry(bots, context).err().unwrap_or(Stop::Complete);
                out.entries.push(PlayerMetadataEntry {
                    start_bit,
                    end_bit: r.bit,
                    fields: std::mem::take(&mut r.fields),
                    stop,
                });
                if stop != Stop::Complete {
                    break;
                }
            }
            if bots && stop == Stop::Complete && data.len() * 8 - r.bit >= 8 {
                stop = Stop::TrailingData;
            }
        }
    }
    out.end_bit = r.bit;
    out.tail.start = r.bit;
    out.stop = stop;
    out
}

#[cfg(test)]
mod tests;
