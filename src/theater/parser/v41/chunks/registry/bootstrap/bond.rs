//! Bounded Compact Binary v2 reader. Protocol rules match Microsoft Bond's
//! compact_binary.h; independent comparison uses reference reader variant.rs.
use crate::theater::parser::packet::BitRange;
use crate::theater::parser::v41::chunks::registry::source::bond::*;
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;

const MAX_DEPTH: usize = 64;
const MAX_VALUES: usize = 100_000;

pub(super) fn read(data: &[u8], start: usize) -> BondRead {
    read_bounded(
        data,
        BitRange {
            start,
            end: data.len().saturating_mul(8).max(start),
        },
    )
}

/// Read one length-prefixed structure within an already established envelope.
/// The limit can end at any bit; later packet fields cannot satisfy this read.
pub(crate) fn read_bounded(data: &[u8], source: BitRange) -> BondRead {
    let mut r = Reader {
        data,
        at: source.start,
        stop: if source.start > source.end {
            BondStop::InvalidLength
        } else if source.end > data.len().saturating_mul(8) {
            BondStop::Truncated
        } else {
            BondStop::Complete
        },
        values: 0,
    };
    let root = r.value(10, source.end, 0);
    BondRead { root, stop: r.stop }
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
    stop: BondStop,
    values: usize,
}
impl Reader<'_> {
    fn ok(&self) -> bool {
        self.stop == BondStop::Complete
    }

    fn bits(&mut self, width: usize, limit: usize) -> BondBits {
        let bit = self.at;
        let end = bit.checked_add(width);
        let raw = if !self.ok() {
            None
        } else if end.is_none_or(|n| n > limit) {
            self.stop = if limit < self.data.len().saturating_mul(8) {
                BondStop::StructureBoundary
            } else {
                BondStop::Truncated
            };
            None
        } else {
            RawBits::from_source(self.data, bit, width)
        };
        if raw.is_some() {
            self.at += width;
        } else if self.ok() {
            self.stop = BondStop::Truncated;
        }
        BondBits { bit, width, raw }
    }

    fn varint(&mut self, width: usize, limit: usize) -> BondVarint {
        let mut out = BondVarint {
            bytes: Vec::new(),
            value: None,
        };
        let mut value = 0u64;
        for i in 0..width.div_ceil(7) {
            if !self.ok() {
                break;
            }
            let byte = self.bits(8, limit);
            let raw = byte.raw.as_ref().map(|r| r.low_u64());
            out.bytes.push(byte);
            let Some(raw) = raw else {
                break;
            };
            let payload = raw & 127;
            let remaining = width - 7 * i;
            if remaining < 7 && payload >= 1 << remaining {
                self.stop = BondStop::IntegerOverflow;
                break;
            }
            value |= payload << (7 * i);
            if raw & 128 == 0 {
                out.value = Some(value);
                return out;
            }
        }
        if self.ok() {
            self.stop = BondStop::IntegerOverflow;
        }
        out
    }

    fn value(&mut self, ty: u8, limit: usize, depth: usize) -> BondValue {
        let start = self.at;
        let mut out = BondValue {
            wire_type: ty,
            source: BitRange { start, end: start },
            body: BondValueBody::Unread,
        };
        if !self.ok() {
            return out;
        }
        if depth > MAX_DEPTH {
            self.stop = BondStop::DepthLimit;
            return out;
        }
        if self.values >= MAX_VALUES {
            self.stop = BondStop::ValueLimit;
            return out;
        }
        self.values += 1;
        out.body = match ty {
            2 | 3 | 14 => BondValueBody::Fixed(self.bits(8, limit)),
            7 => BondValueBody::Fixed(self.bits(32, limit)),
            8 => BondValueBody::Fixed(self.bits(64, limit)),
            4 | 15 => BondValueBody::Integer(self.varint(16, limit)),
            5 | 16 => BondValueBody::Integer(self.varint(32, limit)),
            6 | 17 => BondValueBody::Integer(self.varint(64, limit)),
            9 | 18 => {
                let length = self.varint(32, limit);
                let data = length.value.and_then(|n| {
                    let width =
                        usize::try_from(n)
                            .ok()?
                            .checked_mul(if ty == 18 { 16 } else { 8 })?;
                    Some(self.bits(width, limit))
                });
                if self.ok() && data.is_none() {
                    self.stop = BondStop::InvalidLength;
                }
                BondValueBody::String { length, data }
            }
            10 => self.structure(limit, depth),
            11 | 12 => self.sequence(limit, depth),
            13 => self.map(limit, depth),
            _ => {
                self.stop = BondStop::InvalidType(ty);
                BondValueBody::Unread
            }
        };
        out.source.end = self.at;
        out
    }

    fn field(&mut self, limit: usize) -> BondField {
        let first = self.bits(8, limit);
        let raw = first.raw.as_ref().map(|r| r.low_u64() as u8);
        let mut out = BondField {
            header: vec![first],
            id: None,
            wire_type: raw.map(|v| v & 31),
            value: None,
        };
        let Some(raw) = raw else {
            return out;
        };
        let id = raw >> 5;
        if id < 6 {
            out.id = Some(u16::from(id));
        } else {
            let encoded = self.bits(if id == 6 { 8 } else { 16 }, limit);
            out.id = encoded.raw.as_ref().map(|r| {
                if id == 6 {
                    r.low_u64() as u16
                } else {
                    u16::from_le_bytes([r.bytes[0], r.bytes[1]])
                }
            });
            out.header.push(encoded);
        }
        out
    }

    fn structure(&mut self, limit: usize, depth: usize) -> BondValueBody {
        let length = self.varint(32, limit);
        let mut fields = Vec::new();
        if let Some(n) = length.value {
            let end = usize::try_from(n)
                .ok()
                .and_then(|n| n.checked_mul(8))
                .and_then(|n| self.at.checked_add(n));
            if let Some(end) = end {
                let bound = end.min(limit);
                let mut ended = false;
                while self.ok() && self.at < bound {
                    if self.values >= MAX_VALUES {
                        self.stop = BondStop::ValueLimit;
                        break;
                    }
                    self.values += 1;
                    let mut field = self.field(bound);
                    if self.ok() {
                        match field.wire_type.unwrap() {
                            0 => {
                                ended = true;
                                if self.at != end {
                                    self.stop = BondStop::InvalidLength;
                                }
                            }
                            1 => {}
                            ty => field.value = Some(self.value(ty, bound, depth + 1)),
                        }
                    }
                    fields.push(field);
                    if ended {
                        break;
                    }
                }
                if self.ok() && (!ended || end > limit) {
                    self.stop = if end > limit {
                        if limit < self.data.len().saturating_mul(8) {
                            BondStop::StructureBoundary
                        } else {
                            BondStop::Truncated
                        }
                    } else {
                        BondStop::InvalidLength
                    };
                }
            } else {
                self.stop = BondStop::InvalidLength;
            }
        }
        BondValueBody::Struct { length, fields }
    }

    fn sequence(&mut self, limit: usize, depth: usize) -> BondValueBody {
        let header = self.bits(8, limit);
        let raw = header.raw.as_ref().map(|r| r.low_u64() as u8);
        let mut count_encoding = None;
        let count = raw.and_then(|raw| {
            if raw >> 5 != 0 {
                Some(u32::from((raw >> 5) - 1))
            } else {
                let encoded = self.varint(32, limit);
                let n = encoded.value.map(|n| n as u32);
                count_encoding = Some(encoded);
                n
            }
        });
        let mut values = Vec::new();
        let mut packed = None;
        if let (Some(raw), Some(count)) = (raw, count) {
            let ty = raw & 31;
            if !(2..=18).contains(&ty) {
                self.stop = BondStop::InvalidType(ty);
            }
            let width = match ty {
                2 | 3 | 14 => Some(8),
                7 => Some(32),
                8 => Some(64),
                _ => None,
            };
            if let Some(width) = width {
                if let Some(width) = (count as usize).checked_mul(width) {
                    packed = Some(self.bits(width, limit));
                } else {
                    self.stop = BondStop::InvalidLength;
                }
            } else {
                for _ in 0..count {
                    if !self.ok() {
                        break;
                    }
                    values.push(self.value(ty, limit, depth + 1));
                }
            }
        }
        BondValueBody::Sequence {
            header,
            count_encoding,
            count,
            packed,
            values,
        }
    }

    fn map(&mut self, limit: usize, depth: usize) -> BondValueBody {
        let key_type = self.bits(8, limit);
        let value_type = self.ok().then(|| self.bits(8, limit));
        let count = self.ok().then(|| self.varint(32, limit));
        let mut entries = Vec::new();
        if self.ok() {
            let kt = key_type.raw.as_ref().unwrap().low_u64() as u8;
            let vt = value_type.as_ref().unwrap().raw.as_ref().unwrap().low_u64() as u8;
            for ty in [kt, vt] {
                if !(2..=18).contains(&ty) {
                    self.stop = BondStop::InvalidType(ty);
                    break;
                }
            }
            for _ in 0..count.as_ref().and_then(|n| n.value).unwrap_or(0) {
                if !self.ok() {
                    break;
                }
                let key = self.value(kt, limit, depth + 1);
                let value = self.ok().then(|| self.value(vt, limit, depth + 1));
                entries.push(BondMapEntry { key, value });
            }
        }
        BondValueBody::Map {
            key_type,
            value_type,
            count,
            entries,
        }
    }
}

#[cfg(test)]
mod tests;
