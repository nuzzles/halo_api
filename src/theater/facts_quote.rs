//! Go %q diagnostics for raw cache strings. Printable classification is pinned
//! to the reference runtime rather than the host Rust Unicode version.
use super::facts_quote_tables::*;
use std::fmt::Write;

fn printable(c: u32) -> bool {
    if c <= 255 {
        return (32..=126).contains(&c) || ((161..=255).contains(&c) && c != 173);
    }
    let (ranges, exceptions) = if c < 65536 {
        (PRINT16, NOT_PRINT16)
    } else {
        (PRINT32, NOT_PRINT32)
    };
    let i = ranges.partition_point(|&n| n < c);
    i < ranges.len()
        && ranges[i & !1] <= c
        && c <= ranges[i | 1]
        && (c >= 131072 || exceptions.binary_search(&(c & 65535)).is_err())
}

pub(super) fn quote(bytes: &[u8]) -> String {
    let mut out = String::from("\"");
    let mut at = 0;
    while at < bytes.len() {
        // Decode one rune. Go escapes each invalid UTF-8 byte individually.
        let width = match bytes[at] {
            0..=127 => 1,
            194..=223 => 2,
            224..=239 => 3,
            240..=244 => 4,
            _ => 0,
        };
        let ch = (width > 0)
            .then(|| bytes.get(at..at + width))
            .flatten()
            .and_then(|b| std::str::from_utf8(b).ok())
            .and_then(|s| s.chars().next());
        let Some(c) = ch else {
            write!(out, "\\x{:02x}", bytes[at]).unwrap();
            at += 1;
            continue;
        };
        at += width;
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            c if printable(c as u32) => out.push(c),
            '\u{7}' => out.push_str("\\a"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{b}' => out.push_str("\\v"),
            c if (c as u32) < 32 || c == '\u{7f}' => write!(out, "\\x{:02x}", c as u32).unwrap(),
            c if (c as u32) < 65536 => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => write!(out, "\\U{:08x}", c as u32).unwrap(),
        }
    }
    out.push('"');
    out
}
