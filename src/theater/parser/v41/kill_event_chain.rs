//! Kill-event localization grammar from the pinned native killsource decoder.
//! A localized event is evidence only; roster resolution and kill-feed matching
//! must happen before its assistant or damage shares can be published.
use super::bits::{Bits, Cursor};
pub(crate) use crate::theater::film::replication::kill_event_chain::{
    KillEventFields, NativeEventField, NativeEventFieldStage, NativeEventFieldValue,
    NativeEventListRead, NativeEventListStop, NativeEventRecord,
};
use serde::{Deserialize, Serialize};

const CONFIG: [[i8; 3]; 123] = [
    [1, 1, 7],    // 0
    [1, 8, 7],    // 1
    [1, 8, 7],    // 2
    [0, 8, 7],    // 3
    [0, 8, 7],    // 4
    [5, 8, 7],    // 5
    [5, 8, 7],    // 6
    [1, 8, 7],    // 7
    [2, 3, 7],    // 8
    [2, 8, 7],    // 9
    [8, 8, 7],    // 10
    [2, 8, 7],    // 11
    [8, 8, 7],    // 12
    [4, 8, 7],    // 13
    [-1, -1, -1], // 14
    [8, 8, 7],    // 15
    [8, 8, 7],    // 16
    [-1, -1, -1], // 17
    [6, 8, 7],    // 18
    [0, 8, 7],    // 19
    [1, 1, 7],    // 20
    [4, 8, 7],    // 21
    [1, 1, 7],    // 22
    [0, 8, 7],    // 23
    [0, 2, 7],    // 24
    [0, 2, 7],    // 25
    [0, 2, 7],    // 26
    [0, 0, 7],    // 27
    [0, 8, 7],    // 28
    [6, 8, 7],    // 29
    [4, 8, 7],    // 30
    [1, 8, 7],    // 31
    [1, 8, 7],    // 32
    [3, 4, 7],    // 33
    [8, 8, 7],    // 34
    [1, 8, 7],    // 35
    [1, 8, 7],    // 36
    [1, 8, 7],    // 37
    [2, 8, 7],    // 38
    [2, 8, 7],    // 39
    [2, 0, 7],    // 40
    [3, 8, 7],    // 41
    [2, 8, 7],    // 42
    [2, 0, 7],    // 43
    [2, 0, 7],    // 44
    [2, 8, 7],    // 45
    [4, 8, 7],    // 46
    [4, 8, 7],    // 47
    [4, 8, 7],    // 48
    [3, 2, 7],    // 49
    [-1, -1, -1], // 50
    [2, 8, 7],    // 51
    [2, 0, 7],    // 52
    [1, 1, 7],    // 53
    [1, 1, 7],    // 54
    [6, 8, 7],    // 55
    [5, 1, 7],    // 56
    [2, 0, 7],    // 57
    [1, 8, 7],    // 58
    [0, 8, 7],    // 59
    [0, 8, 7],    // 60
    [0, 8, 7],    // 61
    [6, 8, 7],    // 62
    [2, 8, 7],    // 63
    [6, 8, 7],    // 64
    [3, 8, 7],    // 65
    [6, 8, 7],    // 66
    [6, 8, 7],    // 67
    [6, 8, 7],    // 68
    [0, 8, 7],    // 69
    [2, 8, 7],    // 70
    [2, 8, 7],    // 71
    [2, 8, 7],    // 72
    [2, 8, 7],    // 73
    [2, 8, 7],    // 74
    [1, 8, 7],    // 75
    [1, 8, 7],    // 76
    [6, 8, 7],    // 77
    [4, 8, 7],    // 78
    [-1, -1, -1], // 79
    [8, 8, 7],    // 80
    [0, 8, 7],    // 81
    [0, 8, 7],    // 82
    [0, 8, 7],    // 83
    [0, 8, 7],    // 84
    [8, 8, 7],    // 85
    [1, 8, 7],    // 86
    [6, 8, 7],    // 87
    [6, 8, 7],    // 88
    [6, 8, 7],    // 89
    [6, 8, 7],    // 90
    [6, 8, 7],    // 91
    [6, 8, 7],    // 92
    [2, 0, 7],    // 93
    [6, 8, 7],    // 94
    [6, 8, 7],    // 95
    [6, 8, 7],    // 96
    [6, 8, 7],    // 97
    [1, 1, 7],    // 98
    [8, 8, 7],    // 99
    [1, 8, 7],    // 100
    [6, 8, 7],    // 101
    [2, 8, 7],    // 102
    [0, 0, 7],    // 103
    [0, -1, -1],  // 104
    [0, 0, 7],    // 105
    [0, -1, -1],  // 106
    [0, 8, 7],    // 107
    [6, 8, 7],    // 108
    [0, 0, 7],    // 109
    [1, 8, 7],    // 110
    [6, 8, 7],    // 111
    [6, 8, 7],    // 112
    [6, 8, 7],    // 113
    [6, 8, 7],    // 114
    [-1, -1, -1], // 115
    [-1, -1, -1], // 116
    [-1, -1, -1], // 117
    [1, 8, 7],    // 118
    [2, 0, 7],    // 119
    [8, 8, 7],    // 120
    [8, 8, 7],    // 121
    [8, 8, 7],    // 122
];

