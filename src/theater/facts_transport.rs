//! Transport for LevelUp's derived FilmFacts cache, not native recording bytes.
#[derive(Debug, Clone, Default)]
pub struct NativeFactsWriter {
    bytes: Vec<u8>,
    error: Option<String>,
}
impl NativeFactsWriter {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    /// Native writers keep the first embedded-payload error while accumulating.
    pub fn fail(&mut self, error: impl Into<String>) {
        if self.error.is_none() {
            self.error = Some(error.into());
        }
    }
    pub(crate) fn replace_error(&mut self, error: String) {
        self.error = Some(error);
    }
    pub fn unsigned(&mut self, mut v: u64) {
        while v >= 128 {
            self.bytes.push(v as u8 | 128);
            v >>= 7;
        }
        self.bytes.push(v as u8);
    }
    pub fn signed(&mut self, v: i64) {
        self.unsigned(((v as u64) << 1) ^ ((v >> 63) as u64));
    }
    pub fn byte(&mut self, v: u8) {
        self.bytes.push(v);
    }
    pub fn float32(&mut self, v: f32) {
        self.bytes.extend_from_slice(&v.to_bits().to_le_bytes());
    }
    pub fn float64(&mut self, v: f64) {
        self.bytes.extend_from_slice(&v.to_bits().to_le_bytes());
    }
    pub fn string_bytes(&mut self, v: &[u8]) {
        self.unsigned(v.len() as u64);
        self.bytes.extend_from_slice(v);
    }
    pub fn boolean(&mut self, v: bool) {
        self.byte(u8::from(v));
    }
}
#[derive(Debug, Clone)]
pub struct NativeFactsReader<'a> {
    bytes: &'a [u8],
    offset: usize,
    error: Option<String>,
}
impl<'a> NativeFactsReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            error: None,
        }
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }
    pub fn fail(&mut self, error: impl Into<String>) {
        if self.error.is_none() {
            self.error = Some(error.into());
        }
    }
    pub(crate) fn replace_error(&mut self, error: String) {
        self.error = Some(error);
    }
    fn varint(&mut self, signed: bool) -> u64 {
        if self.error.is_some() {
            return 0;
        }
        let mut value = 0;
        for (i, &byte) in self.bytes[self.offset..].iter().take(10).enumerate() {
            if i == 9 && byte > 1 {
                break;
            }
            value |= u64::from(byte & 127) << (7 * i);
            if byte < 128 {
                self.offset += i + 1;
                return value;
            }
        }
        self.fail(format!(
            "{}varint illisible a l offset {}",
            if signed { "" } else { "u" },
            self.offset
        ));
        0
    }
    pub fn unsigned(&mut self) -> u64 {
        self.varint(false)
    }
    pub fn signed(&mut self) -> i64 {
        let v = self.varint(true);
        ((v >> 1) as i64) ^ -((v & 1) as i64)
    }
    pub fn byte(&mut self) -> u8 {
        if self.error.is_some() {
            return 0;
        }
        if let Some(&b) = self.bytes.get(self.offset) {
            self.offset += 1;
            b
        } else {
            self.fail(format!("fin de flux prematuree a l offset {}", self.offset));
            0
        }
    }
    pub fn float32(&mut self) -> f32 {
        if self.error.is_some() {
            return 0.;
        }
        if self.remaining() < 4 {
            self.fail(format!("float32 tronque a l offset {}", self.offset));
            return 0.;
        }
        let bits = u32::from_le_bytes(self.bytes[self.offset..self.offset + 4].try_into().unwrap());
        self.offset += 4;
        f32::from_bits(bits)
    }
    pub fn boolean(&mut self) -> bool {
        self.byte() == 1
    }
    /// Borrow native string bytes without UTF-8 replacement. Negative/wrapping
    /// native lengths are refused safely instead of reproducing a slice panic.
    pub fn string_bytes(&mut self) -> &'a [u8] {
        let n = self.unsigned() as i64;
        if self.error.is_some() {
            return &[];
        }
        if n < 0 {
            self.fail(format!(
                "invalid signed string length {n} at offset {}",
                self.offset
            ));
            return &[];
        }
        if n as u64 > self.remaining() as u64 {
            self.fail(format!("chaine tronquee a l offset {}", self.offset));
            return &[];
        }
        let end = self.offset + n as usize;
        let bytes = &self.bytes[self.offset..end];
        self.offset = end;
        bytes
    }
    /// Return a borrowed section; None is distinct from a successful empty slice.
    pub fn section(&mut self, n: i64) -> Option<&'a [u8]> {
        if self.error.is_some() {
            return None;
        }
        if n < 0 || n as u64 > self.remaining() as u64 {
            self.fail(format!(
                "tranche de {n} octet(s) a l offset {} : {} disponible(s)",
                self.offset,
                self.remaining()
            ));
            return None;
        }
        let end = self.offset + n as usize;
        let out = &self.bytes[self.offset..end];
        self.offset = end;
        Some(out)
    }
    pub fn count(&mut self, minimum: i64) -> i64 {
        let n = self.unsigned() as i64;
        if self.error.is_some() {
            return 0;
        }
        let minimum = minimum.max(1);
        if n < 0 || n as u64 > self.remaining() as u64 / minimum as u64 {
            self.fail(format!("compte de {n} element(s) a l offset {} : {} octet(s) restants, {minimum} au minimum par element — flux desynchronise",self.offset,self.remaining()));
            return 0;
        }
        n
    }
}
