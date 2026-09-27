//! Native bitwise statborg frame scanner, including sparse and dense component lists.
use super::{StatborgRecord, StatborgValue, bits::Bits};
use std::collections::BTreeMap;
fn truncated(bits: Bits<'_>, at: usize, width: usize) -> u64 {
    bits.read(at.min(bits.len()), width.min(bits.len().saturating_sub(at)))
        .unwrap_or(0)
}
fn variable(bits: Bits<'_>, at: usize) -> Option<(i64, usize)> {
    let width = 8usize << bits.read(at, 2)?;
    if width > 32 {
        return None;
    }
    let raw = bits.read(at.checked_add(2)?, width)?;
    // The native 32-bit path deliberately remains unsigned in its int64 container.
    let value = if width < 32 && raw & (1 << (width - 1)) != 0 {
        raw as i64 - (1i64 << width)
    } else {
        raw as i64
    };
    Some((value, 2 + width))
}
fn component(bits: Bits<'_>, at: usize) -> Option<(StatborgValue, usize)> {
    let mut q = at.checked_add(10)?;
    let (a, n) = variable(bits, q)?;
    q += n;
    let (b, n) = variable(bits, q)?;
    q += n;
    let flags = bits.read(q, 2)?;
    q += 2;
    let mut value = StatborgValue {
        a,
        b,
        ..Default::default()
    };
    if flags & 2 != 0 {
        let (v, n) = variable(bits, q)?;
        q += n;
        value.c = v;
        value.has_c = true;
    }
    if flags & 1 != 0 {
        let (v, n) = variable(bits, q)?;
        q += n;
        value.d = v;
        value.has_d = true;
    }
    Some((value, q - at))
}
fn header(bits: Bits<'_>, b: usize) -> Option<(i64, Vec<i64>, usize)> {
    if b == 0 || truncated(bits, b - 1, 1) != 1 {
        return None;
    }
    let slot = truncated(bits, b, 14) as i64;
    if !(6..=24).contains(&slot) || slot % 2 != 0 || truncated(bits, b + 14, 2) != 2 {
        return None;
    }
    let m = b + 16;
    if truncated(bits, m, 1) != 0 {
        let mask = bits.read(m + 1, 64)?;
        if mask == 0 || mask >> 58 != 0 {
            return None;
        }
        return Some((
            slot,
            (0..58).filter(|i| mask >> i & 1 != 0).collect(),
            m + 65,
        ));
    }
    let n = truncated(bits, m + 1, 3) as usize;
    if n == 0 {
        return None;
    }
    let mut indices = Vec::with_capacity(n);
    let mut prev = -1;
    for i in 0..n {
        let index = truncated(bits, m + 4 + 6 * i, 6) as i64;
        if index >= 58 || index <= prev {
            return None;
        }
        prev = index;
        indices.push(index);
    }
    Some((slot, indices, m + 4 + 6 * n))
}
/// Scan every candidate bit, retaining successfully read component prefixes when
/// a later component is truncated. A/B domain failures reject the whole record;
/// conditional C/D values remain unbounded, matching the reference.
pub fn scan_statborg_frame(payload: &[u8], time_ms: i64) -> Vec<StatborgRecord> {
    let bits = Bits(payload);
    let mut out = Vec::new();
    for b in 1..bits.len().saturating_sub(64) {
        let Some((slot, indices, mut at)) = header(bits, b) else {
            continue;
        };
        let round = truncated(bits, at, 5);
        if round > 7 || round != truncated(bits, at + 5, 5) {
            continue;
        }
        let mut comps = BTreeMap::new();
        for index in indices {
            let Some((v, n)) = component(bits, at) else {
                break;
            };
            at += n;
            comps.insert(index, v);
        }
        if comps.is_empty()
            || comps
                .values()
                .any(|v| v.a.abs() > 1 << 20 || v.b.abs() > 1 << 20)
        {
            continue;
        }
        out.push(StatborgRecord {
            time_ms,
            slot,
            round: round as i64,
            comps,
        });
    }
    out
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmStatborgStream {
    pub records: Vec<StatborgRecord>,
    pub truncated: bool,
}
/// Date each nonzero-type manifest chunk from its first FRAME timestamp.
/// Stop after the frame that reaches the native 33,076-record cap.
pub fn scan_film_statborg(
    chunks: &[crate::clients::hi::models::FilmChunkData],
) -> FilmStatborgStream {
    scan_film_statborg_impl(chunks, None).0
}

/// Scan legacy chunk inputs while retaining the native missing-manifest INFO and
/// record-limit WARN. Each chunk carries its own metadata; no additional manifest
/// rows or absent loaded buffers can be represented by this input type.
pub fn scan_film_statborg_with_diagnostics(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    match_id: &str,
) -> (FilmStatborgStream, Vec<super::StatborgDiagnostic>) {
    scan_film_statborg_impl(chunks, Some(match_id))
}

fn scan_film_statborg_impl(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    match_id: Option<&str>,
) -> (FilmStatborgStream, Vec<super::StatborgDiagnostic>) {
    let mut scan = super::statborg_source::StatborgScan::new(match_id, chunks.len());
    for chunk in chunks.iter().filter(|c| c.metadata.chunk_type != 0) {
        let frames = super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .filter(|p| p.packet_type == 0)
            .map(|p| {
                (
                    p.timestamp_us,
                    &chunk.data[p.payload_offset..p.payload_offset + p.payload_size],
                )
            });
        if scan.chunk(
            i64::from(chunk.metadata.index),
            chunk.metadata.start_time_offset_ms,
            frames,
        ) {
            break;
        }
    }
    scan.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        payload: String,
        time_ms: i64,
        records: Vec<StatborgRecord>,
    }
    #[test]
    fn native_statborg_frame_scanning() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/statborg-decode-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let bytes: Vec<u8> = (0..c.payload.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&c.payload[j..j + 2], 16).unwrap())
                .collect();
            assert_eq!(
                scan_statborg_frame(&bytes, c.time_ms),
                c.records,
                "frame {i}"
            );
        }
    }
}