impl KillEventFields {
    pub(crate) fn plausible(&self) -> bool {
        self.end > 0 && self.killer >= 0 && self.victim >= 0 && self.killer != self.victim
    }
}

/// Walk an event list from an established continuation bit. This reuses the
/// pinned killsource event *layout* grammar, without localization, filtering or
/// roster inference. Unlike the native scanner's selected runtime fallback,
/// an unspecified code-15 gate stops before its body. Fixed-size skipped bodies
/// are retained as opaque ranges, not advertised as interpreted fields.
pub(crate) fn read_native_event_list(
    data: &[u8],
    start_bit: usize,
    gate15: Option<bool>,
    limit: usize,
) -> NativeEventListRead {
    let mut out = NativeEventListRead {
        start_bit,
        end_bit: start_bit,
        gate15,
        records: Vec::new(),
        fields: Vec::new(),
        stop: NativeEventListStop::InvalidStart,
    };
    let Some(mut r) = Reader::new(data, start_bit) else {
        return out;
    };
    r.trace = Some(Vec::new());
    loop {
        if out.records.len() >= limit {
            out.stop = NativeEventListStop::RecordLimit;
            break;
        }
        if r.cursor.position >= data.len() * 8 {
            out.stop = NativeEventListStop::SourceBoundary;
            break;
        }
        let start = r.cursor.position;
        let field_start = r.trace.as_ref().unwrap().len();
        r.stage = NativeEventFieldStage::Header;
        if r.read(1) == 0 {
            out.stop = if r.over {
                NativeEventListStop::Truncated
            } else {
                NativeEventListStop::Terminator
            };
            break;
        }
        let code = r.read(7) as usize;
        let mut record = NativeEventRecord {
            start_bit: start,
            end_bit: r.cursor.position,
            code: (!r.over).then_some(code as u8),
            body_start_bit: None,
            field_range: [field_start, 0],
            layout_complete: false,
            kill_fields: None,
        };
        let stop = if r.over {
            Some(NativeEventListStop::Truncated)
        } else if code >= CONFIG.len() {
            Some(NativeEventListStop::UnsupportedCode)
        } else {
            r.stage = NativeEventFieldStage::References;
            if !presence(&mut r, code) {
                Some(if r.over {
                    NativeEventListStop::Truncated
                } else {
                    NativeEventListStop::UnsupportedReferences
                })
            } else {
                record.body_start_bit = Some(r.cursor.position);
                r.stage = NativeEventFieldStage::Body;
                if code == 85 {
                    record.kill_fields = read_kill_event_fields(data, r.cursor.position);
                }
                if code == 15 && gate15.is_none() {
                    Some(NativeEventListStop::MissingRuntimeGate15)
                } else if body(&mut r, data, code, gate15.unwrap_or(false)) {
                    record.layout_complete = true;
                    None
                } else {
                    Some(if r.over {
                        NativeEventListStop::Truncated
                    } else {
                        NativeEventListStop::UnsupportedBody
                    })
                }
            }
        };
        record.end_bit = r.cursor.position;
        record.field_range[1] = r.trace.as_ref().unwrap().len();
        out.records.push(record);
        if let Some(stop) = stop {
            out.stop = stop;
            break;
        }
    }
    out.end_bit = r.cursor.position;
    out.fields = r.trace.unwrap();
    out
}

