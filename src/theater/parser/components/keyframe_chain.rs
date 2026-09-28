//! Deterministic native chaining between independently established keyframe boundaries.
use super::{
    FrameEncoding, KeyframeRecord, KeyframeStop, decode_native_keyframe_record_contextual,
};
use crate::theater::parser::{
    FilmReadDiagnostics, FilmRegistry, NativeReaderContext, native_bits_at,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyframeChainStop {
    #[serde(rename = "fin-du-payload")]
    End,
    #[serde(rename = "en-tete-invalide")]
    Header,
    #[serde(rename = "composant-non-porte")]
    Desync,
    #[serde(rename = "slot-non-croissant")]
    Slot,
    #[serde(rename = "budget-epuise")]
    Budget,
    InvalidEncoding,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeChainAttempt {
    pub start_bit: i64,
    pub end_bit: i64,
    pub id: u32,
    pub archetype: u32,
    /// Absent for the no-archetype entry, whose body is not read.
    pub record: Option<KeyframeRecord>,
}

fn native_header(data: &[u8], pos: i64) -> Option<(u32, u32)> {
    if pos < 0 || pos.wrapping_add(64) > (data.len() as i64 * 8) {
        return None;
    }
    let id = native_bits_at(data, pos, 32) as u32;
    if id == u32::MAX || id >> 30 == 0 || id & 0x3fff_ffff >= 8192 {
        return None;
    }
    let archetype = native_bits_at(data, pos.wrapping_add(32), 32) as u32;
    (archetype < 50 || archetype == u32::MAX).then_some((id, archetype))
}

impl KeyframeChainAttempt {
    fn complete(&self) -> bool {
        self.record
            .as_ref()
            .is_none_or(|record| record.stop == KeyframeStop::Complete)
    }
}

fn read_attempt(
    data: &[u8],
    registry: &FilmRegistry,
    start: i64,
    id: u32,
    archetype: u32,
    encoding: &FrameEncoding,
    context: Option<&NativeReaderContext>,
) -> Option<KeyframeChainAttempt> {
    if context.is_none() && !encoding.keyframe_layout.valid() {
        return None;
    }
    let (end_bit, record) = if archetype == u32::MAX {
        (
            start.wrapping_add(
                context.map_or(encoding.keyframe_layout.header_bits as i64, |c| {
                    c.profile.keyframe.header_bits
                }),
            ),
            None,
        )
    } else {
        let record =
            decode_native_keyframe_record_contextual(data, start, registry, encoding, context)?;
        (record.end_bit, Some(record))
    };
    Some(KeyframeChainAttempt {
        start_bit: start,
        end_bit,
        id,
        archetype,
        record,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeKeyframeTable {
    pub records: Vec<KeyframeChainAttempt>,
    pub stop: KeyframeChainStop,
    pub diagnostics: FilmReadDiagnostics,
}

pub(crate) fn decode_native_keyframe_table_contextual(
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    context: Option<&NativeReaderContext>,
) -> NativeKeyframeTable {
    let mut out = NativeKeyframeTable {
        records: Vec::new(),
        stop: KeyframeChainStop::End,
        diagnostics: Default::default(),
    };
    let (mut pos, mut previous) = (1i64, None);

    loop {
        if out.records.len() >= 16_384 {
            out.stop = KeyframeChainStop::Budget;
            return out;
        }
        if pos.wrapping_add(64) > (data.len() as i64 * 8) {
            return out;
        }
        let Some((id, archetype)) = native_header(data, pos) else {
            // Native table traversal checks the sentinel even after a rejected
            // negative header. This read may panic, unlike chain traversal.
            if native_bits_at(data, pos, 32) != u32::MAX as u64 {
                out.stop = KeyframeChainStop::Header;
            }
            return out;
        };
        let slot = id & 0x3fff_ffff;
        if previous.is_some_and(|last| slot <= last) {
            out.stop = KeyframeChainStop::Slot;
            return out;
        }
        let Some(attempt) = read_attempt(data, registry, pos, id, archetype, encoding, context)
        else {
            out.stop = KeyframeChainStop::InvalidEncoding;
            return out;
        };
        let complete = attempt.complete();
        if let Some(record) = &attempt.record {
            out.diagnostics.merge(&record.diagnostics);
        }
        pos = attempt.end_bit;
        out.records.push(attempt);
        if !complete {
            out.stop = KeyframeChainStop::Desync;
            return out;
        }
        previous = Some(slot);
    }
}
