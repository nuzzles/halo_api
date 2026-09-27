//! Go encoding/json wire primitives used by the derived facts cache.
use std::collections::BTreeMap;
pub(super) trait FactsJsonValue {
    fn write_json(&self, w: &mut FactsJsonWriter);
}
#[derive(Default)]
pub(super) struct FactsJsonWriter {
    pub bytes: Vec<u8>,
    pub error: Option<String>,
}
impl FactsJsonWriter {
    pub fn raw(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }
    pub fn string(&mut self, bytes: &[u8]) {
        self.raw(b"\"");
        let mut rest = bytes;
        while !rest.is_empty() {
            let (valid, bad) = match std::str::from_utf8(rest) {
                Ok(s) => (s.len(), false),
                Err(e) => (e.valid_up_to(), true),
            };
            for ch in std::str::from_utf8(&rest[..valid]).unwrap().chars() {
                match ch {
                    '"' => self.raw(b"\\\""),
                    '\\' => self.raw(b"\\\\"),
                    '\n' => self.raw(b"\\n"),
                    '\r' => self.raw(b"\\r"),
                    '\t' => self.raw(b"\\t"),
                    '\u{8}' => self.raw(b"\\b"),
                    '\u{c}' => self.raw(b"\\f"),
                    c if c < ' ' || matches!(c, '<' | '>' | '&' | '\u{2028}' | '\u{2029}') => {
                        self.raw(format!("\\u{:04x}", c as u32).as_bytes())
                    }
                    c => {
                        let mut buf = [0; 4];
                        self.raw(c.encode_utf8(&mut buf).as_bytes());
                    }
                }
            }
            rest = &rest[valid..];
            if bad {
                self.raw(b"\\ufffd");
                rest = &rest[1..];
            }
        }
        self.raw(b"\"");
    }
    pub fn field<T: FactsJsonValue>(&mut self, name: &str, value: &T, first: &mut bool) {
        if !*first {
            self.raw(b",");
        }
        *first = false;
        self.string(name.as_bytes());
        self.raw(b":");
        value.write_json(self);
    }
}
macro_rules! number { ($($t:ty),*)=>{$(impl FactsJsonValue for $t { fn write_json(&self,w:&mut FactsJsonWriter){w.raw(self.to_string().as_bytes());} })*}; }
number!(u64, u32, u8, i64, i32, i8);
impl FactsJsonValue for bool {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(if *self { b"true" } else { b"false" });
    }
}
impl FactsJsonValue for f32 {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        if !self.is_finite() {
            if w.error.is_none() {
                w.error = Some(format!(
                    "json: unsupported value: {}",
                    if self.is_nan() {
                        "NaN"
                    } else if self.is_sign_negative() {
                        "-Inf"
                    } else {
                        "+Inf"
                    }
                ));
            }
            return;
        }
        let (mantissa, power) = super::facts_json_float::shortest(*self);
        if self.is_sign_negative() {
            w.raw(b"-");
        }
        if mantissa == 0 {
            w.raw(b"0");
            return;
        }
        let text = mantissa.to_string();
        let exponent = power + text.len() as i32 - 1;
        let digits = text.trim_end_matches('0');
        let abs = self.abs();
        if !(1e-6..1e21).contains(&abs) {
            w.raw(&digits.as_bytes()[..1]);
            if digits.len() > 1 {
                w.raw(b".");
                w.raw(&digits.as_bytes()[1..]);
            }
            w.raw(format!("e{exponent:+}").as_bytes());
        } else {
            let point = exponent + 1;
            if point <= 0 {
                w.raw(b"0.");
                for _ in 0..-point {
                    w.raw(b"0");
                }
                w.raw(digits.as_bytes());
            } else if point as usize >= digits.len() {
                w.raw(digits.as_bytes());
                for _ in digits.len()..point as usize {
                    w.raw(b"0");
                }
            } else {
                let point = point as usize;
                w.raw(&digits.as_bytes()[..point]);
                w.raw(b".");
                w.raw(&digits.as_bytes()[point..]);
            }
        }
    }
}
impl<T: FactsJsonValue> FactsJsonValue for Option<T> {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        if let Some(v) = self {
            v.write_json(w)
        } else {
            w.raw(b"null")
        }
    }
}
impl<T: FactsJsonValue> FactsJsonValue for [T] {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"[");
        for (i, v) in self.iter().enumerate() {
            if i != 0 {
                w.raw(b",");
            }
            v.write_json(w);
        }
        w.raw(b"]");
    }
}
impl<T: FactsJsonValue> FactsJsonValue for Vec<T> {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        self.as_slice().write_json(w)
    }
}
impl<T: FactsJsonValue, const N: usize> FactsJsonValue for [T; N] {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        self.as_slice().write_json(w)
    }
}
impl FactsJsonValue for BTreeMap<u32, i64> {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        let sorted: BTreeMap<_, _> = self.iter().map(|(k, v)| (k.to_string(), v)).collect();
        w.raw(b"{");
        for (i, (k, v)) in sorted.iter().enumerate() {
            if i != 0 {
                w.raw(b",");
            }
            w.string(k.as_bytes());
            w.raw(b":");
            v.write_json(w);
        }
        w.raw(b"}");
    }
}
impl FactsJsonValue for BTreeMap<Vec<u8>, i64> {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        for (i, (k, v)) in self.iter().enumerate() {
            if i != 0 {
                w.raw(b",");
            }
            w.string(k);
            w.raw(b":");
            v.write_json(w);
        }
        w.raw(b"}");
    }
}
impl<T: FactsJsonValue> FactsJsonValue for BTreeMap<i64, T> {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        let sorted: BTreeMap<_, _> = self.iter().map(|(k, v)| (k.to_string(), v)).collect();
        w.raw(b"{");
        for (i, (k, v)) in sorted.iter().enumerate() {
            if i != 0 {
                w.raw(b",");
            }
            w.string(k.as_bytes());
            w.raw(b":");
            v.write_json(w);
        }
        w.raw(b"}");
    }
}