// A failed native read leaves the position unchanged but sticks an overflow flag.
// Later short reads still run. Preserve that behavior for raw truncated fields.
struct Reader<'a> {
    cursor: Cursor<'a>,
    over: bool,
    trace: Option<Vec<NativeEventField>>,
    stage: NativeEventFieldStage,
}
impl<'a> Reader<'a> {
    fn new(data: &'a [u8], position: usize) -> Option<Self> {
        Some(Self {
            cursor: Cursor::new(data, position)?,
            over: false,
            trace: None,
            stage: NativeEventFieldStage::Body,
        })
    }
    fn read(&mut self, n: usize) -> u64 {
        // Native wide reads consume all requested bits, even beyond 64 bits.
        if n > 64 {
            self.skip(n);
            return 0;
        }
        let bit = self.cursor.position;
        let value = self.cursor.read(n);
        if let Some(trace) = &mut self.trace {
            trace.push(NativeEventField {
                bit,
                width: n,
                stage: self.stage,
                value: value.map_or(
                    NativeEventFieldValue::Unavailable,
                    NativeEventFieldValue::Scalar,
                ),
            });
        }
        value.unwrap_or_else(|| {
            self.over = true;
            0
        })
    }
    fn skip(&mut self, n: usize) {
        let bit = self.cursor.position;
        let ok = self.cursor.skip(n).is_some();
        if let Some(trace) = &mut self.trace {
            trace.push(NativeEventField {
                bit,
                width: n,
                stage: self.stage,
                value: if ok {
                    NativeEventFieldValue::Opaque
                } else {
                    NativeEventFieldValue::Unavailable
                },
            });
        }
        if !ok {
            self.over = true;
        }
    }
    fn entity5(&mut self) -> i32 {
        if self.read(1) != 0 {
            -1
        } else {
            self.read(5) as i32
        }
    }
}

/// Decode mandatory code-85 body fields at an established bit position.
/// This does not validate the surrounding chain or assign player identities.
pub(crate) fn read_kill_event_fields(data: &[u8], body: usize) -> Option<KillEventFields> {
    let mut r = Reader::new(data, body)?;
    Some(kill_fields(&mut r))
}

