//! Recorded instant, nanoseconds and numeric offset of a background sidecar time.
use chrono::{Datelike, Timelike};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapBackgroundTime {
    pub unix_seconds: i64,
    pub nanosecond: u32,
    pub offset_seconds: i32,
}
impl Default for MapBackgroundTime {
    fn default() -> Self {
        Self {
            unix_seconds: -62_135_596_800,
            nanosecond: 0,
            offset_seconds: 0,
        }
    }
}
impl MapBackgroundTime {
    /// Go's pinned RFC3339 parser also accepts one-digit hours, comma fractions,
    /// excess fractional digits (truncated), and offsets through 24:60.
    pub fn parse(text: &str) -> Result<Self, String> {
        let b = text.as_bytes();
        let err = || "invalid background RFC3339 timestamp".to_owned();
        let num = |s: &[u8]| -> Option<u32> {
            if s.is_empty() || !s.iter().all(u8::is_ascii_digit) {
                return None;
            }
            Some(s.iter().fold(0, |v, c| v * 10 + u32::from(c - b'0')))
        };
        if b.len() < 19 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' {
            return Err(err());
        }
        let year = num(&b[..4]).ok_or_else(err)? as i32;
        let month = num(&b[5..7]).ok_or_else(err)?;
        let day = num(&b[8..10]).ok_or_else(err)?;
        let mut p = 11;
        let start = p;
        while p < b.len() && b[p].is_ascii_digit() {
            p += 1;
        }
        if !(1..=2).contains(&(p - start)) || b.get(p) != Some(&b':') {
            return Err(err());
        }
        let hour = num(&b[start..p]).ok_or_else(err)?;
        p += 1;
        if p + 5 >= b.len() || b[p + 2] != b':' {
            return Err(err());
        }
        let minute = num(&b[p..p + 2]).ok_or_else(err)?;
        let second = num(&b[p + 3..p + 5]).ok_or_else(err)?;
        p += 5;
        let mut nanosecond = 0;
        let mut count: usize = 0;
        if matches!(b.get(p), Some(b'.' | b',')) {
            p += 1;
            while p < b.len() && b[p].is_ascii_digit() {
                if count < 9 {
                    nanosecond = nanosecond * 10 + u32::from(b[p] - b'0');
                }
                count += 1;
                p += 1;
            }
            if count == 0 {
                return Err(err());
            }
            for _ in count..9 {
                nanosecond *= 10;
            }
        }
        let zone = &b[p..];
        let offset_seconds = if zone == b"Z" {
            0
        } else {
            if zone.len() != 6 || !matches!(zone[0], b'+' | b'-') || zone[3] != b':' {
                return Err(err());
            }
            let h = num(&zone[1..3]).ok_or_else(err)?;
            let m = num(&zone[4..6]).ok_or_else(err)?;
            if h > 24 || m > 60 {
                return Err(err());
            }
            let offset = (h * 3600 + m * 60) as i32;
            if zone[0] == b'-' { -offset } else { offset }
        };
        let date = chrono::NaiveDate::from_ymd_opt(year, month, day).ok_or_else(err)?;
        // Reject leap-second representations accepted by some date libraries.
        if hour > 23 || minute > 59 || second > 59 {
            return Err(err());
        }
        let wall = date
            .and_hms_nano_opt(hour, minute, second, nanosecond)
            .ok_or_else(err)?;
        Ok(Self {
            unix_seconds: wall.and_utc().timestamp() - i64::from(offset_seconds),
            nanosecond,
            offset_seconds,
        })
    }
    pub(super) fn storage_value(self) -> serde_json::Value {
        serde_json::json!({"unix_seconds":self.unix_seconds,"nanosecond":self.nanosecond,"offset_seconds":self.offset_seconds})
    }
    pub fn to_rfc3339(self) -> Result<String, String> {
        let invalid = || "background timestamp cannot be published as RFC3339".to_owned();
        if self.nanosecond >= 1_000_000_000 || self.offset_seconds.unsigned_abs() / 3600 >= 24 {
            return Err(invalid());
        }
        let wall = self
            .unix_seconds
            .checked_add(i64::from(self.offset_seconds))
            .and_then(|s| chrono::DateTime::from_timestamp(s, self.nanosecond))
            .ok_or_else(invalid)?;
        if !(0..=9999).contains(&wall.year()) {
            return Err(invalid());
        }
        let mut out = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            wall.year(),
            wall.month(),
            wall.day(),
            wall.hour(),
            wall.minute(),
            wall.second()
        );
        if self.nanosecond != 0 {
            let fraction = format!("{:09}", self.nanosecond);
            out.push('.');
            out.push_str(fraction.trim_end_matches('0'));
        }
        if self.offset_seconds == 0 {
            out.push('Z');
        } else {
            let offset = self.offset_seconds.unsigned_abs();
            out.push_str(&format!(
                "{}{:02}:{:02}",
                if self.offset_seconds < 0 { '-' } else { '+' },
                offset / 3600,
                (offset / 60) % 60
            ));
        }
        Ok(out)
    }
}
impl Serialize for MapBackgroundTime {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_rfc3339().map_err(serde::ser::Error::custom)?)
    }
}
impl<'de> Deserialize<'de> for MapBackgroundTime {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Text(String),
            Fields {
                unix_seconds: i64,
                nanosecond: u32,
                offset_seconds: i32,
            },
        }
        match Repr::deserialize(d)? {
            Repr::Text(s) => Self::parse(&s).map_err(serde::de::Error::custom),
            Repr::Fields {
                unix_seconds,
                nanosecond,
                offset_seconds,
            } => {
                if nanosecond >= 1_000_000_000 {
                    return Err(serde::de::Error::custom("invalid nanoseconds"));
                }
                Ok(Self {
                    unix_seconds,
                    nanosecond,
                    offset_seconds,
                })
            }
        }
    }
}