fn kill_fields(r: &mut Reader<'_>) -> KillEventFields {
    let victim = r.entity5();
    let killer = r.entity5();
    let killer_pct = r.read(32) as u32;
    let flag = r.read(1) as u8;
    let assist = r.entity5();
    let assist_pct = r.read(32) as u32;
    KillEventFields {
        killer,
        victim,
        assist,
        killer_pct,
        assist_pct,
        flag,
        end: if r.over { -1 } else { r.cursor.position as i64 },
    }
}
fn presence(r: &mut Reader<'_>, code: usize) -> bool {
    const RANGE: [u32; 10] = [7679, 7679, 256, 256, 512, 256, 512, 8191, 8191, 0];
    for mut cfg in CONFIG[code] {
        if r.read(1) == 0 {
            continue;
        }
        if cfg < 0 {
            return false;
        }
        if cfg == 1 && r.read(1) == 1 {
            cfg = 4;
        }
        let range = RANGE[cfg as usize];
        if range > 0 {
            r.read((32 - (range - 1).leading_zeros()) as usize);
        }
        r.read(2);
    }
    !r.over
}
fn body(r: &mut Reader<'_>, data: &[u8], code: usize, gate15: bool) -> bool {
    let fixed = match code {
        3 | 4 | 23 | 24 | 25 | 26 | 33 | 49 | 54 | 57 | 59 | 92 | 103 => Some(0),
        5 => Some(111),
        6 => Some(93),
        7 => Some(118),
        9 => Some(36),
        12 => Some(94),
        21 => Some(2),
        34 => Some(59),
        38 => Some(10),
        40 => Some(78),
        75 => Some(54),
        76 => Some(104),
        _ => None,
    };
    if let Some(n) = fixed {
        r.skip(n);
        return !r.over;
    }
    match code {
        85 => {
            let mut nested = Reader::new(data, r.cursor.position).unwrap();
            nested.trace = r.trace.as_ref().map(|_| Vec::new());
            let k = kill_fields(&mut nested);
            if let (Some(trace), Some(fields)) = (&mut r.trace, nested.trace) {
                trace.extend(fields);
            }
            if k.end < 0 {
                r.over = true;
                return false;
            }
            r.cursor.position = k.end as usize;
        }
        1 => {
            r.read(5);
            if r.read(1) == 0 {
                r.read(4);
            }
            r.read(3);
            if r.read(1) != 0 {
                r.read(19);
            }
        }
        15 => {
            if gate15 {
                r.read(15);
            }
            r.read(13);
            let n = r.read(10) as usize;
            r.skip(n);
        }
        0 => {
            if r.read(1) != 0 {
                r.read(32);
            }
            if r.read(1) == 0 {
                r.read(5);
            }
            r.read(19);
            if r.read(1) != 0 {
                r.read(19);
                r.read(12);
            }
            r.read(5);
            r.read(5);
            r.read(6);
            if r.read(1) != 0 {
                r.read(5);
            }
            r.read(14);
            if r.read(1) != 0 {
                r.read(32);
            }
            r.read(1);
            r.read(3);
            r.read(5);
            r.read(5);
            r.read(1);
            r.read(4);
            if r.read(1) == 0 {
                r.read(10);
            }
            let v58 = r.read(4);
            if r.read(1) != 0 {
                r.read(32);
            }
            if v58 == 1 {
                r.read(8);
            }
            r.read(4);
            if r.read(1) != 0 {
                r.read(13);
                r.read(2);
            }
        }
        82 => {
            r.read(32);
            r.read(8);
            let n = r.read(3);
            for _ in 0..n {
                r.read(32);
                match r.read(3) {
                    0 => {}
                    1 | 2 | 3 | 6 => {
                        r.read(32);
                    }
                    4 => {
                        r.read(1);
                    }
                    5 => r.skip(128),
                    _ => return false,
                }
            }
            if r.read(1) == 1 {
                r.read(32);
                let n = r.read(3);
                for _ in 0..n {
                    match r.read(3) {
                        0 => {}
                        1 => {
                            if r.read(1) == 0 {
                                r.read(5);
                            }
                        }
                        2 => {
                            if r.read(1) == 0 {
                                r.read(32);
                            } else {
                                r.read(24);
                            }
                        }
                        _ => {
                            r.read(32);
                        }
                    }
                }
            }
            r.read(32);
        }
        _ => return false,
    }
    !r.over
}
/// Number of complete supported events before termination, refusal, or the limit.
pub(crate) fn kill_event_chain_length(
    data: &[u8],
    position: usize,
    gate15: bool,
    limit: usize,
) -> usize {
    let Some(mut r) = Reader::new(data, position) else {
        return 0;
    };
    let mut count = 0;
    while count < limit && r.cursor.position < data.len() * 8 {
        if r.read(1) == 0 {
            break;
        }
        let code = r.read(7) as usize;
        if r.over
            || code >= CONFIG.len()
            || !presence(&mut r, code)
            || !body(&mut r, data, code, gate15)
        {
            break;
        }
        count += 1;
    }
    count
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LocalizedKillEvent {
    /// First code bit; the continuation bit immediately precedes it.
    pub bit: usize,
    pub fields: KillEventFields,
    pub chain: usize,
}
/// Scan code-85 candidates, requiring three subsequent supported events.
/// Unknown bodies stop validation; no guessed boundary or content cap is used.
pub(crate) fn scan_kill_event_chains(data: &[u8], gate15: bool) -> Vec<LocalizedKillEvent> {
    let bits = Bits(data);
    let mut out = Vec::new();
    for x in 1..bits.len().saturating_sub(7) {
        if bits.read(x - 1, 1) != Some(1) || bits.read(x, 7) != Some(85) {
            continue;
        }
        let mut r = Reader::new(data, x + 7).unwrap();
        if !presence(&mut r, 85) {
            continue;
        }
        let fields = read_kill_event_fields(data, r.cursor.position).unwrap();
        if !fields.plausible() {
            continue;
        }
        let chain = kill_event_chain_length(data, fields.end as usize, gate15, 12);
        if chain >= 3 {
            out.push(LocalizedKillEvent {
                bit: x,
                fields,
                chain,
            });
        }
    }
    out
}
